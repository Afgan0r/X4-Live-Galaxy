use observation_domain::SectionQuality;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireFrame {
    Observation(WireObservation),
    CompleteMarker(WireCompleteMarker),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireObservation {
    pub scope: String,
    pub entity_id: String,
    pub observed_at_unix_millis: u64,
    pub version: u64,
    pub quality: TracerQuality,
    pub content: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireCompleteMarker {
    pub scope: String,
    pub version: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracerObservation {
    pub entity_id: String,
    pub observed_at_unix_millis: u64,
    pub version: u64,
    pub quality: TracerQuality,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracerQuality {
    Fresh,
    KnownEmpty,
    Unknown,
    Partial,
    Stale,
    Unsupported,
}

impl From<TracerQuality> for SectionQuality {
    fn from(value: TracerQuality) -> Self {
        match value {
            TracerQuality::Fresh => Self::Fresh,
            TracerQuality::KnownEmpty => Self::KnownEmpty,
            TracerQuality::Unknown => Self::Unknown,
            TracerQuality::Partial => Self::Partial,
            TracerQuality::Stale => Self::Stale,
            TracerQuality::Unsupported => Self::Unsupported,
        }
    }
}
