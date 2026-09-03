use std::collections::BTreeMap;

use observation_domain::{DecisionSnapshotId, SectionKey, SectionRevisionId};
use observation_ingest::DecisionRevisionSet;
use sha2::{Digest, Sha256};

use crate::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, ObservationRepository,
    PublicationLimits, PublicationReceipt, PublishOutcome, PublishRequest, RepositoryDiagnostic,
    RepositoryError, RevisionRecord, UnpinOutcome,
};

pub struct FakeObservationRepository {
    limits: PublicationLimits,
    revisions: BTreeMap<(SectionKey, SectionRevisionId), RevisionRecord>,
    receipts: BTreeMap<(SectionKey, SectionRevisionId), PublicationReceipt>,
    current: BTreeMap<SectionKey, SectionRevisionId>,
    pins: BTreeMap<DecisionSnapshotId, DecisionRevisionPin>,
    next_publication: u64,
    next_pin: u64,
}

impl FakeObservationRepository {
    #[must_use]
    pub const fn new(limits: PublicationLimits) -> Self {
        Self {
            limits,
            revisions: BTreeMap::new(),
            receipts: BTreeMap::new(),
            current: BTreeMap::new(),
            pins: BTreeMap::new(),
            next_publication: 1,
            next_pin: 1,
        }
    }

    fn normalized(&self, request: &PublishRequest) -> Option<RevisionRecord> {
        let revision = &request.revision;
        if request.expected_current != revision.context().expected_current()
            || request.frozen_dependencies != *revision.context().dependencies()
            || revision.records().len() > self.limits.max_records.get()
        {
            return None;
        }
        let mut digest = Sha256::new();
        let mut bytes = 0usize;
        for record in revision.records() {
            bytes = bytes.checked_add(record.content.len())?;
            hash(&mut digest, record.record_id.as_str().as_bytes());
            hash(&mut digest, record.entity_id.as_str().as_bytes());
            hash(&mut digest, &record.observation_version.get().to_be_bytes());
            hash(&mut digest, record.content.as_bytes());
        }
        let calculated: [u8; 32] = digest.finalize().into();
        if bytes > self.limits.max_content_bytes.get() || calculated != *revision.content_digest() {
            return None;
        }
        Some(RevisionRecord {
            source_scope: revision.source_scope().clone(),
            section_key: revision.section_key().clone(),
            revision: revision.section_revision(),
            records: revision.records().to_vec(),
            manifest_digest: *revision.manifest_digest(),
            content_digest: calculated,
            context_token: format!("{:?}", revision.context()),
        })
    }

    fn replay(
        &self,
        identity: &(SectionKey, SectionRevisionId),
        record: &RevisionRecord,
    ) -> Option<PublishOutcome> {
        let existing = self.revisions.get(identity)?;
        Some(if existing == record {
            PublishOutcome::CommittedReplay(self.receipts[identity].clone())
        } else {
            PublishOutcome::Conflict(diagnostic("content-conflict"))
        })
    }
}

impl ObservationRepository for FakeObservationRepository {
    fn publish(&mut self, request: PublishRequest) -> PublishOutcome {
        let Some(record) = self.normalized(&request) else {
            return PublishOutcome::PermanentRejection(diagnostic("invalid-revision"));
        };
        let identity = (record.section_key.clone(), record.revision);
        if let Some(outcome) = self.replay(&identity, &record) {
            return outcome;
        }
        if self.current.get(&record.section_key).copied() != request.expected_current {
            return PublishOutcome::StalePointer(diagnostic("stale-pointer"));
        }
        if request
            .frozen_dependencies
            .iter()
            .any(|(key, revision)| self.current.get(key) != Some(revision))
        {
            return PublishOutcome::StaleDependency(diagnostic("stale-dependency"));
        }
        let receipt = PublicationReceipt {
            section_key: record.section_key.clone(),
            revision: record.revision,
            content_digest: record.content_digest,
            previous: request.expected_current,
            ordinal: self.next_publication,
        };
        self.next_publication = self.next_publication.saturating_add(1);
        self.current
            .insert(record.section_key.clone(), record.revision);
        self.receipts.insert(identity.clone(), receipt.clone());
        self.revisions.insert(identity, record);
        PublishOutcome::CommittedNew(receipt)
    }

    fn current(&self, key: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError> {
        let Some(revision) = self.current.get(key) else {
            return Ok(None);
        };
        let identity = (key.clone(), *revision);
        Ok(Some(CurrentRevision {
            revision: self.revisions[&identity].clone(),
            receipt: self.receipts[&identity].clone(),
        }))
    }

    fn pin_decision(
        &mut self,
        set: &DecisionRevisionSet,
    ) -> Result<DecisionPinReceipt, RepositoryError> {
        if set
            .revisions()
            .iter()
            .any(|(key, revision)| !self.revisions.contains_key(&(key.clone(), *revision)))
        {
            return Err(RepositoryError::MissingRevision(diagnostic(
                "pin-revision-missing",
            )));
        }
        if let Some(pin) = self
            .pins
            .values()
            .find(|pin| &pin.revisions == set.revisions())
        {
            return Ok(pin.receipt.clone());
        }
        let decision = DecisionSnapshotId::new(format!("decision:{}", self.next_pin))
            .ok_or(RepositoryError::Storage(diagnostic("pin-identity")))?;
        let receipt = DecisionPinReceipt {
            decision: decision.clone(),
            ordinal: self.next_pin,
        };
        self.next_pin = self.next_pin.saturating_add(1);
        self.pins.insert(
            decision,
            DecisionRevisionPin {
                receipt: receipt.clone(),
                revisions: set.revisions().clone(),
            },
        );
        Ok(receipt)
    }

    fn load_decision_pin(
        &self,
        decision: &DecisionSnapshotId,
    ) -> Result<DecisionRevisionPin, RepositoryError> {
        self.pins
            .get(decision)
            .cloned()
            .ok_or(RepositoryError::MissingRevision(diagnostic(
                "pin-not-found",
            )))
    }

    fn unpin_decision(
        &mut self,
        receipt: &DecisionPinReceipt,
    ) -> Result<UnpinOutcome, RepositoryError> {
        let Some(pin) = self.pins.get(&receipt.decision) else {
            return Ok(UnpinOutcome::AlreadyAbsent);
        };
        if pin.receipt != *receipt {
            return Ok(UnpinOutcome::StaleReceipt);
        }
        self.pins.remove(&receipt.decision);
        Ok(UnpinOutcome::Unpinned)
    }
}

fn hash(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

const fn diagnostic(code: &'static str) -> RepositoryDiagnostic {
    RepositoryDiagnostic { code }
}
