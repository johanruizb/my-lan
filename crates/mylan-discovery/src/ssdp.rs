//! SSDP/UPnP: `M-SEARCH` a `239.255.255.250:1900` con `IP_MULTICAST_IF` = interfaz LAN.
//!
//! Las respuestas llegan *unicast* al socket emisor; se parsea la cabecera `LOCATION`
//! para extraer la IP del dispositivo. Se evita que el multicast salga por otras
//! interfaces fijando `IP_MULTICAST_IF` a la IPv4 de la interfaz LAN.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;

use mylan_core::{Observation, Source};

use crate::iface::LanInterface;

const SSDP_MULTICAST: &str = "239.255.255.250:1900";
const M_SEARCH: &[u8] = b"M-SEARCH * HTTP/1.1\r\n\
HOST: 239.255.255.250:1900\r\n\
MAN: \"ssdp:discover\"\r\n\
MX: 2\r\n\
ST: ssdp:all\r\n\r\n";

/// Ejecuta un `M-SEARCH` SSDP durante `timeout` y devuelve [`Observation`]s crudas.
pub async fn ssdp_discover(iface: &LanInterface, timeout: Duration) -> Vec<Observation> {
    let sock = match open_ssdp_socket(iface.ip) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let multicast_addr: SocketAddr = SSDP_MULTICAST.parse().expect("valid ssdp addr");
    if sock.send_to(M_SEARCH, multicast_addr).await.is_err() {
        return Vec::new();
    }

    let deadline = tokio::time::Instant::now() + timeout;
    let mut buf = [0u8; 4096];
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, sock.recv_from(&mut buf)).await {
            Ok(Ok((n, _))) => {
                let text = String::from_utf8_lossy(&buf[..n]);
                if let Some(ip) = parse_location_ip(&text) {
                    if seen.insert(ip) {
                        let mut obs = Observation::new(Source::Ssdp).with_ip(ip);
                        if let Some(st) = parse_header(&text, "ST") {
                            obs = obs.with_hint("ssdp.st", st);
                        }
                        if let Some(server) = parse_header(&text, "SERVER") {
                            obs = obs.with_hint("ssdp.server", server);
                        }
                        out.push(obs);
                    }
                }
            }
            _ => break,
        }
    }
    out
}

/// Abre un socket UDP multicast saliente con `IP_MULTICAST_IF` fijado a la iface LAN.
fn open_ssdp_socket(iface_ip: Ipv4Addr) -> std::io::Result<UdpSocket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    sock.set_reuse_address(true)?;
    sock.set_multicast_if_v4(&iface_ip)?;
    sock.set_multicast_loop_v4(false)?;
    sock.set_nonblocking(true)?;
    let saddr = socket2::SockAddr::from(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0));
    sock.bind(&saddr)?;
    let std_sock: std::net::UdpSocket = sock.into();
    UdpSocket::from_std(std_sock)
}

/// Extrae la IP del host de la cabecera `LOCATION` (URL absoluta).
fn parse_location_ip(text: &str) -> Option<IpAddr> {
    let location = parse_header(text, "LOCATION")?;
    let after_scheme = location.split("://").nth(1)?;
    let host_port = after_scheme.split('/').next()?;
    let host = host_port.rsplit_once(':').map_or(host_port, |(h, _)| h);
    host.parse::<IpAddr>().ok()
}

/// Devuelve el valor de una cabecera HTTP (case-insensitive).
fn parse_header(text: &str, name: &str) -> Option<String> {
    let needle = format!("{name}:");
    for line in text.lines() {
        if line
            .to_ascii_lowercase()
            .starts_with(&needle.to_ascii_lowercase())
        {
            return Some(line[needle.len()..].trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_location_ip_v4() {
        let resp = "HTTP/1.1 200 OK\r\nLOCATION: http://192.168.1.20:1900/desc.xml\r\nST: upnp:rootdevice\r\n\r\n";
        assert_eq!(
            parse_location_ip(resp),
            Some("192.168.1.20".parse().unwrap())
        );
    }

    #[test]
    fn parses_header_case_insensitive() {
        let resp = "HTTP/1.1 200 OK\r\nserver: Linux/1.0\r\n\r\n";
        assert_eq!(parse_header(resp, "SERVER"), Some("Linux/1.0".to_string()));
        assert_eq!(parse_header(resp, "st"), None);
    }

    #[test]
    fn returns_none_without_location() {
        let resp = "HTTP/1.1 200 OK\r\nST: upnp:rootdevice\r\n\r\n";
        assert_eq!(parse_location_ip(resp), None);
    }

    #[test]
    fn handles_location_without_port() {
        let resp = "LOCATION: http://10.0.0.5/desc.xml\r\n\r\n";
        assert_eq!(parse_location_ip(resp), Some("10.0.0.5".parse().unwrap()));
    }
}
