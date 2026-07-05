//! Descubrimiento con degradación de privilegios (P1).
//!
//! [`discover_with_degradation`] intenta el descubrimiento de la LAN; si los
//! privilegios son insuficientes (sudo no disponible para ARP sweep), las
//! técnicas no-privilegiadas (TCP-connect, ICMP no-root, mDNS, SSDP) siguen
//! funcionando vía `mylan_discovery::discover`, que ya degrada internamente.
//! Nunca crash: si la detección de interfaz falla, devuelve vacío + log.

use mylan_core::{Observation, ScanProfile};
use mylan_discovery::{detect_interface, discover, DiscoverOptions};

/// Descubre hosts en la LAN, con degradación elegante de privilegios.
///
/// `cidr` se usa para logging; `profile` controla los timeouts/técnicas del
/// descubrimiento (M5 fix: antes hardcodeado a `Quick`, ignorando el perfil del
/// config). El descubrimiento usa la interfaz de default route
/// (`detect_interface`). Si la detección falla, devuelve `Vec::new()` (no
/// crash). Sin sudo, el path de fallback (ICMP/TCP-ping/mDNS/SSDP) se ejecuta
/// dentro de `discover`.
pub async fn discover_with_degradation(cidr: &str, profile: ScanProfile) -> Vec<Observation> {
    let iface = match detect_interface(None) {
        Ok(i) => i,
        Err(e) => {
            tracing::warn!(
                %cidr,
                error = %e,
                "no se pudo detectar interfaz LAN; degradación a descubrimiento vacío",
            );
            return Vec::new();
        }
    };
    tracing::debug!(%cidr, iface = %iface.name, profile = ?profile, "descubriendo vía interfaz default route");
    let opts = DiscoverOptions::for_profile(profile);
    discover(&iface, &opts).await
}
