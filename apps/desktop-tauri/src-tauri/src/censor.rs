//! Helpers de enmascaramiento para el "modo censura" (AC-1/AC-2/AC-7).
//!
//! Enmascara identificadores estrictos (IP/MAC/hostname/display_name/cidr/dns)
//! en exports para que ningún valor real salga del proceso mientras el modo
//! está activo. El catálogo de campos sensibles y el formato de cada máscara
//! DEBEN mantenerse sincronizados con `src/lib/censor.ts` (Step 4 del plan
//! `censura-mode`): misma lista de campos, mismo formato, para que UI y
//! exports no diverjan ("what you see is what you share").
//!
//! Campos sensibles (catálogo): `primary_ip`, `primary_mac`, `hostname`,
//! `display_name`, `gateway_ip`, `gateway_mac`, `dns_servers`, `cidr`, `ip`,
//! `mac`, `device_ip`. No sensibles: `vendor`, `manufacturer`, `banner`,
//! `product`, `version`, `port`, `notes`.
//!
//! Nota técnica: `Device.primary_ip` (`Option<IpAddr>`) y `primary_mac`
//! (`Option<MacAddr>`) son campos tipados que no pueden alojar cadenas
//! enmascaradas con `*`, por lo que [`mask_device`] solo muta los campos
//! `String` (`hostname`, `display_name`); IP/MAC se enmascaran en el borde de
//! serialización (JSON vía [`mask_device_value`], CSV vía registros manuales en
//! `commands.rs`). Análogamente [`mask_service_row`] muta `display_name` y
//! `device_ip` se enmascara vía [`mask_service_value`].

use mylan_core::Device;
use mylan_db::service_repo::ServiceExportRow;

/// Máscara constante para MAC (nunca se revela — AC-5). Coincide con `maskMac()`
/// en `src/lib/censor.ts`.
pub fn mask_mac() -> &'static str {
    "••••"
}

/// Enmascara una IP: IPv4 zeroa los dos últimos octetos (`a.b.*.*`); IPv6
/// zeroa los 3 últimos grupos. Coincide con `maskIp()` en `src/lib/censor.ts`.
pub fn mask_ip(s: &str) -> String {
    if s.contains('.') && !s.contains(':') {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() == 4 {
            return format!("{}.{}.*.*", parts[0], parts[1]);
        }
    }
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() > 3 {
            let head = &parts[..parts.len() - 3];
            return format!("{}:*:*:*", head.join(":"));
        }
        return "*:*:*".to_string();
    }
    s.to_string()
}

/// Enmascara un hostname: si tiene puntos conserva la primera etiqueta y
/// enmascara el resto (`router.lan` → `router.*`); si es una etiqueta única,
/// enmascara todo (`*`). Coincide con `maskHostname()` en `src/lib/censor.ts`.
pub fn mask_hostname(s: &str) -> String {
    if let Some((first, _)) = s.split_once('.') {
        format!("{first}.*")
    } else {
        "*".to_string()
    }
}

/// Enmascara un CIDR: enmascara la parte de dirección con [`mask_ip`] y
/// conserva el prefijo (`192.168.1.0/24` → `192.168.*.*/24`). Coincide con
/// `maskCidr()` en `src/lib/censor.ts`.
///
/// Sin uso actual en exports de devices/services (esas filas no incluyen
/// `cidr`), pero se mantiene como parte del catálogo compartido con
/// `src/lib/censor.ts` para que UI y backend no diverjan.
#[allow(dead_code)]
pub fn mask_cidr(s: &str) -> String {
    if let Some((addr, prefix)) = s.split_once('/') {
        format!("{}/{}", mask_ip(addr), prefix)
    } else {
        mask_ip(s)
    }
}

/// Enmascara cada servidor DNS de un vector. Coincide con `maskDns()` en
/// `src/lib/censor.ts`.
///
/// Sin uso actual en exports de devices/services, pero se mantiene como parte
/// del catálogo compartido con `src/lib/censor.ts`.
#[allow(dead_code)]
pub fn mask_dns(vec: &[String]) -> Vec<String> {
    vec.iter().map(|s| mask_ip(s)).collect()
}

/// Enmascara los campos `String` de un `Device` (`hostname`, `display_name`)
/// in situ. IP/MAC se enmascaran en el borde de serialización (ver doc del
/// módulo). Guarda cada campo con `Some(..)` antes de mutar.
pub fn mask_device(d: &mut Device) {
    if let Some(h) = d.hostname.take() {
        d.hostname = Some(mask_hostname(&h));
    }
    if let Some(n) = d.display_name.take() {
        d.display_name = Some(mask_hostname(&n));
    }
}

/// Enmascara `display_name` de una fila de servicios in situ. `device_ip`
/// (`Option<IpAddr>`) se enmascara en el borde de serialización vía
/// [`mask_service_value`].
pub fn mask_service_row(r: &mut ServiceExportRow) {
    if let Some(n) = r.display_name.take() {
        r.display_name = Some(mask_hostname(&n));
    }
}

/// Enmascara los campos IP/MAC de un `serde_json::Value` que representa un
/// `Device` serializado. Sobreescribe `primary_ip` (si es string) con
/// [`mask_ip`] y `primary_mac` con [`mask_mac`]. `hostname`/`display_name` ya
/// vienen enmascarados por [`mask_device`] sobre el struct.
pub fn mask_device_value(v: &mut serde_json::Value) {
    if let Some(obj) = v.as_object_mut() {
        if let Some(ip) = obj.get("primary_ip").and_then(|i| i.as_str()) {
            obj["primary_ip"] = serde_json::Value::String(mask_ip(ip));
        }
        if obj.contains_key("primary_mac") {
            obj["primary_mac"] = serde_json::Value::String(mask_mac().to_string());
        }
    }
}

/// Enmascara `device_ip` de un `serde_json::Value` que representa un
/// `ServiceExportRow` serializado. `display_name` ya viene enmascarado por
/// [`mask_service_row`] sobre el struct.
pub fn mask_service_value(v: &mut serde_json::Value) {
    if let Some(obj) = v.as_object_mut() {
        if let Some(ip) = obj.get("device_ip").and_then(|i| i.as_str()) {
            obj["device_ip"] = serde_json::Value::String(mask_ip(ip));
        }
    }
}

/// Serializa un valor serde a su forma de cadena CSV (replica la salida que
/// produciría `csv::Writer::serialize` sobre el campo): `null` → vacío,
/// `bool` → `"true"`/`"false"`, número → su cadena, string → el valor.
pub fn csv_str<T: serde::Serialize>(v: &T) -> String {
    match serde_json::to_value(v) {
        Ok(serde_json::Value::Null) | Err(_) => String::new(),
        Ok(serde_json::Value::Bool(b)) => b.to_string(),
        Ok(serde_json::Value::Number(n)) => n.to_string(),
        Ok(serde_json::Value::String(s)) => s,
        Ok(other) => other.to_string(),
    }
}