//! `mylan scan` — pipeline de dos fases: liveness → enrichment → persist.
//!
//! NO escanea puertos de toda la /24 (presupuesto AC-12); el port scan es
//! bajo demanda vía `mylan ports <ip>`.

use std::net::IpAddr;
use std::path::Path;

use mylan_core::{Enricher, Network, ScanProfile};
use mylan_db::connection::connect;
use mylan_discovery::{detect_interface, discover, DiscoverOptions};

use crate::ctx::AppContext;
use crate::pipeline::run_scan_pipeline;
use crate::util::{now_rfc3339, print_redaction_note};

/// Ejecuta un escaneo de descubrimiento sobre la red de la interfaz activa.
pub async fn run(
    ctx: &AppContext,
    profile: ScanProfile,
    interface: Option<&str>,
) -> anyhow::Result<()> {
    print_redaction_note();

    let iface = detect_interface(interface)?;
    tracing::info!(interface = %iface.name, ip = %iface.ip, cidr = %iface.cidr(), "interfaz detectada");

    let opts = DiscoverOptions {
        profile,
        interface: interface.map(std::string::ToString::to_string),
        ..DiscoverOptions::for_profile(profile)
    };
    let now = now_rfc3339()?;
    let network = Network {
        id: iface.cidr(),
        name: iface.cidr(),
        cidr: iface.cidr(),
        gateway_ip: iface.gateway_ip,
        dns_servers: iface.dns_servers.clone(),
        created_at: now.clone(),
        updated_at: now,
    };

    tracing::info!("fase liveness: descubriendo hosts...");
    let observations = discover(&iface, &opts).await;
    tracing::info!(
        observations = observations.len(),
        "fase liveness completada"
    );

    let conn = connect(&ctx.db_path)?;
    let enricher = build_enricher(&ctx.signatures_dir, iface.gateway_ip);

    let outcome = run_scan_pipeline(&conn, &network, &observations, &enricher, profile)?;

    println!("Escaneo completado.");
    println!("  Hosts vivos : {}", outcome.hosts_alive);
    println!("  Nuevos      : {}", outcome.hosts_new);
    println!("  Duración    : {} ms", outcome.duration_ms);
    println!("  Red         : {}", outcome.network_id);
    Ok(())
}

/// Construye el `Enricher` de fingerprint, degradando a no-op si falla la carga.
fn build_enricher(signatures_dir: &Path, gateway_ip: Option<IpAddr>) -> Enricher {
    match mylan_fingerprint::Fingerprint::load(signatures_dir, gateway_ip) {
        Ok(fp) => fp.enricher(),
        Err(e) => {
            tracing::warn!(error = %e, "fingerprint no cargado; enrichment no-op");
            mylan_core::noop_enricher()
        }
    }
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

    // Nota de determinismo: `run` ejecuta el pipeline de descubrimiento real
    // (liveness ARP/ICMP/mDNS/SSDP) que requiere red. No se testea el happy
    // path aquí. Sí se testea `build_enricher` (degradación a noop sin
    // signatures) y el error determinista de `run` con interfaz inexistente.

    #[test]
    fn build_enricher_degrades_to_noop_on_missing_signatures() {
        // Un directorio de signatures inexistente → Fingerprint::load falla →
        // noop_enricher. El Enricher resultante no debe entrar en pánico al
        // aplicarse a un Device con observaciones vacías.
        let enricher = build_enricher(std::path::Path::new("/nonexistent-signatures"), None);
        let mut device = mylan_core::Device {
            id: "dev-1".to_string(),
            network_id: "net-1".to_string(),
            primary_mac: None,
            primary_ip: None,
            hostname: None,
            display_name: None,
            vendor: None,
            manufacturer: None,
            model: None,
            device_type: mylan_core::DeviceType::Unknown,
            os_family: None,
            confidence: mylan_core::Confidence::default(),
            is_trusted: false,
            is_hidden: false,
            is_online: false,
            notes: None,
            first_seen_at: "2024-01-01T00:00:00Z".to_string(),
            last_seen_at: "2024-01-01T00:00:00Z".to_string(),
        };
        // Llamar al enricher no debe entrar en pánico; con noop el device queda
        // inalterado (no hay reglas que clasifiquen).
        enricher(&mut device, &[]);
        assert_eq!(device.id, "dev-1");
        assert_eq!(device.device_type, mylan_core::DeviceType::Unknown);
    }

    #[test]
    fn build_enricher_with_empty_signatures_dir_degrades() {
        // Un dir vacío (sin oui.csv ni rules YAML) → load falla → noop.
        let tmp = tempfile::tempdir().expect("tmp");
        let enricher = build_enricher(tmp.path(), None);
        let mut device = mylan_core::Device {
            id: "dev-2".to_string(),
            network_id: "net-1".to_string(),
            primary_mac: None,
            primary_ip: None,
            hostname: None,
            display_name: None,
            vendor: None,
            manufacturer: None,
            model: None,
            device_type: mylan_core::DeviceType::Unknown,
            os_family: None,
            confidence: mylan_core::Confidence::default(),
            is_trusted: false,
            is_hidden: false,
            is_online: false,
            notes: None,
            first_seen_at: "2024-01-01T00:00:00Z".to_string(),
            last_seen_at: "2024-01-01T00:00:00Z".to_string(),
        };
        enricher(&mut device, &[]);
        assert_eq!(device.device_type, mylan_core::DeviceType::Unknown);
    }

    #[tokio::test]
    async fn run_errors_on_nonexistent_interface() {
        // Una interfaz que no existe → detect_interface devuelve InterfaceNotFound.
        // Determinista: get_interfaces() lee info del sistema (sin I/O de red).
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());
        let result = run(&ctx, ScanProfile::Quick, Some("nonexistent_iface_xyz")).await;
        assert!(result.is_err(), "interfaz inexistente debe errar");
    }
}
