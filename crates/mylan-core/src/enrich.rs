//! Interfaz de enrichment como firma de función concreta (NO un trait).
//!
//! Respeta P3: no se introduce un trait de estrategia sin un 2º consumidor real.
//! La fase de enrichment del pipeline (Paso 5/6) recibe un [`Enricher`]: en Paso 5
//! es el no-op de [`noop_enricher`]; en Paso 6 `mylan-fingerprint` construye un
//! closure con estado (mapa OUI + reglas) que implementa la misma firma, de modo
//! que el cambio es aditivo y no reescribe el pipeline.

use crate::models::Device;
use crate::observation::Observation;

/// Función que enriquece un [`Device`] usando las observaciones agregadas de su
/// host. Es un closure con estado propio (`Box<dyn Fn>`), no un trait.
pub type Enricher = Box<dyn Fn(&mut Device, &[Observation]) + Send + Sync + 'static>;

/// Enricher identidad usado por el pipeline de Paso 5 (no modifica el device).
#[must_use]
pub fn noop_enricher() -> Enricher {
    Box::new(|_device, _observations| {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::confidence::Confidence;
    use crate::enums::DeviceType;
    use crate::observation::Source;

    #[test]
    fn noop_leaves_device_untouched() {
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        let before = device.clone();
        let enrich = noop_enricher();
        enrich(&mut device, &[Observation::new(Source::ArpCache)]);
        assert_eq!(device, before);
    }

    #[test]
    fn stateful_closure_matches_signature() {
        // Simulates a Paso 6 enricher capturing owned state (a vendor table).
        let vendor = "Example Vendor".to_string();
        let enrich: Enricher = Box::new(move |device, _obs| {
            device.vendor = Some(vendor.clone());
            device.apply_classification(DeviceType::Router, Confidence::new(80));
        });
        let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
        enrich(&mut device, &[]);
        assert_eq!(device.vendor.as_deref(), Some("Example Vendor"));
        assert_eq!(device.device_type, DeviceType::Router);
    }
}
