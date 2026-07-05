//! Gestión del token de autenticación del API (AC-7, ADR-7).
//!
//! El token es un bearer secret de 32 bytes generado con `getrandom` y
//! codificado base64 URL-safe sin padding (ADR-7 — no se usa `uuid`). Se
//! persiste en un fichero con permisos `0600` en Unix, bajo el directorio de
//! datos de MyLAN (parent dir de `mylan_db::connection::default_db_path()`).
//! El modelo de seguridad es localhost-only (`127.0.0.1`), sin TLS ni auth
//! remoto (no-goal v0.5).

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use base64::Engine;

/// Tamaño del token en bytes (antes de base64). 32 bytes = 256 bits de
/// entropía, suficiente para un bearer secret local.
const TOKEN_BYTES: usize = 32;

/// Genera un token aleatorio de 32 bytes, base64 URL-safe sin padding (ADR-7).
///
/// No toca disco; usar [`load_or_create_token`] para persistirlo.
pub fn generate_token() -> Result<String> {
    let mut buf = [0u8; TOKEN_BYTES];
    getrandom::getrandom(&mut buf).map_err(|e| anyhow!("getrandom: {e}"))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf))
}

/// Path por defecto del fichero de token: parent dir de la DB por defecto
/// (`mylan_db::connection::default_db_path()`), fichero `api-token`.
///
/// Devuelve `None` si no se puede resolver el directorio de datos (p.ej. sin
/// `$HOME`); el caller debe propagar un error en ese caso.
#[must_use]
pub fn default_token_path() -> Option<PathBuf> {
    mylan_db::connection::default_db_path().and_then(|p| p.parent().map(|d| d.join("api-token")))
}

/// Path del fichero de token derivado de la ruta de la DB (C2 fix): parent dir
/// de `db_path` + `api-token`. Robusto a entornos sin `$HOME` (systemd
/// `ProtectHome=true`, Docker) — el caller pasa el `db_path` del config, no el
/// default que depende de `$HOME`.
#[must_use]
pub fn token_path_for_db(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("api-token")
}

/// Carga el token desde `path`, o lo genera y persiste (`0600` Unix) si no
/// existe. Devuelve el token leído/creado.
///
/// Si el fichero existe pero está vacío, se regenera (un token vacío no es
/// válido).
pub fn load_or_create_token(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    if let Some(token) = read_token(path)? {
        return Ok(token);
    }
    let token = generate_token()?;
    write_token_file(path, &token)?;
    tracing::info!(path = %path.display(), "token creado");
    Ok(token)
}

/// Rota el token: genera uno nuevo y lo sobrescribe en `path` (`0600` Unix).
/// Devuelve el nuevo token.
pub fn rotate_token(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let token = generate_token()?;
    write_token_file(path, &token)?;
    tracing::info!(path = %path.display(), "token rotado");
    Ok(token)
}

/// Lee el token de `path`; `Ok(None)` si el fichero no existe o está vacío.
fn read_token(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).context(format!("leyendo token {}", path.display())),
    }
}

/// Escribe `token` en `path` con permisos `0600` en Unix (ACL-restricted en
/// Windows — TODO v0.6). Trunca si existe.
fn write_token_file(path: &Path, token: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creando dir {}", parent.display()))?;
        }
    }
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts
        .open(path)
        .with_context(|| format!("abriendo token {}", path.display()))?;
    file.write_all(token.as_bytes())
        .with_context(|| format!("escribiendo token {}", path.display()))?;
    // Reafirma los permisos si el fichero ya existía con un modo más abierto.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("seteando permisos 0600 en {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_is_url_safe_no_pad_43_chars() {
        let t = generate_token().unwrap();
        // 32 bytes -> base64 sin padding = ceil(32*4/3) = 43 chars, sin '='.
        assert_eq!(t.len(), 43, "32 bytes base64 sin padding = 43 chars");
        assert!(!t.contains('='), "sin padding");
        assert!(
            t.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "URL-safe charset"
        );
    }

    #[test]
    fn generate_token_is_random() {
        let a = generate_token().unwrap();
        let b = generate_token().unwrap();
        assert_ne!(a, b, "dos tokens deben diferir");
    }

    #[test]
    fn load_or_create_then_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-token");
        let t1 = load_or_create_token(&path).unwrap();
        assert!(path.exists());
        // Recargar devuelve el mismo token.
        let t2 = load_or_create_token(&path).unwrap();
        assert_eq!(t1, t2);
    }

    #[test]
    fn rotate_changes_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-token");
        let t1 = load_or_create_token(&path).unwrap();
        let t2 = rotate_token(&path).unwrap();
        assert_ne!(t1, t2, "rotación cambia el token");
        // Recargar devuelve el nuevo.
        let t3 = load_or_create_token(&path).unwrap();
        assert_eq!(t2, t3);
    }

    #[cfg(unix)]
    #[test]
    fn token_file_has_0600_perms() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-token");
        load_or_create_token(&path).unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "perms 0600");
    }

    #[cfg(unix)]
    #[test]
    fn rotate_preserves_0600_perms() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-token");
        load_or_create_token(&path).unwrap();
        // Corromper permisos a 0644 para verificar que rotate los reafirma.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        rotate_token(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "rotate reafirma 0600");
    }

    #[test]
    fn empty_token_regenerates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-token");
        std::fs::write(&path, "   ").unwrap();
        let t = load_or_create_token(&path).unwrap();
        assert!(!t.trim().is_empty());
    }
}
