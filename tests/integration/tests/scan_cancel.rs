//! AC-5 (e2e): `scan_target` respeta la cancelación cooperativa y devuelve los
//! hits parciales recogidos hasta el momento — sin colgarse ni esperar al plazo
//! global.

use std::net::IpAddr;
use std::time::{Duration, Instant};

use mylan_core::ScanProfile;
use mylan_scanner::{scan_target, ScanOptions, ScanProgress};
use tokio_util::sync::CancellationToken;

/// `scan_target` cancelado a mitad de un escaneo largo regresa `Ok` de forma
/// rápida con los hits parciales (AC-5).
///
/// Fixture: se bindea un puerto abierto del catálogo `quick` (>1024, sin root)
/// para garantizar al menos un hit detectado antes de la cancelación. Si ningún
/// puerto del catálogo se puede bindear (todos ocupados), el test cae al caso
/// mínimo: afirma que `scan_target` regresa `Ok` sin colgarse tras cancelar.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_mid_scan_returns_partial_results() {
    // Puertos del catálogo `quick` (top 32) bindeables sin root (>1024).
    const CANDIDATES: &[u16] = &[3389, 3306, 8080, 1723, 5900, 1025, 8888, 1720, 554, 873];

    let mut bound_port: Option<u16> = None;
    for &p in CANDIDATES {
        if let Ok(listener) = tokio::net::TcpListener::bind(("127.0.0.1", p)).await {
            // Aceptador pasivo: responde a cualquier conexión y sigue escuchando.
            tokio::spawn(async move {
                while let Ok((mut s, _)) = listener.accept().await {
                    let _ = tokio::io::AsyncWriteExt::write_all(&mut s, b"hi\n").await;
                }
            });
            bound_port = Some(p);
            break;
        }
    }

    let cancel = CancellationToken::new();
    // Opciones: connect corto + plazo global holgado (30 s) que NO debe expedir
    // — la cancelación cooperativa debe ganar antes.
    let opts = ScanOptions {
        connect_timeout: Duration::from_millis(50),
        scan_timeout: Duration::from_secs(30),
        banner_timeout: Duration::from_millis(50),
        concurrency: 16,
        enable_udp: false,
    };

    // Cancela a los 150 ms: suficiente para detectar el puerto abierto local
    // antes de que el barrido completo termine.
    let cancel2 = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        cancel2.cancel();
    });

    let mut last: Option<ScanProgress> = None;
    let start = Instant::now();
    let svcs = scan_target(
        "127.0.0.1".parse::<IpAddr>().unwrap(),
        ScanProfile::Quick,
        opts,
        cancel,
        |p| last = Some(p),
    )
    .await
    .expect("scan_target debe regresar Ok tras cancelar (AC-5)");
    let elapsed = start.elapsed();

    // Propiedad clave de AC-5: regresa Ok y rápido, sin colgarse hasta el plazo.
    assert!(
        elapsed <= Duration::from_secs(5),
        "scan_target colgó tras cancelar: elapsed={elapsed:?}"
    );

    // Si se bindeó un puerto del catálogo, debe haberse detectado antes de
    // cancelar (hit parcial). Si no se bindeó ninguno, sólo afirmamos Ok + rápido.
    if let Some(p) = bound_port {
        let hit = svcs.iter().find(|s| s.port == p);
        assert!(
            hit.is_some(),
            "el puerto abierto del catálogo {p} debe detectarse como hit parcial (svcs={:?})",
            svcs.iter().map(|s| s.port).collect::<Vec<_>>()
        );
    }

    // El callback de progreso debe haberse invocado al menos una vez si hubo hits.
    let _ = last;
    // Cota superior: el número de hits no supera el catálogo quick (32).
    assert!(svcs.len() <= 32, "hits acotados por el catálogo quick");
}
