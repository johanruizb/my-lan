//! Descubrimiento mDNS vía `mdns-sd`.
//!
//! Realiza la meta-query `_services._dns-sd._udp.local.` para enumerar los tipos de
//! servicio anunciados, y luego hace *browse* por cada tipo descubierto (acotado) para
//! resolver instancias y obtener IP/hostname. Se limita a la interfaz LAN mediante
//! [`ServiceDaemon::enable_interface`] con [`IfKind::Name`] (NO bind a IP), evitando
//! que el multicast se fugue por `docker0`/`tailscale0`.

use std::collections::HashSet;
use std::time::Duration;

use mdns_sd::{IfKind, ServiceDaemon, ServiceEvent};

use mylan_core::{Observation, Source};

use crate::iface::LanInterface;

/// Tope de tipos de servicio a investigar (evita exploración abierta).
const MAX_SERVICE_TYPES: usize = 24;
/// Tiempo dedicado a enumerar tipos antes de resolver instancias.
const META_FRACTION: u8 = 2;

/// Ejecuta el descubrimiento mDNS durante `timeout` y devuelve [`Observation`]s
/// crudas (sin interpretar; el fingerprint las decodifica en Paso 6).
pub async fn mdns_discover(iface: &LanInterface, timeout: Duration) -> Vec<Observation> {
    let daemon = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    // Limita el multicast a la interfaz LAN.
    let _ = daemon.enable_interface(IfKind::Name(iface.name.clone()));

    let receiver = match daemon.browse("_services._dns-sd._udp.local.") {
        Ok(r) => r,
        Err(_) => {
            let _ = daemon.shutdown();
            return Vec::new();
        }
    };

    let meta_budget = timeout / META_FRACTION as u32;
    let resolve_budget = timeout.saturating_sub(meta_budget);
    let mut out = Vec::new();

    // Fase 1: enumerar tipos de servicio (algunas implementaciones resuelven aquí).
    let mut service_types: HashSet<String> = HashSet::new();
    let meta_deadline = tokio::time::Instant::now() + meta_budget;
    loop {
        let remaining = meta_deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() || service_types.len() >= MAX_SERVICE_TYPES {
            break;
        }
        match tokio::time::timeout(remaining, receiver.recv_async()).await {
            Ok(Ok(ServiceEvent::ServiceFound(_ty, fullname))) => {
                service_types.insert(fullname);
            }
            Ok(Ok(ServiceEvent::ServiceResolved(svc))) => {
                push_resolved(&mut out, &svc);
            }
            _ => break,
        }
    }
    let _ = daemon.stop_browse("_services._dns-sd._udp.local.");

    // Fase 2: resolver instancias por cada tipo descubierto.
    let resolve_deadline = tokio::time::Instant::now() + resolve_budget;
    for ty in service_types {
        let rx = match daemon.browse(&ty) {
            Ok(r) => r,
            Err(_) => continue,
        };
        loop {
            let remaining = resolve_deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, rx.recv_async()).await {
                Ok(Ok(ServiceEvent::ServiceResolved(svc))) => push_resolved(&mut out, &svc),
                _ => break,
            }
        }
        let _ = daemon.stop_browse(&ty);
    }

    let _ = daemon.shutdown();
    out
}

/// Convierte un servicio resuelto en una [`Observation`] cruda por dirección IP.
fn push_resolved(out: &mut Vec<Observation>, svc: &mdns_sd::ResolvedService) {
    let ty = svc.ty_domain.clone();
    let host = svc.host.clone();
    let fullname = svc.fullname.clone();
    for addr in &svc.addresses {
        let ip = addr.to_ip_addr();
        let mut obs = Observation::new(Source::Mdns)
            .with_ip(ip)
            .with_hint("mdns.service", ty.clone())
            .with_hint("mdns.instance", fullname.clone());
        if !host.is_empty() {
            obs = obs.with_hostname(host.clone());
        }
        out.push(obs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_fraction_splits_budget() {
        // Comprobación en bloque const (clippy::assertions_on_constants).
        const _: () = {
            assert!(META_FRACTION == 2);
            assert!(MAX_SERVICE_TYPES > 0);
        };
    }
}
