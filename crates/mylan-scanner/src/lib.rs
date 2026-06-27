//! `mylan-scanner` — escaneo de puertos y detección de servicios.
//!
//! Port scan TCP-connect (perfil `quick`: top 32/100) concurrente con rate limiting,
//! timeouts y cancelación; banner grabbing básico y mapeo de servicios. Opera solo sobre
//! hosts vivos (depende de la fase liveness de `mylan-discovery`).
//!
//! Estado: esqueleto (Paso 1). Implementación en Paso 7.
