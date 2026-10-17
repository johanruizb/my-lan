//! Sondeo continuo de salud de red y latencia local (Pilar 5 / Fase 9).
//!
//! Mide RTT, jitter inter-llegada (RFC 3550) y pérdida de paquetes hacia el gateway local
//! y servidores DNS sin requerir privilegios de Administrador / root.

use std::net::IpAddr;
use std::time::Duration;

use mylan_core::NetworkHealthSample;

use crate::ping::ping_host;

/// Ejecuta una ráfaga de sondas hacia una IP destino y consolida la métrica de salud.
pub async fn probe_target_health(
    network_id: &str,
    target_type: &str,
    target_ip: IpAddr,
    sample_count: u32,
    timeout: Duration,
) -> NetworkHealthSample {
    let count = sample_count.max(1);
    let mut latencies: Vec<u64> = Vec::new();
    let mut failed = 0u32;
    let mut jitter = 0.0f64;
    let mut prev_rtt: Option<f64> = None;
    let mut last_method = "icmp".to_string();

    for _ in 0..count {
        match ping_host(target_ip, 1, timeout).await {
            Ok(res) => {
                last_method = match res.method {
                    mylan_core::PingMethod::Icmp => "icmp".to_string(),
                    mylan_core::PingMethod::TcpConnect => "tcp_connect".to_string(),
                };

                if res.reachable {
                    if let Some(rtt) = res.latency_ms {
                        latencies.push(rtt);

                        let rtt_f = rtt as f64;
                        if let Some(prev) = prev_rtt {
                            let d = (rtt_f - prev).abs();
                            jitter += (d - jitter) / 16.0;
                        }
                        prev_rtt = Some(rtt_f);
                    } else {
                        failed += 1;
                    }
                } else {
                    failed += 1;
                }
            }
            Err(_) => {
                failed += 1;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let avg_rtt = if latencies.is_empty() {
        None
    } else {
        Some(latencies.iter().sum::<u64>() / latencies.len() as u64)
    };

    let loss_pct = ((failed as f32) / (count as f32)) * 100.0;

    let now_str = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "2026-10-16T12:00:00Z".to_string());

    NetworkHealthSample {
        id: uuid::Uuid::new_v4().to_string(),
        network_id: network_id.to_string(),
        target_type: target_type.to_string(),
        target_ip: target_ip.to_string(),
        rtt_ms: avg_rtt,
        jitter_ms: (jitter * 100.0).round() / 100.0,
        loss_pct,
        sample_count: count,
        probes_failed: failed,
        method: last_method,
        created_at: now_str,
    }
}

/// Sondea la salud global de la red: gateway y DNS principal.
pub async fn probe_network_health(
    network_id: &str,
    gateway_ip: Option<IpAddr>,
    dns_servers: &[IpAddr],
) -> Vec<NetworkHealthSample> {
    let mut samples = Vec::new();

    // 1. Gateway (ignora si es loopback)
    if let Some(gw) = gateway_ip {
        if !gw.is_loopback() {
            let sample =
                probe_target_health(network_id, "gateway", gw, 4, Duration::from_millis(800)).await;
            samples.push(sample);
        }
    }

    // 2. DNS / Enlace de salida
    // En Linux, `/etc/resolv.conf` suele apuntar a stubs locales en loopback (ej. 127.0.0.53 en
    // systemd-resolved o 127.0.0.1 en dnsmasq). Medir RTT/jitter contra loopback arroja 0 ms
    // y pérdida 0% engañosos, sin reflejar el enlace de salida real.
    let non_loopback_dns = dns_servers.iter().copied().find(|ip| !ip.is_loopback());

    let effective_dns = non_loopback_dns.or_else(|| {
        crate::dns::discover_upstream_dns()
            .into_iter()
            .find(|ip| !ip.is_loopback())
    });

    let (dns_target, target_type) = match effective_dns {
        Some(ip) => (ip, "dns"),
        None => ("1.1.1.1".parse().unwrap(), "internet"),
    };

    let dns_sample = probe_target_health(
        network_id,
        target_type,
        dns_target,
        4,
        Duration::from_millis(1000),
    )
    .await;
    samples.push(dns_sample);

    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_probe_target_health_loopback() {
        let sample = probe_target_health(
            "net-test",
            "gateway",
            "127.0.0.1".parse().unwrap(),
            2,
            Duration::from_millis(500),
        )
        .await;

        assert_eq!(sample.network_id, "net-test");
        assert_eq!(sample.target_ip, "127.0.0.1");
        assert_eq!(sample.sample_count, 2);
    }

    #[tokio::test]
    async fn test_probe_network_health_ignores_loopback_dns() {
        let loopback_dns: Vec<IpAddr> =
            vec!["127.0.0.53".parse().unwrap(), "127.0.0.1".parse().unwrap()];

        let samples = probe_network_health(
            "net-test",
            Some("127.0.0.1".parse().unwrap()),
            &loopback_dns,
        )
        .await;

        // El gateway loopback debe haberse descartado
        assert!(samples.iter().all(|s| s.target_type != "gateway"));

        // El sample de DNS no debe apuntar a ninguna IP de loopback
        let dns_sample = samples
            .iter()
            .find(|s| s.target_type == "dns" || s.target_type == "internet")
            .expect("debe existir muestra de DNS o internet");

        let target_ip: IpAddr = dns_sample.target_ip.parse().expect("IP válida");
        assert!(
            !target_ip.is_loopback(),
            "La IP de destino de salud de DNS nunca debe ser loopback: {}",
            dns_sample.target_ip
        );
    }

    #[tokio::test]
    async fn test_probe_network_health_prefers_non_loopback_dns() {
        let mixed_dns: Vec<IpAddr> =
            vec!["127.0.0.53".parse().unwrap(), "1.1.1.1".parse().unwrap()];

        let samples = probe_network_health("net-test", None, &mixed_dns).await;

        let dns_sample = samples
            .iter()
            .find(|s| s.target_type == "dns")
            .expect("debe existir muestra dns");

        assert_eq!(dns_sample.target_ip, "1.1.1.1");
    }
}
