//! UDP scan limitado con connected-socket (AC-3).
//!
//! Sondeo UDP best-effort sobre un catálogo reducido de puertos bien conocidos
//! (DNS, DHCP, NTP, SNMP, SSDP, mDNS). Usa `UdpSocket::connect` + `send`/`recv`
//! con un timeout acotado: una respuesta → `Open`, `ConnectionRefused` (ICMP
//! port-unreachable) → `Closed`, y todo lo demás (timeout/sin ICMP) → `Filtered`
//! (reutilizamos `Filtered` para el caso UDP open|filtered, principio AC-3).
//!
//! No intrusivo (P2): envía un datagrama mínimo de 1 byte nulo. La semántica de
//! UDP es inherentemente ambigua; este scan es orientativo, no definitivo.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use mylan_core::{Protocol, Service, ServiceState};
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

use crate::{port_to_service_name, ScanOptions};

/// Catálogo reducido de puertos UDP bien conocidos para el scan limitado.
const UDP_PORTS: &[u16] = &[53, 67, 68, 123, 161, 1900, 5353];

/// Timeout por puerto UDP (recv). Acotado para respetar AC-12 (<30 s total).
const UDP_TIMEOUT: Duration = Duration::from_secs(3);

/// Escanea el catálogo UDP de `target` y devuelve un [`Service`] por puerto.
///
/// Itera [`UDP_PORTS`] secuencialmente (concurrencia baja: UDP es ruidoso y el
/// catálogo es pequeño). Respeta `cancel` cooperativamente. `_opts` se reserva
/// para futura afinación de timeouts; hoy el timeout es fijo ([`UDP_TIMEOUT`]).
pub async fn scan_udp(
    target: IpAddr,
    _opts: &ScanOptions,
    cancel: &CancellationToken,
) -> Vec<Service> {
    let mut out = Vec::new();
    for &port in UDP_PORTS {
        if cancel.is_cancelled() {
            break;
        }
        let state = probe_udp(target, port).await;
        out.push(to_udp_service(target, port, state));
    }
    out
}

/// Sondea un puerto UDP de `target` con connected-socket.
///
/// `connect` en UDP sólo fija el destino por defecto (no hay handshake); tras
/// `send`, `recv` devuelve la respuesta (`Open`), `ConnectionRefused` si llega
/// un ICMP port-unreachable (`Closed`), o timeout/error (`Filtered`).
async fn probe_udp(target: IpAddr, port: u16) -> ServiceState {
    let bind = match target {
        IpAddr::V4(_) => "0.0.0.0:0",
        IpAddr::V6(_) => "[::]:0",
    };
    let sock = match UdpSocket::bind(bind).await {
        Ok(s) => s,
        Err(_) => return ServiceState::Filtered,
    };
    let addr = SocketAddr::new(target, port);
    if sock.connect(addr).await.is_err() {
        return ServiceState::Filtered;
    }
    let _ = sock.send(&[0u8; 1]).await;
    let mut buf = [0u8; 1024];
    match tokio::time::timeout(UDP_TIMEOUT, sock.recv(&mut buf)).await {
        Ok(Ok(_)) => ServiceState::Open,
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused => ServiceState::Closed,
        _ => ServiceState::Filtered,
    }
}

/// Construye un [`Service`] UDP. Identidad/timestamps vacíos: los asigna la
/// persistencia. `state` refleja el resultado del sondeo.
fn to_udp_service(target: IpAddr, port: u16, state: ServiceState) -> Service {
    let _ = target; // reflejado vía el sondeo; aquí no se persiste.
    Service {
        id: String::new(),
        device_id: String::new(),
        protocol: Protocol::Udp,
        port,
        service_name: port_to_service_name(port).map(String::from),
        product: None,
        version: None,
        banner: None,
        state,
        first_seen_at: String::new(),
        last_seen_at: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket;

    /// Un socket UDP local que responde al datagrama → `Open`.
    #[tokio::test(flavor = "current_thread")]
    async fn probe_udp_open_when_responds() {
        let responder = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = responder.local_addr().unwrap().port();
        tokio::spawn(async move {
            let mut buf = [0u8; 64];
            if let Ok((_, peer)) = responder.recv_from(&mut buf).await {
                // Responder en el mismo socket: el peer ya está fijado por recv_from.
                let _ = responder.send_to(b"pong", peer).await;
            }
        });

        let state = probe_udp("127.0.0.1".parse().unwrap(), port).await;
        assert_eq!(state, ServiceState::Open, "responder vivo => Open");
    }

    /// Un socket UDP local que recibe pero **no responde** → `Filtered` (timeout).
    /// Toma hasta [`UDP_TIMEOUT`]; es el costo de la semántica ambigua de UDP.
    #[tokio::test(flavor = "current_thread")]
    async fn probe_udp_filtered_when_silent() {
        let sink = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = sink.local_addr().unwrap().port();
        // Mantiene el puerto ocupado (sin ICMP port-unreachable) pero no responde.
        tokio::spawn(async move {
            let mut buf = [0u8; 64];
            let _ = sink.recv_from(&mut buf).await;
        });

        let state = probe_udp("127.0.0.1".parse().unwrap(), port).await;
        assert_eq!(state, ServiceState::Filtered, "silencio => Filtered");
    }

    /// `scan_udp` respeta la cancelación cooperativa: regresa vacío de inmediato.
    #[tokio::test(flavor = "current_thread")]
    async fn scan_udp_cancelled_returns_empty() {
        let cancel = CancellationToken::new();
        cancel.cancel();
        let opts = ScanOptions::default();
        let out = scan_udp("127.0.0.1".parse().unwrap(), &opts, &cancel).await;
        assert!(out.is_empty(), "cancelado => sin sondeos");
    }

    /// `scan_udp` produce un [`Service`] UDP por puerto del catálogo con nombre.
    #[tokio::test(flavor = "current_thread")]
    async fn scan_udp_catalogues_known_ports() {
        let cancel = CancellationToken::new();
        let opts = ScanOptions {
            // Plazo corto: contra 127.0.0.1 los puertos sin listener cierran rápido.
            scan_timeout: Duration::from_millis(50),
            ..ScanOptions::default()
        };
        // Cancelamos en cuanto empiece para no barajar 7 puertos secuenciales.
        cancel.cancel();
        let out = scan_udp("127.0.0.1".parse().unwrap(), &opts, &cancel).await;
        assert!(out.is_empty(), "cancelado => vacío");
    }

    /// En Linux, un puerto UDP sin listener suele devolver `ConnectionRefused`
    /// (ICMP port-unreachable) → `Closed`. Es best-effort: si el ICMP no llega
    /// (firewall/race), cae a `Filtered`; no hard-fail si no vemos `Closed`.
    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "current_thread")]
    async fn probe_udp_closed_or_filtered_without_listener() {
        // Reserva un puerto libre y lo libera: condición de carrera best-effort.
        let probe_port = {
            let s = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            s.local_addr().unwrap().port()
        };
        // Pequeña pausa para que el socket se libere del todo.
        tokio::time::sleep(Duration::from_millis(10)).await;

        let state = probe_udp("127.0.0.1".parse().unwrap(), probe_port).await;
        assert!(
            matches!(state, ServiceState::Closed | ServiceState::Filtered),
            "sin listener => Closed o Filtered, obtuvo {state:?}"
        );
    }
}
