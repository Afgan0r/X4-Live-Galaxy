use crate::ship_detail_codec::{dependency, field};
use crate::{
    InstalledSlot, InstalledSoftware, LoadoutObservation, MissileCargo, ShipDetailError, ShipUnit,
    VirtualSlot, detail_number, detail_outcome,
};
impl LoadoutObservation {
    pub fn from_content(content: &str, maximum: usize) -> Result<Self, ShipDetailError> {
        let mut lines = content.lines();
        if lines.next() != Some("profile=ship_loadout") {
            return Err(ShipDetailError::InvalidField);
        }
        let dependency = dependency(&mut lines)?;
        let names = [
            field(&mut lines, "physical_outcome=")?,
            field(&mut lines, "virtual_outcome=")?,
            field(&mut lines, "software_outcome=")?,
            field(&mut lines, "missiles_outcome=")?,
            field(&mut lines, "units_outcome=")?,
        ];
        if lines.next() != Some("missile_semantics=raw_signed_inferred_items")
            || lines.next() != Some("units_selector=false")
            || lines.next() != Some("virtual_semantics=observed_macro_thruster_inferred")
        {
            return Err(ShipDetailError::InvalidField);
        }
        let mut rows = Rows::default();
        for line in lines {
            rows.push(line, maximum)?;
        }
        let result = Self {
            dependency,
            physical: detail_outcome(names[0], rows.physical)?,
            virtual_slots: detail_outcome(names[1], rows.virtual_slots)?,
            software: detail_outcome(names[2], rows.software)?,
            missiles: detail_outcome(names[3], rows.missiles)?,
            units: detail_outcome(names[4], rows.units)?,
        };
        result.validate(maximum)?;
        if result.canonical_content() != content {
            return Err(ShipDetailError::InvalidField);
        }
        Ok(result)
    }
}
#[derive(Default)]
struct Rows {
    physical: Vec<InstalledSlot>,
    virtual_slots: Vec<VirtualSlot>,
    software: Vec<InstalledSoftware>,
    missiles: Vec<MissileCargo>,
    units: Vec<ShipUnit>,
}
impl Rows {
    fn push(&mut self, line: &str, maximum: usize) -> Result<(), ShipDetailError> {
        let (key, value) = line.split_once('=').ok_or(ShipDetailError::InvalidField)?;
        let p: Vec<_> = value.split('|').collect();
        match (key, p.as_slice()) {
            ("physical", [kind, slot, component, macro_name, path, group]) => {
                limit(self.physical.len(), maximum)?;
                self.physical.push(InstalledSlot {
                    kind: (*kind).into(),
                    slot: unsigned(slot)?,
                    component: (*component).into(),
                    macro_name: (*macro_name).into(),
                    path: (*path).into(),
                    group: (*group).into(),
                });
            }
            ("virtual", [kind, slot, macro_name]) => {
                limit(self.virtual_slots.len(), maximum)?;
                self.virtual_slots.push(VirtualSlot {
                    kind: (*kind).into(),
                    slot: unsigned(slot)?,
                    macro_name: (*macro_name).into(),
                });
            }
            ("software", [max, current]) => {
                limit(self.software.len(), maximum)?;
                self.software.push(InstalledSoftware {
                    maximum: (*max).into(),
                    current: (*current).into(),
                });
            }
            ("missile", [ware, macro_name, amount]) => {
                limit(self.missiles.len(), maximum)?;
                self.missiles.push(MissileCargo {
                    ware: (*ware).into(),
                    macro_name: (*macro_name).into(),
                    amount_raw: signed(amount)?,
                });
            }
            ("unit", [macro_name, category, amount]) => {
                limit(self.units.len(), maximum)?;
                self.units.push(ShipUnit {
                    macro_name: (*macro_name).into(),
                    category: (*category).into(),
                    amount_items: unsigned(amount)?,
                });
            }
            _ => return Err(ShipDetailError::InvalidField),
        }
        Ok(())
    }
}
fn signed(value: &str) -> Result<i32, ShipDetailError> {
    let number = value
        .parse::<i32>()
        .map_err(|_| ShipDetailError::InvalidNumber)?;
    if number.to_string() != value {
        return Err(ShipDetailError::InvalidNumber);
    }
    Ok(number)
}
fn unsigned(v: &str) -> Result<u32, ShipDetailError> {
    u32::try_from(detail_number(v)?).map_err(|_| ShipDetailError::InvalidNumber)
}
const fn limit(size: usize, maximum: usize) -> Result<(), ShipDetailError> {
    if size >= maximum {
        Err(ShipDetailError::ExceededBound)
    } else {
        Ok(())
    }
}
