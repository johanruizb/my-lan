//! Dirección MAC normalizada (`MacAddr`).
//!
//! Acepta los formatos habituales (`:`, `-`, `.`, sin separador, mayúsculas) y
//! los normaliza a una forma canónica de 6 bytes. La representación serde es la
//! cadena canónica en minúsculas `aa:bb:cc:dd:ee:ff`.

use std::fmt;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// Dirección MAC de 48 bits en forma canónica.
///
/// La identidad estable de dispositivo prefiere la MAC sobre la IP (P5); una MAC
/// toda en ceros (entradas ARP incompletas) no constituye identidad: ver
/// [`MacAddr::is_zero`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacAddr([u8; 6]);

impl MacAddr {
    /// Construye una MAC a partir de sus 6 octetos.
    #[must_use]
    pub const fn from_octets(octets: [u8; 6]) -> Self {
        Self(octets)
    }

    /// Octetos en orden de red.
    #[must_use]
    pub const fn octets(&self) -> [u8; 6] {
        self.0
    }

    /// Parsea y normaliza una cadena MAC en cualquiera de los formatos comunes.
    ///
    /// Devuelve `None` si no contiene exactamente 12 dígitos hexadecimales tras
    /// retirar los separadores `:`, `-` y `.`.
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        let mut octets = [0u8; 6];
        let mut nibbles = 0usize;
        for ch in input.trim().chars() {
            match ch {
                ':' | '-' | '.' => continue,
                _ => {}
            }
            let value = ch.to_digit(16)?;
            if nibbles >= 12 {
                return None;
            }
            let idx = nibbles / 2;
            if nibbles.is_multiple_of(2) {
                octets[idx] = (value as u8) << 4;
            } else {
                octets[idx] |= value as u8;
            }
            nibbles += 1;
        }
        if nibbles == 12 {
            Some(Self(octets))
        } else {
            None
        }
    }

    /// `true` si todos los octetos son cero (entrada ARP incompleta).
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        let [a, b, c, d, e, f] = self.0;
        a == 0 && b == 0 && c == 0 && d == 0 && e == 0 && f == 0
    }

    /// Prefijo OUI de 24 bits (primeros 3 octetos) usado para lookup de vendor.
    #[must_use]
    pub const fn oui_prefix(&self) -> [u8; 3] {
        [self.0[0], self.0[1], self.0[2]]
    }

    /// Prefijo OUI como cadena hexadecimal en minúsculas sin separadores
    /// (`aabbcc`), apto como clave de búsqueda contra `signatures/oui`.
    #[must_use]
    pub fn oui_hex(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2])
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d, e, g] = self.0;
        write!(f, "{a:02x}:{b:02x}:{c:02x}:{d:02x}:{e:02x}:{g:02x}")
    }
}

impl Serialize for MacAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for MacAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        MacAddr::parse(&raw).ok_or_else(|| de::Error::custom(format!("invalid MAC address: {raw}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_colon_form() {
        let mac = MacAddr::parse("AA:BB:CC:DD:EE:FF").expect("valid mac");
        assert_eq!(mac.octets(), [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        assert_eq!(mac.to_string(), "aa:bb:cc:dd:ee:ff");
    }

    #[test]
    fn normalizes_all_separator_styles_to_same_value() {
        let colon = MacAddr::parse("aa:bb:cc:dd:ee:ff");
        let dash = MacAddr::parse("AA-BB-CC-DD-EE-FF");
        let dotted = MacAddr::parse("aabb.ccdd.eeff");
        let bare = MacAddr::parse(" AABBCCDDEEFF ");
        assert_eq!(colon, dash);
        assert_eq!(colon, dotted);
        assert_eq!(colon, bare);
    }

    #[test]
    fn rejects_malformed_macs() {
        assert!(MacAddr::parse("").is_none());
        assert!(MacAddr::parse("aa:bb:cc:dd:ee").is_none()); // too short
        assert!(MacAddr::parse("aa:bb:cc:dd:ee:ff:00").is_none()); // too long
        assert!(MacAddr::parse("zz:bb:cc:dd:ee:ff").is_none()); // non-hex
    }

    #[test]
    fn detects_zero_mac() {
        assert!(MacAddr::parse("00:00:00:00:00:00")
            .expect("parses")
            .is_zero());
        assert!(!MacAddr::parse("aa:bb:cc:dd:ee:ff")
            .expect("parses")
            .is_zero());
    }

    #[test]
    fn exposes_oui_prefix() {
        let mac = MacAddr::parse("3c:5a:b4:11:22:33").expect("valid mac");
        assert_eq!(mac.oui_prefix(), [0x3c, 0x5a, 0xb4]);
        assert_eq!(mac.oui_hex(), "3c5ab4");
    }

    #[test]
    fn serde_round_trip_via_string() {
        let mac = MacAddr::parse("3c:5a:b4:11:22:33").expect("valid mac");
        let json = serde_json::to_string(&mac).expect("serialize");
        assert_eq!(json, "\"3c:5a:b4:11:22:33\"");
        let back: MacAddr = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(mac, back);
    }
}
