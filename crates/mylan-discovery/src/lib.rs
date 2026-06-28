//! `mylan-discovery` — descubrimiento de hosts en la LAN.
//!
//! Funciones async concretas por técnica (sin trait de estrategia, principio P3) que
//! producen [`Observation`]s de `mylan-core`: detección de interfaz/gateway/CIDR
//! (`netdev`), lectura de `/proc/net/arp`, barrido TCP-connect, mDNS, SSDP, ICMP
//! no-root (best-effort) y, con privilegios, ARP sweep (`pnet_datalink`) + ICMP raw.
//!
//! Pipeline de **dos fases**: la fase liveness (este crate) descubre hosts y emite
//! `Observation`s crudas; la fase enrichment (Paso 6) las interpreta. La fn de alto
//! nivel [`discover`] combina todas las técnicas, deduplica por identidad estable y
//! devuelve el inventario crudo listo para enriquecer y persistir.
//!
//! Privilegios: el flujo base **nunca** requiere root; el camino sudo amplía cobertura
//! con degradación elegante (P1). El descubrimiento es **no intrusivo** (P2): solo
//! sondas pasivas (ARP cache, TCP-connect, eco ICMP, multicast mDNS/SSDP); cero
//! deauth/ARP-spoof/MITM.

#![allow(clippy::module_name_repetitions)]

pub mod arp;
pub mod dns;
pub mod error;
pub mod icmp;
pub mod iface;
pub mod mdns;
pub mod netutil;
pub mod ping;
pub mod ssdp;
pub mod sudo;
pub mod tcp_ping;
// `traceroute` usa `std::os::unix::io` + `nix` (cola de errores `IP_RECVERR`):
// Linux-only. En otras plataformas se expone el stub `traceroute_host` de abajo.
#[cfg(target_os = "linux")]
pub mod traceroute;

pub use arp::{arp_entries_to_observations, parse_arp_table, read_arp_cache, ArpEntry};
pub use dns::{dns_lookup_host, resolve_host, reverse_lookup, system_resolver};
pub use error::DiscoveryError;
pub use iface::{detect_interface, gateway_observations, resolve_gateway_mac, LanInterface};
pub use netutil::enumerate_hosts;
pub use ping::ping_host;
#[cfg(target_os = "linux")]
pub use traceroute::traceroute_host;

use std::time::Duration;

/// Stub de `traceroute_host` para plataformas no-Linux.
///
/// El traceroute real depende de la cola de errores ICMP (`IP_RECVERR` + `nix`),
/// exclusiva de Linux. Fuera de Linux devuelve [`DiscoveryError::UnsupportedPlatform`]
/// en lugar de no compilar (degradación documentada, Paso 0).
#[cfg(not(target_os = "linux"))]
pub async fn traceroute_host(
    _target: std::net::IpAddr,
    _max_hops: u8,
    _timeout: Duration,
) -> Result<Vec<mylan_core::TraceHop>, DiscoveryError> {
    Err(DiscoveryError::UnsupportedPlatform)
}

use mylan_core::{aggregate, Observation, ScanProfile};

/// Opciones de un descubrimiento. Todas las duraciones son por-técnica.
#[derive(Debug, Clone)]
pub struct DiscoverOptions {
    /// Perfil de profundidad (afecta los timeouts por defecto).
    pub profile: ScanProfile,
    /// Override de interfaz; `None` = auto-detectar default route.
    pub interface: Option<String>,
    /// Timeout por intento de conexión TCP (por puerto).
    pub tcp_timeout: Duration,
    /// Tiempo total de escucha ICMP.
    pub icmp_timeout: Duration,
    /// Tiempo total de escucha mDNS.
    pub mdns_timeout: Duration,
    /// Tiempo total de escucha SSDP.
    pub ssdp_timeout: Duration,
    /// Tiempo total de ARP sweep (solo con CAP_NET_RAW).
    pub arp_sweep_timeout: Duration,
    /// Concurrencia máxima del barrido TCP.
    pub concurrency: usize,
}

impl Default for DiscoverOptions {
    fn default() -> Self {
        Self::for_profile(ScanProfile::Quick)
    }
}

impl DiscoverOptions {
    /// Construye las opciones para un perfil dado.
    #[must_use]
    pub fn for_profile(profile: ScanProfile) -> Self {
        match profile {
            ScanProfile::Quick => Self {
                profile,
                interface: None,
                tcp_timeout: Duration::from_millis(400),
                icmp_timeout: Duration::from_secs(2),
                mdns_timeout: Duration::from_secs(3),
                ssdp_timeout: Duration::from_secs(3),
                arp_sweep_timeout: Duration::from_secs(2),
                concurrency: 256,
            },
            // iot/router gobiernan la selección de puertos en `mylan ports`, no
            // el timing del descubrimiento de hosts: para `mylan scan` degradan
            // a timing Normal (P1).
            ScanProfile::Normal | ScanProfile::Iot | ScanProfile::Router => Self {
                profile,
                interface: None,
                tcp_timeout: Duration::from_millis(800),
                icmp_timeout: Duration::from_secs(4),
                mdns_timeout: Duration::from_secs(5),
                ssdp_timeout: Duration::from_secs(5),
                arp_sweep_timeout: Duration::from_secs(4),
                concurrency: 256,
            },
            ScanProfile::Deep => Self {
                profile,
                interface: None,
                tcp_timeout: Duration::from_secs(1),
                icmp_timeout: Duration::from_secs(6),
                mdns_timeout: Duration::from_secs(8),
                ssdp_timeout: Duration::from_secs(8),
                arp_sweep_timeout: Duration::from_secs(6),
                concurrency: 512,
            },
        }
    }
}

/// Ejecuta la **fase liveness** del descubrimiento sobre `iface` y devuelve las
/// [`Observation`]s agregadas (deduplicadas por identidad estable MAC > IP).
///
/// Orden deliberado: las técnicas activas (TCP/ICMP/ARP sweep/mDNS/SSDP) corren en
/// paralelo y, al terminar, **se relee `/proc/net/arp`** para capturar las MACs que el
/// kernel aprendió durante las conexiones — fuente principal de identidad sin root.
/// El gateway conocido de la interfaz se incluye como observación si está presente.
///
/// **Degradación no-Linux (p.ej. Windows):** las técnicas que dependen de
/// `/proc/net/arp` ([`read_arp_cache`]), el ARP sweep (`pnet`) y el barrido ICMP
/// no-root ([`icmp::icmp_sweep`]) devuelven vacío vía sus stubs cfg-gated. El
/// descubrimiento degrada a TCP-connect + mDNS + SSDP + semilla del gateway, todas
/// cross-platform. La cobertura nativa Windows (Win32 IP Helper) es un follow-up.
pub async fn discover(iface: &LanInterface, opts: &DiscoverOptions) -> Vec<Observation> {
    // Técnicas concurrentes.
    let (tcp, icmp_obs, mdns_obs, ssdp_obs, arp_sweep) = tokio::join!(
        tcp_ping::tcp_sweep(iface, opts.tcp_timeout, opts.concurrency),
        icmp::icmp_sweep(iface, opts.icmp_timeout),
        mdns::mdns_discover(iface, opts.mdns_timeout),
        ssdp::ssdp_discover(iface, opts.ssdp_timeout),
        sudo::arp_sweep(iface, opts.arp_sweep_timeout),
    );

    let mut all = Vec::new();
    // Gateway conocido: aporta identidad estable para el router.
    all.extend(gateway_observations(iface.gateway_ip, iface.gateway_mac));
    all.extend(tcp);
    all.extend(icmp_obs);
    all.extend(mdns_obs);
    all.extend(ssdp_obs);
    all.extend(arp_sweep);

    // Relectura post-sweep de /proc/net/arp: captura MACs aprendidas por el kernel
    // durante las conexiones TCP y los ecos ICMP. Es la clave de cobertura sin root.
    // Las entradas incompletas (sin MAC) son hosts que NO respondieron al ARP durante
    // el barrido (hosts muertos): se descartan en `arp_entries_to_observations` para
    // evitar un falso positivo por cada IP sondeada.
    if let Ok(entries) = read_arp_cache() {
        all.extend(arp_entries_to_observations(&entries, Some(&iface.name)));
    }

    aggregate(&all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    fn iface() -> LanInterface {
        LanInterface {
            name: "enp37s0".into(),
            ip: "192.168.1.3".parse().unwrap(),
            prefix_len: 24,
            mac: None,
            gateway_ip: Some("192.168.1.1".parse::<IpAddr>().unwrap()),
            gateway_mac: Some(mylan_core::MacAddr::parse("aa:bb:cc:dd:ee:ff").unwrap()),
            dns_servers: Vec::new(),
        }
    }

    #[test]
    fn quick_defaults_within_scan_budget() {
        let opts = DiscoverOptions::default();
        assert_eq!(opts.profile, ScanProfile::Quick);
        assert_eq!(opts.concurrency, 256);
        // El cuello de botella (mDNS/SSDP) debe ser < 30 s para AC-12.
        assert!(opts.mdns_timeout.as_secs() < 30);
        assert!(opts.ssdp_timeout.as_secs() < 30);
    }

    #[test]
    fn for_profile_deep_is_slower() {
        let quick = DiscoverOptions::for_profile(ScanProfile::Quick);
        let deep = DiscoverOptions::for_profile(ScanProfile::Deep);
        assert!(deep.tcp_timeout > quick.tcp_timeout);
    }

    #[test]
    fn for_profile_covers_all_five_profiles() {
        // iot/router degradan a timing Normal en discovery (gobiernan puertos en
        // `mylan ports`, no el host-discovery).
        let normal = DiscoverOptions::for_profile(ScanProfile::Normal);
        for profile in [
            ScanProfile::Quick,
            ScanProfile::Normal,
            ScanProfile::Deep,
            ScanProfile::Iot,
            ScanProfile::Router,
        ] {
            let opts = DiscoverOptions::for_profile(profile);
            assert_eq!(
                opts.profile, profile,
                "for_profile retiene el perfil pedido"
            );
        }
        let iot = DiscoverOptions::for_profile(ScanProfile::Iot);
        let router = DiscoverOptions::for_profile(ScanProfile::Router);
        assert_eq!(iot.tcp_timeout, normal.tcp_timeout, "iot degrada a Normal");
        assert_eq!(
            router.tcp_timeout, normal.tcp_timeout,
            "router degrada a Normal"
        );
        assert_eq!(iot.concurrency, normal.concurrency);
        assert_eq!(router.concurrency, normal.concurrency);
    }

    #[test]
    fn gateway_seed_produces_one_observation() {
        let iface = iface();
        let seed = gateway_observations(iface.gateway_ip, iface.gateway_mac);
        assert_eq!(seed.len(), 1);
    }

    #[test]
    fn aggregate_dedups_across_techniques() {
        // Tres técnicas ven el mismo host: ARP (MAC+IP), mDNS (MAC+hostname) y un
        // TCP-ping solo-IP. La observación solo-IP debe fundirse en el host-MAC
        // (misma IP) en vez de crear un duplicado (P5): el resultado es 1 device.
        let mac = mylan_core::MacAddr::parse("aa:bb:cc:dd:ee:ff").unwrap();
        let ip: IpAddr = "192.168.1.5".parse().unwrap();
        let obs = vec![
            mylan_core::Observation::new(mylan_core::Source::ArpCache)
                .with_mac(mac)
                .with_ip(ip),
            mylan_core::Observation::new(mylan_core::Source::Mdns)
                .with_mac(mac)
                .with_hostname("nas.local"),
            mylan_core::Observation::new(mylan_core::Source::TcpPing).with_ip(ip),
        ];
        let agg = aggregate(&obs);
        assert_eq!(agg.len(), 1); // un único host: MAC + IP + hostname fusionados
        let host = &agg[0];
        assert_eq!(host.mac, Some(mac));
        assert_eq!(host.ip, Some(ip));
        assert_eq!(host.hostname.as_deref(), Some("nas.local"));
    }
}
