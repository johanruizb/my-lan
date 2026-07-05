//! Integration tests del repositorio de eventos (AC-4: timeline en `events` +
//! `GET /api/v1/events` ordenado por `created_at`; AC-6: cursor `?since`).
//!
//! Black-box vía la API pública de `mylan-db` (`connection::connect` aplica
//! migraciones → tabla `events` de V4; `network_repo`/`device_repo` montan las
//! FKs). `mylan-core` entra como dev-dep para construir `Event`/`EventType`/
//! `Severity`/`Network`/`Device`.

use std::net::IpAddr;

use mylan_core::{Device, Event, EventType, MacAddr, Network, Severity};

use mylan_db::{connection::connect, device_repo::upsert_device, events_repo, network_repo};

fn ip(s: &str) -> IpAddr {
    s.parse().expect("valid ip")
}

fn mac(s: &str) -> MacAddr {
    MacAddr::parse(s).expect("valid mac")
}

/// Conexión con una red `net-1` ya insertada (FK de `events.network_id`).
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
            created_at: "2026-07-03T00:00:00Z".to_string(),
            updated_at: "2026-07-03T00:00:00Z".to_string(),
        },
    )
    .expect("upsert_network");
    conn
}

fn event(id: &str, net: &str, et: EventType, sev: Severity, created_at: &str) -> Event {
    Event {
        id: id.to_string(),
        network_id: net.to_string(),
        device_id: None,
        event_type: et,
        severity: sev,
        message: Some(format!("event {id}")),
        data_json: Some(r#"{"k":1}"#.to_string()),
        created_at: created_at.to_string(),
    }
}

#[test]
fn insert_then_list_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "rt");
    let e = event(
        "e1",
        "net-1",
        EventType::DeviceNew,
        Severity::Info,
        "2026-07-03T00:00:00Z",
    );
    events_repo::insert_event(&conn, &e).unwrap();

    let got = events_repo::list_events(&conn, None, 10, 0).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(
        got[0], e,
        "round-trip exacto (codec EventType/Severity + campos)"
    );
}

#[test]
fn list_empty_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "empty");
    let got = events_repo::list_events(&conn, None, 10, 0).unwrap();
    assert!(got.is_empty());
    let since = events_repo::list_events_since(&conn, "2000-01-01T00:00:00Z").unwrap();
    assert!(since.is_empty());
}

#[test]
fn list_orders_by_created_at_desc() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "order");
    events_repo::insert_event(
        &conn,
        &event(
            "e0",
            "net-1",
            EventType::DeviceNew,
            Severity::Info,
            "2026-07-03T00:00:00Z",
        ),
    )
    .unwrap();
    events_repo::insert_event(
        &conn,
        &event(
            "e1",
            "net-1",
            EventType::PortOpened,
            Severity::Warning,
            "2026-07-03T00:00:01Z",
        ),
    )
    .unwrap();
    events_repo::insert_event(
        &conn,
        &event(
            "e2",
            "net-1",
            EventType::DeviceOffline,
            Severity::Critical,
            "2026-07-03T00:00:02Z",
        ),
    )
    .unwrap();

    let got = events_repo::list_events(&conn, None, 10, 0).unwrap();
    assert_eq!(
        got.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        ["e2", "e1", "e0"]
    );
}

#[test]
fn list_filters_by_network() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "filter");
    // Segunda red para el filtro.
    network_repo::upsert_network(
        &conn,
        &Network {
            id: "net-2".to_string(),
            name: "guest".to_string(),
            cidr: "10.0.0.0/24".to_string(),
            gateway_ip: Some(ip("10.0.0.1")),
            dns_servers: vec![],
            created_at: "2026-07-03T00:00:00Z".to_string(),
            updated_at: "2026-07-03T00:00:00Z".to_string(),
        },
    )
    .unwrap();

    events_repo::insert_event(
        &conn,
        &event(
            "a1",
            "net-1",
            EventType::DeviceNew,
            Severity::Info,
            "2026-07-03T00:00:00Z",
        ),
    )
    .unwrap();
    events_repo::insert_event(
        &conn,
        &event(
            "b1",
            "net-2",
            EventType::DeviceNew,
            Severity::Info,
            "2026-07-03T00:00:01Z",
        ),
    )
    .unwrap();
    events_repo::insert_event(
        &conn,
        &event(
            "a2",
            "net-1",
            EventType::PortOpened,
            Severity::Warning,
            "2026-07-03T00:00:02Z",
        ),
    )
    .unwrap();

    let n1 = events_repo::list_events(&conn, Some("net-1"), 10, 0).unwrap();
    assert_eq!(
        n1.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        ["a2", "a1"]
    );
    let n2 = events_repo::list_events(&conn, Some("net-2"), 10, 0).unwrap();
    assert_eq!(n2.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), ["b1"]);
    let all = events_repo::list_events(&conn, None, 10, 0).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn list_paginates_limit_offset() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "page");
    for (i, t) in ["00:00", "00:01", "00:02"].iter().enumerate() {
        events_repo::insert_event(
            &conn,
            &event(
                &format!("e{i}"),
                "net-1",
                EventType::DeviceNew,
                Severity::Info,
                &format!("2026-07-03T{t}Z"),
            ),
        )
        .unwrap();
    }
    // Orden DESC total: [e2, e1, e0].
    let page1 = events_repo::list_events(&conn, None, 2, 0).unwrap();
    assert_eq!(
        page1.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        ["e2", "e1"]
    );
    let page2 = events_repo::list_events(&conn, None, 2, 2).unwrap();
    assert_eq!(
        page2.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        ["e0"]
    );
}

#[test]
fn list_since_returns_events_after_cursor_asc() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "since");
    events_repo::insert_event(
        &conn,
        &event(
            "e0",
            "net-1",
            EventType::DeviceNew,
            Severity::Info,
            "2026-07-03T00:00:00Z",
        ),
    )
    .unwrap();
    events_repo::insert_event(
        &conn,
        &event(
            "e1",
            "net-1",
            EventType::DeviceOnline,
            Severity::Info,
            "2026-07-03T00:00:01Z",
        ),
    )
    .unwrap();
    events_repo::insert_event(
        &conn,
        &event(
            "e2",
            "net-1",
            EventType::DeviceOffline,
            Severity::Warning,
            "2026-07-03T00:00:02Z",
        ),
    )
    .unwrap();

    // Cursor = e0.created_at: devuelve e1, e2 en orden cronológico ascendente.
    let got = events_repo::list_events_since(&conn, "2026-07-03T00:00:00Z").unwrap();
    assert_eq!(
        got.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        ["e1", "e2"]
    );
    // Cursor futuro: vacío.
    let future = events_repo::list_events_since(&conn, "2026-07-04T00:00:00Z").unwrap();
    assert!(future.is_empty());
}

#[test]
fn fk_rejects_orphan_network() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "fk");
    let e = event(
        "eX",
        "net-nope",
        EventType::DeviceNew,
        Severity::Info,
        "2026-07-03T00:00:00Z",
    );
    let res = events_repo::insert_event(&conn, &e);
    assert!(
        res.is_err(),
        "FK network_id debe rechazar red inexistente: {res:?}"
    );
}

#[test]
fn device_id_null_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "nulldev");
    let mut e = event(
        "e1",
        "net-1",
        EventType::DeviceNew,
        Severity::Info,
        "2026-07-03T00:00:00Z",
    );
    e.device_id = None;
    events_repo::insert_event(&conn, &e).unwrap();
    let got = events_repo::list_events(&conn, None, 10, 0).unwrap();
    assert_eq!(got.len(), 1);
    assert!(got[0].device_id.is_none(), "device_id NULL preservado");
}

#[test]
fn device_id_references_real_device() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "realdev");
    let mut d = Device::new("dev-1", "net-1", "2026-07-03T00:00:00Z");
    d.primary_mac = Some(mac("aa:bb:cc:dd:ee:ff"));
    d.primary_ip = Some(ip("192.168.1.5"));
    upsert_device(&conn, &d).unwrap();

    let mut e = event(
        "e1",
        "net-1",
        EventType::DeviceNew,
        Severity::Info,
        "2026-07-03T00:00:00Z",
    );
    e.device_id = Some("dev-1".to_string());
    events_repo::insert_event(&conn, &e).unwrap();
    let got = events_repo::list_events(&conn, None, 10, 0).unwrap();
    assert_eq!(got[0].device_id.as_deref(), Some("dev-1"));
}

#[test]
fn event_type_and_severity_codec_all_variants() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "codec");
    let variants = [
        (EventType::DeviceNew, Severity::Info),
        (EventType::DeviceIpChanged, Severity::Info),
        (EventType::DeviceOffline, Severity::Warning),
        (EventType::DeviceOnline, Severity::Warning),
        (EventType::PortOpened, Severity::Critical),
    ];
    for (i, (et, sev)) in variants.iter().enumerate() {
        let e = event(
            &format!("e{i}"),
            "net-1",
            *et,
            *sev,
            &format!("2026-07-03T00:00:0{i}Z"),
        );
        events_repo::insert_event(&conn, &e).unwrap();
    }
    let got = events_repo::list_events(&conn, None, 100, 0).unwrap();
    assert_eq!(got.len(), variants.len());
    // list_events es DESC por created_at → invertimos para comparar contra el orden de inserción.
    let by_insertion: Vec<_> = got.into_iter().rev().collect();
    for (got, (et, sev)) in by_insertion.iter().zip(variants.iter()) {
        assert_eq!(got.event_type, *et, "EventType round-trip");
        assert_eq!(got.severity, *sev, "Severity round-trip");
    }
}

#[test]
fn limit_zero_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let conn = fixture_conn(dir.path(), "limit0");
    events_repo::insert_event(
        &conn,
        &event(
            "e1",
            "net-1",
            EventType::DeviceNew,
            Severity::Info,
            "2026-07-03T00:00:00Z",
        ),
    )
    .unwrap();
    let got = events_repo::list_events(&conn, None, 0, 0).unwrap();
    assert!(got.is_empty(), "limit=0 debe devolver vacío");
}
