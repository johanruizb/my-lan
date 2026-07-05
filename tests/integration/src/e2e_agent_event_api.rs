//! AC-8 e2e: agent + new device → `device_new` en `events` + `GET /events` +
//! `WS /events/live`. Single-process ADR-4: el agent (`scan_network`) y el API
//! (`mylan_api::serve`) corren en un único proceso tokio; el WS `/events/live`
//! suscribe al mismo `broadcast` que el agent emite, así que verificar el
//! broadcast es verificar el WS en topología single-process.

#![cfg(test)]

use std::time::Duration;

use mylan_api::event_channel;
use mylan_agent::{NetworkSchedule, scan_network};
use mylan_core::{Enricher, EventType, ScanProfile, noop_enricher};

/// Puerto efímero libre (race pequeña: el listener se dropea antes del serve
/// bind, pero en práctica el OS no lo reasigna inmediatamente).
fn free_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .expect("bind efímero")
        .local_addr()
        .expect("local_addr")
        .port()
}

#[tokio::test]
async fn e2e_new_device_emits_device_new_in_events_api_and_broadcast() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("e2e.db");
    let port = free_port();
    let token = "e2e-test-token";

    // Canal de broadcast (ADR-4): el agent emite, el API lo guarda en State,
    // el WS /events/live suscribe. Un receiver captura lo que el WS vería.
    let (tx, mut rx) = event_channel(64);

    // Arranca el API embebido en el mismo proceso (ADR-4) como tokio task.
    let serve_db = db_path.clone();
    let serve_token = token.to_string();
    let serve_tx = tx.clone();
    let serve_handle = tokio::spawn(async move {
        let _ = mylan_api::serve(serve_db, port, &serve_token, serve_tx).await;
    });
    wait_for_port(port).await;

    // Inyecta un device nuevo via scan_network (agent single-process).
    let net = NetworkSchedule {
        cidr: "192.168.1.0/24".to_string(),
        profile: ScanProfile::Quick,
    };
    let enricher: Enricher = noop_enricher();
    let obs = vec![mylan_core::Observation::new(mylan_core::Source::ArpCache)
        .with_mac(mylan_core::MacAddr::parse("aa:bb:cc:dd:ee:99").expect("mac"))
        .with_ip("192.168.1.99".parse().expect("ip"))];
    let outcome = scan_network(&db_path, &net, &obs, &enricher, true, &tx)
        .await
        .expect("scan_network");
    assert!(outcome.hosts_alive >= 1);

    // (1) device_new persistido en la DB (events table).
    let conn = mylan_db::connection::connect(&db_path).expect("connect");
    let events = mylan_db::events_repo::list_events(&conn, None, 100, 0).expect("list_events");
    assert!(
        events.iter().any(|e| e.event_type == EventType::DeviceNew),
        "device_new debe estar en la events table"
    );

    // (2) device_new en el broadcast (WS /events/live suscribe al mismo canal).
    let mut got_device_new = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        match rx.try_recv() {
            Ok(ev) if ev.event_type == EventType::DeviceNew => {
                got_device_new = true;
                break;
            }
            Ok(_) => continue,
            Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
        }
    }
    assert!(
        got_device_new,
        "device_new debe llegar al broadcast (WS /events/live)"
    );

    // (3) GET /events vía HTTP real al API embebido (token bearer).
    let body = http_get_events(port, token).await.expect("GET /events 200");
    assert!(
        body.contains("device_new"),
        "GET /events debe incluir device_new: {body}"
    );

    serve_handle.abort();
}

/// Espera a que el puerto del API acepte conexiones (best-effort, hasta 5s).
async fn wait_for_port(port: u16) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("API no arrancó en puerto {port} tras 5s");
}

/// GET /api/v1/events con `Authorization: Bearer <token>`; devuelve el body
/// si la respuesta es 200, `None` en caso contrario. Sin deps HTTP externas
/// (tokio TcpStream + HTTP/1.1 manual).
async fn http_get_events(port: u16, token: &str) -> Option<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .ok()?;
    let req = format!(
        "GET /api/v1/events HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).await.ok()?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.ok()?;
    let resp = String::from_utf8_lossy(&buf).to_string();
    if !resp.contains("200") {
        return None;
    }
    Some(resp)
}