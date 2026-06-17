//! Character module — the prompt indicator symbol (❯ or custom)

use crate::prompt::context::AshContext;
use crate::prompt::module::{PromptModule, PromptSegment, SegmentStyle};
use nu_ansi_term::Color;

pub struct CharacterModule {
    success_char: String,
    error_char: String,
    success_style: SegmentStyle,
    error_style: SegmentStyle,
}

impl CharacterModule {
    pub fn new(config: &super::super::config::AshConfig) -> Self {
        Self {
            success_char: config
                .module_string("character", "success", "❯")
                .to_string(),
            error_char: config
                .module_string("character", "error", "❯")
                .to_string(),
            success_style: SegmentStyle {
                fg: Some(Color::Green),
                bold: true,
                ..Default::default()
            },
            error_style: SegmentStyle {
                fg: Some(Color::Red),
                bold: true,
                ..Default::default()
            },
        }
    }

    /// Plan 322: Create with a custom symbol (for mode switching).
    /// Uses the custom symbol for both success and error states.
    pub fn with_symbol(symbol: &str) -> Self {
        Self {
            success_char: symbol.to_string(),
            error_char: symbol.to_string(),
            success_style: SegmentStyle {
                fg: Some(Color::Green),
                bold: true,
                ..Default::default()
            },
            error_style: SegmentStyle {
                fg: Some(Color::Red),
                bold: true,
                ..Default::default()
            },
        }
    }
}

impl PromptModule for CharacterModule {
    fn name(&self) -> &str {
        "character"
    }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let is_error = ctx.last_status.map(|s| s != 0).unwrap_or(false);
        Some(PromptSegment::new(
            format!(
                "{} ",
                if is_error {
                    &self.error_char
                } else {
                    &self.success_char
                }
            ),
            if is_error {
                self.error_style.clone()
            } else {
                self.success_style.clone()
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::config::AshConfig;
    use std::path::PathBuf;

    #[test]
    fn test_character_success() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            Some(0),
            AshConfig::default(),
        );
        let module = CharacterModule::new(&AshConfig::default());
        let seg = module.render(&ctx).unwrap();
        assert_eq!(seg.content, "❯ ");
        assert_eq!(seg.style.fg, Some(Color::Green));
    }

    #[test]
    fn test_character_error() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            Some(1),
            AshConfig::default(),
        );
        let module = CharacterModule::new(&AshConfig::default());
        let seg = module.render(&ctx).unwrap();
        assert_eq!(seg.content, "❯ ");
        assert_eq!(seg.style.fg, Some(Color::Red));
    }

    #[test]
    fn test_character_no_status() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            None,
            AshConfig::default(),
        );
        let module = CharacterModule::new(&AshConfig::default());
        let seg = module.render(&ctx).unwrap();
        // No status = treat as success
        assert_eq!(seg.style.fg, Some(Color::Green));
    }
}
