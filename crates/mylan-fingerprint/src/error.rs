//! Errores de `mylan-fingerprint`.
//!
//! Usa `thiserror` (lib, sin `anyhow`: principio P3/mylan-core de dominio puro
//! reutilizable). Las cargas de firmas (OUI/reglas) son fallibles y se reportan
//! con contexto suficiente para diagnosticar el fichero problemático.

use thiserror::Error;

/// Error de fingerprinting (carga de OUI/reglas o parseo).
#[derive(Debug, Error)]
pub enum FingerprintError {
    /// Fallo leyendo/parseando el CSV OUI.
    #[error("OUI database load failed: {0}")]
    OuiLoad(String),
    /// Fallo leyendo/parseando un fichero de reglas YAML.
    #[error("rule load failed in `{path}`: {message}")]
    RuleLoad { path: String, message: String },
    /// Fallo de I/O genérico al abrir un fichero de firmas.
    #[error("signature file I/O error: {0}")]
    Io(#[from] std::io::Error),
}
