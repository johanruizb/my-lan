//! Autenticacion OAuth 2.0 PKCE sin servidor (RFC 8252 / RFC 7636).
//!
//! Permite vincular cuentas de Google Drive de manera segura sin backend intermedio:
//! - Servidor loopback efimero en `127.0.0.1:0`
//! - Verificador PKCE aleatorio con reto SHA-256 (S256)
//! - Intercambio y refresco de tokens contra los endpoints oficiales de Google OAuth2

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
pub const GDRIVE_APPDATA_SCOPE: &str = "https://www.googleapis.com/auth/drive.appdata openid email";

#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("Error de I/O en loopback: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error de peticion HTTP: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Error de respuesta OAuth de Google: {0}")]
    GoogleAuthError(String),

    #[error("Fallo de validacion CSRF de state")]
    StateMismatch,

    #[error("Peticion de callback invalida o ausente de codigo")]
    MissingCode,

    #[error("Tiempo de espera agotado esperando autorizacion")]
    Timeout,
}

/// Sesion activa de autorizacion PKCE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkceSession {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
}

impl PkceSession {
    /// Genera una nueva sesion con verifier de 32 bytes y state de 16 bytes base64url.
    pub fn new() -> Result<Self, OAuthError> {
        let mut verifier_bytes = [0u8; 32];
        let mut state_bytes = [0u8; 16];

        getrandom::getrandom(&mut verifier_bytes)
            .map_err(|e| OAuthError::GoogleAuthError(format!("Error getrandom verifier: {e}")))?;
        getrandom::getrandom(&mut state_bytes)
            .map_err(|e| OAuthError::GoogleAuthError(format!("Error getrandom state: {e}")))?;

        let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
        let state = URL_SAFE_NO_PAD.encode(state_bytes);

        // SHA-256 del verifier
        let digest = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(digest);

        Ok(Self {
            code_verifier,
            code_challenge,
            state,
        })
    }
}

/// Construye la URL de autorizacion de Google OAuth 2.0 PKCE.
pub fn build_authorization_url(
    client_id: &str,
    redirect_uri: &str,
    session: &PkceSession,
) -> String {
    let scope_encoded = urlencoding_simple(GDRIVE_APPDATA_SCOPE);
    format!(
        "{GOOGLE_AUTH_URL}?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope={scope_encoded}&code_challenge={}&code_challenge_method=S256&state={}&access_type=offline&prompt=consent",
        session.code_challenge, session.state
    )
}

fn urlencoding_simple(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            b':' => out.push_str("%3A"),
            b'/' => out.push_str("%2F"),
            _ => {
                use std::fmt::Write;
                let _ = write!(&mut out, "%{b:02X}");
            }
        }
    }
    out
}

/// Servidor loopback local que escucha en `127.0.0.1:0` esperando la redireccion de Google.
pub async fn listen_for_oauth_callback(
    listener: TcpListener,
    expected_state: &str,
    timeout_secs: u64,
) -> Result<String, OAuthError> {
    tokio::select! {
        res = accept_and_parse_callback(listener, expected_state) => res,
        _ = tokio::time::sleep(Duration::from_secs(timeout_secs)) => {
            Err(OAuthError::Timeout)
        }
    }
}

async fn accept_and_parse_callback(
    listener: TcpListener,
    expected_state: &str,
) -> Result<String, OAuthError> {
    let (mut socket, _) = listener.accept().await?;
    let mut buf = [0u8; 4096];
    let bytes_read = socket.read(&mut buf).await?;
    let request_str = String::from_utf8_lossy(&buf[..bytes_read]);

    let (code, state) = parse_oauth_query(&request_str)?;

    if state != expected_state {
        let error_html = "<!DOCTYPE html><html><body style='font-family:sans-serif;text-align:center;padding:40px;'><h2 style='color:#ef4444;'>Error de Seguridad CSRF</h2><p>El parametro state no coincide. Cierra esta ventana.</p></body></html>";
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{error_html}",
            error_html.len()
        );
        let _ = socket.write_all(response.as_bytes()).await;
        return Err(OAuthError::StateMismatch);
    }

    let success_html = "<!DOCTYPE html><html><body style='font-family:system-ui,-apple-system,sans-serif;text-align:center;padding:50px;background:#0f172a;color:#f8fafc;'><div style='max-width:480px;margin:auto;border:1px solid #334155;border-radius:12px;padding:32px;background:#1e293b;'><h2 style='color:#38bdf8;margin-top:0;'>Autenticación Exitosa</h2><p>Tu cuenta de Google Drive ha sido vinculada correctamente con <strong>MyLAN</strong>.</p><p style='color:#94a3b8;font-size:0.9em;'>Ya puedes cerrar esta pestaña y volver a la aplicación.</p></div><script>setTimeout(function(){window.close();}, 1500);</script></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{success_html}",
        success_html.len()
    );
    let _ = socket.write_all(response.as_bytes()).await;

    Ok(code)
}

fn parse_oauth_query(request: &str) -> Result<(String, String), OAuthError> {
    let first_line = request.lines().next().unwrap_or_default();
    let path = first_line.split_whitespace().nth(1).unwrap_or_default();

    let query_str = path.split('?').nth(1).unwrap_or_default();
    let mut code = None;
    let mut state = None;

    for param in query_str.split('&') {
        let mut parts = param.split('=');
        let key = parts.next().unwrap_or_default();
        let val = parts.next().unwrap_or_default();
        if key == "code" {
            code = Some(val.to_string());
        } else if key == "state" {
            state = Some(val.to_string());
        }
    }

    match (code, state) {
        (Some(c), Some(s)) => Ok((c, s)),
        _ => Err(OAuthError::MissingCode),
    }
}

/// Tokens de acceso y refresco obtenidos de Google OAuth 2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

/// Intercambia el codigo de autorizacion temporal por tokens permanentes en Google OAuth2.
pub async fn exchange_code_for_tokens(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: Option<&str>,
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<OAuthTokens, OAuthError> {
    let mut params = vec![
        ("client_id", client_id),
        ("code", code),
        ("code_verifier", code_verifier),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];

    let secret_holder;
    if let Some(sec) = client_secret {
        if !sec.is_empty() {
            secret_holder = sec;
            params.push(("client_secret", secret_holder));
        }
    }

    let resp = client.post(GOOGLE_TOKEN_URL).form(&params).send().await?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(OAuthError::GoogleAuthError(format!(
            "Fallo intercambio de token: {err_text}"
        )));
    }

    let bytes = resp.bytes().await?;
    let tokens: OAuthTokens = serde_json::from_slice(&bytes)
        .map_err(|e| OAuthError::GoogleAuthError(format!("Error parseando JSON de tokens: {e}")))?;
    Ok(tokens)
}

/// Renueva el token de acceso utilizando el `refresh_token` almacenado.
pub async fn refresh_access_token(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
) -> Result<OAuthTokens, OAuthError> {
    let mut params = vec![
        ("client_id", client_id),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let secret_holder;
    if let Some(sec) = client_secret {
        if !sec.is_empty() {
            secret_holder = sec;
            params.push(("client_secret", secret_holder));
        }
    }

    let resp = client.post(GOOGLE_TOKEN_URL).form(&params).send().await?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(OAuthError::GoogleAuthError(format!(
            "Fallo refresco de token: {err_text}"
        )));
    }

    let bytes = resp.bytes().await?;
    let tokens: OAuthTokens = serde_json::from_slice(&bytes)
        .map_err(|e| OAuthError::GoogleAuthError(format!("Error parseando JSON de tokens: {e}")))?;
    Ok(tokens)
}

/// Obtiene la direccion de correo del usuario autenticado consultando `userinfo`.
pub async fn fetch_user_email(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<Option<String>, OAuthError> {
    #[derive(Deserialize)]
    struct UserInfo {
        email: Option<String>,
    }

    let resp = client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let bytes = resp.bytes().await?;
    let info: UserInfo = serde_json::from_slice(&bytes)
        .map_err(|e| OAuthError::GoogleAuthError(format!("Error parseando userinfo: {e}")))?;
    Ok(info.email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_session_generation() {
        let session = PkceSession::new().unwrap();
        assert!(!session.code_verifier.is_empty());
        assert!(!session.code_challenge.is_empty());
        assert!(!session.state.is_empty());

        let url = build_authorization_url(
            "test-client-id.apps.googleusercontent.com",
            "http://127.0.0.1:8080/callback",
            &session,
        );

        assert!(url.starts_with(GOOGLE_AUTH_URL));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state="));
        assert!(url.contains("drive.appdata"));
    }

    #[test]
    fn test_parse_oauth_query_success() {
        let request =
            "GET /callback?code=4/0AbCdEf123&state=xyzState456 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        let (code, state) = parse_oauth_query(request).unwrap();
        assert_eq!(code, "4/0AbCdEf123");
        assert_eq!(state, "xyzState456");
    }

    #[test]
    fn test_parse_oauth_query_missing_code() {
        let request = "GET /callback?error=access_denied HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        assert!(parse_oauth_query(request).is_err());
    }

    #[tokio::test]
    async fn test_loopback_callback_reception() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = tokio::spawn(async move {
            listen_for_oauth_callback(listener, "expected_secret_state", 5).await
        });

        // Simulamos la peticion que Google envia tras la autorizacion
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        let req = format!("GET /callback?code=mock_auth_code_777&state=expected_secret_state HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n");
        stream.write_all(req.as_bytes()).await.unwrap();

        let mut res = [0u8; 1024];
        let n = stream.read(&mut res).await.unwrap();
        let res_str = String::from_utf8_lossy(&res[..n]);
        assert!(res_str.contains("200 OK"));
        assert!(res_str.contains("Autenticaci"));

        let code = handle.await.unwrap().unwrap();
        assert_eq!(code, "mock_auth_code_777");
    }
}
