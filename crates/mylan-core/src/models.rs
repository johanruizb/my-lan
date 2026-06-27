//! Modelos de dominio de MyLAN (espejo del esquema DB del plan §8).
//!
//! Tipos puros con `serde`; sin I/O de plataforma (P3). IDs y timestamps se
//! representan como `String` (las columnas DB son `TEXT`): los IDs son UUID y los
//! timestamps RFC3339, generados por las capas `mylan-db`/`apps-cli`.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::confidence::Confidence;
use crate::enums::{DeviceType, Protocol, ScanKind, ScanProfile, ScanStatus, ServiceState};
use crate::identity::DeviceIdentity;
use crate::mac::MacAddr;
use crate::observation::Observation;

/// Red LAN descubierta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    pub id: String,
    pub name: String,
    pub cidr: String,
    pub gateway_ip: Option<IpAddr>,
    #[serde(default)]
    pub dns_servers: Vec<IpAddr>,
    pub created_at: String,
    pub updated_at: String,
}

/// Interfaz de red del host (origen de una [`Network`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    pub ip: Option<IpAddr>,
    pub mac: Option<MacAddr>,
    pub is_default_route: bool,
}

/// Dispositivo del inventario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub network_id: String,
    pub primary_mac: Option<MacAddr>,
    pub primary_ip: Option<IpAddr>,
    pub hostname: Option<String>,
    pub display_name: Option<String>,
    pub vendor: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub device_type: DeviceType,
    pub os_family: Option<String>,
    #[serde(default)]
    pub confidence: Confidence,
    pub first_seen_at: String,
    pub last_seen_at: String,
    #[serde(default)]
    pub is_trusted: bool,
    #[serde(default)]
    pub is_hidden: bool,
    pub notes: Option<String>,
}

impl Device {
    /// Crea un dispositivo vacío con identidad temporal y timestamps iniciales.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        network_id: impl Into<String>,
        now: impl Into<String>,
    ) -> Self {
        let now = now.into();
        Self {
            id: id.into(),
            network_id: network_id.into(),
            primary_mac: None,
            primary_ip: None,
            hostname: None,
            display_name: None,
            vendor: None,
            manufacturer: None,
            model: None,
            device_type: DeviceType::default(),
            os_family: None,
            confidence: Confidence::default(),
            first_seen_at: now.clone(),
            last_seen_at: now,
            is_trusted: false,
            is_hidden: false,
            notes: None,
        }
    }

    /// Identidad estable para upsert (MAC no-cero > IP); ver [`DeviceIdentity`].
    #[must_use]
    pub fn identity(&self) -> Option<DeviceIdentity> {
        DeviceIdentity::derive(self.primary_mac, self.primary_ip)
    }

    /// Aplica una observación (ya agregada) sobre el dispositivo.
    ///
    /// La MAC primaria se fija solo si falta (ancla de identidad estable); la IP
    /// se actualiza a la más reciente observada (DHCP); el hostname se rellena si
    /// falta. `seen_at` actualiza `last_seen_at`.
    pub fn merge_observation(&mut self, obs: &Observation, seen_at: impl Into<String>) {
        if self.primary_mac.is_none() {
            if let Some(mac) = obs.mac.filter(|m| !m.is_zero()) {
                self.primary_mac = Some(mac);
            }
        }
        if let Some(ip) = obs.ip {
            self.primary_ip = Some(ip);
        }
        if self.hostname.is_none() {
            self.hostname.clone_from(&obs.hostname);
        }
        self.last_seen_at = seen_at.into();
    }

    /// Aplica una clasificación de fingerprint respetando precedencia de
    /// confianza: solo sustituye `device_type`/`confidence` si la nueva confianza
    /// es mayor o igual a la actual. Devuelve `true` si hubo cambio.
    pub fn apply_classification(
        &mut self,
        device_type: DeviceType,
        confidence: Confidence,
    ) -> bool {
        if confidence >= self.confidence {
            self.device_type = device_type;
            self.confidence = confidence;
            true
        } else {
            false
        }
    }
}

/// Dirección (IP/MAC) histórica asociada a un [`Device`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceAddress {
    pub id: String,
    pub device_id: String,
    pub ip: Option<IpAddr>,
    pub mac: Option<MacAddr>,
    pub interface_name: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

/// Servicio/puerto detectado en un [`Device`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub device_id: String,
    pub protocol: Protocol,
    pub port: u16,
    pub service_name: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub banner: Option<String>,
    pub state: ServiceState,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

/// Resumen agregado de un escaneo (se persiste como `summary_json`).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanSummary {
    pub hosts_alive: u32,
    pub hosts_new: u32,
    pub duration_ms: u64,
}

/// Ejecución de un escaneo (descubrimiento o puertos).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scan {
    pub id: String,
    pub network_id: String,
    pub scan_type: ScanKind,
    pub profile: ScanProfile,
    pub status: ScanStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub summary: Option<ScanSummary>,
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
    fn merge_observation_anchors_mac_updates_ip_fills_hostname() {
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        let first = Observation::new(crate::observation::Source::ArpCache)
            .with_mac(mac("aa:bb:cc:dd:ee:ff"))
            .with_ip(ip("192.168.1.5"));
        device.merge_observation(&first, "2026-06-27T00:00:01Z");
        assert_eq!(device.primary_mac, Some(mac("aa:bb:cc:dd:ee:ff")));
        assert_eq!(device.primary_ip, Some(ip("192.168.1.5")));
        assert_eq!(device.last_seen_at, "2026-06-27T00:00:01Z");

        // New scan: same MAC, IP changed (DHCP), hostname learned.
        let second = Observation::new(crate::observation::Source::Mdns)
            .with_mac(mac("aa:bb:cc:dd:ee:ff"))
            .with_ip(ip("192.168.1.42"))
            .with_hostname("nas.local");
        device.merge_observation(&second, "2026-06-27T01:00:00Z");
        assert_eq!(device.primary_mac, Some(mac("aa:bb:cc:dd:ee:ff"))); // unchanged anchor
        assert_eq!(device.primary_ip, Some(ip("192.168.1.42"))); // latest wins
        assert_eq!(device.hostname.as_deref(), Some("nas.local"));
        assert_eq!(device.first_seen_at, "2026-06-27T00:00:00Z"); // preserved
    }

    #[test]
    fn classification_respects_confidence_precedence() {
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        assert!(device.apply_classification(DeviceType::Camera, Confidence::new(75)));
        assert_eq!(device.device_type, DeviceType::Camera);

        // Lower confidence is ignored.
        assert!(!device.apply_classification(DeviceType::Iot, Confidence::new(40)));
        assert_eq!(device.device_type, DeviceType::Camera);
        assert_eq!(device.confidence, Confidence::new(75));

        // Higher confidence wins.
        assert!(device.apply_classification(DeviceType::Nas, Confidence::new(90)));
        assert_eq!(device.device_type, DeviceType::Nas);
        assert_eq!(device.confidence, Confidence::new(90));
    }

    #[test]
    fn device_identity_uses_mac_then_ip() {
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        assert!(device.identity().is_none());
        device.primary_ip = Some(ip("192.168.1.5"));
        assert_eq!(
            device.identity(),
            Some(DeviceIdentity::Ip(ip("192.168.1.5")))
        );
        device.primary_mac = Some(mac("aa:bb:cc:dd:ee:ff"));
        assert_eq!(
            device.identity(),
            Some(DeviceIdentity::Mac(mac("aa:bb:cc:dd:ee:ff")))
        );
    }

    #[test]
    fn device_serde_round_trip() {
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        device.primary_mac = Some(mac("aa:bb:cc:dd:ee:ff"));
        device.primary_ip = Some(ip("192.168.1.5"));
        device.hostname = Some("nas.local".to_string());
        device.vendor = Some("Example Vendor".to_string());
        device.apply_classification(DeviceType::Nas, Confidence::new(82));
        let json = serde_json::to_string(&device).expect("serialize");
        let back: Device = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(device, back);
    }

    #[test]
    fn scan_serde_round_trip() {
        let scan = Scan {
            id: "scan-1".to_string(),
            network_id: "net-1".to_string(),
            scan_type: ScanKind::Discovery,
            profile: ScanProfile::Quick,
            status: ScanStatus::Completed,
            started_at: "2026-06-27T00:00:00Z".to_string(),
            finished_at: Some("2026-06-27T00:00:20Z".to_string()),
            summary: Some(ScanSummary {
                hosts_alive: 12,
                hosts_new: 2,
                duration_ms: 18_500,
            }),
        };
        let json = serde_json::to_string(&scan).expect("serialize");
        let back: Scan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(scan, back);
    }
}
