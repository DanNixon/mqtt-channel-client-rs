use derive_builder::Builder;

/// Miscellaneous client configuration.
#[derive(Builder, Debug, Clone)]
#[builder(default)]
pub struct ClientConfig {
    /// Size of the Tokio channel.
    pub(crate) channel_size: usize,

    /// Metric name prefix
    #[cfg(feature = "metrics")]
    pub(crate) metrics_prefix: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            channel_size: 16,
            #[cfg(feature = "metrics")]
            metrics_prefix: "mqtt".into(),
        }
    }
}
