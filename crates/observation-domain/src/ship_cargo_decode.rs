use crate::ship_detail_codec::{dependency, field};
use crate::{
    CargoObservation, CargoStorage, CargoWare, ShipDetailError, detail_number, detail_outcome,
};

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
