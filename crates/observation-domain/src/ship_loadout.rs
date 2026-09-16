use crate::ship_detail::outcome_name;
use crate::{FieldOutcome, ShipDetailDependency, ShipDetailError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledSlot {
    pub kind: String,
    pub slot: u32,
    pub component: String,
    pub macro_name: String,
    pub path: String,
    pub group: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualSlot {
    pub kind: String,
    pub slot: u32,
    pub macro_name: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledSoftware {
    pub maximum: String,
    pub current: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissileCargo {
    pub ware: String,
    pub macro_name: String,
    pub amount_raw: i32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipUnit {
    pub macro_name: String,
    pub category: String,
    pub amount_items: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadoutObservation {
    pub dependency: ShipDetailDependency,
    pub physical: FieldOutcome<Vec<InstalledSlot>>,
    pub virtual_slots: FieldOutcome<Vec<VirtualSlot>>,
    pub software: FieldOutcome<Vec<InstalledSoftware>>,
    pub missiles: FieldOutcome<Vec<MissileCargo>>,
    pub units: FieldOutcome<Vec<ShipUnit>>,
}
fn text(value: &str) -> bool {
    value.len() <= 128 && !value.contains(['|', '\n', '\r'])
}
const fn bounded<T>(outcome: &FieldOutcome<Vec<T>>, maximum: usize) -> Result<(), ShipDetailError> {
    if let FieldOutcome::Value(rows) = outcome
        && (rows.is_empty() || rows.len() > maximum)
    {
        return Err(ShipDetailError::ExceededBound);
    }
    Ok(())
}
impl LoadoutObservation {
    pub fn validate(&self, maximum: usize) -> Result<(), ShipDetailError> {
        self.dependency.validate()?;
        bounded(&self.physical, maximum)?;
        bounded(&self.virtual_slots, maximum)?;
        bounded(&self.software, maximum)?;
        bounded(&self.missiles, maximum)?;
        bounded(&self.units, maximum)?;
        if let FieldOutcome::Value(rows) = &self.physical {
            physical(rows)?;
        }
        if let FieldOutcome::Value(rows) = &self.virtual_slots
            && rows
                .iter()
                .any(|r| r.slot == 0 || !kind(&r.kind) || !text(&r.macro_name))
        {
            return Err(ShipDetailError::InvalidField);
        }
        if let FieldOutcome::Value(rows) = &self.software
            && rows.iter().any(|r| !text(&r.maximum) || !text(&r.current))
        {
            return Err(ShipDetailError::InvalidField);
        }
        if let FieldOutcome::Value(rows) = &self.missiles
            && rows.iter().any(|r| !text(&r.ware) || !text(&r.macro_name))
        {
            return Err(ShipDetailError::InvalidField);
        }
        if let FieldOutcome::Value(rows) = &self.units
            && rows
                .iter()
                .any(|r| !text(&r.category) || !text(&r.macro_name))
        {
            return Err(ShipDetailError::InvalidField);
        }
        Ok(())
    }
    #[must_use]
    pub fn canonical_content(&self) -> String {
        use core::fmt::Write as _;
        let mut result = crate::ship_detail_codec::header("ship_loadout", &self.dependency);
        let _ = write!(
            result,
            "\nphysical_outcome={}\nvirtual_outcome={}\nsoftware_outcome={}\nmissiles_outcome={}\nunits_outcome={}\nmissile_semantics=raw_signed_inferred_items\nunits_selector=false\nvirtual_semantics=observed_macro_thruster_inferred",
            outcome_name(&self.physical),
            outcome_name(&self.virtual_slots),
            outcome_name(&self.software),
            outcome_name(&self.missiles),
            outcome_name(&self.units)
        );
        append_rows(&mut result, self);
        result
    }
}
fn physical(rows: &[InstalledSlot]) -> Result<(), ShipDetailError> {
    for row in rows {
        let id = row
            .component
            .parse::<u64>()
            .map_err(|_| ShipDetailError::InvalidNumber)?;
        if id.to_string() != row.component
            || row.slot == 0
            || !kind(&row.kind)
            || !text(&row.macro_name)
            || !text(&row.path)
            || !text(&row.group)
        {
            return Err(ShipDetailError::InvalidField);
        }
    }
    if rows
        .windows(2)
        .any(|r| (&r[0].kind, r[0].slot) >= (&r[1].kind, r[1].slot))
    {
        return Err(ShipDetailError::InvalidField);
    }
    Ok(())
}
fn append_rows(text: &mut String, value: &LoadoutObservation) {
    use core::fmt::Write as _;
    for r in values(&value.physical) {
        let _ = write!(
            text,
            "\nphysical={}|{}|{}|{}|{}|{}",
            r.kind, r.slot, r.component, r.macro_name, r.path, r.group
        );
    }
    for r in values(&value.virtual_slots) {
        let _ = write!(text, "\nvirtual={}|{}|{}", r.kind, r.slot, r.macro_name);
    }
    for r in values(&value.software) {
        let _ = write!(text, "\nsoftware={}|{}", r.maximum, r.current);
    }
    for r in values(&value.missiles) {
        let _ = write!(
            text,
            "\nmissile={}|{}|{}",
            r.ware, r.macro_name, r.amount_raw
        );
    }
    for r in values(&value.units) {
        let _ = write!(
            text,
            "\nunit={}|{}|{}",
            r.macro_name, r.category, r.amount_items
        );
    }
}
fn values<T>(outcome: &FieldOutcome<Vec<T>>) -> &[T] {
    if let FieldOutcome::Value(rows) = outcome {
        rows
    } else {
        &[]
    }
}
fn kind(v: &str) -> bool {
    matches!(v, "engine" | "shield" | "weapon" | "turret" | "thruster")
}
