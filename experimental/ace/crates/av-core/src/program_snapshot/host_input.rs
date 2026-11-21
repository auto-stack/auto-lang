use std::cell::RefCell;
use std::rc::Rc;

use crate::av::GlobalValue;
use crate::contract::{RecordValue, Value};
use crate::mir::{MirInstr, MirType};
use crate::owned_text::StrArena;
use crate::ExecLimits;

use super::ProgramSnapshot;

/// Data sent by a host binding through a checked input cell. This cannot
/// carry guest references, opaque handles, executable code or raw MIR values.
pub enum HostInput {
    Bool(bool),
    Text(String),
    I64(i64),
    U64(u64),
    Record(Vec<(String, HostInput)>),
    List(Vec<HostInput>),
}

impl ProgramSnapshot {
    /// Install input for one invocation. The destination must have been
    /// admitted at link time and the entry must read it at a checked type.
    pub fn with_host_input<T>(
        &self,
        entry: &str,
        cell: u32,
        input: HostInput,
        limits: &ExecLimits,
        invoke: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        if !self.initialized() || !self.host_inputs.contains(&cell) {
            return Err("global.host-input: no live input cell for this program".into());
        }
        let code = self
            .code
            .iter()
            .find(|code| code.name() == entry)
            .ok_or("global.host-input: entry is not linked")?;
        let unit = code.unit().module();
        let function = &unit.funcs[unit.entry as usize];
        let ty = function
            .blocks
            .iter()
            .flat_map(|block| &block.instrs)
            .find_map(|instr| match instr {
                MirInstr::GlobalGet { dst, global } if *global == cell => {
                    Some(function.slots[*dst as usize].ty)
                }
                _ => None,
            })
            .ok_or("global.host-input: entry does not read its declared input")?;
        let mut remaining = limits.max_slot_bytes;
        let value = materialize(input, ty, &unit.types, &StrArena::new(), &mut remaining, 0)?;
        let original = self.cells.borrow_mut()[cell as usize].replace(GlobalValue::plain(value));
        let _restore = InputScope {
            cells: &self.cells,
            cell: cell as usize,
            original,
        };
        invoke()
    }
}

struct InputScope<'a> {
    cells: &'a RefCell<Vec<Option<GlobalValue>>>,
    cell: usize,
    original: Option<GlobalValue>,
}

impl Drop for InputScope<'_> {
    fn drop(&mut self) {
        self.cells.borrow_mut()[self.cell] = self.original.take();
    }
}

fn materialize(
    input: HostInput,
    ty: u32,
    types: &[MirType],
    arena: &StrArena,
    remaining: &mut usize,
    depth: u8,
) -> Result<Value, String> {
    if depth >= 64 {
        return Err("global.host-input: nesting budget exceeded".into());
    }
    let cell = std::mem::size_of::<Value>();
    charge(remaining, cell)?;
    match (input, types.get(ty as usize)) {
        (HostInput::Bool(value), Some(MirType::Bool)) => Ok(Value::Bool(value)),
        (HostInput::I64(value), Some(MirType::I64)) => Ok(Value::I64(value)),
        (HostInput::U64(value), Some(MirType::U64)) => Ok(Value::U64(value)),
        (HostInput::Text(text), Some(MirType::Str)) => {
            charge(remaining, text.len())?;
            Ok(Value::Str(arena.alloc(text.as_bytes())))
        }
        (HostInput::Record(mut fields), Some(MirType::Record(layout))) => {
            if fields.len() != layout.fields.len() {
                return Err(format!(
                    "global.host-input: incorrect field count for {}",
                    layout.name
                ));
            }
            charge(remaining, std::mem::size_of::<Vec<Value>>())?;
            let mut values = Vec::with_capacity(fields.len());
            for field in &layout.fields {
                let index = fields
                    .iter()
                    .position(|(name, _)| *name == field.name)
                    .ok_or_else(|| format!("global.host-input: missing field {}", field.name))?;
                let input = fields.swap_remove(index).1;
                values.push(materialize(
                    input,
                    field.ty,
                    types,
                    arena,
                    remaining,
                    depth + 1,
                )?);
            }
            Ok(Value::Record(Rc::new(RecordValue { ty, fields: values })))
        }
        (HostInput::List(items), Some(MirType::List(layout))) => {
            charge(remaining, std::mem::size_of::<Vec<Value>>())?;
            if items.len() > u32::MAX as usize || items.len().saturating_mul(cell) > *remaining {
                return Err("global.host-input: size budget exceeded".into());
            }
            let values = items
                .into_iter()
                .map(|input| materialize(input, layout.elem, types, arena, remaining, depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            arena
                .try_alloc_list(values, 0)
                .map(Value::List)
                .map_err(|_| "global.host-input: list allocation failed".into())
        }
        _ => Err("global.host-input: value does not match the checked field type".into()),
    }
}

fn charge(remaining: &mut usize, bytes: usize) -> Result<(), String> {
    *remaining = remaining
        .checked_sub(bytes)
        .ok_or("global.host-input: size budget exceeded")?;
    Ok(())
}
