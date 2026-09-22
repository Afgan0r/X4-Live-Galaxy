use crate::producer_types::{PreparedRecord, ProducerProfile};
use crate::{Producer, ProducerError, SectionEvidence};

impl Producer {
    pub fn begin_faction_census(
        &mut self,
        evidence: SectionEvidence,
        expected_records: usize,
    ) -> Result<(), ProducerError> {
        if self.profile != ProducerProfile::FactionCensus
            || expected_records > self.limits.max_records
        {
            return Err(ProducerError::InvalidInput);
        }
        self.begin(evidence, expected_records)
    }

    pub fn push_faction_census(
        &mut self,
        record: &crate::FactionCensusRecord,
    ) -> Result<(), ProducerError> {
        if self.profile != ProducerProfile::FactionCensus
            || record.discovery_revision == 0
            || !matches!(
                record.disposition.as_str(),
                "included" | "excluded" | "unknown"
            )
        {
            return Err(ProducerError::InvalidInput);
        }
        let content = format!(
            "discovery_revision={}\ndisposition={}\nreason={}\norigin={}\nsource_evidence={}\nmind_candidate={}",
            record.discovery_revision,
            record.disposition,
            record.reason,
            record.origin,
            record.source_evidence,
            record.mind_candidate
        );
        if content.len() > self.limits.max_raw_bytes {
            return Err(ProducerError::DataLimit);
        }
        self.push(PreparedRecord {
            entity_id: format!("x4:faction:{}", record.faction_id),
            content,
        })
    }
}
