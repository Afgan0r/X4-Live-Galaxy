use std::time::Duration;

use crate::{TransportConfig, TransportError, transport_types::validate};

#[derive(Debug)]
pub struct BridgePeer {
    #[cfg(windows)]
    handle: crate::abi_windows_io::RawHandle,
    max_data_message_bytes: usize,
    max_control_message_bytes: usize,
}

impl BridgePeer {
    pub fn connect(config: &TransportConfig, timeout: Duration) -> Result<Self, TransportError> {
        validate(config)?;
        #[cfg(windows)]
        {
            let name = crate::abi_windows_io::wide(&config.pipe_name);
            let handle = crate::abi_windows_peer::open_peer(&name, timeout)?;
            Ok(Self {
                handle,
                max_data_message_bytes: config.max_data_message_bytes,
                max_control_message_bytes: config.max_control_message_bytes,
            })
        }
        #[cfg(not(windows))]
        Err(TransportError::Unavailable)
    }

    pub fn receive(&mut self, capacity: usize) -> Result<Vec<u8>, TransportError> {
        if capacity == 0 || capacity > self.max_data_message_bytes {
            return Err(TransportError::InvalidConfig);
        }
        #[cfg(windows)]
        {
            crate::abi_windows_peer::peer_read(self.handle, capacity)
        }
        #[cfg(not(windows))]
        Err(TransportError::Unavailable)
    }

    pub fn send_control(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
        if bytes.len() > self.max_control_message_bytes {
            return Err(TransportError::MessageTooLarge);
        }
        #[cfg(windows)]
        {
            crate::abi_windows_peer::peer_write(self.handle, bytes)
        }
        #[cfg(not(windows))]
        Err(TransportError::Unavailable)
    }
}

impl Drop for BridgePeer {
    fn drop(&mut self) {
        #[cfg(windows)]
        crate::abi_windows_io::close_handle(self.handle);
    }
}
