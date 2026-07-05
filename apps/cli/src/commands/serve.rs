//! `mylan serve` — foreground agent + API en un proceso (debug alias de
//! `mylan agent run`, ADR-4). Single process: agent loop + API embebido.

use anyhow::{Result, anyhow};

use crate::ctx::AppContext;

/// Arranca el agent + API embebido en foreground (debug alias).
///
/// Requiere `mylan-agent.toml` (default `~/.config/mylan/mylan-agent.toml` o
/// `--config`). El token del API se carga/crea bajo el directorio de datos.
pub async fn run(ctx: &AppContext, port: u16) -> Result<()> {
    let config_path = mylan_agent::AgentConfig::default_config_path().ok_or_else(|| {
        anyhow!(
            "no se pudo resolver config path (sin $HOME); crea \
             ~/.config/mylan/mylan-agent.toml o usa `mylan agent run --config`"
        )
    })?;
    let cfg = mylan_agent::AgentConfig::load(&config_path)?;
    let db_path = cfg.db_path()?;
    // C2 fix: token derivado del db_path del config (no de default_token_path
    // que necesita $HOME, ausente bajo systemd/Docker).
    let token_path = mylan_api::token_path_for_db(&db_path);
    let token = mylan_api::load_or_create_token(&token_path)?;
    if ctx.verbose {
        eprintln!("[mylan] serve: foreground agent + API (debug alias de `mylan agent run`)");
        eprintln!("[mylan]   config: {}", config_path.display());
        eprintln!("[mylan]   db    : {}", db_path.display());
        eprintln!("[mylan]   api   : 127.0.0.1:{port}");
    }
    mylan_agent::run_agent(&config_path, &db_path, port, &token).await
}