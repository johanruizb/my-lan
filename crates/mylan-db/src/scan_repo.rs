//! Repositorio de escaneos (`scans`).
//!
//! Registrar un escaneo lo inserta con estado `running` y `started_at`; al
//! terminar se completa `finished_at`, `status` y opcionalmente el resumen
//! serializado en `summary_json`.

use rusqlite::Connection;

use mylan_core::{Scan, ScanStatus, ScanSummary};

use crate::codec::{enum_from_db, enum_to_db};
use crate::error::{map_sqlite, DbResult};

/// Inserta un escaneo (estado inicial normalmente `running`).
pub fn insert_scan(conn: &Connection, scan: &Scan) -> DbResult<()> {
    let summary = match &scan.summary {
        Some(s) => Some(serde_json::to_string(s)?),
        None => None,
    };
    conn.execute(
        "INSERT INTO scans (
           id, network_id, scan_type, target_ip, profile, status, started_at, finished_at,
           summary_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            scan.id,
            scan.network_id,
            enum_to_db(&scan.scan_type)?,
            scan.target_ip,
            enum_to_db(&scan.profile)?,
            enum_to_db(&scan.status)?,
            scan.started_at,
            scan.finished_at,
            summary,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Marca un escaneo como finalizado: actualiza `status`, `finished_at` y el
/// resumen (opcional). No afecta a escaneos ya completados si así se desea; la
/// llamada es directa (sin filtro de estado previo).
pub fn finish_scan(
    conn: &Connection,
    id: &str,
    status: ScanStatus,
    finished_at: &str,
    summary: Option<&ScanSummary>,
) -> DbResult<()> {
    let summary_json = match summary {
        Some(s) => Some(serde_json::to_string(s)?),
        None => None,
    };
    conn.execute(
        "UPDATE scans SET status = ?1, finished_at = ?2, summary_json = ?3 WHERE id = ?4",
        rusqlite::params![enum_to_db(&status)?, finished_at, summary_json, id],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Resumen ligero de un escaneo para listados de historial (AC-17 IPC `list_scans`).
///
/// No mapea `summary_json` a `ScanSummary` completo (el frontend sólo necesita
/// `hosts_alive`/`hosts_new`/`open_ports`, ya deserializados aquí); evita
/// re-parsear el JSON completo por fila. Read-only.
///
/// `scan_type`/`target_ip` vienen de columnas (no del JSON): distinguen un
/// escaneo de descubrimiento (`scan_type="discovery"`, `target_ip=None`) de un
/// escaneo de puertos (`scan_type="ports"`, `target_ip=Some(ip)`), para que la
/// UI pueda renderizar el target y linkar a `/devices/:ip`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRow {
    pub id: String,
    pub scan_type: String,
    pub target_ip: Option<String>,
    pub profile: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub hosts_alive: u32,
    pub hosts_new: u32,
    pub open_ports: u32,
}

/// Lista los escaneos ordenados por `started_at` descendente (más reciente
/// primero). Read-only: sólo `SELECT`. Los campos `hosts_alive`/`hosts_new`/
/// `open_ports` se extraen del `summary_json` cuando existe; si no, valen 0
/// (`ScanSummary::default` vía `#[serde(default)]` en `open_ports` cubre
/// `summary_json` viejos sin el campo).
pub fn list_scans(conn: &Connection) -> DbResult<Vec<ScanRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, scan_type, target_ip, profile, status, started_at, finished_at, summary_json
         FROM scans ORDER BY started_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let scan_type: String = row.get(1)?;
        let target_ip: Option<String> = row.get(2)?;
        let profile: String = row.get(3)?;
        let status: String = row.get(4)?;
        let started_at: String = row.get(5)?;
        let finished_at: Option<String> = row.get(6)?;
        let summary_raw: Option<String> = row.get(7)?;
        let (hosts_alive, hosts_new, open_ports) = match summary_raw.as_deref() {
            Some(s) if !s.is_empty() => {
                let summary: ScanSummary = serde_json::from_str(s).unwrap_or_default();
                (summary.hosts_alive, summary.hosts_new, summary.open_ports)
            }
            _ => (0, 0, 0),
        };
        Ok(ScanRow {
            id,
            scan_type,
            target_ip,
            profile,
            status,
            started_at,
            finished_at,
            hosts_alive,
            hosts_new,
            open_ports,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Lee un escaneo por su `id`.
pub fn get_scan(conn: &Connection, id: &str) -> DbResult<Option<Scan>> {
    let result = conn.query_row(
        "SELECT id, network_id, scan_type, target_ip, profile, status, started_at, finished_at,
         summary_json
         FROM scans WHERE id = ?1",
        [id],
        |row| {
            Ok::<_, rusqlite::Error>((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        },
    );
    match result {
        Ok((
            id,
            network_id,
            scan_type_raw,
            target_ip,
            profile_raw,
            status_raw,
            started_at,
            finished_at,
            summary_raw,
        )) => {
            let summary = match summary_raw {
                Some(s) if !s.is_empty() => Some(serde_json::from_str::<ScanSummary>(&s)?),
                _ => None,
            };
            Ok(Some(Scan {
                id,
                network_id,
                target_ip,
                scan_type: enum_from_db(&scan_type_raw)?,
                profile: enum_from_db(&profile_raw)?,
                status: enum_from_db(&status_raw)?,
                started_at,
                finished_at,
                summary,
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_sqlite(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::connect;
    use crate::network_repo::upsert_network;
    use mylan_core::{ScanKind, ScanProfile};

    fn ip(s: &str) -> std::net::IpAddr {
        s.parse().unwrap()
    }

    fn fixture(dir: &std::path::Path) -> Connection {
        let conn = connect(dir.join("scan.db")).unwrap();
        upsert_network(
            &conn,
            &mylan_core::Network {
                id: "net-1".to_string(),
                name: "home".to_string(),
                cidr: "192.168.1.0/24".to_string(),
                gateway_ip: Some(ip("192.168.1.1")),
                dns_servers: vec![],
                created_at: "2026-06-27T00:00:00Z".to_string(),
                updated_at: "2026-06-27T00:00:00Z".to_string(),
            },
        )
        .unwrap();
        conn
    }

    #[test]
    fn insert_then_finish_scan() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture(dir.path());
        let scan = Scan {
            id: "scan-1".to_string(),
            network_id: "net-1".to_string(),
            target_ip: None,
            scan_type: ScanKind::Discovery,
            profile: ScanProfile::Quick,
            status: ScanStatus::Running,
            started_at: "2026-06-27T00:00:00Z".to_string(),
            finished_at: None,
            summary: None,
        };
        insert_scan(&conn, &scan).unwrap();

        let summary = ScanSummary {
            hosts_alive: 14,
            hosts_new: 2,
            duration_ms: 18_500,
            open_ports: 0,
        };
        finish_scan(
            &conn,
            "scan-1",
            ScanStatus::Completed,
            "2026-06-27T00:00:20Z",
            Some(&summary),
        )
        .unwrap();

        let back = get_scan(&conn, "scan-1").unwrap().unwrap();
        assert_eq!(back.status, ScanStatus::Completed);
        assert_eq!(back.finished_at.as_deref(), Some("2026-06-27T00:00:20Z"));
        assert_eq!(back.summary, Some(summary));
    }

    #[test]
    fn get_scan_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture(dir.path());
        assert!(get_scan(&conn, "nope").unwrap().is_none());
    }

    #[test]
    fn ports_scan_persists_target_ip_and_open_ports() {
        // Un escaneo de puertos persiste target_ip + scan_type=ports; al
        // finalizar, list_scans devuelve open_ports desde summary_json.
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture(dir.path());
        let scan = Scan {
            id: "scan-ports-1".to_string(),
            network_id: "net-1".to_string(),
            target_ip: Some("192.168.1.42".to_string()),
            scan_type: ScanKind::Ports,
            profile: ScanProfile::Normal,
            status: ScanStatus::Running,
            started_at: "2026-07-07T00:00:00Z".to_string(),
            finished_at: None,
            summary: None,
        };
        insert_scan(&conn, &scan).unwrap();

        let summary = ScanSummary {
            hosts_alive: 0,
            hosts_new: 0,
            duration_ms: 3_200,
            open_ports: 5,
        };
        finish_scan(
            &conn,
            "scan-ports-1",
            ScanStatus::Completed,
            "2026-07-07T00:00:03Z",
            Some(&summary),
        )
        .unwrap();

        // get_scan devuelve target_ip y scan_type correctos.
        let back = get_scan(&conn, "scan-ports-1").unwrap().unwrap();
        assert_eq!(back.scan_type, ScanKind::Ports);
        assert_eq!(back.target_ip.as_deref(), Some("192.168.1.42"));
        assert_eq!(back.summary.as_ref().unwrap().open_ports, 5);

        // list_scans expone scan_type, target_ip y open_ports.
        let rows = list_scans(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.scan_type, "ports");
        assert_eq!(row.target_ip.as_deref(), Some("192.168.1.42"));
        assert_eq!(row.open_ports, 5);
        assert_eq!(row.profile, "normal");
        assert_eq!(row.status, "completed");
    }

    #[test]
    fn discovery_scan_list_has_null_target_ip() {
        // Descubrimiento: target_ip None, scan_type "discovery", open_ports 0.
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture(dir.path());
        insert_scan(
            &conn,
            &Scan {
                id: "scan-disc-1".to_string(),
                network_id: "net-1".to_string(),
                target_ip: None,
                scan_type: ScanKind::Discovery,
                profile: ScanProfile::Quick,
                status: ScanStatus::Running,
                started_at: "2026-07-07T00:00:00Z".to_string(),
                finished_at: None,
                summary: None,
            },
        )
        .unwrap();
        let rows = list_scans(&conn).unwrap();
        let row = &rows[0];
        assert_eq!(row.scan_type, "discovery");
        assert!(row.target_ip.is_none());
        assert_eq!(row.open_ports, 0);
    }

    #[test]
    fn list_scans_old_summary_without_open_ports_defaults_zero() {
        // Backward-compat: un summary_json viejo (pre-v0.5.4) sin `open_ports`
        // deserializa a 0 vía #[serde(default)] en ScanSummary.
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture(dir.path());
        conn.execute(
            "INSERT INTO scans (id, network_id, scan_type, profile, status, started_at,
                 finished_at, summary_json)
             VALUES ('scan-old','net-1','discovery','quick','completed','t0','t1',
                     '{\"hosts_alive\":2,\"hosts_new\":0,\"duration_ms\":500}')",
            [],
        )
        .unwrap();
        let rows = list_scans(&conn).unwrap();
        let row = &rows[0];
        assert_eq!(row.hosts_alive, 2);
        assert_eq!(row.open_ports, 0);
    }
}
