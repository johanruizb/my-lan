//! Integration test `GET /api/v1/events` (AC-4: timeline ordenado por
//! `created_at`). Verifica orden DESC (más reciente primero) + filtro
//! `?network_id=` + cuerpo JSON parseable.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::Router;
use tower::ServiceExt;

use mylan_api::{event_channel, require_token, routes, ws, AppState, TokenMiddleware};
use mylan_core::{Event, EventType, Network, Severity};
use mylan_db::connection::connect;
use mylan_db::{events_repo, network_repo};

fn ip(s: &str) -> std::net::IpAddr {
    s.parse().unwrap()
}

fn ev(id: &str, net: &str, created_at: &str) -> Event {
    Event {
        id: id.to_string(),
        network_id: net.to_string(),
        device_id: None,
        event_type: EventType::DeviceNew,
        severity: Severity::Info,
        message: Some("m".to_string()),
        data_json: None,
        created_at: created_at.to_string(),
    }
}

fn fixture_db(dir: &std::path::Path) -> std::path::PathBuf {
    let db_path = dir.join("ee.db");
    let conn = connect(&db_path).expect("connect");
    for net in ["net-1", "net-2"] {
        network_repo::upsert_network(
            &conn,
            &Network {
                id: net.to_string(),
                name: net.to_string(),
                cidr: "192.168.1.0/24".to_string(),
                gateway_ip: Some(ip("192.168.1.1")),
                dns_servers: vec![],
                created_at: "t0".to_string(),
                updated_at: "t0".to_string(),
            },
        )
        .expect("upsert_network");
    }
    // 3 events en net-1 con created_at creciente + 1 en net-2.
    events_repo::insert_event(&conn, &ev("e0", "net-1", "2026-07-03T00:00:00Z")).expect("ins");
    events_repo::insert_event(&conn, &ev("e1", "net-1", "2026-07-03T00:00:01Z")).expect("ins");
    events_repo::insert_event(&conn, &ev("e2", "net-1", "2026-07-03T00:00:02Z")).expect("ins");
    events_repo::insert_event(&conn, &ev("x1", "net-2", "2026-07-03T00:00:05Z")).expect("ins");
    db_path
}

fn app(db_path: std::path::PathBuf, token: &str) -> Router<()> {
    let (event_tx, _rx) = event_channel(16);
    let state = AppState {
        db_path,
        token: Arc::new(token.to_string()),
        event_tx,
    };
    let mw = TokenMiddleware::new(state.token.clone());
    Router::new()
        .merge(routes::router())
        .merge(ws::router())
        .layer(from_fn_with_state(mw, require_token))
        .with_state(state)
}

async fn get_events(app: Router<()>, token: &str, uri: &str) -> serde_json::Value {
    let req = Request::builder()
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "GET {uri}");
    let body = to_bytes(res.into_body(), 1 << 20).await.unwrap();
    serde_json::from_slice(&body).expect("json")
}

#[tokio::test]
async fn events_ordered_by_created_at_desc() {
    let dir = tempfile::tempdir().unwrap();
    let json = get_events(
        app(fixture_db(dir.path()), "secret"),
        "secret",
        "/api/v1/events",
    )
    .await;
    let arr = json.as_array().expect("array");
    assert_eq!(arr.len(), 4, "4 events totales");
    // DESC por created_at: e2(00:02) > e1(00:01) > e0(00:00) ... pero x1(00:05) es el mayor.
    let ids: Vec<&str> = arr.iter().map(|e| e["id"].as_str().expect("id")).collect();
    assert_eq!(ids, ["x1", "e2", "e1", "e0"], "DESC por created_at");
}

#[tokio::test]
async fn events_filter_by_network() {
    let dir = tempfile::tempdir().unwrap();
    let json = get_events(
        app(fixture_db(dir.path()), "secret"),
        "secret",
        "/api/v1/events?network_id=net-1",
    )
    .await;
    let arr = json.as_array().expect("array");
    assert_eq!(arr.len(), 3, "solo events de net-1");
    let ids: Vec<&str> = arr.iter().map(|e| e["id"].as_str().expect("id")).collect();
    assert_eq!(ids, ["e2", "e1", "e0"], "filtro net-1, DESC");
}

#[tokio::test]
async fn events_401_without_token() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(fixture_db(dir.path()), "secret");
    let req = Request::builder()
        .uri("/api/v1/events")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn events_limit_offset_pagination() {
    let dir = tempfile::tempdir().unwrap();
    let json = get_events(
        app(fixture_db(dir.path()), "secret"),
        "secret",
        "/api/v1/events?limit=2&offset=0",
    )
    .await;
    let arr = json.as_array().expect("array");
    assert_eq!(arr.len(), 2, "limit=2");
    let ids: Vec<&str> = arr.iter().map(|e| e["id"].as_str().expect("id")).collect();
    assert_eq!(ids, ["x1", "e2"], "página 1 DESC");
}
