//! Event types.

use paho_mqtt::Message;

#[derive(Debug, Clone)]
pub enum Event {
    /// Client status has changed.
    Status(StatusEvent),

    /// Send an MQTT message.
    Tx(Message),

    /// Received an MQTT message.
    Rx(Message),
}

/// Meta/status change event.
#[derive(Debug, Clone)]
pub enum StatusEvent {
    /// Client has connected to the MQTT broker.
    Connected,

    /// Client has disconnected from (or lost connection to) the MQTT broker.
    Disconnected,
}
