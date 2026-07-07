//! Base de datos OUI: mapa prefijo 24-bit → vendor.
//!
//! Carga el snapshot IEEE OUI (`signatures/oui/oui.csv`, formato
//! `mac_prefix,vendor` con prefijo en hex minúsculas sin separadores) y resuelve
//! el fabricante a partir de una [`MacAddr`] vía su prefijo OUI de 24 bits.
//! Dominio puro: la carga es el único punto de I/O (lector `csv::Reader`).

use std::collections::HashMap;
use std::io::Read;

use mylan_core::MacAddr;

use crate::error::FingerprintError;

/// Mapa prefijo OUI (24-bit, hex minúsculas `aabbcc`) → nombre de vendor.
#[derive(Debug, Clone, Default)]
pub struct OuiDatabase {
    vendors: HashMap<String, String>,
}

impl OuiDatabase {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Carga desde un lector CSV con cabecera `mac_prefix,vendor`.
    ///
    /// El prefijo se normaliza a hex minúsculas sin separadores para casar con
    /// [`MacAddr::oui_hex`]. Las filas malformadas se saltan (no abortan la
    /// carga: un snapshot IEEE puede tener líneas residuales).
    pub fn load_csv<R: Read>(reader: R) -> Result<Self, FingerprintError> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(reader);
        let mut vendors = HashMap::new();
        for record in rdr.records() {
            let Ok(row) = record else { continue };
            if row.len() < 2 {
                continue;
            }
            let prefix = normalize_prefix(&row[0]);
            let vendor = row[1].trim();
            if prefix.len() == 6 && !vendor.is_empty() {
                vendors.insert(prefix, vendor.to_string());
            }
        }
        Ok(Self { vendors })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.vendors.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vendors.is_empty()
    }

    /// Resuelve el vendor para una MAC dada por su prefijo OUI 24-bit.
    #[must_use]
    pub fn vendor_for(&self, mac: &MacAddr) -> Option<&str> {
        self.vendors.get(&mac.oui_hex()).map(String::as_str)
    }
}

/// Normaliza un prefijo OUI a hex minúsculas sin separadores (6 nibbles).
fn normalize_prefix(raw: &str) -> String {
    raw.trim()
        .chars()
        .filter_map(|c| c.to_digit(16).and_then(|d| char::from_digit(d, 16)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str =
        "mac_prefix,vendor\naabbcc,Example Vendor Inc.\nDEADBE,Deadbeef LLC\nXX,bad\n,empty\n";

    fn mac(s: &str) -> MacAddr {
        MacAddr::parse(s).expect("valid mac")
    }

    #[test]
    fn loads_valid_rows_and_skips_malformed() {
        let db = OuiDatabase::load_csv(CSV.as_bytes()).expect("load");
        assert_eq!(db.len(), 2);
        assert_eq!(
            db.vendor_for(&mac("aa:bb:cc:11:22:33")),
            Some("Example Vendor Inc.")
        );
        assert_eq!(
            db.vendor_for(&mac("de:ad:be:00:00:00")),
            Some("Deadbeef LLC")
        );
    }

    #[test]
    fn unknown_prefix_returns_none() {
        let db = OuiDatabase::load_csv(CSV.as_bytes()).expect("load");
        assert!(db.vendor_for(&mac("00:11:22:33:44:55")).is_none());
    }

    #[test]
    fn matches_arbitrary_mac_from_any_lan() {
        // MAC arbitraria (no de esta LAN): prefijo 3c:5a:b4 (Hewlett Packard en
        // el snapshot real). Verifica contra el snapshot completo empaquetado.
        let oui_csv = std::fs::File::open("../../signatures/oui/oui.csv").expect("oui.csv present");
        let db = OuiDatabase::load_csv(oui_csv).expect("load full oui");
        assert!(
            db.len() > 30_000,
            "full OUI snapshot must be loaded, got {}",
            db.len()
        );
        // Cisco Systems aparece en el snapshot completo (prefijo e8:0a:b9).
        let cisco = db.vendor_for(&mac("e8:0a:b9:01:02:03"));
        assert!(
            cisco.is_some_and(|v| v.contains("Cisco")),
            "expected Cisco for e8:0a:b9, got {cisco:?}"
        );
    }

    #[test]
    fn empty_database_returns_none() {
        let db = OuiDatabase::new();
        assert!(db.is_empty());
        assert!(db.vendor_for(&mac("aa:bb:cc:dd:ee:ff")).is_none());
    }
}
