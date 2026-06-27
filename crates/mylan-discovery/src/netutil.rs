//! Utilidades de red puras (sin I/O) compartidas por las técnicas de descubrimiento.

use std::net::Ipv4Addr;

/// Enumera las direcciones de host de una subred IPv4 (excluye network y broadcast).
///
/// Para una `/24` devuelve los 254 hosts `network+1 .. broadcast-1`. Soporta cualquier
/// longitud de prefijo `0..=32`. Hay un tope de seguridad de 4096 hosts para no
/// explosions en subredes muy anchas (el barrido quick opera sobre `/24`).
#[must_use]
pub fn enumerate_hosts(ip: Ipv4Addr, prefix_len: u8) -> Vec<Ipv4Addr> {
    if prefix_len > 32 {
        return Vec::new();
    }
    let base = u32::from(ip);
    let mask = if prefix_len == 0 {
        0
    } else {
        !0u32 << (32 - prefix_len)
    };
    let network = base & mask;
    let broadcast = network | !mask;
    // /31 y /32 no tienen hosts usables en este modelo.
    if prefix_len >= 31 {
        return Vec::new();
    }
    let mut hosts = Vec::new();
    let mut cur = network.wrapping_add(1);
    while cur < broadcast {
        hosts.push(Ipv4Addr::from(cur));
        if hosts.len() >= 4096 {
            break;
        }
        cur = cur.wrapping_add(1);
    }
    hosts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerates_full_24() {
        let hosts = enumerate_hosts("192.168.1.0".parse().unwrap(), 24);
        assert_eq!(hosts.len(), 254);
        assert_eq!(hosts.first().copied(), Some("192.168.1.1".parse().unwrap()));
        assert_eq!(
            hosts.last().copied(),
            Some("192.168.1.254".parse().unwrap())
        );
    }

    #[test]
    fn excludes_network_and_broadcast() {
        let hosts = enumerate_hosts("10.0.0.0".parse().unwrap(), 24);
        assert!(!hosts.contains(&"10.0.0.0".parse().unwrap()));
        assert!(!hosts.contains(&"10.0.0.255".parse().unwrap()));
    }

    #[test]
    fn empty_for_31_and_32() {
        assert!(enumerate_hosts("192.168.1.0".parse().unwrap(), 31).is_empty());
        assert!(enumerate_hosts("192.168.1.5".parse().unwrap(), 32).is_empty());
    }

    #[test]
    fn invalid_prefix_returns_empty() {
        assert!(enumerate_hosts("192.168.1.0".parse().unwrap(), 40).is_empty());
    }
}
