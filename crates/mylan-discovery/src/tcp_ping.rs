//! Barrido TCP-connect concurrente sobre toda la subred (fase liveness sin root).
//!
//! Para cada host de la subred intenta conectar a una lista de puertos sonda; al
//! primer puerto que acepta la conexión se considera vivo y se emite una
//! [`Observation`] de origen [`Source::TcpPing`] con un hint `tcp.ports`. La
//! concurrencia se acota con un [`tokio::sync::Semaphore`]; cada intento tiene timeout
//! agresivo para mantener el escaneo quick dentro del presupuesto AC-12 (<30 s).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use mylan_core::{Observation, Source};

use crate::iface::LanInterface;
use crate::netutil::enumerate_hosts;
use crate::DiscoveryEvent;

/// Puertos sonda por defecto (servicios comunes en LAN).
pub const PROBE_PORTS: &[u16] = &[80, 443, 22, 445, 53, 8080];

/// Ejecuta el barrido TCP-connect sobre todos los hosts de la subred de `iface`.
///
/// Transmite una [`DiscoveryEvent::Host`] por host vivo y, tras cada host sondeado,
/// un [`DiscoveryEvent::Progress`] *throttled* (cada ~8 IPs) por `tx` — es la única
/// fuente de avance del descubrimiento. `concurrency` limita el número de hosts
/// sondeados en paralelo. Respeta `cancel` de forma cooperativa por iteración.
pub async fn tcp_sweep(
    iface: &LanInterface,
    per_port_timeout: Duration,
    concurrency: usize,
    tx: UnboundedSender<DiscoveryEvent>,
    cancel: CancellationToken,
    sweep_budget: Duration,
) {
    let hosts = enumerate_hosts(iface.ip, iface.prefix_len);
    let total = u32::try_from(hosts.len()).unwrap_or(u32::MAX);
    let concurrency = concurrency.max(1);
    let sem = Arc::new(Semaphore::new(concurrency));
    let ports: Vec<u16> = PROBE_PORTS.to_vec();
    let swept = Arc::new(AtomicU32::new(0));

    // Token local hijo del compartido: el watchdog del budget cancela solo el
    // barrido TCP, sin afectar mDNS/SSDP/ICMP (que usan el token padre). La
    // cancelación padre→hijo sigue propagando (graceful shutdown).
    let local = cancel.child_token();

    let mut handles = Vec::new();
    for host in hosts {
        if local.is_cancelled() {
            break;
        }
        let sem = sem.clone();
        let ports = ports.clone();
        let tx = tx.clone();
        let cancel = local.clone();
        let swept = swept.clone();
        handles.push(tokio::spawn(probe_host(
            host,
            ports,
            per_port_timeout,
            sem,
            tx,
            cancel,
            swept,
            total,
        )));
    }

    // Watchdog: cancela el barrido tras `sweep_budget`. Las tareas en vuelo
    // rompen su `select!` cancel-aware de inmediato (no esperan per_port_timeout).
    let wd_token = local.clone();
    let watchdog = tokio::spawn(async move {
        tokio::time::sleep(sweep_budget).await;
        wd_token.cancel();
    });

    for handle in handles {
        let _ = handle.await;
    }
    watchdog.abort();
}

#[allow(clippy::too_many_arguments)]
async fn probe_host(
    host: Ipv4Addr,
    ports: Vec<u16>,
    per_port_timeout: Duration,
    sem: Arc<Semaphore>,
    tx: UnboundedSender<DiscoveryEvent>,
    cancel: CancellationToken,
    swept: Arc<AtomicU32>,
    total: u32,
) {
    let Ok(_permit) = sem.acquire().await else {
        return;
    };
    if !cancel.is_cancelled() {
        for port in &ports {
            if cancel.is_cancelled() {
                break;
            }
            let addr = SocketAddr::new(IpAddr::V4(host), *port);
            let connect = TcpStream::connect(addr);
            tokio::select! {
                res = tokio::time::timeout(per_port_timeout, connect) => {
                    if res.is_ok_and(|r| r.is_ok()) {
                        let _ = tx.send(DiscoveryEvent::Host(
                            Observation::new(Source::TcpPing)
                                .with_ip(IpAddr::V4(host))
                                .with_hint("tcp.ports", port.to_string()),
                        ));
                        break;
                    }
                }
                () = cancel.cancelled() => break,
            }
        }
    }
    // Cuenta este host como sondeado y emite avance throttled (cada ~8 IPs o el
    // último), para no inundar el canal en subredes anchas.
    let n = swept.fetch_add(1, Ordering::Relaxed) + 1;
    if n.is_multiple_of(8) || n == total {
        let _ = tx.send(DiscoveryEvent::Progress { swept: n, total });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_ports_are_lan_relevant() {
        assert!(PROBE_PORTS.contains(&80));
        assert!(PROBE_PORTS.contains(&443));
        assert!(PROBE_PORTS.contains(&22));
        assert!(PROBE_PORTS.contains(&53));
        assert!(!PROBE_PORTS.is_empty());
    }

    /// AC-7: `tcp_sweep` respeta `sweep_budget` sobre una subnet ancha (/16 →
    /// 4096 hosts capped). La cancelación cooperativa (child token + `select!`
    /// en `probe_host`) hace que el barrido termine en ~budget, no en
    /// 4096 × 2.4 s. IP 240.0.0.0/4 (reservada RFC 1112) evita hit servicios
    /// locales del runner (no loopback); en runners con default route los SYNs a
    /// 240/4 se descartan en la gateway — es el budget, no la IP, quien
    /// garantiza el determinismo (la aserción de tiempo no depende de
    /// respuestas de red, transit-safe per AGENTS.md).
    #[tokio::test]
    async fn tcp_sweep_respects_budget_on_wide_subnet() {
        use tokio_util::sync::CancellationToken;
        let iface = LanInterface {
            // 240.0.0.0/4 reservado RFC 1112: evita servicios locales del runner.
            // El budget (no la IP) garantiza el determinismo via cancel cooperativa.
            name: "lo".into(),
            ip: "240.0.0.1".parse().unwrap(),
            prefix_len: 16,
            mac: None,
            gateway_ip: None,
            gateway_mac: None,
            dns_servers: Vec::new(),
            ssid: None,
        };
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let cancel = CancellationToken::new();
        let start = std::time::Instant::now();
        let res = tokio::time::timeout(
            Duration::from_secs(2),
            tcp_sweep(
                &iface,
                Duration::from_millis(400),
                256,
                tx,
                cancel,
                Duration::from_millis(200),
            ),
        )
        .await;
        assert!(
            res.is_ok(),
            "tcp_sweep debe respetar sweep_budget (200ms) y no colgar 4096 hosts"
        );
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(1),
            "elapsed {elapsed:?} excede margen; budget no respetado"
        );
    }

    /// AC-8: cancelar el child token (watchdog del budget) NO cancela el parent
    /// (mDNS/SSDP/ICMP quedan intactos). Invariante de aislamiento del budget.
    #[test]
    fn child_token_cancel_is_isolated() {
        use tokio_util::sync::CancellationToken;
        let parent = CancellationToken::new();
        let child = parent.child_token();
        child.cancel();
        assert!(
            !parent.is_cancelled(),
            "cancelar child (watchdog budget) no debe cancelar parent (mDNS/SSDP/ICMP)"
        );
        assert!(child.is_cancelled());
    }
}
