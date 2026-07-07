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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_rfc3339_is_utc() {
        let ts = now_rfc3339().expect("now_rfc3339");
        assert!(
            ts.ends_with('Z'),
            "esperaba sufijo 'Z' (UTC); obtenido: {ts}"
        );
    }

    #[test]
    fn now_rfc3339_has_rfc3339_separators() {
        let ts = now_rfc3339().expect("now_rfc3339");
        assert!(ts.contains('T'), "falta separador 'T': {ts}");
        assert!(ts.contains('-'), "falta guion de fecha: {ts}");
        assert!(ts.contains(':'), "falta dos puntos de hora: {ts}");
    }

    #[test]
    fn now_rfc3339_has_expected_length_and_pattern() {
        let ts = now_rfc3339().expect("now_rfc3339");
        // 'YYYY-MM-DDTHH:MM:SS[.fffffff...]Z'. El formateador RFC3339 de `time`
        // emite fracciones de nanosegundos cuando hay precisión disponible, así
        // que la longitud varía (mínimo 20 sin fracciones). Comprobamos el patrón.
        assert!(
            ts.len() >= 20,
            "longitud RFC3339 inesperada ({len}): {ts}",
            len = ts.len()
        );
        assert!(ts[..4].chars().all(char::is_numeric), "año numérico: {ts}");
        assert_eq!(&ts[4..5], "-", "separador de fecha: {ts}");
        assert_eq!(&ts[7..8], "-", "separador de fecha: {ts}");
        assert_eq!(&ts[10..11], "T", "separador T: {ts}");
        assert_eq!(&ts[13..14], ":", "separador de hora: {ts}");
        assert_eq!(&ts[16..17], ":", "separador de hora: {ts}");
    }

    #[test]
    fn now_rfc3339_is_chronologically_consistent() {
        // RFC3339 en UTC ordena lexicográficamente igual que cronológicamente.
        // Dos llamadas consecutivas son non-decreasing (igual o posterior) sin
        // necesidad de sleep (code-review MINOR: evitar 1100ms de retardo).
        let a = now_rfc3339().expect("now_rfc3339");
        let b = now_rfc3339().expect("now_rfc3339");
        assert!(a <= b, "timestamps no deben ser decrecientes: {a} vs {b}");
    }

    #[test]
    fn new_id_is_valid_uuid_v4() {
        let id = new_id();
        let parsed = uuid::Uuid::parse_str(&id).expect("UUID parseable");
        assert_eq!(
            parsed.get_version(),
            Some(uuid::Version::Random),
            "debe ser UUID v4 (random): {id}"
        );
    }

    #[test]
    fn new_id_is_unique_across_calls() {
        let n = 64;
        let mut seen = std::collections::HashSet::with_capacity(n);
        for _ in 0..n {
            let id = new_id();
            assert!(seen.insert(id), "ID duplicado detectado");
        }
        assert_eq!(seen.len(), n);
    }

    #[test]
    fn new_id_is_36_chars_dashed_format() {
        let id = new_id();
        assert_eq!(id.len(), 36, "longitud canónica UUID: {id}");
        assert_eq!(id.matches('-').count(), 4, "4 guiones: {id}");
    }
}
