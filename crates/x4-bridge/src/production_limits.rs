use std::path::Path;

const FIELD_COUNT: usize = 23;
const MAX_CONFIG_BYTES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionLimits {
    pub complete_message_bytes: usize,
    pub control_message_bytes: usize,
    pub max_candidate_raw_bytes: usize,
    pub max_candidate_records: usize,
    pub max_candidate_batches: usize,
    pub max_candidate_work: usize,
    pub max_message_age_millis: usize,
    pub max_message_inactivity_millis: usize,
    pub max_candidates: usize,
    pub max_aggregate_bytes: usize,
    pub max_aggregate_batches: usize,
    pub max_aggregate_records: usize,
    pub max_aggregate_work: usize,
    pub max_publication_records: usize,
    pub max_publication_content_bytes: usize,
    pub max_pending_bytes: usize,
    pub max_total_bytes: usize,
    pub max_lifecycle_work: usize,
    pub max_delivery_attempts: usize,
    pub max_blockers: usize,
    pub reconnect_attempts: usize,
    pub reconnect_delay_millis: usize,
    pub availability_interval_millis: usize,
}

impl ProductionLimits {
    #[must_use]
    pub fn read(path: &Path) -> Option<Self> {
        let contents = std::fs::read_to_string(path).ok()?;
        if contents.is_empty() || contents.len() > MAX_CONFIG_BYTES {
            return None;
        }
        Self::parse(&contents)
    }

    #[must_use]
    pub fn parse(contents: &str) -> Option<Self> {
        let body = contents.trim().strip_prefix('{')?.strip_suffix('}')?;
        let mut values = [None; FIELD_COUNT];
        for entry in body.split(',') {
            let (raw_name, raw_value) = entry.split_once(':')?;
            let name = raw_name.trim().strip_prefix('"')?.strip_suffix('"')?;
            let index = field_index(name)?;
            insert_value(&mut values, index, raw_value)?;
        }
        let value = Self::from_values(&values)?;
        value.relationships_valid().then_some(value)
    }

    fn from_values(v: &[Option<usize>; FIELD_COUNT]) -> Option<Self> {
        Some(Self {
            complete_message_bytes: v[0]?,
            control_message_bytes: v[1]?,
            max_candidate_raw_bytes: v[2]?,
            max_candidate_records: v[3]?,
            max_candidate_batches: v[4]?,
            max_candidate_work: v[5]?,
            max_message_age_millis: v[6]?,
            max_message_inactivity_millis: v[7]?,
            max_candidates: v[8]?,
            max_aggregate_bytes: v[9]?,
            max_aggregate_records: v[10]?,
            max_aggregate_batches: v[11]?,
            max_aggregate_work: v[12]?,
            max_publication_records: v[13]?,
            max_publication_content_bytes: v[14]?,
            max_pending_bytes: v[15]?,
            max_total_bytes: v[16]?,
            max_lifecycle_work: v[17]?,
            max_delivery_attempts: v[18]?,
            max_blockers: v[19]?,
            reconnect_attempts: v[20]?,
            reconnect_delay_millis: v[21]?,
            availability_interval_millis: v[22]?,
        })
    }

    const fn relationships_valid(&self) -> bool {
        self.complete_message_bytes <= self.max_pending_bytes
            && self.max_pending_bytes <= self.max_total_bytes
            && self.max_candidate_raw_bytes <= self.max_aggregate_bytes
            && self.max_publication_records <= self.max_candidate_records
            && self.max_publication_content_bytes <= self.max_total_bytes
            && self.max_candidate_work <= self.max_lifecycle_work
    }
}

fn insert_value(values: &mut [Option<usize>; FIELD_COUNT], index: usize, raw: &str) -> Option<()> {
    let value = raw
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)?;
    values[index].replace(value).is_none().then_some(())
}

fn field_index(name: &str) -> Option<usize> {
    const FIELDS: [&str; FIELD_COUNT] = [
        "complete_message_bytes",
        "control_message_bytes",
        "max_candidate_raw_bytes",
        "max_candidate_records",
        "max_candidate_batches",
        "max_candidate_work",
        "max_message_age_millis",
        "max_message_inactivity_millis",
        "max_candidates",
        "max_aggregate_bytes",
        "max_aggregate_records",
        "max_aggregate_batches",
        "max_aggregate_work",
        "max_publication_records",
        "max_publication_content_bytes",
        "max_pending_bytes",
        "max_total_bytes",
        "max_lifecycle_work",
        "max_delivery_attempts",
        "max_blockers",
        "reconnect_attempts",
        "reconnect_delay_millis",
        "availability_interval_millis",
    ];
    FIELDS.iter().position(|candidate| *candidate == name)
}
