//! Comandos IPC `#[tauri::command]` — wrappers thin sobre las APIs `pub` de
//! `mylan-discovery` / `mylan-scanner` / `mylan-fingerprint` / `mylan-db`.
//!
//! Cada comando mapea errores de core/db a `String` (contrato IPC) y devuelve
//! DTOs serializables (snake_case — ver `dto.rs`). El work sync pesado
//! (pipeline, upserts, export) se envuelve en `tokio::task::spawn_blocking`
//! sobre un `try_clone` de la `Connection` para no stallear el reactor mientras
//! un `scan_ports` concurrente emite `scan:progress` (AC-12).

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use mylan_core::{Enricher, Network, Service};
use mylan_db::service_repo::ServiceFilters;
use mylan_discovery::{detect_interface, discover, DiscoverOptions};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::dto::{
    parse_ip, parse_profile, DeviceDetailDto, LanInterfaceDto, ScanCancelled, ScanFinished,
    ScanHeartbeat, ScanOutcomeDto, ScanStarted, ScanSummaryDto, ServiceFiltersDto, Settings,
};
use crate::state::DesktopState;

// --- DB helpers ------------------------------------------------------------

/// Replica la query `latest_network_id` del CLI (`apps/cli/src/commands/mod.rs`):
/// el `network_id` del escaneo más reciente. `None` si el inventario está vacío.
fn latest_network_id(conn: &rusqlite::Connection) -> Result<Option<String>, String> {
    let result = conn.query_row(
        "SELECT network_id FROM scans ORDER BY started_at DESC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Abre una `Connection` independiente para usarla dentro de `spawn_blocking`.
///
/// `rusqlite` 0.40.1 no expone `try_clone`; reabrir el fichero vía
/// `mylan_db::connection::connect` es equivalente en efecto: WAL + `busy_timeout`
/// se reaplican (`setup`) y las migraciones son idempotentes. La conexión
/// principal del `Mutex` queda disponible para reads concurrentes (AC-12).
fn clone_for_blocking(state: &State<'_, DesktopState>) -> Result<rusqlite::Connection, String> {
    let path = state.db_path.clone();
    mylan_db::connection::connect(&path).map_err(|e| e.to_string())
}

/// Resuelve el directorio de fingerprints. En dev cae al `signatures/` del repo
/// (compilado via `CARGO_MANIFEST_DIR`); empaquetado, al `signatures/` del
/// `resource_dir` de Tauri. Si ninguno existe, `Fingerprint::load` fallará y el
/// llamador degrada a `noop_enricher`.
fn resolve_signatures_dir(app: &AppHandle) -> PathBuf {
    if let Ok(res) = app.path().resource_dir() {
        let candidate = res.join("signatures");
        if candidate.is_dir() {
            return candidate;
        }
    }
    // Dev fallback: repo-root/signatures relativo al crate del backend.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../signatures")
}

/// Construye el `Enricher` de fingerprint, degradando a no-op si falla la carga
/// (mismo patrón que `apps/cli/src/commands/scan.rs::build_enricher`).
fn build_enricher(signatures_dir: &Path, gateway_ip: Option<IpAddr>) -> Enricher {
    match mylan_fingerprint::Fingerprint::load(signatures_dir, gateway_ip) {
        Ok(fp) => fp.enricher(),
        Err(_) => mylan_core::noop_enricher(),
    }
}

/// Rellena los campos de persistencia de un `Service` (patrón de
/// `apps/cli/src/commands/ports.rs::fill_service`).
fn fill_service(svc: &Service, device_id: &str, now: &str) -> Service {
    Service {
        id: mylan_db::util::new_id(),
        device_id: device_id.to_string(),
        protocol: svc.protocol,
        port: svc.port,
        service_name: svc.service_name.clone(),
        product: svc.product.clone(),
        version: svc.version.clone(),
        banner: svc.banner.clone(),
        state: svc.state,
        first_seen_at: now.to_string(),
        last_seen_at: now.to_string(),
    }
}

// --- Comandos: interfaz / lectura ------------------------------------------

#[tauri::command]
pub fn detect_interface_cmd(interface: Option<String>) -> Result<LanInterfaceDto, String> {
    let iface = detect_interface(interface.as_deref()).map_err(|e| e.to_string())?;
    Ok(LanInterfaceDto::from(iface))
}

#[tauri::command]
pub fn list_devices_cmd(state: State<'_, DesktopState>) -> Result<Vec<mylan_core::Device>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let net_id = latest_network_id(&conn)?
        .ok_or_else(|| "No hay inventario; ejecuta un escaneo primero.".to_string())?;
    mylan_db::device_repo::list_devices(&conn, &net_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_device_cmd(
    ip: String,
    state: State<'_, DesktopState>,
) -> Result<DeviceDetailDto, String> {
    let ip_addr = parse_ip(&ip)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let net_id = latest_network_id(&conn)?
        .ok_or_else(|| "No hay inventario; ejecuta un escaneo primero.".to_string())?;
    let device = mylan_db::device_repo::get_device_by_ip(&conn, &net_id, ip_addr)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("No se encontró un dispositivo con IP {ip}."))?;
    let services = mylan_db::service_repo::list_services_by_device(&conn, &device.id)
        .map_err(|e| e.to_string())?;
    Ok(DeviceDetailDto { device, services })
}

#[tauri::command]
pub fn list_services_cmd(
    filters: ServiceFiltersDto,
    state: State<'_, DesktopState>,
) -> Result<Vec<mylan_db::service_repo::ServiceExportRow>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let f = ServiceFilters {
        device_id: filters.device_id,
        port: filters.port,
        protocol: filters.protocol,
        service: filters.service,
    };
    mylan_db::service_repo::list_services(&conn, &f).map_err(|e| e.to_string())
}

/// Lista el historial de escaneos (AC-17, excepción backend read-only).
///
/// Lee la tabla `scans` ordenada por `started_at` desc vía
/// `mylan_db::scan_repo::list_scans` (read-only: sólo `SELECT`). Devuelve
/// `Vec<ScanSummaryDto>` para alimentar la pantalla de historial de Scans.
#[tauri::command]
pub fn list_scans_cmd(state: State<'_, DesktopState>) -> Result<Vec<ScanSummaryDto>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let rows = mylan_db::scan_repo::list_scans(&conn).map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| ScanSummaryDto {
            id: r.id,
            profile: r.profile,
            status: r.status,
            started_at: r.started_at,
            finished_at: r.finished_at,
            hosts_alive: r.hosts_alive,
            hosts_new: r.hosts_new,
        })
        .collect())
}

// --- Comando: run_discovery (pipeline en spawn_blocking) -------------------

#[tauri::command]
pub async fn run_discovery_cmd(
    profile: String,
    interface: Option<String>,
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<ScanOutcomeDto, String> {
    let scan_profile = parse_profile(&profile)?;
    let iface = detect_interface(interface.as_deref()).map_err(|e| e.to_string())?;

    let opts = DiscoverOptions {
        profile: scan_profile,
        interface: interface.clone(),
        ..DiscoverOptions::for_profile(scan_profile)
    };
    let now = mylan_db::util::now_rfc3339().map_err(|e| e.to_string())?;
    let network = Network {
        id: iface.cidr(),
        name: iface.cidr(),
        cidr: iface.cidr(),
        gateway_ip: iface.gateway_ip,
        dns_servers: iface.dns_servers.clone(),
        created_at: now.clone(),
        updated_at: now,
    };

    let _ = app.emit(
        "scan:started",
        &ScanStarted {
            scan_id: String::new(),
            ip: None,
            profile: profile.clone(),
        },
    );

    // Fase liveness (async, ≤30s) — fuera del blocking pool.
    let observations = discover(&iface, &opts).await;

    // Fase enrich+persist (sync, transaction+upserts) en el blocking pool sobre
    // un clone de la conexión: la principal queda disponible para reads
    // concurrentes (AC-12).
    let signatures_dir = resolve_signatures_dir(&app);
    let enricher = build_enricher(&signatures_dir, iface.gateway_ip);
    let conn = clone_for_blocking(&state)?;

    let outcome = tokio::task::spawn_blocking(move || {
        mylan_db::pipeline::run_scan_pipeline(
            &conn,
            &network,
            &observations,
            &enricher,
            scan_profile,
        )
    })
    .await
    .map_err(|e| format!("pipeline join: {e}"))?
    .map_err(|e| e.to_string())?;

    let dto = ScanOutcomeDto::from(outcome);
    let _ = app.emit(
        "scan:finished",
        &ScanFinished {
            scan_id: dto.scan_id.clone(),
        },
    );
    Ok(dto)
}

// --- Comando: scan_ports (async + heartbeat + cancel) ----------------------

#[tauri::command]
pub async fn scan_ports_cmd(
    ip: String,
    profile: String,
    scan_id: String,
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<Vec<Service>, String> {
    let ip_addr = parse_ip(&ip)?;
    let scan_profile = parse_profile(&profile)?;
    let options = mylan_scanner::profile_options(scan_profile);
    let scan_timeout_ms = options.scan_timeout.as_millis().max(1) as u64;

    let cancel = tokio_util::sync::CancellationToken::new();
    state
        .scan_tokens
        .lock()
        .map_err(|e| e.to_string())?
        .insert(scan_id.clone(), cancel.clone());

    let _ = app.emit(
        "scan:started",
        &ScanStarted {
            scan_id: scan_id.clone(),
            ip: Some(ip.clone()),
            profile: profile.clone(),
        },
    );

    let started = Instant::now();
    let app_for_heartbeat = app.clone();
    let scan_id_hb = scan_id.clone();
    let timeout = options.scan_timeout;
    // Heartbeat (AC-8): `scan_target` solo emite `on_progress` en puertos
    // abiertos; este intervalo de 500ms emite `scan:heartbeat` para mantener la
    // barra de tiempo viva contra hosts con 0-1 puertos abiertos.
    let heartbeat_handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(500));
        loop {
            ticker.tick().await;
            let elapsed = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
            let _ = app_for_heartbeat.emit(
                "scan:heartbeat",
                &ScanHeartbeat {
                    scan_id: scan_id_hb.clone(),
                    elapsed_ms: elapsed,
                    scan_timeout_ms,
                },
            );
        }
    });

    // on_progress captura SOLO `app` (Clone+Send+Sync) — no `State` (Send).
    let app_for_progress = app.clone();
    let services = mylan_scanner::scan_target(
        ip_addr,
        scan_profile,
        options,
        cancel,
        move |p: mylan_scanner::ScanProgress| {
            let _ = app_for_progress.emit("scan:progress", &p);
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    heartbeat_handle.abort();

    // Limpieza del token (sin zombies — AC-12).
    state
        .scan_tokens
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&scan_id);

    // Persistencia (sync) en el blocking pool sobre un clone.
    let device_id = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let net_id = latest_network_id(&conn)?
            .ok_or_else(|| "No hay inventario; ejecuta un escaneo primero.".to_string())?;
        let device = mylan_db::device_repo::get_device_by_ip(&conn, &net_id, ip_addr)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("No se encontró un dispositivo con IP {ip}."))?;
        device.id
    };
    let now = mylan_db::util::now_rfc3339().map_err(|e| e.to_string())?;
    let conn = clone_for_blocking(&state)?;
    let services_to_persist: Vec<Service> = services.clone();
    tokio::task::spawn_blocking(move || {
        for svc in &services_to_persist {
            if let Err(e) =
                mylan_db::service_repo::upsert_service(&conn, &fill_service(svc, &device_id, &now))
            {
                eprintln!(
                    "[mylan-desktop] upsert_service falló (puerto {}): {e}",
                    svc.port
                );
            }
        }
    })
    .await
    .map_err(|e| format!("persist join: {e}"))?;

    let _ = timeout; // ya aplicado por scan_target vía options.scan_timeout
    let _ = app.emit(
        "scan:finished",
        &ScanFinished {
            scan_id: scan_id.clone(),
        },
    );
    Ok(services)
}

#[tauri::command]
pub fn cancel_scan_cmd(
    scan_id: String,
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<bool, String> {
    let token = state
        .scan_tokens
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&scan_id);
    let found = token.is_some();
    if let Some(t) = token {
        t.cancel();
    }
    let _ = app.emit("scan:cancelled", &ScanCancelled { scan_id });
    Ok(found)
}

// --- Comandos: settings + db_path ------------------------------------------

#[tauri::command]
pub fn db_path_cmd(state: State<'_, DesktopState>) -> Result<String, String> {
    Ok(state.db_path.clone())
}

#[tauri::command]
pub async fn get_settings_cmd(state: State<'_, DesktopState>) -> Result<Settings, String> {
    Ok(state.settings.read().await.clone())
}

#[tauri::command]
pub async fn set_settings_cmd(
    settings: Settings,
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<(), String> {
    let path = settings_path(&app);
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("no se pudo escribir {:?}: {e}", path))?;
    *state.settings.write().await = settings;
    Ok(())
}

/// Path del fichero de settings: `app_data_dir/mylan-desktop.json`.
pub fn settings_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    dir.join("mylan-desktop.json")
}

/// Lee los settings persistidos (si existen) o devuelve el default.
pub fn load_settings(app: &AppHandle) -> Settings {
    let path = settings_path(app);
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

// --- Comandos: export (Paso 6, AC-10) --------------------------------------

/// Resuelve el `output_path` por defecto: `app_data_dir/exports/<name>.<ext>`.
fn default_export_path(app: &AppHandle, name: &str, ext: &str) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let exports = dir.join("exports");
    let _ = std::fs::create_dir_all(&exports);
    exports.join(format!("{name}.{ext}"))
}

fn ext_for(format: &str) -> Result<&'static str, String> {
    match format.to_ascii_lowercase().as_str() {
        "json" => Ok("json"),
        "csv" => Ok("csv"),
        other => Err(format!("formato no soportado: '{other}' (usar json|csv)")),
    }
}

#[tauri::command]
pub async fn export_devices_cmd(
    format: String,
    output_path: Option<String>,
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<String, String> {
    let ext = ext_for(&format)?;
    let path = match output_path {
        Some(p) => PathBuf::from(p),
        None => default_export_path(&app, "mylan-devices", ext),
    };
    let conn = clone_for_blocking(&state)?;
    let path_clone = path.clone();
    let fmt = format.clone();
    let outcome = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let net_id = latest_network_id(&conn)?
            .ok_or_else(|| "No hay inventario; ejecuta un escaneo primero.".to_string())?;
        let devices =
            mylan_db::device_repo::list_devices(&conn, &net_id).map_err(|e| e.to_string())?;
        if devices.is_empty() {
            return Err("No hay dispositivos para exportar.".to_string());
        }
        match fmt.to_ascii_lowercase().as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&devices).map_err(|e| e.to_string())?;
                std::fs::write(&path_clone, json).map_err(|e| e.to_string())?;
            }
            "csv" => {
                let mut buf = Vec::new();
                {
                    let mut wtr = csv::Writer::from_writer(&mut buf);
                    for d in &devices {
                        wtr.serialize(d).map_err(|e| e.to_string())?;
                    }
                    wtr.flush().map_err(|e| e.to_string())?;
                }
                std::fs::write(&path_clone, &buf).map_err(|e| e.to_string())?;
            }
            _ => return Err(format!("formato no soportado: '{fmt}'")),
        }
        Ok(())
    })
    .await
    .map_err(|e| format!("export join: {e}"))?;
    outcome?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn export_services_cmd(
    format: String,
    output_path: Option<String>,
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<String, String> {
    let ext = ext_for(&format)?;
    let path = match output_path {
        Some(p) => PathBuf::from(p),
        None => default_export_path(&app, "mylan-services", ext),
    };
    let conn = clone_for_blocking(&state)?;
    let path_clone = path.clone();
    let fmt = format.clone();
    let outcome = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let rows = mylan_db::service_repo::list_services(&conn, &ServiceFilters::default())
            .map_err(|e| e.to_string())?;
        if rows.is_empty() {
            return Err("No hay servicios para exportar.".to_string());
        }
        match fmt.to_ascii_lowercase().as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
                std::fs::write(&path_clone, json).map_err(|e| e.to_string())?;
            }
            "csv" => {
                let mut buf = Vec::new();
                {
                    let mut wtr = csv::Writer::from_writer(&mut buf);
                    wtr.write_record([
                        "device_id",
                        "device_ip",
                        "display_name",
                        "protocol",
                        "port",
                        "service_name",
                        "product",
                        "version",
                        "banner",
                        "state",
                        "first_seen_at",
                        "last_seen_at",
                    ])
                    .map_err(|e| e.to_string())?;
                    for r in &rows {
                        let record = vec![
                            r.device_id.clone(),
                            r.device_ip.map(|i| i.to_string()).unwrap_or_default(),
                            r.display_name.clone().unwrap_or_default(),
                            format!("{:?}", r.protocol).to_lowercase(),
                            r.port.to_string(),
                            r.service_name.clone().unwrap_or_default(),
                            r.product.clone().unwrap_or_default(),
                            r.version.clone().unwrap_or_default(),
                            r.banner.clone().unwrap_or_default(),
                            format!("{:?}", r.state).to_lowercase(),
                            r.first_seen_at.clone(),
                            r.last_seen_at.clone(),
                        ];
                        wtr.write_record(record.iter().map(String::as_str))
                            .map_err(|e| e.to_string())?;
                    }
                    wtr.flush().map_err(|e| e.to_string())?;
                }
                std::fs::write(&path_clone, &buf).map_err(|e| e.to_string())?;
            }
            _ => return Err(format!("formato no soportado: '{fmt}'")),
        }
        Ok(())
    })
    .await
    .map_err(|e| format!("export join: {e}"))?;
    outcome?;
    Ok(path.to_string_lossy().to_string())
}
