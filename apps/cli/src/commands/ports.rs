//! `mylan ports <ip> --profile <name>` — escaneo de puertos bajo demanda +
//! persistencia (Fase 3, AC-2, AC-5, AC-7).
//!
//! Usa la API moderna [`mylan_scanner::scan_target`]: catálogo de puertos del
//! `ScanProfile`, cancelación cooperativa vía `CancellationToken` (Ctrl-C) y
//! progreso en vivo. Opera sobre un host vivo (P1): el port scan es explícito,
//! no parte del `scan` de descubrimiento, para preservar el presupuesto AC-12.
//!
//! Precedencia `--top` (AC-7): `scan_target` sondea el catálogo del perfil
//! (`ports_for_profile`). `--top` es informativo para quick/normal/deep (el
//! conteo del perfil tiene prioridad) e ignorado para iot/router (catálogo
//! fijo). Se emite un warning a stderr cuando `--top` difiere del conteo del
//! perfil o el perfil es iot/router.

use std::net::IpAddr;

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use mylan_core::{ScanProfile, Service};
use mylan_db::service_repo::upsert_service;
use mylan_scanner::{ports_for_profile, profile_options, scan_target, ScanError, ScanProgress};
use tokio_util::sync::CancellationToken;

use crate::commands::{latest_network_id, open_db};
use crate::ctx::AppContext;
use crate::util::{new_id, now_rfc3339, print_redaction_note};

/// Escanea los puertos de `ip` con el `profile` dado, persiste los servicios y
/// los muestra. `top` es un hint informativo (AC-7); el catálogo efectivo es el
/// del perfil. Ctrl-C cancela cooperativamente y devuelve los hits parciales.
pub async fn run(
    ctx: &AppContext,
    ip_str: &str,
    top: u16,
    profile: ScanProfile,
) -> anyhow::Result<()> {
    print_redaction_note();

    let ip: IpAddr = ip_str.parse()?;
    let conn = open_db(ctx)?;
    let net_id = match latest_network_id(&conn)? {
        Some(id) => id,
        None => anyhow::bail!("No hay inventario. Ejecuta `mylan scan` antes de escanear puertos."),
    };
    let device = mylan_db::device_repo::get_device_by_ip(&conn, &net_id, ip)?
        .ok_or_else(|| anyhow::anyhow!("No se encontró un dispositivo con IP {ip} en la red {net_id}. Ejecuta `mylan scan` primero."))?;

    // AC-7: el catálogo efectivo es el del perfil. --top es informativo.
    let profile_count = u16::try_from(ports_for_profile(profile).len()).unwrap_or(0);
    let fixed_catalog = matches!(profile, ScanProfile::Iot | ScanProfile::Router);
    if fixed_catalog {
        eprintln!(
            "[mylan] Warning: --top {top} ignorado para el perfil {profile:?} \
             (catálogo fijo de {profile_count} puertos)."
        );
    } else if top != profile_count {
        eprintln!(
            "[mylan] Warning: --top {top} difiere del conteo del perfil {profile:?} \
             ({profile_count}); el perfil tiene prioridad."
        );
    }

    let options = profile_options(profile);

    // AC-5: cancelación cooperativa vía Ctrl-C.
    let cancel = CancellationToken::new();
    let cancel_for_signal = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancel_for_signal.cancel();
        }
    });

    tracing::info!(ip = %ip, profile = ?profile, ?options, "iniciando escaneo de puertos");
    let verbose = ctx.verbose;
    let services = scan_target(ip, profile, options, cancel, |p: ScanProgress| {
        if verbose {
            eprintln!(
                "[mylan] progreso: {}/{} ({}%) último abierto={:?}",
                p.ports_tested, p.ports_total, p.percent_done, p.latest_open_port
            );
        }
    })
    .await
    .map_err(|e| match e {
        ScanError::Cancelled => anyhow::anyhow!("escaneo cancelado"),
        ScanError::Io(io) => anyhow::anyhow!("E/S de escaneo: {io}"),
    })?;
    tracing::info!(open = services.len(), "escaneo de puertos completado");

    if services.is_empty() {
        println!("No se detectaron puertos abiertos en {ip} (perfil {profile:?}).");
        return Ok(());
    }

    let now = now_rfc3339()?;
    for svc in &services {
        upsert_service(&conn, &fill_service(svc, &device.id, &now))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctx::AppContext;
    use mylan_core::{Protocol, Service, ServiceState};

    fn ctx_in(tmp: &std::path::Path) -> AppContext {
        AppContext {
            db_path: tmp.join("mylan.db"),
            signatures_dir: tmp.to_path_buf(),
            verbose: false,
        }
    }

    fn sample_service() -> Service {
        Service {
            id: "template-id".to_string(),
            device_id: "ignored".to_string(),
            protocol: Protocol::Tcp,
            port: 22,
            service_name: Some("ssh".to_string()),
            product: Some("OpenSSH".to_string()),
            version: Some("9.0".to_string()),
            banner: Some("SSH-2.0-OpenSSH_9.0".to_string()),
            state: ServiceState::Open,
            first_seen_at: "ignored".to_string(),
            last_seen_at: "ignored".to_string(),
        }
    }

    #[test]
    fn fill_service_copies_protocol_port_and_metadata() {
        let svc = sample_service();
        let now = "2024-01-01T00:00:00Z";
        let filled = fill_service(&svc, "dev-abc", now);
        assert_eq!(filled.protocol, Protocol::Tcp);
        assert_eq!(filled.port, 22);
        assert_eq!(filled.service_name.as_deref(), Some("ssh"));
        assert_eq!(filled.product.as_deref(), Some("OpenSSH"));
        assert_eq!(filled.version.as_deref(), Some("9.0"));
        assert_eq!(filled.banner.as_deref(), Some("SSH-2.0-OpenSSH_9.0"));
        assert_eq!(filled.state, ServiceState::Open);
    }

    #[test]
    fn fill_service_sets_device_id_and_timestamps() {
        let svc = sample_service();
        let now = "2024-06-15T12:30:00Z";
        let filled = fill_service(&svc, "dev-xyz", now);
        assert_eq!(filled.device_id, "dev-xyz");
        assert_eq!(filled.first_seen_at, now);
        assert_eq!(filled.last_seen_at, now);
    }

    #[test]
    fn fill_service_generates_fresh_uuid_id() {
        let svc = sample_service();
        let filled = fill_service(&svc, "dev-1", "now");
        assert_ne!(filled.id, "template-id");
        assert_eq!(filled.id.len(), 36);
        uuid::Uuid::parse_str(&filled.id).expect("id debe ser UUID válido");
    }

    #[test]
    fn fill_service_generates_unique_ids_across_calls() {
        let svc = sample_service();
        let a = fill_service(&svc, "d1", "t1");
        let b = fill_service(&svc, "d2", "t2");
        assert_ne!(a.id, b.id, "cada fill debe generar un id único");
    }

    #[tokio::test]
    async fn run_rejects_invalid_ip() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());
        let result = run(&ctx, "not-an-ip", 100, ScanProfile::Quick).await;
        assert!(result.is_err(), "IP inválida debe errar");
    }

    #[tokio::test]
    async fn run_errors_when_no_inventory() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());
        let result = run(&ctx, "192.168.1.1", 100, ScanProfile::Quick).await;
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("inventario") || msg.contains("scan"),
            "mensaje debe indicar falta de inventario: {msg}"
        );
    }
}
