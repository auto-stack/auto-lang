//! Time module — shows current time (typically in right prompt)

use crate::prompt::context::AshContext;
use crate::prompt::module::{PromptModule, PromptSegment, SegmentStyle};
use nu_ansi_term::Color;

pub struct TimeModule {
    style: SegmentStyle,
    time_format: String,
}

impl TimeModule {
    pub fn new(config: &super::super::config::AshConfig) -> Self {
        Self {
            style: SegmentStyle {
                fg: Some(Color::Yellow),
                ..Default::default()
            },
            time_format: config
                .module_string("time", "time_format", "%H:%M")
                .to_string(),
        }
    }
}

impl PromptModule for TimeModule {
    fn name(&self) -> &str {
        "time"
    }

    fn render(&self, _ctx: &AshContext) -> Option<PromptSegment> {
        let now = chrono::Local::now();
        let formatted = now.format(&self.time_format).to_string();
        Some(PromptSegment::new(format!("[{}] ", formatted), self.style.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::config::AshConfig;
    use std::path::PathBuf;

    #[test]
    fn test_time_renders() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            None,
            AshConfig::default(),
        );
        let module = TimeModule::new(&AshConfig::default());
        let seg = module.render(&ctx).unwrap();
        assert!(seg.content.starts_with('['));
        assert!(seg.content.contains(':'));
        assert!(seg.content.ends_with("] "));
    }
}
