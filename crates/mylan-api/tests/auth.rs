//! Test del middleware de token (AC-5: 401 sin/invalid token, 200 con token válido).
//!
//! Ejercicio del `TokenMiddleware` + `require_token` como tower layer sobre un
//! router dummy, vía `tower::ServiceExt::oneshot`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

use mylan_api::{require_token, TokenMiddleware};

async fn ok_handler() -> &'static str {
    "ok"
}

fn app(token: &str) -> Router<()> {
    let mw = TokenMiddleware::new(Arc::new(token.to_string()));
    Router::new()
        .route("/ping", get(ok_handler))
        .layer(from_fn_with_state(mw, require_token))
}

#[tokio::test]
async fn rejects_without_token() {
    let app = app("secret");
    let res = app
        .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_wrong_token() {
    let app = app("secret");
    let res = app
        .oneshot(
            Request::builder()
                .uri("/ping")
                .header(header::AUTHORIZATION, "Bearer wrong")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_malformed_header_without_bearer_prefix() {
    let app = app("secret");
    // Sin prefijo "Bearer ": no sigue el formato esperado.
    let res = app
        .oneshot(
            Request::builder()
                .uri("/ping")
                .header(header::AUTHORIZATION, "secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn accepts_valid_token() {
    let app = app("secret");
    let res = app
        .oneshot(
            Request::builder()
                .uri("/ping")
                .header(header::AUTHORIZATION, "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), 64).await.unwrap();
    assert_eq!(&body[..], b"ok");
}
