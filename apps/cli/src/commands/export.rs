//! `mylan export devices --format json|csv` — exportación del inventario.
//!
//! Escribe ficheros válidos (serde_json / csv) con manejo de errores de
//! permisos (error-path): un path no escribible se reporta con `anyhow`.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::commands::{latest_network_id, open_db};
use crate::ctx::AppContext;
use crate::util::print_redaction_note;

/// Formato de exportación soportado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl ExportFormat {
    /// Parsea el flag `--format` (case-insensitive).
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            other => anyhow::bail!("formato no soportado: '{other}' (usar json|csv)"),
        }
    }

    fn ext(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }
}

/// Exporta el inventario de dispositivos al formato indicado.
pub fn run(ctx: &AppContext, format: ExportFormat, output: Option<&str>) -> anyhow::Result<()> {
    print_redaction_note();

    let conn = open_db(ctx)?;
    let net_id = match latest_network_id(&conn)? {
        Some(id) => id,
        None => anyhow::bail!("No hay inventario. Ejecuta `mylan scan` antes de exportar."),
    };
    let devices = mylan_db::device_repo::list_devices(&conn, &net_id)?;
    if devices.is_empty() {
        println!("No hay dispositivos para exportar en la red {net_id}.");
        return Ok(());
    }

    let path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("mylan-devices.{}", format.ext())));

    match format {
        ExportFormat::Json => write_json(&path, &devices)?,
        ExportFormat::Csv => write_csv(&path, &devices)?,
    }
    println!(
        "Exportados {} dispositivos a {}",
        devices.len(),
        path.display()
    );
    Ok(())
}

/// Serializa los dispositivos a JSON pretty en `path`.
fn write_json(path: &std::path::Path, devices: &[mylan_core::Device]) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(devices)?;
    write_file(path, json.as_bytes())
}

/// Serializa los dispositivos a CSV en `path`.
fn write_csv(path: &std::path::Path, devices: &[mylan_core::Device]) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut buf);
        for d in devices {
            wtr.serialize(d)?;
        }
        wtr.flush()?;
    }
    write_file(path, &buf)
}

/// Escribe `bytes` en `path`, mapeando errores de E/O (p.ej. permiso denegado).
fn write_file(path: &std::path::Path, bytes: &[u8]) -> anyhow::Result<()> {
    File::create(path)
        .and_then(|mut f| f.write_all(bytes))
        .map_err(|e| anyhow::anyhow!("no se pudo escribir {path:?}: {e}"))?;
    Ok(())
}
