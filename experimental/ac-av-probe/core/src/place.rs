//! Place/borrow execution model (PLAN-821, Design44 §5–§6).
//!
//! 774 SNAPSHOT semantics (source-text.md: "旧构造的 view 读作陈旧"):
//! a `StrView` SHARES the buffer it was minted over — a later replace of
//! the owning place leaves the view reading the OLD bytes (the share keeps
//! them alive). Views never root at a mutable place, hold no drop right,
//! and reading one never fails for an admitted program. Conflict law
//! (whole-root replace/move vs live borrows, 782 root-conservative) lives
//! in the STATIC analysis (`ownership.rs`); MutRef token rooting arrives
//! with T-03.

use crate::contract::{StrHandle, StrView, StrViewSrc, Value};

/// The resolved backing of a view: a shared (never-owning) handle plus
/// the scalar range.
pub struct ResolvedView {
    pub handle: StrHandle,
    pub start: u32,
    pub end: u32,
}

/// Resolve a SHARE-backed view (static views resolve through the const
/// table in the executor and never come through here).
pub fn resolve_share_view(view: &StrView) -> Option<ResolvedView> {
    match &view.src {
        StrViewSrc::Share(h) => Some(ResolvedView {
            handle: h.clone(),
            start: view.start,
            end: view.end,
        }),
        StrViewSrc::Static { .. } => None,
    }
}

/// Project a field path out of a value (shape walk shared by FIELD_GET's
/// Str-view creation). `None` = shape mismatch.
pub fn field_project<'v>(base: &'v Value, path: &[u16]) -> Option<&'v Value> {
    let mut cur = base;
    for seg in path {
        let r = match cur {
            Value::Record(r) => r,
            _ => return None,
        };
        cur = r.fields.get(*seg as usize)?;
    }
    Some(cur)
}

/// Structural field rebuild along `path` (the immutable-replace write of
/// 820, reused verbatim by the owned profile's FIELD_SET — the ownership
/// transaction around it lives in the executor).
pub fn record_set_path(root: &Value, path: &[u16], val: Value) -> Option<Value> {
    let r = match root {
        Value::Record(r) => r,
        _ => return None,
    };
    let seg = *path.first()? as usize;
    let mut fields = r.fields.clone();
    if path.len() == 1 {
        if seg >= fields.len() {
            return None;
        }
        fields[seg] = val;
    } else {
        let inner = fields.get(seg)?.clone();
        fields[seg] = record_set_path(&inner, &path[1..], val)?;
    }
    Some(Value::Record(std::rc::Rc::new(
        crate::contract::RecordValue { ty: r.ty, fields },
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned_str(bytes: &[u8]) -> Value {
        Value::Str(StrHandle::detached(bytes.to_vec().into_boxed_slice()))
    }

    #[test]
    fn share_view_reads_stale_after_replace() {
        // 774 snapshot semantics: minting a view shares the buffer; a
        // later replace of the owning place leaves the view readable with
        // the OLD bytes.
        let v = owned_str(b"auto");
        let share = match &v {
            Value::Str(h) => StrHandle::clone(h),
            _ => unreachable!(),
        };
        let view = StrView {
            src: StrViewSrc::Share(share),
            start: 0,
            end: 4,
        };
        drop(v);
        let resolved = resolve_share_view(&view).expect("share stays readable");
        assert_eq!(resolved.handle.bytes(), b"auto");
    }

    #[test]
    fn record_path_projects_and_rebuilds() {
        let rec = Value::Record(std::rc::Rc::new(crate::contract::RecordValue {
            ty: 0,
            fields: vec![owned_str(b"auto"), Value::I32(1)],
        }));
        let got = field_project(&rec, &[0]).cloned().unwrap();
        assert!(matches!(got, Value::Str(_)));
        let rebuilt = record_set_path(&rec, &[0], owned_str(b"lang")).unwrap();
        let new_str = field_project(&rebuilt, &[0]).cloned().unwrap();
        match new_str {
            Value::Str(h) => assert_eq!(h.bytes(), b"lang"),
            other => panic!("unexpected {other:?}"),
        }
        // Untouched sibling survives the rebuild.
        assert!(matches!(field_project(&rebuilt, &[1]), Some(Value::I32(1))));
    }
}
