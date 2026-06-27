//! `mylan-fingerprint` — identificación de dispositivos.
//!
//! Resuelve vendor por OUI (snapshot IEEE completo), hostname/reverse-DNS, interpreta
//! observaciones mDNS y aplica un motor de reglas YAML (`signatures/device-rules/`) para
//! inferir `device_type` + `confidence`. Implementa la interfaz de enrichment (función
//! concreta) definida en `mylan-core`, de forma aditiva sobre el pipeline.
//!
//! Estado: esqueleto (Paso 1). Implementación en Paso 6.
