//! `mylan devices` — tabla del inventario con comfy-table.

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use crate::commands::{latest_network_id, open_db};
use crate::ctx::AppContext;
use crate::util::print_redaction_note;

/// Lista los dispositivos del inventario en una tabla.
pub fn run(ctx: &AppContext) -> anyhow::Result<()> {
    print_redaction_note();

    let conn = open_db(ctx)?;
    let net_id = match latest_network_id(&conn)? {
        Some(id) => id,
        None => {
            println!("No hay inventario todavía. Ejecuta `mylan scan` primero.");
            return Ok(());
        }
    };
    let devices = mylan_db::device_repo::list_devices(&conn, &net_id)?;
    if devices.is_empty() {
        println!("No hay dispositivos registrados para la red {net_id}.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("IP").fg(Color::Cyan),
            Cell::new("MAC").fg(Color::Cyan),
            Cell::new("Hostname").fg(Color::Cyan),
            Cell::new("Vendor").fg(Color::Cyan),
            Cell::new("Tipo").fg(Color::Cyan),
            Cell::new("Conf.").fg(Color::Cyan),
            Cell::new("Last seen").fg(Color::Cyan),
        ]);

    for d in &devices {
        table.add_row(vec![
            Cell::new(opt_ip(d.primary_ip)),
            Cell::new(opt_mac(d.primary_mac)),
            Cell::new(d.hostname.clone().unwrap_or_default()),
            Cell::new(d.vendor.clone().unwrap_or_default()),
            Cell::new(device_type_label(&d.device_type)),
            Cell::new(d.confidence.score()),
            Cell::new(&d.last_seen_at),
        ]);
    }
    println!("{table}");
    println!("\nTotal: {} dispositivos", devices.len());
    Ok(())
}

fn opt_ip(ip: Option<std::net::IpAddr>) -> String {
    ip.map(|i| i.to_string()).unwrap_or_else(|| "-".into())
}

fn opt_mac(mac: Option<mylan_core::MacAddr>) -> String {
    mac.map(|m| m.to_string()).unwrap_or_else(|| "-".into())
}

fn device_type_label(t: &mylan_core::DeviceType) -> String {
    format!("{t:?}").to_lowercase()
}
