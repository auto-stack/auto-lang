use std::rc::{Rc, Weak};

use crate::av::GlobalValue;
use crate::contract::Value;

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
