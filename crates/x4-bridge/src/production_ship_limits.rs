use crate::ProductionLimits;
use std::path::Path;

const FIELDS: [&str; 15] = [
    "heavy_profile_version",
    "experimental_profile",
    "group_members",
    "max_inner_records",
    "max_allocation_bytes",
    "max_total_allocation_bytes",
    "max_native_calls",
    "max_collection_steps",
    "callback_budget_millis",
    "heavy_permits",
    "rate_interval_millis",
    "max_overrun_debt",
    "admission_window_millis",
    "freshness_millis",
    "retained_revisions",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeavyShipLimits {
    pub bridge: ProductionLimits,
    pub group_members: usize,
    pub max_inner_records: usize,
    pub max_allocation_bytes: usize,
    pub max_total_allocation_bytes: usize,
    pub max_native_calls: usize,
    pub max_collection_steps: usize,
    pub callback_budget_millis: usize,
    pub heavy_permits: usize,
    pub rate_interval_millis: usize,
    pub max_overrun_debt: usize,
    pub admission_window_millis: usize,
    pub freshness_millis: usize,
    pub retained_revisions: usize,
}

impl HeavyShipLimits {
    #[must_use]
    pub fn read(path: &Path) -> Option<Self> {
        Self::parse(&std::fs::read_to_string(path).ok()?)
    }
    #[must_use]
    pub fn parse(contents: &str) -> Option<Self> {
        if contents.len() > 4096 {
            return None;
        }
        let body = contents.trim().strip_prefix('{')?.strip_suffix('}')?;
        let (bridge, values) = entries(body)?;
        if values[0]? != 1 || values[1]? != 1 {
            return None;
        }
        let value = Self {
            bridge,
            group_members: values[2]?,
            max_inner_records: values[3]?,
            max_allocation_bytes: values[4]?,
            max_total_allocation_bytes: values[5]?,
            max_native_calls: values[6]?,
            max_collection_steps: values[7]?,
            callback_budget_millis: values[8]?,
            heavy_permits: values[9]?,
            rate_interval_millis: values[10]?,
            max_overrun_debt: values[11]?,
            admission_window_millis: values[12]?,
            freshness_millis: values[13]?,
            retained_revisions: values[14]?,
        };
        value.valid().then_some(value)
    }
    pub(crate) fn valid(&self) -> bool {
        self.bridge.valid()
            && [
                self.max_inner_records,
                self.max_allocation_bytes,
                self.max_total_allocation_bytes,
                self.max_native_calls,
                self.max_collection_steps,
                self.callback_budget_millis,
                self.rate_interval_millis,
                self.max_overrun_debt,
                self.admission_window_millis,
                self.freshness_millis,
                self.retained_revisions,
            ]
            .iter()
            .all(|value| *value > 0)
            && self.group_members == self.bridge.max_candidate_records
            && self.heavy_permits == 1
            && self.bridge.max_delivery_attempts == 1
            && self.bridge.reconnect_attempts == 1
            && self.max_inner_records <= self.bridge.max_candidate_records
            && self.max_allocation_bytes <= self.max_total_allocation_bytes
            && self.max_total_allocation_bytes <= self.bridge.max_total_bytes
            && self.max_native_calls <= self.max_collection_steps
            && self.callback_budget_millis < self.rate_interval_millis
            && self.bridge.max_message_age_millis <= self.admission_window_millis
            && self.freshness_millis <= self.admission_window_millis
            && self.retained_revisions >= 2
            && self.bridge.complete_message_bytes <= self.bridge.max_candidate_raw_bytes
            // Compatibility count fields are resource-derived: even the
            // smallest record/batch/operation consumes a byte/work unit.
            && self.bridge.max_candidate_records == self.bridge.max_candidate_raw_bytes
            && self.bridge.max_candidate_batches == self.bridge.max_candidate_raw_bytes
            && self.bridge.max_aggregate_records == self.bridge.max_aggregate_bytes
            && self.bridge.max_aggregate_batches == self.bridge.max_aggregate_bytes
            && self.bridge.max_publication_records == self.bridge.max_publication_content_bytes
            && self.max_inner_records == self.bridge.complete_message_bytes
            && self.bridge.control_message_bytes == 512
            && self.bridge.availability_interval_millis == 5000
            && self.bridge.max_candidate_records <= self.bridge.max_aggregate_records
            && self.bridge.max_candidate_batches <= self.bridge.max_aggregate_batches
            && self.bridge.max_candidate_work <= self.bridge.max_aggregate_work
            && self.bridge.max_aggregate_bytes <= self.bridge.max_total_bytes
            && self.bridge.max_message_inactivity_millis <= self.bridge.max_message_age_millis
    }
}
fn entries(body: &str) -> Option<(ProductionLimits, [Option<usize>; 15])> {
    let mut values = [None; 15];
    let mut bridge = Vec::new();
    for entry in body.split(',') {
        let (name, raw) = entry.split_once(':')?;
        let name = name.trim().strip_prefix('"')?.strip_suffix('"')?;
        let Some(index) = FIELDS.iter().position(|field| *field == name) else {
            bridge.push(entry);
            continue;
        };
        let number = raw.trim();
        if !number.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let value = number.parse::<usize>().ok().filter(|v| *v > 0)?;
        if value.to_string() != number || values[index].replace(value).is_some() {
            return None;
        }
    }
    Some((
        ProductionLimits::parse(&format!("{{{}}}", bridge.join(",")))?,
        values,
    ))
}
