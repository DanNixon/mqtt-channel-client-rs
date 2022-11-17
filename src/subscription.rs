use derive_builder::Builder;

/// MQTT subscription.
#[derive(Builder, Debug, Clone)]
pub struct Subscription {
    pub(crate) topic: String,

    #[builder(default = "0")]
    pub(crate) qos: i32,
}

impl SubscriptionBuilder {
    /// Set the QoS for this subscription to 0 (at most once).
    pub fn qos_at_most_once(&mut self) -> &mut Self {
        self.qos(0)
    }

    /// Set the QoS for this subscription to 1 (at least once).
    pub fn qos_at_least_once(&mut self) -> &mut Self {
        self.qos(1)
    }

    /// Set the QoS for this subscription to 2 (exactly once).
    pub fn qos_exactly_once(&mut self) -> &mut Self {
        self.qos(2)
    }
}
