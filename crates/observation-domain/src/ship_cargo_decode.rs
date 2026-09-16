use crate::{
    CaptureWindow, CargoObservation, CargoStorage, CargoWare, ObservationPolicyVersion,
    SectionRevisionId, ShipDetailDependency, ShipDetailError, ShipIdentity, ShipOwner,
    SourceEvidenceRef, detail_number, detail_outcome,
};

fn field<'a>(lines: &mut std::str::Lines<'a>, prefix: &str) -> Result<&'a str, ShipDetailError> {
    lines
        .next()
        .and_then(|v| v.strip_prefix(prefix))
        .ok_or(ShipDetailError::InvalidField)
}
fn dependency(lines: &mut std::str::Lines<'_>) -> Result<ShipDetailDependency, ShipDetailError> {
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
    })
}
fn rows(
    lines: std::str::Lines<'_>,
    maximum: usize,
) -> Result<(Vec<CargoWare>, Vec<CargoStorage>), ShipDetailError> {
    let mut wares = Vec::new();
    let mut storage = Vec::new();
    for line in lines {
        if let Some(value) = line.strip_prefix("ware=") {
            bound(!storage.is_empty() || wares.len() >= maximum)?;
            let (ware, amount) = value.split_once('|').ok_or(ShipDetailError::InvalidField)?;
            wares.push(CargoWare {
                ware: ware.to_owned(),
                amount_items: detail_number(amount)?,
            });
            continue;
        }
        let value = line
            .strip_prefix("storage=")
            .ok_or(ShipDetailError::InvalidField)?;
        if storage.len() >= maximum {
            return Err(ShipDetailError::ExceededBound);
        }
        let (transport, rest) = value.split_once('|').ok_or(ShipDetailError::InvalidField)?;
        let (capacity, occupied) = rest.split_once('|').ok_or(ShipDetailError::InvalidField)?;
        storage.push(CargoStorage {
            transport: transport.to_owned(),
            capacity_cubic_metres: u32::try_from(detail_number(capacity)?)
                .map_err(|_| ShipDetailError::InvalidNumber)?,
            occupied_cubic_metres: u32::try_from(detail_number(occupied)?)
                .map_err(|_| ShipDetailError::InvalidNumber)?,
        });
    }
    Ok((wares, storage))
}
const fn bound(exceeded: bool) -> Result<(), ShipDetailError> {
    if exceeded {
        return Err(ShipDetailError::ExceededBound);
    }
    Ok(())
}
impl CargoObservation {
    pub fn from_content(content: &str, max_inner: usize) -> Result<Self, ShipDetailError> {
        let mut lines = content.lines();
        if lines.next() != Some("profile=ship_cargo") {
            return Err(ShipDetailError::InvalidField);
        }
        let dependency = dependency(&mut lines)?;
        let wares_outcome = field(&mut lines, "wares_outcome=")?;
        let storage_outcome = field(&mut lines, "storage_outcome=")?;
        if lines.next() != Some("reservation_policy=excluded") {
            return Err(ShipDetailError::InvalidField);
        }
        let (wares, storage) = rows(lines, max_inner)?;
        let record = Self {
            dependency,
            wares: detail_outcome(wares_outcome, wares)?,
            storage: detail_outcome(storage_outcome, storage)?,
        };
        record.validate(max_inner)?;
        if record.canonical_content() != content {
            return Err(ShipDetailError::InvalidField);
        }
        Ok(record)
    }
}
