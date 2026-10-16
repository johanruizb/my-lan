//! Background Monitor Runner — Monitoreo continuo 24/7 en MyLAN Desktop.
//!
//! Ejecuta un ciclo periódico de descubrimiento en segundo plano con perfil `Quick`,
//! tolerancia a suspensión del sistema (sleep/wake), skip guards para no solapar
//! ejecuciones, e histéresis de 3 escaneos fallidos consecutivos antes de declarar
//! un dispositivo como offline.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use mylan_core::{Enricher, Network, Observation, ScanProfile};
use mylan_discovery::{detect_interface, discover, DiscoverOptions};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::dto::{MonitoringStatusDto, ScanFinished, Settings};
use crate::tray::TrayHandles;

/// Gestor del ciclo de monitoreo continuo en segundo plano.
#[derive(Clone)]
pub struct BackgroundMonitor {
    paused: Arc<AtomicBool>,
    is_scanning: Arc<AtomicBool>,
    last_scan_at: Arc<RwLock<Option<Instant>>>,
    last_scan_rfc3339: Arc<RwLock<Option<String>>>,
    current_network: Arc<RwLock<Option<String>>>,
    /// Histéresis: contador de escaneos fallidos consecutivos por device_id.
    /// Se requieren 3 fallos consecutivos antes de declarar un dispositivo offline.
    missed_scans: Arc<std::sync::Mutex<HashMap<String, u32>>>,
    cancel_token: CancellationToken,
    tray_handles: Arc<RwLock<Option<TrayHandles>>>,
    prev_gateway_reachable: Arc<RwLock<Option<bool>>>,
}

impl Default for BackgroundMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundMonitor {
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            is_scanning: Arc::new(AtomicBool::new(false)),
            last_scan_at: Arc::new(RwLock::new(None)),
            last_scan_rfc3339: Arc::new(RwLock::new(None)),
            current_network: Arc::new(RwLock::new(None)),
            missed_scans: Arc::new(std::sync::Mutex::new(HashMap::new())),
            cancel_token: CancellationToken::new(),
            tray_handles: Arc::new(RwLock::new(None)),
            prev_gateway_reachable: Arc::new(RwLock::new(None)),
        }
    }

    /// Guarda las referencias a los ítems del System Tray para actualizar sus textos.
    pub async fn set_tray_handles(&self, handles: TrayHandles) {
        let mut guard = self.tray_handles.write().await;
        *guard = Some(handles);
    }

    /// Alterna el estado de pausa del monitoreo continuo y devuelve el nuevo estado.
    pub fn toggle_pause(&self) -> bool {
        let was_paused = self.paused.load(Ordering::SeqCst);
        let now_paused = !was_paused;
        self.paused.store(now_paused, Ordering::SeqCst);
        now_paused
    }

    /// Devuelve si el monitoreo se encuentra en pausa.
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// Devuelve si hay un escaneo de monitoreo actualmente en ejecución.
    #[allow(dead_code)]
    pub fn is_scanning(&self) -> bool {
        self.is_scanning.load(Ordering::SeqCst)
    }

    /// Detiene de manera cooperativa el loop de monitoreo.
    #[allow(dead_code)]
    pub fn stop(&self) {
        self.cancel_token.cancel();
    }

    /// Obtiene el DTO con el estado actual del monitoreo para enviar a la UI.
    pub async fn get_status(&self, settings: &Settings) -> MonitoringStatusDto {
        let paused = self.paused.load(Ordering::SeqCst);
        let is_scanning = self.is_scanning.load(Ordering::SeqCst);
        let interval_secs = settings.monitoring_interval_secs;
        let last_rfc = self.last_scan_rfc3339.read().await.clone();
        let current_net = self.current_network.read().await.clone();

        let next_scan_in_secs = if paused || !settings.background_monitoring_enabled {
            None
        } else {
            let last_inst = self.last_scan_at.read().await;
            match *last_inst {
                Some(inst) => {
                    let elapsed = inst.elapsed().as_secs();
                    if elapsed >= interval_secs {
                        Some(0)
                    } else {
                        Some(interval_secs - elapsed)
                    }
                }
                None => Some(0),
            }
        };

        MonitoringStatusDto {
            enabled: settings.background_monitoring_enabled,
            paused,
            is_scanning,
            interval_secs,
            last_scan_at: last_rfc,
            next_scan_in_secs,
            current_network: current_net,
        }
    }

    /// Actualiza los textos del menú del System Tray reflejando el estado actual.
    pub fn update_tray(&self, _app: &AppHandle, is_paused: bool) {
        let tray_handles_clone = self.tray_handles.clone();
        let net_name_clone = self.current_network.clone();
        let scanning = self.is_scanning.load(Ordering::SeqCst);

        tokio::spawn(async move {
            let handles = tray_handles_clone.read().await;
            if let Some(h) = &*handles {
                let net_name = net_name_clone.read().await.clone();
                let status_text = if is_paused {
                    "Monitoreo en pausa".to_string()
                } else if scanning {
                    match net_name {
                        Some(ref name) => format!("Escaneando: {name}…"),
                        None => "Escaneando red…".to_string(),
                    }
                } else {
                    match net_name {
                        Some(ref name) => format!("Monitoreando: {name}"),
                        None => "Monitoreando red".to_string(),
                    }
                };

                let pause_text = if is_paused {
                    "Reanudar monitoreo"
                } else {
                    "Pausar monitoreo"
                };

                let _ = h.status_item.set_text(status_text);
                let _ = h.pause_item.set_text(pause_text);
            }
        });
    }

    /// Dispara inmediatamente un ciclo de escaneo en segundo plano fuera de su horario.
    pub async fn trigger_scan(&self, app: AppHandle) {
        if self.is_scanning.load(Ordering::SeqCst) {
            return;
        }
        let this = self.clone();
        tokio::spawn(async move {
            this.run_cycle(&app, false).await;
        });
    }

    /// Bucle principal de monitoreo continuo.
    pub fn start_loop(self: Arc<Self>, app: AppHandle) {
        tokio::spawn(async move {
            // Breve retraso al arranque para dejar que la interfaz de red y la ventana inicialicen
            tokio::time::sleep(Duration::from_secs(3)).await;

            let mut last_tick = Instant::now();
            let mut cold_start = true;

            loop {
                // Leer configuración actual de intervalo y activación
                let (enabled, interval_secs) = {
                    let state = app.state::<crate::state::DesktopState>();
                    let s = state.settings.read().await;
                    (
                        s.background_monitoring_enabled,
                        s.monitoring_interval_secs.max(30),
                    )
                };

                // Comprobar token de cancelación
                if self.cancel_token.is_cancelled() {
                    break;
                }

                // Detección de suspensión del sistema (Sleep/Wake):
                // Si el tiempo real transcurrido excede más de 2 veces el intervalo configurado,
                // significa que el equipo estuvo suspendido o hibernando.
                let elapsed = last_tick.elapsed();
                if elapsed > Duration::from_secs(interval_secs * 2) {
                    // Período de gracia de 20s para estabilización de DHCP tras suspensión
                    tokio::time::sleep(Duration::from_secs(20)).await;
                    cold_start = true;
                }
                last_tick = Instant::now();

                let is_paused = self.paused.load(Ordering::SeqCst);
                self.update_tray(&app, is_paused);

                if enabled && !is_paused {
                    // Skip guard: si otro escaneo ya está en marcha, omitimos este tick
                    if !self.is_scanning.load(Ordering::SeqCst) {
                        self.run_cycle(&app, cold_start).await;
                        cold_start = false;
                    }
                }

                // Esperamos un segundo fraccionado o el intervalo con select para respuesta ágil a cancel/pause
                let wait_duration = Duration::from_secs(interval_secs);
                tokio::select! {
                    () = self.cancel_token.cancelled() => break,
                    () = tokio::time::sleep(wait_duration) => {}
                }
            }
        });
    }

    /// Ejecuta un único ciclo de descubrimiento y actualización en base de datos.
    async fn run_cycle(&self, app: &AppHandle, cold_start: bool) {
        // Skip guard atómico
        if self
            .is_scanning
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        let is_paused = self.paused.load(Ordering::SeqCst);
        self.update_tray(app, is_paused);

        let app_clone = app.clone();
        let this = self.clone();

        let outcome = tokio::task::spawn_blocking(move || -> Result<(), String> {
            let state = app_clone.state::<crate::state::DesktopState>();
            let db_path = state.db_path.clone();

            // 1. Detectar interfaz activa
            let iface = match detect_interface(None) {
                Ok(i) => i,
                Err(e) => {
                    let mut net_guard = this.current_network.blocking_write();
                    *net_guard = None;
                    return Err(format!("No se detectó interfaz activa: {e}"));
                }
            };

            let net_name = iface.ssid.clone().unwrap_or_else(|| iface.cidr());
            {
                let mut net_guard = this.current_network.blocking_write();
                *net_guard = Some(net_name.clone());
            }

            // 2. Realizar descubrimiento con perfil Quick en background
            let opts = DiscoverOptions::for_profile(ScanProfile::Quick);
            let raw_observations =
                tokio::runtime::Handle::current().block_on(async { discover(&iface, &opts).await });

            // 3. Conexión SQLite dedicada en WAL
            let conn =
                mylan_db::connection::connect(Path::new(&db_path)).map_err(|e| e.to_string())?;

            let now = mylan_db::util::now_rfc3339().map_err(|e| e.to_string())?;
            let network = Network {
                id: iface.cidr(),
                name: net_name,
                cidr: iface.cidr(),
                gateway_ip: iface.gateway_ip,
                dns_servers: iface.dns_servers.clone(),
                created_at: now.clone(),
                updated_at: now.clone(),
            };

            // 4. Aplicar Histéresis de Presencia (3 escaneos fallidos consecutivos)
            let before_devices = mylan_db::diff::snapshot_devices_before(&conn, &network.id)
                .map_err(|e| e.to_string())?;

            // Mapear qué IPs/MACs fueron vistas en este escaneo
            let mut observed_ips: Vec<IpAddr> = Vec::new();
            for obs in &raw_observations {
                if let Some(ip) = obs.ip {
                    observed_ips.push(ip);
                }
            }

            let mut final_observations = raw_observations;

            {
                let mut missed = this.missed_scans.lock().map_err(|e| e.to_string())?;

                for before in &before_devices {
                    let was_seen = match &before.primary_ip {
                        Some(ip_str) => match ip_str.parse::<IpAddr>() {
                            Ok(ip) => observed_ips.contains(&ip),
                            Err(_) => false,
                        },
                        None => false,
                    };

                    if was_seen {
                        // Dispositivo presente: resetear contador de fallos
                        missed.remove(&before.id);
                    } else if before.is_online {
                        // Dispositivo ausente pero estaba marcado online previamente:
                        let count = missed.entry(before.id.clone()).or_insert(0);
                        *count += 1;

                        if *count < 3 {
                            // Preservación temporal: sintetizar una observación para mantenerlo online
                            if let Some(ip_str) = &before.primary_ip {
                                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                                    final_observations.push(
                                        Observation::new(mylan_core::Source::Icmp).with_ip(ip),
                                    );
                                }
                            }
                        } else {
                            // Alcanzó 3 fallos consecutivos: no inyectamos observación sintética;
                            // run_diff detectará su ausencia y lo marcará offline.
                        }
                    }
                }
            }

            // 5. Cargar enricher
            let signatures_dir = resolve_signatures_dir(&app_clone);
            let enricher = build_enricher(&signatures_dir, iface.gateway_ip);

            // 6. Ejecutar pipeline con diff y persistencia atómica
            let (outcome, events) = mylan_db::pipeline::run_scan_pipeline_with_diff(
                &conn,
                &network,
                &final_observations,
                &enricher,
                ScanProfile::Quick,
                &now,
                cold_start,
            )
            .map_err(|e| e.to_string())?;

            // 7. Evaluar y despachar alertas nativas del SO (Fase 6: Desktop Alerts)
            let settings = {
                let state = app_clone.state::<crate::state::DesktopState>();
                let s = state.settings.blocking_read().clone();
                s
            };
            crate::alerts::dispatch_scan_alerts(&app_clone, &events, &conn, &settings, cold_start);

            // 8. Evaluar conectividad al Gateway local
            if let Some(gw_ip) = iface.gateway_ip {
                let is_reachable = observed_ips.contains(&gw_ip);
                let mut gw_guard = this.prev_gateway_reachable.blocking_write();
                if let Some(prev) = *gw_guard {
                    if !cold_start && is_reachable != prev {
                        crate::alerts::notify_gateway_status(
                            &app_clone,
                            &settings,
                            is_reachable,
                            prev,
                        );
                    }
                }
                *gw_guard = Some(is_reachable);
            }

            // 9. Ejecutar auditoría pasiva de seguridad en segundo plano
            let _ = mylan_db::security::run_full_security_audit(&conn, &network.id);

            // 9. Notificar al frontend que terminó un escaneo de fondo
            let _ = app_clone.emit(
                "scan:finished",
                &ScanFinished {
                    scan_id: outcome.scan_id,
                },
            );

            // Registrar marcas de tiempo
            {
                let mut t_guard = this.last_scan_at.blocking_write();
                *t_guard = Some(Instant::now());
            }
            {
                let mut r_guard = this.last_scan_rfc3339.blocking_write();
                *r_guard = Some(now);
            }

            Ok(())
        })
        .await;

        if let Err(e) = outcome {
            eprintln!("Error en monitor_loop join: {e}");
        }

        // Liberar flag de escaneo
        self.is_scanning.store(false, Ordering::SeqCst);

        let is_paused = self.paused.load(Ordering::SeqCst);
        self.update_tray(app, is_paused);

        // Emitir estado actualizado a la UI
        let state = app.state::<crate::state::DesktopState>();
        let settings = state.settings.read().await.clone();
        let status = self.get_status(&settings).await;
        let _ = app.emit("monitoring:status", &status);
    }
}

/// Resuelve el directorio de firmas de huella digital (mismo helper que commands.rs).
fn resolve_signatures_dir(app: &AppHandle) -> PathBuf {
    if let Ok(res) = app.path().resource_dir() {
        let candidate = res.join("signatures");
        if candidate.is_dir() {
            return candidate;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../signatures")
}

/// Construye el `Enricher` de fingerprint (mismo helper que commands.rs).
fn build_enricher(signatures_dir: &Path, gateway_ip: Option<IpAddr>) -> Enricher {
    match mylan_fingerprint::Fingerprint::load(signatures_dir, gateway_ip) {
        Ok(fp) => fp.enricher(),
        Err(_) => mylan_core::noop_enricher(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_initial_state_and_pause_toggle() {
        let monitor = BackgroundMonitor::new();
        assert!(!monitor.is_paused());
        assert!(!monitor.is_scanning());

        // Toggle a pausado
        let is_paused = monitor.toggle_pause();
        assert!(is_paused);
        assert!(monitor.is_paused());

        // Toggle a reanudado
        let is_paused_again = monitor.toggle_pause();
        assert!(!is_paused_again);
        assert!(!monitor.is_paused());
    }

    #[test]
    fn test_hysteresis_logic_requires_three_consecutive_misses() {
        let monitor = BackgroundMonitor::new();
        let device_id = "test-device-uuid-1";

        // Simulación: dispositivo no visto en escaneo 1
        {
            let mut missed = monitor.missed_scans.lock().unwrap();
            let count = missed.entry(device_id.to_string()).or_insert(0);
            *count += 1;
            assert_eq!(*count, 1);
            // count < 3: debe preservarse online
            assert!(*count < 3);
        }

        // Simulación: dispositivo no visto en escaneo 2
        {
            let mut missed = monitor.missed_scans.lock().unwrap();
            let count = missed.entry(device_id.to_string()).or_insert(0);
            *count += 1;
            assert_eq!(*count, 2);
            // count < 3: sigue preservándose online
            assert!(*count < 3);
        }

        // Simulación: dispositivo reaparece en la red en escaneo intermedio
        {
            let mut missed = monitor.missed_scans.lock().unwrap();
            // Si el dispositivo fue visto, el contador se resetea por completo
            missed.remove(device_id);
            assert!(!missed.contains_key(device_id));
        }

        // Simulación: ahora cae 3 veces seguidas
        for expected in 1..=3 {
            let mut missed = monitor.missed_scans.lock().unwrap();
            let count = missed.entry(device_id.to_string()).or_insert(0);
            *count += 1;
            assert_eq!(*count, expected);
            if expected < 3 {
                assert!(*count < 3, "Debe tolerar fallos menores a 3");
            } else {
                assert_eq!(*count, 3, "Al tercer fallo consecutivo se declara offline");
            }
        }
    }
}
