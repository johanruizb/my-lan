//! `traceroute` por host — UDP con TTL incremental + cola de errores ICMP (AC-7).
//!
//! En cada salto se envía un datagrama UDP a un puerto alto (probablemente
//! cerrado) con `TTL = hop`, y se lee la **cola de errores** del socket
//! (`IP_RECVERR` + `MSG_ERRQUEUE`) para obtener el remitente del ICMP
//! time-exceeded (= la IP del salto). `socket2` no expone la cola de errores,
//! por eso se usa `nix` (`ControlMessageOwned::Ipv4RecvErr`).
//!
//! **Sin `IP_RECVERR`/error-queue** sólo sería visible el hop final → traceroute
//! degradaría a reachability. Por eso se requiere `nix` (Paso 5, R-MAJOR3).
//!
//! Sin root, sin raw sockets, sin binarios externos. La IP del salto viene del
//! `SO_EE_OFFENDER` (remitente del ICMP), no de `msg_name` (que es el destino
//! original). Reverse DNS por salto vía `system_resolver()` (best-effort).
//!
//! Limitación documentada: IPv6 traceroute queda fuera de este push (requiere
//! `IPV6_RECVERR` + hop-limit + ICMPv6); se devuelve un error claro.

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::unix::io::{AsRawFd, BorrowedFd, RawFd};
use std::time::{Duration, Instant};

use nix::errno::Errno;
use nix::libc;
use nix::sys::socket::{
    recvmsg, setsockopt, sockopt::Ipv4RecvErr, ControlMessageOwned, MsgFlags, SockaddrStorage,
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use mylan_core::TraceHop;

use crate::dns::{reverse_lookup, system_resolver};
use crate::error::DiscoveryError;

/// Salto crudo antes de resolver hostname: (hop_number, ip, latency_ms, state).
type RawHop = (u8, Option<IpAddr>, Option<u64>, String);

/// Puerto base para las sondas UDP (convención de traceroute).
const TRACE_PORT_BASE: u16 = 33434;
/// ICMP Type 11 = Time Exceeded (router intermedio).
const ICMP_TIME_EXCEEDED: u8 = 11;
/// ICMP Type 3 = Destination Unreachable.
const ICMP_DEST_UNREACH: u8 = 3;
/// ICMP Code 3 = Port Unreachable (el destino alcanzó un puerto cerrado).
const ICMP_PORT_UNREACH: u8 = 3;

/// `traceroute` a `target` (AC-7).
///
/// Hasta `max_hops` (default 30) saltos, `timeout` por salto (default 1000 ms).
/// Devuelve un [`TraceHop`] por salto con IP, hostname (reverse DNS best-effort),
/// latencia y estado. IPv6 no soportado en este push (error claro).
pub async fn traceroute_host(
    target: IpAddr,
    max_hops: u8,
    timeout: Duration,
) -> Result<Vec<TraceHop>, DiscoveryError> {
    let target_v4 = match target {
        IpAddr::V4(v4) => v4,
        IpAddr::V6(_) => {
            return Err(DiscoveryError::Dns(
                "IPv6 traceroute aún no soportado en este push (usar IPv4)".to_string(),
            ));
        }
    };
    let max_hops = max_hops.max(1);

    // Núcleo síncrono (socket2 + nix + libc::poll es bloqueante por salto): se
    // ejecuta en `spawn_blocking` para no congelar el runtime tokio.
    let hops_raw =
        tokio::task::spawn_blocking(move || traceroute_sync(target_v4, max_hops, timeout))
            .await
            .map_err(|e| {
                DiscoveryError::Io(io::Error::other(format!("traceroute task join: {e}")))
            })??;

    // Reverse DNS por salto (best-effort, acotado a 500 ms por hop).
    let resolver = system_resolver()?;
    let mut hops = Vec::with_capacity(hops_raw.len());
    for (hop_number, ip, latency_ms, state) in hops_raw {
        let hostname = match ip {
            Some(ip) => {
                tokio::time::timeout(Duration::from_millis(500), reverse_lookup(&resolver, ip))
                    .await
                    .ok()
                    .flatten()
            }
            None => None,
        };
        hops.push(TraceHop {
            hop_number,
            ip,
            hostname,
            latency_ms,
            state,
        });
    }
    Ok(hops)
}

/// Núcleo síncrono: itera TTL 1..=max_hops sondando un socket UDP por salto.
fn traceroute_sync(
    target: Ipv4Addr,
    max_hops: u8,
    timeout: Duration,
) -> Result<Vec<RawHop>, DiscoveryError> {
    let mut hops = Vec::new();
    for ttl in 1..=max_hops {
        let (ip, latency, state, reached) = probe_hop(target, ttl, timeout)?;
        hops.push((ttl, ip, latency, state));
        if reached {
            break;
        }
    }
    Ok(hops)
}

/// Sonda un salto: socket UDP fresco, `IP_RECVERR` + `TTL=ttl`, connect+send,
/// luego drena la cola de errores esperando hasta `timeout`.
fn probe_hop(
    target: Ipv4Addr,
    ttl: u8,
    timeout: Duration,
) -> Result<(Option<IpAddr>, Option<u64>, String, bool), DiscoveryError> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    // IP_RECVERR: sin esto los time-exceeded de los saltos intermedios no se
    // entregan al socket (sólo se vería el hop final → reachability).
    let fd = sock.as_raw_fd();
    let bfd = unsafe { BorrowedFd::borrow_raw(fd) };
    setsockopt(&bfd, Ipv4RecvErr, &true).map_err(io::Error::from)?;
    // Non-blocking: poll(2) es el punto de espera; recv/recvmsg nunca bloquean.
    sock.set_nonblocking(true)?;
    sock.set_ttl_v4(u32::from(ttl))?;

    let port = TRACE_PORT_BASE + u16::from(ttl);
    let dest = SockAddr::from(SocketAddr::new(IpAddr::V4(target), port));
    // UDP connect fija el peer 4-tuple para que el kernel matchee los ICMP
    // (time-exceeded de intermedios + port-unreachable del destino) a este socket.
    sock.connect(&dest)?;

    let start = Instant::now();
    let _ = sock.send(b"mylan-trace"); // best-effort
    let result = poll_errqueue_v4(fd, timeout);
    let elapsed = start.elapsed();

    let (ip, state, reached) = match result {
        ErrQueueResult::Got(ip, ee_type, ee_code) => {
            let reached = ee_type == ICMP_DEST_UNREACH && ee_code == ICMP_PORT_UNREACH;
            let state = if reached {
                "reached".to_string()
            } else if ee_type == ICMP_TIME_EXCEEDED {
                "time-exceeded".to_string()
            } else {
                format!("icmp {ee_type}/{ee_code}")
            };
            (Some(ip), state, reached)
        }
        ErrQueueResult::Timeout => (None, "*".to_string(), false),
        ErrQueueResult::Error => (None, "error".to_string(), false),
    };
    let latency = if ip.is_some() {
        Some(elapsed.as_millis() as u64)
    } else {
        None
    };
    Ok((ip, latency, state, reached))
}

/// Resultado de drenar la cola de errores de un socket.
enum ErrQueueResult {
    /// Se obtuvo un error ICMP: (IP del remitente, ICMP type, ICMP code).
    Got(IpAddr, u8, u8),
    /// Sin error dentro del timeout (salto mudo `*`).
    Timeout,
    /// Error de sistema (socket inválido, etc.).
    Error,
}

/// Bloquea en `poll(2)` esperando que la cola de errores tenga datos, hasta
/// `timeout`. Cuando hay datos, lee un error de la cola con `MSG_ERRQUEUE`.
fn poll_errqueue_v4(fd: RawFd, timeout: Duration) -> ErrQueueResult {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return ErrQueueResult::Timeout;
        }
        let ms = i32::try_from(remaining.as_millis())
            .unwrap_or(i32::MAX)
            .max(1);
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let r = unsafe { libc::poll(&mut pfd as *mut libc::pollfd, 1, ms) };
        if r < 0 {
            if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                continue; // EINTR: reintentar con el timeout restante.
            }
            return ErrQueueResult::Error;
        }
        if r == 0 {
            return ErrQueueResult::Timeout;
        }
        // El kernel señala los errores ICMP pendientes (time-exceeded,
        // port-unreachable) en la cola de errores vía `POLLERR`, no `POLLIN`.
        // Sin esta rama, todo hop llegaría como `Error` (bug original).
        if pfd.revents & libc::POLLERR != 0 {
            return match read_errqueue_v4(fd) {
                Ok(Some((ip, t, c))) => ErrQueueResult::Got(ip, t, c),
                // POLLERR sin error legible en la cola: error de socket real.
                Ok(None) => ErrQueueResult::Error,
                Err(_) => ErrQueueResult::Error,
            };
        }
        if pfd.revents & libc::POLLNVAL != 0 {
            return ErrQueueResult::Error; // fd inválido.
        }
        if pfd.revents & libc::POLLIN != 0 {
            // Datos normales (respuesta UDP inesperada al destino): drenar y
            // seguir esperando el error ICMP dentro del timeout restante.
            let mut drain = [0u8; 1500];
            unsafe {
                let _ = libc::recv(fd, drain.as_mut_ptr() as *mut _, drain.len(), 0);
            }
            continue;
        }
    }
}

/// Lee un error de la cola del socket (`MSG_ERRQUEUE`). Devuelve
/// `Ok(Some((hop_ip, ee_type, ee_code)))` si hay un error ICMP; `Ok(None)` si la
/// cola está vacía (EAGAIN); `Err` ante otro error de sistema.
///
/// La IP del salto es el `SO_EE_OFFENDER` (remitente del ICMP), no `msg_name`
/// (que es el destino original del datagrama).
fn read_errqueue_v4(fd: RawFd) -> io::Result<Option<(IpAddr, u8, u8)>> {
    let mut buf = [0u8; 8];
    let mut iov = [io::IoSliceMut::new(&mut buf)];
    // Buffer de mensajes de control: sobredimensionado (512 B) para albergar un
    // `Ipv4RecvErr` (sock_extended_err + sockaddr_in + cabecera cmsg). `recvmsg`
    // trunca si falta, pero 512 B sobra para un cmsg de error ICMP.
    let mut cspace: Vec<u8> = vec![0u8; 512];
    match recvmsg::<SockaddrStorage>(fd, &mut iov, Some(&mut cspace), MsgFlags::MSG_ERRQUEUE) {
        Ok(m) => {
            let info = m.cmsgs().ok().and_then(|mut cmsgs| {
                cmsgs.find_map(|cm| match cm {
                    ControlMessageOwned::Ipv4RecvErr(ee, offender) => {
                        let ip = offender
                            .map(|o| Ipv4Addr::from(u32::from_be(o.sin_addr.s_addr)))
                            .unwrap_or(Ipv4Addr::UNSPECIFIED);
                        Some((IpAddr::V4(ip), ee.ee_type, ee.ee_code))
                    }
                    _ => None,
                })
            });
            Ok(info)
        }
        Err(Errno::EAGAIN) => Ok(None),
        Err(e) => Err(io::Error::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;

    /// `traceroute 127.0.0.1` (AC-7): al menos 1 hop. Para localhost, TTL=1
    /// entrega el datagrama al destino (puerto cerrado) → port-unreachable
    /// desde 127.0.0.1 → primer hop = 127.0.0.1, estado `reached`.
    #[tokio::test]
    async fn traceroute_loopback_has_at_least_one_hop() {
        let hops = traceroute_host(IpAddr::V4(Ipv4Addr::LOCALHOST), 8, Duration::from_secs(1))
            .await
            .expect("traceroute ok");
        assert!(
            !hops.is_empty(),
            "traceroute a localhost debe tener >=1 hop: {hops:?}"
        );
        let first = &hops[0];
        assert_eq!(first.hop_number, 1);
        if let Some(ip) = first.ip {
            assert_eq!(
                ip,
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                "primer hop de localhost debería ser 127.0.0.1: {first:?}"
            );
            assert_eq!(first.state, "reached");
        }
    }

    /// IPv6 devuelve un error claro (limitación documentada del push).
    #[tokio::test]
    async fn traceroute_ipv6_is_unsupported_error() {
        let res = traceroute_host(
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            8,
            Duration::from_millis(100),
        )
        .await;
        assert!(res.is_err());
        let msg = format!("{}", res.unwrap_err());
        assert!(msg.contains("IPv6"), "mensaje: {msg}");
    }

    /// `max_hops` se respeta y se floora a 1.
    #[tokio::test]
    async fn traceroute_max_hops_floored() {
        // Con max_hops=0 se floora a 1 → al menos 1 sonda → >=1 hop o error controlado.
        let hops = traceroute_host(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            0,
            Duration::from_millis(300),
        )
        .await
        .expect("traceroute ok");
        assert!(!hops.is_empty());
        assert!(hops.len() <= 1, "max_hops=0 (floor 1) → como mucho 1 hop");
    }
}
