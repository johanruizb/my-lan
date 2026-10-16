//! Cliente de Speed Test con Cloudflare (`speed.cloudflare.com`) — Pilar 5 / Fase 9.
//!
//! Utiliza endpoints Anycast HTTP estándar de Cloudflare:
//! - `/cdn-cgi/trace`: Identificación de IP pública, país (`loc`) y datacenter (`colo`).
//! - `/__down?bytes=0`: Medición de latencia pura RTT y cálculo de Jitter RFC 3550.
//! - `/__down?bytes={N}`: Medición de descarga por chunks progresivos (1MB, 10MB, 25MB).
//! - `/__up`: Medición de subida vía POST con payload en memoria (1MB, 5MB, 10MB).
//!
//! Totalmente cooperativo con `CancellationToken` para cancelación instantánea.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::Client;
use tokio_util::sync::CancellationToken;

use mylan_core::{SpeedTestProgress, SpeedTestResult};

use crate::error::DiscoveryError;

const CF_BASE_URL: &str = "https://speed.cloudflare.com";

/// Metadatos devueltos por el endpoint `/cdn-cgi/trace` de Cloudflare.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CloudflareTrace {
    pub ip: String,
    pub colo: String,
    pub loc: String,
    pub city: Option<String>,
}

/// Parsea la respuesta en texto plano de `/cdn-cgi/trace`.
pub fn parse_cf_trace(body: &str) -> CloudflareTrace {
    let mut map = HashMap::new();
    for line in body.lines() {
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim(), v.trim());
        }
    }

    let ip = map.get("ip").unwrap_or(&"").to_string();
    let colo = map.get("colo").unwrap_or(&"").to_string();
    let loc = map.get("loc").unwrap_or(&"").to_string();
    let city = lookup_colo_city(&colo);

    CloudflareTrace {
        ip,
        colo,
        loc,
        city,
    }
}

/// Diccionario de aeropuertos / datacenters Anycast comunes de Cloudflare.
pub fn lookup_colo_city(colo: &str) -> Option<String> {
    let city = match colo.to_ascii_uppercase().as_str() {
        "BOG" => "Bogotá",
        "MDE" => "Medellín",
        "CLO" => "Cali",
        "BAQ" => "Barranquilla",
        "UIO" => "Quito",
        "GYE" => "Guayaquil",
        "LIM" => "Lima",
        "SCL" => "Santiago",
        "EZE" => "Buenos Aires",
        "MVD" => "Montevideo",
        "GRU" => "São Paulo",
        "GIG" => "Rio de Janeiro",
        "BSB" => "Brasília",
        "MEX" => "Ciudad de México",
        "QRO" => "Querétaro",
        "GDL" => "Guadalajara",
        "PTY" => "Ciudad de Panamá",
        "SJO" => "San José",
        "MIA" => "Miami",
        "ATL" => "Atlanta",
        "DFW" => "Dallas",
        "IAH" => "Houston",
        "ORD" => "Chicago",
        "JFK" | "EWR" => "New York",
        "IAD" => "Washington D.C.",
        "LAX" => "Los Angeles",
        "SFO" | "SJC" => "San Francisco / Silicon Valley",
        "SEA" => "Seattle",
        "MAD" => "Madrid",
        "BCN" => "Barcelona",
        "LIS" => "Lisboa",
        "LHR" => "Londres",
        "CDG" => "París",
        "FRA" => "Fráncfort",
        "AMS" => "Ámsterdam",
        _ => return None,
    };
    Some(city.to_string())
}

/// Ejecuta una prueba de velocidad completa contra `speed.cloudflare.com`.
///
/// Emite eventos continuos a través del callback `on_progress` y soporta cancelación cooperativa.
pub async fn run_cloudflare_speedtest<F>(
    network_id: Option<String>,
    cancel_token: CancellationToken,
    mut on_progress: F,
) -> Result<SpeedTestResult, DiscoveryError>
where
    F: FnMut(SpeedTestProgress) + Send + 'static,
{
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| DiscoveryError::Dns(format!("Error inicializando HTTP client: {e}")))?;

    // Fase 1: Identificación y Handshake
    on_progress(SpeedTestProgress {
        phase: "init".to_string(),
        progress_pct: 0.0,
        current_mbps: 0.0,
        bytes_transferred: 0,
        total_bytes: 0,
        latency_ms: None,
        jitter_ms: None,
        server_info: Some("Conectando con Cloudflare Anycast...".to_string()),
        error_message: None,
    });

    if cancel_token.is_cancelled() {
        return Err(DiscoveryError::Cancelled);
    }

    let trace_resp = tokio::select! {
        _ = cancel_token.cancelled() => return Err(DiscoveryError::Cancelled),
        res = client.get(format!("{CF_BASE_URL}/cdn-cgi/trace")).send() => {
            res.map_err(|e| DiscoveryError::Dns(format!("Fallo contactando trace de Cloudflare: {e}")))?
        }
    };

    let trace_text = trace_resp.text().await.unwrap_or_default();
    let trace = parse_cf_trace(&trace_text);
    let server_name = format!(
        "Cloudflare ({}{})",
        trace.colo,
        trace
            .city
            .as_deref()
            .map(|c| format!(" - {c}"))
            .unwrap_or_default()
    );

    // Fase 2: Medición de Latencia y Jitter RFC 3550
    on_progress(SpeedTestProgress {
        phase: "latency".to_string(),
        progress_pct: 5.0,
        current_mbps: 0.0,
        bytes_transferred: 0,
        total_bytes: 0,
        latency_ms: None,
        jitter_ms: None,
        server_info: Some(server_name.clone()),
        error_message: None,
    });

    let mut latencies = Vec::new();
    let mut jitter = 0.0f64;
    let mut prev_rtt: Option<f64> = None;
    const LATENCY_PROBES: usize = 6;

    for i in 0..LATENCY_PROBES {
        if cancel_token.is_cancelled() {
            return Err(DiscoveryError::Cancelled);
        }

        let start = Instant::now();
        let ping_req = client.get(format!("{CF_BASE_URL}/__down?bytes=0")).send();

        let ping_resp = tokio::select! {
            _ = cancel_token.cancelled() => return Err(DiscoveryError::Cancelled),
            res = ping_req => res,
        };

        if ping_resp.is_ok() {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            latencies.push(elapsed_ms);

            let rtt_f = elapsed_ms as f64;
            if let Some(prev) = prev_rtt {
                let d = (rtt_f - prev).abs();
                jitter += (d - jitter) / 16.0;
            }
            prev_rtt = Some(rtt_f);

            let avg_lat = latencies.iter().sum::<u64>() / (latencies.len() as u64);
            let pct = 5.0 + ((i + 1) as f32 / LATENCY_PROBES as f32) * 15.0; // 5% .. 20%

            on_progress(SpeedTestProgress {
                phase: "latency".to_string(),
                progress_pct: pct,
                current_mbps: 0.0,
                bytes_transferred: 0,
                total_bytes: 0,
                latency_ms: Some(avg_lat),
                jitter_ms: Some(jitter),
                server_info: Some(server_name.clone()),
                error_message: None,
            });
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let final_latency_ms = if latencies.is_empty() {
        25
    } else {
        latencies.iter().sum::<u64>() / (latencies.len() as u64)
    };

    // Fase 3: Descarga en chunks progresivos (1MB, 10MB, 25MB)
    on_progress(SpeedTestProgress {
        phase: "download".to_string(),
        progress_pct: 20.0,
        current_mbps: 0.0,
        bytes_transferred: 0,
        total_bytes: 36_000_000,
        latency_ms: Some(final_latency_ms),
        jitter_ms: Some(jitter),
        server_info: Some(server_name.clone()),
        error_message: None,
    });

    let download_chunks = [1_000_000u64, 10_000_000u64, 25_000_000u64];
    let total_dl_target: u64 = download_chunks.iter().sum();
    let mut total_dl_bytes = 0u64;
    let mut final_dl_mbps = 0.0f64;
    let dl_start = Instant::now();

    for chunk_size in download_chunks {
        if cancel_token.is_cancelled() {
            return Err(DiscoveryError::Cancelled);
        }

        let url = format!("{CF_BASE_URL}/__down?bytes={chunk_size}");
        let resp_res = tokio::select! {
            _ = cancel_token.cancelled() => return Err(DiscoveryError::Cancelled),
            res = client.get(&url).send() => res,
        };

        if let Ok(resp) = resp_res {
            let mut stream = resp.bytes_stream();
            while let Some(item) = stream.next().await {
                if cancel_token.is_cancelled() {
                    return Err(DiscoveryError::Cancelled);
                }
                if let Ok(bytes) = item {
                    total_dl_bytes += bytes.len() as u64;
                    let elapsed = dl_start.elapsed().as_secs_f64();
                    if elapsed > 0.05 {
                        final_dl_mbps = (total_dl_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);
                    }
                    let pct =
                        20.0 + ((total_dl_bytes as f32 / total_dl_target as f32) * 40.0).min(40.0); // 20% .. 60%
                    on_progress(SpeedTestProgress {
                        phase: "download".to_string(),
                        progress_pct: pct,
                        current_mbps: final_dl_mbps,
                        bytes_transferred: total_dl_bytes,
                        total_bytes: total_dl_target,
                        latency_ms: Some(final_latency_ms),
                        jitter_ms: Some(jitter),
                        server_info: Some(server_name.clone()),
                        error_message: None,
                    });
                }
            }
        }
    }

    // Fase 4: Subida en chunks progresivos (1MB, 5MB, 10MB)
    on_progress(SpeedTestProgress {
        phase: "upload".to_string(),
        progress_pct: 60.0,
        current_mbps: 0.0,
        bytes_transferred: 0,
        total_bytes: 16_000_000,
        latency_ms: Some(final_latency_ms),
        jitter_ms: Some(jitter),
        server_info: Some(server_name.clone()),
        error_message: None,
    });

    let upload_chunks = [1_000_000usize, 5_000_000usize, 10_000_000usize];
    let total_ul_target: u64 = upload_chunks.iter().map(|&c| c as u64).sum();
    let mut total_ul_bytes = 0u64;
    let mut final_ul_mbps = 0.0f64;
    let ul_start = Instant::now();

    for chunk_size in upload_chunks {
        if cancel_token.is_cancelled() {
            return Err(DiscoveryError::Cancelled);
        }

        // Payload de ceros generado en memoria
        let payload = vec![0u8; chunk_size];
        let post_req = client
            .post(format!("{CF_BASE_URL}/__up"))
            .body(payload)
            .send();

        let post_res = tokio::select! {
            _ = cancel_token.cancelled() => return Err(DiscoveryError::Cancelled),
            res = post_req => res,
        };

        if post_res.is_ok() {
            total_ul_bytes += chunk_size as u64;
            let elapsed = ul_start.elapsed().as_secs_f64();
            if elapsed > 0.05 {
                final_ul_mbps = (total_ul_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);
            }
            let pct = 60.0 + ((total_ul_bytes as f32 / total_ul_target as f32) * 38.0).min(38.0); // 60% .. 98%
            on_progress(SpeedTestProgress {
                phase: "upload".to_string(),
                progress_pct: pct,
                current_mbps: final_ul_mbps,
                bytes_transferred: total_ul_bytes,
                total_bytes: total_ul_target,
                latency_ms: Some(final_latency_ms),
                jitter_ms: Some(jitter),
                server_info: Some(server_name.clone()),
                error_message: None,
            });
        }
    }

    let now_str = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "2026-10-16T12:00:00Z".to_string());

    let final_result = SpeedTestResult {
        id: uuid::Uuid::new_v4().to_string(),
        network_id,
        server_name,
        server_host: "speed.cloudflare.com".to_string(),
        server_city: trace.city,
        server_country: if trace.loc.is_empty() {
            None
        } else {
            Some(trace.loc)
        },
        download_mbps: (final_dl_mbps * 100.0).round() / 100.0,
        upload_mbps: (final_ul_mbps * 100.0).round() / 100.0,
        latency_ms: final_latency_ms,
        jitter_ms: (jitter * 100.0).round() / 100.0,
        bufferbloat_ms: None,
        retransmits: None,
        protocol: "cloudflare".to_string(),
        created_at: now_str,
    };

    on_progress(SpeedTestProgress {
        phase: "finished".to_string(),
        progress_pct: 100.0,
        current_mbps: final_result.download_mbps,
        bytes_transferred: total_dl_bytes + total_ul_bytes,
        total_bytes: total_dl_target + total_ul_target,
        latency_ms: Some(final_result.latency_ms),
        jitter_ms: Some(final_result.jitter_ms),
        server_info: Some(final_result.server_name.clone()),
        error_message: None,
    });

    Ok(final_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cf_trace_extracts_fields() {
        let sample = "\
fl=123f45
h=speed.cloudflare.com
ip=181.135.24.10
ts=1729080000
visit_scheme=https
uag=Mozilla/5.0
colo=BOG
sliver=none
http=http/2
loc=CO
tls=TLSv1.3
";
        let trace = parse_cf_trace(sample);
        assert_eq!(trace.ip, "181.135.24.10");
        assert_eq!(trace.colo, "BOG");
        assert_eq!(trace.loc, "CO");
        assert_eq!(trace.city.as_deref(), Some("Bogotá"));
    }

    #[test]
    fn test_lookup_colo_city_matches_known_and_unknown() {
        assert_eq!(lookup_colo_city("MIA").as_deref(), Some("Miami"));
        assert_eq!(lookup_colo_city("MAD").as_deref(), Some("Madrid"));
        assert_eq!(lookup_colo_city("UNKNOWN_XYZ"), None);
    }
}
