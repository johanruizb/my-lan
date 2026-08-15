//! Banner grabbing básico (lectura corta no bloqueante).
//!
//! Tras establecer la conexión TCP se intenta leer un fragmento inicial del stream
//! con un timeout corto. Los servicios que envían un saludo al conectar (SSH, FTP,
//! SMTP, POP3, IMAP, Redis, etc.) quedan reflejados en el banner; los que esperan
//! una petición previa (HTTP) normalmente no devuelven nada y el banner queda en
//! `None`. La lectura es **no intrusiva** (P2): no se envía ninguna carga al destino,
//! solo se escucha pasivamente.

use std::time::Duration;

use tokio::io::AsyncReadExt as _;
use tokio::net::TcpStream;

/// Tamaño máximo del banner leído (suficiente para la línea de saludo).
const BANNER_MAX: usize = 256;

/// Lee hasta [`BANNER_MAX`] bytes del stream con un timeout de `timeout`.
///
/// Devuelve `None` si no llegan datos a tiempo, la conexión se cierra, o la lectura
/// falla. El banner se recorta de NUL y de espacios en blanco finales. Es
/// estrictamente pasiva: no escribe en el stream.
pub async fn grab_banner(stream: &mut TcpStream, timeout: Duration) -> Option<String> {
    let mut buf = [0u8; BANNER_MAX];
    let read = tokio::time::timeout(timeout, stream.read(&mut buf)).await;
    match read {
        Ok(Ok(0)) => None,
        Ok(Ok(n)) => trim_banner(&buf[..n]),
        Ok(Err(_)) | Err(_) => None,
    }
}

/// Recorta NUL y espacios en blanco extremos; descarta banners vacíos o no-UTF-8.
fn trim_banner(bytes: &[u8]) -> Option<String> {
    let trimmed = bytes
        .iter()
        .copied()
        .take_while(|&b| b != 0)
        .collect::<Vec<_>>();
    if trimmed.is_empty() {
        return None;
    }
    let text = String::from_utf8(trimmed).ok()?;
    let stripped = text.trim();
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tokio::io::AsyncWriteExt;

    /// Un listener que acepta y **no envía nada**: el banner debe agotar el timeout
    /// y devolver None dentro del plazo (respeta el timeout).
    #[tokio::test(flavor = "current_thread")]
    async fn banner_times_out_on_silent_listener() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            // Mantén la conexión abierta sin enviar nada hasta que el cliente cierre.
            let _ = tokio::time::timeout(Duration::from_secs(2), s.read(&mut [0u8; 1])).await;
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let timeout = Duration::from_millis(150);
        let start = Instant::now();
        let banner = grab_banner(&mut stream, timeout).await;
        let elapsed = start.elapsed();

        assert!(banner.is_none(), "listener silencioso => None");
        // El plazo se respeta con una holgura generosa (no debe acercarse a 2 s).
        assert!(elapsed <= timeout * 3, "transcurrido = {elapsed:?}");
        server.abort();
    }

    /// Un listener que envía un saludo: el banner se captura y recorta.
    #[tokio::test(flavor = "current_thread")]
    async fn banner_captures_greeting() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            s.write_all(b"SSH-2.0-OpenSSH_8.9\r\n").await.unwrap();
            // Mantén abierto un instante para que el cliente lea.
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let banner = grab_banner(&mut stream, Duration::from_millis(500)).await;
        assert_eq!(banner.as_deref(), Some("SSH-2.0-OpenSSH_8.9"));
    }

    #[test]
    fn trim_handles_nul_and_whitespace() {
        assert_eq!(
            trim_banner(b"HTTP/1.1 200 OK\r\n"),
            Some("HTTP/1.1 200 OK".into())
        );
        assert_eq!(trim_banner(b"banner\x00garbage"), Some("banner".into()));
        assert_eq!(trim_banner(b"   \r\n\t  "), None);
        assert_eq!(trim_banner(b""), None);
        assert_eq!(trim_banner(b"\x00"), None);
        // Bytes no-UTF-8 => None (sin pánico).
        assert_eq!(trim_banner(&[0xFF, 0xFE, b'x']), None);
    }
}
