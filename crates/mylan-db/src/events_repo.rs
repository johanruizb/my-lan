//! Repositorio de eventos del timeline de diferencias entre escaneos (`events`).
//!
//! Persiste los diff events producidos por el motor de diff (`crate::diff`, v0.5
//! Watch, AC-3) y los lee para la API (`GET /api/v1/events`, AC-4) y el backfill
//! del canal WS (`?since=<ISO8601>`, AC-6). La DB es la fuente de verdad; el canal
//! WS es una vista en vivo del mismo flujo (Principio 4). `EventType`/`Severity`
//! serializan en `snake_case` (reexport de serde) y aquí se proyectan a la columna
//! `TEXT` sin comillas, siguiendo el patrón `codec::enum_from_db`/`enum_to_db`
//! (string-backed en DB, tipado en Rust — idéntico a `device_repo.rs`/`scan_repo.rs`).
//!
//! `Event.id` lo genera el llamador vía `util::new_id()` (UUID v4, `util.rs:21`),
//! tanto el motor de diff como la API; este repo no lo genera para mantener la
//! responsabilidad de identidad en una sola capa.

use rusqlite::{Connection, Row};

use mylan_core::Event;

use crate::codec::{enum_from_db, enum_to_db};
use crate::error::{map_sqlite, DbResult};

/// Columnas leídas de `events` (orden del `SELECT`).
const SELECT_COLS: &str =
    "id, network_id, device_id, event_type, severity, message, data_json, created_at";

/// Fila bruta leída de la DB, antes de decodificar `EventType`/`Severity`.
struct EventRow {
    id: String,
    network_id: String,
    device_id: Option<String>,
    event_type: String,
    severity: String,
    message: Option<String>,
    data_json: Option<String>,
    created_at: String,
}

impl EventRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            network_id: row.get(1)?,
            device_id: row.get(2)?,
            event_type: row.get(3)?,
            severity: row.get(4)?,
            message: row.get(5)?,
            data_json: row.get(6)?,
            created_at: row.get(7)?,
        })
    }

    fn decode(self) -> DbResult<Event> {
        Ok(Event {
            id: self.id,
            network_id: self.network_id,
            device_id: self.device_id,
            event_type: enum_from_db(&self.event_type)?,
            severity: enum_from_db(&self.severity)?,
            message: self.message,
            data_json: self.data_json,
            created_at: self.created_at,
        })
    }
}

/// Inserta un evento del timeline (AC-4). El `id` y `created_at` vienen fijados
/// por el llamador (motor de diff en la misma txn atómica que el scan, ADR-5, o
/// la API). Acepta `&Connection` y también `&Transaction` vía deref coercion, de
/// forma que el diff lo persista dentro de la txn del scan sin abrir una nueva.
///
/// # Errors
/// `DbError::Sqlite` si viola la FK `network_id` (red inexistente) o `device_id`
/// (dispositivo inexistente con valor no-`NULL`), o si el `id` colisiona.
pub fn insert_event(conn: &Connection, event: &Event) -> DbResult<()> {
    conn.execute(
        "INSERT INTO events (
           id, network_id, device_id, event_type, severity, message, data_json, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            event.id,
            event.network_id,
            event.device_id,
            enum_to_db(&event.event_type)?,
            enum_to_db(&event.severity)?,
            event.message,
            event.data_json,
            event.created_at,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Lista eventos del timeline, opcionalmente filtrados por `network_id`,
/// ordenados por `created_at` descendente (más reciente primero — convención de
/// `scan_repo::list_scans` con `started_at DESC`). `limit`/`offset` paganinan
/// (AC-4 `GET /api/v1/events`).
///
/// `limit <= 0` devuelve vacío; `offset` desplaza dentro del orden. Sin filtro de
/// red cuando `network_id` es `None` (timeline global del agente).
pub fn list_events(
    conn: &Connection,
    network_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> DbResult<Vec<Event>> {
    let sql = if network_id.is_some() {
        format!(
            "SELECT {SELECT_COLS} FROM events WHERE network_id = ?1 \
             ORDER BY created_at DESC, id DESC LIMIT ?2 OFFSET ?3"
        )
    } else {
        format!(
            "SELECT {SELECT_COLS} FROM events ORDER BY created_at DESC, id DESC \
             LIMIT ?1 OFFSET ?2"
        )
    };
    let mut stmt = conn.prepare(&sql).map_err(map_sqlite)?;
    let rows = if let Some(nid) = network_id {
        stmt.query_map(rusqlite::params![nid, limit, offset], EventRow::from_row)
            .map_err(map_sqlite)?
    } else {
        stmt.query_map(rusqlite::params![limit, offset], EventRow::from_row)
            .map_err(map_sqlite)?
    };
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_sqlite)?.decode()?);
    }
    Ok(out)
}

/// Lista eventos con `created_at > since` (cursor ISO8601 para backfill de WS,
/// AC-6 `?since=<ISO8601>`). Orden cronológico ascendente — el cliente reconectado
/// recibe los eventos desde el cursor en orden para reconstruir su timeline. Sin
/// filtro de red: el WS transmite todos los eventos del agente.
pub fn list_events_since(conn: &Connection, since: &str) -> DbResult<Vec<Event>> {
    let sql = format!(
        "SELECT {SELECT_COLS} FROM events WHERE created_at > ?1 \
         ORDER BY created_at ASC, id ASC"
    );
    let mut stmt = conn.prepare(&sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map([since], EventRow::from_row)
        .map_err(map_sqlite)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_sqlite)?.decode()?);
    }
    Ok(out)
}
