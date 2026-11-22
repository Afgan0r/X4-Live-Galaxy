use crate::OperationalHistory;
use crate::production_ship_schedule::ShipAdmissionDenial;

#[derive(Clone, Copy)]
pub enum SelectionFailure {
    Cursor,
    Admission(ShipAdmissionDenial),
    Revision,
    Intent,
    Demand,
}

impl SelectionFailure {
    pub(crate) fn record(
        self,
        history: &mut OperationalHistory,
        attempt: u64,
        key: &str,
        revision: Option<u64>,
    ) {
        let section = if matches!(self, Self::Cursor) {
            ""
        } else {
            key
        };
        history.bind_selection(attempt, section, revision);
        let (state, reason) = self.event();
        let _ = history.record(state, reason);
    }

    const fn event(self) -> (&'static str, &'static str) {
        match self {
            Self::Cursor => ("rejected", "next-selection-cursor"),
            Self::Admission(ShipAdmissionDenial::WindowExhausted) => {
                ("waiting", "admission-window-exhausted")
            }
            Self::Admission(ShipAdmissionDenial::SelectionUnavailable) => {
                ("rejected", "next-selection-admission")
            }
            Self::Revision => ("rejected", "next-selection-revision"),
            Self::Intent => ("rejected", "next-selection-intent-send"),
            Self::Demand => ("rejected", "next-selection-demand-send"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_admission_cutoff_is_persisted_as_waiting() {
        let root = std::env::temp_dir().join(format!("history-admission-{}", std::process::id()));
        let mut history = OperationalHistory::open(&root).expect("open history");
        history.bind_selection(131, "ship_core:scaleplate", Some(131));
        assert!(history.record("committed", "committed"));
        SelectionFailure::Admission(ShipAdmissionDenial::WindowExhausted).record(
            &mut history,
            132,
            "ship_cargo:scaleplate:g0",
            None,
        );
        SelectionFailure::Admission(ShipAdmissionDenial::SelectionUnavailable).record(
            &mut history,
            133,
            "ship_core:teladi",
            None,
        );
        SelectionFailure::Demand.record(&mut history, 134, "ship_core:argon", Some(132));
        drop(history);

        let log =
            std::fs::read_to_string(root.join("operational-history.jsonl")).expect("read history");
        std::fs::remove_dir_all(root).expect("remove test history");
        let events: Vec<serde_json::Value> = log
            .lines()
            .map(|line| serde_json::from_str(line).expect("history event"))
            .collect();
        let cutoff = &events[2];
        assert_eq!(cutoff["state"], "waiting");
        assert_eq!(cutoff["reason"], "admission-window-exhausted");
        assert_eq!(cutoff["section"], "ship_cargo:scaleplate:g0");
        assert_eq!(cutoff["attempt"], 132);
        assert_eq!(cutoff.get("revision"), Some(&serde_json::Value::Null));
        assert_eq!(events[3]["state"], "rejected");
        assert_eq!(events[3]["reason"], "next-selection-admission");
        assert_eq!(events[3]["section"], "ship_core:teladi");
        assert_eq!(events[3]["attempt"], 133);
        assert_eq!(events[3].get("revision"), Some(&serde_json::Value::Null));
        assert_eq!(events[4]["state"], "rejected");
        assert_eq!(events[4]["reason"], "next-selection-demand-send");
        assert_eq!(events[4]["revision"], 132);
    }
}
