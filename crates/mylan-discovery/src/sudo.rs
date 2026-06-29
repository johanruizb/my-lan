//! Camino sudo: ARP sweep activo a nivel L2 + detección de `CAP_NET_RAW`.
//!
//! La detección de privilegios es implícita: `datalink::channel` abre un socket
//! `AF_PACKET` que requiere `CAP_NET_RAW`. Si falla, el flujo degrada a lista vacía
//! (nunca se asume root). El recv bloqueante corre en `spawn_blocking` con
//! `read_timeout` corto para poder respetar el *deadline* total.
//!
//! Las tramas ARP se construyen/parsean como bytes crudos (Ethernet + ARP) para
//! evitar la fricción del tipo `Ipv4Addr` propio de `pnet_base`; `pnet_datalink` se
//! usa únicamente para abrir el canal L2 y enviar/recibir.
//!
//! Solo Linux: en otras plataformas [`arp_sweep`] devuelve `vec![]` (impl default
//! portable, sin `todo!()`).

use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};

use mylan_core::{Observation, Source};

use crate::iface::LanInterface;
use crate::netutil::enumerate_hosts;

const ETH_HEADER_LEN: usize = 14;
const ARP_PACKET_LEN: usize = 28;
const ETHERTYPE_ARP: [u8; 2] = [0x08, 0x06];
const ARP_OP_REPLY: [u8; 2] = [0x00, 0x02];

/// ARP sweep activo sobre la subred. Requiere `CAP_NET_RAW`; degrada si no hay.
pub async fn arp_sweep(iface: &LanInterface, timeout: Duration) -> Vec<Observation> {
    arp_sweep_impl(iface, timeout).await
}

#[cfg(target_os = "linux")]
async fn arp_sweep_impl(iface: &LanInterface, timeout: Duration) -> Vec<Observation> {
    let iface = iface.clone();
    tokio::task::spawn_blocking(move || arp_sweep_blocking(&iface, timeout))
        .await
        .unwrap_or_default()
}

#[cfg(not(target_os = "linux"))]
async fn arp_sweep_impl(_iface: &LanInterface, _timeout: Duration) -> Vec<Observation> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn arp_sweep_blocking(iface: &LanInterface, timeout: Duration) -> Vec<Observation> {
    use pnet::datalink::{self, Channel, Config};

    let pnet_iface = match datalink::interfaces()
        .into_iter()
        .find(|i| i.name == iface.name)
    {
        Some(i) => i,
        None => return Vec::new(),
    };
    let src_mac = match pnet_iface.mac {
        Some(m) => m,
        None => return Vec::new(),
    };
    let config = Config {
        read_timeout: Some(Duration::from_millis(50)),
        write_timeout: Some(Duration::from_millis(50)),
        ..Default::default()
    };
    // Detección de CAP_NET_RAW: si channel() falla (EPERM/EACCES), degradamos.
    let channel = match datalink::channel(&pnet_iface, config) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) | Err(_) => return Vec::new(),
    };
    let (mut tx, mut rx) = channel;

    let src_mac_octets = src_mac.octets();
    let src_ip = iface.ip;
    let hosts = enumerate_hosts(src_ip, iface.prefix_len);
    for host in &hosts {
        let frame = build_arp_request(&src_mac_octets, src_ip, *host);
        let _ = tx.send_to(&frame, None);
    }

    let deadline = Instant::now() + timeout;
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    loop {
        if Instant::now() >= deadline {
            break;
        }
        match rx.next() {
            Ok(frame) => {
                let Some((mac, ip)) = parse_arp_reply(frame) else {
                    continue;
                };
                if mac.is_zero() || !seen.insert(ip) {
                    continue;
                }
                out.push(
                    Observation::new(Source::ArpSweep)
                        .with_ip(IpAddr::V4(ip))
                        .with_mac(mac),
                );
            }
            Err(_) => {
                // read_timeout expirado: reevalúa el deadline y sigue.
            }
        }
    }
    out
}

/// Construye una trama Ethernet + ARP request de 42 bytes.
#[cfg(target_os = "linux")]
fn build_arp_request(src_mac: &[u8; 6], src_ip: Ipv4Addr, target_ip: Ipv4Addr) -> Vec<u8> {
    let mut buf = vec![0u8; ETH_HEADER_LEN + ARP_PACKET_LEN];
    // Ethernet header.
    buf[0..6].copy_from_slice(&[0xff; 6]); // broadcast
    buf[6..12].copy_from_slice(src_mac);
    buf[12..14].copy_from_slice(&ETHERTYPE_ARP);
    // ARP header.
    buf[14..16].copy_from_slice(&[0x00, 0x01]); // htype: Ethernet
    buf[16..18].copy_from_slice(&[0x08, 0x00]); // ptype: IPv4
    buf[18] = 6; // hlen
    buf[19] = 4; // plen
    buf[20..22].copy_from_slice(&[0x00, 0x01]); // op: request
    buf[22..28].copy_from_slice(src_mac); // sender hw
    buf[28..32].copy_from_slice(&src_ip.octets()); // sender proto
    buf[32..38].copy_from_slice(&[0x00; 6]); // target hw (unknown for request)
    buf[38..42].copy_from_slice(&target_ip.octets()); // target proto
    buf
}

/// Parsea una trama Ethernet+ARP y devuelve (sender MAC, sender IP) si es una reply.
#[cfg(target_os = "linux")]
fn parse_arp_reply(frame: &[u8]) -> Option<(mylan_core::MacAddr, Ipv4Addr)> {
    if frame.len() < ETH_HEADER_LEN + ARP_PACKET_LEN {
        return None;
    }
    if frame[12..14] != ETHERTYPE_ARP {
        return None;
    }
    if frame[20..22] != ARP_OP_REPLY {
        return None;
    }
    let mac = mylan_core::MacAddr::from_octets(
        frame[22..28]
            .try_into()
            .expect("sender hw slice is 6 bytes"),
    );
    let mut ip_bytes = [0u8; 4];
    ip_bytes.copy_from_slice(&frame[28..32]);
    let ip = Ipv4Addr::from(ip_bytes);
    Some((mac, ip))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn test_iface() -> LanInterface {
        LanInterface {
            name: "enp37s0".into(),
            ip: "192.168.1.3".parse().unwrap(),
            prefix_len: 24,
            mac: None,
            gateway_ip: None,
            gateway_mac: None,
            dns_servers: Vec::new(),
            ssid: None,
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn build_arp_request_has_correct_layout() {
        let src_mac = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
        let frame = build_arp_request(
            &src_mac,
            "10.0.0.1".parse().unwrap(),
            "10.0.0.2".parse().unwrap(),
        );
        assert_eq!(frame.len(), 42);
        // Eth dst = broadcast.
        assert_eq!(&frame[0..6], &[0xff; 6]);
        // Eth src = src_mac.
        assert_eq!(&frame[6..12], &src_mac);
        // Ethertype ARP.
        assert_eq!(&frame[12..14], &[0x08, 0x06]);
        // op = request (1).
        assert_eq!(&frame[20..22], &[0x00, 0x01]);
        // target proto IP.
        assert_eq!(&frame[38..42], &[10, 0, 0, 2]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_arp_reply_extracts_sender() {
        let mut frame = vec![0u8; 42];
        frame[12..14].copy_from_slice(&ETHERTYPE_ARP);
        frame[20..22].copy_from_slice(&ARP_OP_REPLY);
        frame[22..28].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        frame[28..32].copy_from_slice(&[192, 168, 1, 50]);
        let (mac, ip) = parse_arp_reply(&frame).expect("parses reply");
        assert_eq!(mac.octets(), [0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        assert_eq!(ip, "192.168.1.50".parse::<Ipv4Addr>().unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_arp_reply_rejects_non_arp() {
        let mut frame = vec![0u8; 42];
        frame[12..14].copy_from_slice(&[0x08, 0x00]); // IPv4, not ARP
        assert!(parse_arp_reply(&frame).is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_arp_reply_rejects_request() {
        let mut frame = vec![0u8; 42];
        frame[12..14].copy_from_slice(&ETHERTYPE_ARP);
        frame[20..22].copy_from_slice(&[0x00, 0x01]); // request, not reply
        assert!(parse_arp_reply(&frame).is_none());
    }

    #[cfg(not(target_os = "linux"))]
    #[tokio::test]
    async fn arp_sweep_no_op_off_linux() {
        let iface = test_iface();
        assert!(arp_sweep(&iface, Duration::from_millis(10))
            .await
            .is_empty());
    }
}
