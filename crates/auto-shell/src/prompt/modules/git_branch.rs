//! Git branch module — shows current branch name

use crate::prompt::context::AshContext;
use crate::prompt::module::{PromptModule, PromptSegment, SegmentStyle};
use nu_ansi_term::Color;

pub struct GitBranchModule {
    style: SegmentStyle,
    symbol: String,
}

impl GitBranchModule {
    pub fn new(config: &super::super::config::AshConfig) -> Self {
        Self {
            style: SegmentStyle {
                fg: Some(Color::Green),
                bold: true,
                ..Default::default()
            },
            // Default branch glyph: ⎇ (U+2387 "Alternative Key Symbol") — single-width,
            // renders in most fonts (no Nerd Font required), the same glyph Fish uses.
            // Overridable via ~/.config/ash.toml: [git_branch] symbol = "..."
            symbol: config.module_string("git_branch", "symbol", "⎇ ").to_string(),
        }
    }
}

impl PromptModule for GitBranchModule {
    fn name(&self) -> &str {
        "git_branch"
    }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let git_info = ctx.git_info()?;
        Some(PromptSegment::new(
            format!("{}{} ", self.symbol, git_info.branch),
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
    fn test_git_branch_not_in_repo() {
        // /tmp is typically not a git repo
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            None,
            None,
            AshConfig::default(),
        );
        let module = GitBranchModule::new(&AshConfig::default());
        // May or may not be in a git repo, just verify no panic
        let _ = module.render(&ctx);
    }
}
