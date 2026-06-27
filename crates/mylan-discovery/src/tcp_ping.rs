//! Barrido TCP-connect concurrente sobre toda la subred (fase liveness sin root).
//!
//! Para cada host de la subred intenta conectar a una lista de puertos sonda; al
//! primer puerto que acepta la conexión se considera vivo y se emite una
//! [`Observation`] de origen [`Source::TcpPing`] con un hint `tcp.ports`. La
//! concurrencia se acota con un [`tokio::sync::Semaphore`]; cada intento tiene timeout
//! agresivo para mantener el escaneo quick dentro del presupuesto AC-12 (<30 s).

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use mylan_core::{Observation, Source};

use crate::iface::LanInterface;
use crate::netutil::enumerate_hosts;

/// Puertos sonda por defecto (servicios comunes en LAN).
pub const PROBE_PORTS: &[u16] = &[80, 443, 22, 445, 53, 8080];

/// Ejecuta el barrido TCP-connect sobre todos los hosts de la subred de `iface`.
///
/// Devuelve una [`Observation`] por host vivo. `concurrency` limita el número de
/// hosts sondeados en paralelo. Un `per_port_timeout` muy corto mantiene el barrido
/// veloz; la función termina cuando todos los hosts se han probado.
pub async fn tcp_sweep(
    iface: &LanInterface,
    per_port_timeout: Duration,
    concurrency: usize,
) -> Vec<Observation> {
    let hosts = enumerate_hosts(iface.ip, iface.prefix_len);
    let concurrency = concurrency.max(1);
    let sem = std::sync::Arc::new(Semaphore::new(concurrency));
    let ports: Vec<u16> = PROBE_PORTS.to_vec();

    let handles: Vec<JoinHandle<Option<Observation>>> = hosts
        .into_iter()
        .map(|host| {
            let sem = sem.clone();
            let ports = ports.clone();
            tokio::spawn(probe_host(host, ports, per_port_timeout, sem))
        })
        .collect();

    let mut alive = Vec::new();
    for handle in handles {
        if let Ok(Some(obs)) = handle.await {
            alive.push(obs);
        }
    }
    alive
}

async fn probe_host(
    host: std::net::Ipv4Addr,
    ports: Vec<u16>,
    per_port_timeout: Duration,
    sem: std::sync::Arc<Semaphore>,
) -> Option<Observation> {
    let _permit = sem.acquire().await.ok()?;
    for port in ports {
        let addr = SocketAddr::new(IpAddr::V4(host), port);
        let connect = TcpStream::connect(addr);
        if tokio::time::timeout(per_port_timeout, connect)
            .await
            .is_ok_and(|r| r.is_ok())
        {
            return Some(
                Observation::new(Source::TcpPing)
                    .with_ip(IpAddr::V4(host))
                    .with_hint("tcp.ports", port.to_string()),
            );
        }
    }
    None
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
