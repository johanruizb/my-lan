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
        /// Perfil de escaneo (quick | normal | deep | iot | router).
        /// `iot`/`router` gobiernan la selección de puertos en `mylan ports`;
        /// para el descubrimiento de hosts degradan a `normal`.
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
        /// Número de puertos "top" a sondear (override del conteo del perfil para
        /// quick/normal/deep; ignorado —con warning— para iot/router).
        #[arg(long, default_value_t = 100)]
        top: u16,
        /// Perfil de escaneo (quick | normal | deep | iot | router).
        #[arg(long, default_value = "quick", value_parser = parse_profile)]
        profile: ScanProfile,
    },
    /// Listar los servicios del inventario con filtros.
    Services {
        /// Filtrar por IP o ID de dispositivo.
        #[arg(long)]
        device: Option<String>,
        /// Filtrar por puerto.
        #[arg(long)]
        port: Option<u16>,
        /// Filtrar por protocolo (tcp | udp).
        #[arg(long)]
        protocol: Option<String>,
        /// Filtrar por nombre de servicio (substring, case-insensitive).
        #[arg(long)]
        service: Option<String>,
    },
    /// Exportar datos del inventario.
    Export {
        #[command(subcommand)]
        what: ExportTarget,
    },
    /// Diagnóstico de red: eco ICMP/TCP a un host.
    Ping {
        /// IP o hostname del host.
        ip: String,
        /// Número de paquetes (default 4).
        #[arg(long)]
        count: Option<u32>,
        /// Timeout por paquete en ms (default 1000).
        #[arg(long)]
        timeout_ms: Option<u64>,
        /// Forzar IPv4.
        #[arg(long)]
        ipv4: bool,
        /// Forzar IPv6.
        #[arg(long)]
        ipv6: bool,
    },
    /// Diagnóstico de red: traceroute a un host.
    Traceroute {
        /// IP o hostname del host.
        ip: String,
        /// Máximo de saltos (default 30).
        #[arg(long)]
        max_hops: Option<u8>,
        /// Timeout por salto en ms (default 1000).
        #[arg(long)]
        timeout_ms: Option<u64>,
    },
    /// Diagnóstico de red: resolución DNS.
    Dns {
        /// Hostname a resolver.
        host: String,
        /// Tipo de registro (A | AAAA | PTR | MX | TXT; default A/AAAA).
        #[arg(long)]
        rtype: Option<String>,
        /// Forzar IPv4.
        #[arg(long)]
        ipv4: bool,
        /// Forzar IPv6.
        #[arg(long)]
        ipv6: bool,
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
    /// Exportar la lista de servicios.
    Services {
        /// Formato de salida (json | csv).
        #[arg(long, default_value = "json", value_parser = parse_export_format)]
        format: ExportFormatArg,
        /// Fichero de salida (por defecto `mylan-services.<ext>`).
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
        "iot" => Ok(ScanProfile::Iot),
        "router" => Ok(ScanProfile::Router),
        other => Err(format!(
            "perfil no soportado: '{other}' (usar quick|normal|deep|iot|router)"
        )),
    }
}
