//! Repositorio para muestras de salud de red y pruebas de velocidad (Pilar 5 / Fase 9).

use rusqlite::{params, Connection};

use mylan_core::{NetworkHealthSample, SpeedTestResult};

use crate::error::{map_sqlite, DbResult};

/// Inserta una nueva muestra puntual de salud de red.
pub fn insert_health_sample(conn: &Connection, sample: &NetworkHealthSample) -> DbResult<()> {
    conn.execute(
        "INSERT INTO network_health_samples (
            id, network_id, target_type, target_ip, rtt_ms, jitter_ms,
            loss_pct, sample_count, probes_failed, method, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            sample.id,
            sample.network_id,
            sample.target_type,
            sample.target_ip,
            sample.rtt_ms.map(|v| v as i64),
            sample.jitter_ms,
            sample.loss_pct,
            sample.sample_count,
            sample.probes_failed,
            sample.method,
            sample.created_at,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Obtiene las muestras más recientes de salud para una red, ordenadas descendentemente por `created_at`.
pub fn list_health_samples(
    conn: &Connection,
    network_id: &str,
    limit: usize,
) -> DbResult<Vec<NetworkHealthSample>> {
    let limit_i64 = limit.max(1) as i64;
    let mut stmt = conn
        .prepare(
            "SELECT id, network_id, target_type, target_ip, rtt_ms, jitter_ms,
                    loss_pct, sample_count, probes_failed, method, created_at
             FROM network_health_samples
             WHERE network_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .map_err(map_sqlite)?;

    let rows = stmt
        .query_map(params![network_id, limit_i64], |row| {
            let rtt_opt: Option<i64> = row.get(4)?;
            Ok(NetworkHealthSample {
                id: row.get(0)?,
                network_id: row.get(1)?,
                target_type: row.get(2)?,
                target_ip: row.get(3)?,
                rtt_ms: rtt_opt.map(|v| v.max(0) as u64),
                jitter_ms: row.get(5)?,
                loss_pct: row.get(6)?,
                sample_count: row.get(7)?,
                probes_failed: row.get(8)?,
                method: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(map_sqlite)?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(map_sqlite)?);
    }
    Ok(result)
}

/// Obtiene la muestra más reciente de un target_type específico (ej. "gateway" o "dns").
pub fn get_latest_health_sample(
    conn: &Connection,
    network_id: &str,
    target_type: Option<&str>,
) -> DbResult<Option<NetworkHealthSample>> {
    let sql = if target_type.is_some() {
        "SELECT id, network_id, target_type, target_ip, rtt_ms, jitter_ms,
                loss_pct, sample_count, probes_failed, method, created_at
         FROM network_health_samples
         WHERE network_id = ?1 AND target_type = ?2
         ORDER BY created_at DESC
         LIMIT 1"
    } else {
        "SELECT id, network_id, target_type, target_ip, rtt_ms, jitter_ms,
                loss_pct, sample_count, probes_failed, method, created_at
         FROM network_health_samples
         WHERE network_id = ?1
         ORDER BY created_at DESC
         LIMIT 1"
    };

    let mut stmt = conn.prepare(sql).map_err(map_sqlite)?;
    let mut rows = match target_type {
        Some(tt) => stmt.query_map(params![network_id, tt], row_to_health_sample),
        None => stmt.query_map(params![network_id], row_to_health_sample),
    }
    .map_err(map_sqlite)?;

    match rows.next() {
        Some(r) => Ok(Some(r.map_err(map_sqlite)?)),
        None => Ok(None),
    }
}

fn row_to_health_sample(row: &rusqlite::Row<'_>) -> rusqlite::Result<NetworkHealthSample> {
    let rtt_opt: Option<i64> = row.get(4)?;
    Ok(NetworkHealthSample {
        id: row.get(0)?,
        network_id: row.get(1)?,
        target_type: row.get(2)?,
        target_ip: row.get(3)?,
        rtt_ms: rtt_opt.map(|v| v.max(0) as u64),
        jitter_ms: row.get(5)?,
        loss_pct: row.get(6)?,
        sample_count: row.get(7)?,
        probes_failed: row.get(8)?,
        method: row.get(9)?,
        created_at: row.get(10)?,
    })
}

/// Inserta un nuevo resultado de test de velocidad.
pub fn insert_speed_test(conn: &Connection, test: &SpeedTestResult) -> DbResult<()> {
    conn.execute(
        "INSERT INTO speed_tests (
            id, network_id, server_name, server_host, server_city, server_country,
            download_mbps, upload_mbps, latency_ms, jitter_ms, bufferbloat_ms,
            retransmits, protocol, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            test.id,
            test.network_id,
            test.server_name,
            test.server_host,
            test.server_city,
            test.server_country,
            test.download_mbps,
            test.upload_mbps,
            test.latency_ms as i64,
            test.jitter_ms,
            test.bufferbloat_ms.map(|v| v as i64),
            test.retransmits,
            test.protocol,
            test.created_at,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Obtiene el historial de pruebas de velocidad ordenadas descendentemente por `created_at`.
pub fn list_speed_tests(
    conn: &Connection,
    network_id: Option<&str>,
    limit: usize,
) -> DbResult<Vec<SpeedTestResult>> {
    let limit_i64 = limit.max(1) as i64;
    let (sql, params_vec): (&str, Vec<rusqlite::types::Value>) = match network_id {
        Some(nid) => (
            "SELECT id, network_id, server_name, server_host, server_city, server_country,
                    download_mbps, upload_mbps, latency_ms, jitter_ms, bufferbloat_ms,
                    retransmits, protocol, created_at
             FROM speed_tests
             WHERE network_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
            vec![nid.to_string().into(), limit_i64.into()],
        ),
        None => (
            "SELECT id, network_id, server_name, server_host, server_city, server_country,
                    download_mbps, upload_mbps, latency_ms, jitter_ms, bufferbloat_ms,
                    retransmits, protocol, created_at
             FROM speed_tests
             ORDER BY created_at DESC
             LIMIT ?1",
            vec![limit_i64.into()],
        ),
    };

    let mut stmt = conn.prepare(sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec), row_to_speed_test)
        .map_err(map_sqlite)?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(map_sqlite)?);
    }
    Ok(result)
}

/// Obtiene el test de velocidad más reciente.
pub fn get_latest_speed_test(
    conn: &Connection,
    network_id: Option<&str>,
) -> DbResult<Option<SpeedTestResult>> {
    let list = list_speed_tests(conn, network_id, 1)?;
    Ok(list.into_iter().next())
}

fn row_to_speed_test(row: &rusqlite::Row<'_>) -> rusqlite::Result<SpeedTestResult> {
    let lat: i64 = row.get(8)?;
    let bb: Option<i64> = row.get(10)?;
    Ok(SpeedTestResult {
        id: row.get(0)?,
        network_id: row.get(1)?,
        server_name: row.get(2)?,
        server_host: row.get(3)?,
        server_city: row.get(4)?,
        server_country: row.get(5)?,
        download_mbps: row.get(6)?,
        upload_mbps: row.get(7)?,
        latency_ms: lat.max(0) as u64,
        jitter_ms: row.get(9)?,
        bufferbloat_ms: bb.map(|v| v.max(0) as u64),
        retransmits: row.get(11)?,
        protocol: row.get(12)?,
        created_at: row.get(13)?,
    })
}

/// Purga registros de salud y tests de velocidad más antiguos que `max_days` días.
/// Devuelve `(deleted_health_samples, deleted_speed_tests)`.
pub fn prune_health_and_speed_data(conn: &Connection, max_days: u32) -> DbResult<(usize, usize)> {
    let modifier = format!("-{max_days} days");
    let deleted_health = conn
        .execute(
            "DELETE FROM network_health_samples
             WHERE datetime(created_at) < datetime('now', ?1)",
            params![modifier],
        )
        .map_err(map_sqlite)?;

    let deleted_speed = conn
        .execute(
            "DELETE FROM speed_tests
             WHERE datetime(created_at) < datetime('now', ?1)",
            params![modifier],
        )
        .map_err(map_sqlite)?;

    Ok((deleted_health, deleted_speed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::run_migrations;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn.execute(
            "INSERT INTO networks (id, name, cidr, created_at, updated_at)
             VALUES ('net-test', 'Test LAN', '192.168.1.0/24', '2026-10-16T00:00:00Z', '2026-10-16T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_health_samples_crud() {
        let conn = setup_test_db();

        let sample1 = NetworkHealthSample {
            id: "s-1".to_string(),
            network_id: "net-test".to_string(),
            target_type: "gateway".to_string(),
            target_ip: "192.168.1.1".to_string(),
            rtt_ms: Some(2),
            jitter_ms: 0.5,
            loss_pct: 0.0,
            sample_count: 5,
            probes_failed: 0,
            method: "icmp".to_string(),
            created_at: "2026-10-16T10:00:00Z".to_string(),
        };

        let sample2 = NetworkHealthSample {
            id: "s-2".to_string(),
            network_id: "net-test".to_string(),
            target_type: "dns".to_string(),
            target_ip: "1.1.1.1".to_string(),
            rtt_ms: Some(15),
            jitter_ms: 1.8,
            loss_pct: 0.0,
            sample_count: 5,
            probes_failed: 0,
            method: "dns_udp".to_string(),
            created_at: "2026-10-16T10:01:00Z".to_string(),
        };

        insert_health_sample(&conn, &sample1).unwrap();
        insert_health_sample(&conn, &sample2).unwrap();

        let all = list_health_samples(&conn, "net-test", 10).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, "s-2"); // ordered DESC by created_at

        let latest_gw = get_latest_health_sample(&conn, "net-test", Some("gateway"))
            .unwrap()
            .unwrap();
        assert_eq!(latest_gw.id, "s-1");
        assert_eq!(latest_gw.target_ip, "192.168.1.1");
    }

    #[test]
    fn test_speed_tests_crud() {
        let conn = setup_test_db();

        let st1 = SpeedTestResult {
            id: "st-1".to_string(),
            network_id: Some("net-test".to_string()),
            server_name: "Cloudflare (BOG)".to_string(),
            server_host: "speed.cloudflare.com".to_string(),
            server_city: Some("Bogotá".to_string()),
            server_country: Some("CO".to_string()),
            download_mbps: 210.5,
            upload_mbps: 52.0,
            latency_ms: 8,
            jitter_ms: 1.2,
            bufferbloat_ms: Some(12),
            retransmits: Some(0),
            protocol: "cloudflare".to_string(),
            created_at: "2026-10-16T10:05:00Z".to_string(),
        };

        insert_speed_test(&conn, &st1).unwrap();

        let list = list_speed_tests(&conn, Some("net-test"), 10).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].server_city.as_deref(), Some("Bogotá"));
        assert_eq!(list[0].download_mbps, 210.5);

        let latest = get_latest_speed_test(&conn, None).unwrap().unwrap();
        assert_eq!(latest.id, "st-1");
    }

    #[test]
    fn test_prune_old_records() {
        let conn = setup_test_db();

        // Sample viejo (>30 días)
        let old_sample = NetworkHealthSample {
            id: "old-1".to_string(),
            network_id: "net-test".to_string(),
            target_type: "gateway".to_string(),
            target_ip: "192.168.1.1".to_string(),
            rtt_ms: Some(2),
            jitter_ms: 0.5,
            loss_pct: 0.0,
            sample_count: 5,
            probes_failed: 0,
            method: "icmp".to_string(),
            created_at: "2026-08-01T00:00:00Z".to_string(),
        };
        // Sample nuevo (hoy)
        let new_sample = NetworkHealthSample {
            id: "new-1".to_string(),
            network_id: "net-test".to_string(),
            target_type: "gateway".to_string(),
            target_ip: "192.168.1.1".to_string(),
            rtt_ms: Some(2),
            jitter_ms: 0.5,
            loss_pct: 0.0,
            sample_count: 5,
            probes_failed: 0,
            method: "icmp".to_string(),
            created_at: "2026-10-16T12:00:00Z".to_string(),
        };

        insert_health_sample(&conn, &old_sample).unwrap();
        insert_health_sample(&conn, &new_sample).unwrap();

        let (del_h, del_s) = prune_health_and_speed_data(&conn, 30).unwrap();
        assert_eq!(del_h, 1);
        assert_eq!(del_s, 0);

        let remaining = list_health_samples(&conn, "net-test", 10).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "new-1");
    }
}
