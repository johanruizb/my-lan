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
//!
//! Vive en `mylan-db` (no en la CLI) para que CLI, Desktop y la futura
//! `mylan-api` reusen la **misma** orquestación liveness→enrich→persist sin
//! duplicarla ni arrastrar dependencias de CLI (`clap`, `comfy-table`).

use std::time::Instant;

use rusqlite::{Connection, Transaction};

use mylan_core::{
    aggregate, Device, Enricher, Event, Network, Observation, Scan, ScanKind, ScanProfile,
    ScanStatus, ScanSummary,
};

use crate::device_repo::UpsertOutcome;
use crate::util::{new_id, now_rfc3339};
use crate::{device_repo, diff, events_repo, network_repo, scan_repo};

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
///
/// # Errors
/// Propaga cualquier error de la transacción SQLite o del formateo de
/// timestamp; en ese caso la transacción revierte (sin escrituras a medias).
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
///
/// # Errors
/// Igual que [`run_scan_pipeline`]: propaga errores de la transacción SQLite.
pub fn run_scan_pipeline_at(
    conn: &Connection,
    network: &Network,
    observations: &[Observation],
    enricher: &Enricher,
    profile: ScanProfile,
    now: &str,
) -> anyhow::Result<ScanOutcome> {
    // Todo el escaneo es atómico: si algún upsert falla, la transacción revierte
    // y NO queda ni un `Scan` colgado en `running` ni escrituras de dispositivo
    // a medias. El error se propaga al llamante (anyhow → stderr). Thin wrapper
    // sobre `run_scan_pipeline_at_in_tx` (ADR-5 — refactor, no rewrite, AC-14).
    let tx = conn.unchecked_transaction()?;
    let outcome = run_scan_pipeline_at_in_tx(&tx, network, observations, enricher, profile, now)?;
    tx.commit()?;
    Ok(outcome)
}

/// Variante txn-composable (ADR-5): hace todo lo de [`run_scan_pipeline_at`]
/// EXCEPTO abrir la transacción y commitear — opera sobre el `tx` del caller.
///
/// `run_scan_pipeline_at` delega aquí + `tx.commit()` (AC-14: comportamiento
/// observable idéntico). El motor de diff ([`run_scan_pipeline_with_diff`]) la
/// llama dentro de una txn mayor que también persiste los events atómicamente
/// con el scan.
///
/// # Errors
/// Propaga errores de la transacción SQLite; al no commitear, el caller decide
/// el destino (commit o rollback).
pub fn run_scan_pipeline_at_in_tx(
    tx: &Transaction,
    network: &Network,
    observations: &[Observation],
    enricher: &Enricher,
    profile: ScanProfile,
    now: &str,
) -> anyhow::Result<ScanOutcome> {
    let start = Instant::now();

    network_repo::upsert_network(tx, network)?;

    let scan_id = new_id();
    scan_repo::insert_scan(
        tx,
        &Scan {
            id: scan_id.clone(),
            network_id: network.id.clone(),
            target_ip: None,
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
            device_repo::upsert_device(tx, &device)?,
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
        open_ports: 0,
    };
    scan_repo::finish_scan(tx, &scan_id, ScanStatus::Completed, now, Some(&summary))?;

    Ok(ScanOutcome {
        scan_id,
        network_id: network.id.clone(),
        hosts_alive,
        hosts_new,
        duration_ms,
    })
}

/// Pipeline + diff atómico en UNA transacción (ADR-5): snapshot antes → pipeline
/// (sin commitear) → diff → persistir events → commit una vez. Un crash antes
/// del commit revierte TODO (ni devices ni events quedan a medias).
///
/// `cold_start` suprime `device_online`/`device_offline` en el primer scan tras
/// restart del agent (ver [`diff::run_diff`]).
///
/// # Errors
/// Propaga errores SQLite o de diff; en ese caso la txn revierte (sin escrituras).
pub fn run_scan_pipeline_with_diff(
    conn: &Connection,
    network: &Network,
    observations: &[Observation],
    enricher: &Enricher,
    profile: ScanProfile,
    now: &str,
    cold_start: bool,
) -> anyhow::Result<(ScanOutcome, Vec<Event>)> {
    let tx = conn.unchecked_transaction()?;

    // Snapshots "before" dentro de la txn: ven el estado pre-scan (aún sin writes
    // del pipeline). ADR-5 — mismo txn que el scan + events.
    let before_devices = diff::snapshot_devices_before(&tx, &network.id)?;
    let before_device_ids: Vec<String> = before_devices.iter().map(|d| d.id.clone()).collect();
    let before_services = diff::snapshot_services_before(&tx, &before_device_ids)?;

    // Pipeline (NO commitea — opera sobre &tx dentro de esta txn mayor).
    let outcome = run_scan_pipeline_at_in_tx(&tx, network, observations, enricher, profile, now)?;

    // Diff: re-querya "after", escribe is_online, retorna events.
    let events = diff::run_diff(
        &tx,
        &network.id,
        now,
        before_devices,
        before_services,
        cold_start,
    )?;

    // Events persisten en la MISMA txn que el scan (ADR-5 atómico).
    for event in &events {
        events_repo::insert_event(&tx, event)?;
    }

    tx.commit()?;
    Ok((outcome, events))
}
