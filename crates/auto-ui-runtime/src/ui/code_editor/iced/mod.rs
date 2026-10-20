// Plan 413 §3.1 ③ iced adapter — the only iced dependency point of the
// code editor (hard layering constraint: core/ and draw.rs never import
// iced).

pub mod gutter;
pub mod widget;

pub use widget::CodeEditor;
