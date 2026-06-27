//! `ping` por host — eco ICMP no-root con degradación a TCP connect (AC-6).
//!
//! Refactor del barrido ICMP de `icmp.rs` (subnet sweep) a ping por host único.
//! Usa `SOCK_DGRAM` + `IPPROTO_ICMP`/`IPPROTO_ICMPV6` (no root si el proceso
//! cae dentro de `net.ipv4.ping_group_range`). Si el socket ICMP no se puede
//! abrir, degrada a TCP connect (80/443) con `PingMethod::TcpConnect` — **nunca**
//! presenta el fallback como ICMP (P4/AC-6).
//!
//! Sin root, sin binarios externos. El método efectivo queda reflejado en
//! `PingResult.method` para que la CLI lo muestre.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::{TcpStream, UdpSocket};

use mylan_core::{PingMethod, PingResult};

use crate::error::DiscoveryError;

/// Eco ICMP/TCP a `target` (AC-6).
///
/// Envía `count` sondas (default 4) con `timeout` por sonda (default 1000 ms).
/// Devuelve un [`PingResult`] con latencia media, packet loss y método usado.
/// Nunca propaga error de red: un host inalcanzable se reporta como
/// `reachable=false` (no como `Err`).
pub async fn ping_host(
    target: IpAddr,
    count: u32,
    timeout: Duration,
) -> Result<PingResult, DiscoveryError> {
    let count = count.max(1);
    match target {
        IpAddr::V4(v4) => match open_icmp(
            Domain::IPV4,
            Protocol::ICMPV4,
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        ) {
            Ok(sock) => Ok(ping_icmp(
                IpAddr::V4(v4),
                count,
                timeout,
                sock,
                ECHO_REQUEST_V4,
                ECHO_REPLY_V4,
            )
            .await),
            Err(_) => Ok(ping_tcp(IpAddr::V4(v4), count, timeout).await),
        },
        IpAddr::V6(v6) => match open_icmp(
            Domain::IPV6,
            Protocol::ICMPV6,
            IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        ) {
            Ok(sock) => Ok(ping_icmp(
                IpAddr::V6(v6),
                count,
                timeout,
                sock,
                ECHO_REQUEST_V6,
                ECHO_REPLY_V6,
            )
            .await),
            Err(_) => Ok(ping_tcp(IpAddr::V6(v6), count, timeout).await),
        },
    }
}

/// Tipo ICMP "Echo Request" / "Echo Reply" (RFC 792).
const ECHO_REQUEST_V4: u8 = 8;
const ECHO_REPLY_V4: u8 = 0;
/// Tipo ICMPv6 "Echo Request" / "Echo Reply" (RFC 4443).
const ECHO_REQUEST_V6: u8 = 128;
const ECHO_REPLY_V6: u8 = 129;

/// Ping ICMP sobre un socket datagrama ya abierto. Secuencial por paquete:
/// envía eco (id, seq) y espera la respuesta con mismo id+seq dentro de
/// `timeout`. Respuestas tardías de otros seq se descartan sin contar.
async fn ping_icmp(
    target: IpAddr,
    count: u32,
    timeout: Duration,
    sock: UdpSocket,
    req_type: u8,
    reply_type: u8,
) -> PingResult {
    // El id demuxa las respuestas a nivel kernel (SOCK_DGRAM ICMP). Usar el pid
    // reduce colisiones entre procesos concurrentes.
    let id = (std::process::id() as u16).wrapping_add(1);
    let dest = SocketAddr::new(target, 0);
    let mut received = 0u32;
    let mut latencies: Vec<u64> = Vec::new();
    let mut buf = [0u8; 1500];

    for seq in 1..=count {
        let pkt = build_echo(req_type, id, seq as u16);
        let start = tokio::time::Instant::now();
        let _ = sock.send_to(&pkt, dest).await;
        let deadline = start + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, sock.recv_from(&mut buf)).await {
                Ok(Ok((n, _from))) => {
                    if n >= 8
                        && buf[0] == reply_type
                        && id_for(&buf[..n]) == Some(id)
                        && seq_for(&buf[..n]) == Some(seq as u16)
                    {
                        latencies.push(start.elapsed().as_millis() as u64);
                        received += 1;
                        break;
                    }
                    // Respuesta tardía de otro seq: se descarta y se sigue esperando.
                }
                _ => break, // timeout o error de recv: paquete perdido.
            }
        }
    }

    build_result(target, count, received, latencies, PingMethod::Icmp)
}

/// Ping TCP-connect (fallback). Por cada sonda intenta conectar a 80/443
/// dentro de `timeout` total; la primera conexión aceptada cuenta como recibida.
async fn ping_tcp(target: IpAddr, count: u32, timeout: Duration) -> PingResult {
    const PROBE_PORTS: &[u16] = &[80, 443];
    let mut received = 0u32;
    let mut latencies: Vec<u64> = Vec::new();

    for _ in 0..count {
        let start = tokio::time::Instant::now();
        let deadline = start + timeout;
        let mut ok = false;
        for port in PROBE_PORTS {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            let addr = SocketAddr::new(target, *port);
            let connect = TcpStream::connect(addr);
            if tokio::time::timeout(remaining, connect)
                .await
                .is_ok_and(|r| r.is_ok())
            {
                ok = true;
                break;
            }
        }
        if ok {
            latencies.push(start.elapsed().as_millis() as u64);
            received += 1;
        }
    }

    build_result(target, count, received, latencies, PingMethod::TcpConnect)
}

/// Abre un socket datagrama ICMP no-root y lo convierte a `UdpSocket` tokio.
/// `domain`/`protocol` seleccionan IPv4 o IPv6; `bind_addr` es la wildcard.
fn open_icmp(domain: Domain, protocol: Protocol, bind_ip: IpAddr) -> std::io::Result<UdpSocket> {
    let sock = Socket::new(domain, Type::DGRAM, Some(protocol))?;
    sock.set_nonblocking(true)?;
    sock.set_reuse_address(true)?;
    let bind_addr = SocketAddr::new(bind_ip, 0);
    sock.bind(&socket2::SockAddr::from(bind_addr))?;
    let std_sock: std::net::UdpSocket = sock.into();
    UdpSocket::from_std(std_sock)
}

/// Agrega estadísticas en un [`PingResult`].
fn build_result(
    target: IpAddr,
    sent: u32,
    received: u32,
    latencies: Vec<u64>,
    method: PingMethod,
) -> PingResult {
    let reachable = received > 0;
    let latency_ms = if latencies.is_empty() {
        None
    } else {
        let sum: u64 = latencies.iter().sum();
        Some(sum / u64::try_from(latencies.len()).unwrap_or(1))
    };
    let packet_loss = if sent > 0 {
        Some((sent - received) as f32 / sent as f32)
    } else {
        None
    };
    PingResult {
        target,
        reachable,
        latency_ms,
        packet_loss,
        packets_sent: sent,
        packets_received: received,
        method,
    }
}

/// Construye una cabecera ICMP echo request de 8 bytes (sin IP header).
/// El checksum lo rellena el kernel para sockets SOCK_DGRAM (v4 y v6).
fn build_echo(req_type: u8, id: u16, seq: u16) -> Vec<u8> {
    let mut pkt = vec![0u8; 8];
    pkt[0] = req_type;
    pkt[1] = 0;
    pkt[4..6].copy_from_slice(&id.to_be_bytes());
    pkt[6..8].copy_from_slice(&seq.to_be_bytes());
    let cksum = checksum(&pkt);
    pkt[2..4].copy_from_slice(&cksum.to_be_bytes());
    pkt
}

fn id_for(buf: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes([*buf.get(4)?, *buf.get(5)?]))
}

fn seq_for(buf: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes([*buf.get(6)?, *buf.get(7)?]))
}

/// Checksum de complemento a uno (RFC 1071) sobre datos de longitud par.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `ping 127.0.0.1` (AC-6): reachable + método distinguible. Si ICMP está
    /// disponible (caso normal en Linux con `ping_group_range`), reachable=true
    /// y method=Icmp. En sandboxes que bloquean ICMP, method=TcpConnect y no
    /// sobre-asertamos reachable (no falso negativo).
    #[tokio::test]
    async fn ping_loopback_reachable_with_method() {
        let r = ping_host(IpAddr::V4(Ipv4Addr::LOCALHOST), 4, Duration::from_secs(1))
            .await
            .expect("ping ok");
        assert_eq!(r.target, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(r.packets_sent, 4);
        assert!(matches!(
            r.method,
            PingMethod::Icmp | PingMethod::TcpConnect
        ));
        if r.method == PingMethod::Icmp {
            assert!(r.reachable, "ICMP a localhost debe ser reachable: {r:?}");
            assert!(r.packets_received >= 1);
            assert!(r.packet_loss.unwrap_or(1.0) <= 1.0);
        }
    }

    /// `count` se respeta (mínimo 1).
    #[tokio::test]
    async fn ping_respects_count() {
        let r = ping_host(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            2,
            Duration::from_millis(500),
        )
        .await
        .expect("ping ok");
        assert_eq!(r.packets_sent, 2);
    }

    /// Un host inalcanzable (link-local no routable) no responde → reachable=false.
    /// No asertamos method (ICMP o TCP) porque ambos fallarán; lo crítico es que
    /// no haya panic ni Err.
    #[tokio::test]
    async fn ping_unreachable_is_not_reachable() {
        // 240.0.0.1: reservado (RFC 1112), no routable. Timeout corto para el test.
        let target: IpAddr = "240.0.0.1".parse().unwrap();
        let r = ping_host(target, 2, Duration::from_millis(200))
            .await
            .expect("ping ok");
        assert_eq!(r.packets_sent, 2);
        assert_eq!(r.packets_received, 0);
        assert!(!r.reachable);
        assert_eq!(r.packet_loss, Some(1.0));
    }

    /// Red real (internet): `#[ignore]` — requiere red y puede ser flaky.
    #[tokio::test]
    #[ignore = "red real: ejecutar con --ignored-urls/manual"]
    async fn ping_real_internet_host() {
        let target: IpAddr = "1.1.1.1".parse().unwrap();
        let r = ping_host(target, 4, Duration::from_secs(1))
            .await
            .expect("ping ok");
        assert!(r.reachable, "1.1.1.1 debería responder: {r:?}");
    }

    #[test]
    fn echo_request_v4_has_correct_type_and_id() {
        let pkt = build_echo(ECHO_REQUEST_V4, 0x1234, 1);
        assert_eq!(pkt.len(), 8);
        assert_eq!(pkt[0], ECHO_REQUEST_V4);
        assert_eq!(u16::from_be_bytes([pkt[4], pkt[5]]), 0x1234);
        assert_eq!(u16::from_be_bytes([pkt[6], pkt[7]]), 1);
    }

    #[test]
    fn echo_request_v6_uses_type_128() {
        let pkt = build_echo(ECHO_REQUEST_V6, 1, 9);
        assert_eq!(pkt[0], ECHO_REQUEST_V6);
        assert_eq!(seq_for(&pkt), Some(9));
    }

    #[test]
    fn checksum_round_trips_to_zero() {
        let pkt = build_echo(ECHO_REQUEST_V4, 1, 1);
        // Recalcular el checksum sobre el paquete (con checksum ya puesto) da 0.
        assert_eq!(checksum(&pkt), 0);
    }
}
