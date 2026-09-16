use super::{Result, publication_limits};
use observation_domain::{CompletionCoverage, SectionCoverage, SectionKey, SectionRevisionId};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use std::io::Write as _;
use std::path::Path;

pub fn verify(database: &Path, revisions: &[u64], completion: &[u8]) -> Result<()> {
    let repository = SqliteObservationRepository::open(database, publication_limits()?)
        .map_err(|e| format!("independent reopen:{e:?}"))?;
    let key = SectionKey::new("ship_core").ok_or("key")?;
    let current = repository
        .current(&key)
        .map_err(|e| format!("current:{e:?}"))?
        .ok_or("missing current")?;
    if Some(&current.receipt().revision.get()) != revisions.last() {
        return Err("prior current changed".into());
    }
    let observation_domain::CompleteMessage::SectionCompletion(certificate) =
        observation_ingest::decode_complete_message(completion, 4096)
            .map_err(|e| format!("native completion:{e:?}"))?
    else {
        return Err("expected native completion".into());
    };
    if current.receipt().content_digest != certificate.canonical_content_digest {
        return Err("native certificate differs from reopened durable digest".into());
    }
    for number in revisions {
        let (revision, receipt) = repository
            .stored_revision(&key, SectionRevisionId::new(*number).ok_or("revision")?)
            .map_err(|e| format!("history:{e:?}"))?
            .ok_or("missing history")?;
        if receipt.ordinal != *number
            || receipt.revision.get() != *number
            || receipt.content_digest != revision.content_digest
            || receipt.previous.map(SectionRevisionId::get)
                != number.checked_sub(1).filter(|v| *v > 0)
            || revision.coverage != CompletionCoverage::Partial
            || revision.records.len() != 8
            || revision
                .context
                .candidate(revision.dependencies.clone(), revision.expected_current)
                .state()
                .coverage()
                != SectionCoverage::Partial
        {
            return Err("history receipt/coverage mismatch".into());
        }
        verify_records(&revision, *number)?;
    }
    let next = revisions.last().copied().ok_or("empty history")? + 1;
    if repository
        .stored_revision(&key, SectionRevisionId::new(next).ok_or("next revision")?)
        .map_err(|e| format!("rejection readback:{e:?}"))?
        .is_some()
    {
        return Err("replay or malformed replacement added revision".into());
    }
    writeln!(
        std::io::stdout(),
        "INDEPENDENT_READBACK revisions={revisions:?} current={} weak_source_coverage=partial",
        current.receipt().revision.get()
    )?;
    Ok(())
}

fn verify_records(revision: &observation_persistence::RevisionRecord, number: u64) -> Result<()> {
    for (index, record) in revision.records.iter().enumerate() {
        let id = format!("900719925474099{}", index + 2);
        let location = if number == 2 { 2 } else { 1 };
        let expected = format!(
            "profile=ship_core\nidentity={id}\nowner=argon\ntype=destroyer_macro\nclass=destroyer\nlocation=sector:{location}"
        );
        if record.content != expected
            || record.entity_id.as_str() != format!("x4:ship:{id}")
            || record.record_id.as_str() != format!("carrier-b:{number}:{:020}", index + 1)
            || record.observation_version.get() != number
        {
            return Err("ordered core readback mismatch".into());
        }
    }
    Ok(())
}
