//! Detección de la interfaz por defecto (default-route), IP, MAC, gateway y CIDR.
//!
//! Usa `netdev` y filtra interfaces no aptas para descubrimiento LAN: loopback,
//! docker, tailscale y túneles/bridges. La interfaz elegida es la del *default route*
//! (campo `default`) siempre que tenga IPv4 y no esté filtrada.

use std::net::{IpAddr, Ipv4Addr};

use netdev::{get_default_interface, get_interfaces, Interface as NetdevInterface};

use crate::arp::ArpEntry;
use crate::error::DiscoveryError;
use mylan_core::{MacAddr, Observation};

/// Interfaz LAN resuelta lista para barrer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanInterface {
    /// Nombre del interfaz (p.ej. `enp37s0`).
    pub name: String,
    /// IPv4 del host en esa interfaz.
    pub ip: Ipv4Addr,
    /// Longitud de prefijo (CIDR), p.ej. `24`.
    pub prefix_len: u8,
    /// MAC del host (si se pudo leer).
    pub mac: Option<MacAddr>,
    /// IP del gateway (si se detectó).
    pub gateway_ip: Option<IpAddr>,
    /// MAC del gateway (si se detectó y no es cero).
    pub gateway_mac: Option<MacAddr>,
    /// Servidores DNS anunciados para la interfaz.
    pub dns_servers: Vec<IpAddr>,
    /// SSID de la red Wi-Fi conectada (si la interfaz es inalámbrica y se pudo
    /// leer). `None` en interfaces cableadas o cuando la detección no aplica.
    pub ssid: Option<String>,
}

impl LanInterface {
    /// Devuelve la red en notación CIDR (`a.b.c.d/p`), útil para `Network.cidr`.
    #[must_use]
    pub fn cidr(&self) -> String {
        format!("{}/{}", self.ip, self.prefix_len)
    }

    /// Construye una [`Observation`] del propio host (IP + MAC, sin origen) para que
    /// el agregador la descarte si no hay identidad útil.
    #[must_use]
    pub fn self_observation(&self) -> Observation {
        let mut obs = Observation::new(mylan_core::Source::ArpCache).with_ip(IpAddr::V4(self.ip));
        if let Some(mac) = self.mac {
            obs = obs.with_mac(mac);
        }
        obs
    }
}

/// Indica si un nombre de interfaz debe filtrarse (no es LAN útil).
fn is_filtered_name(name: &str) -> bool {
    matches!(
        name,
        "lo" | "docker0" | "tailscale0" | "tun0" | "wg0" | "virbr0" | "br0"
    ) || name.starts_with("docker")
        || name.starts_with("tailscale")
        || name.starts_with("tun")
        || name.starts_with("wg")
        || name.starts_with("virbr")
        || name.starts_with("br-")
}

/// Convierte una [`netdev::Interface`] en [`LanInterface`] si es utilizable.
fn from_netdev(iface: NetdevInterface) -> Result<LanInterface, DiscoveryError> {
    if is_filtered_name(&iface.name) {
        return Err(DiscoveryError::NoDefaultInterface);
    }
    let ipv4net = iface.ipv4.first().ok_or(DiscoveryError::NoIpv4 {
        name: iface.name.clone(),
    })?;
    let ip = ipv4net.addr();
    let prefix_len = ipv4net.prefix_len();
    if prefix_len > 32 {
        return Err(DiscoveryError::NoIpv4 { name: iface.name });
    }
    let mac = iface.mac_addr.map(|m| MacAddr::from_octets(m.octets()));
    let (gateway_ip, gateway_mac) = match iface.gateway.as_ref() {
        Some(gw) => (gw.ipv4.first().copied().map(IpAddr::V4), {
            let m = MacAddr::from_octets(gw.mac_addr.octets());
            if m.is_zero() {
                None
            } else {
                Some(m)
            }
        }),
        None => (None, None),
    };
    let dns_servers = iface.dns_servers.clone();
    Ok(LanInterface {
        name: iface.name,
        ip,
        prefix_len,
        mac,
        gateway_ip,
        gateway_mac,
        dns_servers,
        // El SSID se puebla en `detect_interface` para la interfaz resuelta;
        // `from_netdev` solo mapea los campos de `netdev`.
        ssid: None,
    })
}

/// Detecta la interfaz por defecto, o la indicada por `override_name`.
///
/// Con `override_name = None` usa `netdev::get_default_interface` (default-route). Con
/// `Some(name)` busca por nombre exacto entre todas las interfaces.
pub fn detect_interface(override_name: Option<&str>) -> Result<LanInterface, DiscoveryError> {
    let iface = match override_name {
        Some(name) => {
            let iface = get_interfaces()
                .into_iter()
                .find(|i| i.name == name)
                .ok_or_else(|| DiscoveryError::InterfaceNotFound(name.to_string()))?;
            from_netdev(iface)?
        }
        None => {
            let iface = get_default_interface().map_err(DiscoveryError::from)?;
            match from_netdev(iface) {
                // Si la default está filtrada, busca una alternativa con IPv4 no filtrada.
                Err(DiscoveryError::NoDefaultInterface) => pick_alternative()?,
                other => other?,
            }
        }
    };
    // Pobla el SSID de la interfaz resuelta (best-effort, pura-Rust por OS).
    Ok(with_ssid(iface))
}

/// Puebla `LanInterface.ssid` con el SSID detectado para esa interfaz (o `None`).
fn with_ssid(mut iface: LanInterface) -> LanInterface {
    iface.ssid = crate::ssid::detect_ssid(&iface);
    iface
}

/// Fallback: de las interfaces no filtradas con IPv4, elige la que tenga gateway.
fn pick_alternative() -> Result<LanInterface, DiscoveryError> {
    let mut candidates: Vec<NetdevInterface> = get_interfaces()
        .into_iter()
        .filter(|i| !is_filtered_name(&i.name) && !i.ipv4.is_empty())
        .collect();
    // Prefiere la que tenga gateway (default route real).
    candidates.sort_by_key(|i| i.gateway.is_some());
    let iface = candidates.pop().ok_or(DiscoveryError::NoDefaultInterface)?;
    from_netdev(iface)
}

/// Localiza la MAC del gateway resolviéndolo desde la tabla ARP del kernel.
///
/// Devuelve `None` si no se encuentra. Útil para anclar la identidad del router.
pub fn resolve_gateway_mac(gateway_ip: Option<IpAddr>, arp: &[ArpEntry]) -> Option<MacAddr> {
    let ip = gateway_ip?;
    arp.iter().find(|e| e.ip == ip).and_then(|e| e.mac)
}

/// Observations del propio gateway (IP + MAC si se conoce) para alimentar el
/// agregador y que el router aparezca en el inventario.
#[must_use]
pub fn gateway_observations(
    gateway_ip: Option<IpAddr>,
    gateway_mac: Option<MacAddr>,
) -> Vec<Observation> {
    let Some(ip) = gateway_ip else {
        return Vec::new();
    };
    let mut obs = Observation::new(mylan_core::Source::ArpCache).with_ip(ip);
    if let Some(mac) = gateway_mac {
        if !mac.is_zero() {
            obs = obs.with_mac(mac);
        }
    }
    vec![obs]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_known_non_lan_names() {
        assert!(is_filtered_name("lo"));
        assert!(is_filtered_name("docker0"));
        assert!(is_filtered_name("tailscale0"));
        assert!(is_filtered_name("docker123"));
        assert!(!is_filtered_name("enp37s0"));
        assert!(!is_filtered_name("eth0"));
        assert!(!is_filtered_name("wlan0"));
    }

    #[test]
    fn cidr_format() {
        let iface = LanInterface {
            name: "enp37s0".into(),
            ip: "192.168.1.3".parse().unwrap(),
            prefix_len: 24,
            mac: None,
            gateway_ip: None,
            gateway_mac: None,
            dns_servers: Vec::new(),
            ssid: None,
        };
        assert_eq!(iface.cidr(), "192.168.1.3/24");
    }

    #[test]
    fn resolve_gateway_mac_finds_match() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let mac = MacAddr::parse("aa:bb:cc:dd:ee:ff").unwrap();
        let arp = vec![ArpEntry {
            ip,
            mac: Some(mac),
            device: "enp37s0".into(),
        }];
        assert_eq!(resolve_gateway_mac(Some(ip), &arp), Some(mac));
        assert_eq!(resolve_gateway_mac(Some(ip), &[]), None);
        assert_eq!(resolve_gateway_mac(None, &arp), None);
    }

    #[test]
    fn gateway_observations_empty_without_ip() {
        assert!(gateway_observations(None, None).is_empty());
    }

    #[test]
    fn gateway_observations_with_ip_and_mac() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let mac = MacAddr::parse("aa:bb:cc:dd:ee:ff").unwrap();
        let obs = gateway_observations(Some(ip), Some(mac));
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].ip, Some(ip));
        assert_eq!(obs[0].mac, Some(mac));
    }
}
