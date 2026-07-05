//! Integration tests REST (AC-5: 8 endpoints 200 con token válido, 401 sin token).
//!
//! Ejercicio del router completo (`routes` + `ws` + middleware `require_token` +
//! `AppState`) vía `tower::ServiceExt::oneshot` (sin TCP para los REST). DB en
//! tempdir con una red + un device para que `GET /devices/:id` devuelva 200.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::Router;
use tower::ServiceExt;

use mylan_api::{event_channel, require_token, routes, ws, AppState, TokenMiddleware};
use mylan_core::{Device, MacAddr, Network};
use mylan_db::connection::connect;
use mylan_db::{device_repo, network_repo};

fn ip(s: &str) -> std::net::IpAddr {
    s.parse().unwrap()
}

fn mac(s: &str) -> MacAddr {
    MacAddr::parse(s).expect("valid mac")
}

/// DB en tempdir con red `net-1` + device `dev-1` (para GET /devices/:id).
fn fixture_db(dir: &std::path::Path) -> std::path::PathBuf {
    let db_path = dir.join("api.db");
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
    let mut d = Device::new("dev-1", "net-1", "t0");
    d.primary_mac = Some(mac("aa:bb:cc:dd:ee:ff"));
    d.primary_ip = Some(ip("192.168.1.5"));
    device_repo::upsert_device(&conn, &d).expect("upsert_device");
    db_path
}

/// App completa (routes + ws + middleware + state), lista para `oneshot`.
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

/// Status code para una petición (con token opcional) al `app`.
async fn status_for(
    app: &Router<()>,
    token: Option<&str>,
    method: &str,
    uri: &str,
    body: &str,
) -> StatusCode {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    // `Json<PostScanBody>` requiere `Content-Type: application/json` (si no → 415).
    // Inofensivo para GET (sin body); necesario para POST /scans.
    builder = builder.header(header::CONTENT_TYPE, "application/json");
    let req = builder.body(Body::from(body.to_string())).unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn all_endpoints_401_without_token() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(fixture_db(dir.path()), "secret");
    for (method, uri, body) in [
        ("GET", "/api/v1/status", ""),
        ("GET", "/api/v1/interfaces", ""),
        ("GET", "/api/v1/networks", ""),
        ("GET", "/api/v1/devices", ""),
        ("GET", "/api/v1/devices/dev-1", ""),
        ("GET", "/api/v1/events", ""),
        ("GET", "/api/v1/scans", ""),
        (
            "POST",
            "/api/v1/scans",
            r#"{"network_id":null,"profile":"quick"}"#,
        ),
    ] {
        assert_eq!(
            status_for(&app, None, method, uri, body).await,
            StatusCode::UNAUTHORIZED,
            "{method} {uri} sin token debe ser 401"
        );
    }
}

#[tokio::test]
async fn db_endpoints_200_with_valid_token() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(fixture_db(dir.path()), "secret");
    // Endpoints DB-only (sin I/O de red): 200 con token.
    for (method, uri, body) in [
        ("GET", "/api/v1/status", ""),
        ("GET", "/api/v1/networks", ""),
        ("GET", "/api/v1/devices", ""),
        ("GET", "/api/v1/devices/dev-1", ""),
        ("GET", "/api/v1/events", ""),
        ("GET", "/api/v1/scans", ""),
    ] {
        assert_eq!(
            status_for(&app, Some("secret"), method, uri, body).await,
            StatusCode::OK,
            "{method} {uri} con token debe ser 200"
        );
    }
}

#[tokio::test]
async fn devices_by_unknown_id_returns_404_with_token() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(fixture_db(dir.path()), "secret");
    assert_eq!(
        status_for(&app, Some("secret"), "GET", "/api/v1/devices/nope", "").await,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn devices_filter_by_network_returns_200() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(fixture_db(dir.path()), "secret");
    assert_eq!(
        status_for(
            &app,
            Some("secret"),
            "GET",
            "/api/v1/devices?network_id=net-1",
            ""
        )
        .await,
        StatusCode::OK
    );
    // Filtro de red inexistente: 200 con [] (no 404 — el filtro no es un recurso).
    assert_eq!(
        status_for(
            &app,
            Some("secret"),
            "GET",
            "/api/v1/devices?network_id=nope",
            ""
        )
        .await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn wrong_token_401() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(fixture_db(dir.path()), "secret");
    assert_eq!(
        status_for(&app, Some("wrong"), "GET", "/api/v1/status", "").await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn post_scan_200_with_token_or_graceful() {
    // POST /scans corre discovery real (detect_interface + discover). En CI
    // (GitHub Linux runner con default route) detect_interface suele succeed y
    // discover devuelve [] (sin hosts) → pipeline 0 hosts → 200. Aceptamos 200;
    // si el entorno no tiene interfaz por defecto, degradaría a 500 (no 401).
    let dir = tempfile::tempdir().unwrap();
    let app = app(fixture_db(dir.path()), "secret");
    let status = status_for(
        &app,
        Some("secret"),
        "POST",
        "/api/v1/scans",
        r#"{"network_id":null,"profile":"quick"}"#,
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::INTERNAL_SERVER_ERROR,
        "POST /scans con token: 200 (discovery ok) o 500 (sin interfaz), got {status}"
    );
    assert_ne!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_interfaces_200_with_token_or_graceful() {
    // GET /interfaces usa detect_interface (netdev). 200 si hay default route,
    // 500 si el entorno no tiene interfaz utilizable. Nunca 401 con token.
    let dir = tempfile::tempdir().unwrap();
    let app = app(fixture_db(dir.path()), "secret");
    let status = status_for(&app, Some("secret"), "GET", "/api/v1/interfaces", "").await;
    assert!(
        status == StatusCode::OK || status == StatusCode::INTERNAL_SERVER_ERROR,
        "GET /interfaces con token: 200 o 500 (sin interfaz), got {status}"
    );
    assert_ne!(status, StatusCode::UNAUTHORIZED);
}
