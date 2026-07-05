//! AC-3: scheduler tick — `scan_network` emite events al broadcast, y el skip
//! guard evita scans solapados de la misma red.

use std::net::IpAddr;
use std::time::Duration;

use mylan_agent::{scan_network, NetworkRunner, NetworkSchedule};
use mylan_api::event_channel;
use mylan_core::{noop_enricher, Enricher, EventType, MacAddr, Observation, ScanProfile, Source};

#[tokio::test]
async fn scan_network_emits_device_new_event() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("mylan.db");
    let (tx, mut rx) = event_channel(64);
    let net = NetworkSchedule {
        cidr: "192.168.1.0/24".to_string(),
        profile: ScanProfile::Quick,
    };
    let mac = MacAddr::parse("aa:bb:cc:dd:ee:ff").expect("mac");
    let ip: IpAddr = "192.168.1.5".parse().expect("ip");
    let obs = vec![Observation::new(Source::ArpCache).with_mac(mac).with_ip(ip)];
    let enricher: Enricher = noop_enricher();

    let outcome = scan_network(&db_path, &net, &obs, &enricher, true, &tx)
        .await
        .expect("scan_network");
    assert!(outcome.hosts_alive >= 1, "debe reportar el host inyectado");

    // Drenar el broadcast: device_new se emite para un host nuevo (cold_start
    // suprime online/offline pero NO device_new).
    tokio::time::sleep(Duration::from_millis(50)).await;
    let mut got_device_new = false;
    while let Ok(ev) = rx.try_recv() {
        if ev.event_type == EventType::DeviceNew {
            got_device_new = true;
        }
    }
    assert!(got_device_new, "debe emitir DeviceNew para un host nuevo");
}

#[tokio::test]
async fn scan_network_persists_and_rediscovers_no_duplicate() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("mylan.db");
    let (tx, _rx) = event_channel(64);
    let net = NetworkSchedule {
        cidr: "192.168.1.0/24".to_string(),
        profile: ScanProfile::Quick,
    };
    let mac = MacAddr::parse("aa:bb:cc:dd:ee:01").expect("mac");
    let ip: IpAddr = "192.168.1.10".parse().expect("ip");
    let obs = vec![Observation::new(Source::ArpCache).with_mac(mac).with_ip(ip)];
    let enricher: Enricher = noop_enricher();

    // Primer scan: inserta el device (cold_start=true).
    let o1 = scan_network(&db_path, &net, &obs, &enricher, true, &tx)
        .await
        .expect("scan 1");
    assert_eq!(o1.hosts_new, 1, "primer scan inserta 1 device nuevo");

    // Segundo scan (cold_start=false): mismo host → no es nuevo.
    let o2 = scan_network(&db_path, &net, &obs, &enricher, false, &tx)
        .await
        .expect("scan 2");
    assert_eq!(o2.hosts_alive, 1, "mismo host re-escaneado");
    assert_eq!(o2.hosts_new, 0, "no hay devices nuevos en el segundo scan");
}

#[tokio::test]
async fn skip_guard_skips_overlapping_scan_of_same_network() {
    let runner = NetworkRunner::new();
    assert!(
        runner.try_start("net-A").await,
        "primera adquisición del skip guard"
    );
    assert!(
        !runner.try_start("net-A").await,
        "scan solapado de la misma red → skip"
    );
    runner.mark_done("net-A").await;
    assert!(
        runner.try_start("net-A").await,
        "tras mark_done se puede volver a empezar"
    );
}

#[tokio::test]
async fn skip_guard_independent_per_network() {
    let runner = NetworkRunner::new();
    assert!(runner.try_start("net-A").await, "net-A empieza");
    assert!(
        runner.try_start("net-B").await,
        "net-B empieza independientemente de net-A"
    );
    assert!(
        !runner.try_start("net-A").await,
        "net-A sigue en curso → skip"
    );
    assert!(
        !runner.try_start("net-B").await,
        "net-B sigue en curso → skip"
    );
    runner.mark_done("net-A").await;
    assert!(runner.try_start("net-A").await, "net-A libre de nuevo");
}

#[tokio::test]
async fn scan_network_uses_cidr_as_network_id() {
    // M1 fix: el network id es el CIDR (match CLI `mylan scan` que usa
    // `iface.cidr()` como id), no un hash. Así el agent y el CLI upsertean la
    // misma fila network (no duplican). Verifica vía el network_id de los events
    // (viene del network.id upserteado por el pipeline).
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("mylan.db");
    let (tx, _rx) = event_channel(64);
    let net = NetworkSchedule {
        cidr: "192.168.1.0/24".to_string(),
        profile: ScanProfile::Quick,
    };
    let mac = MacAddr::parse("aa:bb:cc:dd:ee:02").expect("mac");
    let ip: IpAddr = "192.168.1.20".parse().expect("ip");
    let obs = vec![Observation::new(Source::ArpCache).with_mac(mac).with_ip(ip)];
    let enricher: Enricher = noop_enricher();
    scan_network(&db_path, &net, &obs, &enricher, true, &tx)
        .await
        .expect("scan");
    let conn = mylan_db::connection::connect(&db_path).expect("connect");
    let events = mylan_db::events_repo::list_events(&conn, None, 100, 0).expect("list_events");
    assert!(
        events.iter().all(|e| e.network_id == "192.168.1.0/24"),
        "M1: network_id de los events debe ser el CIDR (match CLI), no un hash"
    );
}
