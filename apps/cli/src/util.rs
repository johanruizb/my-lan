//! Utilidades de la CLI: timestamps RFC3339, IDs UUID y la nota de redacción.

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Timestamp actual en UTC con formato RFC3339 (columna `TEXT` de la DB).
pub fn now_rfc3339() -> anyhow::Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}

/// ID UUID v4 para nuevas filas (dispositivos, escaneos, servicios).
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Aviso de redacción: la salida del CLI contiene MACs/IPs reales de la red.
///
/// Se imprime siempre al inicio de los comandos que muestran inventario, para
/// que el usuario sepa que no debe pegar la salida en canales públicos sin
/// censurar.
pub fn print_redaction_note() {
    eprintln!(
        "[mylan] Nota de redacción: la salida incluye MACs/IPs reales de tu red. \
         Censúralas antes de compartirlas en canales públicos."
    );
}
