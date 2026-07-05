//! `mylan-agent` — daemon de escaneo periódico + API embebida (v0.5 Watch, Step 4).
//!
//! Topología ADR-4: un único proceso (`mylan-agent`) posee el canal de broadcast
//! `tokio::sync::broadcast`, spawnea `mylan_api::serve` IN-PROCESS como tarea
//! tokio, y corre el scheduler loop que emite `Event`s al `Sender`. Sin estado
//! mutable compartido salvo SQLite (file-locked).
//!
//! Lifecycle: `CancellationToken` (tokio_util) + ctrl_c/SIGTERM → cancel + exit
//! 0. Privilegios: degradación elegante (ARP sweep sudo opcional, fallback
//! ICMP/TCP-ping/mDNS/SSDP) — nunca crash (P1).

#![forbid(unsafe_code)]

mod config;
mod lifecycle;
mod privilege;
mod scheduler;
mod windows_service;

pub use config::{AgentConfig, NetworkSchedule};
pub use lifecycle::shutdown_token;
pub use privilege::discover_with_degradation;
pub use scheduler::{run_scheduler, scan_network, NetworkRunner};
pub use windows_service::install_service;

use std::path::Path;

use anyhow::Result;
use tokio_util::sync::CancellationToken;

/// Arranca el agent: canal de broadcast + API embebida + scheduler loop (ADR-4).
///
/// `config_path` apunta a `mylan-agent.toml`; `db_path` a la DB SQLite; `api_port`
/// es el puerto del API embebido; `api_token` es el bearer token del API (AC-5).
/// Single process: el agent y el API comparten el canal de events.
pub async fn run_agent(
    config_path: impl AsRef<Path>,
    db_path: impl AsRef<Path>,
    api_port: u16,
    api_token: &str,
) -> Result<()> {
    let config = config::AgentConfig::load(config_path.as_ref())?;
    let cancel = lifecycle::shutdown_token();
    lifecycle::install_shutdown_handlers(cancel.clone())?;
    run_agent_with_cancel(&config, db_path.as_ref(), api_port, api_token, cancel).await
}

/// Variante con `CancellationToken` inyectado (tests): mismo flujo que
/// [`run_agent`] pero el caller controla el shutdown vía `cancel`.
pub async fn run_agent_with_cancel(
    config: &AgentConfig,
    db_path: &std::path::Path,
    api_port: u16,
    api_token: &str,
    cancel: CancellationToken,
) -> Result<()> {
    let (event_tx, _event_rx) = mylan_api::event_channel(1024);

    // API embebida IN-PROCESS (ADR-4). El agent posee el Sender; el API lo
    // guarda en axum::State para que /events/live se suscriba.
    let serve_db = db_path.to_path_buf();
    let serve_token = api_token.to_string();
    let serve_tx = event_tx.clone();
    let serve_handle = tokio::spawn(async move {
        if let Err(e) = mylan_api::serve(serve_db, api_port, &serve_token, serve_tx).await {
            tracing::error!(error = %e, "mylan-api serve finalizada con error");
        }
    });

    // Enricher de fingerprint (signatures). Si falla la carga, degradamos a
    // noop_enricher (el agent sigue funcionando sin clasificación).
    let enricher: mylan_core::Enricher = match load_enricher() {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(error = %e, "fingerprint signatures no cargadas; usando noop_enricher");
            mylan_core::noop_enricher()
        }
    };

    // Scheduler loop: emite events al event_tx; retorna Ok(()) al cancelarse.
    let enricher_arc = std::sync::Arc::new(enricher);
    let sched_result =
        scheduler::run_scheduler(config, db_path, &enricher_arc, &event_tx, cancel).await;

    // Shutdown: abortamos el task del server (comparte el cancel internamente
    // vía el handle; axum::serve termina al abortar el task).
    serve_handle.abort();
    sched_result
}

/// Carga el `Enricher` de `mylan-fingerprint` desde `signatures/`. En tests se
/// usa `mylan_core::noop_enricher` directamente (sin pasar por aquí).
fn load_enricher() -> Result<mylan_core::Enricher> {
    let fp = mylan_fingerprint::Fingerprint::load(std::path::Path::new("signatures"), None)?;
    Ok(fp.enricher())
}
