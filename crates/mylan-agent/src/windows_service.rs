//! Wrapper del Windows service (`#[cfg(windows)]`, Step 5 packaging).
//!
//! En Windows usa el crate `windows-service` (en
//! `[target.'cfg(windows)'.dependencies]`) para registrar/desregistrar el
//! binario como servicio. En otras plataformas expone un stub que reporta "no
//! soportado". La implementación completa (install/uninstall vía `sc create`)
//! se completa en Step 5 (packaging).

#[cfg(windows)]
pub fn install_service() -> anyhow::Result<()> {
    // TODO(Step 5): integración con `windows-service` crate —
    // `ServiceManager::local_computer(None)` + `create_service(...)` con
    // `ServiceInfo { name, display_name, service_type: ServiceType::OWN_PROCESS,
    // start_type: ServiceStartType::Auto, ... }`. Por ahora stub pendiente.
    anyhow::bail!("windows-service install: implementación pendiente (Step 5 packaging)")
}

#[cfg(not(windows))]
pub fn install_service() -> anyhow::Result<()> {
    anyhow::bail!("Windows service management solo disponible en Windows")
}

#[cfg(test)]
mod tests {
    use super::*;

    // En plataformas no-Windows, `install_service` es un stub determinista que
    // siempre devuelve Err con un mensaje fijo. En Windows, la implementación
    // real (Step 5) también devuelve Err (stub pendiente); test cfg-gated.

    #[cfg(not(windows))]
    #[test]
    fn install_service_errors_on_non_windows() {
        let result = install_service();
        assert!(result.is_err(), "stub non-Windows debe errar");
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("Windows"),
            "mensaje debe mencionar Windows: {msg}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn install_service_stub_errors_on_windows() {
        // Hasta Step 5, el stub Windows también devuelve Err (pendiente).
        let result = install_service();
        assert!(result.is_err(), "stub Windows debe errar (pendiente)");
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("pendiente") || msg.contains("Step 5"),
            "mensaje debe indicar implementación pendiente: {msg}"
        );
    }
}
