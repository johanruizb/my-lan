//! Handlers REST de `mylan-api` (AC-5: 8 endpoints + token auth + 127.0.0.1).
//!
//! Cada handler abre una `Connection` fresca sobre `state.db_path` (migraciones
//! idempotentes + FKs on) y devuelve `Json`/`ApiError`. El middleware
//! `require_token` (aplicado en `serve`) cubre `/api/v1/*` → 401 sin token válido;
//! los handlers no re-validan auth. `POST /scans` corre discovery IN-PROCESS
//! (`mylan-discovery` + `mylan-fingerprint` → `run_scan_pipeline_with_diff`) y
//! hace fan-out de los events al broadcast `event_tx` (clientes WS, ADR-4).

use std::net::IpAddr;
use std::path::PathBuf;

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};

use mylan_core::{Device, Enricher, Event, Network, ScanProfile};
use mylan_db::connection::connect;
use mylan_db::pipeline::run_scan_pipeline_with_diff;
use mylan_db::{device_repo, events_repo, network_repo, scan_repo, DbResult};
use mylan_discovery::{detect_interface, discover, DiscoverOptions};

use crate::{ApiError, AppState};

/// Router de los 8 endpoints REST bajo `/api/v1/*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/status", get(status))
        .route("/api/v1/interfaces", get(interfaces))
        .route("/api/v1/networks", get(networks))
        .route("/api/v1/devices", get(devices))
        .route("/api/v1/devices/{id}", get(device_by_id))
        .route("/api/v1/events", get(events))
        .route("/api/v1/scans", get(scans).post(post_scan))
}

/// Mapea un `DbResult` a `Result<_, ApiError>` (error interno 500 con el mensaje).
fn db_err<T>(r: DbResult<T>) -> Result<T, ApiError> {
    r.map_err(|e| ApiError::Internal(format!("db: {e}")))
}

/// `GET /api/v1/status` — salud del agent + DB (abre conexión, aplica migraciones,
/// lee `user_version`).
async fn status(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let conn = db_err(connect(&state.db_path))?;
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| ApiError::Internal(format!("user_version: {e}")))?;
    Ok(Json(json!({
        "status": "ok",
        "db_path": state.db_path.display().to_string(),
        "schema_version": version,
    })))
}

/// `GET /api/v1/interfaces` — interfaz LAN por defecto (`mylan-discovery`).
async fn interfaces(State(_): State<AppState>) -> Result<Json<Value>, ApiError> {
    let iface =
        detect_interface(None).map_err(|e| ApiError::Internal(format!("detect_interface: {e}")))?;
    Ok(Json(json!({
        "name": iface.name,
        "ip": iface.ip.to_string(),
        "cidr": iface.cidr(),
        "prefix_len": iface.prefix_len,
        "mac": iface.mac.map(|m| m.to_string()),
        "gateway_ip": iface.gateway_ip.map(|ip| ip.to_string()),
        "gateway_mac": iface.gateway_mac.map(|m| m.to_string()),
        "dns_servers": iface.dns_servers.iter().map(|ip| ip.to_string()).collect::<Vec<_>>(),
        "ssid": iface.ssid,
    })))
}

/// `GET /api/v1/networks` — todas las redes (`network_repo::list_networks`).
async fn networks(State(state): State<AppState>) -> Result<Json<Vec<Network>>, ApiError> {
    let conn = db_err(connect(&state.db_path))?;
    let nets = db_err(network_repo::list_networks(&conn))?;
    Ok(Json(nets))
}

#[derive(Deserialize)]
struct DevicesQuery {
    network_id: Option<String>,
}

/// `GET /api/v1/devices` — devices de una red (`?network_id=`) o de todas las
/// redes si el filtro se omite. Incluye `is_online` (Step 1).
async fn devices(
    State(state): State<AppState>,
    Query(q): Query<DevicesQuery>,
) -> Result<Json<Vec<Device>>, ApiError> {
    let conn = db_err(connect(&state.db_path))?;
    if let Some(nid) = q.network_id.as_deref() {
        let devs = db_err(device_repo::list_devices(&conn, nid))?;
        Ok(Json(devs))
    } else {
        let nets = db_err(network_repo::list_networks(&conn))?;
        let mut all = Vec::new();
        for net in nets {
            all.extend(db_err(device_repo::list_devices(&conn, &net.id))?);
        }
        Ok(Json(all))
    }
}

/// `GET /api/v1/devices/{id}` — un device por `id` (404 si no existe).
async fn device_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Device>, ApiError> {
    let conn = db_err(connect(&state.db_path))?;
    match db_err(device_repo::get_device(&conn, &id))? {
        Some(d) => Ok(Json(d)),
        None => Err(ApiError::NotFound(format!("device {id}"))),
    }
}

#[derive(Deserialize)]
struct EventsQuery {
    network_id: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

/// `GET /api/v1/events` — timeline de events ordenado por `created_at`
/// (`events_repo::list_events`); `?network_id=`, `?limit=`, `?offset=` opcionales.
async fn events(
    State(state): State<AppState>,
    Query(q): Query<EventsQuery>,
) -> Result<Json<Vec<Event>>, ApiError> {
    let conn = db_err(connect(&state.db_path))?;
    let limit = q.limit.unwrap_or(100);
    let offset = q.offset.unwrap_or(0);
    let evs = db_err(events_repo::list_events(
        &conn,
        q.network_id.as_deref(),
        limit,
        offset,
    ))?;
    Ok(Json(evs))
}

/// `GET /api/v1/scans` — historial de escaneos (`scan_repo::list_scans`). Mapea
/// `ScanRow` a JSON (no deriva `Serialize`).
async fn scans(State(state): State<AppState>) -> Result<Json<Vec<Value>>, ApiError> {
    let conn = db_err(connect(&state.db_path))?;
    let rows = db_err(scan_repo::list_scans(&conn))?;
    let out = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "profile": r.profile,
                "status": r.status,
                "started_at": r.started_at,
                "finished_at": r.finished_at,
                "hosts_alive": r.hosts_alive,
                "hosts_new": r.hosts_new,
                "scan_type": r.scan_type,
                "target_ip": r.target_ip,
                "open_ports": r.open_ports,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(out))
}

#[derive(Deserialize)]
struct PostScanBody {
    network_id: Option<String>,
    profile: Option<String>,
}

/// `POST /api/v1/scans` — dispara un escaneo IN-PROCESS: `detect_interface` →
/// `discover` → `run_scan_pipeline_with_diff` en una `Connection` fresca. Retorna
/// `ScanOutcome` + events emitidos, y hace fan-out al broadcast `event_tx` (WS).
///
/// `network_id` null → construye la red desde la interfaz detectada (como el CLI
/// `mylan scan`). `profile` null → `Quick`. `cold_start=false` (trigger manual,
/// emite todos los events).
async fn post_scan(
    State(state): State<AppState>,
    Json(body): Json<PostScanBody>,
) -> Result<Json<Value>, ApiError> {
    let profile = match body.profile.as_deref() {
        Some(p) => parse_profile(p)?,
        None => ScanProfile::Quick,
    };

    let iface =
        detect_interface(None).map_err(|e| ApiError::Internal(format!("detect_interface: {e}")))?;
    let opts = DiscoverOptions::for_profile(profile);
    let observations = discover(&iface, &opts).await;

    let now = mylan_db::util::now_rfc3339()
        .map_err(|e| ApiError::Internal(format!("now_rfc3339: {e}")))?;
    let conn = db_err(connect(&state.db_path))?;

    // Red: si body.network_id provisto y existe en DB, la reusa; si no, construye
    // desde la interfaz (cidr/gateway/dns) — igual que `mylan scan`.
    let network = if let Some(nid) = body.network_id.as_deref() {
        db_err(network_repo::get_network(&conn, nid))?
            .ok_or_else(|| ApiError::NotFound(format!("network {nid}")))?
    } else {
        Network {
            id: iface.cidr(),
            name: iface.cidr(),
            cidr: iface.cidr(),
            gateway_ip: iface.gateway_ip,
            dns_servers: iface.dns_servers.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    };

    let enricher = build_enricher(iface.gateway_ip);
    let (outcome, events) = run_scan_pipeline_with_diff(
        &conn,
        &network,
        &observations,
        &enricher,
        profile,
        &now,
        false,
    )
    .map_err(|e| ApiError::Internal(format!("pipeline: {e}")))?;

    // Fan-out al broadcast: los clientes WS conectados reciben los events en vivo.
    for ev in &events {
        let _ = state.event_tx.send(ev.clone());
    }

    Ok(Json(json!({
        "scan": {
            "scan_id": outcome.scan_id,
            "network_id": outcome.network_id,
            "hosts_alive": outcome.hosts_alive,
            "hosts_new": outcome.hosts_new,
            "duration_ms": outcome.duration_ms,
        },
        "events": events,
    })))
}

/// Parsea un `ScanProfile` desde su nombre `snake_case` (body de `POST /scans`).
fn parse_profile(s: &str) -> Result<ScanProfile, ApiError> {
    match s.to_lowercase().as_str() {
        "quick" => Ok(ScanProfile::Quick),
        "normal" => Ok(ScanProfile::Normal),
        "deep" => Ok(ScanProfile::Deep),
        "iot" => Ok(ScanProfile::Iot),
        "router" => Ok(ScanProfile::Router),
        other => Err(ApiError::BadRequest(format!("unknown profile: {other}"))),
    }
}

/// Construye el `Enricher` de fingerprint, degradando a no-op si falla la carga
/// de signatures (mismo patrón que `apps/cli`).
fn build_enricher(gateway_ip: Option<IpAddr>) -> Enricher {
    let signatures_dir = default_signatures_dir();
    match mylan_fingerprint::Fingerprint::load(&signatures_dir, gateway_ip) {
        Ok(fp) => fp.enricher(),
        Err(e) => {
            tracing::warn!(error = %e, "fingerprint no cargado; enrichment no-op");
            mylan_core::noop_enricher()
        }
    }
}

/// Directorio de signatures: env `MYLAN_SIGNATURES_DIR` o `./signatures` (relativo
/// al CWD). Igual que `apps/cli::ctx::default_signatures_dir`.
fn default_signatures_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MYLAN_SIGNATURES_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    PathBuf::from("signatures")
}
