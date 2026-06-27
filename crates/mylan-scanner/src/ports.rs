//! Catálogo de puertos comunes y mapeo puerto → nombre de servicio (IANA).
//!
//! Lista ranqueada de los puertos TCP más comunes en LAN (estilo nmap top-ports).
//! [`select_ports`] devuelve los primeros `top` puertos acotados al catálogo. El
//! mapeo [`port_to_service_name`] usa los nombres canónicos IANA para los puertos
//! bien conocidos; los que no están en el mapa devuelven `None` (la heurística de
//! fingerprint puede afinarlos después).

/// Catálogo ranqueado de los 100 puertos TCP más comunes (orden por frecuencia
/// observada en redes domésticas, no por número de puerto). Cubre los perfiles
/// `quick` (top 32) y `--top 100`.
pub const COMMON_PORTS: &[u16] = &[
    // — Top 32 (perfil quick) —
    80, 23, 443, 21, 22, 25, 3389, 110, 139, 143, 445, 53, 135, 3306, 8080, 1723, 111, 995, 993,
    5900, 1025, 587, 8888, 199, 1720, 465, 548, 113, 81, 554, 631, 873,
    // — 33..100 (ampliación a top 100) —
    5060, 5061, 2049, 515, 636, 989, 990, 5357, 49152, 49153, 49154, 49155, 49156, 1900, 2195, 5223,
    2196, 5353, 49157, 49158, 5800, 5901, 5902, 6000, 6001, 10243, 10244, 10245, 10246, 10247,
    10248, 10249, 10250, 17500, 3000, 3001, 3390, 3391, 5000, 5001, 5002, 5003, 5222, 5269, 5280,
    5298, 5355, 6379, 6443, 6667, 7000, 7001, 7777, 8000, 8001, 8008, 8081, 8082, 8443, 9000, 9090,
    9091, 9100, 9999, 10000, 27017, 27018, 27019, 32400,
];

/// Nombre de servicio IANA para un puerto bien conocido, si existe.
///
/// Es solo una etiqueta inicial; el enriquecimiento (Paso 6) puede refinarla a
/// partir del banner. Devuelve `None` para puertos no listados (no unknown/closed).
#[must_use]
pub fn port_to_service_name(port: u16) -> Option<&'static str> {
    match port {
        20 => Some("ftp-data"),
        21 => Some("ftp"),
        22 => Some("ssh"),
        23 => Some("telnet"),
        25 => Some("smtp"),
        53 => Some("domain"),
        80 => Some("http"),
        110 => Some("pop3"),
        111 => Some("rpcbind"),
        113 => Some("ident"),
        135 => Some("msrpc"),
        139 => Some("netbios-ssn"),
        143 => Some("imap"),
        443 => Some("https"),
        445 => Some("microsoft-ds"),
        465 => Some("submissions"),
        515 => Some("printer"),
        548 => Some("afp"),
        554 => Some("rtsp"),
        587 => Some("submission"),
        631 => Some("ipp"),
        636 => Some("ldaps"),
        873 => Some("rsync"),
        990 => Some("ftps"),
        993 => Some("imaps"),
        995 => Some("pop3s"),
        1720 => Some("h323hostcall"),
        1723 => Some("pptp"),
        199 => Some("smux"),
        2049 => Some("nfs"),
        3000 | 3001 => Some("http-alt"),
        3306 => Some("mysql"),
        3389..=3391 => Some("ms-wbt-server"),
        5000..=5003 => Some("upnp"),
        5060 => Some("sip"),
        5061 => Some("sips"),
        5222 => Some("xmpp-client"),
        5269 => Some("xmpp-server"),
        5353 => Some("mdns"),
        5355 => Some("llmnr"),
        5357 => Some("wsdapi"),
        5800 => Some("vnc-http"),
        5900..=5902 => Some("vnc"),
        6000 | 6001 => Some("x11"),
        6379 => Some("redis"),
        6443 => Some("kubernetes"),
        6667 => Some("irc"),
        7000 | 7001 => Some("afs3-fileserver"),
        7777 => Some("http-alt"),
        8000 | 8001 | 8008 | 8080 | 8081 | 8082 | 8888 => Some("http-alt"),
        8443 => Some("https-alt"),
        9000 | 9090 | 9091 => Some("http-alt"),
        9100 => Some("jetdirect"),
        9999 => Some("http-alt"),
        10000 => Some("snet-sensor"),
        17500 => Some("db4o"),
        1900 => Some("ssdp"),
        2195 | 2196 => Some("apns"),
        27017..=27019 => Some("mongodb"),
        32400 => Some("plex"),
        49_152..=49_158 => Some("unknown"),
        _ => None,
    }
}

/// Selecciona los primeros `top` puertos del catálogo [`COMMON_PORTS`].
///
/// `top` se acota al tamaño del catálogo; `top == 0` devuelve vacío. El orden
/// preserva el ranking (los puertos más comunes primero).
#[must_use]
pub fn select_ports(top: u16) -> Vec<u16> {
    let max = COMMON_PORTS.len().min(top as usize);
    COMMON_PORTS[..max].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_at_least_100() {
        assert!(COMMON_PORTS.len() >= 100, "catálogo debe cubrir top 100");
    }

    #[test]
    fn catalog_is_unique_and_sorted_by_rank() {
        let mut seen = std::collections::HashSet::new();
        for &p in COMMON_PORTS {
            assert!(seen.insert(p), "puerto {p} duplicado en COMMON_PORTS");
        }
    }

    #[test]
    fn top_32_selects_32_ports() {
        let ports = select_ports(32);
        assert_eq!(ports.len(), 32);
        assert_eq!(ports[0], 80);
        // El perfil quick cubre los puertos LAN más relevantes.
        assert!(ports.contains(&22));
        assert!(ports.contains(&443));
        assert!(ports.contains(&445));
        assert!(ports.contains(&53));
    }

    #[test]
    fn top_100_selects_100_ports() {
        let ports = select_ports(100);
        assert_eq!(ports.len(), 100);
    }

    #[test]
    fn top_zero_is_empty() {
        assert!(select_ports(0).is_empty());
    }

    #[test]
    fn top_overflow_clamps_to_catalog() {
        let ports = select_ports(u16::MAX);
        assert_eq!(ports.len(), COMMON_PORTS.len());
    }

    #[test]
    fn maps_well_known_ports() {
        assert_eq!(port_to_service_name(22), Some("ssh"));
        assert_eq!(port_to_service_name(80), Some("http"));
        assert_eq!(port_to_service_name(443), Some("https"));
        assert_eq!(port_to_service_name(445), Some("microsoft-ds"));
        assert_eq!(port_to_service_name(3306), Some("mysql"));
        assert_eq!(port_to_service_name(6379), Some("redis"));
        assert_eq!(port_to_service_name(9100), Some("jetdirect"));
    }

    #[test]
    fn unknown_port_returns_none() {
        assert_eq!(port_to_service_name(12_345), None);
        assert_eq!(port_to_service_name(0), None);
    }
}
