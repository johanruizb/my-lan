//! Wake-on-LAN (WoL) — Envío de Magic Packets a través de UDP broadcast.
//!
//! El Magic Packet consiste en 6 bytes 0xFF seguidos de 16 repeticiones
//! de la dirección MAC de 48 bits del dispositivo destino (102 bytes en total).

use mylan_core::MacAddr;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::error::DiscoveryError;

/// Construye el payload del Magic Packet de 102 bytes para una MAC dada.
#[must_use]
pub fn build_magic_packet(mac: &MacAddr) -> [u8; 102] {
    let mut packet = [0u8; 102];
    packet[..6].fill(0xFF);
    let octets = mac.octets();
    for i in 0..16 {
        packet[6 + i * 6..6 + (i + 1) * 6].copy_from_slice(&octets);
    }
    packet
}

/// Envía un Magic Packet de Wake-on-LAN hacia la MAC especificada.
///
/// Si `broadcast_ip` es `None`, se utiliza la dirección de difusión limitada `255.255.255.255:9`.
pub async fn send_wol_packet(
    mac_str: &str,
    broadcast_ip: Option<&str>,
) -> Result<(), DiscoveryError> {
    let mac = MacAddr::parse(mac_str).ok_or_else(|| {
        DiscoveryError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("dirección MAC inválida: '{mac_str}'"),
        ))
    })?;

    let packet = build_magic_packet(&mac);
    let bcast = broadcast_ip.unwrap_or("255.255.255.255");
    let target_addr: SocketAddr = format!("{bcast}:9").parse().map_err(|e| {
        DiscoveryError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("dirección broadcast inválida: '{bcast}': {e}"),
        ))
    })?;

    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(DiscoveryError::Io)?;
    socket.set_broadcast(true).map_err(DiscoveryError::Io)?;
    socket
        .send_to(&packet, target_addr)
        .await
        .map_err(DiscoveryError::Io)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_packet_structure() {
        let mac = MacAddr::parse("aa:bb:cc:dd:ee:ff").expect("valid mac");
        let packet = build_magic_packet(&mac);

        assert_eq!(packet.len(), 102);
        // Primeros 6 bytes deben ser 0xFF
        assert_eq!(&packet[..6], &[0xFF; 6]);

        // Siguientes 16 repeticiones de la MAC
        for i in 0..16 {
            let chunk = &packet[6 + i * 6..6 + (i + 1) * 6];
            assert_eq!(chunk, &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        }
    }

    #[tokio::test]
    async fn test_send_wol_packet_invalid_mac() {
        let result = send_wol_packet("invalid-mac-address", None).await;
        assert!(result.is_err());
    }
}
