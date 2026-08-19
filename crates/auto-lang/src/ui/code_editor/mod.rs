// Plan 413: cross-platform code editor widget (cosmic-text ViEditor engine,
// iced backend). Layering (§3.1):
//
//   core/   ① state machine + rendering contract — no iced imports
//   draw.rs ② EditorDrawList — backend-neutral data types
//   theme.rs② CodeEditorTheme — semantic-color synthesis
//   iced/   ③ adapter — the only iced dependency point
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

pub mod core;
pub mod draw;
pub mod theme;

#[cfg(feature = "ui-iced")]
pub mod iced;

pub use core::{
    code_editor, code_editor_count, code_editor_cursor, code_editor_dispose, code_editor_set_text,
    code_editor_text, storage_key, CodeEditorConfig, CodeEditorCore, CoreOutput, EditorButton,
    EditorClipboard, EditorInput, EditorKey, EditorModifiers, NullClipboard,
};
pub use draw::EditorDrawList;
pub use theme::{current_theme, set_theme_source, CodeEditorTheme, Rgba};

#[cfg(feature = "ui-iced")]
pub use iced::CodeEditor as IcedCodeEditor;
