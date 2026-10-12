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
        assert!(enumerate_hosts("192.168.1.0".parse().unwrap(), 33).is_empty());
        assert!(enumerate_hosts("192.168.1.0".parse().unwrap(), 40).is_empty());
        assert!(enumerate_hosts("192.168.1.0".parse().unwrap(), 255).is_empty());
    }

    #[test]
    fn prefix_30_returns_two_hosts() {
        let hosts = enumerate_hosts("192.168.1.0".parse().unwrap(), 30);
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0], "192.168.1.1".parse::<Ipv4Addr>().unwrap());
        assert_eq!(hosts[1], "192.168.1.2".parse::<Ipv4Addr>().unwrap());
    }

    #[test]
    fn handles_non_network_aligned_ip() {
        let hosts = enumerate_hosts("192.168.1.45".parse().unwrap(), 24);
        assert_eq!(hosts.len(), 254);
        assert_eq!(hosts.first().copied(), Some("192.168.1.1".parse().unwrap()));
        assert_eq!(
            hosts.last().copied(),
            Some("192.168.1.254".parse().unwrap())
        );
    }

    #[test]
    fn prefix_0_capped_at_safety_limit() {
        let hosts = enumerate_hosts("0.0.0.0".parse().unwrap(), 0);
        assert_eq!(hosts.len(), 4096);
        assert_eq!(hosts.first().copied(), Some("0.0.0.1".parse().unwrap()));
    }

    #[test]
    fn wide_subnet_capped_at_4096_hosts() {
        // /16 subnet has 65534 hosts, should be capped at 4096
        let hosts = enumerate_hosts("10.0.0.0".parse().unwrap(), 16);
        assert_eq!(hosts.len(), 4096);
        assert_eq!(hosts.first().copied(), Some("10.0.0.1".parse().unwrap()));

        // /20 subnet has 4094 hosts, which is below 4096 and should return all 4094
        let hosts_20 = enumerate_hosts("10.0.0.0".parse().unwrap(), 20);
        assert_eq!(hosts_20.len(), 4094);
    }
}
