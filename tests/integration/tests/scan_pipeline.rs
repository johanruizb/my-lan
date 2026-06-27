//! Test de integración: scan → enrichment → persist → devices.
//!
//! Inyecta `Observation`s falsas (no requiere red real) y verifica que el
//! pipeline las agrega, enriquece (heurística router), persiste sin duplicar
//! (AC-12) y deja el inventario consultable.

use std::net::IpAddr;

use mylan_cli::run_scan_pipeline;
use mylan_core::{DeviceType, Observation, ScanProfile, Source};

use mylan_integration_tests::{fixture_db, obs, sample_network};

/// Dos hosts distintos (router + NAS) se persisten y cuentan como vivos/nuevos.
#[tokio::test]
async fn scan_persists_two_hosts_and_counts() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = fixture_db(dir.path()).expect("db");
    let network = sample_network();
    let enricher = mylan_core::noop_enricher();

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

    let outcome = run_scan_pipeline(
        &conn,
        &network,
        &observations,
        &enricher,
        ScanProfile::Quick,
    )
    .expect("pipeline ok");
    assert_eq!(outcome.hosts_alive, 2);
    assert_eq!(outcome.hosts_new, 2);

    let devices = mylan_db::device_repo::list_devices(&conn, &network.id).expect("list");
    assert_eq!(devices.len(), 2, "no duplicados");
}

/// Una segunda corrida sobre los mismos hosts actualiza sin duplicar (AC-12).
#[tokio::test]
async fn second_scan_updates_without_duplicates() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = fixture_db(dir.path()).expect("db");
    let network = sample_network();
    let enricher = mylan_core::noop_enricher();

    let first = vec![obs(
        Source::ArpCache,
        "aa:bb:cc:00:00:10",
        "192.168.1.10",
        Some("laptop"),
    )];
    run_scan_pipeline(&conn, &network, &first, &enricher, ScanProfile::Quick).expect("first");

    // Misma MAC, nueva IP (DHCP), timestamp posterior.
    let second = vec![obs(
        Source::ArpCache,
        "aa:bb:cc:00:00:10",
        "192.168.1.42",
        Some("laptop"),
    )];
    let outcome =
        run_scan_pipeline(&conn, &network, &second, &enricher, ScanProfile::Quick).expect("second");
    assert_eq!(outcome.hosts_alive, 1);
    assert_eq!(outcome.hosts_new, 0, "segunda corrida no añade nuevos");

    let devices = mylan_db::device_repo::list_devices(&conn, &network.id).expect("list");
    assert_eq!(devices.len(), 1, "sigue habiendo un solo dispositivo");
    let ip: IpAddr = "192.168.1.42".parse().unwrap();
    assert_eq!(
        devices[0].primary_ip,
        Some(ip),
        "IP actualizada a la más reciente"
    );
}

/// El fingerprint enriquece y clasifica router por gateway.
#[tokio::test]
async fn enrichment_classifies_router_by_gateway() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = fixture_db(dir.path()).expect("db");
    let network = sample_network();

    // Fingerprint real del repo (signatures/ en la raíz del workspace).
    let signatures = std::path::Path::new("../../signatures");
    let enricher = mylan_fingerprint::Fingerprint::load(signatures, network.gateway_ip)
        .expect("load fingerprint")
        .enricher();

    let observations = vec![obs(
        Source::ArpCache,
        "aa:bb:cc:dd:ee:ff",
        "192.168.1.1",
        Some("router.local"),
    )];

    run_scan_pipeline(
        &conn,
        &network,
        &observations,
        &enricher,
        ScanProfile::Quick,
    )
    .expect("pipeline");

    let devices = mylan_db::device_repo::list_devices(&conn, &network.id).expect("list");
    assert_eq!(devices.len(), 1);
    let router = &devices[0];
    // La heurística de gateway clasifica la IP del gateway como Router@70.
    assert_eq!(router.device_type, DeviceType::Router);
    assert_eq!(router.confidence.score(), 70);
    assert_eq!(router.hostname.as_deref(), Some("router.local"));
}

/// Observations sin identidad utilizable se descartan (no generan dispositivo).
#[tokio::test]
async fn observations_without_identity_are_dropped() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = fixture_db(dir.path()).expect("db");
    let network = sample_network();
    let enricher = mylan_core::noop_enricher();

    // Solo un hint, sin IP ni MAC -> sin identidad -> descartada.
    let observations = vec![Observation::new(Source::Ssdp).with_hint("ssdp.st", "upnp:rootdevice")];

    let outcome = run_scan_pipeline(
        &conn,
        &network,
        &observations,
        &enricher,
        ScanProfile::Quick,
    )
    .expect("pipeline");
    assert_eq!(
        outcome.hosts_alive, 0,
        "observación sin identidad descartada"
    );
    assert!(mylan_db::device_repo::list_devices(&conn, &network.id)
        .expect("list")
        .is_empty());
}
