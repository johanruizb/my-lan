//! Middleware de autenticación por bearer token (AC-5, ADR-7).
//!
//! [`TokenMiddleware`] guarda el token esperado y se aplica como `tower` layer
//! vía `axum::middleware::from_fn_with_state` en [`crate::serve`]. Valida el
//! header `Authorization: Bearer <token>` en cada petición a `/api/v1/*`;
//! `401 Unauthorized` si falta, no sigue el formato, o no coincide.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

/// Configuración del middleware de token: guarda el token esperado.
#[derive(Debug, Clone)]
pub struct TokenMiddleware {
    token: Arc<String>,
}

impl TokenMiddleware {
    /// Crea el middleware con el token esperado (32 bytes getrandom + base64,
    /// ver [`crate::token::generate_token`]).
    #[must_use]
    pub fn new(token: Arc<String>) -> Self {
        Self { token }
    }

    /// Compara el token provisto con el esperado. Comparación directa: el
    /// modelo de seguridad es localhost-only (ADR-7), sin amenaza de timing
    /// remoto.
    fn matches(&self, provided: &str) -> bool {
        provided == self.token.as_str()
    }

    /// Acceso al token esperado (para tests/diagnóstico).
    #[must_use]
    pub fn expected(&self) -> &str {
        self.token.as_str()
    }
}

/// Middleware (tower layer vía `from_fn_with_state`) que valida el header
/// `Authorization: Bearer <token>`. Devuelve `401 Unauthorized` si el header
/// falta, no sigue el formato `Bearer <token>`, o el token no coincide.
pub async fn require_token(
    State(mw): State<TokenMiddleware>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if is_authorized(&request, &mw) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// `true` si la petición lleva `Authorization: Bearer <token>` con el token
/// esperado. Toma `&Request` (no consume) para que el caller pueda mover la
/// request a `Next::run`.
fn is_authorized(req: &Request, mw: &TokenMiddleware) -> bool {
    let provided = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim);
    matches!(provided, Some(t) if mw.matches(t))
}
