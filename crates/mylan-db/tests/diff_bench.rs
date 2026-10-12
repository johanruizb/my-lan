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
    format!("10.0.{}.{}", (i / 250) % 250, i % 250 + 1)
        .parse()
        .unwrap()
}

fn mac(i: usize) -> MacAddr {
    let b1 = (i >> 16) & 0xff;
    let b2 = (i >> 8) & 0xff;
    let b3 = i & 0xff;
    MacAddr::parse(&format!("02:00:00:{:02x}:{:02x}:{:02x}", b1, b2, b3)).unwrap()
}

#[test]
fn bench_run_diff_online_and_offline() {
    let dir = tempfile::tempdir().unwrap();
    let conn = connect(dir.path().join("bench.db")).expect("connect");

    network_repo::upsert_network(
        &conn,
        &Network {
            id: "net-1".to_string(),
            name: "home".to_string(),
            cidr: "10.0.0.0/16".to_string(),
            gateway_ip: Some("10.0.0.1".parse().unwrap()),
            dns_servers: vec![],
            created_at: "t0".to_string(),
            updated_at: "t0".to_string(),
        },
    )
    .expect("upsert_network");

    let num_devices = 1000;
    // Insert 1000 devices into DB.
    // First 500: is_online = true in before state, but will be missing in after (online -> offline).
    // Second 500: is_online = false in before state, but will appear in after (offline -> online).
    for i in 0..num_devices {
        let before_online = i < 500;
        let mut d = Device::new(format!("dev-{i}"), "net-1", "t0");
        d.primary_mac = Some(mac(i));
        d.primary_ip = Some(ip(i));
        d.is_online = before_online;
        upsert_device(&conn, &d).unwrap();
    }

    let iterations = 20;
    let mut total_duration = std::time::Duration::ZERO;

    for _ in 0..iterations {
        let tx = conn.unchecked_transaction().unwrap();

        // Reset state: 0..500 are online (last_seen_at = "t0"), 500..1000 are offline (last_seen_at = "t0")
        tx.execute("UPDATE devices SET is_online = 0 WHERE id >= 'dev-500'", [])
            .unwrap();
        tx.execute("UPDATE devices SET is_online = 1 WHERE id < 'dev-500'", [])
            .unwrap();
        tx.execute("UPDATE devices SET last_seen_at = 't0'", [])
            .unwrap();

        // Re-upsert devices 500..1000 with last_seen_at = "t1" (marking them as seen in this scan)
        for i in 500..1000 {
            let mut d = Device::new(format!("dev-{i}"), "net-1", "t1");
            d.primary_mac = Some(mac(i));
            d.primary_ip = Some(ip(i));
            upsert_device(&tx, &d).unwrap();
        }

        let before_devices = snapshot_devices_before(&tx, "net-1").unwrap();
        let before_ids: Vec<String> = before_devices.iter().map(|d| d.id.clone()).collect();
        let before_services = snapshot_services_before(&tx, &before_ids).unwrap();

        let start = Instant::now();
        let events = run_diff(&tx, "net-1", "t1", before_devices, before_services, false).unwrap();
        let elapsed = start.elapsed();

        total_duration += elapsed;
        assert!(!events.is_empty());

        tx.rollback().unwrap();
    }

    let avg_time = total_duration / iterations as u32;
    println!("\n=== BENCHMARK RESULT ===");
    println!("Total iterations: {}", iterations);
    println!(
        "Average time per run_diff (500 offline + 500 online changes): {:?}",
        avg_time
    );
    println!("========================\n");
}
