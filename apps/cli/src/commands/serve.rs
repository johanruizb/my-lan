//! `mylan serve` — stub de la API local (fase futura).

use crate::ctx::AppContext;

/// Imprime un aviso: la API local no está implementada en este push.
pub fn run(ctx: &AppContext, port: u16) -> anyhow::Result<()> {
    if ctx.verbose {
        eprintln!("[mylan] 'serve' es un stub planificado para una fase posterior.");
    }
    println!("[mylan] serve --port {port}: no implementado en este push (fase futura).");
    Ok(())
}
