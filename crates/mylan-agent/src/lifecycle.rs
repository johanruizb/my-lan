//! Lifecycle del agent: `CancellationToken` + signal handlers (ctrl_c, SIGTERM).
//!
//! [`shutdown_token`] crea el token; [`install_shutdown_handlers`] spawnea tasks
//! que lo cancelan al recibir ctrl_c (multiplataforma) o SIGTERM (Unix). El
//! scheduler loop y el API embebido consultan el mismo token para un shutdown
//! graceful (flush + exit 0).

use anyhow::Result;
use tokio_util::sync::CancellationToken;

/// Crea un `CancellationToken` para el shutdown del agent.
#[must_use]
pub fn shutdown_token() -> CancellationToken {
    CancellationToken::new()
}

/// Instala handlers para ctrl_c (multiplataforma) y SIGTERM (Unix) que cancelan
/// `token` al recibir la señal. Devuelve `Ok(())` tras instalar (los handlers
/// corren como tasks tokio en background).
///
/// # Errors
/// Nunca devuelve error actualmente (los handlers son best-effort; fallos de
/// instalación se loguean, no se propagan, para no bloquear el arranque).
pub fn install_shutdown_handlers(token: CancellationToken) -> Result<()> {
    let ctrl_c_token = token.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("ctrl_c recibido; cancelando agent");
            ctrl_c_token.cancel();
        }
    });

    #[cfg(unix)]
    {
        let sigterm_token = token.clone();
        tokio::spawn(async move {
            use tokio::signal::unix::{signal, SignalKind};
            match signal(SignalKind::terminate()) {
                Ok(mut s) => {
                    s.recv().await;
                    tracing::info!("SIGTERM recibido; cancelando agent");
                    sigterm_token.cancel();
                }
                Err(e) => {
                    tracing::warn!(error = %e, "no se pudo instalar handler SIGTERM");
                }
            }
        });
    }

    #[cfg(not(unix))]
    {
        let _ = token;
    }

    Ok(())
}
