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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctx::AppContext;

    fn ctx_in(tmp: &std::path::Path) -> AppContext {
        AppContext {
            db_path: tmp.join("mylan.db"),
            signatures_dir: tmp.to_path_buf(),
            verbose: false,
        }
    }

    // Nota de determinismo: `run` llama a `detect_interface(None)` que enumera
    // las interfaces de red del host (vía `netdev`, sin I/O de red: solo lee
    // info del sistema). Requiere que el host tenga al menos una interfaz no
    // filtrada con IPv4 — condición cierta en CI y máquinas de desarrollo.
    // No envía paquetes; el camino de error (sin interfaz) no se fuerza aquí.

    #[test]
    fn run_returns_ok_with_tempfile_db_when_interface_present() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());
        // DB tempfile vacía: latest_network_id → None → count 0 (sin error).
        // detect_interface(None) requiere una interfaz real en el host.
        let result = run(&ctx);
        assert!(
            result.is_ok(),
            "status con DB tempfile debe ser Ok (requiere interfaz de red): {res:?}",
            res = result
        );
    }

    #[test]
    fn run_db_unavailable_does_not_propagate_error() {
        // Si la DB no es abrible, `run` NO propaga el error: lo imprime y devuelve Ok.
        // Usamos un path inválido ( directorio como si fuera fichero ) para forzar
        // el fallo de `open_db` sin tocar `detect_interface` (que va antes).
        // Construye un ctx cuyo db_path es un directorio existente (no un fichero):
        // rusqlite::open falla al abrir un directorio como DB.
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = AppContext {
            db_path: tmp.path().to_path_buf(), // directorio, no fichero
            signatures_dir: tmp.path().to_path_buf(),
            verbose: false,
        };
        let result = run(&ctx);
        // En hosts Unix/CI con interfaz de red, `detect_interface` Ok → el
        // error de DB se atrapa en el `match` → `run` devuelve Ok (contrato:
        // "no propaga el error de DB no disponible"). En plataformas sin
        // interfaz garantizada, solo verificamos que no entra en pánico.
        #[cfg(unix)]
        assert!(
            result.is_ok(),
            "run no debe propagar el error de DB no disponible: {res:?}",
            res = result
        );
        #[cfg(not(unix))]
        let _ = result;
    }
}
