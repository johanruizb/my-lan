//! Identidad estable de dispositivo (`DeviceIdentity`).
//!
//! Regla (P5): la MAC normalizada no-cero manda; si no hay MAC utilizable se cae
//! a la IP. La identidad es la clave de deduplicación/upsert para que re-escanear
//! actualice en vez de duplicar.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::mac::MacAddr;

/// Clave de identidad estable de un dispositivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceIdentity {
    /// Identidad por MAC (preferida).
    Mac(MacAddr),
    /// Fallback por IP cuando no hay MAC utilizable.
    Ip(IpAddr),
}

impl DeviceIdentity {
    /// Deriva la identidad a partir de una MAC y/o IP candidatas.
    ///
    /// Una MAC en ceros se descarta (entrada ARP incompleta) y se intenta la IP.
    /// Devuelve `None` si no hay ninguna pista utilizable.
    #[must_use]
    pub fn derive(mac: Option<MacAddr>, ip: Option<IpAddr>) -> Option<Self> {
        match mac {
            Some(mac) if !mac.is_zero() => Some(Self::Mac(mac)),
            _ => ip.map(Self::Ip),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("valid ip")
    }

    #[test]
    fn prefers_mac_over_ip() {
        let mac = MacAddr::parse("aa:bb:cc:dd:ee:ff");
        let id = DeviceIdentity::derive(mac, Some(ip("192.168.1.10"))).expect("identity");
        assert_eq!(id, DeviceIdentity::Mac(mac.expect("mac")));
    }

    #[test]
    fn falls_back_to_ip_when_mac_missing() {
        let id = DeviceIdentity::derive(None, Some(ip("192.168.1.10"))).expect("identity");
        assert_eq!(id, DeviceIdentity::Ip(ip("192.168.1.10")));
    }

    #[test]
    fn ignores_zero_mac_and_uses_ip() {
        let zero = MacAddr::parse("00:00:00:00:00:00");
        let id = DeviceIdentity::derive(zero, Some(ip("192.168.1.10"))).expect("identity");
        assert_eq!(id, DeviceIdentity::Ip(ip("192.168.1.10")));
    }

    #[test]
    fn none_without_any_hint() {
        assert!(DeviceIdentity::derive(None, None).is_none());
        let zero = MacAddr::parse("00:00:00:00:00:00");
        assert!(DeviceIdentity::derive(zero, None).is_none());
    }
}
