//! DTOs serializables que cruzan el IPC Tauri ↔ frontend React.
//!
//! Los tipos de `mylan-core` (`Device`, `Service`, `ServiceExportRow`,
//! `ScanProgress`) ya derivan `Serialize` y se emiten con sus nombres de campo
//! snake_case por defecto. Para mantener **una sola convención de nombres** en
//! toda la capa TS (sin tener que tocar los `#[derive]` de core), los DTOs
//! propios aquí **también** usan snake_case: el frontend `src/lib/tauri.ts`
//! espeja exactamente los nombres que emite serde. Documentado en
//! `src/lib/tauri.ts`.

use std::net::IpAddr;

use mylan_core::{Device, ScanProfile, Service};
use mylan_discovery::LanInterface;

/// Espejo serializable de [`LanInterface`]. `ip`/`mac`/`gateway_ip`/`gateway_mac`
/// se serializan como cadenas para no acoplar el frontend a `Ipv4Addr`/`MacAddr`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LanInterfaceDto {
    pub name: String,
    pub ip: String,
    pub prefix_len: u8,
    pub mac: Option<String>,
    pub gateway_ip: Option<String>,
    pub gateway_mac: Option<String>,
    pub dns_servers: Vec<String>,
    pub cidr: String,
}

impl LanInterfaceDto {
    pub fn from(iface: LanInterface) -> Self {
        let cidr = iface.cidr();
        Self {
            name: iface.name,
            ip: iface.ip.to_string(),
            prefix_len: iface.prefix_len,
            mac: iface.mac.map(|m| m.to_string()),
            gateway_ip: iface.gateway_ip.map(|i| i.to_string()),
            gateway_mac: iface.gateway_mac.map(|m| m.to_string()),
            dns_servers: iface
                .dns_servers
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            cidr,
        }
    }
}

/// Detalle de un dispositivo + sus servicios (comando `get_device`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceDetailDto {
    pub device: Device,
    pub services: Vec<Service>,
}

/// Resultado agregado de un escaneo de descubrimiento (comando `run_discovery`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanOutcomeDto {
    pub scan_id: String,
    pub network_id: String,
    pub hosts_alive: u32,
    pub hosts_new: u32,
    pub duration_ms: u64,
}

impl From<mylan_db::pipeline::ScanOutcome> for ScanOutcomeDto {
    fn from(o: mylan_db::pipeline::ScanOutcome) -> Self {
        Self {
            scan_id: o.scan_id,
            network_id: o.network_id,
            hosts_alive: o.hosts_alive,
            hosts_new: o.hosts_new,
            duration_ms: o.duration_ms,
        }
    }
}

/// Filtros para `list_services` (espejo de `ServiceFilters`; llega como JSON
/// desde el frontend). Los campos son opcionales y se combinan como AND.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ServiceFiltersDto {
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
}

/// Payload del evento `scan:heartbeat` (AC-8: barra suave de tiempo transcurrido
/// mientras `scan_target` no emite `on_progress` por puerto abierto).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanHeartbeat {
    pub scan_id: String,
    pub elapsed_ms: u64,
    pub scan_timeout_ms: u64,
}

/// Payload del evento `scan:cancelled` (cancelación por evento, no por `Result`
/// err — `scan_target` devuelve `Ok(partial)` al cancelar).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanCancelled {
    pub scan_id: String,
}

/// Payload del evento `scan:finished`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanFinished {
    pub scan_id: String,
}

/// Payload del evento `scan:started`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanStarted {
    pub scan_id: String,
    pub ip: Option<String>,
    pub profile: String,
}

/// Resumen de un escaneo para el historial de la pantalla Scans (AC-17 IPC
/// `list_scans`). Read-only: espejo de `mylan_db::scan_repo::ScanRow` con
/// nombres snake_case (convención IPC).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanSummaryDto {
    pub id: String,
    pub profile: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub hosts_alive: u32,
    pub hosts_new: u32,
}

/// Configuración persistida de la app (AC-9). Vive en
/// `app_data_dir/mylan-desktop.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    /// Path absoluto de la SQLite (resuelto en `setup`; informativo en la UI).
    pub db_path: String,
    /// Perfil de scan por defecto usado por Dashboard ("Escanear ahora").
    pub default_profile: String,
    /// Tema de la UI: `"light"` | `"dark"` (AC-3). Persistido y aplicado al
    /// arranque. `#[serde(default)]` para no romper settings antiguos sin el campo.
    #[serde(default)]
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            db_path: String::new(),
            default_profile: "normal".to_string(),
            theme: "light".to_string(),
        }
    }
}

/// Parsea el nombre de perfil serializado (snake_case) a `ScanProfile`.
/// Devuelve `Err(String)` si el nombre no coincide con ninguna variante.
pub fn parse_profile(s: &str) -> Result<ScanProfile, String> {
    match s {
        "quick" => Ok(ScanProfile::Quick),
        "normal" => Ok(ScanProfile::Normal),
        "deep" => Ok(ScanProfile::Deep),
        "iot" => Ok(ScanProfile::Iot),
        "router" => Ok(ScanProfile::Router),
        other => Err(format!(
            "perfil desconocido: '{other}' (usar quick|normal|deep|iot|router)"
        )),
    }
}

/// Parsea una cadena IP a `IpAddr` mapeando el error a `String` (para IPC).
pub fn parse_ip(s: &str) -> Result<IpAddr, String> {
    s.parse::<IpAddr>()
        .map_err(|e| format!("IP inválida '{s}': {e}"))
}
