//! Utilidades de la CLI: timestamps RFC3339, IDs UUID y la nota de redacción.
//!
//! `new_id`/`now_rfc3339` se movieron a [`mylan_db::util`] (Fase 4-A) para
//! compartirlos con el pipeline reusable; se reexportan aquí para que los
//! comandos de la CLI sigan usando `crate::util::{new_id, now_rfc3339}`.

pub use mylan_db::util::{new_id, now_rfc3339};

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
