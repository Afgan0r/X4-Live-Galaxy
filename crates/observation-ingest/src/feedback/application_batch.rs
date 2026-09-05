use observation_domain::{BatchId, TransportEpoch};

use super::ApplicationContextIdentity;

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImmutableApplicationBatch {
    epoch: TransportEpoch,
    identity: BatchId,
    bytes: Vec<u8>,
    context: ApplicationContextIdentity,
}

impl ImmutableApplicationBatch {
    #[must_use]
    pub fn new(
        epoch: TransportEpoch,
        identity: BatchId,
        bytes: Vec<u8>,
        context: ApplicationContextIdentity,
    ) -> Option<Self> {
        (!bytes.is_empty()).then_some(Self {
            epoch,
            identity,
            bytes,
            context,
        })
    }
    pub const fn epoch(&self) -> TransportEpoch {
        self.epoch
    }
    pub const fn identity(&self) -> &BatchId {
        &self.identity
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn context(&self) -> &ApplicationContextIdentity {
        &self.context
    }
}
