//! Comandos IPC de la Suite de Herramientas de Red (Fase 7: Network Tools).
//!
//! Proporciona utilidades interactivas de diagnóstico bajo demanda con soporte
//! de streaming en tiempo real y cancelación cooperativa limpia:
//! - Ping continuo / ráfaga con métricas de jitter (RFC 3550).
//! - Traceroute interactivo salto por salto (Linux).
//! - DNS Lookup multidominio con selección de proveedor (Sistema, Cloudflare, Google, Quad9).
//! - Wake-on-LAN (Magic Packet).
//! - Port Checker rápido sin escaneo masivo.

use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, State};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;

use crate::state::DesktopState;

/// Evento emitido al descubrir un nuevo salto en un traceroute interactivo.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TraceHopEvent {
    pub session_id: String,
    pub hop: mylan_core::TraceHop,
}

/// Evento emitido al finalizar o cancelar un traceroute.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TraceDoneEvent {
    pub session_id: String,
    pub hops: Vec<mylan_core::TraceHop>,
}

/// Resultado de la comprobación rápida de un puerto específico.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PortCheckResult {
    pub target: String,
    pub port: u16,
    pub state: String, // "open" | "closed" | "filtered"
    pub latency_ms: Option<u64>,
    pub banner: Option<String>,
}

/// Resuelve una cadena a `IpAddr` (IP directa o resolución DNS si es hostname).
async fn resolve_target_ip(target: &str) -> Result<IpAddr, String> {
    if let Ok(ip) = target.trim().parse::<IpAddr>() {
        return Ok(ip);
    }
    let ips = mylan_discovery::resolve_host(target.trim(), false, false)
        .await
        .map_err(|e| format!("no se pudo resolver el host '{target}': {e}"))?;
    ips.into_iter()
        .next()
        .ok_or_else(|| format!("no se encontraron direcciones IP para '{target}'"))
}

/// Inicia una sesión de Ping en streaming con métricas de jitter y latencia en tiempo real.
#[tauri::command]
pub async fn start_tool_ping_cmd(
    app: AppHandle,
    state: State<'_, DesktopState>,
    target: String,
    session_id: String,
    interval_ms: Option<u64>,
    timeout_ms: Option<u64>,
) -> Result<(), String> {
    let ip = resolve_target_ip(&target).await?;
    let interval = Duration::from_millis(interval_ms.unwrap_or(1000).max(100));
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(1000).max(100));

    let cancel_token = CancellationToken::new();
    {
        let mut tokens = state.tool_tokens.lock().map_err(|e| e.to_string())?;
        tokens.insert(session_id.clone(), cancel_token.clone());
    }

    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let cancel_token_clone = cancel_token.clone();

    tokio::spawn(async move {
        let sid = session_id_clone.clone();
        let app_emit = app_clone.clone();

        let summary_res =
            mylan_discovery::ping_stream(ip, interval, timeout, cancel_token_clone, move |pkt| {
                let _ = app_emit.emit(
                    "tool:ping_packet",
                    serde_json::json!({
                        "session_id": sid,
                        "packet": pkt,
                    }),
                );
            })
            .await;

        if let Ok(summary) = summary_res {
            let _ = app_clone.emit(
                "tool:ping_summary",
                serde_json::json!({
                    "session_id": session_id_clone,
                    "summary": summary,
                }),
            );
        }

        // Limpieza de token
        {
            let state = app_clone.state::<DesktopState>();
            if let Ok(mut tokens) = state.tool_tokens.lock() {
                tokens.remove(&session_id_clone);
            };
        }
    });

    Ok(())
}

/// Inicia un traceroute interactivo emitiendo cada salto a medida que responde.
#[tauri::command]
pub async fn start_tool_traceroute_cmd(
    app: AppHandle,
    state: State<'_, DesktopState>,
    target: String,
    session_id: String,
    max_hops: Option<u8>,
    timeout_ms: Option<u64>,
) -> Result<(), String> {
    let ip = resolve_target_ip(&target).await?;
    let hops_limit = max_hops.unwrap_or(30).clamp(1, 64);
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(1000).max(100));

    let cancel_token = CancellationToken::new();
    {
        let mut tokens = state.tool_tokens.lock().map_err(|e| e.to_string())?;
        tokens.insert(session_id.clone(), cancel_token.clone());
    }

    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let cancel_token_clone = cancel_token.clone();

    tokio::spawn(async move {
        let sid = session_id_clone.clone();
        let app_emit = app_clone.clone();

        let hops_res = mylan_discovery::traceroute_stream(
            ip,
            hops_limit,
            timeout,
            cancel_token_clone,
            move |hop| {
                let _ = app_emit.emit(
                    "tool:trace_hop",
                    TraceHopEvent {
                        session_id: sid.clone(),
                        hop,
                    },
                );
            },
        )
        .await;

        let hops = hops_res.unwrap_or_default();
        let _ = app_clone.emit(
            "tool:trace_done",
            TraceDoneEvent {
                session_id: session_id_clone.clone(),
                hops,
            },
        );

        // Limpieza de token
        {
            let state = app_clone.state::<DesktopState>();
            if let Ok(mut tokens) = state.tool_tokens.lock() {
                tokens.remove(&session_id_clone);
            };
        }
    });

    Ok(())
}

/// Cancela cooperativamente una herramienta interactiva en curso (Ping o Traceroute).
#[tauri::command]
pub fn cancel_tool_cmd(state: State<'_, DesktopState>, session_id: String) -> Result<bool, String> {
    let mut tokens = state.tool_tokens.lock().map_err(|e| e.to_string())?;
    if let Some(token) = tokens.remove(&session_id) {
        token.cancel();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Consulta registros DNS de un dominio alternando entre el sistema o proveedores públicos.
#[tauri::command]
pub async fn dns_lookup_cmd(
    name: String,
    record_type: String,
    provider: Option<String>,
) -> Result<Vec<mylan_core::DnsRecord>, String> {
    mylan_discovery::dns_lookup_with_provider(&name, record_type, provider.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Envía un Magic Packet de Wake-on-LAN a una dirección MAC destino.
#[tauri::command]
pub async fn send_wol_cmd(mac: String, broadcast_ip: Option<String>) -> Result<(), String> {
    mylan_discovery::send_wol_packet(&mac, broadcast_ip.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Comprueba el estado de un puerto TCP específico sin realizar un escaneo masivo.
#[tauri::command]
pub async fn check_port_cmd(
    target: String,
    port: u16,
    timeout_ms: Option<u64>,
) -> Result<PortCheckResult, String> {
    let ip = resolve_target_ip(&target).await?;
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(1500).clamp(100, 10000));
    let addr = SocketAddr::new(ip, port);

    let start = Instant::now();
    match tokio::time::timeout(timeout, TcpStream::connect(addr)).await {
        Ok(Ok(stream)) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            // Lectura de banner opcional no bloqueante con timeout breve de 400ms
            let mut buf = [0u8; 256];
            let banner = tokio::time::timeout(Duration::from_millis(400), stream.readable())
                .await
                .ok()
                .and_then(|_| stream.try_read(&mut buf).ok())
                .and_then(|n| {
                    if n > 0 {
                        String::from_utf8(buf[..n].to_vec())
                            .ok()
                            .map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                });

            Ok(PortCheckResult {
                target,
                port,
                state: "open".to_string(),
                latency_ms: Some(latency_ms),
                banner,
            })
        }
        Ok(Err(e)) => {
            let state = if e.kind() == std::io::ErrorKind::ConnectionRefused {
                "closed".to_string()
            } else {
                "filtered".to_string()
            };
            Ok(PortCheckResult {
                target,
                port,
                state,
                latency_ms: None,
                banner: None,
            })
        }
        Err(_) => Ok(PortCheckResult {
            target,
            port,
            state: "filtered".to_string(), // Timeout
            latency_ms: None,
            banner: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_target_ip_loopback() {
        let ip = resolve_target_ip("127.0.0.1").await.unwrap();
        assert_eq!(ip, "127.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[tokio::test]
    async fn test_check_port_loopback_closed_port() {
        // Puerto alto poco probable que esté abierto en loopback
        let res = check_port_cmd("127.0.0.1".to_string(), 59999, Some(200))
            .await
            .unwrap();
        assert_eq!(res.port, 59999);
        // Debe ser closed o filtered, nunca open
        assert!(res.state == "closed" || res.state == "filtered");
    }
}
