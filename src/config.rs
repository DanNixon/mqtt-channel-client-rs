use derive_builder::Builder;
use std::time::Duration;

/// Miscellaneous client configuration.
#[derive(Builder, Debug, Clone)]
#[builder(default)]
pub struct ClientConfig {
    /// Size of the Tokio channel.
    pub(crate) channel_size: usize,

    /// Size of the MQTT client buffer.
    pub(crate) stream_size: usize,

    /// Iteration interval for internal client event polling.
    pub(crate) beat_interval: Duration,

    /// Metric name prefix
    #[cfg(feature = "metrics")]
    pub(crate) metrics_prefix: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            channel_size: 16,
            stream_size: 25,
            beat_interval: Duration::from_millis(100),
            #[cfg(feature = "metrics")]
            metrics_prefix: "mqtt".into(),
        }
    }
}
