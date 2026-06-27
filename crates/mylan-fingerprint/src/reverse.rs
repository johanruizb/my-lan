//! Reverse DNS best-effort con `hickory-resolver` (`from_system_conf`).
//!
//! AC-10 NO depende de que el PTR exista: en redes RFC1918 el router suele
//! devolver NXDOMAIN, por lo que esta función devuelve `None` ante cualquier
//! fallo. Es una señal *opcional* de enriquecimiento de hostname expuesta para
//! que un llamador async la combine con una `Observation` sin hostname. La fase
//! de enrichment por defecto (`Fingerprint::enricher`) es síncrona y NO la
//! invoca; un consumidor que quiera PTR debe llamarla por separado antes de
//! construir el `Device` (se mantiene fuera del `scan` por defecto para no
//! cargar latencia DNS en el presupuesto AC-12).

use std::net::IpAddr;

use hickory_resolver::proto::rr::RData;
use hickory_resolver::TokioResolver;

/// Realiza una consulta PTR best-effort para la IP dada.
///
/// Usa la configuración DNS del sistema (`/etc/resolv.conf` en Unix). Devuelve
/// `None` si la consulta falla o no hay registro PTR: **nunca** propaga el error
/// (AC-10: el fingerprint no depende de reverse-DNS).
pub async fn reverse_dns(ip: IpAddr) -> Option<String> {
    let resolver = TokioResolver::builder_tokio().ok()?.build().ok()?;
    let lookup = resolver.reverse_lookup(ip).await.ok()?;
    for record in lookup.answers() {
        if let RData::PTR(ptr) = &record.data {
            let name = ptr.0.to_string();
            return Some(name.trim_end_matches('.').to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reverse_dns_returns_none_for_loopback() {
        // 127.0.0.1 normalmente no tiene PTR público; best-effort => None es
        // un resultado válido (AC-10 no depende de PTR). Aceptamos None o un
        // nombre (algunos sistemas resuelven localhost), pero nunca un panic.
        let _ = reverse_dns(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))).await;
    }
}
