//! `Fingerprint`: orquesta OUI + reglas + heurísticas para construir el
//! [`Enricher`] de `mylan-core`.
//!
//! La fase de enrichment del pipeline (Paso 5/6) recibe un [`Enricher`]; aquí se
//! construye un closure con estado (mapa OUI + reglas + IP del gateway) que
//! implementa la firma `Fn(&mut Device, &[Observation])`. El cambio es aditivo:
//! el pipeline de Paso 5 sustituye el no-op por esta llamada sin reescribirse.

use std::net::IpAddr;
use std::path::Path;

use mylan_core::{Confidence, Device, DeviceType, Enricher, Observation};

use crate::error::FingerprintError;
use crate::oui::OuiDatabase;
use crate::rules::RuleSet;

/// Configuración de fingerprinting cargada desde `signatures/`.
///
/// Inmutable tras la carga; `enricher` la mueve al closure para que el enricher
/// sea `Send + Sync + 'static` y autónomo.
pub struct Fingerprint {
    oui: OuiDatabase,
    rules: RuleSet,
    gateway_ip: Option<IpAddr>,
}

impl Fingerprint {
    /// Carga OUI (`signatures/oui/oui.csv`) y reglas (`signatures/device-rules/`)
    /// y fija la IP del gateway de la red escaneada (para la heurística router).
    pub fn load(
        signatures_dir: &Path,
        gateway_ip: Option<IpAddr>,
    ) -> Result<Self, FingerprintError> {
        let oui_path = signatures_dir.join("oui").join("oui.csv");
        let oui = if oui_path.exists() {
            let file = std::fs::File::open(&oui_path)?;
            OuiDatabase::load_csv(file)?
        } else {
            OuiDatabase::new()
        };
        let rules = RuleSet::load_dir(&signatures_dir.join("device-rules"))?;
        Ok(Self {
            oui,
            rules,
            gateway_ip,
        })
    }

    /// Construye el [`Enricher`] a partir de esta configuración.
    ///
    /// El closure: rellena `vendor`/`manufacturer` por OUI, rellena `hostname`
    /// desde las observaciones si falta, aplica la heurística de router por
    /// gateway y evalúa las reglas YAML (mayor confidence gana vía
    /// `Device::apply_classification`).
    #[must_use]
    pub fn enricher(self) -> Enricher {
        let oui = std::sync::Arc::new(self.oui);
        let rules = std::sync::Arc::new(self.rules);
        let gateway_ip = self.gateway_ip;
        Box::new(move |device, observations| {
            enrich_device(device, observations, &oui, &rules, gateway_ip);
        })
    }
}

/// Lógica de enrichment pura (testeable sin construir el closure).
fn enrich_device(
    device: &mut Device,
    observations: &[Observation],
    oui: &OuiDatabase,
    rules: &RuleSet,
    gateway_ip: Option<IpAddr>,
) {
    // 1. Vendor por OUI (primera MAC no-cero del dispositivo).
    if device.vendor.is_none() {
        if let Some(mac) = device.primary_mac.filter(|m| !m.is_zero()) {
            if let Some(vendor) = oui.vendor_for(&mac) {
                let v = vendor.to_string();
                device.vendor = Some(v.clone());
                if device.manufacturer.is_none() {
                    device.manufacturer = Some(v);
                }
            }
        } else if let Some(mac) = observations
            .iter()
            .find_map(|o| o.mac.filter(|m| !m.is_zero()))
        {
            if let Some(vendor) = oui.vendor_for(&mac) {
                let v = vendor.to_string();
                device.vendor = Some(v.clone());
                if device.manufacturer.is_none() {
                    device.manufacturer = Some(v);
                }
            }
        }
    }

    // 2. Hostname desde observaciones (si merge_observation no lo rellenó).
    if device.hostname.is_none() {
        if let Some(h) = observations.iter().find_map(|o| o.hostname.clone()) {
            device.hostname = Some(h);
        }
    }

    // 3. Heurística router por gateway: si la IP primaria es el gateway.
    if let Some(gw) = gateway_ip {
        if device.primary_ip == Some(gw) {
            device.apply_classification(DeviceType::Router, Confidence::new(70));
        }
    }

    // 4. Reglas YAML: la mejor clasificación (mayor confidence) gana; la
    //    precedencia la decide `apply_classification`.
    if let Some((dtype, conf)) = rules.evaluate(observations) {
        device.apply_classification(dtype, conf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mylan_core::{MacAddr, Source};
    use std::net::IpAddr;

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("valid ip")
    }

    fn mac(s: &str) -> MacAddr {
        MacAddr::parse(s).expect("valid mac")
    }

    fn oui_db() -> OuiDatabase {
        OuiDatabase::load_csv("mac_prefix,vendor\naabbcc,Example Vendor Inc.\n".as_bytes())
            .expect("load")
    }

    #[test]
    fn enricher_fills_vendor_from_oui() {
        let oui = oui_db();
        let rules = RuleSet::new();
        let enrich = Fingerprint {
            oui,
            rules,
            gateway_ip: None,
        }
        .enricher();
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        device.primary_mac = Some(mac("aa:bb:cc:11:22:33"));
        enrich(&mut device, &[]);
        assert_eq!(device.vendor.as_deref(), Some("Example Vendor Inc."));
        assert_eq!(device.manufacturer.as_deref(), Some("Example Vendor Inc."));
    }

    #[test]
    fn enricher_fills_hostname_from_observations() {
        let oui = OuiDatabase::new();
        let rules = RuleSet::new();
        let enrich = Fingerprint {
            oui,
            rules,
            gateway_ip: None,
        }
        .enricher();
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        device.primary_ip = Some(ip("192.168.1.5"));
        let obs = [Observation::new(Source::Mdns).with_hostname("nas.local")];
        enrich(&mut device, &obs);
        assert_eq!(device.hostname.as_deref(), Some("nas.local"));
    }

    #[test]
    fn enricher_classifies_router_when_ip_is_gateway() {
        let oui = OuiDatabase::new();
        let rules = RuleSet::new();
        let enrich = Fingerprint {
            oui,
            rules,
            gateway_ip: Some(ip("192.168.1.1")),
        }
        .enricher();
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        device.primary_ip = Some(ip("192.168.1.1"));
        enrich(&mut device, &[]);
        assert_eq!(device.device_type, DeviceType::Router);
        assert_eq!(device.confidence.score(), 70);
    }

    #[test]
    fn enricher_applies_camera_rule_over_router_when_higher_confidence() {
        // Gateway heuristic gives Router@70; camera rule gives Camera@75.
        // apply_classification: 75 >= 70 -> Camera wins.
        let oui = OuiDatabase::new();
        let rules = RuleSet::load_dir(std::path::Path::new("../../signatures/device-rules"))
            .expect("load rules");
        let enrich = Fingerprint {
            oui,
            rules,
            gateway_ip: Some(ip("192.168.1.1")),
        }
        .enricher();
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        device.primary_ip = Some(ip("192.168.1.1"));
        let obs = [Observation::new(Source::Mdns)
            .with_ip(ip("192.168.1.1"))
            .with_hint("mdns.service", "_rtsp._tcp")];
        enrich(&mut device, &obs);
        assert_eq!(device.device_type, DeviceType::Camera);
        assert_eq!(device.confidence.score(), 75);
    }

    #[test]
    fn enricher_leaves_unknown_when_no_signals() {
        let oui = oui_db();
        let rules = RuleSet::new();
        let enrich = Fingerprint {
            oui,
            rules,
            gateway_ip: None,
        }
        .enricher();
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        device.primary_mac = Some(mac("00:11:22:33:44:55")); // unknown OUI
        enrich(&mut device, &[]);
        assert_eq!(device.device_type, DeviceType::Unknown);
        assert!(device.vendor.is_none());
    }
}
