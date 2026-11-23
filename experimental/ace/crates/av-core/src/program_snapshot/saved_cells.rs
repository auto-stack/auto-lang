use std::rc::{Rc, Weak};

use crate::av::GlobalValue;
use crate::contract::{RecordValue, Value};
use crate::mir::{MirModule, MirType};
use crate::owned_text::StrArena;

use super::ProgramSnapshot;

/// Guest values captured from specific verified cells in one live snapshot.
/// This storage carries the values' ownership without admitting raw host data.
#[derive(Clone)]
pub struct SavedGlobalCells {
    owner: Weak<()>,
    cells: Vec<(usize, Option<GlobalValue>)>,
}

impl SavedGlobalCells {
    /// Observe a captured value by its position in the saved cell group.
    pub fn value_at(&self, index: usize) -> Option<&Value> {
        self.cells.get(index)?.1.as_ref().map(|cell| &cell.value)
    }

    /// Charge retained state conservatively, including shared references.
    pub fn charge(&self, remaining: &mut usize) -> bool {
        self.cells.iter().all(|(_, value)| {
            let Some(value) = value else { return true };
            let Some(bytes) = value.value.deep_size_capped(&mut 0, *remaining) else {
                return false;
            };
            *remaining -= bytes;
            true
        })
    }
}

impl ProgramSnapshot {
    /// Copy checked Store fields between the root model and an installed child
    /// model. Both cells belong to this program. Child-owned fields stay local.
    pub fn copy_model_fields(
        &self,
        producer: &MirModule,
        source: u32,
        destination: u32,
        fields: &[(usize, usize)],
    ) -> Result<(), String> {
        let layout = |slot: u32| {
            let global = self.globals.get(slot as usize)?;
            let authored = producer.globals.get(slot as usize)?;
            if !global.mutable || global.name != authored.name || global.ty != authored.ty
                || self.host_inputs.contains(&slot)
            {
                return None;
            }
            match producer.types.get(global.ty as usize) {
                Some(MirType::Record(layout)) => Some(layout),
                _ => None,
            }
        };
        if !self.initialized() {
            return Err("global.shared-store: models are not initialized".into());
        }
        let from_layout = layout(source).ok_or("global.shared-store: invalid source model")?;
        let to_layout = layout(destination).ok_or("global.shared-store: invalid destination model")?;
        let mut cells = self.cells.borrow_mut();
        let record = |slot| match &cells.get(slot as usize)?.as_ref()?.value {
            Value::Record(record) => Some(record),
            _ => None,
        };
        let from = record(source).ok_or("global.shared-store: missing source model")?;
        let to = record(destination).ok_or("global.shared-store: missing destination model")?;
        let mut candidate = to.fields.clone();
        let arena = StrArena::new();
        for &(read, write) in fields {
            let Some((left, right)) = from_layout.fields.get(read).zip(to_layout.fields.get(write)) else {
                return Err("global.shared-store: model field is out of range".into());
            };
            if left.ty != right.ty {
                return Err("global.shared-store: incompatible checked field types".into());
            }
            let value = from.fields.get(read).ok_or("global.shared-store: missing source field")?;
            let slot = candidate.get_mut(write).ok_or("global.shared-store: missing destination field")?;
            *slot = match value {
                Value::List(list) => {
                    let items = list.rc.borrow().elems.clone();
                    Value::List(arena.try_alloc_list(items, 0)
                        .map_err(|_| "global.shared-store: list allocation failed")?)
                }
                value => value.clone(),
            };
        }
        let model = Value::Record(Rc::new(RecordValue { ty: to.ty, fields: candidate }));
        cells[destination as usize].as_mut().expect("checked model").value = model;
        Ok(())
    }

    pub fn save_global_cells(&self, slots: &[u32]) -> Result<SavedGlobalCells, String> {
        let cells = self.cells.borrow();
        let cells = slots
            .iter()
            .map(|&slot| {
                let index = slot as usize;
                if !self.initialized()
                    || !self.globals.get(index).is_some_and(|global| global.mutable)
                    || self.host_inputs.contains(&slot)
                {
                    return Err(format!(
                        "global.saved: cell #{slot} is not a live mutable cell"
                    ));
                }
                Ok((index, cells[index].clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SavedGlobalCells {
            owner: Rc::downgrade(&self.cell_owner),
            cells,
        })
    }

    pub fn install_saved_global_cells(&self, saved: &SavedGlobalCells) -> Result<(), String> {
        if !self.initialized()
            || !saved
                .owner
                .upgrade()
                .is_some_and(|owner| Rc::ptr_eq(&owner, &self.cell_owner))
        {
            return Err("global.saved: saved cells belong to a different snapshot".into());
        }
        let mut cells = self.cells.borrow_mut();
        for (index, value) in &saved.cells {
            cells[*index] = value.clone();
        }
        Ok(())
    }
}
