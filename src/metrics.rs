use derive_builder::Builder;
use prometheus_client::{
    encoding::{EncodeLabelSet, EncodeLabelValue},
    metrics::{counter::Counter, family::Family},
};

#[derive(Default, Clone)]
pub(crate) struct MetricCollection {
    pub(crate) messages: Family<MessageLabels, Counter>,
    pub(crate) connection_events: Family<ConnectionEventLabels, Counter>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelValue)]
pub(crate) enum MessageDirection {
    Sent,
    Received,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelValue)]
pub(crate) enum MessageResult {
    Success,
    Failure,
}

#[derive(Debug, Builder, Clone, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(crate) struct MessageLabels {
    direction: MessageDirection,
    topic: String,
    #[builder(default = "MessageResult::Success")]
    result: MessageResult,
}

impl MessageLabelsBuilder {
    pub(crate) fn sent(&mut self) -> &mut Self {
        self.direction(MessageDirection::Sent)
    }

    pub(crate) fn received(&mut self) -> &mut Self {
        self.direction(MessageDirection::Received)
    }

    pub(crate) fn failure(&mut self) -> &mut Self {
        self.result(MessageResult::Failure)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelValue)]
pub(crate) enum ConnectionEvent {
    Connected,
    Disconnected,
    Lost,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(crate) struct ConnectionEventLabels {
    kind: ConnectionEvent,
}

impl ConnectionEventLabels {
    pub(crate) fn connected() -> Self {
        Self {
            kind: ConnectionEvent::Connected,
        }
    }

    pub(crate) fn disconnected() -> Self {
        Self {
            kind: ConnectionEvent::Disconnected,
        }
    }

    pub(crate) fn lost() -> Self {
        Self {
            kind: ConnectionEvent::Lost,
        }
    }
}
