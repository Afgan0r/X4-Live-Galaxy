use crate::{
    CaptureWindow, ObservationPolicyVersion, SectionRevisionId, ShipDetailDependency,
    ShipDetailError, ShipIdentity, ShipOwner, SourceEvidenceRef, detail_number,
};

pub fn field<'a>(
    lines: &mut std::str::Lines<'a>,
    prefix: &str,
) -> Result<&'a str, ShipDetailError> {
    lines
        .next()
        .and_then(|v| v.strip_prefix(prefix))
        .ok_or(ShipDetailError::InvalidField)
}
pub fn dependency(
    lines: &mut std::str::Lines<'_>,
) -> Result<ShipDetailDependency, ShipDetailError> {
    Ok(ShipDetailDependency {
        identity: ShipIdentity::new(field(lines, "identity=")?)
            .map_err(|_| ShipDetailError::InvalidField)?,
        owner: ShipOwner::new(field(lines, "owner=")?)
            .map_err(|_| ShipDetailError::InvalidField)?,
        core_revision: SectionRevisionId::new(detail_number(field(lines, "core_revision=")?)?)
            .ok_or(ShipDetailError::InvalidNumber)?,
        member_revision: SectionRevisionId::new(detail_number(field(lines, "member_revision=")?)?)
            .ok_or(ShipDetailError::InvalidNumber)?,
        policy: ObservationPolicyVersion::new(detail_number(field(lines, "policy=")?)?)
            .ok_or(ShipDetailError::InvalidNumber)?,
        capture: CaptureWindow::new(
            detail_number(field(lines, "capture_start=")?)?,
            detail_number(field(lines, "capture_end=")?)?,
        )
        .ok_or(ShipDetailError::InvalidNumber)?,
        source: SourceEvidenceRef::new(field(lines, "source=")?)
            .ok_or(ShipDetailError::InvalidField)?,
        consistency: crate::ShipRecordConsistency::from_fields(
            field(lines, "consistency=")?,
            field(lines, "consistency_reason=")?,
        )?,
    })
}
pub fn header(profile: &str, d: &ShipDetailDependency) -> String {
    format!(
        "profile={profile}\nidentity={}\nowner={}\ncore_revision={}\nmember_revision={}\npolicy={}\ncapture_start={}\ncapture_end={}\nsource={}\nconsistency={}\nconsistency_reason={}",
        d.identity.as_str(),
        d.owner.as_str(),
        d.core_revision.get(),
        d.member_revision.get(),
        d.policy.get(),
        d.capture.start_millis(),
        d.capture.end_millis(),
        d.source.as_str(),
        d.consistency.fields().0,
        d.consistency.fields().1
    )
}
