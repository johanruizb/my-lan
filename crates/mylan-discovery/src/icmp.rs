//! ICMP no-root best-effort (`SOCK_DGRAM` + `IPPROTO_ICMP`).
//!
//! En Linux, si el proceso cae dentro de `net.ipv4.ping_group_range`, se puede abrir
//! un socket datagrama ICMP **sin root** y enviar eco ICMP. El kernel añade la
//! cabecera IP y demuxa las respuestas por el *id* del datagrama. Si el socket no se
//! puede crear (sin permisos), la función degrada a una lista vacía: **nunca** asume
//! root. Las respuestas se reciben de forma asíncrona vía un [`tokio::net::UdpSocket`]
//! convertido desde el socket `socket2` (el fd datagrama soporta `sendto`/`recvfrom`
//! sin importar el protocolo L4).

use std::time::Duration;

use mylan_core::Observation;

use crate::iface::LanInterface;

#[cfg(target_os = "linux")]
use mylan_core::Source;
#[cfg(target_os = "linux")]
use socket2::{Domain, Protocol, Socket, Type};
#[cfg(target_os = "linux")]
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
#[cfg(target_os = "linux")]
use tokio::net::UdpSocket;

#[cfg(target_os = "linux")]
use crate::netutil::enumerate_hosts;

/// Stub no-Linux: el socket datagrama `IPPROTO_ICMP` no-root es específico de Linux
/// (`net.ipv4.ping_group_range`). Fuera de Linux no hay barrido ICMP → `vec![]`
/// (degradación documentada, Paso 0; no es un error porque la fn devuelve `Vec`).
#[cfg(not(target_os = "linux"))]
pub async fn icmp_sweep(_iface: &LanInterface, _timeout: Duration) -> Vec<Observation> {
    Vec::new()
}

/// Intenta un barrido ICMP sobre la subred. Best-effort: si el socket no se puede
/// abrir (sin `ping_group_range`), devuelve `vec![]`.
#[cfg(target_os = "linux")]
pub async fn icmp_sweep(iface: &LanInterface, timeout: Duration) -> Vec<Observation> {
    let sock = match open_icmp_socket() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let id = (std::process::id() as u16).wrapping_add(1);
    let hosts = enumerate_hosts(iface.ip, iface.prefix_len);
    // Envía eco a cada host (no bloquea: send_to es async).
    let mut seq = 1u16;
    for host in &hosts {
        let pkt = build_echo_request(id, seq);
        let target = SocketAddr::new(IpAddr::V4(*host), 0);
        let _ = sock.send_to(&pkt, target).await;
        seq = seq.wrapping_add(1);
    }
    // Recoge respuestas durante `timeout`, dedup por IP.
    let deadline = tokio::time::Instant::now() + timeout;
    let mut buf = [0u8; 1500];
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, sock.recv_from(&mut buf)).await {
            Ok(Ok((n, from))) => {
                if n >= 8
                    && buf[0] == 0
                    && matches!(id_for(&buf[..n]), Some(echo_id) if echo_id == id)
                {
                    if let IpAddr::V4(responder) = from.ip() {
                        if seen.insert(responder) {
                            out.push(Observation::new(Source::Icmp).with_ip(IpAddr::V4(responder)));
                        }
                    }
                }
            }
            _ => break,
        }
    }
    out
}

/// Abre un socket datagrama ICMP no-root y lo convierte a un socket tokio.
#[cfg(target_os = "linux")]
fn open_icmp_socket() -> std::io::Result<UdpSocket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
    sock.set_nonblocking(true)?;
    sock.set_reuse_address(true)?;
    // Enlaza a un puerto efímero para recibir respuestas.
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
    let saddr = socket2::SockAddr::from(bind_addr);
    sock.bind(&saddr)?;
    let std_sock: std::net::UdpSocket = sock.into();
    UdpSocket::from_std(std_sock)
}

#[cfg(target_os = "linux")]
fn id_for(buf: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes([*buf.get(4)?, *buf.get(5)?]))
}

/// Construye una cabecera ICMP echo request de 8 bytes (sin IP header).
#[cfg(target_os = "linux")]
fn build_echo_request(id: u16, seq: u16) -> Vec<u8> {
    let mut pkt = vec![0u8; 8];
    pkt[0] = 8; // Type: Echo Request
    pkt[1] = 0; // Code
                // checksum en [2..4] se calcula después
    pkt[4..6].copy_from_slice(&id.to_be_bytes());
    pkt[6..8].copy_from_slice(&seq.to_be_bytes());
    let cksum = checksum(&pkt);
    pkt[2..4].copy_from_slice(&cksum.to_be_bytes());
    pkt
}

/// Checksum de complemento a uno (RFC 1071) sobre datos de longitud par.
#[cfg(target_os = "linux")]
fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += u32::from(u16::from_be_bytes([data[i], data[i + 1]]));
        i += 2;
    }
    if i < data.len() {
        sum += u32::from(data[i]) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn echo_request_has_correct_type_and_id() {
        let pkt = build_echo_request(0x1234, 1);
        assert_eq!(pkt.len(), 8);
        assert_eq!(pkt[0], 8); // echo request
        assert_eq!(pkt[1], 0);
        assert_eq!(u16::from_be_bytes([pkt[4], pkt[5]]), 0x1234);
        assert_eq!(u16::from_be_bytes([pkt[6], pkt[7]]), 1);
    }

    #[test]
    fn checksum_is_not_zero_for_nonzero_data() {
        let pkt = build_echo_request(1, 1);
        let cksum = u16::from_be_bytes([pkt[2], pkt[3]]);
        assert_ne!(cksum, 0);
        // Verifica el checksum sobre el paquete completo (con checksum ya puesto,
        // el complemento debe dar 0 si se recalcula incluyendo el campo).
        let verify = checksum(&pkt);
        assert_eq!(verify, 0);
    }

    #[test]
    fn id_for_extracts_identifier() {
        let pkt = build_echo_request(0xabcd, 5);
        assert_eq!(id_for(&pkt), Some(0xabcd));
        assert_eq!(id_for(&[0, 1, 2, 3]), None); // too short
    }
}
