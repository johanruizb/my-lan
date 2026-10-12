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
/// Imprime el aviso de redacción en cualquier escritor (`std::io::Write`).
pub fn print_redaction_note_to(w: &mut impl std::io::Write) {
    let _ = writeln!(
        w,
        "[mylan] Nota de redacción: la salida incluye MACs/IPs reales de tu red. \
         Censúralas antes de compartirlas en canales públicos."
    );
}

pub fn print_redaction_note() {
    print_redaction_note_to(&mut std::io::stderr());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_redaction_note_to_writer() {
        let mut buffer = Vec::new();
        print_redaction_note_to(&mut buffer);
        let output = String::from_utf8(buffer).expect("output should be valid UTF-8");
        assert!(output.contains("[mylan] Nota de redacción"));
        assert!(output.contains("Censúralas antes de compartirlas en canales públicos."));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_print_redaction_note_stderr() {
        // Asegura que la función sin argumentos no hace pánico.
        print_redaction_note();
    }
}
