//! `mylan export devices --format json|csv` — exportación del inventario.
//!
//! Escribe ficheros válidos (serde_json / csv) con manejo de errores de
//! permisos (error-path): un path no escribible se reporta con `anyhow`.

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use rusqlite::Connection;

use mylan_db::service_repo::{list_services, ServiceExportRow, ServiceFilters};

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

/// Exporta el inventario de servicios al formato indicado.
///
/// Usa [`mylan_db::service_repo::list_services`] con filtros vacíos y acota a la
/// red activa. El CSV se escribe con writer manual (header + filas; no
/// `serialize`) para garantizar el orden y conjunto exacto de columnas:
/// `device_id,device_ip,display_name,protocol,port,service_name,product,version,
/// banner,state,first_seen_at,last_seen_at`. El JSON serializa el mismo
/// `ServiceExportRow` (consistencia de campos con el CSV).
pub fn export_services(
    conn: &Connection,
    format: ExportFormat,
    output: Option<&str>,
) -> anyhow::Result<()> {
    let mut rows = list_services(conn, &ServiceFilters::default())?;

    // Acota a la red activa (consistencia con `mylan export devices`).
    if let Some(net_id) = latest_network_id(conn)? {
        let net_ids: HashSet<String> = mylan_db::device_repo::list_devices(conn, &net_id)?
            .iter()
            .map(|d| d.id.clone())
            .collect();
        rows.retain(|r| net_ids.contains(&r.device_id));
    }

    if rows.is_empty() {
        println!("No hay servicios para exportar.");
        return Ok(());
    }

    let path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("mylan-services.{}", format.ext())));

    match format {
        ExportFormat::Json => write_services_json(&path, &rows)?,
        ExportFormat::Csv => write_services_csv(&path, &rows)?,
    }
    println!("Exportados {} servicios a {}", rows.len(), path.display());
    Ok(())
}

/// Serializa los servicios a CSV en `path` (writer manual: header + filas).
fn write_services_csv(path: &std::path::Path, rows: &[ServiceExportRow]) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut buf);
        wtr.write_record([
            "device_id",
            "device_ip",
            "display_name",
            "protocol",
            "port",
            "service_name",
            "product",
            "version",
            "banner",
            "state",
            "first_seen_at",
            "last_seen_at",
        ])?;
        for r in rows {
            let record = vec![
                r.device_id.clone(),
                r.device_ip.map(|i| i.to_string()).unwrap_or_default(),
                r.display_name.clone().unwrap_or_default(),
                format!("{:?}", r.protocol).to_lowercase(),
                r.port.to_string(),
                r.service_name.clone().unwrap_or_default(),
                r.product.clone().unwrap_or_default(),
                r.version.clone().unwrap_or_default(),
                r.banner.clone().unwrap_or_default(),
                format!("{:?}", r.state).to_lowercase(),
                r.first_seen_at.clone(),
                r.last_seen_at.clone(),
            ];
            wtr.write_record(record.iter().map(String::as_str))?;
        }
        wtr.flush()?;
    }
    write_file(path, &buf)
}

/// Serializa los servicios a JSON pretty en `path` (mismo `ServiceExportRow`).
fn write_services_json(path: &std::path::Path, rows: &[ServiceExportRow]) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(rows)?;
    write_file(path, json.as_bytes())
}
