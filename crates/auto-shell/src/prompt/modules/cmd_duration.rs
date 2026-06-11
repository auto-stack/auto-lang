//! Command duration module — shows last command execution time

use crate::prompt::context::AshContext;
use crate::prompt::module::{PromptModule, PromptSegment, SegmentStyle};
use nu_ansi_term::Color;

pub struct CmdDurationModule {
    style: SegmentStyle,
    min_time: u64,
}

impl CmdDurationModule {
    pub fn new(config: &super::super::config::AshConfig) -> Self {
        let min_time = config.module_int("cmd_duration", "min_time", -1);
        Self {
            style: SegmentStyle {
                fg: Some(Color::Yellow),
                ..Default::default()
            },
            min_time: if min_time >= 0 {
                min_time as u64
            } else {
                config.cmd_duration_threshold
            },
        }
    }
}

impl PromptModule for CmdDurationModule {
    fn name(&self) -> &str {
        "cmd_duration"
    }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let ms = ctx.cmd_duration_ms?;
        if ms < self.min_time {
            return None;
        }

        let content = if ms < 1000 {
            format!("{}ms ", ms)
        } else if ms < 60_000 {
            format!("{:.1}s ", ms as f64 / 1000.0)
        } else {
            let mins = ms / 60_000;
            let secs = (ms % 60_000) / 1000;
            format!("{}m{}s ", mins, secs)
        };

        Some(PromptSegment::new(content, self.style.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::config::AshConfig;
    use std::path::PathBuf;

    fn make_ctx(duration_ms: Option<u64>) -> AshContext {
        AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            duration_ms,
            None,
            AshConfig::default(),
        )
    }

    #[test]
    fn test_no_duration() {
        let ctx = make_ctx(None);
        let module = CmdDurationModule::new(&AshConfig::default());
        assert!(module.render(&ctx).is_none());
    }

    #[test]
    fn test_below_threshold() {
        let ctx = make_ctx(Some(500));
        let module = CmdDurationModule::new(&AshConfig::default());
        // Default threshold is 2000ms
        assert!(module.render(&ctx).is_none());
    }

    #[test]
    fn test_above_threshold() {
        // 2500ms > 2000ms (default threshold), but < 1000 is the boundary for ms vs s format
        // So 2500ms should show as "2.5s"
        let ctx = make_ctx(Some(2500));
        let module = CmdDurationModule::new(&AshConfig::default());
        let seg = module.render(&ctx).unwrap();
        assert_eq!(seg.content, "2.5s ");
    }

    #[test]
    fn test_seconds() {
        let ctx = make_ctx(Some(5200));
        let module = CmdDurationModule::new(&AshConfig::default());
        let seg = module.render(&ctx).unwrap();
        assert_eq!(seg.content, "5.2s ");
    }

    #[test]
    fn test_minutes_seconds() {
        let ctx = make_ctx(Some(125000));
        let module = CmdDurationModule::new(&AshConfig::default());
        let seg = module.render(&ctx).unwrap();
        assert_eq!(seg.content, "2m5s ");
    }
}
