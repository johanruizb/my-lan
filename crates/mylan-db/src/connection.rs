//! Gestión de la conexión `rusqlite`.
//!
//! El path es configurable (override por la app/CLI); por defecto
//! `~/.local/share/mylan/mylan.db` respetando `$XDG_DATA_HOME`. Se crean los
//! directorios padres y se habilita la integridad referencial. Las migraciones
//! se aplican implícitamente al conectar.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::{map_sqlite, DbError, DbResult};
use crate::migrations::run_migrations;

/// Directorio por defecto de la base de datos (`$XDG_DATA_HOME/mylan` o
/// `~/.local/share/mylan`).
///
/// Devuelve `None` si no se puede determinar un `$HOME` (entorno mínimal).
#[must_use]
pub fn default_data_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("mylan"));
        }
    }
    let home = std::env::var("HOME").ok().filter(|h| !h.is_empty())?;
    Some(
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("mylan"),
    )
}

/// Path por defecto del fichero SQLite, si se puede resolver un directorio.
#[must_use]
pub fn default_db_path() -> Option<PathBuf> {
    default_data_dir().map(|d| d.join("mylan.db"))
}

/// Abre (o crea) la base de datos en `path`, aplica las migraciones y habilita
/// las claves foráneas.
///
/// Garantiza la existencia del directorio padre. Un path no escribible produce
/// [`DbError::Io`].
pub fn connect(path: impl AsRef<Path>) -> DbResult<Connection> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(DbError::Io)?;
        }
    }
    let conn = Connection::open(path).map_err(map_sqlite)?;
    setup(&conn)?;
    Ok(conn)
}

/// Configura una conexión ya abierta: WAL + `busy_timeout`, FKs on, migraciones.
///
/// `journal_mode=WAL` permite lecturas (`list_devices`, eventos `scan:progress`)
/// concurrentes con una escritura (la transacción de `run_scan_pipeline`) sin
/// bloquearse mutuamente; `busy_timeout=5000` hace que un escritor en contención
/// reintente hasta 5 s en vez de fallar de inmediato con `SQLITE_BUSY` (AC-12).
/// Nota: `try_clone` **no** hereda estos pragmas → setearlos también en el clon.
pub fn setup(conn: &Connection) -> DbResult<()> {
    // WAL + busy_timeout antes de las FKs: concurrencia read+write sin SQLITE_BUSY.
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
        .map_err(map_sqlite)?;
    // Fuerza la integridad referencial (FKs del esquema §8).
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(map_sqlite)?;
    run_migrations(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_creates_parent_dirs() {
        let dir = tempfile::tempdir().expect("tmp");
        let nested = dir.path().join("a/b/c").join("mylan.db");
        let conn = connect(&nested).expect("connect");
        assert!(nested.exists());
        // La migración ya está aplicada.
        let v: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert!(v >= 1);
    }

    #[test]
    fn connect_unwritable_path_errors() {
        // /proc no admite crear ficheros; usar un path bajo /proc/sys/kernel
        // tipicamente falla al abrir/escribir.
        let bad = Path::new("/proc/sys/kernel/nonexistent_dir/mylan.db");
        let res = connect(bad);
        assert!(res.is_err(), "expected error for unwritable path");
    }

    #[test]
    fn default_data_dir_respects_xdg() {
        // No asume nada del entorno del runner: solo comprueba que cuando
        // XDG_DATA_HOME está puesto, se respeta.
        let old = std::env::var("XDG_DATA_HOME").ok();
        std::env::set_var("XDG_DATA_HOME", "/tmp/xdg-test-mylan");
        let dir = default_data_dir();
        std::env::set_var("XDG_DATA_HOME", old.unwrap_or_default());
        assert_eq!(dir, Some(PathBuf::from("/tmp/xdg-test-mylan/mylan")));
    }
}
