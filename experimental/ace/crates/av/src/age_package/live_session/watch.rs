use ace_av_core::program_snapshot::SavedGlobalCells;
use ace_core::source::age_package::{AgeWidgetRuntimeWatch, AgeWidgetWatchSource};

use super::Value;

pub(super) struct Watch {
    pub entry: String,
    sources: Vec<AgeWidgetWatchSource>,
    immediate: bool,
}

pub(super) struct Watched<'a> {
    pub inputs: Option<&'a SavedGlobalCells>,
    pub model: &'a SavedGlobalCells,
}

impl Watch {
    pub fn new(entry: String, watch: &AgeWidgetRuntimeWatch) -> Self {
        Self {
            entry,
            sources: watch.sources.clone(),
            immediate: watch.immediate,
        }
    }
}

impl Watched<'_> {
    fn source(&self, source: AgeWidgetWatchSource) -> Option<&Value> {
        match source {
            AgeWidgetWatchSource::Property(index) => self.inputs?.value_at(index),
            AgeWidgetWatchSource::Model(index) => {
                let Value::Record(model) = self.model.value_at(0)? else {
                    return None;
                };
                model.fields.get(index)
            }
        }
    }
}

pub(super) fn run(
    watches: &[Watch],
    previous: Option<Watched<'_>>,
    next: Watched<'_>,
    mut transition: impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    for watch in watches {
        let mut changed = previous.is_none() && watch.immediate;
        if let Some(previous) = &previous {
            for &source in &watch.sources {
                let before = previous
                    .source(source)
                    .ok_or("ace.desktop.widget-watch: missing previous source")?;
                let after = next
                    .source(source)
                    .ok_or("ace.desktop.widget-watch: missing current source")?;
                if !same_value(before, after) {
                    changed = true;
                    break;
                }
            }
        }
        if changed {
            transition(&watch.entry)?;
        }
    }
    Ok(())
}

// The slot has already established the checked type on both sides. Record
// indices are local to AV units and need not match for equal field values.
fn same_value(left: &Value, right: &Value) -> bool {
    let mut pairs = vec![(left.clone(), right.clone())];
    while let Some((left, right)) = pairs.pop() {
        match (&left, &right) {
            (Value::Record(left), Value::Record(right))
                if left.fields.len() == right.fields.len() =>
            {
                pairs.extend(
                    left.fields
                        .iter()
                        .cloned()
                        .zip(right.fields.iter().cloned()),
                );
            }
            (Value::Record(_), Value::Record(_)) => return false,
            (Value::List(left), Value::List(right)) => {
                let left = left.rc.borrow();
                let right = right.rc.borrow();
                if left.elems.len() != right.elems.len() {
                    return false;
                }
                pairs.extend(left.elems.iter().cloned().zip(right.elems.iter().cloned()));
            }
            _ if left == right => {}
            _ => return false,
        }
    }
    true
}
