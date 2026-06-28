//! `mylan-scanner` — escaneo de puertos y detección de servicios.
//!
//! Port scan TCP-connect async (perfil `quick`: top 32/100 puertos comunes),
//! concurrente con rate limiting ([`tokio::sync::Semaphore`]), timeout configurable
//! por puerto y plazo global de cancelación. Banner grabbing básico (lectura corta
//! pasiva) + mapeo puerto → `service_name`. Produce [`Service`] de `mylan-core`.
//!
//! Opera sobre **hosts vivos**: recibe una IP ya confirmada por la fase liveness de
//! `mylan-discovery` (principio P1: el port scan es bajo demanda vía `mylan ports`,
//! no dentro del `scan` de descubrimiento, para preservar el presupuesto AC-12).
//!
//! Solo descubre puertos abiertos (P2: scan no intrusivo — conexión TCP normal, sin
//! cargas ofensivas). El [`Service`] devuelto trae `id`/`device_id`/timestamps
//! vacíos: la capa de persistencia (Paso 8) asigna UUID, device_id y RFC3339.
//!
//! La API moderna es [`scan_target`]: recibe un [`ScanProfile`] y un
//! [`CancellationToken`], emite progreso vía callback y devuelve los hits parciales
//! al cancelarse/agotar el plazo (AC-5). Las funciones legacy [`scan_ports`] /
//! [`scan_ports_with`] quedan como envolturas deprecadas.

#![allow(clippy::module_name_repetitions)]

mod banner;
mod ports;
mod probes;
mod profile;
mod udp;

pub use ports::{port_to_service_name, select_ports, COMMON_PORTS};
pub use probes::ProbeResult;
pub use profile::{ports_for_profile, profile_options};

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use mylan_core::{Protocol, ScanProfile, Service, ServiceState};

use banner::grab_banner;
use ports::select_ports as ranked_ports;
use probes::probe_service;

/// Opciones de un escaneo de puertos. Todas las duraciones son acotadas.
#[derive(Debug, Clone, Copy)]
pub struct ScanOptions {
    /// Timeout por intento de conexión TCP (por puerto).
    pub connect_timeout: Duration,
    /// Plazo global del escaneo completo (cancelación cooperativa).
    pub scan_timeout: Duration,
    /// Timeout de la lectura pasiva del banner tras conectar.
    pub banner_timeout: Duration,
    /// Concurrencia máxima (rate limiting por semáforo).
    pub concurrency: usize,
    /// Activar scan UDP limitado (perfil `deep` o flag `--enable-udp`).
    pub enable_udp: bool,
}

impl Default for ScanOptions {
    /// Valores conservadores para una LAN doméstica /24 (AC-12: <30 s).
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_millis(600),
            scan_timeout: Duration::from_secs(20),
            banner_timeout: Duration::from_millis(400),
            concurrency: 128,
            enable_udp: false,
        }
    }
}

/// Progreso de un escaneo emitido vía el callback `on_progress` de [`scan_target`].
#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct ScanProgress {
    /// Porcentaje completado `0..=100`.
    pub percent_done: u8,
    /// Puertos sondeados hasta el momento.
    pub ports_tested: usize,
    /// Total de puertos en el catálogo del perfil.
    pub ports_total: usize,
    /// Último puerto abierto detectado (si aplica).
    pub latest_open_port: Option<u16>,
}

/// Errores de un escaneo ([`scan_target`]).
#[derive(Debug, Error)]
pub enum ScanError {
    /// El escaneo fue cancelado vía [`CancellationToken`] antes de completarse.
    #[error("scan cancelado")]
    Cancelled,
    /// Error de E/S durante el sondeo.
    #[error("E/S: {0}")]
    Io(#[from] std::io::Error),
}

/// Resultado bruto del sondeo de un puerto (antes de construir el [`Service`]).
struct PortHit {
    port: u16,
    banner: Option<String>,
    /// Probe activo (product/version/banner) cuando `profile != Quick`.
    probe: Option<ProbeResult>,
}

/// Escaneo moderno (AC-2, AC-5): sondea el catálogo de `profile` sobre `target`
/// con cancelación cooperativa y progreso en vivo.
///
/// Diseño:
/// - Un *spawn* por puerto (rate-limited por `Semaphore` de `options.concurrency`).
/// - La recogida de hits se hace en la **tarea llamadora** (el callback
///   `on_progress` es `FnMut` y puede no ser `Send`).
/// - `cancel` cancela los spawns hijos (token hijo) y rompe el bucle de recogida.
/// - `scan_timeout` es un plazo global (deadline); al vencer cancela los spawns y
///   devuelve los hits recogidos hasta ese momento.
/// - Al cancelar/agotar plazo se devuelven **hits parciales** (AC-5): no se pierde
///   lo ya detectado.
/// - Tras el barrido TCP, si `options.enable_udp` (perfil `deep`) y no cancelado,
///   se añade el scan UDP limitado ([`udp::scan_udp`], AC-3).
///
/// Los puertos cerrados/filtrados/no alcanzables no producen [`Service`].
pub async fn scan_target(
    target: IpAddr,
    profile: ScanProfile,
    options: ScanOptions,
    cancel: CancellationToken,
    mut on_progress: impl FnMut(ScanProgress),
) -> Result<Vec<Service>, ScanError> {
    let port_list = ports_for_profile(profile);
    let total = port_list.len();
    if total == 0 {
        return Ok(Vec::new());
    }

    let concurrency = options.concurrency.max(1);
    let sem = Arc::new(Semaphore::new(concurrency));
    // Token hijo: cancelar los spawns sin cancelar el recibidor del llamador.
    let probe_cancel = cancel.child_token();
    let (tx, mut rx) = mpsc::channel::<PortHit>(total.max(1));

    let connect_timeout = options.connect_timeout;
    let banner_timeout = options.banner_timeout;
    // Quick sólo hace banner pasivo; el resto ejecuta probes activos.
    let do_probes = !matches!(profile, ScanProfile::Quick);

    for port in port_list {
        let tx = tx.clone();
        let sem = sem.clone();
        let probe_cancel = probe_cancel.clone();
        tokio::spawn(async move {
            if probe_cancel.is_cancelled() {
                return;
            }
            // Rate limiting: acquire cooperativo (ignora cancelación del semáforo).
            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => return,
            };
            if probe_cancel.is_cancelled() {
                return;
            }
            let addr = SocketAddr::new(target, port);
            let connect = TcpStream::connect(addr);
            let stream = tokio::time::timeout(connect_timeout, connect).await;
            if let Ok(Ok(mut stream)) = stream {
                let banner = grab_banner(&mut stream, banner_timeout).await;
                let probe = if do_probes {
                    probe_service(&mut stream, port, banner_timeout).await
                } else {
                    None
                };
                let _ = tx
                    .send(PortHit {
                        port,
                        banner,
                        probe,
                    })
                    .await;
            }
        });
    }
    // Soltar el tx original: cuando todos los spawns terminen, rx.recv() → None.
    drop(tx);

    let mut hits = Vec::new();
    let mut tested = 0usize;
    // Deadline global fijo (no se reinicia en cada iteración del select).
    let deadline = tokio::time::sleep(options.scan_timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => break,
            _ = &mut deadline => {
                probe_cancel.cancel();
                break;
            },
            recv = rx.recv() => match recv {
                Some(hit) => {
                    tested += 1;
                    on_progress(ScanProgress {
                        percent_done: u8::try_from((tested * 100) / total.max(1))
                            .unwrap_or(100),
                        ports_tested: tested,
                        ports_total: total,
                        latest_open_port: Some(hit.port),
                    });
                    hits.push(hit);
                }
                None => break,
            },
        }
    }

    let mut services: Vec<Service> = hits.into_iter().map(to_service_with_probe).collect();

    // AC-3: scan UDP limitado tras el barrido TCP (perfil deep / flag).
    if options.enable_udp && !cancel.is_cancelled() {
        services.extend(udp::scan_udp(target, &options, &cancel).await);
    }

    Ok(services)
}

/// Construye un [`Service`] TCP a partir de un hit abierto, enriquecido con el
/// probe activo (product/version) cuando esté disponible. Identidad/timestamps
/// vacíos: los asigna la persistencia.
fn to_service_with_probe(hit: PortHit) -> Service {
    let product = hit.probe.as_ref().and_then(|p| p.product.clone());
    let version = hit.probe.as_ref().and_then(|p| p.version.clone());
    // El banner del probe (si lo extrajo) tiene prioridad sobre el pasivo.
    let banner = hit
        .probe
        .as_ref()
        .and_then(|p| p.banner.clone())
        .or(hit.banner);
    Service {
        id: String::new(),
        device_id: String::new(),
        protocol: Protocol::Tcp,
        port: hit.port,
        service_name: port_to_service_name(hit.port).map(String::from),
        product,
        version,
        banner,
        state: ServiceState::Open,
        first_seen_at: String::new(),
        last_seen_at: String::new(),
    }
}

/// Escanea los `top` puertos más comunes de `ip` con opciones por defecto.
///
/// Devuelve un [`Service`] por cada puerto abierto detectado, con `protocol`,
/// `port`, `service_name` y `banner` rellenos; `state = Open`. Los campos
/// `id`/`device_id`/`first_seen_at`/`last_seen_at` quedan vacíos para que la capa
/// de persistencia los asigne.
#[deprecated(note = "usar scan_target")]
pub async fn scan_ports(ip: IpAddr, top: u16) -> Vec<Service> {
    #[allow(deprecated)]
    scan_ports_with(ip, top, &ScanOptions::default()).await
}

/// Escaneo configurable: sondea los `top` puertos de `ip` bajo `opts`.
///
/// La concurrencia se acota con un [`Semaphore`]; cada puerto tiene su propio
/// `connect_timeout`. El plazo global `scan_timeout` cancela el escaneo completo
/// (cancelación cooperativa): al vencer, se devuelven los hits recogidos hasta ese
/// momento. Los puertos cerrados/filtrados/no alcanzables no producen [`Service`].
#[deprecated(note = "usar scan_target")]
pub async fn scan_ports_with(ip: IpAddr, top: u16, opts: &ScanOptions) -> Vec<Service> {
    let port_list = ranked_ports(top);
    if port_list.is_empty() {
        return Vec::new();
    }
    let concurrency = opts.concurrency.max(1);
    let sem = Arc::new(Semaphore::new(concurrency));
    let banner_timeout = opts.banner_timeout;

    let scan = async {
        let handles: Vec<JoinHandle<Option<PortHit>>> = port_list
            .into_iter()
            .map(|port| {
                let sem = sem.clone();
                tokio::spawn(probe_port(
                    ip,
                    port,
                    opts.connect_timeout,
                    banner_timeout,
                    sem,
                ))
            })
            .collect();
        let mut hits = Vec::new();
        for handle in handles {
            if let Ok(Some(hit)) = handle.await {
                hits.push(hit);
            }
        }
        hits
    };

    // Plazo global: al vencer aborta los spawns y devuelve lo recogido hasta ahora.
    let hits: Vec<PortHit> = tokio::time::timeout(opts.scan_timeout, scan)
        .await
        .unwrap_or_default();
    hits.into_iter().map(to_service).collect()
}

/// Sondea un puerto: conecta con timeout y, si abre, captura el banner.
async fn probe_port(
    ip: IpAddr,
    port: u16,
    connect_timeout: Duration,
    banner_timeout: Duration,
    sem: Arc<Semaphore>,
) -> Option<PortHit> {
    let _permit = sem.acquire().await.ok()?;
    let addr = SocketAddr::new(ip, port);
    let connect = TcpStream::connect(addr);
    let mut stream = tokio::time::timeout(connect_timeout, connect)
        .await
        .ok()?
        .ok()?;
    let banner = grab_banner(&mut stream, banner_timeout).await;
    Some(PortHit {
        port,
        banner,
        probe: None,
    })
}

/// Construye un [`Service`] a partir de un puerto abierto (legacy, sin probe).
/// Identidad/timestamps vacíos: los asigna la persistencia.
fn to_service(hit: PortHit) -> Service {
    Service {
        id: String::new(),
        device_id: String::new(),
        protocol: Protocol::Tcp,
        port: hit.port,
        service_name: port_to_service_name(hit.port).map(String::from),
        product: None,
        version: None,
        banner: hit.banner,
        state: ServiceState::Open,
        first_seen_at: String::new(),
        last_seen_at: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tokio::io::AsyncWriteExt;

    /// Servicio construido desde un hit abierto rellena los campos de escaneo y deja
    /// vacíos los de persistencia.
    #[test]
    fn open_hit_maps_to_service() {
        let svc = to_service(PortHit {
            port: 22,
            banner: Some("SSH-2.0-OpenSSH_8.9".into()),
            probe: None,
        });
        assert_eq!(svc.protocol, Protocol::Tcp);
        assert_eq!(svc.port, 22);
        assert_eq!(svc.service_name.as_deref(), Some("ssh"));
        assert_eq!(svc.banner.as_deref(), Some("SSH-2.0-OpenSSH_8.9"));
        assert_eq!(svc.state, ServiceState::Open);
        assert_eq!(svc.id, "");
        assert_eq!(svc.device_id, "");
        assert_eq!(svc.first_seen_at, "");
        assert_eq!(svc.last_seen_at, "");
    }

    /// `to_service_with_probe` enriquece product/version desde el probe y deja el
    /// banner del probe con prioridad sobre el pasivo.
    #[test]
    fn probe_hit_enriches_service() {
        let svc = to_service_with_probe(PortHit {
            port: 80,
            banner: Some("pasivo".into()),
            probe: Some(ProbeResult {
                product: Some("nginx".into()),
                version: Some("1.2.3".into()),
                banner: Some("MyPage".into()),
            }),
        });
        assert_eq!(svc.product.as_deref(), Some("nginx"));
        assert_eq!(svc.version.as_deref(), Some("1.2.3"));
        // Banner del probe gana sobre el pasivo.
        assert_eq!(svc.banner.as_deref(), Some("MyPage"));
        assert_eq!(svc.state, ServiceState::Open);
    }

    /// Sin probe, `to_service_with_probe` cae al banner pasivo.
    #[test]
    fn probe_hit_without_probe_falls_back_to_banner() {
        let svc = to_service_with_probe(PortHit {
            port: 22,
            banner: Some("SSH-2.0-x".into()),
            probe: None,
        });
        assert_eq!(svc.product, None);
        assert_eq!(svc.version, None);
        assert_eq!(svc.banner.as_deref(), Some("SSH-2.0-x"));
    }

    /// Mapeo puerto → service_name integrado con la selección de puertos.
    #[test]
    fn selected_ports_map_to_known_services() {
        let ports = select_ports(32);
        for &p in &[80, 443, 22, 445, 53, 3306] {
            assert!(ports.contains(&p), "top 32 incluye {p}");
            assert!(port_to_service_name(p).is_some(), "{p} tiene nombre");
        }
    }

    /// El plazo global se respeta: un escaneo contra una IP blackhole con un plazo
    /// corto regresa dentro de un margen acotado (cancelación cooperativa).
    #[tokio::test(flavor = "current_thread")]
    #[allow(deprecated)]
    async fn scan_respects_global_timeout() {
        // 240.0.0.1 = TEST-NET-1 (RFC 5737): no enrutable, las conexiones caducan.
        let ip: IpAddr = "240.0.0.1".parse().unwrap();
        let opts = ScanOptions {
            connect_timeout: Duration::from_millis(80),
            scan_timeout: Duration::from_millis(250),
            banner_timeout: Duration::from_millis(80),
            concurrency: 16,
            enable_udp: false,
        };
        let start = Instant::now();
        let svcs = scan_ports_with(ip, 32, &opts).await;
        let elapsed = start.elapsed();
        assert!(svcs.is_empty(), "TEST-NET no abre puertos");
        assert!(
            elapsed <= opts.scan_timeout * 3,
            "transcurrido={elapsed:?} fuera de margen"
        );
    }

    /// `scan_ports` con top 0 no sondea nada y regresa de inmediato.
    #[tokio::test(flavor = "current_thread")]
    #[allow(deprecated)]
    async fn top_zero_returns_empty() {
        let svcs = scan_ports("127.0.0.1".parse().unwrap(), 0).await;
        assert!(svcs.is_empty());
    }

    /// Un puerto abierto local se detecta como hit con state Open.
    #[tokio::test(flavor = "current_thread")]
    async fn open_local_port_is_detected() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            if let Ok((mut s, _)) = listener.accept().await {
                let _ = s.write_all(b"hello\r\n").await;
            }
        });
        let sem = Arc::new(Semaphore::new(4));
        let hit = probe_port(
            "127.0.0.1".parse().unwrap(),
            port,
            Duration::from_millis(500),
            Duration::from_millis(200),
            sem,
        )
        .await
        .expect("puerto local abierto");
        assert_eq!(hit.port, port);
        assert_eq!(to_service(hit).state, ServiceState::Open);
    }

    /// `enable_udp` es `false` por defecto (sólo el perfil `deep` o el flag
    /// `--enable-udp` lo activan).
    #[test]
    fn scan_options_default_disables_udp() {
        let opts = ScanOptions::default();
        assert!(!opts.enable_udp, "Default desactiva UDP");
    }

    /// `scan_target` contra una IP blackhole regresa vacío dentro del plazo global.
    #[tokio::test(flavor = "current_thread")]
    async fn scan_target_blackhole_returns_empty_within_timeout() {
        let ip: IpAddr = "240.0.0.1".parse().unwrap();
        let opts = ScanOptions {
            connect_timeout: Duration::from_millis(80),
            scan_timeout: Duration::from_millis(250),
            banner_timeout: Duration::from_millis(80),
            concurrency: 16,
            enable_udp: false,
        };
        let cancel = CancellationToken::new();
        let start = Instant::now();
        let svcs = scan_target(ip, ScanProfile::Quick, opts, cancel, |_| ())
            .await
            .expect("ok");
        let elapsed = start.elapsed();
        assert!(svcs.is_empty(), "TEST-NET no abre puertos");
        assert!(
            elapsed <= Duration::from_secs(2),
            "transcurrido={elapsed:?} fuera de margen"
        );
    }

    /// `scan_target` cancelado antes de empezar regresa rápido (sin sondeos).
    #[tokio::test(flavor = "current_thread")]
    async fn scan_target_cancelled_returns_quickly() {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let start = Instant::now();
        let svcs = scan_target(
            ip,
            ScanProfile::Quick,
            ScanOptions::default(),
            cancel,
            |_| (),
        )
        .await
        .expect("ok");
        let elapsed = start.elapsed();
        // Cancelado: los spawns abortan y el bucle rompe de inmediato.
        assert!(
            elapsed <= Duration::from_secs(1),
            "transcurrido={elapsed:?}"
        );
        // Puede haber algún hit suelto si un spawn ya había conectado a 127.0.0.1
        // antes de observar la cancelación; no afirmamos vacío, sólo rapidez.
        let _ = svcs;
    }

    /// `scan_target` detecta un puerto abierto del catálogo `iot` (best-effort:
    /// si el puerto no se puede bindear localmente, se omite el test).
    #[tokio::test(flavor = "current_thread")]
    async fn scan_target_detects_open_iot_port() {
        // 7547 (TR-069) está en IOT_PORTS y es >1024 (bindeable sin root).
        const PROBE_PORT: u16 = 7547;
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", PROBE_PORT)).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("skip: no se pudo bindear 127.0.0.1:{PROBE_PORT}: {e}");
                return;
            }
        };
        tokio::spawn(async move {
            if let Ok((mut s, _)) = listener.accept().await {
                let _ = s.write_all(b"hello\r\n").await;
            }
        });

        let opts = ScanOptions {
            connect_timeout: Duration::from_millis(300),
            scan_timeout: Duration::from_secs(3),
            banner_timeout: Duration::from_millis(200),
            concurrency: 16,
            enable_udp: false,
        };
        let cancel = CancellationToken::new();
        let svcs = scan_target(
            "127.0.0.1".parse().unwrap(),
            ScanProfile::Iot,
            opts,
            cancel,
            |_| (),
        )
        .await
        .expect("ok");

        let hit = svcs
            .iter()
            .find(|s| s.port == PROBE_PORT && s.protocol == Protocol::Tcp);
        assert!(hit.is_some(), "7547 abierto debe detectarse");
        assert_eq!(hit.unwrap().state, ServiceState::Open);
    }
}
