//! Structured pipeline operators — shell-level DSL for filtering, sorting,
//! selecting, and transforming structured data (Plan 320).
//!
//! These are NOT commands (they don't go through the command registry). The
//! pipeline parser detects DSL stages (`.field op value`, `sort .field`, etc.)
//! and translates them into [`PipelineOp`] variants. The executor calls
//! [`apply`] to transform `Value::Array` between pipe stages.

use auto_val::{Array, Obj, Value};

/// A shell-DSL pipeline operation (non-command pipe stage).
#[derive(Debug, Clone)]
pub enum PipelineOp {
    /// `.field op value` → filter rows where the comparison holds.
    Filter {
        field: String,
        op: CmpOp,
        value: Value,
    },
    /// `sort .field [desc]`
    SortBy {
        field: String,
        descending: bool,
    },
    /// `select .f1 .f2 ...` → keep only listed fields per object.
    Select {
        fields: Vec<String>,
    },
    /// `.field` → project to a list of single values.
    Map {
        field: String,
    },
    /// `first N`
    Take(usize),
    /// `last N`
    SkipBack(usize),
    /// `count`
    Count,
}

/// Comparison operators for predicates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmpOp {
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Neq,
    Contains,
    StartsWith,
    EndsWith,
}

impl CmpOp {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            ">" => Some(Self::Gt),
            "<" => Some(Self::Lt),
            ">=" => Some(Self::Ge),
            "<=" => Some(Self::Le),
            "==" => Some(Self::Eq),
            "!=" => Some(Self::Neq),
            "contains" => Some(Self::Contains),
            "starts-with" => Some(Self::StartsWith),
            "ends-with" => Some(Self::EndsWith),
            _ => None,
        }
    }
}

/// Apply a pipeline operation to a `Value::Array`, returning the transformed value.
pub fn apply(op: &PipelineOp, data: &Value) -> Value {
    let arr = match data {
        Value::Array(a) => a,
        _ => return data.clone(), // Non-array passthrough (no-op).
    };

    match op {
        PipelineOp::Filter { field, op, value } => {
            let filtered: Vec<Value> = arr
                .iter()
                .filter(|item| {
                    let field_val = get_field(item, field);
                    compare(&field_val, *op, value)
                })
                .cloned()
                .collect();
            Value::Array(Array::from_vec(filtered))
        }
        PipelineOp::SortBy { field, descending } => {
            let mut items: Vec<Value> = arr.iter().cloned().collect();
            items.sort_by(|a, b| {
                let va = get_field(a, field);
                let vb = get_field(b, field);
                let ord = compare_order(&va, &vb);
                if *descending {
                    ord.reverse()
                } else {
                    ord
                }
            });
            Value::Array(Array::from_vec(items))
        }
        PipelineOp::Select { fields } => {
            let selected: Vec<Value> = arr
                .iter()
                .map(|item| {
                    if let Value::Obj(obj) = item {
                        let mut out = Obj::new();
                        for f in fields {
                            if let Some(v) = obj.get(f.as_str()) {
                                out.set(f.as_str(), v);
                            }
                        }
                        Value::Obj(out)
                    } else {
                        item.clone()
                    }
                })
                .collect();
            Value::Array(Array::from_vec(selected))
        }
        PipelineOp::Map { field } => {
            let mapped: Vec<Value> = arr
                .iter()
                .map(|item| get_field(item, field))
                .collect();
            Value::Array(Array::from_vec(mapped))
        }
        PipelineOp::Take(n) => {
            let taken: Vec<Value> = arr.iter().take(*n).cloned().collect();
            Value::Array(Array::from_vec(taken))
        }
        PipelineOp::SkipBack(n) => {
            let len = arr.len();
            let skip = len.saturating_sub(*n);
            let taken: Vec<Value> = arr.iter().skip(skip).cloned().collect();
            Value::Array(Array::from_vec(taken))
        }
        PipelineOp::Count => Value::USize(arr.len()),
    }
}

/// Get a field value from a Value (if it's an Obj), or return Nil.
fn get_field(item: &Value, field: &str) -> Value {
    if let Value::Obj(obj) = item {
        obj.get(field).unwrap_or(Value::Nil)
    } else {
        Value::Nil
    }
}

/// Compare two Values using a CmpOp. Returns true if the comparison holds.
fn compare(a: &Value, op: CmpOp, b: &Value) -> bool {
    // Try numeric comparison first.
    if let (Some(na), Some(nb)) = (as_f64(a), as_f64(b)) {
        return match op {
            CmpOp::Gt => na > nb,
            CmpOp::Lt => na < nb,
            CmpOp::Ge => na >= nb,
            CmpOp::Le => na <= nb,
            CmpOp::Eq => (na - nb).abs() < f64::EPSILON,
            CmpOp::Neq => (na - nb).abs() >= f64::EPSILON,
            _ => false, // Contains/StartsWith/EndsWith don't apply to numbers.
        };
    }
    // String comparison.
    let sa = as_string(a);
    let sb = as_string(b);
    match op {
        CmpOp::Eq => sa == sb,
        CmpOp::Neq => sa != sb,
        CmpOp::Contains => sa.contains(&sb),
        CmpOp::StartsWith => sa.starts_with(&sb),
        CmpOp::EndsWith => sa.ends_with(&sb),
        _ => false,
    }
}

/// Ordering for sort: compare two Values (numeric then string).
fn compare_order(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    if let (Some(na), Some(nb)) = (as_f64(a), as_f64(b)) {
        return na.partial_cmp(&nb).unwrap_or(Ordering::Equal);
    }
    as_string(a).cmp(&as_string(b))
}

/// Extract a numeric value from a Value (Int/I64/Float/Uint/USize/etc.).
fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Int(i) => Some(*i as f64),
        Value::I64(i) => Some(*i as f64),
        Value::Uint(u) => Some(*u as f64),
        Value::USize(u) => Some(*u as f64),
        Value::Float(f) => Some(*f),
        Value::Double(f) => Some(*f),
        Value::Byte(b) => Some(*b as f64),
        Value::I8(i) => Some(*i as f64),
        Value::U8(u) => Some(*u as f64),
        _ => None,
    }
}

/// Extract a string from a Value.
fn as_string(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::String(s) => s.to_string(),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_obj(name: &str, ty: &str, size: i64) -> Value {
        let mut o = Obj::new();
        o.set("name", Value::str(name));
        o.set("type", Value::str(ty));
        o.set("size", Value::I64(size));
        Value::Obj(o)
    }

    fn sample_list() -> Value {
        Value::Array(Array::from_vec(vec![
            file_obj("app", "dir", 0),
            file_obj("big.tar", "file", 20_000_000),
            file_obj("small.txt", "file", 500),
            file_obj("docs", "dir", 0),
        ]))
    }

    #[test]
    fn filter_size_gt() {
        let data = sample_list();
        let op = PipelineOp::Filter {
            field: "size".into(),
            op: CmpOp::Gt,
            value: Value::I64(1000),
        };
        let result = apply(&op, &data);
        if let Value::Array(a) = result {
            assert_eq!(a.len(), 1);
            assert!(a.get(0).unwrap().to_string().contains("big.tar"));
        } else {
            panic!("expected array");
        }
    }

    #[test]
    fn filter_type_eq_string() {
        let data = sample_list();
        let op = PipelineOp::Filter {
            field: "type".into(),
            op: CmpOp::Eq,
            value: Value::str("dir"),
        };
        let result = apply(&op, &data);
        if let Value::Array(a) = result {
            assert_eq!(a.len(), 2); // app, docs
        }
    }

    #[test]
    fn filter_contains() {
        let data = sample_list();
        let op = PipelineOp::Filter {
            field: "name".into(),
            op: CmpOp::Contains,
            value: Value::str("tar"),
        };
        let result = apply(&op, &data);
        if let Value::Array(a) = result {
            assert_eq!(a.len(), 1);
        }
    }

    #[test]
    fn sort_by_size_ascending() {
        let data = sample_list();
        let op = PipelineOp::SortBy {
            field: "size".into(),
            descending: false,
        };
        let result = apply(&op, &data);
        if let Value::Array(a) = result {
            // Smallest first: 0 (app), 0 (docs), 500, 20000000
            let sizes: Vec<i64> = a.iter().map(|v| {
                if let Value::Obj(o) = v { o.get("size").and_then(|s| if let Value::I64(n) = s { Some(n) } else { None }).unwrap_or(0) } else { 0 }
            }).collect();
            assert_eq!(sizes, vec![0, 0, 500, 20_000_000]);
        }
    }

    #[test]
    fn sort_by_size_descending() {
        let data = sample_list();
        let op = PipelineOp::SortBy {
            field: "size".into(),
            descending: true,
        };
        let result = apply(&op, &data);
        if let Value::Array(a) = result {
            let sizes: Vec<i64> = a.iter().map(|v| {
                if let Value::Obj(o) = v { o.get("size").and_then(|s| if let Value::I64(n) = s { Some(n) } else { None }).unwrap_or(0) } else { 0 }
            }).collect();
            assert_eq!(sizes[0], 20_000_000);
        }
    }

    #[test]
    fn select_fields() {
        let data = sample_list();
        let op = PipelineOp::Select {
            fields: vec!["name".into(), "size".into()],
        };
        let result = apply(&op, &data);
        if let Value::Array(a) = result {
            if let Value::Obj(o) = a.get(0).unwrap() {
                assert!(o.get("name").is_some());
                assert!(o.get("size").is_some());
                assert!(o.get("type").is_none()); // type was dropped
            }
        }
    }

    #[test]
    fn map_to_field() {
        let data = sample_list();
        let op = PipelineOp::Map { field: "name".into() };
        let result = apply(&op, &data);
        if let Value::Array(a) = result {
            assert_eq!(a.len(), 4);
            // Each element should be the name value. Value::Str.to_string() may
            // add quotes, so check substring containment.
            let names: String = a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
            assert!(names.contains("app"), "names: {names}");
            assert!(names.contains("big.tar"), "names: {names}");
        }
    }

    #[test]
    fn take_and_skipback() {
        let data = sample_list();
        let take_op = PipelineOp::Take(2);
        let result = apply(&take_op, &data);
        if let Value::Array(a) = &result {
            assert_eq!(a.len(), 2);
        }
        let skip_op = PipelineOp::SkipBack(2);
        let result = apply(&skip_op, &data);
        if let Value::Array(a) = &result {
            assert_eq!(a.len(), 2); // last 2 of 4
        }
    }

    #[test]
    fn count() {
        let data = sample_list();
        let op = PipelineOp::Count;
        let result = apply(&op, &data);
        assert!(matches!(result, Value::USize(4)));
    }

    #[test]
    fn cmp_op_from_str() {
        assert_eq!(CmpOp::from_str(">"), Some(CmpOp::Gt));
        assert_eq!(CmpOp::from_str("=="), Some(CmpOp::Eq));
        assert_eq!(CmpOp::from_str("contains"), Some(CmpOp::Contains));
        assert_eq!(CmpOp::from_str("??"), None);
    }
}
