//! Conversión entre tipos de `mylan-core` y columnas SQLite (TEXT/INTEGER).
//!
//! Pequeño pegamento: las enumeraciones serializan en `snake_case` (reexport de
//! serde) y aquí se proyectan a la columna `TEXT` sin las comillas del JSON;
//! `MacAddr`/`IpAddr` van como cadena canónica y se parsean de vuelta.

use std::net::IpAddr;

use serde::{de::DeserializeOwned, Serialize};

use mylan_core::MacAddr;

use crate::error::{DbError, DbResult};

/// Serializa una enumeración a su nombre `snake_case` (sin comillas).
pub(crate) fn enum_to_db<E: Serialize>(value: &E) -> DbResult<String> {
    let json = serde_json::to_string(value)?;
    // `serde_json` rodea la cadena con comillas: `"router"` -> `router`.
    if json.len() >= 2 && json.starts_with('"') && json.ends_with('"') {
        Ok(json[1..json.len() - 1].to_string())
    } else {
        Err(DbError::Serde(
            <serde_json::Error as serde::de::Error>::custom("esperaba un literal string"),
        ))
    }
}

/// Deserializa una enumeración desde su nombre `snake_case`.
pub(crate) fn enum_from_db<E: DeserializeOwned>(s: &str) -> DbResult<E> {
    let wrapped = format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    Ok(serde_json::from_str(&wrapped)?)
}

/// Convierte una `MacAddr` opcional a su representación `TEXT` (puede ser `NULL`).
pub(crate) fn mac_to_db(mac: Option<MacAddr>) -> Option<String> {
    mac.map(|m| m.to_string())
}

/// Parsea una columna `TEXT` (MAC canónica) de vuelta a `MacAddr`.
pub(crate) fn mac_from_db(raw: Option<String>) -> DbResult<Option<MacAddr>> {
    match raw {
        Some(s) => MacAddr::parse(&s).map(Some).ok_or_else(|| {
            DbError::Serde(<serde_json::Error as serde::de::Error>::custom(
                "MAC inválida",
            ))
        }),
        None => Ok(None),
    }
}

/// Convierte una `IpAddr` opcional a su representación `TEXT`.
pub(crate) fn ip_to_db(ip: Option<IpAddr>) -> Option<String> {
    ip.map(|i| i.to_string())
}

/// Parsea una columna `TEXT` (IP) de vuelta a `IpAddr`.
pub(crate) fn ip_from_db(raw: Option<String>) -> DbResult<Option<IpAddr>> {
    match raw {
        Some(s) => {
            let parsed = s.parse::<IpAddr>().map_err(|_| {
                DbError::Serde(<serde_json::Error as serde::de::Error>::custom(
                    "IP inválida",
                ))
            })?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mylan_core::{DeviceType, Protocol, ServiceState};

    #[test]
    fn enum_round_trip() {
        assert_eq!(enum_to_db(&DeviceType::Router).unwrap(), "router");
        assert_eq!(enum_to_db(&Protocol::Tcp).unwrap(), "tcp");
        assert_eq!(enum_to_db(&ServiceState::Open).unwrap(), "open");
        assert_eq!(
            enum_from_db::<DeviceType>("router").unwrap(),
            DeviceType::Router
        );
        assert_eq!(enum_from_db::<Protocol>("udp").unwrap(), Protocol::Udp);
        assert_eq!(
            enum_from_db::<ServiceState>("filtered").unwrap(),
            ServiceState::Filtered
        );
    }

    #[test]
    fn mac_round_trip() {
        let m = MacAddr::parse("aa:bb:cc:dd:ee:ff").unwrap();
        assert_eq!(mac_to_db(Some(m)), Some("aa:bb:cc:dd:ee:ff".to_string()));
        assert_eq!(
            mac_from_db(Some("aa:bb:cc:dd:ee:ff".to_string())).unwrap(),
            Some(m)
        );
        assert_eq!(mac_from_db(None).unwrap(), None);
    }

    #[test]
    fn ip_round_trip() {
        let ip: IpAddr = "192.168.1.5".parse().unwrap();
        assert_eq!(ip_to_db(Some(ip)), Some("192.168.1.5".to_string()));
        assert_eq!(
            ip_from_db(Some("192.168.1.5".to_string())).unwrap(),
            Some(ip)
        );
        assert!(ip_from_db(Some("nope".to_string())).is_err());
    }
}
