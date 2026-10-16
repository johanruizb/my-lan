//! Motor de Evaluación de Riesgos y Auditoría de Seguridad Pasiva (§4).
//!
//! Evalúa la postura de seguridad de la red y sus dispositivos aplicando un catálogo
//! de 8 reglas pasivas y deterministas sobre datos ya recolectados (sin tráfico invasivo).

use std::net::IpAddr;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use mylan_core::{
    Device, DeviceType, FindingStatus, Network, RiskSeverity, SecurityFinding, Service,
    ServiceState,
};

use crate::error::{map_sqlite, DbResult};
use crate::findings_repo::{
    auto_resolve_missing_device_findings, calculate_network_score, list_findings_by_network,
    upsert_finding,
};
use crate::util::{new_id, now_rfc3339};
use crate::{device_repo, network_repo, service_repo};

/// Reporte consolidado de auditoría de seguridad de red.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditReport {
    pub network_id: String,
    pub score: u32,
    pub total_findings: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub findings: Vec<SecurityFinding>,
}

pub const RULE_SEC_001_TELNET: &str = "SEC-001-TELNET-OPEN";
pub const RULE_SEC_002_FTP: &str = "SEC-002-FTP-PLAINTEXT";
pub const RULE_SEC_003_ROUTER_HTTP: &str = "SEC-003-ROUTER-HTTP-ADMIN";
pub const RULE_SEC_004_SMB: &str = "SEC-004-SMB-LEGACY";
pub const RULE_SEC_005_UPNP: &str = "SEC-005-UPNP-ACTIVE";
pub const RULE_SEC_006_IOT_CAM: &str = "SEC-006-IOT-UNTRUSTED-OPEN-CAM";
pub const RULE_SEC_007_SNMP: &str = "SEC-007-SNMP-PUBLIC";
pub const RULE_SEC_008_DNS: &str = "SEC-008-DNS-UNENCRYPTED";

/// Evalúa las reglas a nivel de red (ej. servidores DNS en texto plano sin cifrar).
pub fn evaluate_network_rules(network: &Network, now: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // SEC-008-DNS-UNENCRYPTED: Detecta si la red utiliza DNS públicos externos en texto plano (puerto 53).
    let mut unencrypted_external_dns = Vec::new();
    for ip in &network.dns_servers {
        let is_local = match ip {
            IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
            IpAddr::V6(v6) => v6.is_loopback() || (v6.segments()[0] & 0xffc0 == 0xfe80),
        };
        if !is_local {
            unencrypted_external_dns.push(ip.to_string());
        }
    }

    if !unencrypted_external_dns.is_empty() {
        findings.push(SecurityFinding {
            id: new_id(),
            device_id: None,
            network_id: network.id.clone(),
            rule_id: RULE_SEC_008_DNS.to_string(),
            severity: RiskSeverity::Low,
            title: "Servidores DNS externos sin cifrado (DNS en texto plano)".to_string(),
            description: "Las consultas de nombres de dominio se envían a servidores DNS externos en texto claro vía UDP 53, permitiendo a terceros o ISPs registrar los sitios visitados.".to_string(),
            evidence: Some(format!(
                "Servidores DNS públicos detectados: {}",
                unencrypted_external_dns.join(", ")
            )),
            remediation_advice: Some(
                "Configura un resolver DNS con cifrado DNS-over-HTTPS (DoH) o DNS-over-TLS (DoT) como Cloudflare (1.1.1.1), NextDNS, o AdGuard Home en tu red local.".to_string(),
            ),
            status: FindingStatus::Open,
            first_seen_at: now.to_string(),
            last_seen_at: now.to_string(),
            resolved_at: None,
        });
    }

    findings
}

/// Evalúa el catálogo de reglas pasivas sobre un dispositivo y sus servicios abiertos.
pub fn evaluate_device_rules(
    network: &Network,
    device: &Device,
    services: &[Service],
    now: &str,
) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    let open_services: Vec<&Service> = services
        .iter()
        .filter(|s| s.state == ServiceState::Open)
        .collect();

    let is_router = device.device_type == DeviceType::Router
        || match (&network.gateway_ip, &device.primary_ip) {
            (Some(gw), Some(ip)) => gw == ip,
            _ => false,
        };

    // SEC-001-TELNET-OPEN (Critical)
    for svc in &open_services {
        let is_telnet = svc.port == 23
            || svc
                .service_name
                .as_deref()
                .is_some_and(|n| n.eq_ignore_ascii_case("telnet"));

        if is_telnet {
            let banner_part = svc
                .banner
                .as_deref()
                .map(|b| format!(" — Banner: '{b}'"))
                .unwrap_or_default();
            findings.push(SecurityFinding {
                id: new_id(),
                device_id: Some(device.id.clone()),
                network_id: network.id.clone(),
                rule_id: RULE_SEC_001_TELNET.to_string(),
                severity: RiskSeverity::Critical,
                title: "Servicio Telnet sin cifrar expuesto (Puerto 23)".to_string(),
                description: "Telnet transmite credenciales y datos de sesión en texto plano, permitiendo que cualquiera en la red intercepte o capture contraseñas.".to_string(),
                evidence: Some(format!("Puerto TCP {} abierto (Telnet){banner_part}", svc.port)),
                remediation_advice: Some(
                    "Deshabilita el servicio Telnet en la configuración del dispositivo y utiliza SSH (puerto 22) para administración segura.".to_string(),
                ),
                status: FindingStatus::Open,
                first_seen_at: now.to_string(),
                last_seen_at: now.to_string(),
                resolved_at: None,
            });
            break;
        }
    }

    // SEC-002-FTP-PLAINTEXT (High)
    for svc in &open_services {
        let is_ftp = svc.port == 21
            || svc
                .service_name
                .as_deref()
                .is_some_and(|n| n.eq_ignore_ascii_case("ftp"));

        if is_ftp {
            let banner_part = svc
                .banner
                .as_deref()
                .map(|b| format!(" — Banner: '{b}'"))
                .unwrap_or_default();
            findings.push(SecurityFinding {
                id: new_id(),
                device_id: Some(device.id.clone()),
                network_id: network.id.clone(),
                rule_id: RULE_SEC_002_FTP.to_string(),
                severity: RiskSeverity::High,
                title: "Servicio FTP en texto plano (Puerto 21)".to_string(),
                description: "El protocolo FTP estándar transmite credenciales de acceso y archivos transferidos sin cifrar a través de la red local.".to_string(),
                evidence: Some(format!("Puerto TCP {} abierto (FTP){banner_part}", svc.port)),
                remediation_advice: Some(
                    "Reemplaza FTP por SFTP (vía SSH) o FTPS (FTP con TLS implícito/explícito), o inhabilita el acceso anónimo o no autenticado.".to_string(),
                ),
                status: FindingStatus::Open,
                first_seen_at: now.to_string(),
                last_seen_at: now.to_string(),
                resolved_at: None,
            });
            break;
        }
    }

    // SEC-003-ROUTER-HTTP-ADMIN (Critical)
    if is_router {
        let has_http_80 = open_services.iter().any(|s| s.port == 80);
        if has_http_80 {
            findings.push(SecurityFinding {
                id: new_id(),
                device_id: Some(device.id.clone()),
                network_id: network.id.clone(),
                rule_id: RULE_SEC_003_ROUTER_HTTP.to_string(),
                severity: RiskSeverity::Critical,
                title: "Panel de administración del router sin TLS (HTTP Puerto 80)".to_string(),
                description: "El router o gateway expone su panel de administración a través de HTTP sin cifrado, permitiendo el robo o secuestro de la clave de acceso de administrador.".to_string(),
                evidence: Some("Gateway / Router principal con puerto TCP 80 abierto sin forzar HTTPS".to_string()),
                remediation_advice: Some(
                    "Accede a los ajustes del router y activa la administración exclusiva por HTTPS (puerto 443) o inhabilita la gestión web en texto plano.".to_string(),
                ),
                status: FindingStatus::Open,
                first_seen_at: now.to_string(),
                last_seen_at: now.to_string(),
                resolved_at: None,
            });
        }
    }

    // SEC-004-SMB-LEGACY (High)
    if !device.is_trusted {
        let has_smb = open_services.iter().any(|s| s.port == 445 || s.port == 139);
        if has_smb {
            findings.push(SecurityFinding {
                id: new_id(),
                device_id: Some(device.id.clone()),
                network_id: network.id.clone(),
                rule_id: RULE_SEC_004_SMB.to_string(),
                severity: RiskSeverity::High,
                title: "Servicio de compartición SMB expuesto en host no verificado".to_string(),
                description: "Los puertos de compartición de archivos SMB/NetBIOS (139, 445) están abiertos en un host no marcado como confiable, siendo vectores comunes de propagación de malware o ransomware.".to_string(),
                evidence: Some("Puerto TCP 445/139 abierto en host no marcado como confiable".to_string()),
                remediation_advice: Some(
                    "Si este dispositivo no es un servidor de archivos o NAS autorizado, deshabilita la compartición de archivos o márcalo como confiable tras verificarlo.".to_string(),
                ),
                status: FindingStatus::Open,
                first_seen_at: now.to_string(),
                last_seen_at: now.to_string(),
                resolved_at: None,
            });
        }
    }

    // SEC-005-UPNP-ACTIVE (Medium)
    if is_router {
        let has_upnp_port = open_services.iter().any(|s| {
            s.port == 1900
                || s.port == 5000
                || s.port == 2869
                || s.port == 49152
                || s.service_name
                    .as_deref()
                    .is_some_and(|n| n.to_lowercase().contains("upnp"))
                || s.banner
                    .as_deref()
                    .is_some_and(|b| b.to_lowercase().contains("upnp"))
        });

        if has_upnp_port {
            findings.push(SecurityFinding {
                id: new_id(),
                device_id: Some(device.id.clone()),
                network_id: network.id.clone(),
                rule_id: RULE_SEC_005_UPNP.to_string(),
                severity: RiskSeverity::Medium,
                title: "Protocolo UPnP activo en el Gateway/Router".to_string(),
                description: "UPnP (Universal Plug and Play) permite a cualquier dispositivo o software en la red abrir puertos hacia Internet automáticamente sin requerir autorización.".to_string(),
                evidence: Some("Puerto UPnP/SSDP detectado abierto en el router".to_string()),
                remediation_advice: Some(
                    "Desactiva UPnP en los ajustes avanzados de tu router para impedir la apertura automática y no autorizada de puertos hacia Internet.".to_string(),
                ),
                status: FindingStatus::Open,
                first_seen_at: now.to_string(),
                last_seen_at: now.to_string(),
                resolved_at: None,
            });
        }
    }

    // SEC-006-IOT-UNTRUSTED-OPEN-CAM (High)
    let is_camera_or_iot = device.device_type == DeviceType::Camera
        || device.device_type == DeviceType::Iot
        || device.vendor.as_deref().is_some_and(|v| {
            let vl = v.to_lowercase();
            vl.contains("hikvision")
                || vl.contains("dahua")
                || vl.contains("foscam")
                || vl.contains("reolink")
                || vl.contains("amcrest")
                || vl.contains("axis")
        })
        || device.hostname.as_deref().is_some_and(|h| {
            let hl = h.to_lowercase();
            hl.contains("cam") || hl.contains("dvr") || hl.contains("nvr") || hl.contains("ipcam")
        });

    if is_camera_or_iot && !device.is_trusted {
        let has_cam_port = open_services
            .iter()
            .any(|s| s.port == 554 || s.port == 80 || s.port == 8080 || s.port == 8888);

        if has_cam_port {
            findings.push(SecurityFinding {
                id: new_id(),
                device_id: Some(device.id.clone()),
                network_id: network.id.clone(),
                rule_id: RULE_SEC_006_IOT_CAM.to_string(),
                severity: RiskSeverity::High,
                title: "Cámara IP o dispositivo IoT no verificado con puertos abiertos".to_string(),
                description: "Se detectó una cámara o dispositivo IoT no marcado como confiable con transmisión RTSP (puerto 554) o servidor web accesible sin verificar.".to_string(),
                evidence: Some("Puerto RTSP (554) o HTTP abierto en dispositivo IoT/Cámara no confiable".to_string()),
                remediation_advice: Some(
                    "Establece una contraseña robusta para la cámara, aísla el dispositivo en una VLAN o red de invitados separada, y márcalo como confiable si es legítimo.".to_string(),
                ),
                status: FindingStatus::Open,
                first_seen_at: now.to_string(),
                last_seen_at: now.to_string(),
                resolved_at: None,
            });
        }
    }

    // SEC-007-SNMP-PUBLIC (Medium)
    let has_snmp = open_services.iter().any(|s| {
        s.port == 161
            || s.service_name
                .as_deref()
                .is_some_and(|n| n.to_lowercase().contains("snmp"))
    });
    if has_snmp {
        findings.push(SecurityFinding {
            id: new_id(),
            device_id: Some(device.id.clone()),
            network_id: network.id.clone(),
            rule_id: RULE_SEC_007_SNMP.to_string(),
            severity: RiskSeverity::Medium,
            title: "Servicio SNMP expuesto con riesgo de divulgación de red".to_string(),
            description: "SNMP en puerto 161 puede permitir la lectura no autorizada del mapa completo de interfaces, tablas de enrutamiento y nombres de host.".to_string(),
            evidence: Some("Puerto SNMP 161 detectado abierto".to_string()),
            remediation_advice: Some(
                "Inhabilita SNMP si no es utilizado para monitorización central, o migra a SNMPv3 con autenticación y cifrado SHA/AES.".to_string(),
            ),
            status: FindingStatus::Open,
            first_seen_at: now.to_string(),
            last_seen_at: now.to_string(),
            resolved_at: None,
        });
    }

    findings
}

/// Evalúa las reglas para un dispositivo individual (ej. al finalizar su escaneo de puertos),
/// persiste los hallazgos y auto-resuelve los que hayan desaparecido.
pub fn evaluate_and_persist_device(
    conn: &Connection,
    network: &Network,
    device: &Device,
    services: &[Service],
    now: &str,
) -> DbResult<Vec<SecurityFinding>> {
    let findings = evaluate_device_rules(network, device, services, now);

    let active_rule_ids: Vec<&str> = findings.iter().map(|f| f.rule_id.as_str()).collect();

    // 1. Upsert hallazgos activos
    for f in &findings {
        upsert_finding(conn, f)?;
    }

    // 2. Auto-resolver hallazgos previos que ya no están activos
    auto_resolve_missing_device_findings(conn, &device.id, &active_rule_ids, now)?;

    Ok(findings)
}

/// Ejecuta una auditoría de seguridad completa sobre toda la red activa:
/// evalúa reglas de red + todas las reglas de dispositivos e inventario de servicios,
/// persiste los resultados y devuelve el reporte consolidado con Score.
pub fn run_full_security_audit(
    conn: &Connection,
    network_id: &str,
) -> DbResult<SecurityAuditReport> {
    let now = now_rfc3339().unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    let network = match network_repo::get_network(conn, network_id)? {
        Some(net) => net,
        None => return Err(map_sqlite(rusqlite::Error::QueryReturnedNoRows)),
    };

    let devices = device_repo::list_devices(conn, network_id)?;

    // 1. Evaluar reglas de red
    let net_findings = evaluate_network_rules(&network, &now);
    for f in &net_findings {
        upsert_finding(conn, f)?;
    }

    // 2. Evaluar cada dispositivo cargando sus servicios
    for dev in &devices {
        let svcs = service_repo::list_services_by_device(conn, &dev.id)?;
        evaluate_and_persist_device(conn, &network, dev, &svcs, &now)?;
    }

    // 3. Calcular métricas finales
    let score = calculate_network_score(conn, network_id)?;
    let all_findings = list_findings_by_network(conn, network_id)?;

    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;

    for f in &all_findings {
        if f.status == FindingStatus::Open {
            match f.severity {
                RiskSeverity::Critical => critical_count += 1,
                RiskSeverity::High => high_count += 1,
                RiskSeverity::Medium => medium_count += 1,
                RiskSeverity::Low => low_count += 1,
            }
        }
    }

    Ok(SecurityAuditReport {
        network_id: network_id.to_string(),
        score,
        total_findings: all_findings.len(),
        critical_count,
        high_count,
        medium_count,
        low_count,
        findings: all_findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::run_migrations;
    use mylan_core::{MacAddr, Protocol};

    fn setup_test_env() -> (Connection, Network, Device) {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();

        let network = Network {
            id: "net-test".to_string(),
            name: "HomeNet".to_string(),
            cidr: "192.168.1.0/24".to_string(),
            gateway_ip: Some("192.168.1.1".parse().unwrap()),
            dns_servers: vec!["8.8.8.8".parse().unwrap(), "192.168.1.1".parse().unwrap()],
            created_at: "2026-10-16T12:00:00Z".to_string(),
            updated_at: "2026-10-16T12:00:00Z".to_string(),
        };
        network_repo::upsert_network(&conn, &network).unwrap();

        let mut device = Device::new("dev-test", "net-test", "2026-10-16T12:00:00Z");
        device.primary_mac = Some(MacAddr::parse("aa:bb:cc:dd:ee:ff").unwrap());
        device.primary_ip = Some("192.168.1.10".parse().unwrap());
        device.hostname = Some("iot-camera".to_string());
        device.vendor = Some("Dahua".to_string());
        device.device_type = DeviceType::Camera;
        device.is_online = true;
        device.is_trusted = false;
        device_repo::upsert_device(&conn, &device).unwrap();

        (conn, network, device)
    }

    #[test]
    fn sec_001_and_002_trigger_on_telnet_ftp() {
        let (_conn, network, device) = setup_test_env();
        let services = vec![
            Service {
                id: "s1".to_string(),
                device_id: device.id.clone(),
                protocol: Protocol::Tcp,
                port: 23,
                service_name: Some("telnet".to_string()),
                product: None,
                version: None,
                banner: Some("Welcome to router telnet".to_string()),
                state: ServiceState::Open,
                first_seen_at: "t0".to_string(),
                last_seen_at: "t0".to_string(),
            },
            Service {
                id: "s2".to_string(),
                device_id: device.id.clone(),
                protocol: Protocol::Tcp,
                port: 21,
                service_name: Some("ftp".to_string()),
                product: None,
                version: None,
                banner: None,
                state: ServiceState::Open,
                first_seen_at: "t0".to_string(),
                last_seen_at: "t0".to_string(),
            },
        ];

        let findings = evaluate_device_rules(&network, &device, &services, "2026-10-16T12:00:00Z");
        assert!(findings.iter().any(|f| f.rule_id == RULE_SEC_001_TELNET));
        assert!(findings.iter().any(|f| f.rule_id == RULE_SEC_002_FTP));
    }

    #[test]
    fn sec_003_triggers_on_router_http_80() {
        let (_conn, network, mut device) = setup_test_env();
        device.primary_ip = Some("192.168.1.1".parse().unwrap());
        device.device_type = DeviceType::Router;

        let services = vec![Service {
            id: "s-http".to_string(),
            device_id: device.id.clone(),
            protocol: Protocol::Tcp,
            port: 80,
            service_name: Some("http".to_string()),
            product: None,
            version: None,
            banner: None,
            state: ServiceState::Open,
            first_seen_at: "t0".to_string(),
            last_seen_at: "t0".to_string(),
        }];

        let findings = evaluate_device_rules(&network, &device, &services, "2026-10-16T12:00:00Z");
        assert!(findings
            .iter()
            .any(|f| f.rule_id == RULE_SEC_003_ROUTER_HTTP));
    }

    #[test]
    fn sec_004_triggers_smb_only_on_untrusted_devices() {
        let (_conn, network, mut device) = setup_test_env();
        device.is_trusted = false;

        let services = vec![Service {
            id: "s-smb".to_string(),
            device_id: device.id.clone(),
            protocol: Protocol::Tcp,
            port: 445,
            service_name: Some("microsoft-ds".to_string()),
            product: None,
            version: None,
            banner: None,
            state: ServiceState::Open,
            first_seen_at: "t0".to_string(),
            last_seen_at: "t0".to_string(),
        }];

        let findings_untrusted =
            evaluate_device_rules(&network, &device, &services, "2026-10-16T12:00:00Z");
        assert!(findings_untrusted
            .iter()
            .any(|f| f.rule_id == RULE_SEC_004_SMB));

        device.is_trusted = true;
        let findings_trusted =
            evaluate_device_rules(&network, &device, &services, "2026-10-16T12:00:00Z");
        assert!(!findings_trusted
            .iter()
            .any(|f| f.rule_id == RULE_SEC_004_SMB));
    }

    #[test]
    fn sec_008_triggers_on_external_dns() {
        let (_conn, network, _device) = setup_test_env();
        let findings = evaluate_network_rules(&network, "2026-10-16T12:00:00Z");
        assert!(findings.iter().any(|f| f.rule_id == RULE_SEC_008_DNS));
    }

    #[test]
    fn full_audit_and_auto_resolution_flow() {
        let (conn, network, device) = setup_test_env();

        // 1. Insertar servicio Telnet en dev-test
        service_repo::insert_service(
            &conn,
            &Service {
                id: "svc-telnet".to_string(),
                device_id: device.id.clone(),
                protocol: Protocol::Tcp,
                port: 23,
                service_name: Some("telnet".to_string()),
                product: None,
                version: None,
                banner: None,
                state: ServiceState::Open,
                first_seen_at: "t0".to_string(),
                last_seen_at: "t0".to_string(),
            },
        )
        .unwrap();

        // 2. Correr auditoría completa
        let report = run_full_security_audit(&conn, &network.id).unwrap();
        assert!(report.score < 100);
        assert!(report.critical_count >= 1);
        assert!(report
            .findings
            .iter()
            .any(|f| f.rule_id == RULE_SEC_001_TELNET && f.status == FindingStatus::Open));

        // 3. El usuario cierra o reconfigura el puerto (pasa a Closed)
        service_repo::upsert_service(
            &conn,
            &Service {
                id: "svc-telnet".to_string(),
                device_id: device.id.clone(),
                protocol: Protocol::Tcp,
                port: 23,
                service_name: Some("telnet".to_string()),
                product: None,
                version: None,
                banner: None,
                state: ServiceState::Closed,
                first_seen_at: "t0".to_string(),
                last_seen_at: "t1".to_string(),
            },
        )
        .unwrap();

        // 4. Re-auditar: el hallazgo SEC-001 debe resolverse automáticamente
        let report_after = run_full_security_audit(&conn, &network.id).unwrap();
        let telnet_finding = report_after
            .findings
            .iter()
            .find(|f| f.rule_id == RULE_SEC_001_TELNET)
            .unwrap();
        assert_eq!(telnet_finding.status, FindingStatus::Resolved);
        assert!(telnet_finding.resolved_at.is_some());
        assert_eq!(report_after.critical_count, 0);
    }
}
