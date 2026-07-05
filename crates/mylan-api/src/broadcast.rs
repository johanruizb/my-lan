//! Canal de broadcast de eventos (ADR-4).
//!
//! El agent (único proceso) posee el `Sender<Event>` y lo pasa al API vía
//! [`crate::serve`]; el API lo guarda en `AppState` para que el WS
//! `/api/v1/events/live` se suscriba al `Receiver`. Si no hay clientes WS, el
//! `send` es no-op (sin backpressure); la DB (`events` table) es la fuente de
//! verdad, el WS es una vista en vivo (Principle 4).

use tokio::sync::broadcast;

use mylan_core::Event;

/// Crea un canal de broadcast de eventos con la capacidad dada.
///
/// Convención del plan: `capacity = 1024`. El `Sender` es `Clone` (barato,
/// referencia contada al canal); el `Receiver` no es `Clone`.
#[must_use]
pub fn event_channel(capacity: usize) -> (broadcast::Sender<Event>, broadcast::Receiver<Event>) {
    broadcast::channel(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_then_recv_round_trip() {
        let (tx, mut rx) = event_channel(16);
        let event = Event {
            id: "evt-1".to_string(),
            network_id: "net-1".to_string(),
            device_id: Some("dev-1".to_string()),
            event_type: mylan_core::EventType::DeviceNew,
            severity: mylan_core::Severity::Info,
            message: Some("nuevo device".to_string()),
            data_json: None,
            created_at: "2026-07-03T00:00:00Z".to_string(),
        };
        tx.send(event.clone()).expect("send");
        let got = rx.recv().await.expect("recv");
        assert_eq!(got, event);
    }

    #[tokio::test]
    async fn send_with_no_receivers_is_ok_no_block() {
        let (tx, _rx) = event_channel(16);
        // Sin receptores vivos: send es no-op (no bloquea, no backpressure).
        let event = Event {
            id: "evt-2".to_string(),
            network_id: "net-1".to_string(),
            device_id: None,
            event_type: mylan_core::EventType::DeviceOffline,
            severity: mylan_core::Severity::Warning,
            message: None,
            data_json: None,
            created_at: "2026-07-03T00:00:01Z".to_string(),
        };
        // drop el receiver para simular "sin clientes".
        drop(_rx);
        // send devuelve Err (no hay receptores) pero NO bloquea ni entra en
        // pánico — el caller puede ignorarlo.
        let _ = tx.send(event);
    }
}
