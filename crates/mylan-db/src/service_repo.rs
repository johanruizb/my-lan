//! Repositorio de servicios/puertos (`services`).

use std::net::IpAddr;

use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};

use mylan_core::{Protocol, Service, ServiceState};

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

/// Filtros para [`list_services`]. Todos opcionales; la combinación es AND.
#[derive(Debug, Clone, Default)]
pub struct ServiceFilters {
    /// Coincidencia exacta por `device_id`.
    pub device_id: Option<String>,
    /// Coincidencia exacta por `port`.
    pub port: Option<u16>,
    /// Coincidencia exacta por `protocol` (snake_case: `tcp`/`udp`).
    pub protocol: Option<String>,
    /// Substring case-insensitive sobre `service_name`.
    pub service: Option<String>,
}

/// Fila de servicio para reportes/exportación (`mylan services`,
/// `mylan export services`).
///
/// Join de `services` con `devices`: incluye `device_ip` (la `primary_ip` del
/// dispositivo) y `display_name`. Nombrado `ServiceExportRow` para no colisionar
/// con el privado [`ServiceRow`] (fila cruda de `services` sin join).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceExportRow {
    pub device_id: String,
    pub device_ip: Option<IpAddr>,
    pub display_name: Option<String>,
    pub protocol: Protocol,
    pub port: u16,
    pub service_name: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub banner: Option<String>,
    pub state: ServiceState,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

struct ServiceExportRowRaw {
    device_id: String,
    device_ip_raw: Option<String>,
    display_name: Option<String>,
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

impl ServiceExportRowRaw {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            device_id: row.get(0)?,
            device_ip_raw: row.get(1)?,
            display_name: row.get(2)?,
            protocol: row.get(3)?,
            port: row.get(4)?,
            service_name: row.get(5)?,
            product: row.get(6)?,
            version: row.get(7)?,
            banner: row.get(8)?,
            state: row.get(9)?,
            first_seen_at: row.get(10)?,
            last_seen_at: row.get(11)?,
        })
    }

    fn decode(self) -> DbResult<ServiceExportRow> {
        Ok(ServiceExportRow {
            device_id: self.device_id,
            device_ip: crate::codec::ip_from_db(self.device_ip_raw)?,
            display_name: self.display_name,
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

/// Lista servicios aplicando filtros AND, en join con `devices` para
/// `device_ip`/`display_name`.
///
/// Filtros (todos AND): `device_id` exacto, `port` exacto, `protocol` exacto
/// (snake_case) y `service` como substring case-insensitive sobre `service_name`.
/// El resultado se ordena por IP del dispositivo y puerto.
pub fn list_services(
    conn: &Connection,
    filters: &ServiceFilters,
) -> DbResult<Vec<ServiceExportRow>> {
    let mut sql = String::from(
        "SELECT s.device_id, d.primary_ip, d.display_name, s.protocol, s.port, \
         s.service_name, s.product, s.version, s.banner, s.state, \
         s.first_seen_at, s.last_seen_at \
         FROM services s \
         LEFT JOIN devices d ON d.id = s.device_id \
         WHERE 1 = 1",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(device_id) = &filters.device_id {
        sql.push_str(" AND s.device_id = ?");
        params.push(Box::new(device_id.clone()));
    }
    if let Some(port) = filters.port {
        sql.push_str(" AND s.port = ?");
        params.push(Box::new(i64::from(port)));
    }
    if let Some(protocol) = &filters.protocol {
        sql.push_str(" AND s.protocol = ?");
        params.push(Box::new(protocol.clone()));
    }
    if let Some(service) = &filters.service {
        sql.push_str(" AND LOWER(s.service_name) LIKE LOWER(?) ESCAPE '\\'");
        params.push(Box::new(like_pattern(service)));
    }
    sql.push_str(" ORDER BY d.primary_ip, s.port");

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| &**b).collect();
    let mut stmt = conn.prepare(&sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map(param_refs.as_slice(), ServiceExportRowRaw::from_row)
        .map_err(map_sqlite)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_sqlite)?.decode()?);
    }
    Ok(out)
}

/// Construye un patrón `LIKE` de substring: escapa `%`/`_`/`\` y envuelve con `%`.
fn like_pattern(substring: &str) -> String {
    let mut out = String::from('%');
    for c in substring.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('%');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::connect;
    use crate::device_repo::upsert_device;
    use crate::network_repo::upsert_network;
    use mylan_core::{Device, MacAddr, Protocol, Service, ServiceState};

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

    fn fixture_two_devices(dir: &std::path::Path) -> Connection {
        let conn = connect(dir.join("svc_list.db")).unwrap();
        upsert_network(
            &conn,
            &mylan_core::Network {
                id: "net-1".to_string(),
                name: "home".to_string(),
                cidr: "192.168.1.0/24".to_string(),
                gateway_ip: Some(ip("192.168.1.1")),
                dns_servers: vec![],
                created_at: "t0".to_string(),
                updated_at: "t0".to_string(),
            },
        )
        .unwrap();
        let mut d1 = Device::new("dev-1", "net-1", "t0");
        d1.primary_mac = Some(mac("aa:bb:cc:dd:ee:01"));
        d1.primary_ip = Some(ip("192.168.1.1"));
        d1.display_name = Some("router".to_string());
        upsert_device(&conn, &d1).unwrap();
        let mut d2 = Device::new("dev-2", "net-1", "t0");
        d2.primary_mac = Some(mac("aa:bb:cc:dd:ee:02"));
        d2.primary_ip = Some(ip("192.168.1.2"));
        d2.display_name = Some("nas".to_string());
        upsert_device(&conn, &d2).unwrap();
        conn
    }

    fn svc(device_id: &str, protocol: Protocol, port: u16, name: Option<&str>) -> Service {
        Service {
            id: format!("{device_id}-{protocol:?}-{port}"),
            device_id: device_id.to_string(),
            protocol,
            port,
            service_name: name.map(str::to_string),
            product: Some("nginx".to_string()),
            version: None,
            banner: Some("HTTP/1.1 200".to_string()),
            state: ServiceState::Open,
            first_seen_at: "t0".to_string(),
            last_seen_at: "t1".to_string(),
        }
    }

    #[test]
    fn list_services_no_filters_returns_all_ordered() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_two_devices(dir.path());
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 22, Some("ssh"))).unwrap();
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 80, Some("http"))).unwrap();
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 443, Some("https"))).unwrap();
        insert_service(&conn, &svc("dev-2", Protocol::Tcp, 80, Some("http"))).unwrap();
        insert_service(&conn, &svc("dev-2", Protocol::Udp, 161, Some("snmp"))).unwrap();

        let rows = list_services(&conn, &ServiceFilters::default()).unwrap();
        assert_eq!(rows.len(), 5);
        // Orden: device_ip asc (192.168.1.1 < 192.168.1.2 lexicográfico), luego port.
        assert_eq!(rows[0].device_ip, Some(ip("192.168.1.1")));
        assert_eq!(rows[0].port, 22);
        assert_eq!(rows[1].port, 80);
        assert_eq!(rows[2].port, 443);
        assert_eq!(rows[3].device_ip, Some(ip("192.168.1.2")));
        assert_eq!(rows[3].port, 80);
        assert_eq!(rows[4].port, 161);
        assert_eq!(rows[4].protocol, Protocol::Udp);
        // El join puebla device_ip y display_name.
        assert_eq!(rows[0].display_name.as_deref(), Some("router"));
        assert_eq!(rows[3].display_name.as_deref(), Some("nas"));
    }

    #[test]
    fn list_services_filter_device_and_port_and() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_two_devices(dir.path());
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 22, Some("ssh"))).unwrap();
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 80, Some("http"))).unwrap();
        insert_service(&conn, &svc("dev-2", Protocol::Tcp, 80, Some("http"))).unwrap();

        let filters = ServiceFilters {
            device_id: Some("dev-1".to_string()),
            port: Some(80),
            ..Default::default()
        };
        let rows = list_services(&conn, &filters).unwrap();
        assert_eq!(rows.len(), 1, "AND: device_id=dev-1 AND port=80");
        assert_eq!(rows[0].device_id, "dev-1");
        assert_eq!(rows[0].port, 80);
    }

    #[test]
    fn list_services_filter_protocol_and_service_substring() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_two_devices(dir.path());
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 80, Some("http"))).unwrap();
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 443, Some("https"))).unwrap();
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 22, Some("ssh"))).unwrap();
        insert_service(&conn, &svc("dev-2", Protocol::Udp, 161, Some("snmp"))).unwrap();
        insert_service(&conn, &svc("dev-2", Protocol::Tcp, 8080, Some("http-alt"))).unwrap();

        let filters = ServiceFilters {
            protocol: Some("tcp".to_string()),
            service: Some("HT".to_string()), // substring case-insensitive
            ..Default::default()
        };
        let rows = list_services(&conn, &filters).unwrap();
        // tcp + service_name contiene "ht" (case-insensitive): http, https, http-alt.
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|r| r.protocol == Protocol::Tcp));
        assert!(rows.iter().all(|r| {
            r.service_name
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains("ht")
        }));
    }

    #[test]
    fn list_services_filter_port_no_match() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_two_devices(dir.path());
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 80, Some("http"))).unwrap();

        let filters = ServiceFilters {
            port: Some(9999),
            ..Default::default()
        };
        let rows = list_services(&conn, &filters).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn list_services_protocol_filter_exact() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_two_devices(dir.path());
        insert_service(&conn, &svc("dev-1", Protocol::Tcp, 80, Some("http"))).unwrap();
        insert_service(&conn, &svc("dev-2", Protocol::Udp, 161, Some("snmp"))).unwrap();

        let filters = ServiceFilters {
            protocol: Some("udp".to_string()),
            ..Default::default()
        };
        let rows = list_services(&conn, &filters).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].port, 161);
        assert_eq!(rows[0].protocol, Protocol::Udp);
    }
}
