//! `mylan-db` — persistencia SQLite local de MyLAN.
//!
//! Conexión `rusqlite` (feature `bundled`), migraciones SQL embebidas gobernadas
//! por `PRAGMA user_version`, y repositorios (upsert de dispositivos por
//! identidad estable, servicios, scans). Esquema según el plan §8.
//!
//! Modelo de uso típico desde `apps/cli`:
//! ```ignore
//! use mylan_db::connection::{connect, default_db_path};
//! let conn = connect(default_db_path().expect("home dir"))?;
//! // migraciones ya aplicadas implícitamente en connect().
//! mylan_db::device_repo::upsert_device(&conn, &device)?;
//! ```

#![forbid(unsafe_code)]

pub mod codec;
pub mod connection;
pub mod device_repo;
pub mod error;
pub mod migrations;
pub mod network_repo;
pub mod scan_repo;
pub mod service_repo;

// Re-exports ergonómicos de la API pública.
pub use error::{DbError, DbResult};

#[cfg(test)]
mod concurrency_tests {
    //! Concurrencia básica: dos conexiones compiten por un único fichero SQLite.
    //! El segundo escritor debe obtener un error (no colgar) durante la
    //! contención, lo que demuestra que el bloqueo se respeta.

    use super::*;
    use crate::device_repo::upsert_device;
    use crate::network_repo::upsert_network;
    use mylan_core::{Device, MacAddr};
    use rusqlite::Connection;
    use std::net::IpAddr;
    use std::thread;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }
    fn mac(s: &str) -> MacAddr {
        MacAddr::parse(s).unwrap()
    }

    fn fixture_conn(path: std::path::PathBuf) -> Connection {
        let conn = connection::connect(path).unwrap();
        upsert_network(
            &conn,
            &mylan_core::Network {
                id: "net-1".to_string(),
                name: "home".to_string(),
                cidr: "192.168.1.0/24".to_string(),
                gateway_ip: Some(ip("192.168.1.1")),
                dns_servers: vec![],
                created_at: "2026-06-27T00:00:00Z".to_string(),
                updated_at: "2026-06-27T00:00:00Z".to_string(),
            },
        )
        .unwrap();
        conn
    }

    #[test]
    fn two_writers_contention_errors_instead_of_hanging() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lock.db");
        // Primera conexión retiene un bloqueo de escritura con BEGIN IMMEDIATE.
        let conn_a = fixture_conn(path.clone());
        conn_a
            .execute_batch(
                "BEGIN IMMEDIATE;\
                 INSERT INTO devices (id, network_id, primary_mac, primary_ip, device_type,\
                 confidence, first_seen_at, last_seen_at)\
                 VALUES ('holder','net-1',NULL,NULL,'unknown',0,'t','t');",
            )
            .unwrap();

        // Un segundo hilo intenta escribir mientras el primero retiene el lock.
        let path_b = path.clone();
        let handle = thread::spawn(move || {
            let conn_b = connection::connect(&path_b).unwrap();
            let mut d = Device::new("dev-contended", "net-1", "2026-06-27T00:00:00Z");
            d.primary_mac = Some(mac("aa:bb:cc:00:00:10"));
            d.primary_ip = Some(ip("192.168.1.50"));
            // Timeout corto para que no cuelgue el test: SQLite reintenta y falla.
            conn_b
                .busy_timeout(std::time::Duration::from_millis(50))
                .ok();
            upsert_device(&conn_b, &d)
        });

        let outcome = handle.join().expect("thread");
        assert!(
            outcome.is_err(),
            "expected contention error, got {outcome:?}"
        );

        conn_a.execute_batch("COMMIT;").unwrap();
    }
}
