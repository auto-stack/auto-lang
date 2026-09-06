// PLAN-009 P1 ② iced adapter — the only iced dependency point of the
// terminal component (hard layering constraint: mod.rs never imports iced).

pub mod widget;

pub use widget::{Terminal, TerminalState};
