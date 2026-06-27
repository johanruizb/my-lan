//! `Observation`: resultado normalizado de cualquier técnica de descubrimiento.
//!
//! Cada técnica de `mylan-discovery` (ARP cache, TCP-ping, mDNS, SSDP, ICMP, ARP
//! sweep) emite `Observation`s crudas. El agregador las deduplica por identidad
//! estable y las fusiona; el fingerprint (Paso 6) las interpreta vía la fase de
//! enrichment. Dominio puro: sin I/O de plataforma.

use std::collections::BTreeMap;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::identity::DeviceIdentity;
use crate::mac::MacAddr;

/// Técnica de descubrimiento que originó una [`Observation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Lectura de la caché ARP del kernel (`/proc/net/arp`).
    ArpCache,
    /// Barrido ARP activo a nivel L2 (requiere `CAP_NET_RAW`).
    ArpSweep,
    /// Sonda TCP-connect (liveness sin privilegios).
    TcpPing,
    /// Eco ICMP (best-effort sin root, raw con sudo).
    Icmp,
    /// Anuncio/respuesta mDNS (`mdns-sd`).
    Mdns,
    /// Respuesta SSDP/UPnP a `M-SEARCH`.
    Ssdp,
}

/// Observación normalizada de un host por una técnica concreta.
///
/// Los campos opcionales reflejan que cada técnica aporta un subconjunto de la
/// información. `hints` lleva señales crudas adicionales (p.ej. tipos de servicio
/// mDNS, `ST` de SSDP, puertos sonda) con claves namespaced para que el
/// fingerprint las interprete sin acoplar el dominio a un formato concreto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub ip: Option<IpAddr>,
    pub mac: Option<MacAddr>,
    pub hostname: Option<String>,
    pub source: Source,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hints: BTreeMap<String, String>,
}

impl Observation {
    /// Crea una observación mínima con solo el origen.
    #[must_use]
    pub fn new(source: Source) -> Self {
        Self {
            ip: None,
            mac: None,
            hostname: None,
            source,
            hints: BTreeMap::new(),
        }
    }

    /// Builder: fija la IP.
    #[must_use]
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.ip = Some(ip);
        self
    }

    /// Builder: fija la MAC.
    #[must_use]
    pub fn with_mac(mut self, mac: MacAddr) -> Self {
        self.mac = Some(mac);
        self
    }

    /// Builder: fija el hostname.
    #[must_use]
    pub fn with_hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = Some(hostname.into());
        self
    }

    /// Builder: añade un hint namespaced.
    #[must_use]
    pub fn with_hint(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.hints.insert(key.into(), value.into());
        self
    }

    /// Identidad estable de esta observación (MAC no-cero > IP).
    #[must_use]
    pub fn identity(&self) -> Option<DeviceIdentity> {
        DeviceIdentity::derive(self.mac, self.ip)
    }

    /// Fusiona `other` sobre `self` (unión de campos).
    ///
    /// Los campos ya presentes en `self` se conservan (precedencia del primero);
    /// los ausentes se rellenan desde `other`. Los `hints` se unen sin pisar
    /// claves existentes. `source` se mantiene como el de `self`.
    pub fn merge_from(&mut self, other: &Observation) {
        if self.ip.is_none() {
            self.ip = other.ip;
        }
        if self.mac.is_none() {
            self.mac = other.mac;
        }
        if self.hostname.is_none() {
            self.hostname.clone_from(&other.hostname);
        }
        for (key, value) in &other.hints {
            self.hints
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
    }
}

/// Agrega observaciones crudas en una por identidad estable (dedup + merge).
///
/// Preserva el orden de primera aparición de cada identidad para resultados
/// deterministas. Las observaciones sin identidad utilizable se descartan: no
/// pueden mapearse a un dispositivo.
///
/// Reconciliación IP↔MAC: una observación solo-IP (p.ej. la semilla del gateway
/// sin MAC, o un TCP-ping) se funde con el host cuya MAC fue vista junto a esa
/// misma IP por otra técnica (relectura ARP). Sin esto, el mismo host aparecería
/// dos veces —una por `Ip(x)` y otra por `Mac(..)`— violando el determinismo de
/// inventario (P5). En una LAN, una IP en un instante mapea a un único host, así
/// que la fusión es correcta.
#[must_use]
pub fn aggregate(observations: &[Observation]) -> Vec<Observation> {
    // Mapa IP → identidad-MAC para las IPs que coocurren con una MAC utilizable.
    let mut ip_to_mac: BTreeMap<IpAddr, DeviceIdentity> = BTreeMap::new();
    for obs in observations {
        if let (Some(ip), Some(id @ DeviceIdentity::Mac(_))) = (obs.ip, obs.identity()) {
            ip_to_mac.entry(ip).or_insert(id);
        }
    }

    let mut order: Vec<DeviceIdentity> = Vec::new();
    let mut merged: BTreeMap<DeviceIdentity, Observation> = BTreeMap::new();
    for obs in observations {
        let Some(mut id) = obs.identity() else {
            continue;
        };
        // Funde una identidad solo-IP en el host-MAC que posee esa IP.
        if let DeviceIdentity::Ip(ip) = id {
            if let Some(mac_id) = ip_to_mac.get(&ip) {
                id = *mac_id;
            }
        }
        match merged.get_mut(&id) {
            Some(existing) => existing.merge_from(obs),
            None => {
                order.push(id);
                merged.insert(id, obs.clone());
            }
        }
    }
    order
        .into_iter()
        .filter_map(|id| merged.remove(&id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("valid ip")
    }

    fn mac(s: &str) -> MacAddr {
        MacAddr::parse(s).expect("valid mac")
    }

    #[test]
    fn identity_prefers_mac() {
        let obs = Observation::new(Source::ArpCache)
            .with_mac(mac("aa:bb:cc:dd:ee:ff"))
            .with_ip(ip("192.168.1.5"));
        assert_eq!(
            obs.identity(),
            Some(DeviceIdentity::Mac(mac("aa:bb:cc:dd:ee:ff")))
        );
    }

    #[test]
    fn merge_fills_missing_fields_and_unions_hints() {
        let mut base = Observation::new(Source::ArpCache)
            .with_mac(mac("aa:bb:cc:dd:ee:ff"))
            .with_ip(ip("192.168.1.5"))
            .with_hint("tcp.ports", "80");
        let other = Observation::new(Source::Mdns)
            .with_ip(ip("192.168.1.5"))
            .with_hostname("printer.local")
            .with_hint("mdns.service", "_ipp._tcp")
            .with_hint("tcp.ports", "443"); // existing key not overwritten
        base.merge_from(&other);
        assert_eq!(base.hostname.as_deref(), Some("printer.local"));
        assert_eq!(
            base.hints.get("mdns.service").map(String::as_str),
            Some("_ipp._tcp")
        );
        assert_eq!(base.hints.get("tcp.ports").map(String::as_str), Some("80"));
        assert_eq!(base.source, Source::ArpCache); // self source preserved
    }

    #[test]
    fn aggregate_dedups_by_identity_preserving_order() {
        let observations = vec![
            // Same host seen by ARP (mac+ip) and mDNS (ip+hostname) -> one device.
            Observation::new(Source::ArpCache)
                .with_mac(mac("aa:bb:cc:dd:ee:ff"))
                .with_ip(ip("192.168.1.5")),
            Observation::new(Source::TcpPing).with_ip(ip("192.168.1.9")),
            Observation::new(Source::Mdns)
                .with_mac(mac("aa:bb:cc:dd:ee:ff"))
                .with_hostname("nas.local"),
            // No identity -> dropped.
            Observation::new(Source::Ssdp).with_hint("ssdp.st", "upnp:rootdevice"),
        ];
        let result = aggregate(&observations);
        assert_eq!(result.len(), 2);
        // First identity is the MAC host, merged with hostname from mDNS.
        assert_eq!(
            result[0].identity(),
            Some(DeviceIdentity::Mac(mac("aa:bb:cc:dd:ee:ff")))
        );
        assert_eq!(result[0].hostname.as_deref(), Some("nas.local"));
        assert_eq!(
            result[1].identity(),
            Some(DeviceIdentity::Ip(ip("192.168.1.9")))
        );
    }

    #[test]
    fn aggregate_folds_ip_only_into_mac_host() {
        // Escenario real del gateway: la semilla solo-IP (router sin MAC conocida)
        // y la relectura ARP (misma IP, con MAC) son el MISMO host. Deben fundirse
        // en un único device, no aparecer dos veces (P5).
        let gw: IpAddr = ip("192.168.1.1");
        let gw_mac = mac("02:00:5e:10:00:01"); // MAC sintética (no de red real)
        let observations = vec![
            Observation::new(Source::Mdns).with_ip(gw), // semilla gateway solo-IP
            Observation::new(Source::ArpCache)
                .with_mac(gw_mac)
                .with_ip(gw),
        ];
        let result = aggregate(&observations);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].identity(), Some(DeviceIdentity::Mac(gw_mac)));
        assert_eq!(result[0].ip, Some(gw));
    }

    #[test]
    fn serde_round_trip() {
        let obs = Observation::new(Source::Mdns)
            .with_ip(ip("192.168.1.5"))
            .with_mac(mac("aa:bb:cc:dd:ee:ff"))
            .with_hostname("nas.local")
            .with_hint("mdns.service", "_smb._tcp");
        let json = serde_json::to_string(&obs).expect("serialize");
        let back: Observation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(obs, back);
    }
}
