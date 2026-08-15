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

    // Mutex para serializar tests que mutan `XDG_CONFIG_HOME`/`HOME` (evita
    // carrera con otros tests del crate que puedan leer las variables
    // concurrentemente).
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn ctx_in(tmp: &std::path::Path) -> AppContext {
        AppContext {
            db_path: tmp.join("mylan.db"),
            signatures_dir: tmp.to_path_buf(),
            verbose: false,
        }
    }

    /// RAII guard que captura el valor previo de una var de entorno al
    /// construirse y lo restaura en `Drop`, incluso ante pánico. Evita
    /// corrupción del entorno de proceso y cascadas de envenenamiento del
    /// `ENV_LOCK` si un `block_on` o `assert!` panic en medio de la mutación.
    struct EnvGuard {
        key: &'static str,
        old: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: Option<&std::ffi::OsStr>) -> Self {
            let old = std::env::var_os(key);
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
            EnvGuard { key, old }
        }

        fn remove(key: &'static str) -> Self {
            let old = std::env::var_os(key);
            std::env::remove_var(key);
            EnvGuard { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match self.old.take() {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
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
        // unwrap_or_else sobre poison: si un test previo envenenó el mutex,
        // recuperamos el guard interno en vez de cascader el fallo.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());

        // Apunta XDG_CONFIG_HOME al tempdir: default_config_path devuelve
        // `<tmp>/mylan/mylan-agent.toml` que no existe → AgentConfig::load falla.
        let _xdg = EnvGuard::set("XDG_CONFIG_HOME", Some(tmp.path().as_os_str()));

        let rt = tokio::runtime::Runtime::new().expect("tokio rt");
        let result = rt.block_on(run(&ctx, 43117));

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
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tmp");
        let ctx = ctx_in(tmp.path());

        let _xdg = EnvGuard::remove("XDG_CONFIG_HOME");
        let _home = EnvGuard::remove("HOME");

        let rt = tokio::runtime::Runtime::new().expect("tokio rt");
        let result = rt.block_on(run(&ctx, 43117));

        assert!(result.is_err(), "run sin HOME/XDG debe errar");
    }
}
