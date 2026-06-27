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
           id, network_id, scan_type, profile, status, started_at, finished_at, summary_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            scan.id,
            scan.network_id,
            enum_to_db(&scan.scan_type)?,
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

/// Lee un escaneo por su `id`.
pub fn get_scan(conn: &Connection, id: &str) -> DbResult<Option<Scan>> {
    let result = conn.query_row(
        "SELECT id, network_id, scan_type, profile, status, started_at, finished_at, summary_json
         FROM scans WHERE id = ?1",
        [id],
        |row| {
            Ok::<_, rusqlite::Error>((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        },
    );
    match result {
        Ok((
            id,
            network_id,
            scan_type_raw,
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
}
