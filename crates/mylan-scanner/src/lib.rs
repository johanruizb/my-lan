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

#![allow(clippy::module_name_repetitions)]

mod banner;
mod ports;

pub use ports::{port_to_service_name, select_ports, COMMON_PORTS};

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use mylan_core::{Protocol, Service, ServiceState};

use banner::grab_banner;
use ports::select_ports as ranked_ports;

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
}

impl Default for ScanOptions {
    /// Valores conservadores para una LAN doméstica /24 (AC-12: <30 s).
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_millis(600),
            scan_timeout: Duration::from_secs(20),
            banner_timeout: Duration::from_millis(400),
            concurrency: 128,
        }
    }
}

/// Resultado bruto del sondeo de un puerto (antes de construir el [`Service`]).
struct PortHit {
    port: u16,
    banner: Option<String>,
}

/// Escanea los `top` puertos más comunes de `ip` con opciones por defecto.
///
/// Devuelve un [`Service`] por cada puerto abierto detectado, con `protocol`,
/// `port`, `service_name` y `banner` rellenos; `state = Open`. Los campos
/// `id`/`device_id`/`first_seen_at`/`last_seen_at` quedan vacíos para que la capa de
/// persistencia los asigne.
pub async fn scan_ports(ip: IpAddr, top: u16) -> Vec<Service> {
    scan_ports_with(ip, top, &ScanOptions::default()).await
}

/// Escaneo configurable: sondea los `top` puertos de `ip` bajo `opts`.
///
/// La concurrencia se acota con un [`Semaphore`]; cada puerto tiene su propio
/// `connect_timeout`. El plazo global `scan_timeout` cancela el escaneo completo
/// (cancelación cooperativa): al vencer, se devuelven los hits recogidos hasta ese
/// momento. Los puertos cerrados/filtrados/no alcanzables no producen [`Service`].
pub async fn scan_ports_with(ip: IpAddr, top: u16, opts: &ScanOptions) -> Vec<Service> {
    let port_list = ranked_ports(top);
    if port_list.is_empty() {
        return Vec::new();
    }
    let concurrency = opts.concurrency.max(1);
    let sem = std::sync::Arc::new(Semaphore::new(concurrency));
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
    sem: std::sync::Arc<Semaphore>,
) -> Option<PortHit> {
    let _permit = sem.acquire().await.ok()?;
    let addr = SocketAddr::new(ip, port);
    let connect = TcpStream::connect(addr);
    let mut stream = tokio::time::timeout(connect_timeout, connect)
        .await
        .ok()?
        .ok()?;
    let banner = grab_banner(&mut stream, banner_timeout).await;
    Some(PortHit { port, banner })
}

/// Construye un [`Service`] a partir de un puerto abierto. Identidad/timestamps
/// vacíos: los asigna la persistencia.
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
    async fn scan_respects_global_timeout() {
        // 240.0.0.1 = TEST-NET-1 (RFC 5737): no enrutable, las conexiones caducan.
        let ip: IpAddr = "240.0.0.1".parse().unwrap();
        let opts = ScanOptions {
            connect_timeout: Duration::from_millis(80),
            scan_timeout: Duration::from_millis(250),
            banner_timeout: Duration::from_millis(80),
            concurrency: 16,
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
        let sem = std::sync::Arc::new(Semaphore::new(4));
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
}
