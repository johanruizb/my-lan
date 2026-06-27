//! Mapeo `ScanProfile` → catálogo de puertos + `ScanOptions` (plan §7.3, AC-1).
//!
//! Los perfiles `quick`/`normal`/`deep` reutilizan el catálogo ranqueado
//! [`crate::ports::COMMON_PORTS`] vía [`select_ports`]; `iot`/`router` usan
//! catálogos fijos orientados a los puertos típicos de esos dispositivos.
//! `deep` (o el flag `--enable-udp` del CLI) activa el scan UDP limitado (Paso 3).

use std::time::Duration;

use mylan_core::ScanProfile;

use crate::{select_ports, ScanOptions};

/// Puertos fijos del perfil `iot` (RTSP/ONVIF, HTTP/S, MQTT, CoAP, UPnP, TR-069).
const IOT_PORTS: &[u16] = &[554, 80, 443, 1883, 5683, 1900, 5000, 7547];

/// Puertos fijos del perfil `router` (admin/SSH/Telnet/DNS/DHCP/UPnP/TR-069).
const ROUTER_PORTS: &[u16] = &[22, 23, 80, 443, 1900, 53, 67, 7547];

/// Devuelve el catálogo de puertos TCP para un perfil (AC-1).
///
/// `quick`/`normal`/`deep` reutilizan [`select_ports`] (top 32/100/1000; el
/// catálogo acota a ~100 puertos hoy); `iot`/`router` devuelven catálogos fijos.
#[must_use]
pub fn ports_for_profile(profile: ScanProfile) -> Vec<u16> {
    match profile {
        ScanProfile::Quick => select_ports(32),
        ScanProfile::Normal => select_ports(100),
        ScanProfile::Deep => select_ports(1000),
        ScanProfile::Iot => IOT_PORTS.to_vec(),
        ScanProfile::Router => ROUTER_PORTS.to_vec(),
    }
}

/// Devuelve las `ScanOptions` (timeouts/concurrencia/UDP) para un perfil (AC-1).
#[must_use]
pub fn profile_options(profile: ScanProfile) -> ScanOptions {
    match profile {
        ScanProfile::Quick => ScanOptions {
            connect_timeout: Duration::from_secs(1),
            banner_timeout: Duration::from_millis(500),
            concurrency: 100,
            scan_timeout: Duration::from_secs(15),
            enable_udp: false,
        },
        ScanProfile::Normal => ScanOptions {
            connect_timeout: Duration::from_secs(2),
            banner_timeout: Duration::from_secs(1),
            concurrency: 100,
            scan_timeout: Duration::from_secs(30),
            enable_udp: false,
        },
        ScanProfile::Deep => ScanOptions {
            connect_timeout: Duration::from_secs(3),
            banner_timeout: Duration::from_secs(2),
            concurrency: 50,
            scan_timeout: Duration::from_secs(90),
            enable_udp: true,
        },
        // iot/router: gobiernan puertos (catálogos fijos), no timing; timing
        // equivalente a normal con concurrencia 80.
        ScanProfile::Iot | ScanProfile::Router => ScanOptions {
            connect_timeout: Duration::from_secs(2),
            banner_timeout: Duration::from_secs(1),
            concurrency: 80,
            scan_timeout: Duration::from_secs(15),
            enable_udp: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_maps_top_32() {
        assert_eq!(ports_for_profile(ScanProfile::Quick).len(), 32);
    }

    #[test]
    fn normal_maps_top_100() {
        assert_eq!(ports_for_profile(ScanProfile::Normal).len(), 100);
    }

    #[test]
    fn deep_enables_udp_and_is_slower_than_quick() {
        let deep = profile_options(ScanProfile::Deep);
        let quick = profile_options(ScanProfile::Quick);
        assert!(deep.enable_udp, "deep activa UDP");
        assert!(!quick.enable_udp, "quick no activa UDP");
        assert!(deep.connect_timeout > quick.connect_timeout);
    }

    #[test]
    fn deep_selects_at_least_normal_catalog() {
        // select_ports(1000) acota al catálogo; deep no selecciona menos que normal.
        assert!(
            ports_for_profile(ScanProfile::Deep).len()
                >= ports_for_profile(ScanProfile::Normal).len()
        );
    }

    #[test]
    fn iot_ports_are_fixed_catalog() {
        assert_eq!(
            ports_for_profile(ScanProfile::Iot),
            vec![554, 80, 443, 1883, 5683, 1900, 5000, 7547]
        );
        let opts = profile_options(ScanProfile::Iot);
        assert_eq!(opts.concurrency, 80);
        assert!(!opts.enable_udp);
    }

    #[test]
    fn router_ports_are_fixed_catalog() {
        assert_eq!(
            ports_for_profile(ScanProfile::Router),
            vec![22, 23, 80, 443, 1900, 53, 67, 7547]
        );
        let opts = profile_options(ScanProfile::Router);
        assert_eq!(opts.concurrency, 80);
        assert!(!opts.enable_udp);
    }

    #[test]
    fn all_profiles_produce_non_empty_ports() {
        for profile in [
            ScanProfile::Quick,
            ScanProfile::Normal,
            ScanProfile::Deep,
            ScanProfile::Iot,
            ScanProfile::Router,
        ] {
            assert!(
                !ports_for_profile(profile).is_empty(),
                "{profile:?} sin puertos"
            );
        }
    }
}
