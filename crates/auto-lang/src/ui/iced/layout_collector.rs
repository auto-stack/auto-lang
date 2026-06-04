//! # Layout Bounds Collector Operation (Plan 282)
//!
//! Custom `iced::advanced::widget::Operation` that collects actual rendered layout
//! rectangles from iced container/scrollable widgets that have been assigned
//! `aura_N` IDs via `.id()`.

use std::collections::HashMap;

use iced::advanced::widget::Operation;
use iced::advanced::widget::operation::{Focusable, Outcome, Scrollable, TextInput};
use iced::Rectangle;
use iced::Vector;
use iced::widget::Id;

/// Collected layout bounds: widget ID string → (x, y, width, height).
pub type BoundsMap = HashMap<String, (f32, f32, f32, f32)>;

/// Operation that traverses the iced widget tree and collects bounds
/// for all containers/scrollables/inputs with `aura_`-prefixed IDs.
pub struct LayoutCollector {
    bounds: BoundsMap,
}

impl LayoutCollector {
    pub fn new() -> Self {
        Self {
            bounds: HashMap::new(),
        }
    }

    /// Try to extract an aura ID string from an iced widget Id.
    /// The Id was created via `Id::from(format!("aura_{}", N))`.
    /// Debug format: `Id(Custom("aura_0"))`
    fn aura_id_str(id: &Id) -> Option<String> {
        let debug = format!("{:?}", id);
        // Extract "aura_N" from Debug output like Id(Custom("aura_0"))
        let start = debug.find("aura_")?;
        let rest = &debug[start..];
        let end = rest.find('"').unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }

    fn try_record(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if let Some(id) = id {
            if let Some(key) = Self::aura_id_str(id) {
                self.bounds.insert(key, (bounds.x, bounds.y, bounds.width, bounds.height));
            }
        }
    }
}

impl Operation<BoundsMap> for LayoutCollector {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<BoundsMap>)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.try_record(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        self.try_record(id, bounds);
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _state: &mut dyn Focusable,
    ) {
        self.try_record(id, bounds);
    }

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _state: &mut dyn TextInput,
    ) {
        self.try_record(id, bounds);
    }

    fn finish(&self) -> Outcome<BoundsMap> {
        Outcome::Some(self.bounds.clone())
    }
}
