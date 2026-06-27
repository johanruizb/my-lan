//! Errores de `mylan-discovery`.
//!
//! `thiserror` (lib, no app): los errores son estructurados para que la CLI y los
//! tests puedan reaccionar a variantes concretas (p.ej. falta de iface por defecto).

use thiserror::Error;

/// Error de descubrimiento de red.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// No se encontró una interfaz por defecto utilizable (todas filtradas o DOWN).
    #[error("no default network interface found (loopback/docker/tailscale filtered or all DOWN)")]
    NoDefaultInterface,
    /// La interfaz solicitada vía override no existe o no es utilizable.
    #[error("interface `{0}` not found or not usable")]
    InterfaceNotFound(String),
    /// La interfaz no tiene dirección IPv4.
    #[error("interface `{name}` has no IPv4 address")]
    NoIpv4 { name: String },
    /// `netdev` devuelve `Result<_, String>`; lo envolvemos aquí.
    #[error("netdev: {0}")]
    Netdev(String),
    /// Error de E/S subyacente (lectura de `/proc/net/arp`, sockets, etc.).
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Error de resolución DNS (hickory-resolver) o de configuración del
    /// resolver del sistema. String legible; no expone el tipo de hickory para
    /// mantener el error estable entre versiones del crate.
    #[error("dns: {0}")]
    Dns(String),
}

impl From<String> for DiscoveryError {
    /// Convierte el `String` de error de `netdev` en [`DiscoveryError::Netdev`].
    fn from(value: String) -> Self {
        Self::Netdev(value)
    }
}
