//! `mylan-cli` — lógica del binario `mylan` (lib + bin).
//!
//! Pipeline de **dos fases** (Paso 5 + integración): `liveness` (descubrimiento
//! → `Observation`s) → `enrichment` (fingerprint) → `persist` (upsert) →
//! registrar `Scan`. La lógica vive en la lib para que los tests de integración
//! la ejerciten inyectando `Observation`s falsas (sin red real).

#![forbid(unsafe_code)]

pub mod cli;
pub mod commands;
pub mod ctx;
pub mod pipeline;
pub mod util;

pub use pipeline::{run_scan_pipeline, ScanOutcome};
