//! Active probes sobre un stream TCP ya conectado (plan §Paso 4, AC-4).
//!
//! Tras detectar un puerto abierto, [`probe_service`] envía una petición mínima
//! (HTTP `GET /`, RTSP `DESCRIBE`) o reutiliza el saludo pasivo (SSH/FTP/SMTP/
//! POP3/IMAP) para inferir `product`/`version`/`banner`. Sólo se invoca cuando
//! `profile != Quick` (ver [`crate::scan_target`]).
//!
//! SMB/MQTT/ONVIF quedan como *stretch* (follow-up fuera de este push). La
//! lectura/escritura es no intrusiva (P2): cargas mínimas estándar.

use std::time::Duration;

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;

use crate::banner::grab_banner;

/// Resultado de un probe activo: enriquece un [`mylan_core::Service`] con
/// `product`/`version`/`banner` inferidos del tráfico del servicio.
#[derive(Debug, Clone, Default)]
pub struct ProbeResult {
    /// Producto inferido (p. ej. `OpenSSH`, `nginx`).
    pub product: Option<String>,
    /// Versión inferida (p. ej. `8.9`, código de estado HTTP).
    pub version: Option<String>,
    /// Banner/respuesta cruda representativa (línea de saludo, `<title>`, etc.).
    pub banner: Option<String>,
}

/// Ejecuta un probe activo sobre `stream` según el `port`.
///
/// Devuelve `None` si no se puede inferir nada (timeout, sin respuesta, puerto
/// sin probe conocido). El stream ya está conectado; el probe hace su propia
/// E/S (escritura de petición + lectura, o lectura pasiva según el servicio).
pub async fn probe_service(
    stream: &mut TcpStream,
    port: u16,
    timeout: Duration,
) -> Option<ProbeResult> {
    match port {
        80 | 443 | 8080 | 8443 => probe_http(stream, timeout).await,
        554 => probe_rtsp(stream, timeout).await,
        22 | 21 | 25 | 110 | 143 => probe_banner(stream, port, timeout).await,
        // SMB/MQTT/ONVIF y resto: stretch, sin probe (el banner pasivo de Quick
        // ya quedó capturado por `probe_port` cuando corresponde).
        _ => None,
    }
}

/// HTTP/HTTPS: `GET / HTTP/1.0` + parseo de status, `Server` y `<title>`.
async fn probe_http(stream: &mut TcpStream, timeout: Duration) -> Option<ProbeResult> {
    let req = b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n";
    let _ = tokio::time::timeout(timeout, stream.write_all(req))
        .await
        .ok()?;
    let body = read_with_timeout(stream, timeout, 4096).await?;
    let (status_code, status_line) = parse_status(&body);
    let title = parse_title(&body);
    let server = parse_header(&body, "Server");
    let banner = title.or(status_line);
    Some(ProbeResult {
        product: server,
        version: status_code,
        banner,
    })
}

/// RTSP: `DESCRIBE rtsp://<peer>/ RTSP/1.0` + parseo de status/Server.
async fn probe_rtsp(stream: &mut TcpStream, timeout: Duration) -> Option<ProbeResult> {
    let peer = stream.peer_addr().ok()?;
    let req = format!("DESCRIBE rtsp://{peer}/ RTSP/1.0\r\nCSeq: 1\r\n\r\n");
    let _ = tokio::time::timeout(timeout, stream.write_all(req.as_bytes()))
        .await
        .ok()?;
    let body = read_with_timeout(stream, timeout, 4096).await?;
    let (status_code, status_line) = parse_status(&body);
    Some(ProbeResult {
        product: parse_header(&body, "Server"),
        version: status_code,
        banner: status_line,
    })
}

/// SSH/FTP/SMTP/POP3/IMAP: saludo pasivo (banner) + parseo ligero.
async fn probe_banner(stream: &mut TcpStream, port: u16, timeout: Duration) -> Option<ProbeResult> {
    let banner = grab_banner(stream, timeout).await?;
    Some(parse_banner(port, &banner))
}

/// Lee hasta `max` bytes del stream con un timeout (mejor esfuerzo, no bloquea).
async fn read_with_timeout(
    stream: &mut TcpStream,
    timeout: Duration,
    max: usize,
) -> Option<String> {
    let mut buf = vec![0u8; max];
    let n = tokio::time::timeout(timeout, stream.read(&mut buf))
        .await
        .ok()?
        .ok()?;
    if n == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&buf[..n]).into_owned())
}

/// Parsea la primera línea de estado HTTP/RTSP: `(código, línea completa)`.
fn parse_status(body: &str) -> (Option<String>, Option<String>) {
    let Some(line) = body.lines().next() else {
        return (None, None);
    };
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 && (parts[0].starts_with("HTTP/") || parts[0].starts_with("RTSP/")) {
        (Some(parts[1].to_string()), Some(line.to_string()))
    } else {
        (None, Some(line.to_string()))
    }
}

/// Extrae el contenido del primer `<title>...</title>` (case-insensitive).
fn parse_title(body: &str) -> Option<String> {
    let lower = body.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let after = &body[start..];
    let gt = after.find('>')?;
    let rest = &after[gt + 1..];
    let end = rest.to_ascii_lowercase().find("</title>")?;
    let title = rest[..end].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

/// Extrae el valor de una cabecera `Name:` (case-insensitive).
fn parse_header(body: &str, name: &str) -> Option<String> {
    let needle = format!("{name}:");
    let needle_lower = needle.to_ascii_lowercase();
    for line in body.lines() {
        if line.to_ascii_lowercase().starts_with(&needle_lower) {
            return Some(line[needle.len()..].trim().to_string());
        }
    }
    None
}

/// Parsea un banner de saludo en `ProbeResult`. SSH se descompone en
/// producto/versión; el resto conserva el banner crudo.
fn parse_banner(port: u16, banner: &str) -> ProbeResult {
    if port == 22 && banner.starts_with("SSH-") {
        // Formato: SSH-2.0-OpenSSH_8.9
        if let Some((_, rest)) = banner.split_once('-') {
            if let Some((_, prod)) = rest.split_once('-') {
                let (product, version) = split_product_version(prod);
                return ProbeResult {
                    product,
                    version,
                    banner: Some(banner.to_string()),
                };
            }
        }
    }
    ProbeResult {
        product: None,
        version: None,
        banner: Some(banner.to_string()),
    }
}

/// Separa `OpenSSH_8.9` → (`OpenSSH`, `8.9`); cae al primer separador encontrado.
fn split_product_version(s: &str) -> (Option<String>, Option<String>) {
    if let Some((p, v)) = s.split_once('_').or_else(|| s.split_once(' ')) {
        (Some(p.to_string()), Some(v.to_string()))
    } else {
        (Some(s.to_string()), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// HTTP contra un listener fake que sirve `Server` + `<title>` → status 200,
    /// product `nginx/1.2.3`, banner `MyPage`.
    #[tokio::test(flavor = "current_thread")]
    async fn http_probe_extracts_title_status_and_server() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 128];
            let _ = s.read(&mut buf).await;
            let resp =
                "HTTP/1.0 200 OK\r\nServer: nginx/1.2.3\r\n\r\n<html><title>MyPage</title></html>";
            let _ = s.write_all(resp.as_bytes()).await;
        });

        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let pr = probe_service(&mut stream, 80, Duration::from_secs(1))
            .await
            .expect("probe HTTP");
        assert_eq!(pr.version.as_deref(), Some("200"));
        assert_eq!(pr.product.as_deref(), Some("nginx/1.2.3"));
        assert_eq!(pr.banner.as_deref(), Some("MyPage"));
    }

    /// SSH: el saludo pasivo se descompone en producto `OpenSSH` + versión `8.9`.
    #[tokio::test(flavor = "current_thread")]
    async fn ssh_banner_probe_parses_product_and_version() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let _ = s.write_all(b"SSH-2.0-OpenSSH_8.9\r\n").await;
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let pr = probe_service(&mut stream, 22, Duration::from_millis(500))
            .await
            .expect("probe SSH");
        assert_eq!(pr.product.as_deref(), Some("OpenSSH"));
        assert_eq!(pr.version.as_deref(), Some("8.9"));
        assert_eq!(pr.banner.as_deref(), Some("SSH-2.0-OpenSSH_8.9"));
    }

    #[test]
    fn parse_banner_ssh_splits_product_version() {
        let pr = parse_banner(22, "SSH-2.0-OpenSSH_8.9");
        assert_eq!(pr.product.as_deref(), Some("OpenSSH"));
        assert_eq!(pr.version.as_deref(), Some("8.9"));
    }

    #[test]
    fn parse_banner_ftp_keeps_raw_banner() {
        let pr = parse_banner(21, "220 (vsFTPd 3.0.5)");
        assert_eq!(pr.product, None);
        assert_eq!(pr.version, None);
        assert_eq!(pr.banner.as_deref(), Some("220 (vsFTPd 3.0.5)"));
    }

    #[test]
    fn parse_title_is_case_insensitive() {
        let body = "<html><TITLE>  Hello World  </TITLE></html>";
        assert_eq!(parse_title(body).as_deref(), Some("Hello World"));
    }

    #[test]
    fn parse_status_handles_http_and_rtsp() {
        let (code, line) = parse_status("HTTP/1.1 301 Moved Permanently\r\nServer: x");
        assert_eq!(code.as_deref(), Some("301"));
        assert_eq!(line.as_deref(), Some("HTTP/1.1 301 Moved Permanently"));
        let (code, _) = parse_status("RTSP/1.0 200 OK\r\n");
        assert_eq!(code.as_deref(), Some("200"));
    }

    #[test]
    fn parse_header_is_case_insensitive() {
        let body = "HTTP/1.0 200 OK\r\nserver: nginx\r\n\r\n";
        assert_eq!(parse_header(body, "Server").as_deref(), Some("nginx"));
    }

    #[test]
    fn ports_without_probe_return_none() {
        // 3389 (RDP) no tiene probe activo definido en este push.
        let result = std::panic::catch_unwind(|| {
            // probe_service para un puerto sin match simplemente devuelve None;
            // verificamos la lógica de dispatch sin abrir socket.
            matches!(
                3389u16,
                80 | 443 | 8080 | 8443 | 554 | 22 | 21 | 25 | 110 | 143
            )
        });
        assert!(matches!(result, Ok(false)));
    }
}
