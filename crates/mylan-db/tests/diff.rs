//! Integration tests del motor de diff (AC-3: 5 event types + cold-start).
//!
//! Cubre [`mylan_db::diff::run_diff`] con synthetic before/after device+service
//! sets: `device_new`, `device_offline`, `device_online`, `device_ip_changed`,
//! `port_opened` + supresión cold-start. Black-box vía API pública de `mylan-db`;
//! `mylan-core` entra como dev-dep para construir `Device`/`Network`/`Service`.

use std::net::IpAddr;

use mylan_core::{Device, EventType, MacAddr, Network, Protocol, Service, ServiceState};

use mylan_db::{
    connection::connect,
    device_repo::{list_devices, upsert_device},
    diff::{run_diff, snapshot_devices_before, snapshot_services_before},
    network_repo, service_repo,
};

fn ip(s: &str) -> IpAddr {
    s.parse().expect("valid ip")
}

fn mac(s: &str) -> MacAddr {
    MacAddr::parse(s).expect("valid mac")
}

/// Conexión con red `net-1` insertada (FK de devices/services).
fn fixture_conn(dir: &std::path::Path, name: &str) -> rusqlite::Connection {
    let conn = connect(dir.join(format!("{name}.db"))).expect("connect");
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
    conn
}

/// Device con MAC única (para upsert por identidad), `is_online` y `last_seen`
/// controlables. `first_seen_at` = `last_seen` (Device::new).
fn dev(
    id: &str,
    mac_addr: MacAddr,
    ip_addr: Option<IpAddr>,
    is_online: bool,
    last_seen: &str,
) -> Device {
    let mut d = Device::new(id, "net-1", last_seen);
    d.primary_mac = Some(mac_addr);
    d.primary_ip = ip_addr;
    d.is_online = is_online;
    d
}

/// Device "visto este scan" (is_online=true por defecto, last_seen=scan_now).
fn scan_dev(id: &str, mac_addr: MacAddr, ip_addr: Option<IpAddr>, scan_now: &str) -> Device {
    let mut d = Device::new(id, "net-1", scan_now);
    d.primary_mac = Some(mac_addr);
    d.primary_ip = ip_addr;
    d
}

fn svc(device_id: &str, protocol: Protocol, port: u16, name: Option<&str>) -> Service {
    Service {
        id: format!("svc-{device_id}-{port}"),
        device_id: device_id.to_string(),
        protocol,
        port,
        service_name: name.map(str::to_string),
        product: None,
        version: None,
        banner: None,
        state: ServiceState::Open,
        first_seen_at: "t1".to_string(),
        last_seen_at: "t1".to_string(),
    }
}

/// `is_online` de un device por id (leído dentro de la txn via &tx).
fn is_online_of(conn: &rusqlite::Connection, id: &str) -> bool {
    list_devices(conn, "net-1")
        .expect("list_devices")
        .iter()
        .find(|d| d.id == id)
        .map(|d| d.is_online)
        .expect("device found")
}

/// Snapshots before + apertura de txn. Devuelve (tx, before_devices, before_services).
fn before_snapshot(
    tx: &rusqlite::Transaction<'_>,
) -> (
    Vec<mylan_db::diff::DeviceSnapshot>,
    std::collections::HashMap<String, Vec<mylan_db::diff::ServiceSnapshot>>,
) {
    let before_devices = snapshot_devices_before(tx, "net-1").expect("snapshot_devices_before");
    let before_ids: Vec<String> = before_devices.iter().map(|d| d.id.clone()).collect();
    let before_services =
        snapshot_services_before(tx, &before_ids).expect("snapshot_services_before");
    (before_devices, before_services)
}

#[test]
fn device_new_emitted_for_fresh_device() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "new");
    // Before: 1 device ya conocido (is_online=true, last_seen="t0").
    upsert_device(
        &conn,
        &dev(
            "dev-old",
            mac("aa:bb:cc:dd:ee:01"),
            Some(ip("192.168.1.10")),
            true,
            "t0",
        ),
    )
    .unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    let (before, before_svc) = before_snapshot(&tx);
    // Scan: re-escanea dev-old (last_seen="t1") + inserta dev-new (fresh).
    upsert_device(
        &tx,
        &scan_dev(
            "dev-old",
            mac("aa:bb:cc:dd:ee:01"),
            Some(ip("192.168.1.10")),
            "t1",
        ),
    )
    .unwrap();
    upsert_device(
        &tx,
        &scan_dev(
            "dev-new",
            mac("aa:bb:cc:dd:ee:99"),
            Some(ip("192.168.1.99")),
            "t1",
        ),
    )
    .unwrap();

    let events = run_diff(&tx, "net-1", "t1", before, before_svc, false).unwrap();
    let new_events: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventType::DeviceNew)
        .collect();
    assert_eq!(new_events.len(), 1, "solo dev-new es device_new");
    assert_eq!(new_events[0].device_id.as_deref(), Some("dev-new"));
}

#[test]
fn device_offline_emitted_and_is_online_zero() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "offline");
    upsert_device(
        &conn,
        &dev(
            "dev-a",
            mac("aa:bb:cc:dd:ee:01"),
            Some(ip("192.168.1.10")),
            true,
            "t0",
        ),
    )
    .unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    let (before, before_svc) = before_snapshot(&tx);
    // Scan: dev-a NO visto (no se re-upserta). cold_start=false.
    let events = run_diff(&tx, "net-1", "t1", before, before_svc, false).unwrap();
    let offlines: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventType::DeviceOffline)
        .collect();
    assert_eq!(offlines.len(), 1);
    assert_eq!(offlines[0].device_id.as_deref(), Some("dev-a"));
    assert!(
        !is_online_of(&tx, "dev-a"),
        "is_online=0 tras offline (escrito por run_diff)"
    );
}

#[test]
fn device_online_emitted_and_is_online_one() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "online");
    // Before: dev-b offline (is_online=false, last_seen="t0").
    upsert_device(
        &conn,
        &dev(
            "dev-b",
            mac("aa:bb:cc:dd:ee:02"),
            Some(ip("192.168.1.11")),
            false,
            "t0",
        ),
    )
    .unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    let (before, before_svc) = before_snapshot(&tx);
    // Scan: dev-b visto de nuevo (last_seen="t1"). upsert merge → is_online=1.
    upsert_device(
        &tx,
        &scan_dev(
            "dev-b",
            mac("aa:bb:cc:dd:ee:02"),
            Some(ip("192.168.1.11")),
            "t1",
        ),
    )
    .unwrap();
    let events = run_diff(&tx, "net-1", "t1", before, before_svc, false).unwrap();
    let onlines: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventType::DeviceOnline)
        .collect();
    assert_eq!(onlines.len(), 1);
    assert_eq!(onlines[0].device_id.as_deref(), Some("dev-b"));
    assert!(
        is_online_of(&tx, "dev-b"),
        "is_online=1 tras returning online"
    );
}

#[test]
fn device_ip_changed_emitted_with_old_new_ip() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "ipchange");
    upsert_device(
        &conn,
        &dev(
            "dev-c",
            mac("aa:bb:cc:dd:ee:03"),
            Some(ip("192.168.1.20")),
            true,
            "t0",
        ),
    )
    .unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    let (before, before_svc) = before_snapshot(&tx);
    // Scan: dev-c visto con IP nueva (DHCP).
    upsert_device(
        &tx,
        &scan_dev(
            "dev-c",
            mac("aa:bb:cc:dd:ee:03"),
            Some(ip("192.168.1.99")),
            "t1",
        ),
    )
    .unwrap();
    let events = run_diff(&tx, "net-1", "t1", before, before_svc, false).unwrap();
    let ipchanges: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventType::DeviceIpChanged)
        .collect();
    assert_eq!(ipchanges.len(), 1);
    assert_eq!(ipchanges[0].device_id.as_deref(), Some("dev-c"));
    let data = ipchanges[0].data_json.as_deref().expect("data_json");
    assert!(data.contains("192.168.1.20"), "old_ip en data_json: {data}");
    assert!(data.contains("192.168.1.99"), "new_ip en data_json: {data}");
}

#[test]
fn port_opened_emitted_for_new_service() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "port");
    upsert_device(
        &conn,
        &dev(
            "dev-d",
            mac("aa:bb:cc:dd:ee:04"),
            Some(ip("192.168.1.30")),
            true,
            "t0",
        ),
    )
    .unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    let (before, before_svc) = before_snapshot(&tx);
    // Scan: dev-d visto (last_seen="t1") + se inserta un service nuevo (mylan ports).
    upsert_device(
        &tx,
        &scan_dev(
            "dev-d",
            mac("aa:bb:cc:dd:ee:04"),
            Some(ip("192.168.1.30")),
            "t1",
        ),
    )
    .unwrap();
    service_repo::insert_service(&tx, &svc("dev-d", Protocol::Tcp, 80, Some("http"))).unwrap();

    let events = run_diff(&tx, "net-1", "t1", before, before_svc, false).unwrap();
    let ports: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventType::PortOpened)
        .collect();
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0].device_id.as_deref(), Some("dev-d"));
    let data = ports[0].data_json.as_deref().expect("data_json");
    assert!(data.contains("\"port\":80"), "port en data_json: {data}");
    assert!(
        data.contains("\"protocol\":\"tcp\""),
        "protocol en data_json: {data}"
    );
    assert!(
        data.contains("\"service_name\":\"http\""),
        "service_name en data_json: {data}"
    );
}

#[test]
fn cold_start_suppresses_offline_and_online_events() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "cold");
    // Before: A online, B offline.
    upsert_device(
        &conn,
        &dev(
            "dev-a",
            mac("aa:bb:cc:dd:ee:01"),
            Some(ip("192.168.1.10")),
            true,
            "t0",
        ),
    )
    .unwrap();
    upsert_device(
        &conn,
        &dev(
            "dev-b",
            mac("aa:bb:cc:dd:ee:02"),
            Some(ip("192.168.1.11")),
            false,
            "t0",
        ),
    )
    .unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    let (before, before_svc) = before_snapshot(&tx);
    // Scan: B visto (returning online, upsert merge → is_online=1), A no visto
    // (would be offline), C fresh (device_new). cold_start=true.
    upsert_device(
        &tx,
        &scan_dev(
            "dev-b",
            mac("aa:bb:cc:dd:ee:02"),
            Some(ip("192.168.1.11")),
            "t1",
        ),
    )
    .unwrap();
    upsert_device(
        &tx,
        &scan_dev(
            "dev-c",
            mac("aa:bb:cc:dd:ee:09"),
            Some(ip("192.168.1.99")),
            "t1",
        ),
    )
    .unwrap();
    let events = run_diff(&tx, "net-1", "t1", before, before_svc, true).unwrap();

    let offlines = events
        .iter()
        .filter(|e| e.event_type == EventType::DeviceOffline)
        .count();
    let onlines = events
        .iter()
        .filter(|e| e.event_type == EventType::DeviceOnline)
        .count();
    let news = events
        .iter()
        .filter(|e| e.event_type == EventType::DeviceNew)
        .count();
    assert_eq!(offlines, 0, "cold_start suprime DeviceOffline");
    assert_eq!(onlines, 0, "cold_start suprime DeviceOnline");
    assert_eq!(news, 1, "device_new NO se suprime en cold_start");
    // A NO marcado offline (sin escritura): is_online stays true.
    assert!(
        is_online_of(&tx, "dev-a"),
        "cold_start no escribe is_online=0"
    );
    // B is_online=1 (vía upsert merge, no vía run_diff).
    assert!(is_online_of(&tx, "dev-b"), "B online vía upsert merge");
}

#[test]
fn no_events_when_no_changes() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "nochange");
    upsert_device(
        &conn,
        &dev(
            "dev-a",
            mac("aa:bb:cc:dd:ee:01"),
            Some(ip("192.168.1.10")),
            true,
            "t0",
        ),
    )
    .unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    let (before, before_svc) = before_snapshot(&tx);
    // Scan: dev-a re-escaneado, misma IP, last_seen="t1". Sin cambios.
    upsert_device(
        &tx,
        &scan_dev(
            "dev-a",
            mac("aa:bb:cc:dd:ee:01"),
            Some(ip("192.168.1.10")),
            "t1",
        ),
    )
    .unwrap();
    let events = run_diff(&tx, "net-1", "t1", before, before_svc, false).unwrap();
    assert!(
        events.is_empty(),
        "sin cambios → sin events (got {events:?})"
    );
}
