// Component abstraction - improved version aligned with Auto language

use super::view::View;
use std::fmt::Debug;

/// Core component trait - simplified and aligned with Auto's `fn on` syntax
///
/// # Example
/// ```rust
/// struct Counter { count: i64 }
///
/// #[derive(Clone)]
/// enum Msg { Inc, Dec }
///
/// impl Component for Counter {
///     type Msg = Msg;
///
///     fn on(&mut self, msg: Self::Msg) {
///         match msg {
///             Msg::Inc => self.count += 1,
///             Msg::Dec => self.count -= 1,
///         }
///     }
///
///     fn view(&self) -> View<Self::Msg> {
///         View::col()
///             .spacing(10)
///             .child(View::button("+", Msg::Inc))
///             .child(View::text(self.count.to_string()))
///             .child(View::button("-", Msg::Dec))
///     }
/// }
/// ```
pub trait Component: Sized + Debug {
    /// Message type - must be cloneable for event handling
    type Msg: Clone + Debug + 'static;

    /// Handle messages - Auto's equivalent of `fn on(ev Msg)`
    ///
    /// This is where state mutations happen based on incoming messages.
    fn on(&mut self, msg: Self::Msg);

    /// Render the view - Auto's equivalent of `fn view() View`
    ///
    /// Returns the abstract view tree that will be adapted to specific backends.
    fn view(&self) -> View<Self::Msg>;

    /// Optional periodic tick interval in milliseconds (e.g., `.Tick` handlers).
    ///
    /// Plan 365 W1 follow-up: this replaces the former `subscription()` method
    /// which leaked `iced::Subscription` into this backend-neutral trait. The
    /// iced backend reads this value and builds `iced::time::every(...)` from it;
    /// other backends ignore it. Default: no ticking.
    fn tick_interval_ms(&self) -> Option<u32> {
        None
    }

    /// Snapshot of this component's scalar state fields, keyed by field name.
    ///
    /// Used by the rust-mode MCP `autoui_state` tool (Plan 371 Task 21): in
    /// rust mode there is no VM heap to read state from, so the DevTools layer
    /// calls this each frame and pushes the result into `SharedState.state`.
    ///
    /// The default returns an empty map -- VM mode never reads this (it reads
    /// state directly off the VM heap), so existing `impl Component for` sites
    /// are unaffected. The a2r generator overrides this per-struct to emit the
    /// scalar fields it knows about (`String`/`i32`/`bool`/`f64`/...), skipping
    /// collections and nested components.
    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_val::Value> {
        std::collections::HashMap::new()
    }
}
