use std::sync::{TryLockError, atomic::Ordering};

use crate::{HandleToken, NativeTransport, TransportError, TransportPoll};

impl NativeTransport {
    pub fn poll_control(&self, token: HandleToken, capacity: usize) -> TransportPoll {
        if token != self.shared.token {
            return TransportPoll::Rejected(TransportError::StaleGeneration);
        }
        if self.shared.closed.load(Ordering::Acquire) {
            return TransportPoll::Closed;
        }
        let mut pending = match self.control_pending.try_lock() {
            Ok(value) => value,
            Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => {
                return TransportPoll::NoMessage;
            }
        };
        select_current(self, &mut pending, capacity)
    }

    fn try_receive_control(&self) -> Option<(u64, Vec<u8>)> {
        let Ok(receiver) = self.control_rx.try_lock() else {
            return None;
        };
        receiver.try_recv().ok()
    }

    pub fn request_reconnect(&self, token: HandleToken) -> Result<(), TransportError> {
        if token != self.shared.token {
            return Err(TransportError::StaleGeneration);
        }
        self.shared
            .reconnect_requested
            .store(true, Ordering::Release);
        Ok(())
    }
}

fn select_current(
    transport: &NativeTransport,
    pending: &mut Option<(u64, Vec<u8>)>,
    capacity: usize,
) -> TransportPoll {
    let Some(value) = pending.take().or_else(|| transport.try_receive_control()) else {
        return TransportPoll::NoMessage;
    };
    let generation = transport
        .shared
        .connection_generation
        .load(Ordering::Acquire);
    if value.0 != generation {
        return select_current(transport, pending, capacity);
    }
    if value.1.len() > capacity {
        *pending = Some(value);
        return TransportPoll::Rejected(TransportError::InvalidOutputCapacity);
    }
    TransportPoll::Message(value.1)
}
