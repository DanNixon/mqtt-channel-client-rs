use crate::Event;

/// Result type for MQTT errors.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for fallible operations in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MQTT error")]
    MqttError(#[from] paho_mqtt::Error),

    #[error("Task join error")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Channel error")]
    ChannelError(#[from] tokio::sync::broadcast::error::SendError<Event>),

    #[error("Client was requested to start but is already started")]
    ClientAlreadyStarted,

    #[error("Client was requested to stop but is already stopped")]
    ClientAlreadyStopped,
}
