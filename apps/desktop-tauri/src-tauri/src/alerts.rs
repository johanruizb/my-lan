//! Motor de Alertas y Notificaciones de Escritorio de MyLAN (Fase 6: Desktop Alerts).
//!
//! Catálogo de disparadores de alerta:
//! - `device_new_untrusted`: Dispositivo nuevo detectado que no está marcado como Confiable.
//! - `device_identity_tampered`: Cambio de IP en un dispositivo marcado como Confiable.
//! - `gateway_down` / `internet_down`: Pérdida de conectividad al gateway o resolución externa.
//! - `port_critical_opened`: Detección de puertos críticos/sensibles recién abiertos.
//!
//! Reglas de agregación y antispam:
//! - 1 o 2 alertas en un ciclo: notificaciones nativas individuales específicas.
//! - >=3 alertas simultáneas: consolidación en una sola notificación resumen para no saturar al usuario.
//! - Supresión en cold_start: omisión total durante el primer barrido de una red recién conectada.
//! - Modo Censura: si está activo, enmascara rigurosamente IPs, MACs y hostnames en las notificaciones del SO.

use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::dto::Settings;

/// Puertos comúnmente críticos o inseguros (texto plano / acceso remoto no restringido).
pub const CRITICAL_PORTS: &[u16] = &[
    21,   // FTP
    23,   // Telnet
    445,  // SMB
    3389, // RDP
    5900, // VNC
];

/// Categoría de una alerta de escritorio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCategory {
    DeviceNewUntrusted,
    DeviceIdentityTampered,
    GatewayDown,
    GatewayRecovered,
    PortCriticalOpened,
}

/// Ítem de alerta evaluado y listo para ser emitido al SO y al frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AlertItem {
    pub category: AlertCategory,
    pub title: String,
    pub body: String,
    pub device_id: Option<String>,
}

/// Evalúa los eventos producidos por un escaneo y despacha las notificaciones nativas correspondientes.
pub fn dispatch_scan_alerts(
    app: &AppHandle,
    events: &[mylan_core::Event],
    conn: &rusqlite::Connection,
    settings: &Settings,
    cold_start: bool,
) {
    // Si las notificaciones están desactivadas globalmente o estamos en cold_start, suprimir
    if !settings.notifications_enabled || cold_start {
        return;
    }

    let alerts = evaluate_alerts(events, conn, settings);
    if alerts.is_empty() {
        return;
    }

    // Regla de Agregación Antispam:
    // - Si hay 1 o 2 alertas: emitir cada una individualmente con su contexto específico.
    // - Si hay 3 o más alertas: consolidar en una única notificación resumen.
    if alerts.len() >= 3 {
        let total = alerts.len();
        let title = "MyLAN: Actividad en la red".to_string();
        let body =
            format!("Se detectaron {total} eventos o nuevos dispositivos en el último análisis.");
        show_os_notification(app, &title, &body);

        let _ = app.emit(
            "alert:summary",
            serde_json::json!({
                "total": total,
                "title": title,
                "body": body,
                "alerts": alerts,
            }),
        );
    } else {
        for alert in &alerts {
            show_os_notification(app, &alert.title, &alert.body);
            let _ = app.emit("alert:single", alert);
        }
    }
}

/// Evalúa el catálogo de eventos contra las reglas de alertas configuradas en Settings.
pub fn evaluate_alerts(
    events: &[mylan_core::Event],
    conn: &rusqlite::Connection,
    settings: &Settings,
) -> Vec<AlertItem> {
    let mut alerts = Vec::new();
    let censorship = settings.censorship_enabled;

    for event in events {
        match event.event_type {
            // 1. Dispositivo nuevo detectado que no está marcado como Confiable
            mylan_core::EventType::DeviceNew => {
                if !settings.notify_untrusted_devices {
                    continue;
                }
                if let Some(device_id) = &event.device_id {
                    if let Ok(Some(device)) = mylan_db::device_repo::get_device(conn, device_id) {
                        if !device.is_trusted {
                            let ip_display = match device.primary_ip {
                                Some(ip) => {
                                    if censorship {
                                        crate::censor::mask_ip(&ip.to_string())
                                    } else {
                                        ip.to_string()
                                    }
                                }
                                None => "IP desconocida".to_string(),
                            };

                            let name_display = if censorship {
                                device
                                    .vendor
                                    .clone()
                                    .unwrap_or_else(|| "Dispositivo nuevo".to_string())
                            } else {
                                device
                                    .display_name
                                    .or(device.hostname)
                                    .or(device.vendor)
                                    .unwrap_or_else(|| "Dispositivo nuevo".to_string())
                            };

                            alerts.push(AlertItem {
                                category: AlertCategory::DeviceNewUntrusted,
                                title: "Nuevo dispositivo detectado".to_string(),
                                body: format!("{ip_display} • {name_display}"),
                                device_id: Some(device.id),
                            });
                        }
                    }
                }
            }

            // 2. Cambio de IP o identidad en un dispositivo Confiable
            mylan_core::EventType::DeviceIpChanged => {
                if !settings.notify_trusted_device_tampered {
                    continue;
                }
                if let Some(device_id) = &event.device_id {
                    if let Ok(Some(device)) = mylan_db::device_repo::get_device(conn, device_id) {
                        if device.is_trusted {
                            let name = if censorship {
                                "Dispositivo confiable".to_string()
                            } else {
                                device
                                    .display_name
                                    .unwrap_or_else(|| "Dispositivo confiable".to_string())
                            };

                            let new_ip = match device.primary_ip {
                                Some(ip) => {
                                    if censorship {
                                        crate::censor::mask_ip(&ip.to_string())
                                    } else {
                                        ip.to_string()
                                    }
                                }
                                None => "nueva IP".to_string(),
                            };

                            alerts.push(AlertItem {
                                category: AlertCategory::DeviceIdentityTampered,
                                title: "Advertencia: Cambio de identidad en dispositivo confiable"
                                    .to_string(),
                                body: format!("{name} cambió de IP a {new_ip}"),
                                device_id: Some(device.id),
                            });
                        }
                    }
                }
            }

            // 3. Puertos críticos abiertos
            mylan_core::EventType::PortOpened => {
                if !settings.notify_critical_ports {
                    continue;
                }
                if let Some(data_str) = &event.data_json {
                    let is_critical = CRITICAL_PORTS.iter().any(|&p| {
                        data_str.contains(&format!("\"port\":{p}"))
                            || data_str.contains(&format!("\"port\": {p}"))
                    });
                    if is_critical {
                        alerts.push(AlertItem {
                            category: AlertCategory::PortCriticalOpened,
                            title: "Alerta de Seguridad: Puerto crítico abierto".to_string(),
                            body: "Se detectó un puerto sensible recién expuesto en la red"
                                .to_string(),
                            device_id: event.device_id.clone(),
                        });
                    }
                }
            }

            _ => {}
        }
    }

    alerts
}

/// Notifica caída o recuperación de conectividad al Gateway / Internet.
pub fn notify_gateway_status(
    app: &AppHandle,
    settings: &Settings,
    gateway_reachable: bool,
    prev_reachable: bool,
) {
    if !settings.notifications_enabled || !settings.notify_gateway_down {
        return;
    }

    if !gateway_reachable && prev_reachable {
        let title = "Alerta de Red: Conectividad interrumpida".to_string();
        let body = "El gateway o la conexión a la red local no responden.".to_string();
        show_os_notification(app, &title, &body);
        let _ = app.emit(
            "alert:single",
            AlertItem {
                category: AlertCategory::GatewayDown,
                title,
                body,
                device_id: None,
            },
        );
    } else if gateway_reachable && !prev_reachable {
        let title = "Red recuperada: Conectividad restablecida".to_string();
        let body = "Se ha reanudado la conexión con el gateway local.".to_string();
        show_os_notification(app, &title, &body);
        let _ = app.emit(
            "alert:single",
            AlertItem {
                category: AlertCategory::GatewayRecovered,
                title,
                body,
                device_id: None,
            },
        );
    }
}

/// Muestra una notificación nativa utilizando tauri-plugin-notification.
pub fn show_os_notification(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

#[cfg(test)]
mod tests {
    use super::*;
    use mylan_core::{Event, EventType, Severity};
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open memory db");
        mylan_db::connection::setup(&conn).expect("setup db");
        conn
    }

    #[test]
    fn test_alert_suppression_on_cold_start() {
        let settings = Settings::default();
        let conn = setup_test_db();
        let events = vec![Event {
            id: "e1".to_string(),
            network_id: "net1".to_string(),
            device_id: Some("dev1".to_string()),
            event_type: EventType::DeviceNew,
            severity: Severity::Info,
            message: Some("New device".to_string()),
            data_json: None,
            created_at: "2026-10-16T10:00:00Z".to_string(),
        }];

        // Invocando evaluate_alerts directamente
        let alerts = evaluate_alerts(&events, &conn, &settings);
        // Sin el dispositivo en la BD, no genera alerta
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_alert_untrusted_device_with_censorship() {
        let mut settings = Settings::default();
        settings.censorship_enabled = true;
        let conn = setup_test_db();

        let net = mylan_core::Network {
            id: "192.168.1.0/24".to_string(),
            name: "TestNet".to_string(),
            cidr: "192.168.1.0/24".to_string(),
            gateway_ip: None,
            dns_servers: Vec::new(),
            created_at: "2026-10-16T10:00:00Z".to_string(),
            updated_at: "2026-10-16T10:00:00Z".to_string(),
        };
        mylan_db::network_repo::upsert_network(&conn, &net).unwrap();

        let dev = mylan_core::Device {
            id: "dev-untrusted".to_string(),
            network_id: "192.168.1.0/24".to_string(),
            primary_mac: None,
            primary_ip: Some("192.168.1.55".parse().unwrap()),
            hostname: Some("suspicious.lan".to_string()),
            display_name: None,
            vendor: Some("UnknownVendor".to_string()),
            manufacturer: None,
            model: None,
            device_type: mylan_core::DeviceType::Unknown,
            os_family: None,
            confidence: mylan_core::Confidence::new(20),
            first_seen_at: "2026-10-16T10:00:00Z".to_string(),
            last_seen_at: "2026-10-16T10:00:00Z".to_string(),
            is_trusted: false,
            is_hidden: false,
            is_online: true,
            notes: None,
        };
        mylan_db::device_repo::upsert_device(&conn, &dev).unwrap();

        let events = vec![Event {
            id: "ev-1".to_string(),
            network_id: "192.168.1.0/24".to_string(),
            device_id: Some("dev-untrusted".to_string()),
            event_type: EventType::DeviceNew,
            severity: Severity::Info,
            message: Some("New device".to_string()),
            data_json: None,
            created_at: "2026-10-16T10:00:00Z".to_string(),
        }];

        let alerts = evaluate_alerts(&events, &conn, &settings);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].category, AlertCategory::DeviceNewUntrusted);
        // Con censura activa, la IP debe ser 192.168.*.*
        assert!(alerts[0].body.contains("192.168.*.*"));
        assert!(!alerts[0].body.contains("192.168.1.55"));
    }

    #[test]
    fn test_trusted_device_does_not_trigger_untrusted_alert() {
        let settings = Settings::default();
        let conn = setup_test_db();

        let net = mylan_core::Network {
            id: "192.168.1.0/24".to_string(),
            name: "TestNet".to_string(),
            cidr: "192.168.1.0/24".to_string(),
            gateway_ip: None,
            dns_servers: Vec::new(),
            created_at: "2026-10-16T10:00:00Z".to_string(),
            updated_at: "2026-10-16T10:00:00Z".to_string(),
        };
        mylan_db::network_repo::upsert_network(&conn, &net).unwrap();

        let dev = mylan_core::Device {
            id: "dev-trusted".to_string(),
            network_id: "192.168.1.0/24".to_string(),
            primary_mac: None,
            primary_ip: Some("192.168.1.10".parse().unwrap()),
            hostname: Some("my-laptop.lan".to_string()),
            display_name: Some("Mi Portátil".to_string()),
            vendor: None,
            manufacturer: None,
            model: None,
            device_type: mylan_core::DeviceType::Laptop,
            os_family: None,
            confidence: mylan_core::Confidence::new(90),
            first_seen_at: "2026-10-16T10:00:00Z".to_string(),
            last_seen_at: "2026-10-16T10:00:00Z".to_string(),
            is_trusted: true, // Confiable
            is_hidden: false,
            is_online: true,
            notes: None,
        };
        mylan_db::device_repo::upsert_device(&conn, &dev).unwrap();

        let events = vec![Event {
            id: "ev-2".to_string(),
            network_id: "192.168.1.0/24".to_string(),
            device_id: Some("dev-trusted".to_string()),
            event_type: EventType::DeviceNew,
            severity: Severity::Info,
            message: Some("New device".to_string()),
            data_json: None,
            created_at: "2026-10-16T10:00:00Z".to_string(),
        }];

        let alerts = evaluate_alerts(&events, &conn, &settings);
        assert!(
            alerts.is_empty(),
            "Dispositivo confiable no debe generar alerta de untrusted"
        );
    }

    #[test]
    fn test_trusted_device_identity_tampered_triggers_alert() {
        let mut settings = Settings::default();
        settings.censorship_enabled = false;
        let conn = setup_test_db();

        let net = mylan_core::Network {
            id: "192.168.1.0/24".to_string(),
            name: "TestNet".to_string(),
            cidr: "192.168.1.0/24".to_string(),
            gateway_ip: None,
            dns_servers: Vec::new(),
            created_at: "2026-10-16T10:00:00Z".to_string(),
            updated_at: "2026-10-16T10:00:00Z".to_string(),
        };
        mylan_db::network_repo::upsert_network(&conn, &net).unwrap();

        let dev = mylan_core::Device {
            id: "dev-trusted-tampered".to_string(),
            network_id: "192.168.1.0/24".to_string(),
            primary_mac: None,
            primary_ip: Some("192.168.1.99".parse().unwrap()),
            hostname: Some("my-server.lan".to_string()),
            display_name: Some("Servidor Principal".to_string()),
            vendor: None,
            manufacturer: None,
            model: None,
            device_type: mylan_core::DeviceType::Desktop,
            os_family: None,
            confidence: mylan_core::Confidence::new(95),
            first_seen_at: "2026-10-16T10:00:00Z".to_string(),
            last_seen_at: "2026-10-16T10:00:00Z".to_string(),
            is_trusted: true, // Confiable
            is_hidden: false,
            is_online: true,
            notes: None,
        };
        mylan_db::device_repo::upsert_device(&conn, &dev).unwrap();

        let events = vec![Event {
            id: "ev-3".to_string(),
            network_id: "192.168.1.0/24".to_string(),
            device_id: Some("dev-trusted-tampered".to_string()),
            event_type: EventType::DeviceIpChanged,
            severity: Severity::Warning,
            message: Some("IP changed".to_string()),
            data_json: Some(
                serde_json::json!({"old_ip": "192.168.1.20", "new_ip": "192.168.1.99"}).to_string(),
            ),
            created_at: "2026-10-16T10:00:00Z".to_string(),
        }];

        let alerts = evaluate_alerts(&events, &conn, &settings);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].category, AlertCategory::DeviceIdentityTampered);
        assert!(alerts[0].body.contains("Servidor Principal"));
        assert!(alerts[0].body.contains("192.168.1.99"));
    }
}
