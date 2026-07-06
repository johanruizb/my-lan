//! `mylan ping|traceroute|dns` — herramientas de diagnóstico de red (Fase 3, Paso 5).
//!
//! Delega en los impls reales de `mylan-discovery` (`ping_host`, `traceroute_host`,
//! `dns_lookup_host`, `resolve_host`). El dispatch vive en `apps/cli/src/main.rs`.

use std::net::IpAddr;
use std::time::Duration;

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use mylan_discovery::{dns, ping, traceroute_host};

use crate::ctx::AppContext;

/// `mylan ping <ip> [--count N] [--timeout-ms MS] [--ipv4] [--ipv6]`.
///
/// Muestra estadísticas de latencia/packet loss. Default count 4, timeout 1000 ms.
/// ICMP no-root con degradación a TCP connect (`PingMethod` distinguible).
pub async fn run_ping(
    ctx: &AppContext,
    ip: &str,
    count: Option<u32>,
    timeout_ms: Option<u64>,
    ipv4: bool,
    ipv6: bool,
) -> anyhow::Result<()> {
    let _ = ctx;
    let count = count.unwrap_or(4);
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(1000));

    let target = resolve_target(ip, ipv4, ipv6).await?;

    tracing::info!(target = %target, count, ?timeout, "iniciando ping");
    let r = ping::ping_host(target, count, timeout)
        .await
        .map_err(|e| anyhow::anyhow!("ping: {e}"))?;

    let method = format!("{:?}", r.method).to_lowercase();
    let loss = match r.packet_loss {
        Some(p) => format!("{:.0}%", p * 100.0),
        None => "-".to_string(),
    };
    let latency = match r.latency_ms {
        Some(ms) => ms.to_string(),
        None => "-".to_string(),
    };

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Target").fg(Color::Cyan),
            Cell::new("Sent").fg(Color::Cyan),
            Cell::new("Received").fg(Color::Cyan),
            Cell::new("Loss").fg(Color::Cyan),
            Cell::new("Latency(ms)").fg(Color::Cyan),
            Cell::new("Method").fg(Color::Cyan),
        ])
        .add_row(vec![
            Cell::new(r.target.to_string()),
            Cell::new(r.packets_sent),
            Cell::new(r.packets_received),
            Cell::new(loss),
            Cell::new(latency),
            Cell::new(method),
        ]);
    println!("Ping a {target}:");
    println!("{table}");
    println!(
        "Estado: {}",
        if r.reachable {
            "reachable"
        } else {
            "unreachable"
        }
    );
    Ok(())
}

/// `mylan traceroute <ip> [--max-hops N] [--timeout-ms MS]`.
///
/// Muestra saltos con IP, hostname (reverse DNS) y latencia. Default max-hops 30.
/// UDP incrementales con TTL + error-queue ICMP vía `nix`.
pub async fn run_traceroute(
    ctx: &AppContext,
    ip: &str,
    max_hops: Option<u8>,
    timeout_ms: Option<u64>,
) -> anyhow::Result<()> {
    let _ = ctx;
    let max_hops = max_hops.unwrap_or(30);
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(1000));

    let target = resolve_target(ip, false, false).await?;

    tracing::info!(target = %target, max_hops, ?timeout, "iniciando traceroute");
    let hops = traceroute_host(target, max_hops, timeout)
        .await
        .map_err(|e| anyhow::anyhow!("traceroute: {e}"))?;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Hop").fg(Color::Cyan),
            Cell::new("IP").fg(Color::Cyan),
            Cell::new("Hostname").fg(Color::Cyan),
            Cell::new("Latency(ms)").fg(Color::Cyan),
            Cell::new("State").fg(Color::Cyan),
        ]);
    for h in &hops {
        table.add_row(vec![
            Cell::new(h.hop_number),
            Cell::new(
                h.ip.map(|i| i.to_string())
                    .unwrap_or_else(|| "*".to_string()),
            ),
            Cell::new(h.hostname.clone().unwrap_or_default()),
            Cell::new(
                h.latency_ms
                    .map(|ms| ms.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::new(h.state.clone()),
        ]);
    }
    println!("Traceroute a {target}:");
    println!("{table}");
    Ok(())
}

/// `mylan dns <host> [--type A|AAAA|PTR|MX|TXT] [--ipv4] [--ipv6]`.
///
/// Resuelve registros vía `hickory-resolver` (helper `system_resolver()`).
pub async fn run_dns(
    ctx: &AppContext,
    host: &str,
    rtype: Option<&str>,
    _ipv4: bool,
    _ipv6: bool,
) -> anyhow::Result<()> {
    let _ = ctx;

    tracing::info!(host, rtype = ?rtype, "iniciando dns lookup");
    let records = dns::dns_lookup_host(host, rtype.unwrap_or("").to_string())
        .await
        .map_err(|e| anyhow::anyhow!("dns: {e}"))?;

    if records.is_empty() {
        println!("No se encontraron registros DNS para {host}.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("Type").fg(Color::Cyan),
            Cell::new("Value").fg(Color::Cyan),
            Cell::new("TTL").fg(Color::Cyan),
        ]);
    for r in &records {
        table.add_row(vec![
            Cell::new(r.name.clone()),
            Cell::new(r.record_type.clone()),
            Cell::new(r.value.clone()),
            Cell::new(r.ttl),
        ]);
    }
    println!("DNS {host}:");
    println!("{table}");
    Ok(())
}

/// Resuelve un target (IP literal o hostname) a una única `IpAddr`.
///
/// Si `ip` parsea como `IpAddr` se usa directamente. Si es un hostname, se
/// resuelve vía `resolve_host` aplicando los filtros de familia y se toma la
/// primera dirección. Sin resultado → error claro (no falso positivo).
async fn resolve_target(name: &str, ipv4: bool, ipv6: bool) -> anyhow::Result<IpAddr> {
    if let Ok(addr) = name.parse::<IpAddr>() {
        return Ok(addr);
    }
    let ips = dns::resolve_host(name, ipv4, ipv6)
        .await
        .map_err(|e| anyhow::anyhow!("resolver {name}: {e}"))?;
    ips.into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no se pudo resolver ninguna IP para '{name}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Nota de determinismo: los caminos `run_ping`/`run_traceroute`/`run_dns`
    // y la resolución DNS de `resolve_target` para hostnames requieren red
    // (ICMP/UDP/DNS) y no se testean aquí. Solo el camino de IP literal (sin
    // I/O de red) es determinista y se cubre abajo. Ver AC-4 del plan.

    #[tokio::test]
    async fn resolve_target_ipv4_literal_skips_dns() {
        // Una IP literal parsea directamente; no hay llamada DNS.
        let addr = resolve_target("192.168.1.1", false, false)
            .await
            .expect("ipv4 literal");
        assert_eq!(addr.to_string(), "192.168.1.1");
    }

    #[tokio::test]
    async fn resolve_target_ipv6_literal_skips_dns() {
        let addr = resolve_target("::1", false, false)
            .await
            .expect("ipv6 literal");
        assert_eq!(addr.to_string(), "::1");
    }

    #[tokio::test]
    async fn resolve_target_loopback_literal() {
        let addr = resolve_target("127.0.0.1", false, false)
            .await
            .expect("loopback literal");
        assert_eq!(addr.to_string(), "127.0.0.1");
    }

    #[tokio::test]
    async fn resolve_target_ignores_family_filters_for_literal() {
        // Los flags ipv4/ipv6 no aplican a literales: la IP se usa tal cual.
        let addr = resolve_target("10.0.0.1", true, false)
            .await
            .expect("literal con ipv4=true");
        assert_eq!(addr.to_string(), "10.0.0.1");
    }
}
