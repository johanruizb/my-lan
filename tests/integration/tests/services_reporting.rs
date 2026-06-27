//! Test de integración: reporte/export de servicios (Fase 3, Paso 6).
//!
//! Verifica que `mylan export services` produce CSV con columnas exactas
//! (parseable con `csv::Reader`) y JSON con las keys de `ServiceExportRow`,
//! ambos consistentes y acotados a la red activa.

use std::path::Path;

use mylan_cli::commands::export::{export_services, ExportFormat};
use mylan_cli::commands::services::{run_export_services, run_services};
use mylan_cli::ctx::AppContext;
use mylan_cli::run_scan_pipeline;
use mylan_core::{Protocol, ScanProfile, Service, ServiceState, Source};

use mylan_integration_tests::{fixture_db, obs, sample_network};

/// Puebla el inventario (2 dispositivos vía pipeline) y registra `tcp/80` en cada uno.
fn populate_with_services(dir: &Path) -> rusqlite::Connection {
    let conn = fixture_db(dir).expect("db");
    let network = sample_network();
    let observations = vec![
        obs(
            Source::ArpCache,
            "aa:bb:cc:00:00:01",
            "192.168.1.1",
            Some("router.local"),
        ),
        obs(
            Source::ArpCache,
            "aa:bb:cc:00:00:02",
            "192.168.1.5",
            Some("nas.local"),
        ),
    ];
    run_scan_pipeline(
        &conn,
        &network,
        &observations,
        &mylan_core::noop_enricher(),
        ScanProfile::Quick,
    )
    .expect("pipeline");

    let net_id = network.id;
    let devices = mylan_db::device_repo::list_devices(&conn, &net_id).expect("list devices");
    assert_eq!(devices.len(), 2);
    for d in &devices {
        let svc = Service {
            id: format!("{}-tcp-80", d.id),
            device_id: d.id.clone(),
            protocol: Protocol::Tcp,
            port: 80,
            service_name: Some("http".to_string()),
            product: Some("nginx".to_string()),
            version: Some("1.25".to_string()),
            banner: Some("HTTP/1.1 200 OK".to_string()),
            state: ServiceState::Open,
            first_seen_at: "2026-06-27T00:00:00Z".to_string(),
            last_seen_at: "2026-06-27T00:00:10Z".to_string(),
        };
        mylan_db::service_repo::upsert_service(&conn, &svc).expect("upsert service");
    }
    conn
}

/// Columnas exactas del CSV de servicios (orden contractual).
const EXPECTED_COLUMNS: [&str; 12] = [
    "device_id",
    "device_ip",
    "display_name",
    "protocol",
    "port",
    "service_name",
    "product",
    "version",
    "banner",
    "state",
    "first_seen_at",
    "last_seen_at",
];

/// CSV exportado tiene las 12 columnas exactas (header) y una fila por servicio.
#[test]
fn export_services_csv_has_exact_columns_and_rows() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = populate_with_services(dir.path());

    let csv_path = dir.path().join("mylan-services.csv");
    export_services(&conn, ExportFormat::Csv, csv_path.to_str()).expect("export csv");

    let mut rdr = csv::Reader::from_path(&csv_path).expect("read csv");
    let header: Vec<String> = rdr
        .headers()
        .expect("header")
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, EXPECTED_COLUMNS);

    let rows: Vec<csv::StringRecord> = rdr.records().collect::<Result<_, _>>().expect("read rows");
    assert_eq!(rows.len(), 2, "un servicio por dispositivo");
    for r in &rows {
        assert_eq!(r.len(), 12, "12 columnas por fila");
        assert_eq!(r.get(3), Some("tcp"), "columna protocol = tcp");
        assert_eq!(r.get(4), Some("80"), "columna port = 80");
        assert_eq!(r.get(9), Some("open"), "columna state = open");
    }
    // device_ip proviene del join con devices (primary_ip).
    let ips: Vec<&str> = rows.iter().map(|r| r.get(1).unwrap()).collect();
    assert!(ips.contains(&"192.168.1.1"));
    assert!(ips.contains(&"192.168.1.5"));
}

/// JSON exportado es un array parseable con las 12 keys de `ServiceExportRow`.
#[test]
fn export_services_json_has_expected_keys() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = populate_with_services(dir.path());

    let json_path = dir.path().join("mylan-services.json");
    export_services(&conn, ExportFormat::Json, json_path.to_str()).expect("export json");

    let content = std::fs::read_to_string(&json_path).expect("read json");
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).expect("parse json array");
    assert_eq!(parsed.len(), 2);
    for v in &parsed {
        let obj = v.as_object().expect("object");
        for key in EXPECTED_COLUMNS {
            assert!(obj.contains_key(key), "falta la key {key}");
        }
        assert_eq!(obj.get("protocol").and_then(|p| p.as_str()), Some("tcp"));
        assert_eq!(obj.get("port").and_then(|p| p.as_u64()), Some(80u64));
        assert_eq!(obj.get("state").and_then(|p| p.as_str()), Some("open"));
    }
}

/// Sin servicios en el inventario: exporta cero sin errorar ni escribir fichero.
#[test]
fn export_services_empty_when_no_services() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = fixture_db(dir.path()).expect("db");
    let csv_path = dir.path().join("empty-services.csv");
    export_services(&conn, ExportFormat::Csv, csv_path.to_str()).expect("export ok");
    assert!(
        !csv_path.exists(),
        "no se escribe fichero cuando no hay servicios"
    );
}

/// Smoke test end-to-end de los entrypoints reales del CLI (`run_services` +
/// `run_export_services`) vía `AppContext` con DB temporal: sin panic, ficheros
/// generados, y filtros `--device`/`--protocol` aceptados.
#[test]
fn run_services_and_export_via_app_context() {
    let dir = tempfile::tempdir().expect("tmp");
    let db_path = dir.path().join("mylan.db");

    // Puebla red + dispositivo (pipeline registra scan) + un servicio tcp/80.
    let conn = mylan_db::connection::connect(&db_path).expect("connect");
    let network = sample_network();
    let observations = vec![obs(
        Source::ArpCache,
        "aa:bb:cc:00:00:01",
        "192.168.1.1",
        Some("router.local"),
    )];
    run_scan_pipeline(
        &conn,
        &network,
        &observations,
        &mylan_core::noop_enricher(),
        ScanProfile::Quick,
    )
    .expect("pipeline");
    let devices = mylan_db::device_repo::list_devices(&conn, &network.id).expect("list");
    let d = devices.first().expect("device");
    let svc = Service {
        id: format!("{}-tcp-80", d.id),
        device_id: d.id.clone(),
        protocol: Protocol::Tcp,
        port: 80,
        service_name: Some("http".to_string()),
        product: Some("nginx".to_string()),
        version: None,
        banner: Some("HTTP/1.1 200 OK".to_string()),
        state: ServiceState::Open,
        first_seen_at: "2026-06-27T00:00:00Z".to_string(),
        last_seen_at: "2026-06-27T00:00:10Z".to_string(),
    };
    mylan_db::service_repo::upsert_service(&conn, &svc).expect("upsert");
    drop(conn);

    let ctx = AppContext {
        db_path: db_path.clone(),
        signatures_dir: dir.path().join("signatures"),
        verbose: false,
    };

    // `mylan services` sin filtros y con --device <ip> --protocol tcp.
    run_services(&ctx, None, None, None, None).expect("services list");
    run_services(&ctx, Some("192.168.1.1"), None, Some("tcp"), None).expect("services filtered");

    // `mylan export services --format csv|json` escribe ficheros.
    let csv_path = dir.path().join("out.csv");
    run_export_services(&ctx, ExportFormat::Csv, csv_path.to_str()).expect("export csv");
    assert!(csv_path.exists(), "CSV escrito");
    let json_path = dir.path().join("out.json");
    run_export_services(&ctx, ExportFormat::Json, json_path.to_str()).expect("export json");
    assert!(json_path.exists(), "JSON escrito");
}
