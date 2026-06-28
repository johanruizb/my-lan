//! Estado compartido de la app Tauri: conexión SQLite reutilizada, tokens de
//! cancelación de scans activos y settings persistidos.
//!
//! `db` usa `std::sync::Mutex` (no async) porque `rusqlite::Connection` no es
//! `Send` a través de `.await`. El lock se sostiene solo brevemente dentro de
//! cada comando; el work sync pesado (pipeline, upserts) se mueve a
//! `spawn_blocking` sobre un `try_clone` de la conexión para no serializar los
//! reads concurrentes (AC-12).

use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::Connection;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::dto::Settings;

/// Estado gestionado por `tauri::Builder::manage`.
pub struct DesktopState {
    /// Conexión SQLite única, abierta en `setup`. Los comandos toman
    /// `state.db.lock()` y pasan `&Connection` a los repos.
    pub db: Mutex<Connection>,
    /// Path absoluto de la SQLite (inmutable tras `setup`).
    pub db_path: String,
    /// Tokens de cancelación de scans de puertos en curso, indexados por
    /// `scan_id`. Permite a `cancel_scan_cmd` cancelar un scan por id.
    pub scan_tokens: Mutex<HashMap<String, CancellationToken>>,
    /// Settings persistidas (AC-9).
    pub settings: RwLock<Settings>,
}

impl DesktopState {
    pub fn new(db: Connection, db_path: String, settings: Settings) -> Self {
        Self {
            db: Mutex::new(db),
            db_path,
            scan_tokens: Mutex::new(HashMap::new()),
            settings: RwLock::new(settings),
        }
    }
}
