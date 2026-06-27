//! Repositorio de redes (`networks`).
//!
//! Upsert por `id` (clave primaria). `dns_servers` se serializa como un array
//! JSON de IPs en la columna `TEXT`.

use rusqlite::Connection;

use mylan_core::Network;

use crate::codec::{ip_from_db, ip_to_db};
use crate::error::{map_sqlite, DbResult};

/// Inserta o actualiza una red por su `id` (upsert `ON CONFLICT`).
pub fn upsert_network(conn: &Connection, net: &Network) -> DbResult<()> {
    let dns = serde_json::to_string(&net.dns_servers)?;
    conn.execute(
        "INSERT INTO networks (id, name, cidr, gateway_ip, dns_servers, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
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
}
