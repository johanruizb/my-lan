//! Definiciones de `clap` para el CLI `mylan`.

use clap::{Parser, Subcommand};

use mylan_core::ScanProfile;

/// MyLAN — Tu red, bajo control. Descubre, monitorea y protege tu red local.
#[derive(Parser)]
#[command(name = "mylan", version, about, long_about = None)]
pub struct Cli {
    /// Verbosidad: activa trazas técnicas (`tracing`).
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Estado general de MyLAN y de la interfaz activa.
    Status,
    /// Escanear la red local actual y actualizar el inventario.
    Scan {
        /// Perfil de escaneo (quick | normal | deep).
        #[arg(long, default_value = "quick", value_parser = parse_profile)]
        profile: ScanProfile,
        /// Forzar una interfaz concreta (por defecto: la de la ruta por defecto).
        #[arg(long)]
        interface: Option<String>,
    },
    /// Listar los dispositivos del inventario.
    Devices,
    /// Mostrar el detalle de un dispositivo por IP.
    Device {
        /// Dirección IP del dispositivo.
        ip: String,
    },
    /// Escanear los puertos de un dispositivo.
    Ports {
        /// Dirección IP del dispositivo.
        ip: String,
        /// Número de puertos "top" a sondear.
        #[arg(long, default_value_t = 100)]
        top: u16,
    },
    /// Exportar datos del inventario.
    Export {
        #[command(subcommand)]
        what: ExportTarget,
    },
    /// Servir la API local (stub; fase futura).
    Serve {
        /// Puerto de escucha.
        #[arg(long, default_value_t = 43117)]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum ExportTarget {
    /// Exportar la lista de dispositivos.
    Devices {
        /// Formato de salida (json | csv).
        #[arg(long, default_value = "json", value_parser = parse_export_format)]
        format: ExportFormatArg,
        /// Fichero de salida (por defecto `mylan-devices.<ext>`).
        #[arg(long)]
        output: Option<String>,
    },
}

/// Wrapper para que clap valide `--format` reutilizando `ExportFormat::parse`.
#[derive(Debug, Clone, Copy)]
pub struct ExportFormatArg(pub crate::commands::export::ExportFormat);

fn parse_export_format(s: &str) -> Result<ExportFormatArg, String> {
    crate::commands::export::ExportFormat::parse(s)
        .map(ExportFormatArg)
        .map_err(|e| e.to_string())
}

fn parse_profile(s: &str) -> Result<ScanProfile, String> {
    match s.to_ascii_lowercase().as_str() {
        "quick" => Ok(ScanProfile::Quick),
        "normal" => Ok(ScanProfile::Normal),
        "deep" => Ok(ScanProfile::Deep),
        other => Err(format!(
            "perfil no soportado: '{other}' (usar quick|normal|deep)"
        )),
    }
}
