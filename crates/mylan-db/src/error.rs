//! Errores de `mylan-db`.
//!
//! `thiserror` porque `mylan-db` es una librería (regla del plan: anyhow para apps,
//! thiserror para libs).

use thiserror::Error;

/// Fallo de una operación de persistencia.
#[derive(Debug, Error)]
pub enum DbError {
    /// Error del enlace `rusqlite`.
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// Error de E/S (creación de directorios, apertura del fichero).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Error de (de)serialización JSON (`dns_servers`, `summary_json`).
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    /// La base de datos está bloqueada por otro escritor (concurrencia).
    #[error("database is locked or busy")]
    Locked,
}

/// Resultado abreviado para las operaciones del crate.
pub type DbResult<T> = Result<T, DbError>;

/// Convierte un error `rusqlite` en `DbError`, mapeando `SQLITE_BUSY`/`LOCKED`
/// a [`DbError::Locked`] para que los tests de concurrencia lo distingan.
pub(crate) fn map_sqlite(err: rusqlite::Error) -> DbError {
    if let rusqlite::Error::SqliteFailure(ref f, _) = err {
        // SQLITE_BUSY = 5, SQLITE_LOCKED = 6.
        if matches!(f.code, rusqlite::ErrorCode::DatabaseBusy) {
            return DbError::Locked;
        }
    }
    DbError::Sqlite(err)
}
