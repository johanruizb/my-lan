//! Scheduler loop: escaneo periódico por red con skip guard + Semaphore +
//! cold_start tracking. Emite `Event`s al broadcast (ADR-4).
//!
//! - **Skip guard** ([`NetworkRunner`]): si un scan de una red ya está en curso,
//!   el siguiente tick la SKIP (no solapa scans de la misma red) — distinto de
//!   un Semaphore que solo throttla concurrencia global.
//! - **Semaphore**: throttle de concurrencia total (match `AGENTS.md`).
//! - **cold_start** por red: `true` en el primer tick tras (re)start del agent
//!   → suprime `device_online`/`device_offline` (plan Step 2, Q5).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::Result;
use tokio::sync::{Mutex, Semaphore, broadcast};
use tokio_util::sync::CancellationToken;

use mylan_core::{Enricher, Event, Network, Observation};
use mylan_db::util::now_rfc3339;

use crate::config::{AgentConfig, NetworkSchedule};
use crate::privilege::discover_with_degradation;

/// Ejecuta una scan individual para una red: abre conn, construye el `Network`,
/// llama `run_scan_pipeline_with_diff`, emite los events al broadcast.
///
/// `observations` se inyecta (en producción lo produce
/// [`discover_with_degradation`]; en tests se pasa directo). Devuelve el
/// `ScanOutcome`.
///
/// # Errors
/// Propaga errores de conexión SQLite, parseo de CIDR, o del pipeline+diff.
pub async fn scan_network(
    db_path: &Path,
    net: &NetworkSchedule,
    observations: &[Observation],
    enricher: &Enricher,
    cold_start: bool,
    event_tx: &broadcast::Sender<Event>,
) -> Result<mylan_db::pipeline::ScanOutcome> {
    let conn = mylan_db::connection::connect(db_path)?;
    let network = build_network(net)?;
    let now = now_rfc3339()?;
    let (outcome, events) = mylan_db::pipeline::run_scan_pipeline_with_diff(
        &conn,
        &network,
        observations,
        enricher,
        net.profile,
        &now,
        cold_start,
    )?;
    for event in events {
        // broadcast::send es no-op si no hay receptores (no backpressure); la
        // DB (events table) es la fuente de verdad, el WS es vista en vivo.
        let _ = event_tx.send(event);
    }
    Ok(outcome)
}

/// Loop del scheduler: cada `interval_secs` intenta escanear cada red de la
/// config. Retorna `Ok(())` al cancelarse vía `cancel`.
///
/// # Errors
/// Propaga errores solo si el setup inicial falla; los errores de scan
/// individuales se loguean (no abortan el loop).
pub async fn run_scheduler(
    config: &AgentConfig,
    db_path: &Path,
    enricher: &std::sync::Arc<Enricher>,
    event_tx: &broadcast::Sender<Event>,
    cancel: CancellationToken,
) -> Result<()> {
    let runner = Arc::new(NetworkRunner::new());
    let semaphore = Arc::new(Semaphore::new(config.networks.len().max(1)));
    let cold_start: Arc<Mutex<HashMap<String, bool>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut ticker = tokio::time::interval(Duration::from_secs(config.interval_secs));
    loop {
        tokio::select! {
            () = cancel.cancelled() => {
                tracing::info!("scheduler cancelado; saliendo");
                return Ok(());
            }
            _ = ticker.tick() => {
                for net in &config.networks {
                    let key = net.cidr.clone();
                    if !runner.try_start(&key).await {
                        tracing::warn!(%key, "scan previo aún en curso; skip este tick");
                        continue;
                    }
                    // cold_start: true la primera vez que se ve esta red.
                    let cs = {
                        let mut cs_map = cold_start.lock().await;
                        let was_cold = !cs_map.contains_key(&key);
                        cs_map.insert(key.clone(), false);
                        was_cold
                    };
                    let db = db_path.to_path_buf();
                    let net_clone = net.clone();
                    let enricher_clone = std::sync::Arc::clone(enricher);
                    let event_tx_clone = event_tx.clone();
                    let runner_clone = Arc::clone(&runner);
                    let sem_clone = Arc::clone(&semaphore);
                    tokio::spawn(async move {
                        let _permit = match sem_clone.acquire_owned().await {
                            Ok(p) => p,
                            Err(e) => {
                                tracing::error!(error = %e, "semaphore acquire falló");
                                runner_clone.mark_done(&key).await;
                                return;
                            }
                        };
                        let observations =
                            discover_with_degradation(&net_clone.cidr, net_clone.profile).await;
                        if let Err(e) = scan_network(
                            &db,
                            &net_clone,
                            &observations,
                            &enricher_clone,
                            cs,
                            &event_tx_clone,
                        )
                        .await
                        {
                            tracing::error!(cidr = %net_clone.cidr, error = %e, "scan falló");
                        }
                        runner_clone.mark_done(&key).await;
                    });
                }
            }
        }
    }
}

/// Skip guard por red: trackea qué redes tienen un scan en curso. Si una red ya
/// está being scanned, el siguiente tick la SKIP — distinto de un Semaphore
/// (que solo throttla concurrencia global, no evita solapamiento de la misma red).
#[derive(Default)]
pub struct NetworkRunner {
    running: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl NetworkRunner {
    /// Crea un `NetworkRunner` vacío.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Intenta empezar un scan para `network_key`. Devuelve `true` si se puede
    /// empezar (no había uno en curso); `false` si ya hay uno (skip).
    pub async fn try_start(&self, network_key: &str) -> bool {
        let mut map = self.running.lock().await;
        if let Some(flag) = map.get(network_key) {
            if flag.load(Ordering::SeqCst) {
                return false;
            }
        }
        map.insert(network_key.to_string(), Arc::new(AtomicBool::new(true)));
        true
    }

    /// Marca el scan de `network_key` como terminado (libera el skip guard).
    pub async fn mark_done(&self, network_key: &str) {
        let mut map = self.running.lock().await;
        map.remove(network_key);
    }
}

/// Construye un `Network` de mylan-core a partir del `NetworkSchedule`. El id
/// es el CIDR (M1 fix: match CLI `mylan scan` que usa `iface.cidr()` como id)
/// para que re-escaneos hagan upsert sin duplicar redes entre agent y CLI.
fn build_network(net: &NetworkSchedule) -> Result<Network> {
    let now = now_rfc3339()?;
    Ok(Network {
        id: net.cidr.clone(),
        name: net.cidr.clone(),
        cidr: net.cidr.clone(),
        gateway_ip: None,
        dns_servers: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    })
}