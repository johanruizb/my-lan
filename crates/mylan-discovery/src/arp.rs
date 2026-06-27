//! Parser de la tabla ARP del kernel (`/proc/net/arp`).
//!
//! El *parser* es puro (opera sobre una cadena) y compila en cualquier objetivo; la
//! lectura del fichero solo existe en Linux tras `#[cfg(target_os = "linux")]`. En
//! el resto de plataformas [`read_arp_cache`] devuelve una lista vacía (impl default
//! portable, sin `todo!()`), de modo que el crate compila sin Linux.

use std::net::IpAddr;

use mylan_core::{MacAddr, Observation, Source};

/// Entrada ARP parseada de `/proc/net/arp`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpEntry {
    /// IP del vecino.
    pub ip: IpAddr,
    /// MAC resuelta; `None` si la entrada está incompleta (MAC en ceros o `*`).
    pub mac: Option<MacAddr>,
    /// Interfaz donde se aprendió la entrada (p.ej. `enp37s0`).
    pub device: String,
}

/// Parsea el contenido de `/proc/net/arp` en entradas ARP.
///
/// Formato esperado (separado por espacios):
/// ```text
/// IP address       HW type     Flags       HW address            Mask     Device
/// 192.168.1.1      0x1         0x2         aa:bb:cc:dd:ee:ff     *        enp37s0
/// ```
/// La cabecera se detecta por el campo `HW address` y se ignora. Las entradas con MAC
/// `00:00:00:00:00:00` o `*` se consideran incompletas y se reportan con `mac = None`.
/// Líneas mal formadas se descartan silenciosamente (best-effort).
#[must_use]
pub fn parse_arp_table(text: &str) -> Vec<ArpEntry> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Cabecera: contiene la marca literal "HW address".
        if trimmed.to_ascii_lowercase().contains("hw address") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        // IP, HW type, Flags, HW address, [Mask], [Device]
        if parts.len() < 4 {
            continue;
        }
        let Ok(ip) = parts[0].parse::<IpAddr>() else {
            continue;
        };
        let mac_field = parts[3];
        // `*` o MAC en ceros = entrada incompleta (ip-only). Una MAC no parseable
        // indica línea malformada: se descarta.
        let mac = match mac_field {
            "*" => None,
            s => match MacAddr::parse(s) {
                Some(m) if m.is_zero() => None,
                Some(m) => Some(m),
                None => continue,
            },
        };
        // `device` es el último campo cuando existe.
        let device = parts.last().copied().unwrap_or("").to_string();
        // Heurística: si `device` se ve como una MAC/máscara, lo descartamos.
        if device.is_empty() || device.contains(':') {
            continue;
        }
        out.push(ArpEntry { ip, mac, device });
    }
    out
}

/// Convierte entradas ARP en [`Observation`]s de origen [`Source::ArpCache`].
///
/// Solo las entradas **completas** (con MAC resuelta) representan un host vivo: el
/// kernel aprendió la MAC durante una conexión TCP o un eco ICMP. Las entradas
/// incompletas (sin MAC) son hosts que NO respondieron al ARP durante el barrido
/// (hosts muertos); incluirlas crearía un falso positivo (un dispositivo IP-only)
/// por cada IP sondeada. Si `iface` se indica, se filtran las entradas de otras
/// interfaces (p.ej. `docker0`).
#[must_use]
pub fn arp_entries_to_observations(entries: &[ArpEntry], iface: Option<&str>) -> Vec<Observation> {
    entries
        .iter()
        .filter(|e| iface.is_none_or(|name| e.device == name))
        .filter_map(|e| {
            let mac = e.mac?;
            Some(
                Observation::new(Source::ArpCache)
                    .with_ip(e.ip)
                    .with_mac(mac),
            )
        })
        .collect()
}

/// Lee la tabla ARP del kernel. En Linux lee `/proc/net/arp`; en otras plataformas
/// devuelve `Ok(vec![])` (degradación portable: el resto del flujo sigue funcionando).
pub fn read_arp_cache() -> std::io::Result<Vec<ArpEntry>> {
    read_arp_cache_impl()
}

#[cfg(target_os = "linux")]
fn read_arp_cache_impl() -> std::io::Result<Vec<ArpEntry>> {
    let text = std::fs::read_to_string("/proc/net/arp")?;
    Ok(parse_arp_table(&text))
}

#[cfg(not(target_os = "linux"))]
fn read_arp_cache_impl() -> std::io::Result<Vec<ArpEntry>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn mac(s: &str) -> MacAddr {
        MacAddr::parse(s).unwrap()
    }

    const SAMPLE: &str =
        "IP address       HW type     Flags       HW address            Mask     Device\n\
192.168.1.1      0x1         0x2         aa:bb:cc:dd:ee:ff     *        enp37s0\n\
192.168.1.5      0x1         0x2         00:1c:c4:11:22:33     *        enp37s0\n\
192.168.1.9      0x1         0x0         00:00:00:00:00:00     *        enp37s0\n\
10.0.0.2         0x1         0x2         11:22:33:44:55:66     *        docker0\n";

    #[test]
    fn parses_valid_entries() {
        let entries = parse_arp_table(SAMPLE);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].ip, ip("192.168.1.1"));
        assert_eq!(entries[0].mac, Some(mac("aa:bb:cc:dd:ee:ff")));
        assert_eq!(entries[0].device, "enp37s0");
        assert_eq!(entries[2].mac, None); // zero mac -> None
        assert_eq!(entries[3].device, "docker0");
    }

    #[test]
    fn skips_header_and_blank_lines() {
        let text =
            "IP address       HW type     Flags       HW address            Mask     Device\n\n\
192.168.1.1      0x1         0x2         aa:bb:cc:dd:ee:ff     *        enp37s0\n";
        let entries = parse_arp_table(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ip, ip("192.168.1.1"));
    }

    #[test]
    fn discards_malformed_lines() {
        let text = "garbage line with no mac\n\
not an ip        0x1         0x2         aa:bb:cc:dd:ee:ff     *        enp37s0\n\
192.168.1.2      0x1         0x2         zz:bb:cc:dd:ee:ff     *        enp37s0\n\
192.168.1.3      0x1         0x2         aa:bb:cc:dd:ee:ff     *        enp37s0\n";
        let entries = parse_arp_table(text);
        // Only the last line is valid (valid IP + valid MAC + device).
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ip, ip("192.168.1.3"));
    }

    #[test]
    fn handles_star_mac_field() {
        let text =
            "192.168.1.50     0x1         0x0         *                    *        enp37s0\n";
        let entries = parse_arp_table(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].mac, None);
    }

    #[test]
    fn observations_filter_by_interface() {
        let entries = parse_arp_table(SAMPLE);
        let obs = arp_entries_to_observations(&entries, Some("enp37s0"));
        // Dos entradas completas en enp37s0 (.1 y .5); la incompleta .9 se descarta.
        assert_eq!(obs.len(), 2);
        assert!(obs.iter().all(|o| o.source == Source::ArpCache));
        // docker0 entry filtered out.
        assert!(!obs.iter().any(|o| o.ip == Some(ip("10.0.0.2"))));
    }

    #[test]
    fn observations_exclude_incomplete_entries() {
        let entries = parse_arp_table(SAMPLE);
        let obs = arp_entries_to_observations(&entries, None);
        // La entrada incompleta 192.168.1.9 (sin MAC) NO genera observación: es un
        // host muerto que no respondió al ARP durante el barrido.
        assert!(obs.iter().all(|o| o.mac.is_some()));
        assert!(!obs.iter().any(|o| o.ip == Some(ip("192.168.1.9"))));
        // Las entradas completas sí se incluyen.
        assert_eq!(obs.len(), 3);
    }

    #[test]
    fn empty_input_yields_empty() {
        assert!(parse_arp_table("").is_empty());
    }
}
