//! Repositorio de hallazgos de seguridad y auditoría de riesgos (`security_findings`).

use rusqlite::{params, Connection, Row};

use mylan_core::{FindingStatus, RiskSeverity, SecurityFinding};

use crate::error::{map_sqlite, DbResult};
use crate::util::{new_id, now_rfc3339};

struct FindingRow {
    id: String,
    device_id: Option<String>,
    network_id: String,
    rule_id: String,
    severity: String,
    title: String,
    description: String,
    evidence: Option<String>,
    remediation_advice: Option<String>,
    status: String,
    first_seen_at: String,
    last_seen_at: String,
    resolved_at: Option<String>,
}

impl FindingRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            device_id: row.get(1)?,
            network_id: row.get(2)?,
            rule_id: row.get(3)?,
            severity: row.get(4)?,
            title: row.get(5)?,
            description: row.get(6)?,
            evidence: row.get(7)?,
            remediation_advice: row.get(8)?,
            status: row.get(9)?,
            first_seen_at: row.get(10)?,
            last_seen_at: row.get(11)?,
            resolved_at: row.get(12)?,
        })
    }

    fn decode(self) -> SecurityFinding {
        let severity = RiskSeverity::parse_str(&self.severity).unwrap_or(RiskSeverity::Low);
        let status = FindingStatus::parse_str(&self.status).unwrap_or(FindingStatus::Open);

        SecurityFinding {
            id: self.id,
            device_id: self.device_id,
            network_id: self.network_id,
            rule_id: self.rule_id,
            severity,
            title: self.title,
            description: self.description,
            evidence: self.evidence,
            remediation_advice: self.remediation_advice,
            status,
            first_seen_at: self.first_seen_at,
            last_seen_at: self.last_seen_at,
            resolved_at: self.resolved_at,
        }
    }
}

const SELECT_FINDING_COLS: &str = "\
  id, device_id, network_id, rule_id, severity, title, description, \
  evidence, remediation_advice, status, first_seen_at, last_seen_at, resolved_at \
";

/// Inserta o actualiza un hallazgo de seguridad según la clave única
/// `(network_id, coalesce(device_id, ''), rule_id)`.
///
/// Si el hallazgo ya existía:
/// - Si estaba en `resolved`, se reabre (`open`) limpiando `resolved_at`.
/// - Si estaba en `acknowledged` o `ignored`, preserva el estado del usuario.
/// - Actualiza `last_seen_at`, `evidence`, `remediation_advice`, `title`, `description` y `severity`.
pub fn upsert_finding(conn: &Connection, finding: &SecurityFinding) -> DbResult<String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, status, first_seen_at FROM security_findings \
             WHERE network_id = ?1 AND coalesce(device_id, '') = coalesce(?2, '') AND rule_id = ?3",
        )
        .map_err(map_sqlite)?;

    let dev_id_param = finding.device_id.as_deref();
    let existing = match stmt.query_row(
        params![finding.network_id, dev_id_param, finding.rule_id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        },
    ) {
        Ok(row) => Some(row),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err(map_sqlite(e)),
    };

    if let Some((existing_id, existing_status_str, _)) = existing {
        let existing_status =
            FindingStatus::parse_str(&existing_status_str).unwrap_or(FindingStatus::Open);

        // Si estaba resuelto pero la vulnerabilidad vuelve a detectarse, reabrimos
        let (new_status, new_resolved_at) = match existing_status {
            FindingStatus::Resolved => (FindingStatus::Open, None::<String>),
            other => (other, None::<String>),
        };

        conn.execute(
            "UPDATE security_findings SET \
             severity = ?1, \
             title = ?2, \
             description = ?3, \
             evidence = ?4, \
             remediation_advice = ?5, \
             status = ?6, \
             last_seen_at = ?7, \
             resolved_at = ?8 \
             WHERE id = ?9",
            params![
                finding.severity.as_str(),
                finding.title,
                finding.description,
                finding.evidence,
                finding.remediation_advice,
                new_status.as_str(),
                finding.last_seen_at,
                new_resolved_at,
                existing_id,
            ],
        )
        .map_err(map_sqlite)?;

        Ok(existing_id)
    } else {
        let id = if finding.id.is_empty() {
            new_id()
        } else {
            finding.id.clone()
        };

        conn.execute(
            "INSERT INTO security_findings ( \
             id, device_id, network_id, rule_id, severity, title, description, \
             evidence, remediation_advice, status, first_seen_at, last_seen_at, resolved_at \
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                id,
                finding.device_id,
                finding.network_id,
                finding.rule_id,
                finding.severity.as_str(),
                finding.title,
                finding.description,
                finding.evidence,
                finding.remediation_advice,
                finding.status.as_str(),
                finding.first_seen_at,
                finding.last_seen_at,
                finding.resolved_at,
            ],
        )
        .map_err(map_sqlite)?;

        Ok(id)
    }
}

/// Actualiza el estado del ciclo de vida de un hallazgo (`open`, `acknowledged`, `ignored`, `resolved`).
///
/// Si el nuevo estado es `resolved`, establece `resolved_at = now`.
/// En cualquier otro estado, limpia `resolved_at = NULL`.
pub fn update_finding_status(
    conn: &Connection,
    finding_id: &str,
    new_status: FindingStatus,
) -> DbResult<bool> {
    let now = now_rfc3339().unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    let resolved_at = if new_status == FindingStatus::Resolved {
        Some(now)
    } else {
        None
    };

    let count = conn
        .execute(
            "UPDATE security_findings SET status = ?1, resolved_at = ?2 WHERE id = ?3",
            params![new_status.as_str(), resolved_at, finding_id],
        )
        .map_err(map_sqlite)?;

    Ok(count > 0)
}

/// Obtiene un hallazgo por su identificador único.
pub fn get_finding_by_id(conn: &Connection, id: &str) -> DbResult<Option<SecurityFinding>> {
    let sql = format!("SELECT {SELECT_FINDING_COLS} FROM security_findings WHERE id = ?1");
    let mut stmt = conn.prepare(&sql).map_err(map_sqlite)?;
    let mut rows = stmt
        .query_map(params![id], FindingRow::from_row)
        .map_err(map_sqlite)?;

    if let Some(row) = rows.next() {
        let r = row.map_err(map_sqlite)?;
        Ok(Some(r.decode()))
    } else {
        Ok(None)
    }
}

/// Lista los hallazgos asociados a una red, ordenados por severidad (Critical primero) y fecha.
pub fn list_findings_by_network(
    conn: &Connection,
    network_id: &str,
) -> DbResult<Vec<SecurityFinding>> {
    let sql = format!(
        "SELECT {SELECT_FINDING_COLS} FROM security_findings \
         WHERE network_id = ?1 \
         ORDER BY \
           CASE lower(severity) \
             WHEN 'critical' THEN 1 \
             WHEN 'high' THEN 2 \
             WHEN 'medium' THEN 3 \
             WHEN 'low' THEN 4 \
             ELSE 5 \
           END ASC, \
           first_seen_at DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map(params![network_id], FindingRow::from_row)
        .map_err(map_sqlite)?;

    let mut findings = Vec::new();
    for r in rows {
        findings.push(r.map_err(map_sqlite)?.decode());
    }
    Ok(findings)
}

/// Lista los hallazgos asociados a un dispositivo específico.
pub fn list_findings_by_device(
    conn: &Connection,
    device_id: &str,
) -> DbResult<Vec<SecurityFinding>> {
    let sql = format!(
        "SELECT {SELECT_FINDING_COLS} FROM security_findings \
         WHERE device_id = ?1 \
         ORDER BY \
           CASE lower(severity) \
             WHEN 'critical' THEN 1 \
             WHEN 'high' THEN 2 \
             WHEN 'medium' THEN 3 \
             WHEN 'low' THEN 4 \
             ELSE 5 \
           END ASC, \
           first_seen_at DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map(params![device_id], FindingRow::from_row)
        .map_err(map_sqlite)?;

    let mut findings = Vec::new();
    for r in rows {
        findings.push(r.map_err(map_sqlite)?.decode());
    }
    Ok(findings)
}

/// Calcula el Score Global de Seguridad (0–100) para una red (§4).
///
/// Fórmula: `Score = max(0, 100 - sum(penalizaciones))`.
/// Sólo los hallazgos con `status == FindingStatus::Open` restan puntos.
/// `Critical = -25`, `High = -15`, `Medium = -5`, `Low = -1`.
pub fn calculate_network_score(conn: &Connection, network_id: &str) -> DbResult<u32> {
    let sql = "SELECT severity FROM security_findings WHERE network_id = ?1 AND status = 'open'";
    let mut stmt = conn.prepare(sql).map_err(map_sqlite)?;
    let severities = stmt
        .query_map(params![network_id], |r| r.get::<_, String>(0))
        .map_err(map_sqlite)?;

    let mut total_penalty = 0u32;
    for sev_res in severities {
        let sev_str = sev_res.map_err(map_sqlite)?;
        if let Some(sev) = RiskSeverity::parse_str(&sev_str) {
            total_penalty += sev.penalty_points();
        }
    }

    Ok(100u32.saturating_sub(total_penalty))
}

/// Auto-resuelve hallazgos que ya no están presentes en una evaluación reciente.
///
/// Si un hallazgo activo (`open`, `acknowledged`, `ignored`) en la red para un dispositivo
/// (o red si `device_id` es None) ya no está en `active_rule_ids`, se conmuta automáticamente
/// a `resolved` con timestamp `resolved_at = now`.
pub fn auto_resolve_missing_device_findings(
    conn: &Connection,
    device_id: &str,
    active_rule_ids: &[&str],
    now: &str,
) -> DbResult<usize> {
    let sql =
        "SELECT id, rule_id FROM security_findings WHERE device_id = ?1 AND status != 'resolved'";
    let mut stmt = conn.prepare(sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map(params![device_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(map_sqlite)?;

    let mut to_resolve = Vec::new();
    for row in rows {
        let (id, rule_id) = row.map_err(map_sqlite)?;
        if !active_rule_ids.contains(&rule_id.as_str()) {
            to_resolve.push(id);
        }
    }

    for id in &to_resolve {
        conn.execute(
            "UPDATE security_findings SET status = 'resolved', resolved_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(map_sqlite)?;
    }

    Ok(to_resolve.len())
}

/// Auto-resuelve hallazgos de red (`device_id IS NULL`) que ya no se detectan.
///
/// Los hallazgos de red (p. ej. SEC-008 DNS) no tienen dispositivo y el
/// auto-resolvedor por dispositivo nunca los ve. Sin esto, un cambio de
/// configuración deja el hallazgo abierto para siempre y penaliza el score.
pub fn auto_resolve_missing_network_findings(
    conn: &Connection,
    network_id: &str,
    active_rule_ids: &[&str],
    now: &str,
) -> DbResult<usize> {
    let sql = "SELECT id, rule_id FROM security_findings \
               WHERE network_id = ?1 AND device_id IS NULL AND status != 'resolved'";
    let mut stmt = conn.prepare(sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map(params![network_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(map_sqlite)?;

    let mut to_resolve = Vec::new();
    for row in rows {
        let (id, rule_id) = row.map_err(map_sqlite)?;
        if !active_rule_ids.contains(&rule_id.as_str()) {
            to_resolve.push(id);
        }
    }

    for id in &to_resolve {
        conn.execute(
            "UPDATE security_findings SET status = 'resolved', resolved_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(map_sqlite)?;
    }

    Ok(to_resolve.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::run_migrations;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();

        // Crear network y device base para FKs
        conn.execute(
            "INSERT INTO networks (id, name, cidr, created_at, updated_at) \
             VALUES ('net-1', 'TestNet', '192.168.1.0/24', '2026-10-16T12:00:00Z', '2026-10-16T12:00:00Z')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO devices (id, network_id, primary_mac, first_seen_at, last_seen_at) \
             VALUES ('dev-1', 'net-1', 'aa:bb:cc:dd:ee:01', '2026-10-16T12:00:00Z', '2026-10-16T12:00:00Z')",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn upsert_inserts_new_finding_and_lists() {
        let conn = setup_test_db();
        let finding = SecurityFinding {
            id: "find-1".to_string(),
            device_id: Some("dev-1".to_string()),
            network_id: "net-1".to_string(),
            rule_id: "SEC-001-TELNET-OPEN".to_string(),
            severity: RiskSeverity::Critical,
            title: "Telnet abierto".to_string(),
            description: "Texto plano".to_string(),
            evidence: Some("Port 23 open".to_string()),
            remediation_advice: Some("Disable telnet".to_string()),
            status: FindingStatus::Open,
            first_seen_at: "2026-10-16T12:00:00Z".to_string(),
            last_seen_at: "2026-10-16T12:00:00Z".to_string(),
            resolved_at: None,
        };

        upsert_finding(&conn, &finding).expect("upsert ok");

        let loaded = get_finding_by_id(&conn, "find-1").expect("get ok").unwrap();
        assert_eq!(loaded.rule_id, "SEC-001-TELNET-OPEN");
        assert_eq!(loaded.severity, RiskSeverity::Critical);
        assert_eq!(loaded.status, FindingStatus::Open);

        let by_dev = list_findings_by_device(&conn, "dev-1").unwrap();
        assert_eq!(by_dev.len(), 1);

        let by_net = list_findings_by_network(&conn, "net-1").unwrap();
        assert_eq!(by_net.len(), 1);
    }

    #[test]
    fn upsert_preserves_user_status_acknowledged_and_ignored() {
        let conn = setup_test_db();
        let mut finding = SecurityFinding {
            id: "find-1".to_string(),
            device_id: Some("dev-1".to_string()),
            network_id: "net-1".to_string(),
            rule_id: "SEC-001-TELNET-OPEN".to_string(),
            severity: RiskSeverity::Critical,
            title: "Telnet abierto".to_string(),
            description: "Texto plano".to_string(),
            evidence: Some("Port 23 open".to_string()),
            remediation_advice: None,
            status: FindingStatus::Open,
            first_seen_at: "2026-10-16T12:00:00Z".to_string(),
            last_seen_at: "2026-10-16T12:00:00Z".to_string(),
            resolved_at: None,
        };

        upsert_finding(&conn, &finding).unwrap();

        // Usuario acepta el riesgo
        update_finding_status(&conn, "find-1", FindingStatus::Acknowledged).unwrap();
        let loaded = get_finding_by_id(&conn, "find-1").unwrap().unwrap();
        assert_eq!(loaded.status, FindingStatus::Acknowledged);

        // Nuevo escaneo detecta de nuevo el hallazgo
        finding.last_seen_at = "2026-10-16T13:00:00Z".to_string();
        upsert_finding(&conn, &finding).unwrap();

        // Debe mantenerse Acknowledged, no volver a Open
        let after_rescan = get_finding_by_id(&conn, "find-1").unwrap().unwrap();
        assert_eq!(after_rescan.status, FindingStatus::Acknowledged);
        assert_eq!(after_rescan.last_seen_at, "2026-10-16T13:00:00Z");
    }

    #[test]
    fn upsert_reopens_resolved_finding_if_vulnerability_returns() {
        let conn = setup_test_db();
        let mut finding = SecurityFinding {
            id: "find-1".to_string(),
            device_id: Some("dev-1".to_string()),
            network_id: "net-1".to_string(),
            rule_id: "SEC-001-TELNET-OPEN".to_string(),
            severity: RiskSeverity::Critical,
            title: "Telnet abierto".to_string(),
            description: "Texto plano".to_string(),
            evidence: None,
            remediation_advice: None,
            status: FindingStatus::Open,
            first_seen_at: "2026-10-16T12:00:00Z".to_string(),
            last_seen_at: "2026-10-16T12:00:00Z".to_string(),
            resolved_at: None,
        };

        upsert_finding(&conn, &finding).unwrap();

        // Se resuelve
        update_finding_status(&conn, "find-1", FindingStatus::Resolved).unwrap();
        let loaded = get_finding_by_id(&conn, "find-1").unwrap().unwrap();
        assert_eq!(loaded.status, FindingStatus::Resolved);
        assert!(loaded.resolved_at.is_some());

        // Vulnerabilidad regresa
        finding.last_seen_at = "2026-10-16T14:00:00Z".to_string();
        upsert_finding(&conn, &finding).unwrap();

        let reopened = get_finding_by_id(&conn, "find-1").unwrap().unwrap();
        assert_eq!(reopened.status, FindingStatus::Open);
        assert_eq!(reopened.resolved_at, None);
    }

    #[test]
    fn calculate_network_score_deducts_only_open_findings() {
        let conn = setup_test_db();

        // Sin hallazgos: Score = 100
        assert_eq!(calculate_network_score(&conn, "net-1").unwrap(), 100);

        // 1 Critical (-25) + 1 High (-15) + 1 Medium (-5) + 1 Low (-1) = 100 - 46 = 54
        upsert_finding(
            &conn,
            &SecurityFinding {
                id: "f-1".to_string(),
                device_id: Some("dev-1".to_string()),
                network_id: "net-1".to_string(),
                rule_id: "SEC-001".to_string(),
                severity: RiskSeverity::Critical,
                title: "Crit".to_string(),
                description: "".to_string(),
                evidence: None,
                remediation_advice: None,
                status: FindingStatus::Open,
                first_seen_at: "2026-10-16T12:00:00Z".to_string(),
                last_seen_at: "2026-10-16T12:00:00Z".to_string(),
                resolved_at: None,
            },
        )
        .unwrap();

        upsert_finding(
            &conn,
            &SecurityFinding {
                id: "f-2".to_string(),
                device_id: Some("dev-1".to_string()),
                network_id: "net-1".to_string(),
                rule_id: "SEC-002".to_string(),
                severity: RiskSeverity::High,
                title: "High".to_string(),
                description: "".to_string(),
                evidence: None,
                remediation_advice: None,
                status: FindingStatus::Open,
                first_seen_at: "2026-10-16T12:00:00Z".to_string(),
                last_seen_at: "2026-10-16T12:00:00Z".to_string(),
                resolved_at: None,
            },
        )
        .unwrap();

        assert_eq!(calculate_network_score(&conn, "net-1").unwrap(), 60);

        // Si el usuario ignora f-1 (Critical), ya no penaliza
        update_finding_status(&conn, "f-1", FindingStatus::Ignored).unwrap();
        assert_eq!(calculate_network_score(&conn, "net-1").unwrap(), 85);

        // Si f-2 se resuelve, score vuelve a 100
        update_finding_status(&conn, "f-2", FindingStatus::Resolved).unwrap();
        assert_eq!(calculate_network_score(&conn, "net-1").unwrap(), 100);
    }

    #[test]
    fn auto_resolve_missing_device_findings_works() {
        let conn = setup_test_db();
        upsert_finding(
            &conn,
            &SecurityFinding {
                id: "f-1".to_string(),
                device_id: Some("dev-1".to_string()),
                network_id: "net-1".to_string(),
                rule_id: "SEC-001".to_string(),
                severity: RiskSeverity::Critical,
                title: "".to_string(),
                description: "".to_string(),
                evidence: None,
                remediation_advice: None,
                status: FindingStatus::Open,
                first_seen_at: "2026-10-16T12:00:00Z".to_string(),
                last_seen_at: "2026-10-16T12:00:00Z".to_string(),
                resolved_at: None,
            },
        )
        .unwrap();

        upsert_finding(
            &conn,
            &SecurityFinding {
                id: "f-2".to_string(),
                device_id: Some("dev-1".to_string()),
                network_id: "net-1".to_string(),
                rule_id: "SEC-002".to_string(),
                severity: RiskSeverity::High,
                title: "".to_string(),
                description: "".to_string(),
                evidence: None,
                remediation_advice: None,
                status: FindingStatus::Open,
                first_seen_at: "2026-10-16T12:00:00Z".to_string(),
                last_seen_at: "2026-10-16T12:00:00Z".to_string(),
                resolved_at: None,
            },
        )
        .unwrap();

        // En la re-evaluación, sólo SEC-002 sigue activo (SEC-001 ya no)
        let resolved_count = auto_resolve_missing_device_findings(
            &conn,
            "dev-1",
            &["SEC-002"],
            "2026-10-16T15:00:00Z",
        )
        .unwrap();
        assert_eq!(resolved_count, 1);

        let f1 = get_finding_by_id(&conn, "f-1").unwrap().unwrap();
        assert_eq!(f1.status, FindingStatus::Resolved);
        assert_eq!(f1.resolved_at.as_deref(), Some("2026-10-16T15:00:00Z"));

        let f2 = get_finding_by_id(&conn, "f-2").unwrap().unwrap();
        assert_eq!(f2.status, FindingStatus::Open);
    }
}
