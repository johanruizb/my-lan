//! MyLAN CLI (`mylan`).
//!
//! Paso 1: esqueleto de comandos (todos imprimen "no implementado"). La lógica real
//! llega en pasos posteriores: descubrimiento (Paso 4-5), fingerprint (Paso 6),
//! port scanner (Paso 7).

use clap::{Parser, Subcommand};

/// MyLAN — Tu red, bajo control. Descubre, monitorea y protege tu red local.
#[derive(Parser)]
#[command(name = "mylan", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Estado general de MyLAN y de la interfaz activa.
    Status,
    /// Escanear la red local actual y actualizar el inventario.
    Scan {
        /// Perfil de escaneo (quick | normal | deep).
        #[arg(long, default_value = "quick")]
        profile: String,
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
enum ExportTarget {
    /// Exportar la lista de dispositivos.
    Devices {
        /// Formato de salida (json | csv).
        #[arg(long, default_value = "json")]
        format: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Status => todo_step("status", "Paso 4-5"),
        Command::Scan { profile, interface } => {
            todo_step(
                &format!("scan (profile={profile}, interface={interface:?})"),
                "Paso 4-5",
            );
        }
        Command::Devices => todo_step("devices", "Paso 5"),
        Command::Device { ip } => todo_step(&format!("device {ip}"), "Paso 5"),
        Command::Ports { ip, top } => todo_step(&format!("ports {ip} --top {top}"), "Paso 7"),
        Command::Export { what } => match what {
            ExportTarget::Devices { format } => {
                todo_step(&format!("export devices --format {format}"), "Paso 5");
            }
        },
        Command::Serve { port } => todo_step(&format!("serve --port {port}"), "fase futura"),
    }
    Ok(())
}

/// Imprime un aviso de comando aún no implementado y el paso del plan donde llegará.
fn todo_step(cmd: &str, step: &str) {
    eprintln!("[mylan] '{cmd}' aún no implementado (planificado: {step}).");
}
