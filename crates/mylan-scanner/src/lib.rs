//! `mylan-scanner` — escaneo de puertos y detección de servicios.
//!
//! Port scan TCP-connect asíncrono sobre un catálogo de puertos definido por perfil
//! ([`ScanProfile`]), concurrente con rate limiting ([`tokio::sync::Semaphore`]),
//! timeout configurable por puerto y plazo global de cancelación vía
//! [`CancellationToken`]. Banner grabbing pasivo + probe activo (producto/versión)
//! + mapeo puerto → `service_name`. Produce [`Service`] de `mylan-core`.
//!
//! Opera sobre **hosts vivos** ya confirmados por la fase de liveness de
//! `mylan-discovery` — no re-descubre. Solo detecta puertos abiertos (no
//! intrusivo, P2). Los [`Service`] devueltos traen `id`/`device_id`/timestamps
//! vacíos: la capa de persistencia los asigna.
//!
//! La API principal es [`scan_target`]: recibe un [`ScanProfile`] y un
//! [`CancellationToken`], emite progreso vía callback `FnMut(ScanProgress)` y
//! devuelve hits parciales al cancelarse o agotar el plazo (AC-5).

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
use tokio_util::sync::CancellationToken;

use mylan_core::{Protocol, ScanProfile, Service, ServiceState};

use banner::grab_banner;
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

/// Mensaje del canal de sondeo: un puerto abierto (hit) o cerrado/filtrado (miss).
/// Ambos avanzan el progreso; sólo los hits producen [`Service`].
enum ScanMsg {
    /// Puerto abierto: produce un [`Service`] vía [`to_service_with_probe`].
    Hit(PortHit),
    /// Puerto cerrado/filtrado/no alcanzable: avanza el progreso sin service.
    Miss,
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
    let (tx, mut rx) = mpsc::channel::<ScanMsg>(total.max(1));

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
            match stream {
                Ok(Ok(mut stream)) => {
                    let banner = grab_banner(&mut stream, banner_timeout).await;
                    let probe = if do_probes {
                        probe_service(&mut stream, port, banner_timeout).await
                    } else {
                        None
                    };
                    let _ = tx
                        .send(ScanMsg::Hit(PortHit {
                            port,
                            banner,
                            probe,
                        }))
                        .await;
                }
                // Puerto cerrado/filtrado/timeout: avanza el progreso sin hit.
                _ => {
                    let _ = tx.send(ScanMsg::Miss).await;
                }
            }
        });
    }
    // Soltar el tx original: cuando todos los spawns terminen, rx.recv() → None.
    drop(tx);

    let mut hits = Vec::new();
    let mut probed = 0usize;
    let mut latest_open_port: Option<u16> = None;
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
                Some(msg) => {
                    probed += 1;
                    match msg {
                        ScanMsg::Hit(hit) => {
                            latest_open_port = Some(hit.port);
                            hits.push(hit);
                        }
                        ScanMsg::Miss => {}
                    }
                    // Throttle: cada 8 puertos o el último, para no inundar el
                    // callback en catálogos anchos (patrón de tcp_ping.rs).
                    if probed.is_multiple_of(8) || probed == total {
                        on_progress(ScanProgress {
                            percent_done: u8::try_from((probed * 100) / total.max(1))
                                .unwrap_or(100),
                            ports_tested: probed,
                            ports_total: total,
                            latest_open_port,
                        });
                    }
                }
                None => {
                    // Avance terminal: lleva la barra al 100% en éxito (no en
                    // cancelación/timeout, donde se congela en su último valor).
                    if !cancel.is_cancelled() {
                        on_progress(ScanProgress {
                            percent_done: u8::try_from((probed * 100) / total.max(1))
                                .unwrap_or(100),
                            ports_tested: probed,
                            ports_total: total,
                            latest_open_port,
                        });
                    }
                    break;
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tokio::io::AsyncWriteExt;

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

    /// `scan_target` reporta progreso también para puertos cerrados, no solo
    /// abiertos. Antes del fix, `on_progress` sólo disparaba en hits (puertos
    /// abiertos), dejando la barra congelada cerca del 0% hasta el final.
    #[tokio::test(flavor = "current_thread")]
    async fn scan_target_reports_progress_for_closed_ports() {
        let ip: IpAddr = "240.0.0.1".parse().unwrap();
        let opts = ScanOptions {
            connect_timeout: Duration::from_millis(80),
            scan_timeout: Duration::from_secs(2),
            banner_timeout: Duration::from_millis(80),
            concurrency: 16,
            enable_udp: false,
        };
        let cancel = CancellationToken::new();
        let mut progress_calls = Vec::new();
        let svcs = scan_target(
            ip,
            ScanProfile::Quick,
            opts,
            cancel,
            |p| progress_calls.push(p),
        )
        .await
        .expect("ok");

        // TEST-NET: todos los puertos cerrados → 0 servicios.
        assert!(svcs.is_empty(), "TEST-NET no abre puertos");
        // El progreso debe disparar aunque no haya puertos abiertos.
        assert!(
            !progress_calls.is_empty(),
            "debe haber progreso para puertos cerrados"
        );
        // Con throttle cada 8 y Quick (32 puertos): dispara en 8/16/24/32 + final.
        assert!(
            progress_calls.len() >= 2,
            "debe disparar múltiples veces, no solo al final"
        );
        // El último reporte debe llevar ports_tested al total y 100%.
        let last = progress_calls.last().unwrap();
        assert_eq!(last.ports_total, last.ports_tested, "debe sondear todos los puertos");
        assert_eq!(last.percent_done, 100, "debe terminar en 100%");
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
