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
) {
    let hosts = enumerate_hosts(iface.ip, iface.prefix_len);
    let total = u32::try_from(hosts.len()).unwrap_or(u32::MAX);
    let concurrency = concurrency.max(1);
    let sem = Arc::new(Semaphore::new(concurrency));
    let ports: Vec<u16> = PROBE_PORTS.to_vec();
    let swept = Arc::new(AtomicU32::new(0));

    let mut handles = Vec::new();
    for host in hosts {
        if cancel.is_cancelled() {
            break;
        }
        let sem = sem.clone();
        let ports = ports.clone();
        let tx = tx.clone();
        let cancel = cancel.clone();
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
    for handle in handles {
        let _ = handle.await;
    }
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
            let addr = SocketAddr::new(IpAddr::V4(host), *port);
            let connect = TcpStream::connect(addr);
            if tokio::time::timeout(per_port_timeout, connect)
                .await
                .is_ok_and(|r| r.is_ok())
            {
                let _ = tx.send(DiscoveryEvent::Host(
                    Observation::new(Source::TcpPing)
                        .with_ip(IpAddr::V4(host))
                        .with_hint("tcp.ports", port.to_string()),
                ));
                break;
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
}
