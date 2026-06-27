//! Test de integración: export de dispositivos (JSON/CSV round-trip + error-path).
//!
//! Verifica que los ficheros exportados son parseables de vuelta y que un path
//! no escribible (permiso denegado) produce un error en vez de un panic.

use std::io::Write;
use std::path::Path;

use mylan_cli::run_scan_pipeline;
use mylan_core::{Device, ScanProfile, Source};

use mylan_integration_tests::{fixture_db, obs, sample_network};

/// Puebla el inventario con un par de dispositivos y devuelve la red activa.
fn populate(dir: &Path) -> rusqlite::Connection {
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
    conn
}

/// JSON exportado es parseable de vuelta con el esquema esperado de `Device`.
#[test]
fn export_json_round_trip() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = populate(dir.path());
    let net_id = sample_network().id;
    let devices = mylan_db::device_repo::list_devices(&conn, &net_id).expect("list");
    assert_eq!(devices.len(), 2);

    let json = serde_json::to_string_pretty(&devices).expect("serialize");
    let path = dir.path().join("mylan-devices.json");
    std::fs::write(&path, json.as_bytes()).expect("write");

    let read_back = std::fs::read_to_string(&path).expect("read");
    let parsed: Vec<Device> = serde_json::from_str(&read_back).expect("deserialize");
    assert_eq!(parsed.len(), 2);
    // El esquema contiene los campos clave (validación estructural).
    assert!(read_back.contains("\"primary_mac\""));
    assert!(read_back.contains("\"device_type\""));
    assert!(read_back.contains("\"confidence\""));
    // Las MACs/IPs reales están presentes (no censuradas en el fichero).
    assert!(read_back.contains("192.168.1.1"));
}

/// CSV exportado es parseable de vuelta con el número esperado de filas.
#[test]
fn export_csv_round_trip() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = populate(dir.path());
    let net_id = sample_network().id;
    let devices = mylan_db::device_repo::list_devices(&conn, &net_id).expect("list");

    let mut buf = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut buf);
        for d in &devices {
            wtr.serialize(d).expect("serialize row");
        }
        wtr.flush().expect("flush");
    }
    let path = dir.path().join("mylan-devices.csv");
    std::fs::write(&path, &buf).expect("write");

    let mut rdr = csv::Reader::from_path(&path).expect("read csv");
    let rows: Vec<Device> = rdr
        .deserialize()
        .collect::<Result<_, _>>()
        .expect("deserialize");
    assert_eq!(rows.len(), 2);
}

/// Escribir en un path no escribible produce un error (no un panic).
#[test]
fn export_to_unwritable_path_errors() {
    let dir = tempfile::tempdir().expect("tmp");
    let conn = populate(dir.path());
    let net_id = sample_network().id;
    let devices = mylan_db::device_repo::list_devices(&conn, &net_id).expect("list");

    // /proc/sys no admite crear ficheros arbitrarios -> error de permiso.
    let bad = Path::new("/proc/sys/kernel/mylan-export.json");
    let json = serde_json::to_string(&devices).expect("serialize");
    let res = std::fs::File::create(bad).and_then(|mut f| f.write_all(json.as_bytes()));
    assert!(
        res.is_err(),
        "esperado error de escritura en path no escribible"
    );
}
