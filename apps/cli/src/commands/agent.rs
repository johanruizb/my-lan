//! `mylan agent` — gestión del daemon (start | run | stop).
//!
//! ADR-4: el agent y el API corren en un único proceso. `run` es foreground
//! (debug); `start` lanza el binario `mylan-agent` en background + pidfile;
//! `stop` envía SIGTERM al pid del pidfile.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::cli::AgentSub;
use crate::ctx::AppContext;

/// Despacha el subcomando `agent`.
pub async fn run(ctx: &AppContext, what: AgentSub) -> Result<()> {
    match what {
        AgentSub::Start { config, api_port } => run_start(ctx, config.as_deref(), api_port),
        AgentSub::Run { config, api_port } => run_run(ctx, config.as_deref(), api_port).await,
        AgentSub::Stop => run_stop(ctx),
    }
}

/// `mylan agent run` — foreground: agent loop + API en un proceso (debug, ADR-4).
async fn run_run(_ctx: &AppContext, config: Option<&str>, api_port: Option<u16>) -> Result<()> {
    let config_path = resolve_config_path(config)?;
    let cfg = mylan_agent::AgentConfig::load(&config_path)?;
    let port = api_port.unwrap_or(cfg.api_port);
    let db_path = cfg.db_path()?;
    let token = load_token(&db_path)?;
    println!("[mylan] agent run: foreground (agent + API en un proceso, ADR-4)");
    println!("[mylan]   config: {}", config_path.display());
    println!("[mylan]   db    : {}", db_path.display());
    println!("[mylan]   api   : 127.0.0.1:{port}");
    mylan_agent::run_agent(&config_path, &db_path, port, &token).await
}

/// `mylan agent start` — daemon en background (exec `mylan-agent` + pidfile).
fn run_start(ctx: &AppContext, config: Option<&str>, api_port: Option<u16>) -> Result<()> {
    let config_path = resolve_config_path(config)?;
    let port = resolve_api_port(api_port, &config_path)?;
    let pidfile = pidfile_path(ctx);
    if let Some(pid) = read_pid(&pidfile) {
        if is_process_alive(pid) {
            return Err(anyhow!(
                "agent ya corre (pid {pid}); usa `mylan agent stop` primero"
            ));
        }
    }
    let mut cmd = std::process::Command::new(mylan_agent_binary());
    cmd.arg("--serve-api")
        .arg("--config")
        .arg(&config_path)
        .arg("--api-port")
        .arg(port.to_string());
    let child = cmd
        .spawn()
        .map_err(|e| anyhow!("arrancando `mylan-agent` (¿en PATH?): {e}"))?;
    let pid = child.id();
    std::fs::write(&pidfile, pid.to_string())
        .map_err(|e| anyhow!("escribiendo pidfile {}: {e}", pidfile.display()))?;
    println!(
        "[mylan] agent start: daemon pid {pid} (pidfile {})",
        pidfile.display()
    );
    // Detached: soltamos el handle para no esperar al child.
    drop(child);
    Ok(())
}

/// `mylan agent stop` — SIGTERM al pid del pidfile.
fn run_stop(ctx: &AppContext) -> Result<()> {
    let pidfile = pidfile_path(ctx);
    let pid = read_pid(&pidfile).ok_or_else(|| {
        anyhow!(
            "pidfile {} no encontrado o inválido (¿agent no corre?)",
            pidfile.display()
        )
    })?;
    send_sigterm(pid).map_err(|e| anyhow!("enviando SIGTERM a pid {pid}: {e}"))?;
    let _ = std::fs::remove_file(&pidfile);
    println!("[mylan] agent stop: SIGTERM a pid {pid}");
    Ok(())
}

/// Resuelve la ruta de config: override o default (~/.config/mylan/mylan-agent.toml).
fn resolve_config_path(config: Option<&str>) -> Result<PathBuf> {
    if let Some(p) = config {
        return Ok(PathBuf::from(p));
    }
    mylan_agent::AgentConfig::default_config_path()
        .ok_or_else(|| anyhow!("no se pudo resolver config path (sin $HOME); usa --config"))
}

/// Resuelve el puerto del API: override o el de la config.
fn resolve_api_port(api_port: Option<u16>, config_path: &Path) -> Result<u16> {
    if let Some(p) = api_port {
        return Ok(p);
    }
    let cfg = mylan_agent::AgentConfig::load(config_path)?;
    Ok(cfg.api_port)
}

/// Carga (o crea) el token del API, derivado del `db_path` del config (C2 fix:
/// robusto a entornos sin `$HOME` como systemd/Docker — no usa
/// `default_token_path()` que depende de `$HOME`).
fn load_token(db_path: &Path) -> Result<String> {
    let path = mylan_api::token_path_for_db(db_path);
    mylan_api::load_or_create_token(&path)
}

/// Path del pidfile: junto a la DB (parent dir de `db_path`).
fn pidfile_path(ctx: &AppContext) -> PathBuf {
    ctx.db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("mylan-agent.pid")
}

/// Lee el pid del pidfile (si existe y es válido).
fn read_pid(pidfile: &Path) -> Option<i32> {
    let s = std::fs::read_to_string(pidfile).ok()?;
    s.trim().parse::<i32>().ok()
}

/// Nombre del binario `mylan-agent` (asume que está en `PATH`).
fn mylan_agent_binary() -> PathBuf {
    PathBuf::from("mylan-agent")
}

/// `true` si el proceso `pid` está vivo.
#[cfg(unix)]
fn is_process_alive(pid: i32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_process_alive(pid: i32) -> bool {
    std::process::Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {pid}"))
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}

/// Envía SIGTERM (Unix) / taskkill (Windows) al pid.
#[cfg(unix)]
fn send_sigterm(pid: i32) -> Result<()> {
    let status = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status()
        .map_err(|e| anyhow!("kill -TERM {pid}: {e}"))?;
    if !status.success() {
        return Err(anyhow!("kill -TERM {pid} falló (exit {:?})", status.code()));
    }
    Ok(())
}

#[cfg(not(unix))]
fn send_sigterm(pid: i32) -> Result<()> {
    let status = std::process::Command::new("taskkill")
        .arg("/PID")
        .arg(pid.to_string())
        .arg("/T")
        .status()
        .map_err(|e| anyhow!("taskkill {pid}: {e}"))?;
    if !status.success() {
        return Err(anyhow!("taskkill {pid} falló"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::AgentSub;
    use crate::ctx::AppContext;

    fn ctx_in(tmp: &std::path::Path) -> AppContext {
        AppContext {
            db_path: tmp.join("mylan.db"),
            signatures_dir: tmp.to_path_buf(),
            verbose: false,
        }
    }

    #[test]
    fn mylan_agent_binary_is_pathname() {
        assert_eq!(mylan_agent_binary(), PathBuf::from("mylan-agent"));
    }

    #[test]
    fn pidfile_path_uses_db_parent_dir() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());
        let pidfile = pidfile_path(&ctx);
        assert_eq!(pidfile.file_name().unwrap(), "mylan-agent.pid");
        // El pidfile vive junto a la DB (parent dir de `db_path`).
        assert_eq!(pidfile.parent().unwrap(), ctx.db_path.parent().unwrap());
    }

    #[test]
    fn read_pid_returns_none_when_missing() {
        let tmp = tempfile::tempdir().expect("tmp");
        let pidfile = tmp.path().join("no-existe.pid");
        assert!(read_pid(&pidfile).is_none());
    }

    #[test]
    fn read_pid_returns_none_for_invalid_content() {
        let tmp = tempfile::tempdir().expect("tmp");
        let pidfile = tmp.path().join("bad.pid");
        std::fs::write(&pidfile, "not-a-number").unwrap();
        assert!(read_pid(&pidfile).is_none());
    }

    #[test]
    fn read_pid_parses_valid_pid() {
        let tmp = tempfile::tempdir().expect("tmp");
        let pidfile = tmp.path().join("agent.pid");
        std::fs::write(&pidfile, "12345\n").unwrap();
        assert_eq!(read_pid(&pidfile), Some(12345));
    }

    #[test]
    fn read_pid_trims_whitespace() {
        let tmp = tempfile::tempdir().expect("tmp");
        let pidfile = tmp.path().join("agent.pid");
        std::fs::write(&pidfile, "  6789  \n").unwrap();
        assert_eq!(read_pid(&pidfile), Some(6789));
    }

    #[test]
    fn resolve_config_path_explicit_override() {
        let p = resolve_config_path(Some("/tmp/mylan-agent.toml")).unwrap();
        assert_eq!(p, PathBuf::from("/tmp/mylan-agent.toml"));
    }

    #[test]
    fn resolve_config_path_explicit_relative() {
        let p = resolve_config_path(Some("custom.toml")).unwrap();
        assert_eq!(p, PathBuf::from("custom.toml"));
    }

    #[tokio::test]
    async fn run_stop_errors_when_no_pidfile() {
        // Sin pidfile previo, `agent stop` debe reportar error determinista
        // (no depende de procesos externos ni red).
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());
        let result = run(&ctx, AgentSub::Stop).await;
        assert!(result.is_err(), "agent stop sin pidfile debe errar");
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("pidfile"),
            "mensaje debe mencionar pidfile: {msg}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn is_process_alive_current_pid_is_alive() {
        // El proceso del test mismo está vivo (determinista: no requiere red).
        let pid = std::process::id() as i32;
        assert!(is_process_alive(pid), "pid propio ({pid}) debe estar vivo");
    }

    #[cfg(unix)]
    #[test]
    fn is_process_alive_dead_pid_is_false() {
        // PID muy alto, fuera del rango máximo de pids reales (Linux: 4194304
        // por defecto). `kill -0` sobre un pid inexistente → ESRCH → exit != 0.
        let dead_pid = i32::MAX - 1;
        assert!(
            !is_process_alive(dead_pid),
            "pid {dead_pid} no debe estar vivo"
        );
    }

    #[cfg(unix)]
    #[test]
    fn send_sigterm_dead_pid_errors() {
        let dead_pid = i32::MAX - 1;
        let result = send_sigterm(dead_pid);
        assert!(result.is_err(), "SIGTERM a pid inexistente debe errar");
    }
}
