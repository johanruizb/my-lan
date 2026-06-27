//! `mylan` — punto de entrada del binario.
//!
//! Parsea la CLI, inicializa `tracing` con `--verbose` y despacha al comando.

use clap::Parser;

use mylan_cli::cli::{Cli, Command, ExportTarget};
use mylan_cli::commands;
use mylan_cli::ctx::AppContext;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    let ctx = AppContext::new(cli.verbose);

    match cli.command {
        Command::Status => commands::status::run(&ctx),
        Command::Scan { profile, interface } => {
            commands::scan::run(&ctx, profile, interface.as_deref()).await
        }
        Command::Devices => commands::devices::run(&ctx),
        Command::Device { ip } => commands::device::run(&ctx, &ip),
        Command::Ports { ip, top } => commands::ports::run(&ctx, &ip, top).await,
        Command::Export { what } => match what {
            ExportTarget::Devices { format, output } => {
                commands::export::run(&ctx, format.0, output.as_deref())
            }
        },
        Command::Serve { port } => commands::serve::run(&ctx, port),
    }
}

/// Inicializa `tracing-subscriber`. Sin `--verbose` solo muestra errores.
fn init_tracing(verbose: bool) {
    let level = if verbose { "info" } else { "warn" };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(format!(
            "mylan={level},mylan_discovery={level},mylan_fingerprint={level}"
        ))
        .with_target(false)
        .try_init();
}
