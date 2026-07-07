//! `mylan services` y `mylan export services` — reporte de servicios (Fase 3, Paso 6).
//!
//! `mylan services` lista el inventario de servicios/puertos con filtros AND,
//! orden por IP de dispositivo + puerto, y resaltado visual de puertos sensibles
//! (23, 21, 445, 22, 80, 8080, 8443, 554, 161). `mylan export services` delega
//! en [`crate::commands::export::export_services`] (CSV/JSON con `ServiceExportRow`).
//! Ambos leen la DB SQLite local; sin tráfico de red. El flag `sensitive` es solo
//! visual (no se persiste).

use std::collections::HashSet;
use std::io::IsTerminal;
use std::net::IpAddr;

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use mylan_core::{Protocol, ServiceState};
use mylan_db::service_repo::{list_services, ServiceExportRow, ServiceFilters};

use crate::commands::export::{export_services, ExportFormat};
use crate::commands::{latest_network_id, open_db};
use crate::ctx::AppContext;
use crate::util::print_redaction_note;

/// Puertos marcados como sensibles (admin/clear-text/management).
const SENSITIVE_PORTS: &[u16] = &[23, 21, 445, 22, 80, 8080, 8443, 554, 161];

/// `mylan services [--device ..] [--port ..] [--protocol ..] [--service ..]`.
///
/// Lista servicios del inventario con filtros AND (substring case-insensitive
/// para `--service`), orden por IP de dispositivo y puerto. Resalta puertos
/// sensibles de forma visual (`[!]` siempre, color si la salida es TTY).
pub fn run_services(
    ctx: &AppContext,
    device: Option<&str>,
    port: Option<u16>,
    protocol: Option<&str>,
    service: Option<&str>,
) -> anyhow::Result<()> {
    print_redaction_note();

    let conn = open_db(ctx)?;
    let net_id = match latest_network_id(&conn)? {
        Some(id) => id,
        None => {
            println!("No hay inventario todavía. Ejecuta `mylan scan` primero.");
            return Ok(());
        }
    };

    let device_filter = match device {
        Some(d) => match d.parse::<IpAddr>() {
            Ok(ip) => match mylan_db::device_repo::get_device_by_ip(&conn, &net_id, ip)? {
                Some(dev) => Some(dev.id),
                None => {
                    println!("No se encontró un dispositivo con IP {ip} en la red {net_id}.");
                    return Ok(());
                }
            },
            Err(_) => Some(d.to_string()),
        },
        None => None,
    };

    let filters = ServiceFilters {
        device_id: device_filter,
        port,
        protocol: protocol.map(|p| p.to_ascii_lowercase()),
        service: service.map(str::to_string),
    };
    let mut rows = list_services(&conn, &filters)?;

    // Acota a la red activa (consistencia con `mylan devices`/`mylan device`).
    let net_ids: HashSet<String> = mylan_db::device_repo::list_devices(&conn, &net_id)?
        .iter()
        .map(|d| d.id.clone())
        .collect();
    rows.retain(|r| net_ids.contains(&r.device_id));

    if rows.is_empty() {
        println!("No hay servicios que coincidan con los filtros en la red {net_id}.");
        return Ok(());
    }
    if rows.len() > 1000 {
        eprintln!(
            "[mylan] Advertencia: {} servicios encontrados. \
             Considera acotar con --device/--port/--protocol/--service.",
            rows.len()
        );
    }

    render_services_table(&rows);
    println!("\nTotal: {} servicios", rows.len());
    Ok(())
}

/// `mylan export services --format json|csv [--output ..]`.
///
/// CSV con columnas exactas `device_id,device_ip,display_name,protocol,port,
/// service_name,product,version,banner,state,first_seen_at,last_seen_at`
/// (writer manual). JSON usa el mismo `ServiceExportRow`.
pub fn run_export_services(
    ctx: &AppContext,
    format: ExportFormat,
    output: Option<&str>,
) -> anyhow::Result<()> {
    print_redaction_note();
    let conn = open_db(ctx)?;
    export_services(&conn, format, output)
}

/// Renderiza la tabla de servicios con resaltado de puertos sensibles.
fn render_services_table(rows: &[ServiceExportRow]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("IP").fg(Color::Cyan),
            Cell::new("Proto").fg(Color::Cyan),
            Cell::new("Puerto").fg(Color::Cyan),
            Cell::new("Servicio").fg(Color::Cyan),
            Cell::new("Producto").fg(Color::Cyan),
            Cell::new("Versión").fg(Color::Cyan),
            Cell::new("Estado").fg(Color::Cyan),
            Cell::new("Sens").fg(Color::Cyan),
        ]);

    let tty = std::io::stdout().is_terminal();
    for r in rows {
        let sensitive = is_sensitive(r.port);
        let port_cell = Cell::new(r.port);
        let port_cell = if sensitive && tty {
            port_cell.fg(Color::Yellow)
        } else {
            port_cell
        };
        let sens_cell = Cell::new(if sensitive { "[!]" } else { "" });
        let sens_cell = if sensitive && tty {
            sens_cell.fg(Color::Red)
        } else {
            sens_cell
        };
        table.add_row(vec![
            Cell::new(opt_ip(r.device_ip)),
            Cell::new(protocol_label(r.protocol)),
            port_cell,
            Cell::new(r.service_name.clone().unwrap_or_default()),
            Cell::new(r.product.clone().unwrap_or_default()),
            Cell::new(r.version.clone().unwrap_or_default()),
            Cell::new(state_label(r.state)),
            sens_cell,
        ]);
    }
    println!("{table}");
}

fn is_sensitive(port: u16) -> bool {
    SENSITIVE_PORTS.contains(&port)
}

fn opt_ip(ip: Option<IpAddr>) -> String {
    ip.map(|i| i.to_string()).unwrap_or_else(|| "-".into())
}

fn protocol_label(p: Protocol) -> String {
    format!("{p:?}").to_lowercase()
}

fn state_label(s: ServiceState) -> String {
    format!("{s:?}").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_ports_flagged() {
        for port in [23u16, 21, 445, 22, 80, 8080, 8443, 554, 161] {
            assert!(
                is_sensitive(port),
                "port {port} debe marcarse como sensible"
            );
        }
    }

    #[test]
    fn non_sensitive_ports_not_flagged() {
        for port in [443u16, 8081, 123, 53, 9000, 0] {
            assert!(
                !is_sensitive(port),
                "port {port} no debe marcarse como sensible"
            );
        }
    }

    #[test]
    fn protocol_and_state_labels_are_snake_case() {
        assert_eq!(protocol_label(Protocol::Tcp), "tcp");
        assert_eq!(protocol_label(Protocol::Udp), "udp");
        assert_eq!(state_label(ServiceState::Open), "open");
        assert_eq!(state_label(ServiceState::Filtered), "filtered");
        assert_eq!(state_label(ServiceState::Closed), "closed");
    }
}
