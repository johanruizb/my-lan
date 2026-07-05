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
