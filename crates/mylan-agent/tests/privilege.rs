//! AC-2: degradación de privilegios — sin sudo, el path de fallback
//! (ICMP/TCP-ping/mDNS/SSDP) se ejecuta vía `mylan_discovery::discover`.
//! Verificamos que `discover_with_degradation` no paniquea ni cuelga (CI sin
//! LAN real → devuelve un `Vec`, posiblemente vacío).

use std::time::Duration;

use mylan_agent::discover_with_degradation;
use mylan_core::ScanProfile;

#[tokio::test]
async fn discover_with_degradation_does_not_crash_without_sudo() {
    // En CI sin sudo, el descubrimiento degrada a TCP-connect + ICMP no-root +
    // mDNS + SSDP. Sin red LAN real puede devolver vacío; lo importante es que
    // no paniquea ni cuelga (detect_interface + discover best-effort). M5 fix:
    // el profile se pasa y controla los timeouts (aquí Quick para mantener el
    // presupuesto del test).
    let result = tokio::time::timeout(
        Duration::from_secs(20),
        discover_with_degradation("127.0.0.0/8", ScanProfile::Quick),
    )
    .await;
    assert!(
        result.is_ok(),
        "discover_with_degradation no debe colgar ni paniquear"
    );
}
