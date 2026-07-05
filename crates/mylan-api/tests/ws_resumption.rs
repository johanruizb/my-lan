//! Integration test WS resumption (AC-6 `?since=<ISO8601>` cursor).
//!
//! El WS client conecta con `?since=<cursor>`: recibe primero los events
//! backfilled desde la DB (`events_repo::list_events_since`) en orden cronológico,
//! luego los events live del broadcast. Verifica ambas fases en un solo test.

use std::sync::Arc;

use axum::middleware::from_fn_with_state;
use axum::Router;
use futures_util::StreamExt;

use mylan_api::{event_channel, require_token, routes, ws, AppState, TokenMiddleware};
use mylan_core::{Event, EventType, Network, Severity};
use mylan_db::connection::connect;
use mylan_db::{events_repo, network_repo};

fn ip(s: &str) -> std::net::IpAddr {
    s.parse().unwrap()
}

fn ev(id: &str, created_at: &str) -> Event {
    Event {
        id: id.to_string(),
        network_id: "net-1".to_string(),
        device_id: None,
        event_type: EventType::DeviceNew,
        severity: Severity::Info,
        message: Some("m".to_string()),
        data_json: None,
        created_at: created_at.to_string(),
    }
}

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

async fn recv_text(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> String {
    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next())
        .await
        .expect("timeout")
        .expect("closed")
        .expect("ws error");
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(t) => t,
        other => panic!("esperaba Text, got {other:?}"),
    }
}

#[tokio::test]
async fn ws_resumption_backfills_then_streams_live() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("wsr.db");
    let conn = connect(&db_path).expect("connect");
    network_repo::upsert_network(
        &conn,
        &Network {
            id: "net-1".to_string(),
            name: "home".to_string(),
            cidr: "192.168.1.0/24".to_string(),
            gateway_ip: Some(ip("192.168.1.1")),
            dns_servers: vec![],
            created_at: "t0".to_string(),
            updated_at: "t0".to_string(),
        },
    )
    .expect("upsert_network");
    // Events pasados en la DB con created_at > cursor (backfill ASC).
    let cursor = "2026-07-03T00:00:00Z";
    events_repo::insert_event(&conn, &ev("past-1", "2026-07-03T00:00:01Z")).expect("ins");
    events_repo::insert_event(&conn, &ev("past-2", "2026-07-03T00:00:02Z")).expect("ins");

    let (addr, event_tx) = start_server(db_path, "secret").await;

    // Conecta con ?since=cursor → backfill + live.
    use tokio_tungstenite::tungstenite::handshake::client::generate_key;
    let path = format!("/api/v1/events/live?since={cursor}");
    let req = axum::http::Request::builder()
        .method("GET")
        .uri(format!("ws://{addr}{path}"))
        .header("Authorization", "Bearer secret")
        .header("Host", addr.to_string())
        .header("Upgrade", "websocket")
        .header("Connection", "upgrade")
        .header("Sec-WebSocket-Key", generate_key())
        .header("Sec-WebSocket-Version", "13")
        .body(())
        .unwrap();
    let (mut ws_stream, _resp) = tokio_tungstenite::connect_async(req)
        .await
        .expect("ws connect");

    // 1. Backfill desde la DB (ASC: past-1, past-2).
    let m1 = recv_text(&mut ws_stream).await;
    assert!(m1.contains("past-1"), "backfill past-1: {m1}");
    let m2 = recv_text(&mut ws_stream).await;
    assert!(m2.contains("past-2"), "backfill past-2: {m2}");

    // 2. Live: emitir un event nuevo y recibirlo.
    // Tras el backfill el server ya suscribió al broadcast; pequeño sleep de
    // seguridad por si el subscribe aún no completó.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    event_tx
        .send(ev("live-1", "2026-07-03T00:00:03Z"))
        .expect("send");
    let m3 = recv_text(&mut ws_stream).await;
    assert!(m3.contains("live-1"), "live event: {m3}");
}
