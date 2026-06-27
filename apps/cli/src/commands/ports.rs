//! `mylan ports <ip> --top N` — escaneo de puertos bajo demanda + persistencia.
//!
//! Opera sobre un host vivo (P1): el port scan es explícito, no parte del
//! `scan` de descubrimiento, para preservar el presupuesto AC-12.

use std::net::IpAddr;

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use mylan_core::Service;
use mylan_db::service_repo::insert_service;

use crate::commands::{latest_network_id, open_db};
use crate::ctx::AppContext;
use crate::util::{new_id, now_rfc3339, print_redaction_note};

/// Escanea los `top` puertos de `ip`, persiste los servicios y los muestra.
pub async fn run(ctx: &AppContext, ip_str: &str, top: u16) -> anyhow::Result<()> {
    print_redaction_note();

    let ip: IpAddr = ip_str.parse()?;
    let conn = open_db(ctx)?;
    let net_id = match latest_network_id(&conn)? {
        Some(id) => id,
        None => anyhow::bail!("No hay inventario. Ejecuta `mylan scan` antes de escanear puertos."),
    };
    let device = mylan_db::device_repo::get_device_by_ip(&conn, &net_id, ip)?
        .ok_or_else(|| anyhow::anyhow!("No se encontró un dispositivo con IP {ip} en la red {net_id}. Ejecuta `mylan scan` primero."))?;

    tracing::info!(ip = %ip, top, "iniciando escaneo de puertos");
    let services = mylan_scanner::scan_ports(ip, top).await;
    tracing::info!(open = services.len(), "escaneo de puertos completado");

    if services.is_empty() {
        println!("No se detectaron puertos abiertos en {ip} (top {top}).");
        return Ok(());
    }

    let now = now_rfc3339()?;
    for svc in &services {
        insert_service(&conn, &fill_service(svc, &device.id, &now))?;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Proto").fg(Color::Cyan),
            Cell::new("Puerto").fg(Color::Cyan),
            Cell::new("Servicio").fg(Color::Cyan),
            Cell::new("Estado").fg(Color::Cyan),
            Cell::new("Banner").fg(Color::Cyan),
        ]);
    for s in &services {
        table.add_row(vec![
            Cell::new(format!("{:?}", s.protocol).to_lowercase()),
            Cell::new(s.port),
            Cell::new(s.service_name.clone().unwrap_or_default()),
            Cell::new(format!("{:?}", s.state).to_lowercase()),
            Cell::new(s.banner.clone().unwrap_or_default()),
        ]);
    }
    println!("Puertos abiertos en {ip}:");
    println!("{table}");
    println!(
        "\n{} servicios persistidos para el dispositivo {}.",
        services.len(),
        device.id
    );
    Ok(())
}

/// Rellena los campos de persistencia (id/device_id/timestamps) de un `Service`.
fn fill_service(svc: &Service, device_id: &str, now: &str) -> Service {
    Service {
        id: new_id(),
        device_id: device_id.to_string(),
        protocol: svc.protocol,
        port: svc.port,
        service_name: svc.service_name.clone(),
        product: svc.product.clone(),
        version: svc.version.clone(),
        banner: svc.banner.clone(),
        state: svc.state,
        first_seen_at: now.to_string(),
        last_seen_at: now.to_string(),
    }
}
