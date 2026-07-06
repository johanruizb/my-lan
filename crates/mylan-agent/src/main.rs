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
    let parsed = parse_args(&args)?;
    let ParsedArgs {
        serve_api,
        config_path,
        api_port,
    } = match parsed {
        Some(p) => p,
        // --help/-h: ya se imprimió el uso; salir limpio.
        None => return Ok(()),
    };
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

/// Resultado del parseo de argumentos del binario `mylan-agent`.
struct ParsedArgs {
    serve_api: bool,
    config_path: Option<PathBuf>,
    api_port: Option<u16>,
}

/// Parsea los argumentos de la línea de comandos del binario `mylan-agent`.
///
/// Devuelve `Ok(None)` si se solicitó `--help`/`-h` (uso ya impreso), o
/// `Ok(Some(ParsedArgs))` con los flags parseados. Los argumentos desconocidos
/// se ignoran con un warning a stderr (comportamiento tolerante). El flag
/// `--config` es obligatorio; la validación la hace el llamador (`try_main`).
fn parse_args(args: &[String]) -> Result<Option<ParsedArgs>> {
    let mut serve_api = false;
    let mut config_path: Option<PathBuf> = None;
    let mut api_port: Option<u16> = None;
    let mut i = 1; // args[0] es el nombre del binario
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
                return Ok(None);
            }
            other => {
                eprintln!("mylan-agent: arg desconocido: {other}");
            }
        }
        i += 1;
    }
    Ok(Some(ParsedArgs {
        serve_api,
        config_path,
        api_port,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Nota de determinismo: `main`/`try_main` leen `std::env::args()` y
    // arrancan el runtime + agent (red/tokio). No se testean directamente. Sí
    // se testea `parse_args` (parseo puro sobre un slice de args controlado).

    fn args(raw: &[&str]) -> Vec<String> {
        std::iter::once("mylan-agent".to_string())
            .chain(raw.iter().map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn parse_args_serve_api_flag() {
        let parsed = parse_args(&args(&["--serve-api", "--config", "x.toml"]))
            .expect("ok")
            .expect("some");
        assert!(parsed.serve_api);
        assert_eq!(
            parsed.config_path.as_deref(),
            Some(std::path::Path::new("x.toml"))
        );
    }

    #[test]
    fn parse_args_serve_api_defaults_false() {
        let parsed = parse_args(&args(&["--config", "c.toml"]))
            .expect("ok")
            .expect("some");
        assert!(!parsed.serve_api);
    }

    #[test]
    fn parse_args_config_path_captured() {
        let parsed = parse_args(&args(&["--config", "/etc/mylan/agent.toml"]))
            .expect("ok")
            .expect("some");
        assert_eq!(
            parsed.config_path.as_deref(),
            Some(std::path::Path::new("/etc/mylan/agent.toml"))
        );
    }

    #[test]
    fn parse_args_config_missing_when_flag_without_value() {
        // `--config` al final sin valor → config_path = None (no entra en pánico).
        let parsed = parse_args(&args(&["--config"])).expect("ok").expect("some");
        assert!(parsed.config_path.is_none());
    }

    #[test]
    fn parse_args_api_port_parsed() {
        let parsed = parse_args(&args(&["--config", "x.toml", "--api-port", "8080"]))
            .expect("ok")
            .expect("some");
        assert_eq!(parsed.api_port, Some(8080));
    }

    #[test]
    fn parse_args_api_port_invalid_defaults_none() {
        // Valor no numérico → None (no error; el default de la config aplica).
        let parsed = parse_args(&args(&["--config", "x.toml", "--api-port", "not-a-port"]))
            .expect("ok")
            .expect("some");
        assert!(parsed.api_port.is_none());
    }

    #[test]
    fn parse_args_api_port_missing_value() {
        let parsed = parse_args(&args(&["--config", "x.toml", "--api-port"]))
            .expect("ok")
            .expect("some");
        assert!(parsed.api_port.is_none());
    }

    #[test]
    fn parse_args_help_returns_none() {
        let parsed = parse_args(&args(&["--help"])).expect("ok");
        assert!(parsed.is_none(), "--help debe devolver None");
    }

    #[test]
    fn parse_args_short_help_returns_none() {
        let parsed = parse_args(&args(&["-h"])).expect("ok");
        assert!(parsed.is_none(), "-h debe devolver None");
    }

    #[test]
    fn parse_args_help_takes_precedence_over_other_flags() {
        // Aunque haya otros flags, -h se atiende cuando se alcanza.
        let parsed = parse_args(&args(&["--serve-api", "-h"])).expect("ok");
        assert!(parsed.is_none());
    }

    #[test]
    fn parse_args_unknown_arg_ignored() {
        // Los args desconocidos no rompen el parseo (warning a stderr).
        let parsed = parse_args(&args(&["--config", "x.toml", "--bogus", "--serve-api"]))
            .expect("ok")
            .expect("some");
        assert!(parsed.serve_api);
        assert_eq!(
            parsed.config_path.as_deref(),
            Some(std::path::Path::new("x.toml"))
        );
    }

    #[test]
    fn parse_args_no_args_yields_empty_parsed() {
        // Solo el nombre del binario → todo por defecto (vacío).
        let parsed = parse_args(&args(&[])).expect("ok").expect("some");
        assert!(!parsed.serve_api);
        assert!(parsed.config_path.is_none());
        assert!(parsed.api_port.is_none());
    }

    #[test]
    fn parse_args_full_invocation() {
        let parsed = parse_args(&args(&[
            "--serve-api",
            "--config",
            "/var/lib/mylan/agent.toml",
            "--api-port",
            "43117",
        ]))
        .expect("ok")
        .expect("some");
        assert!(parsed.serve_api);
        assert_eq!(
            parsed.config_path.as_deref(),
            Some(std::path::Path::new("/var/lib/mylan/agent.toml"))
        );
        assert_eq!(parsed.api_port, Some(43117));
    }
}
