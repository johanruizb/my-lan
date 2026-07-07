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
    #[serde(default)]
    pub is_online: bool,
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
            is_online: true,
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
///
/// `open_ports` lleva el conteo de puertos abiertos en un escaneo de puertos
/// (`scan_type = ports`); es 0 para descubrimiento. `#[serde(default)]` para que
/// los `summary_json` persistidos antes de v0.5.4 (sin el campo) deserialicen a
/// `0` y no rompan el historial existente.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanSummary {
    pub hosts_alive: u32,
    pub hosts_new: u32,
    pub duration_ms: u64,
    #[serde(default)]
    pub open_ports: u32,
}

/// Ejecución de un escaneo (descubrimiento o puertos).
///
/// `target_ip` fija la IP sondeada en un escaneo de puertos; `None` para
/// descubrimiento (escaneo de toda la red, sin target único). Se persiste en la
/// columna `target_ip` (v5) y alimenta el link `→ /devices/:ip` del historial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scan {
    pub id: String,
    pub network_id: String,
    pub target_ip: Option<String>,
    pub scan_type: ScanKind,
    pub profile: ScanProfile,
    pub status: ScanStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub summary: Option<ScanSummary>,
}

/// Evento del timeline de diferencias entre escaneos (v0.5 Watch, Step 1).
///
/// Producido por el motor de diff (`mylan-db::diff`) y persistido en la tabla
/// `events`; el canal WS `/events/live` es una vista en vivo del mismo flujo.
/// `serde` para export/JSON y broadcast. IDs como `String` (UUID) y timestamps
/// RFC3339, generados por `mylan-db`/`apps-cli`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub network_id: String,
    pub device_id: Option<String>,
    pub event_type: EventType,
    pub severity: Severity,
    pub message: Option<String>,
    pub data_json: Option<String>,
    pub created_at: String,
}

/// Tipo de evento del timeline de diferencias (AC-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Dispositivo visto por primera vez.
    DeviceNew,
    /// IP primaria cambiada (DHCP).
    DeviceIpChanged,
    /// Dispositivo antes online, no visto en este escaneo.
    DeviceOffline,
    /// Dispositivo antes offline, visto de nuevo.
    DeviceOnline,
    /// Servicio/puerto abierto desde el último escaneo.
    PortOpened,
}

/// Severidad de un [`Event`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

// ---------------------------------------------------------------------------
// Modelos de diagnóstico (Fase 3, Paso 5 — AC-6/7/8).
//
// Tipos puros producidos por `mylan ping|traceroute|dns` (fn en `mylan-discovery`).
// No se persisten: son salida de herramientas de diagnóstico. `serde` para
// futura export/JSON. Viven en `mylan-core` para que CLI y UI los compartan.
// ---------------------------------------------------------------------------

/// Método usado por `ping_host` para medir reachability.
///
/// Distinguir ICMP de TCP connect es parte del contrato (P4/AC-6): nunca se
/// presenta un fallback TCP como si fuera ICMP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PingMethod {
    /// Eco ICMP vía socket datagrama no-root (`SOCK_DGRAM` + `IPPROTO_ICMP`).
    Icmp,
    /// TCP connect a puertos comunes (fallback cuando ICMP no está disponible).
    TcpConnect,
}

/// Resultado de un `ping` a un host (AC-6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PingResult {
    /// Host sondeado.
    pub target: IpAddr,
    /// `true` si al menos un paquete obtuvo respuesta.
    pub reachable: bool,
    /// Latencia media (ms) de los paquetes respondidos.
    pub latency_ms: Option<u64>,
    /// Fracción de paquetes perdidos en `0.0..=1.0`.
    pub packet_loss: Option<f32>,
    /// Paquetes enviados.
    pub packets_sent: u32,
    /// Paquetes con respuesta.
    pub packets_received: u32,
    /// Método efectivamente usado ([`PingMethod`]).
    pub method: PingMethod,
}

/// Un salto de un `traceroute` (AC-7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceHop {
    /// Número de salto (TTL, base 1).
    pub hop_number: u8,
    /// IP del salto (ICMP time-exceeded sender) o `None` si no respondió.
    pub ip: Option<IpAddr>,
    /// Hostname vía reverse DNS (best-effort).
    pub hostname: Option<String>,
    /// Latencia (ms) del salto o `None` si no respondió.
    pub latency_ms: Option<u64>,
    /// Estado legible: `reached`, `time-exceeded`, `*`, `error`, ...
    pub state: String,
}

/// Un registro DNS resuelto (AC-8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsRecord {
    /// Nombre consultado.
    pub name: String,
    /// Tipo de registro (`A`, `AAAA`, `PTR`, `MX`, `TXT`).
    pub record_type: String,
    /// Valor del registro (IP, hostname, texto, ...).
    pub value: String,
    /// TTL en segundos.
    pub ttl: u32,
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
            target_ip: None,
            scan_type: ScanKind::Discovery,
            profile: ScanProfile::Quick,
            status: ScanStatus::Completed,
            started_at: "2026-06-27T00:00:00Z".to_string(),
            finished_at: Some("2026-06-27T00:00:20Z".to_string()),
            summary: Some(ScanSummary {
                hosts_alive: 12,
                hosts_new: 2,
                duration_ms: 18_500,
                open_ports: 0,
            }),
        };
        let json = serde_json::to_string(&scan).expect("serialize");
        let back: Scan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(scan, back);
    }

    #[test]
    fn scan_summary_default_open_ports_backward_compat() {
        // summary_json viejo (pre-v0.5.4) sin `open_ports` deserializa a 0:
        // el #[serde(default)] cubre el historial existente.
        let old_json = r#"{"hosts_alive":3,"hosts_new":1,"duration_ms":1200}"#;
        let summary: ScanSummary = serde_json::from_str(old_json).expect("deserialize");
        assert_eq!(summary.open_ports, 0);
        assert_eq!(summary.hosts_alive, 3);
    }

    #[test]
    fn ping_method_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&PingMethod::Icmp).expect("ser"),
            "\"icmp\""
        );
        assert_eq!(
            serde_json::to_string(&PingMethod::TcpConnect).expect("ser"),
            "\"tcp_connect\""
        );
    }

    #[test]
    fn ping_result_serde_round_trip() {
        let result = PingResult {
            target: ip("127.0.0.1"),
            reachable: true,
            latency_ms: Some(3),
            packet_loss: Some(0.0),
            packets_sent: 4,
            packets_received: 4,
            method: PingMethod::Icmp,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let back: PingResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, back);
    }

    #[test]
    fn trace_hop_serde_round_trip() {
        let hop = TraceHop {
            hop_number: 1,
            ip: Some(ip("127.0.0.1")),
            hostname: Some("localhost".to_string()),
            latency_ms: Some(0),
            state: "reached".to_string(),
        };
        let json = serde_json::to_string(&hop).expect("serialize");
        let back: TraceHop = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(hop, back);
    }

    #[test]
    fn dns_record_serde_round_trip() {
        let rec = DnsRecord {
            name: "localhost".to_string(),
            record_type: "A".to_string(),
            value: "127.0.0.1".to_string(),
            ttl: 300,
        };
        let json = serde_json::to_string(&rec).expect("serialize");
        let back: DnsRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(rec, back);
    }

    #[test]
    fn device_is_online_default_true() {
        let device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        assert!(device.is_online);
    }

    #[test]
    fn event_serde_round_trip() {
        let event = Event {
            id: "evt-1".to_string(),
            network_id: "net-1".to_string(),
            device_id: Some("dev-1".to_string()),
            event_type: EventType::DeviceNew,
            severity: Severity::Info,
            message: Some("New device discovered".to_string()),
            data_json: Some(r#"{"ip":"192.168.1.5"}"#.to_string()),
            created_at: "2026-07-03T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let back: Event = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    #[test]
    fn event_type_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&EventType::DeviceNew).expect("ser"),
            "\"device_new\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::DeviceIpChanged).expect("ser"),
            "\"device_ip_changed\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::DeviceOffline).expect("ser"),
            "\"device_offline\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::DeviceOnline).expect("ser"),
            "\"device_online\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::PortOpened).expect("ser"),
            "\"port_opened\""
        );
        assert_eq!(
            serde_json::to_string(&Severity::Info).expect("ser"),
            "\"info\""
        );
        assert_eq!(
            serde_json::to_string(&Severity::Warning).expect("ser"),
            "\"warning\""
        );
        assert_eq!(
            serde_json::to_string(&Severity::Critical).expect("ser"),
            "\"critical\""
        );
    }
}
