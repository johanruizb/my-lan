//! Contexto de la CLI: rutas de DB y signatures, y bandera de verbosidad.

use std::path::PathBuf;

/// Rutas y configuración compartidas por los comandos.
#[derive(Debug, Clone)]
pub struct AppContext {
    /// Path del fichero SQLite (default: `~/.local/share/mylan/mylan.db`).
    pub db_path: PathBuf,
    /// Directorio de signatures (OUI + reglas YAML).
    pub signatures_dir: PathBuf,
    /// Verbosidad de trazado (`--verbose`).
    pub verbose: bool,
}

impl AppContext {
    /// Construye el contexto con las rutas por defecto.
    ///
    /// `signatures_dir` apunta al directorio `signatures/` relativo al binario
    /// (envoltorio del repo); si no existe, el fingerprint degrada a no-op.
    #[must_use]
    pub fn new(verbose: bool) -> Self {
        let db_path =
            mylan_db::connection::default_db_path().unwrap_or_else(|| PathBuf::from("mylan.db"));
        Self {
            db_path,
            signatures_dir: mylan_fingerprint::default_signatures_dir(),
            verbose,
        }
    }
}
