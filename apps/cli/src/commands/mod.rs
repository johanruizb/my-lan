//! Comandos del CLI `mylan`.
//!
//! Cada subcomando es un módulo pequeño con su handler. Los handlers comparten
//! los helpers de [`mod`] (apertura de DB y resolución de la red activa).

pub mod agent;
pub mod device;
pub mod devices;
pub mod diagnose;
pub mod export;
pub mod ports;
pub mod scan;
pub mod serve;
pub mod services;
pub mod status;

use rusqlite::Connection;

use crate::ctx::AppContext;

/// Abre la DB configurada en el contexto (crea migraciones implícitamente).
pub fn open_db(ctx: &AppContext) -> anyhow::Result<Connection> {
    let conn = mylan_db::connection::connect(&ctx.db_path)?;
    Ok(conn)
}

/// Resuelve el `network_id` de la red activa: la del escaneo más reciente.
///
/// Si no hay escaneos previos devuelve `None` (el inventario aún está vacío).
pub fn latest_network_id(conn: &Connection) -> anyhow::Result<Option<String>> {
    let result = conn.query_row(
        "SELECT network_id FROM scans ORDER BY started_at DESC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
