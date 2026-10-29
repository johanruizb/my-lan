//! Utilidades de la CLI: timestamps RFC3339, IDs UUID y la nota de redacción.
//!
//! `new_id`/`now_rfc3339` se movieron a [`mylan_db::util`] (Fase 4-A) para
//! compartirlos con el pipeline reusable; se reexportan aquí para que los
//! comandos de la CLI sigan usando `crate::util::{new_id, now_rfc3339}`.

use std::io::Write;

pub use mylan_db::util::{new_id, now_rfc3339};

/// Escribe el aviso de redacción en el `writer` especificado.
pub fn print_redaction_note_to(writer: &mut impl Write) {
    let _ = writeln!(
        writer,
        "[mylan] Nota de redacción: la salida incluye MACs/IPs reales de tu red. \
         Censúralas antes de compartirlas en canales públicos."
    );
}

/// Aviso de redacción: la salida del CLI contiene MACs/IPs reales de la red.
///
/// Se imprime siempre al inicio de los comandos que muestran inventario, para
/// que el usuario sepa que no debe pegar la salida en canales públicos sin
/// censurar.
pub fn print_redaction_note() {
    print_redaction_note_to(&mut std::io::stderr());
}

/// Neutraliza secuencias de control en texto proveniente de la red antes de
/// imprimirlo en una terminal. Evita inyección ANSI/OSC (clipboard, limpiar
/// pantalla, falsificar salida) desde hostnames, banners, DNS o notas.
#[must_use]
pub fn sanitize_terminal(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '\n' | '\r' | '\t' => ' ',
            c if c.is_control() => '\u{FFFD}',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_redaction_note_content() {
        let mut buf = Vec::new();
        print_redaction_note_to(&mut buf);
        let output = String::from_utf8(buf).expect("output must be valid UTF-8");

        assert!(
            output.contains(
                "[mylan] Nota de redacción: la salida incluye MACs/IPs reales de tu red."
            ),
            "La nota debe indicar el prefijo e advertencia de redacción"
        );
        assert!(
            output.contains("Censúralas antes de compartirlas en canales públicos."),
            "La nota debe advertir sobre la censura antes de compartir"
        );
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_print_redaction_note_does_not_panic() {
        print_redaction_note();
    }

    #[test]
    fn sanitize_terminal_strips_escape_sequences() {
        let raw = "host\u{1b}]52;c;Zm9v\u{7}name\u{1b}[2J";
        let clean = sanitize_terminal(raw);
        assert!(!clean.contains('\u{1b}'));
        assert!(!clean.contains('\u{7}'));
        assert!(clean.contains("host"));
        assert!(clean.contains("name"));
    }
}
