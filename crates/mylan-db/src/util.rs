//! Utilidades compartidas de persistencia: timestamps RFC3339 e IDs UUID.
//!
//! Viven en `mylan-db` (no en la CLI) para que el pipeline reusable
//! ([`crate::pipeline`]) y sus consumidores (CLI, Desktop, futura `mylan-api`)
//! generen IDs/timestamps de forma consistente sin duplicar la lógica.

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Timestamp actual en UTC con formato RFC3339 (columna `TEXT` de la DB).
///
/// # Errors
/// Devuelve error si el formateo RFC3339 falla (no debería ocurrir con un
/// instante válido).
pub fn now_rfc3339() -> anyhow::Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}

/// ID UUID v4 para nuevas filas (dispositivos, escaneos, servicios).
#[must_use]
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
