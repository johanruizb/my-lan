//! Motor de diff entre escaneos (v0.5 Watch, Step 2, AC-3).
//!
//! Snapshot antes/después de [`run_scan_pipeline_at_in_tx`](crate::pipeline) →
//! emite `Event`s del timeline (`device_new`, `device_ip_changed`, `device_offline`,
//! `device_online`, `port_opened`). [`run_diff`] escribe el estado `is_online` a
//! `devices` vía la txn del scan (offline→0, returning→1) y RETORNA los events para
//! que el caller los persista en la misma txn atómica (ADR-5). La supresión de
//! cold-start evita la tormenta de `device_online`/`device_offline` en el primer
//! scan tras restart del agent (Q5).
//!
//! `port_opened` se detecta comparando snapshots de `services` antes/después del
//! pipeline (ADR-6): el pipeline NO inserta services (port scan es on-demand vía
//! `mylan ports <ip>`), así que el snapshot captura los services insertados entre
//! scans por `mylan ports <ip>`.

use std::collections::HashMap;

use rusqlite::{params_from_iter, Connection, Transaction};

use mylan_core::{Event, EventType, Severity};

use crate::error::{map_sqlite, DbResult};
use crate::util::new_id;

/// Snapshot de un dispositivo (diff por `id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSnapshot {
    pub id: String,
    pub primary_ip: Option<String>,
    pub first_seen_at: String,
    pub is_online: bool,
}

/// Snapshot de un servicio (diff por `(protocol, port)` dentro de un dispositivo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSnapshot {
    pub device_id: String,
    pub protocol: String,
    pub port: u16,
    pub service_name: Option<String>,
}

/// Snapshots de los dispositivos de una red (estado pre-pipeline).
pub fn snapshot_devices_before(
    conn: &Connection,
    network_id: &str,
) -> DbResult<Vec<DeviceSnapshot>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, primary_ip, first_seen_at, is_online FROM devices WHERE network_id = ?1",
        )
        .map_err(map_sqlite)?;
    let rows = stmt
        .query_map([network_id], |row| {
            Ok(DeviceSnapshot {
                id: row.get(0)?,
                primary_ip: row.get(1)?,
                first_seen_at: row.get(2)?,
                is_online: row.get::<_, i64>(3)? != 0,
            })
        })
        .map_err(map_sqlite)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_sqlite)?);
    }
    Ok(out)
}

/// Snapshots de los servicios de un conjunto de dispositivos (estado
/// pre-pipeline), agrupados por `device_id`. `device_ids` vacío → mapa vacío
/// (sin query `IN ()`, inválido en SQL).
pub fn snapshot_services_before(
    conn: &Connection,
    device_ids: &[String],
) -> DbResult<HashMap<String, Vec<ServiceSnapshot>>> {
    query_services_for(conn, device_ids)
}

/// Query interna de servicios por `device_id` (compartida por el snapshot before y
/// el re-query after de [`run_diff`]). Acepta `&Connection` (un `&Transaction`
/// coerciona vía `Deref`).
fn query_services_for(
    conn: &Connection,
    device_ids: &[String],
) -> DbResult<HashMap<String, Vec<ServiceSnapshot>>> {
    let mut out: HashMap<String, Vec<ServiceSnapshot>> = HashMap::new();
    if device_ids.is_empty() {
        return Ok(out);
    }
    // `IN (?, ?, ...)` construido con un placeholder por id; `params_from_iter`
    // pasa la lista como params de rusqlite (no hay `IN (?)` vectorial nativo).
    let placeholders = (0..device_ids.len())
        .map(|i| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT device_id, protocol, port, service_name FROM services WHERE device_id IN ({placeholders})"
    );
    let mut stmt = conn.prepare(&sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map(
            params_from_iter(device_ids.iter().map(String::as_str)),
            |row| {
                Ok(ServiceSnapshot {
                    device_id: row.get(0)?,
                    protocol: row.get(1)?,
                    port: u16::try_from(row.get::<_, i64>(2)?).unwrap_or(0),
                    service_name: row.get(3)?,
                })
            },
        )
        .map_err(map_sqlite)?;
    for row in rows {
        let svc = row.map_err(map_sqlite)?;
        out.entry(svc.device_id.clone()).or_default().push(svc);
    }
    Ok(out)
}

/// Ejecuta el diff antes/después del pipeline y retorna los `Event`s del timeline.
///
/// Re-querya el estado "after" (devices con `last_seen_at == scan_now` + sus
/// services) dentro de `tx`, compara contra los snapshots "before", escribe
/// `is_online` a `devices` (offline→0, returning→1) y RETORNA los events — el
/// caller los persiste en la misma txn (ADR-5 atómico con el scan).
///
/// # Cold-start (`cold_start == true`)
/// Suprime `device_online`/`device_offline` — evento **y** escritura de
/// `is_online` — en el primer scan tras restart del agent: no se puede saber si un
/// device no-visto es realmente offline o simplemente aún no descubierto. Evita la
/// tormenta de events al restart y el falso `device_online` en el scan siguiente
/// (los demás event types — `device_new`, `device_ip_changed`, `port_opened` — no
/// se suprimen).
///
/// # Errors
/// Propaga errores SQLite (escritura de `is_online`, re-querys) como `anyhow`.
pub fn run_diff(
    tx: &Transaction,
    network_id: &str,
    scan_now: &str,
    before_devices: Vec<DeviceSnapshot>,
    before_services: HashMap<String, Vec<ServiceSnapshot>>,
    cold_start: bool,
) -> anyhow::Result<Vec<Event>> {
    let mut events = Vec::new();

    // After devices: vistos este scan (last_seen_at == scan_now).
    let after_devices = after_devices(tx, network_id, scan_now)?;
    let after_by_id: HashMap<&str, &DeviceSnapshot> =
        after_devices.iter().map(|d| (d.id.as_str(), d)).collect();
    let before_by_id: HashMap<&str, &DeviceSnapshot> =
        before_devices.iter().map(|d| (d.id.as_str(), d)).collect();

    // After services: de los devices vistos este scan (ADR-6 — re-query post-pipeline).
    let after_device_ids: Vec<String> = after_devices.iter().map(|d| d.id.clone()).collect();
    let after_services = query_services_for(tx, &after_device_ids)?;

    // device_new: in after con first_seen_at == scan_now (recién insertado).
    for after in &after_devices {
        if after.first_seen_at == scan_now {
            events.push(make_event(
                network_id,
                Some(&after.id),
                EventType::DeviceNew,
                Severity::Info,
                Some("New device discovered".to_string()),
                None,
                scan_now,
            ));
        }
    }

    // device_offline: in before is_online==1, no in after. Suprimido en cold_start.
    if !cold_start {
        for before in &before_devices {
            if before.is_online && !after_by_id.contains_key(before.id.as_str()) {
                tx.execute(
                    "UPDATE devices SET is_online = 0 WHERE id = ?1",
                    rusqlite::params![before.id],
                )?;
                events.push(make_event(
                    network_id,
                    Some(&before.id),
                    EventType::DeviceOffline,
                    Severity::Warning,
                    Some("Device offline".to_string()),
                    None,
                    scan_now,
                ));
            }
        }
    }

    // device_online: in before is_online==0, in after. Suprimido en cold_start.
    // (upsert ya fijó is_online=1 vía merge incoming.is_online||existing; la
    // escritura aquí es idempotente y por contrato del plan.)
    if !cold_start {
        for before in &before_devices {
            if !before.is_online && after_by_id.contains_key(before.id.as_str()) {
                tx.execute(
                    "UPDATE devices SET is_online = 1 WHERE id = ?1",
                    rusqlite::params![before.id],
                )?;
                events.push(make_event(
                    network_id,
                    Some(&before.id),
                    EventType::DeviceOnline,
                    Severity::Info,
                    Some("Device back online".to_string()),
                    None,
                    scan_now,
                ));
            }
        }
    }

    // device_ip_changed: in ambos, primary_ip difiere.
    for after in &after_devices {
        if let Some(before) = before_by_id.get(after.id.as_str()) {
            if before.primary_ip != after.primary_ip {
                let data = serde_json::json!({
                    "old_ip": before.primary_ip,
                    "new_ip": after.primary_ip,
                })
                .to_string();
                events.push(make_event(
                    network_id,
                    Some(&after.id),
                    EventType::DeviceIpChanged,
                    Severity::Info,
                    Some("Device IP changed".to_string()),
                    Some(data),
                    scan_now,
                ));
            }
        }
    }

    // port_opened: after_services[d] - before_services[d] (ADR-6).
    for after_dev in &after_devices {
        let after_svcs = after_services
            .get(&after_dev.id)
            .cloned()
            .unwrap_or_default();
        let before_svcs = before_services
            .get(&after_dev.id)
            .cloned()
            .unwrap_or_default();
        for svc in &after_svcs {
            let is_new = !before_svcs
                .iter()
                .any(|b| b.protocol == svc.protocol && b.port == svc.port);
            if is_new {
                let data = serde_json::json!({
                    "port": svc.port,
                    "protocol": svc.protocol,
                    "service_name": svc.service_name,
                })
                .to_string();
                events.push(make_event(
                    network_id,
                    Some(&after_dev.id),
                    EventType::PortOpened,
                    Severity::Info,
                    Some("New port open".to_string()),
                    Some(data),
                    scan_now,
                ));
            }
        }
    }

    Ok(events)
}

/// Re-querya los devices vistos este scan (`last_seen_at == scan_now`).
fn after_devices(
    tx: &Transaction,
    network_id: &str,
    scan_now: &str,
) -> DbResult<Vec<DeviceSnapshot>> {
    let mut stmt = tx
        .prepare(
            "SELECT id, primary_ip, first_seen_at, is_online FROM devices \
             WHERE network_id = ?1 AND last_seen_at = ?2",
        )
        .map_err(map_sqlite)?;
    let rows = stmt
        .query_map(rusqlite::params![network_id, scan_now], |row| {
            Ok(DeviceSnapshot {
                id: row.get(0)?,
                primary_ip: row.get(1)?,
                first_seen_at: row.get(2)?,
                is_online: row.get::<_, i64>(3)? != 0,
            })
        })
        .map_err(map_sqlite)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_sqlite)?);
    }
    Ok(out)
}

/// Construye un `Event` con id nuevo (`util::new_id`) y `created_at` del scan.
fn make_event(
    network_id: &str,
    device_id: Option<&str>,
    event_type: EventType,
    severity: Severity,
    message: Option<String>,
    data_json: Option<String>,
    created_at: &str,
) -> Event {
    Event {
        id: new_id(),
        network_id: network_id.to_string(),
        device_id: device_id.map(str::to_string),
        event_type,
        severity,
        message,
        data_json,
        created_at: created_at.to_string(),
    }
}
