use std::net::IpAddr;
use std::time::Instant;

use mylan_core::{Device, MacAddr, Network};
use mylan_db::{
    connection::connect,
    device_repo::upsert_device,
    diff::{run_diff, snapshot_devices_before, snapshot_services_before},
    network_repo,
};

fn ip(i: usize) -> IpAddr {
    format!("192.168.1.{}", (i % 250) + 1).parse().unwrap()
}

fn mac(i: usize) -> MacAddr {
    let b1 = (i >> 16) as u8;
    let b2 = (i >> 8) as u8;
    let b3 = i as u8;
    MacAddr::parse(&format!("02:00:00:{:02x}:{:02x}:{:02x}", b1, b2, b3)).unwrap()
}

fn setup_db(dir: &std::path::Path, count: usize) -> rusqlite::Connection {
    let conn = connect(dir.join("bench.db")).expect("connect");
    network_repo::upsert_network(
        &conn,
        &Network {
            id: "net-1".to_string(),
            name: "home".to_string(),
            cidr: "192.168.1.0/24".to_string(),
            gateway_ip: Some("192.168.1.1".parse().unwrap()),
            dns_servers: vec![],
            created_at: "t0".to_string(),
            updated_at: "t0".to_string(),
        },
    )
    .expect("upsert_network");

    for i in 0..count {
        let mut dev = Device::new(format!("dev-{i}"), "net-1", "t0");
        dev.primary_mac = Some(mac(i));
        dev.primary_ip = Some(ip(i));
        dev.is_online = true;
        upsert_device(&conn, &dev).expect("upsert_device");
    }

    conn
}

#[test]
fn bench_device_offline_and_online_diff() {
    let dir = tempfile::tempdir().unwrap();
    let count = 500;
    let conn = setup_db(dir.path(), count);

    // Snapshot before state (500 online devices)
    let before_devices = snapshot_devices_before(&conn, "net-1").unwrap();
    let before_ids: Vec<String> = before_devices.iter().map(|d| d.id.clone()).collect();
    let before_services = snapshot_services_before(&conn, &before_ids).unwrap();

    let tx = conn.unchecked_transaction().unwrap();

    // 500 devices go offline (none seen in scan at "t1")
    let start = Instant::now();
    let events = run_diff(&tx, "net-1", "t1", before_devices, before_services, false).unwrap();
    let elapsed_offline = start.elapsed();

    assert_eq!(events.len(), count);
    println!(
        "BENCHMARK: 500 devices offline processing took: {:?}",
        elapsed_offline
    );

    tx.commit().unwrap();

    // Now test returning online
    let before_devices_2 = snapshot_devices_before(&conn, "net-1").unwrap();
    let before_ids_2: Vec<String> = before_devices_2.iter().map(|d| d.id.clone()).collect();
    let before_services_2 = snapshot_services_before(&conn, &before_ids_2).unwrap();

    let tx2 = conn.unchecked_transaction().unwrap();

    // Re-upsert all 500 devices as seen in "t2" scan
    for i in 0..count {
        let mut dev = Device::new(format!("dev-{i}"), "net-1", "t2");
        dev.primary_mac = Some(mac(i));
        dev.primary_ip = Some(ip(i));
        upsert_device(&tx2, &dev).expect("upsert_device");
    }

    let start_online = Instant::now();
    let events_online = run_diff(
        &tx2,
        "net-1",
        "t2",
        before_devices_2,
        before_services_2,
        false,
    )
    .unwrap();
    let elapsed_online = start_online.elapsed();

    assert_eq!(events_online.len(), count);
    println!(
        "BENCHMARK: 500 devices online returning took: {:?}",
        elapsed_online
    );

    tx2.commit().unwrap();
}
