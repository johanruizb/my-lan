//! Repositorio de servicios/puertos (`services`).

use rusqlite::{Connection, Row};

use mylan_core::Service;

use crate::codec::{enum_from_db, enum_to_db};
use crate::error::{map_sqlite, DbResult};

struct ServiceRow {
    id: String,
    device_id: String,
    protocol: String,
    port: i64,
    service_name: Option<String>,
    product: Option<String>,
    version: Option<String>,
    banner: Option<String>,
    state: String,
    first_seen_at: String,
    last_seen_at: String,
}

impl ServiceRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            device_id: row.get(1)?,
            protocol: row.get(2)?,
            port: row.get(3)?,
            service_name: row.get(4)?,
            product: row.get(5)?,
            version: row.get(6)?,
            banner: row.get(7)?,
            state: row.get(8)?,
            first_seen_at: row.get(9)?,
            last_seen_at: row.get(10)?,
        })
    }

    fn decode(self) -> DbResult<Service> {
        Ok(Service {
            id: self.id,
            device_id: self.device_id,
            protocol: enum_from_db(&self.protocol)?,
            port: u16::try_from(self.port).unwrap_or(0),
            service_name: self.service_name,
            product: self.product,
            version: self.version,
            banner: self.banner,
            state: enum_from_db(&self.state)?,
            first_seen_at: self.first_seen_at,
            last_seen_at: self.last_seen_at,
        })
    }
}

/// Inserta un servicio/puerto.
pub fn insert_service(conn: &Connection, service: &Service) -> DbResult<()> {
    conn.execute(
        "INSERT INTO services (
           id, device_id, protocol, port, service_name, product, version, banner,
           state, first_seen_at, last_seen_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            service.id,
            service.device_id,
            enum_to_db(&service.protocol)?,
            i64::from(service.port),
            service.service_name,
            service.product,
            service.version,
            service.banner,
            enum_to_db(&service.state)?,
            service.first_seen_at,
            service.last_seen_at,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Inserta o actualiza un servicio por identidad `(device_id, protocol, port)`.
///
/// Re-escanear puertos del mismo host **actualiza** la fila (estado/banner/
/// timestamps) en vez de acumular duplicados, preservando `first_seen_at` (mismo
/// espíritu P5 que el upsert de dispositivos).
pub fn upsert_service(conn: &Connection, service: &Service) -> DbResult<()> {
    let existing_id: Option<String> = match conn.query_row(
        "SELECT id FROM services WHERE device_id = ?1 AND protocol = ?2 AND port = ?3",
        rusqlite::params![
            service.device_id,
            enum_to_db(&service.protocol)?,
            i64::from(service.port),
        ],
        |row| row.get(0),
    ) {
        Ok(id) => Some(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err(map_sqlite(e)),
    };

    if let Some(id) = existing_id {
        conn.execute(
            "UPDATE services SET service_name = ?1, product = ?2, version = ?3,
               banner = ?4, state = ?5, last_seen_at = ?6 WHERE id = ?7",
            rusqlite::params![
                service.service_name,
                service.product,
                service.version,
                service.banner,
                enum_to_db(&service.state)?,
                service.last_seen_at,
                id,
            ],
        )
        .map_err(map_sqlite)?;
    } else {
        insert_service(conn, service)?;
    }
    Ok(())
}

/// Lista los servicios de un dispositivo.
pub fn list_services_by_device(conn: &Connection, device_id: &str) -> DbResult<Vec<Service>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, device_id, protocol, port, service_name, product, version, banner,
                    state, first_seen_at, last_seen_at
             FROM services WHERE device_id = ?1 ORDER BY port",
        )
        .map_err(map_sqlite)?;
    let rows = stmt
        .query_map([device_id], ServiceRow::from_row)
        .map_err(map_sqlite)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_sqlite)?.decode()?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::connect;
    use crate::device_repo::upsert_device;
    use crate::network_repo::upsert_network;
    use mylan_core::{Device, MacAddr, Protocol, ServiceState};

    fn ip(s: &str) -> std::net::IpAddr {
        s.parse().unwrap()
    }
    fn mac(s: &str) -> MacAddr {
        MacAddr::parse(s).unwrap()
    }

    fn fixture(dir: &std::path::Path) -> Connection {
        let conn = connect(dir.join("svc.db")).unwrap();
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
        let mut d = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        d.primary_mac = Some(mac("aa:bb:cc:dd:ee:ff"));
        d.primary_ip = Some(ip("192.168.1.5"));
        upsert_device(&conn, &d).unwrap();
        conn
    }

    fn sample_service(port: u16) -> Service {
        Service {
            id: format!("svc-{port}"),
            device_id: "dev-1".to_string(),
            protocol: Protocol::Tcp,
            port,
            service_name: Some("http".to_string()),
            product: Some("nginx".to_string()),
            version: None,
            banner: Some("HTTP/1.1 200".to_string()),
            state: ServiceState::Open,
            first_seen_at: "2026-06-27T00:00:00Z".to_string(),
            last_seen_at: "2026-06-27T00:00:10Z".to_string(),
        }
    }

    #[test]
    fn insert_and_list_services() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture(dir.path());
        insert_service(&conn, &sample_service(80)).unwrap();
        insert_service(&conn, &sample_service(443)).unwrap();
        let svc = list_services_by_device(&conn, "dev-1").unwrap();
        assert_eq!(svc.len(), 2);
        assert_eq!(svc[0].port, 80);
        assert_eq!(svc[1].port, 443);
        assert_eq!(svc[0].protocol, Protocol::Tcp);
        assert_eq!(svc[0].state, ServiceState::Open);
    }

    #[test]
    fn upsert_service_no_duplicate_on_rescan() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture(dir.path());
        let mut s = sample_service(80);
        s.first_seen_at = "t0".to_string();
        s.last_seen_at = "t0".to_string();
        upsert_service(&conn, &s).unwrap();
        // Re-escaneo del mismo puerto: banner/estado/last_seen cambian, sin dup.
        s.banner = Some("HTTP/1.1 301".to_string());
        s.last_seen_at = "t1".to_string();
        s.first_seen_at = "t1".to_string(); // entrante distinto: NO debe pisar el original
        upsert_service(&conn, &s).unwrap();
        let svc = list_services_by_device(&conn, "dev-1").unwrap();
        assert_eq!(svc.len(), 1, "un solo servicio, no duplicado");
        assert_eq!(svc[0].banner.as_deref(), Some("HTTP/1.1 301"));
        assert_eq!(svc[0].last_seen_at, "t1");
        assert_eq!(svc[0].first_seen_at, "t0", "first_seen preservado");
    }

    #[test]
    fn service_fk_rejects_orphan() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(dir.path().join("orphan.db")).unwrap();
        let s = sample_service(22);
        let mut orphan = s.clone();
        orphan.device_id = "ghost-device".to_string();
        let res = insert_service(&conn, &orphan);
        assert!(res.is_err(), "FK should reject orphan service");
    }
}
