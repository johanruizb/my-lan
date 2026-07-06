//! Enumeraciones de dominio (valores cerrados) compartidas por los modelos.
//!
//! Todas serializan en `snake_case` para mapear de forma estable a las columnas
//! `TEXT` de la DB (plan §8) y a la salida JSON/CSV.

use serde::{Deserialize, Serialize};

/// Protocolo de transporte de un [`crate::Service`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Tcp,
    Udp,
}

/// Estado observado de un puerto/servicio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Open,
    Closed,
    Filtered,
}

/// Tipo probable de dispositivo (resultado del fingerprint, AC-10).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Router,
    Phone,
    Laptop,
    Desktop,
    Tv,
    Printer,
    Camera,
    Nas,
    Console,
    Iot,
    Tablet,
    #[default]
    Unknown,
}

/// Perfil de profundidad de un escaneo (plan §7.3).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanProfile {
    #[default]
    Quick,
    Normal,
    Deep,
    /// Perfil IoT: catálogo fijo (RTSP/ONVIF, MQTT, CoAP, UPnP, TR-069).
    Iot,
    /// Perfil router: catálogo fijo (admin/SSH/Telnet/DNS/DHCP/UPnP/TR-069).
    Router,
}

/// Naturaleza de un escaneo: descubrimiento de hosts vs. escaneo de puertos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanKind {
    Discovery,
    Ports,
}

/// Estado del ciclo de vida de un [`crate::Scan`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    #[default]
    Running,
    Completed,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stable() {
        assert_eq!(DeviceType::default(), DeviceType::Unknown);
        assert_eq!(ScanProfile::default(), ScanProfile::Quick);
        assert_eq!(ScanStatus::default(), ScanStatus::Running);
    }

    #[test]
    fn serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&DeviceType::Router).expect("ser"),
            "\"router\""
        );
        assert_eq!(
            serde_json::to_string(&Protocol::Tcp).expect("ser"),
            "\"tcp\""
        );
        assert_eq!(
            serde_json::to_string(&ScanKind::Discovery).expect("ser"),
            "\"discovery\""
        );
    }

    #[test]
    fn round_trips_device_type() {
        for variant in [
            DeviceType::Router,
            DeviceType::Phone,
            DeviceType::Camera,
            DeviceType::Iot,
            DeviceType::Tablet,
            DeviceType::Unknown,
        ] {
            let json = serde_json::to_string(&variant).expect("ser");
            let back: DeviceType = serde_json::from_str(&json).expect("de");
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn scan_profile_serializes_iot_router() {
        assert_eq!(
            serde_json::to_string(&ScanProfile::Iot).expect("ser"),
            "\"iot\""
        );
        assert_eq!(
            serde_json::to_string(&ScanProfile::Router).expect("ser"),
            "\"router\""
        );
    }

    #[test]
    fn round_trips_scan_profile() {
        for variant in [
            ScanProfile::Quick,
            ScanProfile::Normal,
            ScanProfile::Deep,
            ScanProfile::Iot,
            ScanProfile::Router,
        ] {
            let json = serde_json::to_string(&variant).expect("ser");
            let back: ScanProfile = serde_json::from_str(&json).expect("de");
            assert_eq!(variant, back);
        }
    }
}
