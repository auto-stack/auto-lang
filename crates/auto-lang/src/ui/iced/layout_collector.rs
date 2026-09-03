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
    /// PLAN-530 步骤2 表面追踪：同一 widget id 被遍历到第二次即"双份布局"
    /// 实证（树单份/绘制双份假设的判定面）。env P530_TRACE=1 时 finish()
    /// 打印到 stderr。
    dup: Vec<(String, (f32, f32, f32, f32), (f32, f32, f32, f32))>,
}

impl LayoutCollector {
    pub fn new() -> Self {
        Self {
            bounds: HashMap::new(),
            dup: Vec::new(),
        }
    }

    /// Try to extract an aura ID string from an iced widget Id.
    /// The Id was created via `Id::from(format!("aura_{}", N))`.
    /// Debug format: `Id(Custom("aura_0"))`
    fn aura_id_str(id: &Id) -> Option<String> {
        let debug = format!("{:?}", id);
        // G4e (411 P2-B #1): recognize both id conventions — `aura_N` (F12
        // wrap_debug) and `vnode_<hash>` (the deterministic path hash the
        // VTree uses; bounds backfill parses it without a registration map).
        for prefix in ["vnode_", "aura_"] {
            if let Some(start) = debug.find(prefix) {
                let rest = &debug[start..];
                let end = rest.find('"').unwrap_or(rest.len());
                return Some(rest[..end].to_string());
            }
        }
        None
    }

    fn try_record(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if let Some(id) = id {
            if let Some(key) = Self::aura_id_str(id) {
                if let Some(prev) = self.bounds.get(&key) {
                    self.dup.push((key.clone(), *prev, (bounds.x, bounds.y, bounds.width, bounds.height)));
                }
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
        // PLAN-530 步骤2：P530_TRACE=1 时输出重复 id 的双组 bounds。
        if std::env::var("P530_TRACE").as_deref() == Ok("1") {
            if self.dup.is_empty() {
                eprintln!("[P530-TRACE] layout: no duplicate widget ids ({} total)", self.bounds.len());
            } else {
                eprintln!("[P530-TRACE] layout: {} DUPLICATE widget ids ({} total):", self.dup.len(), self.bounds.len());
                for (key, a, b) in self.dup.iter().take(20) {
                    eprintln!("[P530-TRACE]   dup {key}: first=({:.0},{:.0},{:.0}x{:.0}) again=({:.0},{:.0},{:.0}x{:.0})",
                        a.0, a.1, a.2, a.3, b.0, b.1, b.2, b.3);
                }
            }
        }
        Outcome::Some(self.bounds.clone())
    }
}
