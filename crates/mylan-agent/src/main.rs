//! Binary `mylan-agent` (standalone, para systemd/docker). Flag `--serve-api`
//! para spawn el API in-process (ADR-4); `--config` y `--api-port` para config.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, Result};

use mylan_agent::{run_agent, AgentConfig};

fn main() -> ExitCode {
    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mylan-agent: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut serve_api = false;
    let mut config_path: Option<PathBuf> = None;
    let mut api_port: Option<u16> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--serve-api" => serve_api = true,
            "--config" => {
                i += 1;
                config_path = args.get(i).map(PathBuf::from);
            }
            "--api-port" => {
                i += 1;
                api_port = args.get(i).and_then(|s| s.parse().ok());
            }
            "-h" | "--help" => {
                eprintln!(
                    "uso: mylan-agent --serve-api --config <path> --api-port <port>\n\
                     \n\
                     --serve-api   sirve el API REST+WS in-process (ADR-4)\n\
                     --config <p>  ruta a mylan-agent.toml\n\
                     --api-port <n> puerto del API (default: el de la config)"
                );
                return Ok(());
            }
            other => {
                eprintln!("mylan-agent: arg desconocido: {other}");
            }
        }
        i += 1;
    }
    let config_path = config_path.ok_or_else(|| anyhow!("--config es requerido"))?;
    let config = AgentConfig::load(&config_path)?;
    let api_port = api_port.unwrap_or(config.api_port);
    let db_path = config.db_path()?;
    // C2 fix: el token se deriva del db_path del config (no de default_token_path
    // que necesita $HOME, ausente bajo systemd ProtectHome=true / Docker).
    let token_path = mylan_api::token_path_for_db(&db_path);
    let api_token = mylan_api::load_or_create_token(&token_path)?;
    if !serve_api {
        tracing::warn!("--serve-api no seteado; el API no se servirá (solo scheduler)");
    }
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_agent(&config_path, &db_path, api_port, &api_token))?;
    Ok(())
}
