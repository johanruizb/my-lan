//! `mylan-fingerprint` — identificación de dispositivos.
//!
//! Resuelve vendor por OUI (snapshot IEEE completo en `signatures/oui/oui.csv`),
//! hostname/reverse-DNS best-effort ([`reverse_dns`]), interpreta observaciones
//! mDNS/SSDP y aplica un motor de reglas YAML (`signatures/device-rules/`) para
//! inferir `device_type` + `confidence`. Implementa la interfaz de enrichment
//! ([`Fingerprint::enricher`]) definida en `mylan-core` como función concreta, de
//! forma aditiva sobre el pipeline de dos fases.
//!
//! # Ética (P2)
//! Solo descubrimiento pasivo de señales ya recogidas por `mylan-discovery`;
//! cero técnicas activas/intrusivas aquí.
//!
//! # Ejemplo
//! ```no_run
//! use std::path::Path;
//! use mylan_fingerprint::Fingerprint;
//! use mylan_core::{Device, Enricher, Observation, Source};
//!
//! let fp = Fingerprint::load(Path::new("signatures"), None).expect("load");
//! let enricher: Enricher = fp.enricher();
//! let mut device = Device::new("dev-1", "net-1", "2026-06-27T00:00:00Z");
//! let obs = vec![Observation::new(Source::Mdns).with_hint("mdns.service", "_rtsp._tcp")];
//! enricher(&mut device, &obs);
//! ```

#![forbid(unsafe_code)]

mod error;
mod fingerprint;
mod oui;
mod reverse;
mod rules;

pub use error::FingerprintError;
pub use fingerprint::Fingerprint;
pub use oui::OuiDatabase;
pub use reverse::reverse_dns;
pub use rules::{Match, Matcher, Rule, RuleSet};
