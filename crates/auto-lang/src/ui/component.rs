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
}
