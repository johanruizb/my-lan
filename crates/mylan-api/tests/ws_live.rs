//! Integration test WS `/events/live` (AC-6: broadcast → WS client, single-process).
//!
//! Servidor real en `127.0.0.1:0` (puerto OS-assigned) + cliente
//! `tokio-tungstenite` con bearer token. El test emite un `Event` al broadcast
//! `event_tx` y afirma que el WS client lo recibe en vivo (ADR-4 — agent y API
//! comparten el canal en un solo proceso).

use std::sync::Arc;

use axum::middleware::from_fn_with_state;
use axum::Router;
use futures_util::StreamExt;

use mylan_api::{event_channel, require_token, routes, ws, AppState, TokenMiddleware};
use mylan_core::{Event, EventType, Severity};
use mylan_db::connection::connect;

fn ev(id: &str, created_at: &str) -> Event {
    Event {
        id: id.to_string(),
        network_id: "net-1".to_string(),
        device_id: Some("dev-1".to_string()),
        event_type: EventType::DeviceNew,
        severity: Severity::Info,
        message: Some("live".to_string()),
        data_json: None,
        created_at: created_at.to_string(),
    }
}

/// Arranca el servidor (routes+ws+middleware+state) en un puerto OS-assigned.
/// Devuelve la dirección y el `Sender` del broadcast para emitir events.
async fn start_server(
    db_path: std::path::PathBuf,
    token: &str,
) -> (std::net::SocketAddr, tokio::sync::broadcast::Sender<Event>) {
    let (event_tx, _rx) = event_channel(16);
    let state = AppState {
        db_path,
        token: Arc::new(token.to_string()),
        event_tx: event_tx.clone(),
    };
    let mw = TokenMiddleware::new(state.token.clone());
    let app = Router::new()
        .merge(routes::router())
        .merge(ws::router())
        .layer(from_fn_with_state(mw, require_token))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    // axum::serve retorna `Serve` (IntoFuture, no Future); envolver en async move
    // para que tokio::spawn reciba un Future real.
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (addr, event_tx)
}

/// WS request con handshake válido + bearer token opcional. `connect_async` con
/// un `http::Request` custom NO auto-genera los headers WS (Sec-WebSocket-Key,
/// Upgrade, etc.) — hay que añadirlos manualmente.
fn ws_request(
    addr: &std::net::SocketAddr,
    token: Option<&str>,
    path: &str,
) -> axum::http::Request<()> {
    use tokio_tungstenite::tungstenite::handshake::client::generate_key;
    let mut builder = axum::http::Request::builder()
        .method("GET")
        .uri(format!("ws://{addr}{path}"))
        .header("Host", addr.to_string())
        .header("Upgrade", "websocket")
        .header("Connection", "upgrade")
        .header("Sec-WebSocket-Key", generate_key())
        .header("Sec-WebSocket-Version", "13");
    if let Some(t) = token {
        builder = builder.header("Authorization", format!("Bearer {t}"));
    }
    builder.body(()).unwrap()
}

#[tokio::test]
async fn ws_live_receives_broadcast_event() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("ws.db");
    let _conn = connect(&db_path).expect("connect");
    let (addr, event_tx) = start_server(db_path, "secret").await;

    let (mut ws_stream, _resp) =
        tokio_tungstenite::connect_async(ws_request(&addr, Some("secret"), "/api/v1/events/live"))
            .await
            .expect("ws connect");
    // Pequeño sleep para que el server suscriba al broadcast tras el handshake.
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    event_tx
        .send(ev("evt-live-1", "2026-07-03T00:00:00Z"))
        .expect("send");

    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws_stream.next())
        .await
        .expect("timeout")
        .expect("closed")
        .expect("ws error");
    let text = match msg {
        tokio_tungstenite::tungstenite::Message::Text(t) => t,
        other => panic!("esperaba Text, got {other:?}"),
    };
    assert!(
        text.contains("evt-live-1"),
        "WS recibió el event del broadcast: {text}"
    );
}

#[tokio::test]
async fn ws_live_401_without_token() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("ws401.db");
    let _conn = connect(&db_path).expect("connect");
    let (addr, _event_tx) = start_server(db_path, "secret").await;

    // WS request válida (handshake completo) SIN token → el middleware devuelve
    // 401 antes del upgrade → connect_async falla (no 101).
    let req = ws_request(&addr, None, "/api/v1/events/live");
    let res = tokio_tungstenite::connect_async(req).await;
    assert!(
        res.is_err(),
        "WS sin token debe rechazarse (401, no upgrade): {res:?}"
    );
}
