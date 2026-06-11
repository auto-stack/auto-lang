//! Git status module — shows staged/unstaged/untracked counts

use crate::prompt::context::AshContext;
use crate::prompt::module::{PromptModule, PromptSegment, SegmentStyle};
use nu_ansi_term::Color;

pub struct GitStatusModule {
    style: SegmentStyle,
}

impl GitStatusModule {
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

impl PromptModule for GitStatusModule {
    fn name(&self) -> &str {
        "git_status"
    }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let git_info = ctx.git_info()?;
        let s = &git_info.status;

        let mut parts = Vec::new();
        if s.staged > 0 {
            parts.push(format!("+{}", s.staged));
        }
        if s.unstaged > 0 {
            parts.push(format!("!{}", s.unstaged));
        }
        if s.untracked > 0 {
            parts.push(format!("?{}", s.untracked));
        }
        if s.conflicted > 0 {
            parts.push(format!("~{}", s.conflicted));
        }
        if s.ahead > 0 {
            parts.push(format!("⇡{}", s.ahead));
        }
        if s.behind > 0 {
            parts.push(format!("⇣{}", s.behind));
        }

        if parts.is_empty() {
            return None;
        }

        Some(PromptSegment::new(
            format!("[{}] ", parts.join(" ")),
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
    fn test_git_status_clean_or_no_repo() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            None,
            AshConfig::default(),
        );
        let module = GitStatusModule::new(&AshConfig::default());
        // Clean repo or no repo = None
        let _ = module.render(&ctx);
    }
}
