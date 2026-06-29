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
pub mod ssid;
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
pub use ssid::{detect_ssid, SsidDetector};
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
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

/// Evento del descubrimiento en streaming: un host descubierto o un avance del
/// barrido (IPs sondeadas / total del CIDR). Comparten un único canal para que el
/// consumidor los drene en un solo bucle.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// Una [`Observation`] cruda de un host (cualquier técnica).
    Host(Observation),
    /// Avance del barrido TCP: `swept` IPs sondeadas de `total` en el CIDR.
    Progress { swept: u32, total: u32 },
}

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
    // Adaptador batch sobre `discover_stream`: drena el canal a un Vec y agrega.
    // Mantiene la firma pública para la CLI; el token nunca se cancela.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<DiscoveryEvent>();
    let cancel = CancellationToken::new();
    let mut all = Vec::new();
    let drain = async {
        while let Some(ev) = rx.recv().await {
            if let DiscoveryEvent::Host(obs) = ev {
                all.push(obs);
            }
        }
    };
    tokio::join!(discover_stream(iface, opts, tx, cancel), drain);
    aggregate(&all)
}

/// Variante en streaming de [`discover`]: emite cada host y cada avance del barrido
/// por `tx` a medida que se descubren, en vez de devolver un `Vec` agregado al final.
///
/// Emite primero `Progress{0, total}` (total = hosts del CIDR vía
/// [`enumerate_hosts`]) y la semilla del gateway. Lanza las 5 técnicas en paralelo:
/// TCP e ICMP transmiten de forma nativa (con cancelación cooperativa); mDNS, SSDP y
/// el ARP sweep se esperan dentro de un `tokio::select!` contra `cancel` y se
/// reenvían en bloque sin tocar sus módulos. Un poller periódico de `/proc/net/arp`
/// (cada 750 ms) hace que las MAC/vendor lleguen en vivo; una relectura final
/// preserva la semántica del barrido batch. Al retornar se suelta `tx`, cerrando el
/// canal. Sólo TCP emite `Progress` (única fuente de avance, evita overshoot).
pub async fn discover_stream(
    iface: &LanInterface,
    opts: &DiscoverOptions,
    tx: UnboundedSender<DiscoveryEvent>,
    cancel: CancellationToken,
) {
    let total =
        u32::try_from(enumerate_hosts(iface.ip, iface.prefix_len).len()).unwrap_or(u32::MAX);
    let _ = tx.send(DiscoveryEvent::Progress { swept: 0, total });

    // Gateway conocido: aporta identidad estable para el router.
    for obs in gateway_observations(iface.gateway_ip, iface.gateway_mac) {
        let _ = tx.send(DiscoveryEvent::Host(obs));
    }

    // Poller periódico de la caché ARP: emite las MAC que el kernel aprende durante
    // los barridos TCP/ICMP en vivo. Se detiene con `poller_cancel` al terminar.
    let poller_cancel = cancel.child_token();
    let poller_stop = poller_cancel.clone();
    let poller_tx = tx.clone();
    let iface_name = iface.name.clone();
    let poller = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(750));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Ok(entries) = read_arp_cache() {
                        for obs in arp_entries_to_observations(&entries, Some(&iface_name)) {
                            let _ = poller_tx.send(DiscoveryEvent::Host(obs));
                        }
                    }
                }
                () = poller_cancel.cancelled() => break,
            }
        }
    });

    // Técnicas concurrentes. TCP/ICMP transmiten nativamente; mDNS/SSDP/ARP sweep se
    // esperan bajo `select!` contra `cancel` y se reenvían en bloque (sin editarlas).
    tokio::join!(
        tcp_ping::tcp_sweep(
            iface,
            opts.tcp_timeout,
            opts.concurrency,
            tx.clone(),
            cancel.clone(),
        ),
        icmp::icmp_sweep(iface, opts.icmp_timeout, tx.clone(), cancel.clone()),
        forward_burst(mdns::mdns_discover(iface, opts.mdns_timeout), &tx, &cancel),
        forward_burst(ssdp::ssdp_discover(iface, opts.ssdp_timeout), &tx, &cancel),
        forward_burst(sudo::arp_sweep(iface, opts.arp_sweep_timeout), &tx, &cancel),
    );

    // Detiene el poller y relee /proc/net/arp una última vez: captura las MAC
    // aprendidas por el kernel durante las conexiones (preserva la semántica batch).
    poller_stop.cancel();
    let _ = poller.await;
    if let Ok(entries) = read_arp_cache() {
        for obs in arp_entries_to_observations(&entries, Some(&iface.name)) {
            let _ = tx.send(DiscoveryEvent::Host(obs));
        }
    }

    // Avance terminal -> lleva la barra al 100% en éxito. En cancelación se congela
    // en su último valor (no se fuerza al 100%).
    if !cancel.is_cancelled() {
        let _ = tx.send(DiscoveryEvent::Progress {
            swept: total,
            total,
        });
    }
}

/// Espera una técnica de tipo `Vec<Observation>` bajo `cancel` y reenvía cada
/// observación como [`DiscoveryEvent::Host`]. Si `cancel` se dispara antes, la
/// técnica se cae (drop) sin reenviar nada.
async fn forward_burst(
    fut: impl std::future::Future<Output = Vec<Observation>>,
    tx: &UnboundedSender<DiscoveryEvent>,
    cancel: &CancellationToken,
) {
    tokio::select! {
        obs = fut => {
            for o in obs {
                let _ = tx.send(DiscoveryEvent::Host(o));
            }
        }
        () = cancel.cancelled() => {}
    }
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
            ssid: None,
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
