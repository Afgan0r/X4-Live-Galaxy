use crate::{ProducerIncarnationId, TransportEpoch};

#[must_use]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceSessionIdentity {
    producer_incarnation: ProducerIncarnationId,
    transport_epoch: TransportEpoch,
}

impl SourceSessionIdentity {
    pub const fn new(
        producer_incarnation: ProducerIncarnationId,
        transport_epoch: TransportEpoch,
    ) -> Self {
        Self {
            producer_incarnation,
            transport_epoch,
        }
    }

    pub const fn producer_incarnation(&self) -> &ProducerIncarnationId {
        &self.producer_incarnation
    }

    pub const fn transport_epoch(&self) -> TransportEpoch {
        self.transport_epoch
    }
}
