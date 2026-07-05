//! Integration tests de atomicidad ADR-5: el pipeline+diff persiste devices y
//! events en UNA transacción, o no persiste nada (rollback ante crash pre-commit).
//!
//! - `pipeline_with_diff_commits_devices_and_events`: happy path — commit escribe
//!   devices + events atómicamente.
//! - `pipeline_atomic_no_commit_rolls_back_devices_and_events`: simula un crash
//!   antes del commit (snapshot + `run_scan_pipeline_at_in_tx` + `run_diff` +
//!   `insert_event` sin `tx.commit()`) → `drop(tx)` rueda atrás → ni devices ni
//!   events quedan escritos.

use std::net::IpAddr;

use mylan_core::{Device, EventType, MacAddr, Network, Observation, ScanProfile, Source};

use mylan_db::{
    connection::connect,
    device_repo::{list_devices, upsert_device},
    diff::{run_diff, snapshot_devices_before, snapshot_services_before},
    events_repo::{insert_event, list_events},
    network_repo,
    pipeline::{run_scan_pipeline_at_in_tx, run_scan_pipeline_with_diff},
};

fn ip(s: &str) -> IpAddr {
    s.parse().expect("valid ip")
}

fn mac(s: &str) -> MacAddr {
    MacAddr::parse(s).expect("valid mac")
}

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

fn obs(mac_addr: MacAddr, ip_addr: &str, hostname: Option<&str>) -> Observation {
    let mut o = Observation::new(Source::ArpCache)
        .with_mac(mac_addr)
        .with_ip(ip(ip_addr));
    if let Some(h) = hostname {
        o = o.with_hostname(h);
    }
    o
}

fn before_device() -> Device {
    let mut d = Device::new("dev-before", "net-1", "t0");
    d.primary_mac = Some(mac("aa:bb:cc:00:00:aa"));
    d.primary_ip = Some(ip("192.168.1.50"));
    d
}

#[test]
fn pipeline_with_diff_commits_devices_and_events() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "happy");
    let enricher = mylan_core::noop_enricher();
    let observations = vec![
        obs(
            mac("aa:bb:cc:00:00:01"),
            "192.168.1.1",
            Some("router.local"),
        ),
        obs(mac("aa:bb:cc:00:00:02"), "192.168.1.5", Some("nas.local")),
    ];

    let (outcome, events) = run_scan_pipeline_with_diff(
        &conn,
        &fixture_network(),
        &observations,
        &enricher,
        ScanProfile::Quick,
        "t1",
        false,
    )
    .expect("pipeline_with_diff ok");

    assert_eq!(outcome.hosts_alive, 2);
    assert_eq!(outcome.hosts_new, 2);
    // 2 device_new events (ambos devices frescos).
    let news = events
        .iter()
        .filter(|e| e.event_type == EventType::DeviceNew)
        .count();
    assert_eq!(news, 2, "device_new por cada device nuevo");

    // Devices persistidos en la DB.
    let devices = list_devices(&conn, "net-1").unwrap();
    assert_eq!(devices.len(), 2, "devices committed");

    // Events persistidos en la DB (atómico con el scan — ADR-5).
    let db_events = list_events(&conn, None, 100, 0).unwrap();
    assert_eq!(db_events.len(), 2, "events committed en la misma txn");
}

#[test]
fn pipeline_atomic_no_commit_rolls_back_devices_and_events() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "atomic");
    let enricher = mylan_core::noop_enricher();
    // Estado before: 1 device conocido.
    upsert_device(&conn, &before_device()).unwrap();

    let devices_before = list_devices(&conn, "net-1").unwrap().len();
    let events_before = list_events(&conn, None, 100, 0).unwrap().len();

    // Simula un crash ANTES del commit: abre txn, hace snapshot + pipeline + diff
    // + insert events, PERO NO commitea. `drop(tx)` → rollback (rusqlite
    // Transaction::drop ejecuta ROLLBACK si no se commiteó).
    let tx = conn.unchecked_transaction().unwrap();
    let before = snapshot_devices_before(&tx, "net-1").unwrap();
    let before_ids: Vec<String> = before.iter().map(|d| d.id.clone()).collect();
    let before_svc = snapshot_services_before(&tx, &before_ids).unwrap();
    let observations = vec![obs(
        mac("aa:bb:cc:00:00:bb"),
        "192.168.1.60",
        Some("newhost.local"),
    )];
    let _outcome = run_scan_pipeline_at_in_tx(
        &tx,
        &fixture_network(),
        &observations,
        &enricher,
        ScanProfile::Quick,
        "t1",
    )
    .expect("_in_tx ok");
    let events = run_diff(&tx, "net-1", "t1", before, before_svc, false).expect("run_diff ok");
    for event in &events {
        insert_event(&tx, event).expect("insert_event ok");
    }
    // NO tx.commit() — crash antes del commit.
    drop(tx);

    let devices_after = list_devices(&conn, "net-1").unwrap().len();
    let events_after = list_events(&conn, None, 100, 0).unwrap().len();
    assert_eq!(
        devices_after, devices_before,
        "rollback: no se persistieron devices nuevos"
    );
    assert_eq!(
        events_after, events_before,
        "rollback: no se persistieron events (ADR-5 atómico)"
    );
}

/// Red `net-1` reusada por ambos tests (misma forma que fixture_conn).
fn fixture_network() -> Network {
    Network {
        id: "net-1".to_string(),
        name: "home".to_string(),
        cidr: "192.168.1.0/24".to_string(),
        gateway_ip: Some(ip("192.168.1.1")),
        dns_servers: vec![],
        created_at: "t0".to_string(),
        updated_at: "t0".to_string(),
    }
}
