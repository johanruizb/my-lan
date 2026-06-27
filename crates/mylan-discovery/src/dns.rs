//! Resolución DNS de diagnóstico (AC-8) + helpers compartidos.
//!
//! Este módulo centraliza la construcción del resolver del sistema
//! ([`system_resolver`]) para que lo reutilicen `dns_lookup_host` y
//! `traceroute.rs` (reverse DNS por salto). Soporta A/AAAA/PTR/MX/TXT.
//!
//! Sin root, sin binarios externos: `hickory-resolver` con `from_system_conf`
//! (lee `/etc/resolv.conf` en Unix). `mylan-fingerprint::reverse::reverse_dns`
//! mantiene su propia construcción (distinto crate, sin ciclo).

use std::net::IpAddr;

use hickory_resolver::lookup::Lookup;
use hickory_resolver::proto::rr::{RData, RecordType};
use hickory_resolver::TokioResolver;

use mylan_core::DnsRecord;

use crate::error::DiscoveryError;

/// Construye un [`TokioResolver`] a partir de la configuración DNS del sistema.
///
/// Helper compartido por `dns_lookup_host` y `traceroute.rs` (reverse DNS por
/// salto). Devuelve un error estructurado si la configuración no se puede leer.
pub fn system_resolver() -> Result<TokioResolver, DiscoveryError> {
    TokioResolver::builder_tokio()
        .map_err(|e| DiscoveryError::Dns(format!("configuración del resolver: {e}")))?
        .build()
        .map_err(|e| DiscoveryError::Dns(format!("construcción del resolver: {e}")))
}

/// Reverse DNS best-effort para una IP (usado por `traceroute.rs` por salto).
///
/// Devuelve `None` ante cualquier fallo (NXDOMAIN típico en redes RFC1918):
/// el reverse DNS es una señal opcional, nunca bloquea el diagnóstico.
pub async fn reverse_lookup(resolver: &TokioResolver, ip: IpAddr) -> Option<String> {
    let lookup = resolver.reverse_lookup(ip).await.ok()?;
    for record in lookup.answers() {
        if let RData::PTR(ptr) = &record.data {
            return Some(ptr.0.to_string().trim_end_matches('.').to_string());
        }
    }
    None
}

/// Resuelve un hostname a direcciones IP (forward lookup, A/AAAA).
///
/// Usado por los comandos `ping`/`traceroute` cuando el target es un hostname.
/// `ipv4_only`/`ipv6_only` filtran la familia (ambos false = todas).
pub async fn resolve_host(
    name: &str,
    ipv4_only: bool,
    ipv6_only: bool,
) -> Result<Vec<IpAddr>, DiscoveryError> {
    let resolver = system_resolver()?;
    let lookup = resolver
        .lookup_ip(name)
        .await
        .map_err(|e| DiscoveryError::Dns(e.to_string()))?;
    let ips: Vec<IpAddr> = lookup
        .iter()
        .filter(|ip| {
            let is_v4 = matches!(ip, IpAddr::V4(_));
            let is_v6 = matches!(ip, IpAddr::V6(_));
            if ipv4_only && !ipv6_only {
                is_v4
            } else if ipv6_only && !ipv4_only {
                is_v6
            } else {
                true
            }
        })
        .collect();
    Ok(ips)
}

/// Resuelve registros DNS para un nombre (AC-8).
///
/// `record_type` (case-insensitive) acepta: `A`, `AAAA`, `A+AAAA`/vacío
/// (default dual-stack), `PTR`, `MX`, `TXT`. Para `PTR` con un nombre que es
/// una IP se usa `reverse_lookup` (API idiomática); si es un hostname, se hace
/// una consulta PTR directa (raro pero válido).
pub async fn dns_lookup_host(
    name: &str,
    record_type: String,
) -> Result<Vec<DnsRecord>, DiscoveryError> {
    let resolver = system_resolver()?;
    let rt = record_type.trim().to_ascii_uppercase();
    match rt.as_str() {
        "" | "A+AAAA" | "ANY" => {
            // Default dual-stack: A y AAAA en una sola llamada lógica.
            let mut out = Vec::new();
            for (label, rtype) in [("A", RecordType::A), ("AAAA", RecordType::AAAA)] {
                if let Ok(lookup) = resolver.lookup(name, rtype).await {
                    records_into(name, label, &lookup, &mut out);
                }
            }
            Ok(out)
        }
        "A" => lookup_one(&resolver, name, "A", RecordType::A).await,
        "AAAA" => lookup_one(&resolver, name, "AAAA", RecordType::AAAA).await,
        "MX" => lookup_one(&resolver, name, "MX", RecordType::MX).await,
        "TXT" => lookup_one(&resolver, name, "TXT", RecordType::TXT).await,
        "PTR" => ptr_lookup(&resolver, name).await,
        other => Err(DiscoveryError::Dns(format!(
            "tipo de registro no soportado: '{other}' (usar A|AAAA|PTR|MX|TXT)"
        ))),
    }
}

/// Consulta un único `RecordType` y mapea las respuestas a `DnsRecord`.
async fn lookup_one(
    resolver: &TokioResolver,
    name: &str,
    label: &str,
    rtype: RecordType,
) -> Result<Vec<DnsRecord>, DiscoveryError> {
    match resolver.lookup(name, rtype).await {
        Ok(lookup) => {
            let mut out = Vec::new();
            records_into(name, label, &lookup, &mut out);
            Ok(out)
        }
        Err(e) => Err(DiscoveryError::Dns(e.to_string())),
    }
}

/// Consulta PTR: IP → reverse_lookup; hostname → consulta PTR directa.
async fn ptr_lookup(
    resolver: &TokioResolver,
    name: &str,
) -> Result<Vec<DnsRecord>, DiscoveryError> {
    if let Ok(ip) = name.parse::<IpAddr>() {
        // reverse_lookup del resolver entrega los PTR con su TTL real.
        return match resolver.reverse_lookup(ip).await {
            Ok(lookup) => {
                let mut out = Vec::new();
                records_into(name, "PTR", &lookup, &mut out);
                Ok(out)
            }
            // NXDOMAIN es un resultado válido para PTR (redes RFC1918).
            Err(_) => Ok(Vec::new()),
        };
    }
    match resolver.lookup(name, RecordType::PTR).await {
        Ok(lookup) => {
            let mut out = Vec::new();
            records_into(name, "PTR", &lookup, &mut out);
            Ok(out)
        }
        Err(e) => Err(DiscoveryError::Dns(e.to_string())),
    }
}

/// Convierte un `&RData` en su valor textual (None si el tipo no es de los que
/// reportamos). Centraliza el formateo de A/AAAA/MX/TXT/PTR.
fn rdata_value(data: &RData) -> Option<String> {
    match data {
        RData::A(a) => Some(a.0.to_string()),
        RData::AAAA(a) => Some(a.0.to_string()),
        RData::MX(mx) => Some(format!("{} {}", mx.preference, mx.exchange)),
        RData::TXT(txt) => Some(
            txt.txt_data
                .iter()
                .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
                .collect::<Vec<_>>()
                .join(" "),
        ),
        RData::PTR(ptr) => Some(ptr.0.to_string().trim_end_matches('.').to_string()),
        _ => None,
    }
}

/// Vuelca las respuestas de un [`Lookup`] en `out` como `DnsRecord`.
fn records_into(name: &str, label: &str, lookup: &Lookup, out: &mut Vec<DnsRecord>) {
    for record in lookup.answers() {
        if let Some(value) = rdata_value(&record.data) {
            out.push(DnsRecord {
                name: name.to_string(),
                record_type: label.to_string(),
                value,
                ttl: record.ttl,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `dns localhost` debe resolver (A/AAAA de loopback). Red real → no `#[ignore]`
    /// porque localhost es determinista. Aceptamos 0 o más registros: lo crítico es
    /// que no panic y que devuelva Ok.
    #[tokio::test]
    async fn dns_lookup_localhost_returns_ok() {
        let result = dns_lookup_host("localhost", String::new()).await;
        assert!(result.is_ok(), "dns_lookup_host(localhost) debe ser Ok");
        let records = result.expect("ok");
        // localhost resuelve a 127.0.0.1 y/o ::1 en prácticamente todo Linux.
        assert!(
            records
                .iter()
                .any(|r| r.value == "127.0.0.1" || r.value == "::1"),
            "esperaba al menos un registro de loopback, obtuvo: {records:?}"
        );
        assert!(records
            .iter()
            .all(|r| r.record_type == "A" || r.record_type == "AAAA"));
    }

    /// PTR de 127.0.0.1: algunos sistemas resuelven `localhost`, otros NXDOMAIN.
    /// Aceptamos cualquiera (best-effort), pero nunca un panic ni un Err.
    #[tokio::test]
    async fn dns_ptr_loopback_is_ok() {
        let result = dns_lookup_host("127.0.0.1", "PTR".to_string()).await;
        assert!(result.is_ok(), "PTR lookup de 127.0.0.1 debe ser Ok");
    }

    /// Tipo no soportado → Err con mensaje claro (AC-8: errores claros).
    #[tokio::test]
    async fn dns_unsupported_type_is_error() {
        let result = dns_lookup_host("localhost", "SRV".to_string()).await;
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("no soportado"), "mensaje: {msg}");
    }

    /// `resolve_host` de localhost entrega al menos 127.0.0.1 o ::1.
    #[tokio::test]
    async fn resolve_host_localhost_works() {
        let ips = resolve_host("localhost", false, false)
            .await
            .expect("resolve");
        assert!(
            ips.iter()
                .any(|ip| matches!(ip, IpAddr::V4(v) if v.is_loopback())
                    || matches!(ip, IpAddr::V6(v) if v.is_loopback())),
            "esperaba una IP de loopback, obtuvo: {ips:?}"
        );
    }

    /// `system_resolver` debe construirse sin error en un host Unix normal.
    #[test]
    fn system_resolver_builds() {
        assert!(system_resolver().is_ok());
    }
}
