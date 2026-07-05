//! WebSocket `/api/v1/events/live` (AC-6: timeline en vivo + backfill `?since`).
//!
//! ADR-4: el handler se suscribe al `event_tx` (`broadcast::Sender<Event>`)
//! guardado en `AppState` y le hace upgrade a WebSocket. Handshake con
//! `?since=<ISO8601>` → backfill desde la DB (`events_repo::list_events_since`)
//! antes de streamear el live del `broadcast::Receiver`. `RecvError::Lagged` (el
//! cliente se quedó atrás por broadcasts lentos) → notificación `{"lagged":N}` y
//! sigue; `Closed` (agent apagado) → cierra. La DB es la fuente de verdad; el WS
//! es una vista en vivo (Principle 4).

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use tokio::sync::broadcast;

use mylan_core::Event;
use mylan_db::connection::connect;
use mylan_db::events_repo;

use crate::AppState;

/// Router del WS `/api/v1/events/live`.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/events/live", get(live))
}

#[derive(Deserialize)]
struct LiveQuery {
    /// Cursor ISO8601 para backfill: eventos con `created_at > since` se envían
    /// primero (desde la DB) antes de streamear el live.
    since: Option<String>,
}

/// `GET /api/v1/events/live` — upgrade a WebSocket. El middleware `require_token`
/// (aplicado en `serve`) valida el bearer token en el handshake.
async fn live(
    State(state): State<AppState>,
    Query(q): Query<LiveQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_stream(socket, state, q.since))
}

/// Bucle del WebSocket: backfill desde la DB (si `since`) + stream live del
/// broadcast.
async fn ws_stream(mut socket: WebSocket, state: AppState, since: Option<String>) {
    // 1. Backfill desde la DB si hay cursor `?since=<ISO8601>`.
    if let Some(since) = since.as_deref() {
        if let Ok(conn) = connect(&state.db_path) {
            if let Ok(past) = events_repo::list_events_since(&conn, since) {
                for ev in past {
                    if send_event(&mut socket, &ev).await.is_err() {
                        return; // cliente cerró durante el backfill
                    }
                }
            }
        }
    }

    // 2. Stream live: suscribe al broadcast del agent (ADR-4).
    let mut rx = state.event_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(ev) => {
                if send_event(&mut socket, &ev).await.is_err() {
                    return; // cliente cerró
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                // El cliente se quedó atrás (broadcast capacity superado). Notifica
                // y sigue streameando desde el presente (no desconecta).
                let _ = socket
                    .send(Message::Text(format!("{{\"lagged\":{n}}}").into()))
                    .await;
            }
            Err(broadcast::error::RecvError::Closed) => return, // canal cerrado (agent apagado)
        }
    }
}

/// Envía un `Event` serializado como `Text` por el WebSocket. `Err` si el socket
/// se cerró (caller cierra el bucle).
async fn send_event(socket: &mut WebSocket, ev: &Event) -> Result<(), axum::Error> {
    let json = serde_json::to_string(ev).unwrap_or_else(|_| "null".to_string());
    socket.send(Message::Text(json.into())).await
}
