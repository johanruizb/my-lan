//! Repositorio de dispositivos (`devices` + `device_addresses`).
//!
//! Upsert por identidad estable (MAC no-cero > IP) dentro de la red, de forma
//! que re-escanear actualiza sin duplicar (P5). También ofrece listar
//! dispositivos de una red y obtener un dispositivo por su IP.

use std::net::IpAddr;

use rusqlite::{Connection, Row};

use mylan_core::{Confidence, Device, DeviceAddress};

use crate::codec::{enum_from_db, enum_to_db, ip_from_db, ip_to_db, mac_from_db, mac_to_db};
use crate::error::{map_sqlite, DbResult};

/// Resultado de un upsert de dispositivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsertOutcome {
    /// Se insertó un nuevo dispositivo.
    Inserted,
    /// Se actualizó un dispositivo existente (misma identidad).
    Updated,
}

/// Fila bruta leída de la DB, antes de decodificar tipos complejos.
struct DeviceRow {
    id: String,
    network_id: String,
    primary_mac: Option<String>,
    primary_ip: Option<String>,
    hostname: Option<String>,
    display_name: Option<String>,
    vendor: Option<String>,
    manufacturer: Option<String>,
    model: Option<String>,
    device_type: String,
    os_family: Option<String>,
    confidence: i64,
    first_seen_at: String,
    last_seen_at: String,
    is_trusted: i64,
    is_hidden: i64,
    notes: Option<String>,
}

impl DeviceRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            network_id: row.get(1)?,
            primary_mac: row.get(2)?,
            primary_ip: row.get(3)?,
            hostname: row.get(4)?,
            display_name: row.get(5)?,
            vendor: row.get(6)?,
            manufacturer: row.get(7)?,
            model: row.get(8)?,
            device_type: row.get(9)?,
            os_family: row.get(10)?,
            confidence: row.get(11)?,
            first_seen_at: row.get(12)?,
            last_seen_at: row.get(13)?,
            is_trusted: row.get(14)?,
            is_hidden: row.get(15)?,
            notes: row.get(16)?,
        })
    }

    fn decode(self) -> DbResult<Device> {
        Ok(Device {
            id: self.id,
            network_id: self.network_id,
            primary_mac: mac_from_db(self.primary_mac)?,
            primary_ip: ip_from_db(self.primary_ip)?,
            hostname: self.hostname,
            display_name: self.display_name,
            vendor: self.vendor,
            manufacturer: self.manufacturer,
            model: self.model,
            device_type: enum_from_db(&self.device_type)?,
            os_family: self.os_family,
            confidence: Confidence::new(u8::try_from(self.confidence).unwrap_or(0)),
            first_seen_at: self.first_seen_at,
            last_seen_at: self.last_seen_at,
            is_trusted: self.is_trusted != 0,
            is_hidden: self.is_hidden != 0,
            notes: self.notes,
        })
    }
}

const SELECT_COLS: &str =
    "id, network_id, primary_mac, primary_ip, hostname, display_name, vendor, manufacturer, \
     model, device_type, os_family, confidence, first_seen_at, last_seen_at, is_trusted, \
     is_hidden, notes";

/// Busca el `id` de un dispositivo existente por identidad estable.
///
/// Intenta primero por MAC no-cero y, si no hay, por IP, ambas dentro del
/// `network_id`. Devuelve `None` si no hay coincidencia.
fn find_existing_id(conn: &Connection, device: &Device) -> DbResult<Option<String>> {
    let mac_ok = device.primary_mac.is_some_and(|m| !m.is_zero());
    let candidates: [&str; 2] = [
        "SELECT id FROM devices WHERE network_id = ?1 AND primary_mac = ?2",
        "SELECT id FROM devices WHERE network_id = ?1 AND primary_ip = ?2",
    ];
    let key: Option<String> = if mac_ok {
        device.primary_mac.map(|m| m.to_string())
    } else {
        ip_to_db(device.primary_ip)
    };
    let Some(key) = key else { return Ok(None) };
    let stmt_idx = if mac_ok { 0 } else { 1 };
    let result = conn
        .query_row(
            candidates[stmt_idx],
            rusqlite::params![device.network_id, key],
            |row| row.get::<_, String>(0),
        )
        .map(Some);
    match result {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_sqlite(e)),
    }
}

/// Inserta o actualiza un dispositivo por identidad estable (P5).
///
/// Si existe un dispositivo con la misma MAC (o IP, en fallback) dentro de la
/// red, actualiza sus columnas mutables preservando `first_seen_at`; en caso
/// contrario inserta una nueva fila. Devuelve si fue inserción o actualización.
pub fn upsert_device(conn: &Connection, device: &Device) -> DbResult<UpsertOutcome> {
    if let Some(existing_id) = find_existing_id(conn, device)? {
        conn.execute(
            "UPDATE devices SET
               primary_mac = ?1, primary_ip = ?2, hostname = ?3, display_name = ?4,
               vendor = ?5, manufacturer = ?6, model = ?7, device_type = ?8,
               os_family = ?9, confidence = ?10, last_seen_at = ?11,
               is_trusted = ?12, is_hidden = ?13, notes = ?14
             WHERE id = ?15",
            rusqlite::params![
                mac_to_db(device.primary_mac),
                ip_to_db(device.primary_ip),
                device.hostname,
                device.display_name,
                device.vendor,
                device.manufacturer,
                device.model,
                enum_to_db(&device.device_type)?,
                device.os_family,
                i64::from(device.confidence.score()),
                device.last_seen_at,
                device.is_trusted,
                device.is_hidden,
                device.notes,
                existing_id,
            ],
        )
        .map_err(map_sqlite)?;
        Ok(UpsertOutcome::Updated)
    } else {
        conn.execute(
            "INSERT INTO devices (
               id, network_id, primary_mac, primary_ip, hostname, display_name, vendor,
               manufacturer, model, device_type, os_family, confidence, first_seen_at,
               last_seen_at, is_trusted, is_hidden, notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            rusqlite::params![
                device.id,
                device.network_id,
                mac_to_db(device.primary_mac),
                ip_to_db(device.primary_ip),
                device.hostname,
                device.display_name,
                device.vendor,
                device.manufacturer,
                device.model,
                enum_to_db(&device.device_type)?,
                device.os_family,
                i64::from(device.confidence.score()),
                device.first_seen_at,
                device.last_seen_at,
                device.is_trusted,
                device.is_hidden,
                device.notes,
            ],
        )
        .map_err(map_sqlite)?;
        Ok(UpsertOutcome::Inserted)
    }
}

/// Lista todos los dispositivos de una red, ordenados por `last_seen_at` desc.
pub fn list_devices(conn: &Connection, network_id: &str) -> DbResult<Vec<Device>> {
    let sql = format!(
        "SELECT {SELECT_COLS} FROM devices WHERE network_id = ?1 ORDER BY last_seen_at DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(map_sqlite)?;
    let rows = stmt
        .query_map([network_id], DeviceRow::from_row)
        .map_err(map_sqlite)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_sqlite)?.decode()?);
    }
    Ok(out)
}

/// Obtiene un dispositivo por su IP primaria dentro de una red.
pub fn get_device_by_ip(
    conn: &Connection,
    network_id: &str,
    ip_addr: IpAddr,
) -> DbResult<Option<Device>> {
    let sql = format!(
        "SELECT {SELECT_COLS} FROM devices WHERE network_id = ?1 AND primary_ip = ?2 LIMIT 1"
    );
    let result = conn.query_row(
        &sql,
        rusqlite::params![network_id, ip_addr.to_string()],
        DeviceRow::from_row,
    );
    match result {
        Ok(row) => Ok(Some(row.decode()?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_sqlite(e)),
    }
}

/// Inserta una dirección histórica de un dispositivo.
pub fn insert_device_address(conn: &Connection, addr: &DeviceAddress) -> DbResult<()> {
    conn.execute(
        "INSERT INTO device_addresses (
           id, device_id, ip, mac, interface_name, first_seen_at, last_seen_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            addr.id,
            addr.device_id,
            ip_to_db(addr.ip),
            mac_to_db(addr.mac),
            addr.interface_name,
            addr.first_seen_at,
            addr.last_seen_at,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::connect;
    use crate::error::DbError;
    use crate::network_repo::upsert_network;
    use mylan_core::{DeviceType, MacAddr};

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }
    fn mac(s: &str) -> MacAddr {
        MacAddr::parse(s).unwrap()
    }

    fn fixture_conn(dir: &std::path::Path, name: &str) -> Connection {
        let conn = connect(dir.join(format!("{name}.db"))).unwrap();
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

    fn device(id: &str, mac_addr: Option<MacAddr>, ip_addr: Option<IpAddr>, now: &str) -> Device {
        let mut d = Device::new(id, "net-1", now);
        d.primary_mac = mac_addr;
        d.primary_ip = ip_addr;
        d
    }

    #[test]
    fn upsert_inserts_then_updates_no_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "dup");
        let m = mac("aa:bb:cc:dd:ee:ff");
        // First insert.
        let mut d = device(
            "dev-1",
            Some(m),
            Some(ip("192.168.1.5")),
            "2026-06-27T00:00:00Z",
        );
        assert_eq!(upsert_device(&conn, &d).unwrap(), UpsertOutcome::Inserted);
        // Same MAC, new IP (DHCP) and later timestamp -> update, not insert.
        d.primary_ip = Some(ip("192.168.1.42"));
        d.last_seen_at = "2026-06-27T01:00:00Z".to_string();
        assert_eq!(upsert_device(&conn, &d).unwrap(), UpsertOutcome::Updated);
        // Different id supplied but same identity -> still update, keeps original id+first_seen.
        let mut d2 = device(
            "dev-999",
            Some(m),
            Some(ip("192.168.1.42")),
            "2026-06-27T02:00:00Z",
        );
        d2.hostname = Some("nas.local".to_string());
        assert_eq!(upsert_device(&conn, &d2).unwrap(), UpsertOutcome::Updated);

        let all = list_devices(&conn, "net-1").unwrap();
        assert_eq!(all.len(), 1, "no duplicate device");
        assert_eq!(all[0].id, "dev-1"); // original id preserved
        assert_eq!(all[0].primary_ip, Some(ip("192.168.1.42")));
        assert_eq!(all[0].hostname.as_deref(), Some("nas.local"));
        assert_eq!(all[0].first_seen_at, "2026-06-27T00:00:00Z"); // preserved
        assert_eq!(all[0].last_seen_at, "2026-06-27T02:00:00Z");
    }

    #[test]
    fn upsert_by_ip_when_no_mac() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "ipid");
        let mut d = device(
            "dev-ip",
            None,
            Some(ip("192.168.1.9")),
            "2026-06-27T00:00:00Z",
        );
        assert_eq!(upsert_device(&conn, &d).unwrap(), UpsertOutcome::Inserted);
        d.hostname = Some("laptop".to_string());
        d.last_seen_at = "2026-06-27T03:00:00Z".to_string();
        assert_eq!(upsert_device(&conn, &d).unwrap(), UpsertOutcome::Updated);
        assert_eq!(list_devices(&conn, "net-1").unwrap().len(), 1);
    }

    #[test]
    fn get_device_by_ip_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "byip");
        let d = device(
            "dev-x",
            Some(mac("11:22:33:44:55:66")),
            Some(ip("192.168.1.7")),
            "2026-06-27T00:00:00Z",
        );
        upsert_device(&conn, &d).unwrap();
        let got = get_device_by_ip(&conn, "net-1", ip("192.168.1.7"))
            .unwrap()
            .expect("found");
        assert_eq!(got.id, "dev-x");
        assert!(get_device_by_ip(&conn, "net-1", ip("192.168.1.99"))
            .unwrap()
            .is_none());
    }

    #[test]
    fn preserves_device_type_and_confidence() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "cls");
        let mut d = device(
            "dev-c",
            Some(mac("aa:bb:cc:00:00:01")),
            Some(ip("192.168.1.20")),
            "2026-06-27T00:00:00Z",
        );
        d.apply_classification(DeviceType::Camera, Confidence::new(82));
        upsert_device(&conn, &d).unwrap();
        let got = get_device_by_ip(&conn, "net-1", ip("192.168.1.20"))
            .unwrap()
            .unwrap();
        assert_eq!(got.device_type, DeviceType::Camera);
        assert_eq!(got.confidence, Confidence::new(82));
    }

    #[test]
    fn foreign_key_rejects_orphan_device() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(dir.path().join("fk.db")).unwrap();
        let d = device(
            "dev-orphan",
            Some(mac("00:11:22:33:44:55")),
            Some(ip("10.0.0.5")),
            "2026-06-27T00:00:00Z",
        );
        // No network 'net-x' exists -> FK violation (foreign_keys ON).
        let mut orphan = d.clone();
        orphan.network_id = "net-x".to_string();
        let res = upsert_device(&conn, &orphan);
        assert!(
            matches!(res, Err(DbError::Sqlite(_))),
            "FK should reject: {res:?}"
        );
    }

    #[test]
    fn insert_device_address_persists() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "addr");
        let d = device(
            "dev-a",
            Some(mac("aa:bb:cc:00:00:02")),
            Some(ip("192.168.1.30")),
            "2026-06-27T00:00:00Z",
        );
        upsert_device(&conn, &d).unwrap();
        let addr = DeviceAddress {
            id: "addr-1".to_string(),
            device_id: "dev-a".to_string(),
            ip: Some(ip("192.168.1.30")),
            mac: Some(mac("aa:bb:cc:00:00:02")),
            interface_name: Some("enp37s0".to_string()),
            first_seen_at: "2026-06-27T00:00:00Z".to_string(),
            last_seen_at: "2026-06-27T00:00:10Z".to_string(),
        };
        insert_device_address(&conn, &addr).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM device_addresses WHERE device_id = ?1",
                ["dev-a"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
