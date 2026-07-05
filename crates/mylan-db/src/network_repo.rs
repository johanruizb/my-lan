//! Repositorio de redes (`networks`).
//!
//! Upsert por `id` (clave primaria). `dns_servers` se serializa como un array
//! JSON de IPs en la columna `TEXT`.

use rusqlite::Connection;

use mylan_core::Network;

use crate::codec::{ip_from_db, ip_to_db};
use crate::error::{map_sqlite, DbResult};

/// Inserta o actualiza una red por su `id` (upsert `ON CONFLICT`).
///
/// **Precedencia de override del nombre** (atómica, sin SELECT-then-UPDATE): si
/// la fila existente tiene `name_source = 'user'` (etiqueta editada por el
/// usuario), el `name`/`name_source` NO se sobrescriben con el valor auto-detectado
/// (`net.name`, que el llamador resuelve a SSID o CIDR-fallback); en caso contrario
/// se actualizan a `excluded.name` / `'auto'`. `name_source` no es un campo del
/// struct `Network` (es columna DB pura): en INSERT cae al `DEFAULT 'auto'` y en
/// UPDATE-auto se fija con el literal SQL `'auto'`.
pub fn upsert_network(conn: &Connection, net: &Network) -> DbResult<()> {
    let dns = serde_json::to_string(&net.dns_servers)?;
    conn.execute(
        "INSERT INTO networks (id, name, cidr, gateway_ip, dns_servers, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
           name = CASE WHEN networks.name_source = 'user'
                       THEN networks.name ELSE excluded.name END,
           name_source = CASE WHEN networks.name_source = 'user'
                              THEN networks.name_source ELSE 'auto' END,
           cidr = excluded.cidr,
           gateway_ip = excluded.gateway_ip,
           dns_servers = excluded.dns_servers,
           updated_at = excluded.updated_at",
        rusqlite::params![
            net.id,
            net.name,
            net.cidr,
            ip_to_db(net.gateway_ip),
            dns,
            net.created_at,
            net.updated_at,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Lee `(name, name_source)` de una red por su `id` (CIDR). `None` si no existe.
///
/// Separada de [`get_network`] (que no mapea `name_source` al struct puro
/// `Network`): la usa `get_network_name_cmd` para mostrar el nombre + el origen
/// (`auto`/`user`) en el pie de la sidebar.
pub fn get_network_name(conn: &Connection, id: &str) -> DbResult<Option<(String, String)>> {
    let result = conn.query_row(
        "SELECT name, name_source FROM networks WHERE id = ?1",
        [id],
        |row| Ok::<_, rusqlite::Error>((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );
    match result {
        Ok(pair) => Ok(Some(pair)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_sqlite(e)),
    }
}

/// Fija la etiqueta de usuario de una red (`name_source = 'user'`), upsert por `id`.
///
/// Upsert (no UPDATE) para que editar el nombre funcione incluso antes del primer
/// escaneo, cuando la fila aún no existe: en ese caso se inserta con `cidr = id`
/// (el `id` ya ES el CIDR) y `name_source = 'user'`. Un escaneo posterior pasará
/// por [`upsert_network`], cuya precedencia de override conserva esta etiqueta.
/// `now` lo genera el llamador (patrón de timestamps del crate).
pub fn set_network_name(conn: &Connection, id: &str, label: &str, now: &str) -> DbResult<()> {
    conn.execute(
        "INSERT INTO networks (id, name, cidr, gateway_ip, dns_servers, name_source, created_at, updated_at)
         VALUES (?1, ?2, ?1, NULL, '[]', 'user', ?3, ?3)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           name_source = 'user',
           updated_at = excluded.updated_at",
        rusqlite::params![id, label, now],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Lee una red por su `id`.
pub fn get_network(conn: &Connection, id: &str) -> DbResult<Option<Network>> {
    let result = conn.query_row(
        "SELECT id, name, cidr, gateway_ip, dns_servers, created_at, updated_at
         FROM networks WHERE id = ?1",
        [id],
        |row| {
            Ok::<_, rusqlite::Error>((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        },
    );
    match result {
        Ok((id, name, cidr, gateway, dns_raw, created_at, updated_at)) => {
            let dns_servers: Vec<std::net::IpAddr> = match dns_raw {
                Some(s) if !s.is_empty() => serde_json::from_str(&s)?,
                _ => Vec::new(),
            };
            Ok(Some(Network {
                id,
                name,
                cidr,
                gateway_ip: ip_from_db(gateway)?,
                dns_servers,
                created_at,
                updated_at,
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_sqlite(e)),
    }
}

/// Lista todas las redes, ordenadas por `updated_at` descendente (más reciente
/// primero). Para `GET /api/v1/networks`.
pub fn list_networks(conn: &Connection) -> DbResult<Vec<Network>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, cidr, gateway_ip, dns_servers, created_at, updated_at
             FROM networks ORDER BY updated_at DESC",
        )
        .map_err(map_sqlite)?;
    let rows = stmt
        .query_map([], |row| {
            Ok::<_, rusqlite::Error>((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(map_sqlite)?;
    let mut out = Vec::new();
    for row in rows {
        let (id, name, cidr, gateway, dns_raw, created_at, updated_at) = row.map_err(map_sqlite)?;
        let dns_servers: Vec<std::net::IpAddr> = match dns_raw {
            Some(s) if !s.is_empty() => serde_json::from_str(&s)?,
            _ => Vec::new(),
        };
        out.push(Network {
            id,
            name,
            cidr,
            gateway_ip: ip_from_db(gateway)?,
            dns_servers,
            created_at,
            updated_at,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::connect;

    fn ip(s: &str) -> std::net::IpAddr {
        s.parse().unwrap()
    }

    fn sample_net(id: &str) -> Network {
        Network {
            id: id.to_string(),
            name: "home".to_string(),
            cidr: "192.168.1.0/24".to_string(),
            gateway_ip: Some(ip("192.168.1.1")),
            dns_servers: vec![ip("192.168.1.1"), ip("8.8.8.8")],
            created_at: "2026-06-27T00:00:00Z".to_string(),
            updated_at: "2026-06-27T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn upsert_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(dir.path().join("n.db")).unwrap();
        let net = sample_net("net-1");
        upsert_network(&conn, &net).unwrap();
        upsert_network(&conn, &net).unwrap(); // second time updates, not inserts
        let back = get_network(&conn, "net-1").unwrap().expect("found");
        assert_eq!(back, net);
        assert_eq!(back.dns_servers, vec![ip("192.168.1.1"), ip("8.8.8.8")]);
    }

    #[test]
    fn upsert_updates_fields() {
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(dir.path().join("n.db")).unwrap();
        let mut net = sample_net("net-2");
        upsert_network(&conn, &net).unwrap();
        net.name = "guest".to_string();
        net.gateway_ip = Some(ip("192.168.1.254"));
        net.updated_at = "2026-06-28T00:00:00Z".to_string();
        upsert_network(&conn, &net).unwrap();
        let back = get_network(&conn, "net-2").unwrap().unwrap();
        assert_eq!(back.name, "guest");
        assert_eq!(back.gateway_ip, Some(ip("192.168.1.254")));
        // created_at preservado (no tocado).
        assert_eq!(back.created_at, "2026-06-27T00:00:00Z");
    }

    #[test]
    fn auto_upsert_updates_name_when_not_user_edited() {
        // Sin override de usuario, una re-detección actualiza `name` y mantiene
        // `name_source = 'auto'`.
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(dir.path().join("n.db")).unwrap();
        let mut net = sample_net("192.168.1.0/24");
        net.name = "OldSSID".to_string();
        upsert_network(&conn, &net).unwrap();
        net.name = "NewSSID".to_string();
        upsert_network(&conn, &net).unwrap();
        let (name, source) = get_network_name(&conn, "192.168.1.0/24").unwrap().unwrap();
        assert_eq!(name, "NewSSID");
        assert_eq!(source, "auto");
    }

    #[test]
    fn user_label_survives_auto_redetection() {
        // AC-3: tras editar el nombre (override de usuario), una re-detección de
        // SSID (upsert auto) NO debe pisar la etiqueta del usuario.
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(dir.path().join("n.db")).unwrap();
        let mut net = sample_net("192.168.1.0/24");
        net.name = "AutoSSID".to_string();
        upsert_network(&conn, &net).unwrap();

        set_network_name(&conn, "192.168.1.0/24", "Mi Casa", "2026-06-28T01:00:00Z").unwrap();

        // Re-detección posterior con otro SSID auto.
        net.name = "AnotherSSID".to_string();
        net.updated_at = "2026-06-28T02:00:00Z".to_string();
        upsert_network(&conn, &net).unwrap();

        let (name, source) = get_network_name(&conn, "192.168.1.0/24").unwrap().unwrap();
        assert_eq!(name, "Mi Casa", "la etiqueta de usuario debe sobrevivir");
        assert_eq!(source, "user");
    }

    #[test]
    fn set_network_name_inserts_when_row_missing() {
        // Editar antes del primer escaneo: la fila no existe → upsert la inserta.
        let dir = tempfile::tempdir().unwrap();
        let conn = connect(dir.path().join("n.db")).unwrap();
        assert!(get_network_name(&conn, "10.0.0.0/24").unwrap().is_none());
        set_network_name(&conn, "10.0.0.0/24", "Oficina", "2026-06-28T00:00:00Z").unwrap();
        let (name, source) = get_network_name(&conn, "10.0.0.0/24").unwrap().unwrap();
        assert_eq!(name, "Oficina");
        assert_eq!(source, "user");
        // `cidr` se rellenó con el `id` (id == cidr).
        let net = get_network(&conn, "10.0.0.0/24").unwrap().unwrap();
        assert_eq!(net.cidr, "10.0.0.0/24");
    }
}
