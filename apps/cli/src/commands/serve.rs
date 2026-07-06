//! `mylan serve` — foreground agent + API en un proceso (debug alias de
//! `mylan agent run`, ADR-4). Single process: agent loop + API embebido.

use anyhow::{anyhow, Result};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctx::AppContext;

    // Mutex para serializar tests que mutan `XDG_CONFIG_HOME` (evita carrera
    // con otros tests del crate que puedan leer la variable concurrentemente).
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn ctx_in(tmp: &std::path::Path) -> AppContext {
        AppContext {
            db_path: tmp.join("mylan.db"),
            signatures_dir: tmp.to_path_buf(),
            verbose: false,
        }
    }

    // Nota de determinismo: `run` arranca el agent + API (red/tokio runtime).
    // No se testea el happy path. Sí se testea el error determinista cuando el
    // config no existe (XDG_CONFIG_HOME apunta a un tempdir vacío).

    // Nota: usamos `block_on` (sync) en vez de `#[tokio::test]` + `.await` para
    // que el `MutexGuard` de `ENV_LOCK` no se mantenga a través de un punto
    // `.await` (clippy `await_holding_lock`). El guard serializa los tests que
    // mutan `XDG_CONFIG_HOME`/`HOME`; `block_on` corre el future en el hilo
    // actual sin soltar el lock.

    #[test]
    fn run_errors_when_config_file_missing() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());

        // Apunta XDG_CONFIG_HOME al tempdir: default_config_path devuelve
        // `<tmp>/mylan/mylan-agent.toml` que no existe → AgentConfig::load falla.
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let rt = tokio::runtime::Runtime::new().expect("tokio rt");
        let result = rt.block_on(run(&ctx, 43117));

        // Restaurar el entorno.
        match old_xdg {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }

        assert!(result.is_err(), "run debe errar si el config no existe");
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("config") || msg.contains("leyendo"),
            "mensaje debe mencionar config: {msg}"
        );
    }

    #[test]
    fn run_errors_when_no_home_no_xdg() {
        // Sin XDG_CONFIG_HOME ni HOME, default_config_path devuelve None →
        // error "no se pudo resolver config path".
        let _guard = ENV_LOCK.lock().expect("env lock");
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());

        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let old_home = std::env::var("HOME").ok();
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("HOME");

        let rt = tokio::runtime::Runtime::new().expect("tokio rt");
        let result = rt.block_on(run(&ctx, 43117));

        match old_xdg {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
        match old_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }

        assert!(result.is_err(), "run sin HOME/XDG debe errar");
    }
}
