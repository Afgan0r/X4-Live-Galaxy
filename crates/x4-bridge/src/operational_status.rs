use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::diagnostics::escape;

pub fn write(
    history_path: &Path,
    state: &str,
    reason: &str,
    history_gaps: u64,
    suppressed: u64,
    status_gaps: u64,
) -> bool {
    let path = history_path.with_file_name("operational-status.json");
    let value = format!(
        "{{\"component\":\"x4-bridge\",\"state\":\"{}\",\"reason\":\"{}\",\"history_gap_count\":{},\"status_gap_count\":{},\"suppressed_count\":{}}}\n",
        escape(state),
        escape(reason),
        history_gaps,
        status_gaps,
        suppressed
    );
    std::fs::write(path, value).is_ok()
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| {
            u64::try_from(value.as_millis()).unwrap_or(u64::MAX)
        })
}
