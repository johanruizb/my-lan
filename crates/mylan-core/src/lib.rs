//! `mylan-core` — modelos de dominio y tipos compartidos de MyLAN.
//!
//! Dominio puro, sin I/O de plataforma (principio P3). Contiene:
//! - Los modelos del inventario espejo del esquema DB (plan §8): [`Network`],
//!   [`Interface`], [`Device`], [`DeviceAddress`], [`Scan`], [`Service`].
//! - Las enumeraciones de dominio: [`Protocol`], [`DeviceType`], [`ScanProfile`],
//!   [`ScanKind`], [`ScanStatus`], [`ServiceState`] y la [`Confidence`].
//! - [`Observation`]: resultado normalizado de cualquier técnica de descubrimiento.
//! - La identidad estable de dispositivo ([`DeviceIdentity`], MAC > IP) y la
//!   lógica de merge/precedencia de confianza (dominio puro, P5/P3).
//! - La interfaz de enrichment como firma de función concreta ([`Enricher`]),
//!   NO un trait (P3).

#![forbid(unsafe_code)]

mod confidence;
mod enrich;
mod enums;
mod identity;
mod mac;
mod models;
mod observation;

pub use confidence::Confidence;
pub use enrich::{noop_enricher, Enricher};
pub use enums::{DeviceType, Protocol, ScanKind, ScanProfile, ScanStatus, ServiceState};
pub use identity::DeviceIdentity;
pub use mac::MacAddr;
pub use models::{
    Device, DeviceAddress, DnsRecord, Event, EventType, Interface, Network, PingMethod, PingResult,
    Scan, ScanSummary, Service, Severity, TraceHop,
};
pub use observation::{aggregate, Observation, Source};
