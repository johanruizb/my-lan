//! `mylan device <ip>` — detalle de un dispositivo por su IP.

use std::net::IpAddr;

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use crate::commands::{latest_network_id, open_db};
use crate::ctx::AppContext;
use crate::util::print_redaction_note;

/// Muestra el detalle de un dispositivo identificado por su IP primaria.
pub fn run(ctx: &AppContext, ip_str: &str) -> anyhow::Result<()> {
    print_redaction_note();

    let ip: IpAddr = ip_str.parse()?;
    let conn = open_db(ctx)?;
    let net_id = match latest_network_id(&conn)? {
        Some(id) => id,
        None => anyhow::bail!("No hay inventario todavía. Ejecuta `mylan scan` primero."),
    };
    let device = mylan_db::device_repo::get_device_by_ip(&conn, &net_id, ip)?.ok_or_else(|| {
        anyhow::anyhow!("No se encontró un dispositivo con IP {ip} en la red {net_id}")
    })?;

    println!("Detalle del dispositivo {ip}");
    println!("  ID            : {}", device.id);
    println!("  Red           : {}", device.network_id);
    println!("  MAC primaria  : {}", opt_mac(device.primary_mac));
    println!("  IP primaria   : {}", opt_ip(device.primary_ip));
    println!(
        "  Hostname      : {}",
        device.hostname.as_deref().unwrap_or("-")
    );
    println!(
        "  Display name  : {}",
        device.display_name.as_deref().unwrap_or("-")
    );
    println!(
        "  Vendor        : {}",
        device.vendor.as_deref().unwrap_or("-")
    );
    println!(
        "  Manufacturer  : {}",
        device.manufacturer.as_deref().unwrap_or("-")
    );
    println!(
        "  Modelo        : {}",
        device.model.as_deref().unwrap_or("-")
    );
    println!("  Tipo          : {}", type_label(&device.device_type));
    println!(
        "  OS family     : {}",
        device.os_family.as_deref().unwrap_or("-")
    );
    println!("  Confianza     : {}/100", device.confidence.score());
    println!("  De confianza  : {}", device.is_trusted);
    println!("  Oculto        : {}", device.is_hidden);
    println!(
        "  Notas         : {}",
        device.notes.as_deref().unwrap_or("-")
    );
    println!("  Primera vez   : {}", device.first_seen_at);
    println!("  Última vez    : {}", device.last_seen_at);

    let services = mylan_db::service_repo::list_services_by_device(&conn, &device.id)?;
    if services.is_empty() {
        println!("\nServicios: (ninguno escaneado). Usa `mylan ports {ip}`.");
    } else {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Proto").fg(Color::Cyan),
                Cell::new("Puerto").fg(Color::Cyan),
                Cell::new("Servicio").fg(Color::Cyan),
                Cell::new("Producto").fg(Color::Cyan),
                Cell::new("Estado").fg(Color::Cyan),
                Cell::new("Banner").fg(Color::Cyan),
            ]);
        for s in &services {
            table.add_row(vec![
                Cell::new(format!("{:?}", s.protocol).to_lowercase()),
                Cell::new(s.port),
                Cell::new(s.service_name.clone().unwrap_or_default()),
                Cell::new(s.product.clone().unwrap_or_default()),
                Cell::new(format!("{:?}", s.state).to_lowercase()),
                Cell::new(s.banner.clone().unwrap_or_default()),
            ]);
        }
        println!("\nServicios:");
        println!("{table}");
    }
    Ok(())
}

fn opt_ip(ip: Option<IpAddr>) -> String {
    ip.map(|i| i.to_string()).unwrap_or_else(|| "-".into())
}

fn opt_mac(mac: Option<mylan_core::MacAddr>) -> String {
    mac.map(|m| m.to_string()).unwrap_or_else(|| "-".into())
}

fn type_label(t: &mylan_core::DeviceType) -> String {
    format!("{t:?}").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctx::AppContext;
    use mylan_core::DeviceType;

    fn ctx_in(tmp: &std::path::Path) -> AppContext {
        AppContext {
            db_path: tmp.join("mylan.db"),
            signatures_dir: tmp.to_path_buf(),
            verbose: false,
        }
    }

    #[test]
    fn opt_ip_formats_known_address() {
        let ip: IpAddr = "192.168.1.42".parse().unwrap();
        assert_eq!(opt_ip(Some(ip)), "192.168.1.42");
    }

    #[test]
    fn opt_ip_dash_for_missing() {
        assert_eq!(opt_ip(None), "-");
    }

    #[test]
    fn opt_mac_formats_known_address() {
        let mac = mylan_core::MacAddr::parse("aa:bb:cc:dd:ee:ff").unwrap();
        assert_eq!(opt_mac(Some(mac)), "aa:bb:cc:dd:ee:ff");
    }

    #[test]
    fn opt_mac_dash_for_missing() {
        assert_eq!(opt_mac(None), "-");
    }

    #[test]
    fn type_label_is_lowercase_snake() {
        assert_eq!(type_label(&DeviceType::Router), "router");
        assert_eq!(type_label(&DeviceType::Phone), "phone");
        assert_eq!(type_label(&DeviceType::Iot), "iot");
        assert_eq!(type_label(&DeviceType::Unknown), "unknown");
    }

    #[test]
    fn run_rejects_invalid_ip() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());
        let result = run(&ctx, "not-an-ip");
        assert!(result.is_err(), "IP inválida debe errar");
    }

    #[test]
    fn run_errors_when_no_inventory() {
        // DB vacía (sin scans) → no hay red activa → bail "No hay inventario".
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());
        let result = run(&ctx, "192.168.1.1");
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("inventario") || msg.contains("scan"),
            "mensaje debe indicar falta de inventario: {msg}"
        );
    }
}
