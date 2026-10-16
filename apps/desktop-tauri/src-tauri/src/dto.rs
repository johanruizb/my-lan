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
    /// SSID de la red Wi-Fi conectada (`None` en cableado / no detectable). El
    /// DTO solo deriva `Serialize`, así que `#[serde(default)]` sería un no-op:
    /// se omite a propósito (a diferencia de `Settings`, que sí deserializa).
    pub ssid: Option<String>,
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
            ssid: iface.ssid,
        }
    }
}

/// Nombre de red + su origen (`auto`/`user`) para el comando `get_network_name`.
/// Alimenta el pie de la sidebar (nombre de red persistido por CIDR).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NetworkNameDto {
    pub name: String,
    pub source: String,
}

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

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanFinished {
    pub scan_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanStarted {
    pub scan_id: String,
    pub ip: Option<String>,
    pub profile: String,
}

/// Payload del evento `scan:discovery_progress` (AC-3/AC-4: % = IPs sondeadas /
/// total del CIDR). `total == 0` => barra indeterminada en el frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiscoveryProgress {
    pub scan_id: String,
    pub swept: u32,
    pub total: u32,
}

/// Payload del evento `scan:device` (AC-5/AC-6: cada host descubierto llega en vivo
/// como `Device` ya enriquecido y persistido). `device` es `mylan_core::Device`, que
/// ya serializa snake_case.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanDevice {
    pub scan_id: String,
    pub device: Device,
}

/// Resumen de un escaneo para el historial de la pantalla Scans (AC-17 IPC
/// `list_scans`). Read-only: espejo de `mylan_db::scan_repo::ScanRow` con
/// nombres snake_case (convención IPC).
///
/// `scan_type` (`"discovery"` | `"ports"`) distingue ambos caminos para que la
/// UI pueda renderizar el target y el contador de puertos; `target_ip` es la IP
/// sondeada en un scan de puertos (`None` en descubrimiento); `open_ports` es el
/// conteo de puertos abiertos persistido en `summary_json` (0 para descubrimiento).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanSummaryDto {
    pub id: String,
    pub scan_type: String,
    pub target_ip: Option<String>,
    pub profile: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub hosts_alive: u32,
    pub hosts_new: u32,
    pub open_ports: u32,
}

/// Valor por defecto para `Settings::censorship_enabled` en deserialización:
/// `true` (AC-6 — settings antiguos sin el campo migran a censura activada).
fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_monitoring_interval() -> u64 {
    300
}

/// Estado actual del monitoreo continuo en segundo plano emitido al frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitoringStatusDto {
    pub enabled: bool,
    pub paused: bool,
    pub is_scanning: bool,
    pub interval_secs: u64,
    pub last_scan_at: Option<String>,
    pub next_scan_in_secs: Option<u64>,
    pub current_network: Option<String>,
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
    /// Modo censura (AC-1..AC-7): enmascara identificadores estrictos en UI y
    /// exports. `#[serde(default = "default_true")]` para que settings antiguos
    /// sin el campo deserialicen a `true` (AC-6).
    #[serde(default = "default_true")]
    pub censorship_enabled: bool,
    /// Búsqueda automática de actualizaciones al arrancar la app.
    #[serde(default = "default_true")]
    pub check_updates_on_startup: bool,
    /// Monitoreo continuo en segundo plano activo.
    #[serde(default = "default_true")]
    pub background_monitoring_enabled: bool,
    /// Intervalo de monitoreo continuo en segundos (por defecto 300s / 5min).
    #[serde(default = "default_monitoring_interval")]
    pub monitoring_interval_secs: u64,
    /// Minimizar a la bandeja del sistema en lugar de cerrar la app al hacer clic en X.
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    /// Autoarranque con el sistema operativo.
    #[serde(default = "default_false")]
    pub autostart_enabled: bool,
    /// Notificaciones nativas de escritorio activas.
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    /// Notificar cuando se detecten dispositivos no confiables.
    #[serde(default = "default_true")]
    pub notify_untrusted_devices: bool,
    /// Notificar cambios de IP o MAC en dispositivos confiables.
    #[serde(default = "default_true")]
    pub notify_trusted_device_tampered: bool,
    /// Notificar caída o recuperación de Internet y Gateway.
    #[serde(default = "default_true")]
    pub notify_gateway_down: bool,
    /// Notificar puertos críticos abiertos recién detectados.
    #[serde(default = "default_true")]
    pub notify_critical_ports: bool,
    /// ID del workspace activo en la UI (Pilar 6 / Fase 10).
    #[serde(default = "default_active_workspace")]
    pub active_workspace_id: String,
    /// Sincronización Serverless con Google Drive activa (Pilar 7 / Fase 11).
    #[serde(default = "default_false")]
    pub gdrive_sync_enabled: bool,
    /// Intervalo de sincronización periódica en minutos (por defecto 30m).
    #[serde(default = "default_sync_interval")]
    pub gdrive_sync_interval_mins: u32,
    /// Client ID personalizado de Google OAuth (opcional).
    #[serde(default)]
    pub gdrive_client_id: Option<String>,
    /// Hash de verificación de la clave maestra Zero-Knowledge (para validación).
    #[serde(default)]
    pub gdrive_master_key_hash: Option<String>,
    /// Fecha/hora RFC3339 de la última sincronización exitosa.
    #[serde(default)]
    pub last_synced_at: Option<String>,
    /// Epoch UNIX de la última sincronización exitosa.
    #[serde(default)]
    pub last_synced_epoch: Option<i64>,
}

fn default_active_workspace() -> String {
    "default".to_string()
}

fn default_sync_interval() -> u32 {
    30
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            db_path: String::new(),
            default_profile: "normal".to_string(),
            theme: "light".to_string(),
            censorship_enabled: true,
            check_updates_on_startup: true,
            background_monitoring_enabled: true,
            monitoring_interval_secs: 300,
            minimize_to_tray: true,
            autostart_enabled: false,
            notifications_enabled: true,
            notify_untrusted_devices: true,
            notify_trusted_device_tampered: true,
            notify_gateway_down: true,
            notify_critical_ports: true,
            active_workspace_id: default_active_workspace(),
            gdrive_sync_enabled: false,
            gdrive_sync_interval_mins: 30,
            gdrive_client_id: None,
            gdrive_master_key_hash: None,
            last_synced_at: None,
            last_synced_epoch: None,
        }
    }
}

/// Estado reportado a la UI para Google Drive Sync.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GdriveSyncStatus {
    pub is_authenticated: bool,
    pub email: Option<String>,
    pub has_master_key: bool,
    pub last_synced_at: Option<String>,
    pub sync_enabled: bool,
}

/// Resultado de iniciar el flujo OAuth 2.0 PKCE.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StartGdriveAuthResult {
    pub auth_url: String,
    pub port: u16,
}

/// Resultado de configurar la clave maestra.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MasterKeySetupDto {
    pub verification_hash: String,
    pub mnemonic_phrase: String,
}

/// DTO para crear un workspace desde la interfaz.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateWorkspaceDto {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// DTO para actualizar un workspace desde la interfaz.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateWorkspaceDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub color: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_settings_deserialization_gets_correct_defaults() {
        // Simula un archivo JSON antiguo de v0.4 sin los campos de monitoreo continuo
        let legacy_json = r#"{
            "db_path": "/home/user/.local/share/mylan/mylan.db",
            "default_profile": "normal",
            "theme": "dark",
            "censorship_enabled": true
        }"#;

        let parsed: Settings = serde_json::from_str(legacy_json).expect("should deserialize");
        assert_eq!(parsed.db_path, "/home/user/.local/share/mylan/mylan.db");
        assert_eq!(parsed.default_profile, "normal");
        assert_eq!(parsed.theme, "dark");
        assert!(parsed.censorship_enabled);
        assert!(parsed.check_updates_on_startup);
        assert!(parsed.background_monitoring_enabled);
        assert_eq!(parsed.monitoring_interval_secs, 300);
        assert!(parsed.minimize_to_tray);
        assert!(!parsed.autostart_enabled);
        assert!(parsed.notifications_enabled);
        assert!(parsed.notify_untrusted_devices);
        assert!(parsed.notify_trusted_device_tampered);
        assert!(parsed.notify_gateway_down);
        assert!(parsed.notify_critical_ports);
    }

    #[test]
    fn test_custom_settings_roundtrip() {
        let original = Settings {
            db_path: "/tmp/custom.db".to_string(),
            default_profile: "quick".to_string(),
            theme: "light".to_string(),
            censorship_enabled: false,
            check_updates_on_startup: false,
            background_monitoring_enabled: false,
            monitoring_interval_secs: 600,
            minimize_to_tray: false,
            autostart_enabled: true,
            notifications_enabled: false,
            notify_untrusted_devices: false,
            notify_trusted_device_tampered: true,
            notify_gateway_down: false,
            notify_critical_ports: true,
            active_workspace_id: "custom-ws".to_string(),
            gdrive_sync_enabled: true,
            gdrive_sync_interval_mins: 15,
            gdrive_client_id: Some("custom-id".to_string()),
            gdrive_master_key_hash: Some("mock-hash".to_string()),
            last_synced_at: Some("2026-10-16T12:00:00Z".to_string()),
            last_synced_epoch: Some(12345678),
        };

        let json = serde_json::to_string(&original).expect("serialize");
        let restored: Settings = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.db_path, original.db_path);
        assert_eq!(restored.active_workspace_id, "custom-ws");
        assert!(restored.gdrive_sync_enabled);
        assert_eq!(restored.gdrive_sync_interval_mins, 15);
        assert_eq!(restored.gdrive_client_id.as_deref(), Some("custom-id"));
        assert!(!restored.background_monitoring_enabled);
        assert_eq!(restored.monitoring_interval_secs, 600);
        assert!(!restored.minimize_to_tray);
        assert!(restored.autostart_enabled);
        assert!(!restored.notifications_enabled);
        assert!(!restored.notify_untrusted_devices);
        assert!(restored.notify_trusted_device_tampered);
        assert!(!restored.notify_gateway_down);
        assert!(restored.notify_critical_ports);
    }
}
