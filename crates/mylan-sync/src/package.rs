//! Empaquetado, serializacion, exportacion e importacion de snapshots de Workspace.

use mylan_core::{
    Device, FindingStatus, Network, NetworkHealthSample, RiskSeverity, SecurityFinding,
    SpeedTestResult, Workspace,
};
use mylan_db::{
    device_repo::{upsert_device, UpsertOutcome},
    workspace_repo::{create_workspace, get_workspace, update_workspace},
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Error de serializacion JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Error de base de datos durante export/import: {0}")]
    DatabaseError(String),

    #[error("Workspace no encontrado localmente: {0}")]
    WorkspaceNotFound(String),
}

/// Metadatos del snapshot para verificacion de version y procedencia.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotMetadata {
    pub workspace_id: String,
    pub workspace_name: String,
    pub device_origin: String,
    pub app_version: String,
    pub schema_version: u32,
    pub snapshot_epoch: i64,
}

/// Snapshot autocontenido y cifrable de un Workspace especifico.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceSnapshot {
    pub metadata: SnapshotMetadata,
    pub workspace: Workspace,
    pub networks: Vec<Network>,
    pub devices: Vec<Device>,
    pub security_findings: Vec<SecurityFinding>,
    pub network_health_samples: Vec<NetworkHealthSample>,
    pub speed_tests: Vec<SpeedTestResult>,
}

impl WorkspaceSnapshot {
    /// Serializa el snapshot a JSON UTF-8.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, PackageError> {
        serde_json::to_vec(self).map_err(PackageError::JsonError)
    }

    /// Deserializa el snapshot a partir de bytes JSON.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, PackageError> {
        serde_json::from_slice(bytes).map_err(PackageError::JsonError)
    }
}

/// Resumen estadistico de la operacion de importacion/fusion.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportSummary {
    pub devices_imported: usize,
    pub devices_updated: usize,
    pub networks_imported: usize,
    pub findings_imported: usize,
    pub samples_imported: usize,
    pub speed_tests_imported: usize,
}

/// Exporta el estado completo de un Workspace desde SQLite hacia un `WorkspaceSnapshot`.
pub fn export_workspace_snapshot(
    conn: &Connection,
    workspace_id: &str,
    device_origin: &str,
    app_version: &str,
) -> Result<WorkspaceSnapshot, PackageError> {
    let ws = get_workspace(conn, workspace_id)
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?
        .ok_or_else(|| PackageError::WorkspaceNotFound(workspace_id.to_string()))?;

    let now_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let metadata = SnapshotMetadata {
        workspace_id: ws.id.clone(),
        workspace_name: ws.name.clone(),
        device_origin: device_origin.to_string(),
        app_version: app_version.to_string(),
        schema_version: 8,
        snapshot_epoch: now_epoch,
    };

    // 1. Redes del workspace
    let mut stmt = conn
        .prepare(
            "SELECT id, name, cidr, gateway_ip, dns_servers, created_at, updated_at
             FROM networks WHERE workspace_id = ?1",
        )
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    let networks_rows = stmt
        .query_map([workspace_id], |row| {
            let gw_str: Option<String> = row.get(3)?;
            let dns_str: Option<String> = row.get(4)?;
            let gateway_ip = gw_str.and_then(|s| s.parse().ok());
            let dns_servers = dns_str
                .map(|s| s.split(',').filter_map(|p| p.trim().parse().ok()).collect())
                .unwrap_or_default();

            Ok(Network {
                id: row.get(0)?,
                name: row.get(1)?,
                cidr: row.get(2)?,
                gateway_ip,
                dns_servers,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    let mut networks = Vec::new();
    for net in networks_rows {
        networks.push(net.map_err(|e| PackageError::DatabaseError(e.to_string()))?);
    }

    // 2. Dispositivos del workspace
    let devices = mylan_db::device_repo::list_devices_by_workspace(conn, workspace_id)
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    // 3. Hallazgos de seguridad del workspace
    let mut stmt = conn
        .prepare(
            "SELECT id, device_id, network_id, rule_id, severity, title, description, evidence, remediation_advice, status, first_seen_at, last_seen_at, resolved_at
             FROM security_findings WHERE workspace_id = ?1",
        )
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    let findings_rows = stmt
        .query_map([workspace_id], |row| {
            let sev_str: String = row.get(4)?;
            let status_str: String = row.get(9)?;
            let severity = match sev_str.as_str() {
                "Critical" => RiskSeverity::Critical,
                "High" => RiskSeverity::High,
                "Medium" => RiskSeverity::Medium,
                _ => RiskSeverity::Low,
            };
            let status = match status_str.as_str() {
                "acknowledged" => FindingStatus::Acknowledged,
                "ignored" => FindingStatus::Ignored,
                "resolved" => FindingStatus::Resolved,
                _ => FindingStatus::Open,
            };

            Ok(SecurityFinding {
                id: row.get(0)?,
                device_id: row.get(1)?,
                network_id: row.get(2)?,
                rule_id: row.get(3)?,
                severity,
                title: row.get(5)?,
                description: row.get(6)?,
                evidence: row.get(7)?,
                remediation_advice: row.get(8)?,
                status,
                first_seen_at: row.get(10)?,
                last_seen_at: row.get(11)?,
                resolved_at: row.get(12)?,
            })
        })
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    let mut security_findings = Vec::new();
    for f in findings_rows {
        security_findings.push(f.map_err(|e| PackageError::DatabaseError(e.to_string()))?);
    }

    // 4. Muestras de salud de red (ultimas 200)
    let mut stmt = conn
        .prepare(
            "SELECT id, network_id, target_type, target_ip, rtt_ms, jitter_ms, loss_pct, sample_count, probes_failed, method, created_at
             FROM network_health_samples WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT 200",
        )
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    let health_rows = stmt
        .query_map([workspace_id], |row| {
            let rtt_i64: Option<i64> = row.get(4)?;
            Ok(NetworkHealthSample {
                id: row.get(0)?,
                network_id: row.get(1)?,
                target_type: row.get(2)?,
                target_ip: row.get(3)?,
                rtt_ms: rtt_i64.map(|v| v as u64),
                jitter_ms: row.get(5)?,
                loss_pct: row.get(6)?,
                sample_count: row.get(7)?,
                probes_failed: row.get(8)?,
                method: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    let mut network_health_samples = Vec::new();
    for h in health_rows {
        network_health_samples.push(h.map_err(|e| PackageError::DatabaseError(e.to_string()))?);
    }

    // 5. Test de velocidad (ultimos 50)
    let mut stmt = conn
        .prepare(
            "SELECT id, network_id, server_name, server_host, server_city, server_country, download_mbps, upload_mbps, latency_ms, jitter_ms, bufferbloat_ms, retransmits, protocol, created_at
             FROM speed_tests WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT 50",
        )
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    let speed_rows = stmt
        .query_map([workspace_id], |row| {
            let latency_i64: i64 = row.get(8)?;
            let bufferbloat_i64: Option<i64> = row.get(10)?;
            Ok(SpeedTestResult {
                id: row.get(0)?,
                network_id: row.get(1)?,
                server_name: row.get(2)?,
                server_host: row.get(3)?,
                server_city: row.get(4)?,
                server_country: row.get(5)?,
                download_mbps: row.get(6)?,
                upload_mbps: row.get(7)?,
                latency_ms: latency_i64 as u64,
                jitter_ms: row.get(9)?,
                bufferbloat_ms: bufferbloat_i64.map(|v| v as u64),
                retransmits: row.get(11)?,
                protocol: row.get(12)?,
                created_at: row.get(13)?,
            })
        })
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    let mut speed_tests = Vec::new();
    for s in speed_rows {
        speed_tests.push(s.map_err(|e| PackageError::DatabaseError(e.to_string()))?);
    }

    Ok(WorkspaceSnapshot {
        metadata,
        workspace: ws,
        networks,
        devices,
        security_findings,
        network_health_samples,
        speed_tests,
    })
}

/// Importa y reconcilia un `WorkspaceSnapshot` dentro de la base de datos local SQLite.
pub fn import_workspace_snapshot(
    conn: &mut Connection,
    snapshot: &WorkspaceSnapshot,
) -> Result<ImportSummary, PackageError> {
    let mut summary = ImportSummary::default();

    let tx = conn
        .transaction()
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    // 1. Reconciliar Workspace
    let existing_ws = get_workspace(&tx, &snapshot.workspace.id)
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    if existing_ws.is_none() {
        create_workspace(&tx, &snapshot.workspace)
            .map_err(|e| PackageError::DatabaseError(e.to_string()))?;
    } else {
        update_workspace(&tx, &snapshot.workspace)
            .map_err(|e| PackageError::DatabaseError(e.to_string()))?;
    }

    // 2. Reconciliar Redes
    for net in &snapshot.networks {
        let dns_str = net
            .dns_servers
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let inserted = tx.execute(
            "INSERT INTO networks (id, name, cidr, gateway_ip, dns_servers, created_at, updated_at, workspace_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                updated_at = MAX(networks.updated_at, excluded.updated_at)",
            rusqlite::params![
                net.id,
                net.name,
                net.cidr,
                net.gateway_ip.map(|ip| ip.to_string()),
                dns_str,
                net.created_at,
                net.updated_at,
                snapshot.workspace.id,
            ],
        ).map_err(|e| PackageError::DatabaseError(e.to_string()))?;

        if inserted > 0 {
            summary.networks_imported += 1;
        }
    }

    // 3. Reconciliar Dispositivos usando upsert_device
    for dev in &snapshot.devices {
        match upsert_device(&tx, dev) {
            Ok(outcome) => match outcome {
                UpsertOutcome::Inserted => {
                    summary.devices_imported += 1;
                    let _ = tx.execute(
                        "UPDATE devices SET workspace_id = ?1 WHERE id = ?2",
                        rusqlite::params![snapshot.workspace.id, dev.id],
                    );
                }
                UpsertOutcome::Updated => {
                    summary.devices_updated += 1;
                    let _ = tx.execute(
                        "UPDATE devices SET workspace_id = ?1 WHERE id = ?2",
                        rusqlite::params![snapshot.workspace.id, dev.id],
                    );
                }
            },
            Err(e) => return Err(PackageError::DatabaseError(e.to_string())),
        }
    }

    // 4. Reconciliar Hallazgos de Seguridad
    for f in &snapshot.security_findings {
        let sev_str = match f.severity {
            RiskSeverity::Critical => "Critical",
            RiskSeverity::High => "High",
            RiskSeverity::Medium => "Medium",
            RiskSeverity::Low => "Low",
        };
        let status_str = match f.status {
            FindingStatus::Open => "open",
            FindingStatus::Acknowledged => "acknowledged",
            FindingStatus::Ignored => "ignored",
            FindingStatus::Resolved => "resolved",
        };

        let rows = tx.execute(
            "INSERT INTO security_findings (id, device_id, network_id, rule_id, severity, title, description, evidence, remediation_advice, status, first_seen_at, last_seen_at, resolved_at, workspace_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                resolved_at = excluded.resolved_at,
                last_seen_at = MAX(security_findings.last_seen_at, excluded.last_seen_at)",
            rusqlite::params![
                f.id,
                f.device_id,
                f.network_id,
                f.rule_id,
                sev_str,
                f.title,
                f.description,
                f.evidence,
                f.remediation_advice,
                status_str,
                f.first_seen_at,
                f.last_seen_at,
                f.resolved_at,
                snapshot.workspace.id,
            ],
        ).map_err(|e| PackageError::DatabaseError(e.to_string()))?;

        if rows > 0 {
            summary.findings_imported += 1;
        }
    }

    // 5. Muestras de salud de red
    for h in &snapshot.network_health_samples {
        let rows = tx.execute(
            "INSERT OR IGNORE INTO network_health_samples (id, network_id, target_type, target_ip, rtt_ms, jitter_ms, loss_pct, sample_count, probes_failed, method, created_at, workspace_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                h.id,
                h.network_id,
                h.target_type,
                h.target_ip,
                h.rtt_ms.map(|v| v as i64),
                h.jitter_ms,
                h.loss_pct,
                h.sample_count,
                h.probes_failed,
                h.method,
                h.created_at,
                snapshot.workspace.id,
            ],
        ).map_err(|e| PackageError::DatabaseError(e.to_string()))?;

        if rows > 0 {
            summary.samples_imported += 1;
        }
    }

    // 6. Test de velocidad
    for s in &snapshot.speed_tests {
        let rows = tx.execute(
            "INSERT OR IGNORE INTO speed_tests (id, network_id, server_name, server_host, server_city, server_country, download_mbps, upload_mbps, latency_ms, jitter_ms, bufferbloat_ms, retransmits, protocol, created_at, workspace_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                s.id,
                s.network_id,
                s.server_name,
                s.server_host,
                s.server_city,
                s.server_country,
                s.download_mbps,
                s.upload_mbps,
                s.latency_ms as i64,
                s.jitter_ms,
                s.bufferbloat_ms.map(|v| v as i64),
                s.retransmits,
                s.protocol,
                s.created_at,
                snapshot.workspace.id,
            ],
        ).map_err(|e| PackageError::DatabaseError(e.to_string()))?;

        if rows > 0 {
            summary.speed_tests_imported += 1;
        }
    }

    tx.commit()
        .map_err(|e| PackageError::DatabaseError(e.to_string()))?;

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mylan_core::{Confidence, DeviceType, MacAddr};
    use mylan_db::migrations::run_migrations;
    use std::net::IpAddr;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_export_and_import_snapshot_roundtrip() {
        let conn_a = setup_test_db();

        // Creamos un workspace adicional
        let ws = Workspace {
            id: "ws-lab".to_string(),
            name: "Laboratorio".to_string(),
            description: Some("Red de pruebas".to_string()),
            icon: "flask".to_string(),
            color: "#10b981".to_string(),
            created_at: "2026-10-16T10:00:00Z".to_string(),
            updated_at: "2026-10-16T10:00:00Z".to_string(),
        };
        create_workspace(&conn_a, &ws).unwrap();

        // Creamos una red en ese workspace
        let net = Network {
            id: "net-lab".to_string(),
            name: "Subred Lab".to_string(),
            cidr: "10.0.0.0/24".to_string(),
            gateway_ip: Some("10.0.0.1".parse().unwrap()),
            dns_servers: vec!["1.1.1.1".parse().unwrap()],
            created_at: "2026-10-16T10:00:00Z".to_string(),
            updated_at: "2026-10-16T10:00:00Z".to_string(),
        };
        conn_a.execute(
            "INSERT INTO networks (id, name, cidr, gateway_ip, dns_servers, created_at, updated_at, workspace_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                net.id,
                net.name,
                net.cidr,
                net.gateway_ip.map(|i| i.to_string()),
                "1.1.1.1",
                net.created_at,
                net.updated_at,
                "ws-lab",
            ],
        ).unwrap();

        // Insertamos un dispositivo
        let ip_addr: IpAddr = "10.0.0.50".parse().unwrap();
        let mac_addr: MacAddr = MacAddr::parse("00:11:22:33:44:55").unwrap();
        let dev = Device {
            id: "dev-lab-1".to_string(),
            network_id: "net-lab".to_string(),
            primary_ip: Some(ip_addr),
            primary_mac: Some(mac_addr),
            hostname: Some("server.lab".to_string()),
            display_name: Some("Servidor LAB".to_string()),
            vendor: Some("Ubuntu".to_string()),
            manufacturer: None,
            model: None,
            device_type: DeviceType::Nas,
            os_family: Some("Linux".to_string()),
            confidence: Confidence::new(90),
            first_seen_at: "2026-10-16T10:00:00Z".to_string(),
            last_seen_at: "2026-10-16T12:00:00Z".to_string(),
            is_trusted: true,
            is_hidden: false,
            is_online: true,
            notes: Some("Servidor principal".to_string()),
        };
        upsert_device(&conn_a, &dev).unwrap();
        conn_a
            .execute(
                "UPDATE devices SET workspace_id = 'ws-lab' WHERE id = 'dev-lab-1'",
                [],
            )
            .unwrap();

        // Exportar snapshot
        let snapshot =
            export_workspace_snapshot(&conn_a, "ws-lab", "laptop-origin", "1.0.0").unwrap();
        assert_eq!(snapshot.metadata.workspace_id, "ws-lab");
        assert_eq!(snapshot.devices.len(), 1);
        assert_eq!(snapshot.devices[0].primary_ip, Some(ip_addr));

        // Serializar a JSON y restaurar
        let json_bytes = snapshot.to_json_bytes().unwrap();
        let restored_snapshot = WorkspaceSnapshot::from_json_bytes(&json_bytes).unwrap();
        assert_eq!(restored_snapshot.metadata.workspace_name, "Laboratorio");

        // Importar en una segunda BD vacia
        let mut conn_b = setup_test_db();
        let summary = import_workspace_snapshot(&mut conn_b, &restored_snapshot).unwrap();
        assert_eq!(summary.devices_imported, 1);
        assert_eq!(summary.networks_imported, 1);

        // Verificar que existe en conn_b
        let imported_ws = get_workspace(&conn_b, "ws-lab").unwrap().unwrap();
        assert_eq!(imported_ws.name, "Laboratorio");

        let imported_devs =
            mylan_db::device_repo::list_devices_by_workspace(&conn_b, "ws-lab").unwrap();
        assert_eq!(imported_devs.len(), 1);
        assert_eq!(
            imported_devs[0].display_name.as_deref(),
            Some("Servidor LAB")
        );
    }
}
