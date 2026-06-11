//! Status module — shows last command exit code when non-zero

use crate::prompt::context::AshContext;
use crate::prompt::module::{PromptModule, PromptSegment, SegmentStyle};
use nu_ansi_term::Color;

pub struct StatusModule {
    style: SegmentStyle,
}

impl StatusModule {
    pub fn new(_config: &super::super::config::AshConfig) -> Self {
        Self {
            style: SegmentStyle {
                fg: Some(Color::Red),
                bold: true,
                ..Default::default()
            },
        }
    }
}

impl PromptModule for StatusModule {
    fn name(&self) -> &str {
        "status"
    }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let status = ctx.last_status?;
        if status == 0 {
            return None;
        }
        Some(PromptSegment::new(
            format!("{} ", status),
            self.style.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::config::AshConfig;
    use std::path::PathBuf;

    #[test]
    fn test_status_zero() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            Some(0),
            AshConfig::default(),
        );
        let module = StatusModule::new(&AshConfig::default());
        assert!(module.render(&ctx).is_none());
    }

    #[test]
    fn test_status_nonzero() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            Some(127),
            AshConfig::default(),
        );
        let module = StatusModule::new(&AshConfig::default());
        let seg = module.render(&ctx).unwrap();
        assert_eq!(seg.content, "127 ");
    }

    #[test]
    fn test_status_none() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            None,
            AshConfig::default(),
        );
        let module = StatusModule::new(&AshConfig::default());
        assert!(module.render(&ctx).is_none());
    }
}
