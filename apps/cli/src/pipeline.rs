//! Pipeline de dos fases del `mylan scan` (Paso 5 + integración final).
//!
//! Fase 1 — liveness: la capa `mylan-discovery` produce `Observation`s crudas
//! (en los tests se inyectan directamente). Fase 2 — enrichment + persist:
//! se agregan por identidad, se construye un `Device` por observación, se
//! enriquece con el `Enricher` (fingerprint) y se hace upsert en la DB. Al
//! final se registra el `Scan` con su resumen.
//!
//! `mylan scan` **no** escanea puertos de toda la /24 (presupuesto AC-12): el
//! port scan es bajo demanda vía `mylan ports <ip>`.

use std::time::Instant;

use rusqlite::Connection;

use mylan_core::{
    aggregate, Device, Enricher, Network, Observation, Scan, ScanKind, ScanProfile, ScanStatus,
    ScanSummary,
};
use mylan_db::device_repo::UpsertOutcome;
use mylan_db::{device_repo, network_repo, scan_repo};

use crate::util::{new_id, now_rfc3339};

/// Resultado agregado de un escaneo de descubrimiento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanOutcome {
    pub scan_id: String,
    pub network_id: String,
    pub hosts_alive: u32,
    pub hosts_new: u32,
    pub duration_ms: u64,
}

/// Ejecuta el pipeline enrichment → persist → registrar scan sobre las
/// `observations` ya descubiertas (fase liveness externa).
///
/// Es deliberadamente independiente de `mylan-discovery`: recibe las
/// `Observation`s para que los tests las inyecten sin red real. El comando
/// `scan` se encarga de llamar a `discover()` y luego a esta función.
///
/// El `network` se upserta primero (clave por `id`); cada `Observation`
/// agregada se mapea a un `Device`, se enriquece y se persiste por identidad
/// estable (sin duplicar, P5). `hosts_new` cuenta las inserciones.
pub fn run_scan_pipeline(
    conn: &Connection,
    network: &Network,
    observations: &[Observation],
    enricher: &Enricher,
    profile: ScanProfile,
) -> anyhow::Result<ScanOutcome> {
    let now = now_rfc3339()?;
    run_scan_pipeline_at(conn, network, observations, enricher, profile, &now)
}

/// Variante con timestamp inyectado (determinismo en tests).
pub fn run_scan_pipeline_at(
    conn: &Connection,
    network: &Network,
    observations: &[Observation],
    enricher: &Enricher,
    profile: ScanProfile,
    now: &str,
) -> anyhow::Result<ScanOutcome> {
    let start = Instant::now();

    // Todo el escaneo es atómico: si algún upsert falla, la transacción revierte
    // y NO queda ni un `Scan` colgado en `running` ni escrituras de dispositivo
    // a medias. El error se propaga al llamante (anyhow → stderr).
    let tx = conn.unchecked_transaction()?;

    network_repo::upsert_network(&tx, network)?;

    let scan_id = new_id();
    scan_repo::insert_scan(
        &tx,
        &Scan {
            id: scan_id.clone(),
            network_id: network.id.clone(),
            scan_type: ScanKind::Discovery,
            profile,
            status: ScanStatus::Running,
            started_at: now.to_string(),
            finished_at: None,
            summary: None,
        },
    )?;

    let aggregated = aggregate(observations);
    let hosts_alive = u32::try_from(aggregated.len()).unwrap_or(u32::MAX);
    let mut hosts_new = 0u32;

    for obs in &aggregated {
        let mut device = Device::new(new_id(), &network.id, now);
        device.merge_observation(obs, now);
        // El enricher recibe la observación agregada de este host (lleva los
        // hints mDNS/SSDP fusionados) para que el motor de reglas los evalúe.
        enricher(&mut device, std::slice::from_ref(obs));
        if matches!(
            device_repo::upsert_device(&tx, &device)?,
            UpsertOutcome::Inserted
        ) {
            hosts_new += 1;
        }
    }

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let summary = ScanSummary {
        hosts_alive,
        hosts_new,
        duration_ms,
    };
    scan_repo::finish_scan(&tx, &scan_id, ScanStatus::Completed, now, Some(&summary))?;

    tx.commit()?;

    Ok(ScanOutcome {
        scan_id,
        network_id: network.id.clone(),
        hosts_alive,
        hosts_new,
        duration_ms,
    })
}
