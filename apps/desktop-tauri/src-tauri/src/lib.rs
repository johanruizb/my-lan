//! `mylan-desktop` — backend Tauri 2 de MyLAN Desktop Alpha (Fase 4-B2).
//!
//! Capa fina de comandos `#[tauri::command]` sobre `mylan-discovery`,
//! `mylan-scanner`, `mylan-fingerprint` y `mylan-db`. Es dueña del ciclo de
//! vida de la SQLite (abre/crea/migra en `app_data_dir/mylan.db` al primer
//! arranque) e importa opcionalmente la DB del CLI para usuarios brownfield
//! (AC-11). Sin servidor HTTP (`mylan-api` se difiere a Fase 7).

mod commands;
mod dto;
mod state;

use std::path::Path;
use std::time::Duration;

use rusqlite::backup::Backup;
use rusqlite::OpenFlags;
use tauri::{Emitter, Manager};

use crate::commands::{load_settings, settings_path};
use crate::state::DesktopState;

/// Importa la DB del CLI al path del Desktop (one-shot, AC-11).
///
/// Con WAL habilitado en la DB origen, una `fs::copy` de solo `mylan.db`
/// daría un snapshot stale/torn (SQLite escribe a `-wal`/`-shm`); `Backup` toma
/// un snapshot consistente manejando WAL automáticamente. Devuelve `true` si
/// importó, `false` si no había nada que importar. Falla silenciosamente (log)
/// — nunca bloquea el arranque.
fn import_brownfield(app_data_db: &Path) -> bool {
    let Some(src) = mylan_db::connection::default_db_path() else {
        return false;
    };
    if !src.exists() || app_data_db.exists() {
        return false;
    }
    // Abrir origen read-only para no competir con un CLI en ejecución.
    let src_conn = match rusqlite::Connection::open_with_flags(
        &src,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(_) => return false,
    };
    // Destino: fichero nuevo; `Backup` lo inicializa.
    if let Some(parent) = app_data_db.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut dest_conn = match rusqlite::Connection::open(app_data_db) {
        Ok(c) => c,
        Err(_) => return false,
    };
    match Backup::new(&src_conn, &mut dest_conn)
        .and_then(|b| b.run_to_completion(100, Duration::from_millis(10), None))
    {
        Ok(_) => true,
        Err(_) => {
            // Limpia un dest parcial para no dejar un fichero corrupto.
            let _ = std::fs::remove_file(app_data_db);
            false
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let _ = std::fs::create_dir_all(&app_data_dir);
            let db_path = app_data_dir.join("mylan.db");

            // Importación brownfield (AC-11): si la DB del CLI existe y la del
            // Desktop no, copia un snapshot consistente. One-shot, sin sync
            // posterior. Caveat: cerrar el CLI primero para no competir.
            if import_brownfield(&db_path) {
                let _ = app.emit("db:imported", &());
            }

            // Conexión única reutilizada (WAL + busy_timeout aplicados en
            // `mylan_db::connection::setup`).
            let conn = mylan_db::connection::connect(&db_path)?;

            // Settings: persiste el db_path resuelto si aún no existía.
            let mut settings = load_settings(app.handle());
            if settings.db_path.is_empty() {
                settings.db_path = db_path.to_string_lossy().to_string();
                if let Ok(json) = serde_json::to_string_pretty(&settings) {
                    let _ = std::fs::write(settings_path(app.handle()), json);
                }
            }

            app.manage(DesktopState::new(
                conn,
                db_path.to_string_lossy().to_string(),
                settings,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_interface_cmd,
            commands::list_devices_cmd,
            commands::get_device_cmd,
            commands::run_discovery_cmd,
            commands::scan_ports_cmd,
            commands::cancel_scan_cmd,
            commands::list_services_cmd,
            commands::export_devices_cmd,
            commands::export_services_cmd,
            commands::db_path_cmd,
            commands::get_settings_cmd,
            commands::set_settings_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
