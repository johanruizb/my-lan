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
