//! AC-2: degradación de privilegios — sin sudo, el path de fallback
//! (ICMP/TCP-ping/mDNS/SSDP) se ejecuta vía `mylan_discovery::discover`.
//! Verificamos que `discover` no paniquea ni cuelga sobre una iface mock sin
//! hosts (loopback /32). Determinista: no depende del prefix del runner CI.

use std::time::Duration;

use mylan_core::ScanProfile;
use mylan_discovery::{discover, DiscoverOptions, LanInterface};

fn loopback_iface() -> LanInterface {
    LanInterface {
        name: "lo".into(),
        ip: "127.0.0.1".parse().unwrap(),
        prefix_len: 32,
        mac: None,
        gateway_ip: None,
        gateway_mac: None,
        dns_servers: Vec::new(),
        ssid: None,
    }
}

#[tokio::test]
async fn discover_degrades_without_sudo_does_not_crash() {
    // AC-2: sin sudo, `discover` (path degradado TCP/ICMP/mDNS/SSDP) no cuelga
    // ni paniquea. Iface mock loopback /32 -> 0 hosts -> TCP/ICMP sweep vacío;
    // mDNS/SSDP bounded por sus timeouts (Quick 3s). Determinista, independiente
    // del prefix del runner CI.
    let iface = loopback_iface();
    let opts = DiscoverOptions::for_profile(ScanProfile::Quick);
    let result = tokio::time::timeout(Duration::from_secs(20), discover(&iface, &opts)).await;
    assert!(
        result.is_ok(),
        "discover no debe colgar ni paniquear sin sudo"
    );
}
