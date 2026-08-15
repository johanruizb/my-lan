//! Pipeline de dos fases del `mylan scan` — reexportado desde `mylan-db`.
//!
//! La orquestación liveness→enrich→persist se movió a [`mylan_db::pipeline`]
//! (Fase 4-A) para que CLI, Desktop y la futura `mylan-api` la reusen sin
//! duplicarla. Este módulo conserva la ruta `mylan_cli::pipeline::*` (y el
//! re-export `mylan_cli::{run_scan_pipeline, ScanOutcome}`) para no romper a los
//! consumidores existentes de la CLI.

pub use mylan_db::pipeline::{run_scan_pipeline, ScanOutcome};
