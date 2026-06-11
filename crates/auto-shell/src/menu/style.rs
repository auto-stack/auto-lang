//! Color scheme for completion types

use nu_ansi_term::{Color, Style};

use crate::completions::CompletionKind;

/// Get the display color for a completion kind
pub fn kind_color(kind: CompletionKind) -> Style {
    match kind {
        CompletionKind::Command => Color::Cyan.bold(),
        CompletionKind::External => Color::Green.normal(),
        CompletionKind::File => Color::White.normal(),
        CompletionKind::Directory => Color::Blue.bold(),
        CompletionKind::Variable => Color::Magenta.normal(),
        CompletionKind::Flag => Color::Yellow.normal(),
        CompletionKind::Subcommand => Color::Cyan.normal(),
        CompletionKind::AiSuggested => Color::LightGreen.italic(),
    }
}

/// Style for selected items (reverse + bold)
pub fn selected_overlay() -> Style {
    Color::Green.bold().reverse()
}

/// Style for the search bar
pub fn search_style() -> Style {
    Color::Yellow.bold()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_color_returns_style() {
        let style = kind_color(CompletionKind::Command);
        // Just verify it doesn't panic and returns a style
        let _ = style.prefix();
    }

    #[test]
    fn test_selected_overlay() {
        let style = selected_overlay();
        let _ = style.prefix();
    }
}
