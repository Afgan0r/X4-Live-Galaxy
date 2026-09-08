use std::{
    fmt,
    sync::{
        Arc, Mutex,
        atomic::Ordering,
        mpsc::{SyncSender, TrySendError, sync_channel},
    },
    time::{Duration, Instant},
};

use crate::{
    CloseProgress, HandleToken, SecurityEvidence, TransportConfig, TransportError,
    TransportSendOutcome, WorkerSnapshot,
    transport_types::{Shared, validate},
};

#[path = "transport_control.rs"]
mod control;
#[cfg(test)]
#[path = "transport_test_seam.rs"]
mod test_seam;

pub struct NativeTransport {
    config: TransportConfig,
    evidence: SecurityEvidence,
    shared: Arc<Shared>,
    data_tx: SyncSender<Vec<u8>>,
    control_rx: Mutex<std::sync::mpsc::Receiver<(u64, Vec<u8>)>>,
    control_pending: Mutex<Option<(u64, Vec<u8>)>>,
}

impl fmt::Debug for NativeTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeTransport")
            .field("config", &self.config)
            .field("snapshot", &self.snapshot())
            .finish_non_exhaustive()
    }
}

impl NativeTransport {
    pub fn start(config: TransportConfig, token: HandleToken) -> Result<Self, TransportError> {
        validate(&config)?;
        #[cfg(windows)]
        {
            let security = crate::abi_windows_security::PipeSecurity::current_logon()?;
            let evidence = security.evidence();
            let name = crate::abi_windows_io::wide(&config.pipe_name);
            let data = u32::try_from(config.max_data_message_bytes)
                .map_err(|_| TransportError::InvalidConfig)?;
            let control = u32::try_from(config.max_control_message_bytes)
                .map_err(|_| TransportError::InvalidConfig)?;
            let pipe =
                crate::abi_windows_io::create_server(&name, data, control, security.attributes())
                    .map_err(|_| TransportError::PipeCreationFailed)?;
            let shared = Arc::new(Shared::new(token));
            update_clock(&shared);
            let (data_tx, data_rx) = sync_channel(1);
            let (control_tx, control_rx) = sync_channel(1);
            let context = Box::new(crate::transport_worker::WorkerContext {
                config: config.clone(),
                pipe,
                shared: Arc::clone(&shared),
                data_rx,
                control_tx,
            });
            crate::transport_worker::start(context)
                .map_err(|_| TransportError::WorkerStartFailed)?;
            Ok(Self {
                config,
                evidence,
                shared,
                data_tx,
                control_rx: Mutex::new(control_rx),
                control_pending: Mutex::new(None),
            })
        }
        #[cfg(not(windows))]
        Err(TransportError::Unavailable)
    }

    #[must_use]
    pub const fn security_evidence(&self) -> SecurityEvidence {
        self.evidence
    }

    pub fn try_send(&self, token: HandleToken, bytes: &[u8]) -> TransportSendOutcome {
        if token != self.shared.token {
            return TransportSendOutcome::Rejected(TransportError::StaleGeneration);
        }
        if bytes.len() > self.config.max_data_message_bytes {
            return TransportSendOutcome::Rejected(TransportError::MessageTooLarge);
        }
        if self.shared.close_requested.load(Ordering::Acquire) {
            return TransportSendOutcome::CapacityUnavailable;
        }
        if self
            .shared
            .data_busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return TransportSendOutcome::CapacityUnavailable;
        }
        match self.data_tx.try_send(bytes.to_vec()) {
            Ok(()) => TransportSendOutcome::LocalHandoff,
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                self.shared.data_busy.store(false, Ordering::Release);
                TransportSendOutcome::CapacityUnavailable
            }
        }
    }

    pub fn request_close(&self, token: HandleToken) -> CloseProgress {
        if token != self.shared.token {
            return CloseProgress::StaleGeneration;
        }
        if self.shared.closed.load(Ordering::Acquire) {
            return CloseProgress::Complete;
        }
        self.shared.close_requested.store(true, Ordering::Release);
        CloseProgress::Requested
    }

    #[must_use]
    pub fn snapshot(&self) -> WorkerSnapshot {
        WorkerSnapshot {
            connected: self.shared.connected.load(Ordering::Acquire),
            connection_generation: self.shared.connection_generation.load(Ordering::Acquire),
            pending_operation_owners: self.shared.owners.load(Ordering::Acquire),
            monotonic_millis: self
                .shared
                .clock_available
                .load(Ordering::Acquire)
                .then(|| self.shared.millis.load(Ordering::Acquire)),
            closed: self.shared.closed.load(Ordering::Acquire),
        }
    }

    pub fn wait_closed_for_test(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while !self.shared.closed.load(Ordering::Acquire) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        self.shared.closed.load(Ordering::Acquire)
    }
}

impl Drop for NativeTransport {
    fn drop(&mut self) {
        self.shared.close_requested.store(true, Ordering::Release);
    }
}

fn update_clock(shared: &Shared) {
    let Ok(now) = crate::abi_windows::monotonic_millis() else {
        return;
    };
    shared.millis.store(now, Ordering::Release);
    shared.clock_available.store(true, Ordering::Release);
}
