//! Prompt module trait and core types
//!
//! Each prompt element (directory, git, time, etc.) implements `PromptModule`.
//! Modules are rendered in parallel by the prompt engine.

use nu_ansi_term::{Color, Style};

/// Prompt module trait — each element of the prompt is an independent module.
///
/// Modules are pure functions: they receive context and return a styled segment.
/// Heavy I/O (git, filesystem) is handled lazily by `AshContext`.
pub trait PromptModule: Send + Sync {
    /// Module name (used as config key in TOML)
    fn name(&self) -> &str;

    /// Render the module. Returns `None` if the module should not be displayed.
    fn render(&self, ctx: &super::context::AshContext) -> Option<PromptSegment>;
}

/// A styled text segment produced by a prompt module.
///
/// Multiple segments are concatenated to form the final prompt string.
#[derive(Debug, Clone)]
pub struct PromptSegment {
    /// The content text
    pub content: String,
    /// Styling (foreground, background, bold, etc.)
    pub style: SegmentStyle,
}

/// Style definition for a prompt segment.
///
/// Simplified version — no prev_fg/prev_bg chaining like Starship.
#[derive(Debug, Clone, Default)]
pub struct SegmentStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl PromptSegment {
    /// Create a new segment with content and style
    pub fn new(content: impl Into<String>, style: SegmentStyle) -> Self {
        Self {
            content: content.into(),
            style,
        }
    }

    /// Create a plain text segment (no styling)
    pub fn plain(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: SegmentStyle::default(),
        }
    }

    /// Convert to ANSI-colored string for terminal output
    pub fn to_ansi_string(&self) -> String {
        let mut style = Style::new();
        if let Some(fg) = self.style.fg {
            style = style.fg(fg);
        }
        if let Some(bg) = self.style.bg {
            style = style.on(bg);
        }
        if self.style.bold {
            style = style.bold();
        }
        if self.style.italic {
            style = style.italic();
        }
        if self.style.underline {
            style = style.underline();
        }
        style.paint(&self.content).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_segment() {
        let seg = PromptSegment::plain("hello");
        assert_eq!(seg.content, "hello");
        assert!(seg.style.fg.is_none());
    }

    #[test]
    fn test_styled_segment_ansi() {
        let seg = PromptSegment::new(
            "test",
            SegmentStyle {
                fg: Some(Color::Red),
                bold: true,
                ..Default::default()
            },
        );
        let ansi = seg.to_ansi_string();
        assert!(ansi.contains("test"));
        assert!(ansi.contains("\x1b["));
    }

    #[test]
    fn test_default_style_no_ansi() {
        let seg = PromptSegment::plain("plain");
        let ansi = seg.to_ansi_string();
        // Default style should still produce the content
        assert!(ansi.contains("plain"));
    }
}
