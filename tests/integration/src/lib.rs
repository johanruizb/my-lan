//! Helpers compartidos para los tests de integración de MyLAN.
//!
//! Fixture de red (DB temporal), constructor de [`Observation`]s falsas y
//! utilidades para validar el pipeline scan → enrichment → persist → export
//! sin requerir red real.

#![allow(clippy::missing_panics_doc)]

use std::net::IpAddr;
use std::path::Path;

use mylan_core::{MacAddr, Network, Observation, Source};

/// Abre una DB temporal aislada y registra la red fixture `net-1`.
pub fn fixture_db(dir: &Path) -> mylan_db::DbResult<rusqlite::Connection> {
    let conn = mylan_db::connection::connect(dir.join("integration.db"))?;
    mylan_db::network_repo::upsert_network(&conn, &sample_network())?;
    Ok(conn)
}

/// Red fixture estable usada por todos los tests.
pub fn sample_network() -> Network {
    Network {
        id: "net-1".to_string(),
        name: "home".to_string(),
        cidr: "192.168.1.0/24".to_string(),
        gateway_ip: Some(ip("192.168.1.1")),
        dns_servers: Vec::new(),
        created_at: "2026-06-27T00:00:00Z".to_string(),
        updated_at: "2026-06-27T00:00:00Z".to_string(),
    }
}

/// Constructor de [`Observation`] con MAC + IP + hostname opcional.
pub fn obs(source: Source, mac_addr: &str, ip_addr: &str, hostname: Option<&str>) -> Observation {
    let mut o = Observation::new(source)
        .with_mac(mac(mac_addr))
        .with_ip(ip(ip_addr));
    if let Some(h) = hostname {
        o = o.with_hostname(h);
    }
    o
}

/// Constructor de [`Observation`] con un hint (p.ej. servicio mDNS).
pub fn obs_with_hint(source: Source, ip_addr: &str, key: &str, value: &str) -> Observation {
    Observation::new(source)
        .with_ip(ip(ip_addr))
        .with_hint(key, value)
}

fn ip(s: &str) -> IpAddr {
    s.parse().expect("valid ip")
}

fn mac(s: &str) -> MacAddr {
    MacAddr::parse(s).expect("valid mac")
}
