//! `mylan status` — estado general: interfaz activa + conteo de inventario.

use crate::commands::{latest_network_id, open_db};
use crate::ctx::AppContext;
use crate::util::print_redaction_note;

/// Muestra la interfaz LAN activa (auto-detectada) y el conteo de dispositivos.
pub fn run(ctx: &AppContext) -> anyhow::Result<()> {
    print_redaction_note();

    let iface = mylan_discovery::detect_interface(None)?;
    println!("Interfaz activa : {}", iface.name);
    println!("IP local        : {}", iface.ip);
    println!("CIDR           : {}", iface.cidr());
    if let Some(gw) = iface.gateway_ip {
        println!("Gateway        : {gw}");
    } else {
        println!("Gateway        : (no detectado)");
    }
    if !iface.dns_servers.is_empty() {
        let dns: Vec<String> = iface
            .dns_servers
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        println!("DNS            : {}", dns.join(", "));
    }

    match open_db(ctx) {
        Ok(conn) => {
            let count = match latest_network_id(&conn)? {
                Some(net_id) => mylan_db::device_repo::list_devices(&conn, &net_id)?.len(),
                None => 0,
            };
            println!("Dispositivos    : {count}");
        }
        Err(e) => {
            println!("Dispositivos    : (DB no disponible: {e})");
        }
    }
    Ok(())
}
