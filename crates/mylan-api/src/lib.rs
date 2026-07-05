//! `mylan-api` — facade REST+WS read-mostly sobre `mylan-db` (v0.5 Watch, Step 3).
//!
//! Topología ADR-4: el agent (único proceso) posee el canal de broadcast
//! `tokio::sync::broadcast` y pasa el `Sender<Event>` al API vía [`serve`]; el
//! API lo guarda en `axum::State` para que `/api/v1/events/live` (WS) pueda
//! suscribirse. Sin estado mutable compartido salvo SQLite (file-locked).
//!
//! Skeleton (Task #6): [`serve`] arranca el servidor; `auth`/`token`/`broadcast`
//! están implementados; `routes`/`ws` son stubs que worker-7 (Task #7) llenará
//! con los 8 endpoints REST + el WS `/events/live`.
//!
//! Modelo de seguridad: localhost-only (`127.0.0.1`), bearer token (ADR-7), sin
//! TLS ni auth remoto (no-goal v0.5).

#![forbid(unsafe_code)]

mod auth;
mod broadcast;
pub mod routes;
mod token;
pub mod ws;

pub use auth::{require_token, TokenMiddleware};
pub use broadcast::event_channel;
pub use token::{
    default_token_path, generate_token, load_or_create_token, rotate_token, token_path_for_db,
};

use std::path::PathBuf;
use std::sync::Arc;

use axum::middleware;
use axum::Router;
use thiserror::Error;

use mylan_core::Event;

/// Estado compartido por los handlers del API (axum `State`).
///
/// `Clone` barato: `Arc<String>` y `broadcast::Sender<Event>` son referencias
/// contadas; `db_path` se clona por valor.
#[derive(Clone)]
pub struct AppState {
    /// Ruta de la DB SQLite (el API abre conexiones de lectura bajo demanda).
    pub db_path: PathBuf,
    /// Token bearer esperado en `Authorization: Bearer <token>` (AC-5, ADR-7).
    pub token: Arc<String>,
    /// `Sender` del canal de broadcast que el agent posee (ADR-4). El API lo
    /// comparte vía `axum::State` para que el WS `/events/live` se suscriba.
    pub event_tx: tokio::sync::broadcast::Sender<Event>,
}

/// Error del API con conversión a respuesta HTTP (worker-7 extiende las
/// variantes en Task #7).
#[derive(Debug, Error)]
pub enum ApiError {
    /// El endpoint está implementado como stub en el skeleton (Task #6) y se
    /// completará en Task #7.
    #[error("not implemented: skeleton stub pending task #7")]
    NotImplemented,
    /// Recurso no encontrado (p.ej. `GET /devices/:id` sin fila).
    #[error("not found: {0}")]
    NotFound(String),
    /// Petición mal formada (param/body inválido).
    #[error("bad request: {0}")]
    BadRequest(String),
    /// Error interno (DB, discovery, pipeline).
    #[error("internal error: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let status = match &self {
            ApiError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

/// Arranca el servidor API en `127.0.0.1:port` (ADR-4).
///
/// El `event_tx` es el `Sender` del canal de broadcast que el agent posee; el
/// API lo guarda en `AppState` para que `/api/v1/events/live` (WS) se suscriba.
/// Bind solo a `127.0.0.1` (localhost-only, guardrail v0.5 — no `0.0.0.0`).
pub async fn serve(
    db_path: PathBuf,
    port: u16,
    token: &str,
    event_tx: tokio::sync::broadcast::Sender<Event>,
) -> anyhow::Result<()> {
    let state = AppState {
        db_path,
        token: Arc::new(token.to_string()),
        event_tx,
    };
    let mw = TokenMiddleware::new(state.token.clone());
    let app = Router::new()
        .merge(routes::router())
        .merge(ws::router())
        .layer(middleware::from_fn_with_state(mw, require_token))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    tracing::info!(port, "mylan-api listening on 127.0.0.1");
    axum::serve(listener, app).await?;
    Ok(())
}
