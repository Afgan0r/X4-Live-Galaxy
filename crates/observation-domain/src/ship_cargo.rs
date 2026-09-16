use crate::ship_detail::{detail_number, detail_token, outcome_name};
use crate::{FieldOutcome, ShipDetailDependency, ShipDetailError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoWare {
    pub ware: String,
    pub amount_items: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoStorage {
    pub transport: String,
    pub capacity_cubic_metres: u32,
    pub occupied_cubic_metres: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoObservation {
    pub dependency: ShipDetailDependency,
    pub wares: FieldOutcome<Vec<CargoWare>>,
    pub storage: FieldOutcome<Vec<CargoStorage>>,
}

impl CargoObservation {
    pub fn validate(&self, max_inner: usize) -> Result<(), ShipDetailError> {
        self.dependency.validate()?;
        if !detail_token(self.dependency.owner.as_str()) {
            return Err(ShipDetailError::InvalidField);
        }
        validate_wares(&self.wares, max_inner)?;
        validate_storage(&self.storage, max_inner)?;
        Ok(())
    }

    #[must_use]
    pub fn canonical_content(&self) -> String {
        let d = &self.dependency;
        let mut text = format!(
            "profile=ship_cargo\nidentity={}\nowner={}\ncore_revision={}\nmember_revision={}\npolicy={}\ncapture_start={}\ncapture_end={}\nsource={}\nwares_outcome={}\nstorage_outcome={}\nreservation_policy=excluded",
            d.identity.as_str(),
            d.owner.as_str(),
            d.core_revision.get(),
            d.member_revision.get(),
            d.policy.get(),
            d.capture.start_millis(),
            d.capture.end_millis(),
            d.source.as_str(),
            outcome_name(&self.wares),
            outcome_name(&self.storage)
        );
        append_wares(&mut text, &self.wares);
        append_storage(&mut text, &self.storage);
        text
    }
}

fn validate_wares(
    outcome: &FieldOutcome<Vec<CargoWare>>,
    maximum: usize,
) -> Result<(), ShipDetailError> {
    let FieldOutcome::Value(values) = outcome else {
        return Ok(());
    };
    if values.is_empty() || values.len() > maximum {
        return Err(ShipDetailError::ExceededBound);
    }
    if values.windows(2).any(|v| v[0].ware >= v[1].ware) {
        return Err(ShipDetailError::InvalidField);
    }
    for value in values {
        if !detail_token(&value.ware) {
            return Err(ShipDetailError::InvalidField);
        }
        detail_number(&value.amount_items.to_string())?;
    }
    Ok(())
}
fn validate_storage(
    outcome: &FieldOutcome<Vec<CargoStorage>>,
    maximum: usize,
) -> Result<(), ShipDetailError> {
    let FieldOutcome::Value(values) = outcome else {
        return Ok(());
    };
    if values.is_empty() || values.len() > maximum {
        return Err(ShipDetailError::ExceededBound);
    }
    if values.windows(2).any(|v| v[0].transport >= v[1].transport)
        || values.iter().any(|v| {
            !detail_token(&v.transport) || v.occupied_cubic_metres > v.capacity_cubic_metres
        })
    {
        return Err(ShipDetailError::InvalidField);
    }
    Ok(())
}
fn append_wares(text: &mut String, outcome: &FieldOutcome<Vec<CargoWare>>) {
    use core::fmt::Write as _;
    let FieldOutcome::Value(values) = outcome else {
        return;
    };
    for v in values {
        let _ = write!(text, "\nware={}|{}", v.ware, v.amount_items);
    }
}
fn append_storage(text: &mut String, outcome: &FieldOutcome<Vec<CargoStorage>>) {
    use core::fmt::Write as _;
    let FieldOutcome::Value(values) = outcome else {
        return;
    };
    for v in values {
        let _ = write!(
            text,
            "\nstorage={}|{}|{}",
            v.transport, v.capacity_cubic_metres, v.occupied_cubic_metres
        );
    }
}
