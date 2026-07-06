//! Repositorio de dispositivos (`devices` + `device_addresses`).
//!
//! Upsert por identidad estable (MAC no-cero > IP) dentro de la red, de forma
//! que re-escanear actualiza sin duplicar (P5). También ofrece listar
//! dispositivos de una red y obtener un dispositivo por su IP.

use std::net::IpAddr;

use rusqlite::{params_from_iter, Connection, Row, ToSql};

use mylan_core::{Confidence, Device, DeviceAddress, DeviceType};

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
    is_online: i64,
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
            is_online: row.get(17)?,
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
            is_online: self.is_online != 0,
        })
    }
}

const SELECT_COLS: &str =
    "id, network_id, primary_mac, primary_ip, hostname, display_name, vendor, manufacturer, \
     model, device_type, os_family, confidence, first_seen_at, last_seen_at, is_trusted, \
     is_hidden, notes, is_online";

/// Ejecuta una consulta de un único `id` opcional.
fn query_opt_id(
    conn: &Connection,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
) -> DbResult<Option<String>> {
    match conn.query_row(sql, params, |row| row.get::<_, String>(0)) {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_sqlite(e)),
    }
}

/// Busca el `id` de un dispositivo existente por identidad estable.
///
/// Con MAC no-cero: coincide por MAC y, si no hay, **promueve** una fila previa
/// solo-IP de la misma IP (un host visto antes sin MAC y ahora con ella es el
/// mismo dispositivo — evita el seam de duplicación cross-scan, P5). Sin MAC:
/// coincide por IP. Devuelve `None` si no hay coincidencia.
fn find_existing_id(conn: &Connection, device: &Device) -> DbResult<Option<String>> {
    let mac_ok = device.primary_mac.is_some_and(|m| !m.is_zero());
    if mac_ok {
        let mac = device.primary_mac.map(|m| m.to_string());
        if let Some(mac) = mac {
            if let Some(id) = query_opt_id(
                conn,
                "SELECT id FROM devices WHERE network_id = ?1 AND primary_mac = ?2",
                rusqlite::params![device.network_id, mac],
            )? {
                return Ok(Some(id));
            }
        }
        // Sin fila por MAC: promueve una fila solo-IP de la misma IP (si la hay).
        if let Some(ip) = ip_to_db(device.primary_ip) {
            return query_opt_id(
                conn,
                "SELECT id FROM devices WHERE network_id = ?1 AND primary_ip = ?2 \
                 AND (primary_mac IS NULL OR primary_mac = '')",
                rusqlite::params![device.network_id, ip],
            );
        }
        Ok(None)
    } else if let Some(ip) = ip_to_db(device.primary_ip) {
        query_opt_id(
            conn,
            "SELECT id FROM devices WHERE network_id = ?1 AND primary_ip = ?2",
            rusqlite::params![device.network_id, ip],
        )
    } else {
        Ok(None)
    }
}

/// Lee un dispositivo por su `id` interno.
fn get_device_by_id(conn: &Connection, id: &str) -> DbResult<Option<Device>> {
    let sql = format!("SELECT {SELECT_COLS} FROM devices WHERE id = ?1 LIMIT 1");
    match conn.query_row(&sql, [id], DeviceRow::from_row) {
        Ok(row) => Ok(Some(row.decode()?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_sqlite(e)),
    }
}

/// Lee un dispositivo por su `id` (público, para `GET /api/v1/devices/:id`).
///
/// Wrapper público sobre el helper interno [`get_device_by_id`]; incluye `is_online`
/// (Step 1) en el `Device` devuelto. `None` si no existe.
pub fn get_device(conn: &Connection, id: &str) -> DbResult<Option<Device>> {
    get_device_by_id(conn, id)
}

/// Funde la observación `incoming` de este escaneo sobre la fila `existing`
/// preservando el conocimiento acumulado (P5): un re-escaneo más pobre **no**
/// borra datos previos.
///
/// - MAC: ancla estable; solo se fija si faltaba (nunca se sustituye por `None`).
/// - IP: se actualiza a la más reciente observada (DHCP) si `incoming` la trae.
/// - hostname/vendor/manufacturer/model/os_family/display_name: se conservan si
///   `incoming` no aporta valor (`COALESCE` hacia el existente).
/// - device_type/confidence: precedencia por confianza (igual que
///   `Device::apply_classification`): gana la mayor confianza; con `confidence`
///   igual se permite un cambio lateral a un tipo no-`Unknown` (p.ej.
///   camera@75 → nas@75), y un `Unknown` entrante nunca degrada un tipo ya
///   clasificado. Nunca baja el `confidence` absoluto.
/// - is_trusted/is_hidden/notes: campos del usuario; el pipeline no los fija, se
///   conservan tal cual.
/// - last_seen_at: siempre el del escaneo actual.
fn merge_for_update(existing: &Device, incoming: &Device) -> Device {
    let (device_type, confidence) = if incoming.confidence >= existing.confidence
        && incoming.device_type != DeviceType::Unknown
    {
        (incoming.device_type, incoming.confidence)
    } else {
        (existing.device_type, existing.confidence)
    };
    Device {
        id: existing.id.clone(),
        network_id: existing.network_id.clone(),
        primary_mac: existing.primary_mac.or(incoming.primary_mac),
        primary_ip: incoming.primary_ip.or(existing.primary_ip),
        hostname: incoming
            .hostname
            .clone()
            .or_else(|| existing.hostname.clone()),
        display_name: existing.display_name.clone(),
        vendor: incoming.vendor.clone().or_else(|| existing.vendor.clone()),
        manufacturer: incoming
            .manufacturer
            .clone()
            .or_else(|| existing.manufacturer.clone()),
        model: incoming.model.clone().or_else(|| existing.model.clone()),
        device_type,
        os_family: incoming
            .os_family
            .clone()
            .or_else(|| existing.os_family.clone()),
        confidence,
        first_seen_at: existing.first_seen_at.clone(),
        last_seen_at: incoming.last_seen_at.clone(),
        is_trusted: existing.is_trusted,
        is_hidden: existing.is_hidden,
        notes: existing.notes.clone(),
        is_online: incoming.is_online || existing.is_online,
    }
}

/// Inserta o actualiza un dispositivo por identidad estable (P5).
///
/// Si existe un dispositivo con la misma MAC (o IP, en fallback) dentro de la
/// red, actualiza sus columnas mutables preservando `first_seen_at`; en caso
/// contrario inserta una nueva fila. Devuelve si fue inserción o actualización.
pub fn upsert_device(conn: &Connection, device: &Device) -> DbResult<UpsertOutcome> {
    if let Some(existing_id) = find_existing_id(conn, device)? {
        // Funde sobre la fila existente para no borrar conocimiento acumulado
        // en un re-escaneo más pobre (P5). Si la fila desapareció entre la
        // búsqueda y ahora (no debería), cae a los valores entrantes.
        let merged = match get_device_by_id(conn, &existing_id)? {
            Some(existing) => merge_for_update(&existing, device),
            None => device.clone(),
        };
        conn.execute(
            "UPDATE devices SET
               primary_mac = ?1, primary_ip = ?2, hostname = ?3, display_name = ?4,
               vendor = ?5, manufacturer = ?6, model = ?7, device_type = ?8,
               os_family = ?9, confidence = ?10, last_seen_at = ?11,
               is_trusted = ?12, is_hidden = ?13, notes = ?14, is_online = ?15
             WHERE id = ?16",
            rusqlite::params![
                mac_to_db(merged.primary_mac),
                ip_to_db(merged.primary_ip),
                merged.hostname,
                merged.display_name,
                merged.vendor,
                merged.manufacturer,
                merged.model,
                enum_to_db(&merged.device_type)?,
                merged.os_family,
                i64::from(merged.confidence.score()),
                merged.last_seen_at,
                merged.is_trusted,
                merged.is_hidden,
                merged.notes,
                merged.is_online,
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
               last_seen_at, is_trusted, is_hidden, notes, is_online
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
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
                device.is_trusted || (device.device_type == DeviceType::Router),
                device.is_hidden,
                device.notes,
                device.is_online,
            ],
        )
        .map_err(map_sqlite)?;
        Ok(UpsertOutcome::Inserted)
    }
}

/// Actualiza parcialmente los campos editables por el usuario (`display_name`,
/// `is_trusted`, `notes`) de un dispositivo por su `id` (UUID `String` de
/// `Device.id`, NO `i64`).
///
/// Solo fija los campos pasados como `Some`; `None` significa "no tocar". Así
/// se preservan `first_seen_at`, `device_type`, `confidence`, `primary_mac`,
/// `primary_ip`, `hostname`, `is_online`, `is_hidden`, `network_id`,
/// `manufacturer`, `model`, `os_family`, `vendor` — ninguna otra columna entra
/// en el `SET`.
///
/// Si ningún campo es `Some`, retorna `Ok(())` sin ejecutar `UPDATE`.
///
/// El SQL se construye con fragmentos constantes (`Vec<&'static str>`) y valores
/// bind (`Vec<Box<dyn ToSql>>`) ejecutados vía `rusqlite::params_from_iter`, de
/// forma que los valores nunca se interpolan en el string (evita SQL injection)
/// y los placeholders `?` no se calculan a mano (evita off-by-one).
pub fn update_device_fields(
    conn: &Connection,
    id: &str,
    display_name: Option<&str>,
    is_trusted: Option<bool>,
    notes: Option<&str>,
) -> DbResult<()> {
    let mut set_fragments: Vec<&'static str> = Vec::new();
    let mut values: Vec<Box<dyn ToSql>> = Vec::new();
    if let Some(name) = display_name {
        set_fragments.push("display_name = ?");
        values.push(Box::new(name.to_string()));
    }
    if let Some(trusted) = is_trusted {
        set_fragments.push("is_trusted = ?");
        values.push(Box::new(trusted));
    }
    if let Some(note) = notes {
        set_fragments.push("notes = ?");
        values.push(Box::new(note.to_string()));
    }
    if set_fragments.is_empty() {
        return Ok(());
    }
    let sql = format!(
        "UPDATE devices SET {} WHERE id = ?",
        set_fragments.join(", ")
    );
    values.push(Box::new(id.to_string()));
    conn.execute(&sql, params_from_iter(values))
        .map_err(map_sqlite)?;
    Ok(())
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
    fn incomplete_rescan_preserves_accumulated_fields() {
        // Escaneo 1: ARP (MAC+IP) + mDNS (hostname) + clasificación cámara.
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "preserve");
        let m = mac("aa:bb:cc:dd:ee:ff");
        let mut d1 = device("dev-1", Some(m), Some(ip("192.168.1.5")), "t0");
        d1.hostname = Some("nas.local".to_string());
        d1.vendor = Some("Synology".to_string());
        d1.apply_classification(DeviceType::Nas, Confidence::new(80));
        upsert_device(&conn, &d1).unwrap();
        // El usuario marca el dispositivo como confiable y le pone una nota.
        conn.execute(
            "UPDATE devices SET is_trusted = 1, notes = 'mi NAS' WHERE id = 'dev-1'",
            [],
        )
        .unwrap();

        // Escaneo 2 pobre: solo ARP responde (mDNS callado, sin vendor/hostname,
        // sin clasificación). NO debe borrar lo aprendido.
        let mut d2 = device("dev-x", Some(m), Some(ip("192.168.1.5")), "t1");
        d2.last_seen_at = "t1".to_string();
        assert_eq!(upsert_device(&conn, &d2).unwrap(), UpsertOutcome::Updated);

        let got = get_device_by_ip(&conn, "net-1", ip("192.168.1.5"))
            .unwrap()
            .unwrap();
        assert_eq!(
            got.hostname.as_deref(),
            Some("nas.local"),
            "hostname preservado"
        );
        assert_eq!(got.vendor.as_deref(), Some("Synology"), "vendor preservado");
        assert_eq!(got.device_type, DeviceType::Nas, "clasificación preservada");
        assert_eq!(got.confidence, Confidence::new(80));
        assert_eq!(got.primary_mac, Some(m), "MAC ancla preservada");
        assert!(got.is_trusted, "campo de usuario preservado");
        assert_eq!(got.notes.as_deref(), Some("mi NAS"));
        assert_eq!(got.last_seen_at, "t1", "last_seen actualizado");
        assert_eq!(got.first_seen_at, "t0", "first_seen preservado");
    }

    #[test]
    fn rescan_only_raises_classification_never_downgrades() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "cls2");
        let m = mac("aa:bb:cc:dd:ee:01");
        let mut d1 = device("dev-1", Some(m), Some(ip("192.168.1.6")), "t0");
        d1.apply_classification(DeviceType::Camera, Confidence::new(75));
        upsert_device(&conn, &d1).unwrap();
        // Re-escaneo con clasificación de MENOR confianza: se ignora.
        let mut d2 = device("dev-2", Some(m), Some(ip("192.168.1.6")), "t1");
        d2.apply_classification(DeviceType::Iot, Confidence::new(40));
        upsert_device(&conn, &d2).unwrap();
        let got = get_device_by_ip(&conn, "net-1", ip("192.168.1.6"))
            .unwrap()
            .unwrap();
        assert_eq!(got.device_type, DeviceType::Camera);
        assert_eq!(got.confidence, Confidence::new(75));
    }

    #[test]
    fn ip_only_row_promoted_to_mac_no_duplicate() {
        // Escaneo 1: host visto solo por IP (TCP-ping, ARP frío) -> fila sin MAC.
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "promote");
        let d1 = device("dev-ip", None, Some(ip("192.168.1.8")), "t0");
        assert_eq!(upsert_device(&conn, &d1).unwrap(), UpsertOutcome::Inserted);
        // Escaneo 2: el mismo host ahora con MAC (ARP caliente). Debe PROMOVER la
        // fila solo-IP, no crear una segunda (P5 cross-scan).
        let m = mac("aa:bb:cc:dd:ee:02");
        let d2 = device("dev-mac", Some(m), Some(ip("192.168.1.8")), "t1");
        assert_eq!(upsert_device(&conn, &d2).unwrap(), UpsertOutcome::Updated);

        let all = list_devices(&conn, "net-1").unwrap();
        assert_eq!(all.len(), 1, "promovido, no duplicado");
        assert_eq!(all[0].id, "dev-ip", "id original preservado");
        assert_eq!(all[0].primary_mac, Some(m), "MAC promovida a la fila");
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

    #[test]
    fn is_online_round_trips_via_upsert() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "online");
        let m = mac("aa:bb:cc:dd:ee:03");
        // Nuevo device: `Device::new` fija `is_online = true`.
        let d = device("dev-1", Some(m), Some(ip("192.168.1.11")), "t0");
        assert_eq!(upsert_device(&conn, &d).unwrap(), UpsertOutcome::Inserted);
        let got = get_device_by_ip(&conn, "net-1", ip("192.168.1.11"))
            .unwrap()
            .unwrap();
        assert!(got.is_online, "nuevo device online");

        // El motor de diff marca el device offline directamente en la DB
        // (fuera de `upsert_device`).
        conn.execute("UPDATE devices SET is_online = 0 WHERE id = 'dev-1'", [])
            .unwrap();
        let offline = get_device_by_ip(&conn, "net-1", ip("192.168.1.11"))
            .unwrap()
            .unwrap();
        assert!(!offline.is_online, "marcado offline por el diff");

        // Re-escaneo: el device se ve de nuevo (`incoming.is_online = true` por
        // `Device::new`); `merge_for_update` aplica OR → restaura online.
        let d2 = device("dev-1", Some(m), Some(ip("192.168.1.11")), "t1");
        assert_eq!(upsert_device(&conn, &d2).unwrap(), UpsertOutcome::Updated);
        let back = get_device_by_ip(&conn, "net-1", ip("192.168.1.11"))
            .unwrap()
            .unwrap();
        assert!(back.is_online, "OR con incoming true restaura online");
        assert_eq!(back.last_seen_at, "t1");
    }

    #[test]
    fn update_device_fields_partial_display_name_only() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "udf_dname");
        let m = mac("aa:bb:cc:dd:ee:10");
        let mut d = device("dev-1", Some(m), Some(ip("192.168.1.50")), "t0");
        d.display_name = Some("original".to_string());
        d.notes = Some("nota inicial".to_string());
        upsert_device(&conn, &d).unwrap();

        update_device_fields(&conn, "dev-1", Some("nuevo nombre"), None, None).unwrap();

        let got = get_device(&conn, "dev-1").unwrap().unwrap();
        assert_eq!(got.display_name.as_deref(), Some("nuevo nombre"));
        assert_eq!(
            got.notes.as_deref(),
            Some("nota inicial"),
            "notes preservado"
        );
        assert!(!got.is_trusted, "is_trusted preservado");
        assert_eq!(got.first_seen_at, "t0", "first_seen_at preservado");
    }

    #[test]
    fn update_device_fields_all_none_no_change() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "udf_none");
        let m = mac("aa:bb:cc:dd:ee:11");
        let mut d = device("dev-1", Some(m), Some(ip("192.168.1.51")), "t0");
        d.display_name = Some("original".to_string());
        d.notes = Some("nota inicial".to_string());
        upsert_device(&conn, &d).unwrap();

        update_device_fields(&conn, "dev-1", None, None, None).unwrap();

        let got = get_device(&conn, "dev-1").unwrap().unwrap();
        assert_eq!(got.display_name.as_deref(), Some("original"));
        assert_eq!(got.notes.as_deref(), Some("nota inicial"));
        assert!(!got.is_trusted);
    }

    #[test]
    fn update_device_fields_is_trusted_toggle_false_to_true() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "udf_toggle");
        let m = mac("aa:bb:cc:dd:ee:12");
        let d = device("dev-1", Some(m), Some(ip("192.168.1.52")), "t0");
        upsert_device(&conn, &d).unwrap();
        // Insert path: device.is_trusted=false y no-Router => is_trusted=false.
        let got = get_device(&conn, "dev-1").unwrap().unwrap();
        assert!(!got.is_trusted, "insert deja is_trusted=false");

        update_device_fields(&conn, "dev-1", None, Some(true), None).unwrap();
        let got = get_device(&conn, "dev-1").unwrap().unwrap();
        assert!(got.is_trusted, "toggle false->true");

        update_device_fields(&conn, "dev-1", None, Some(false), None).unwrap();
        let got = get_device(&conn, "dev-1").unwrap().unwrap();
        assert!(!got.is_trusted, "toggle true->false");
    }

    #[test]
    fn update_device_fields_notes_only_display_name_none() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "udf_notes");
        let m = mac("aa:bb:cc:dd:ee:13");
        let mut d = device("dev-1", Some(m), Some(ip("192.168.1.53")), "t0");
        d.display_name = Some("original".to_string());
        upsert_device(&conn, &d).unwrap();

        update_device_fields(&conn, "dev-1", None, None, Some("mi nota nueva")).unwrap();

        let got = get_device(&conn, "dev-1").unwrap().unwrap();
        assert_eq!(got.notes.as_deref(), Some("mi nota nueva"));
        assert_eq!(
            got.display_name.as_deref(),
            Some("original"),
            "display_name preservado (None no toca)"
        );
    }

    #[test]
    fn router_insert_marks_trusted() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "router_trust");
        let m = mac("aa:bb:cc:dd:ee:20");
        let mut d = device("dev-router", Some(m), Some(ip("192.168.1.1")), "t0");
        d.device_type = DeviceType::Router;
        d.is_trusted = false; // explícito: el pipeline no lo marca
        assert_eq!(upsert_device(&conn, &d).unwrap(), UpsertOutcome::Inserted);

        let got = get_device(&conn, "dev-router").unwrap().unwrap();
        assert!(got.is_trusted, "router insert => is_trusted=true");
    }

    #[test]
    fn non_router_insert_respects_device_is_trusted() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "phone_laptop_trust");
        let m = mac("aa:bb:cc:dd:ee:21");
        let mut d = device("dev-phone", Some(m), Some(ip("192.168.1.54")), "t0");
        d.device_type = DeviceType::Phone;
        d.is_trusted = false;
        assert_eq!(upsert_device(&conn, &d).unwrap(), UpsertOutcome::Inserted);
        let got = get_device(&conn, "dev-phone").unwrap().unwrap();
        assert!(!got.is_trusted, "phone false => false (no forzado)");

        let m2 = mac("aa:bb:cc:dd:ee:22");
        let mut d2 = device("dev-laptop", Some(m2), Some(ip("192.168.1.55")), "t0");
        d2.device_type = DeviceType::Laptop;
        d2.is_trusted = true;
        assert_eq!(upsert_device(&conn, &d2).unwrap(), UpsertOutcome::Inserted);
        let got2 = get_device(&conn, "dev-laptop").unwrap().unwrap();
        assert!(
            got2.is_trusted,
            "laptop true => true (respeta device.is_trusted)"
        );
    }

    #[test]
    fn is_trusted_preserved_after_rescan_untrust() {
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "preserve_untrust");
        let m = mac("aa:bb:cc:dd:ee:23");
        let mut d = device("dev-1", Some(m), Some(ip("192.168.1.56")), "t0");
        d.device_type = DeviceType::Phone;
        upsert_device(&conn, &d).unwrap();
        // Usuario marca confiable y luego lo desmarca directamente en la DB.
        conn.execute("UPDATE devices SET is_trusted = 1 WHERE id = 'dev-1'", [])
            .unwrap();
        conn.execute("UPDATE devices SET is_trusted = 0 WHERE id = 'dev-1'", [])
            .unwrap();

        // Re-escaneo: misma MAC => UPDATE path (merge_for_update preserva
        // existing.is_trusted=false, línea 219).
        let mut d2 = device("dev-1", Some(m), Some(ip("192.168.1.56")), "t1");
        d2.device_type = DeviceType::Phone;
        assert_eq!(upsert_device(&conn, &d2).unwrap(), UpsertOutcome::Updated);

        let got = get_device(&conn, "dev-1").unwrap().unwrap();
        assert!(
            !got.is_trusted,
            "is_trusted=false del usuario preservado por merge_for_update"
        );
    }

    #[test]
    fn non_router_then_router_rescan_preserves_existing_trust() {
        // Edge case: un dispositivo se inserta como Phone (is_trusted=false).
        // En un re-escaneo, enrich_device lo reclasifica como Router (p.ej. su IP
        // cambió al gateway). El UPDATE path de upsert_device preserva
        // existing.is_trusted=false (merge_for_update, línea 219), por lo que NO
        // se marca confiable automáticamente — "solo en insert" es consistente.
        // El usuario puede marcarlo confiable manualmente vía update_device_fields.
        let dir = tempfile::tempdir().unwrap();
        let conn = fixture_conn(dir.path(), "router_rescan");
        let m = mac("aa:bb:cc:dd:ee:24");
        let mut d1 = device("dev-1", Some(m), Some(ip("192.168.1.57")), "t0");
        d1.device_type = DeviceType::Phone;
        d1.is_trusted = false;
        assert_eq!(upsert_device(&conn, &d1).unwrap(), UpsertOutcome::Inserted);

        // Re-escaneo: misma MAC, ahora clasificado como Router (simula IP cambió
        // al gateway). El INSERT-path `|| Router` NO aplica: es UPDATE path.
        let mut d2 = device("dev-1", Some(m), Some(ip("192.168.1.57")), "t1");
        d2.device_type = DeviceType::Router;
        d2.is_trusted = false; // incoming no fuerza true
        assert_eq!(upsert_device(&conn, &d2).unwrap(), UpsertOutcome::Updated);

        let got = get_device(&conn, "dev-1").unwrap().unwrap();
        assert!(
            !got.is_trusted,
            "router descubierto tras cambio de IP NO se marca confiable \
             (UPDATE path preserva existing.is_trusted=false)"
        );
    }
}
