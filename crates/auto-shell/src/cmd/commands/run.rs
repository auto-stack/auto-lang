//! `auto run` command
//!
//! Generate backend code and start development server
//!
//! # Usage
//!
//! ```bash
//! auto run                 # Run all backends
//! auto run --target vue    # Only run vue dev
//! auto run --target tauri  # Only run tauri dev
//! ```

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use miette::Result;

/// `auto run` command
pub struct RunCommand;

impl Command for RunCommand {
    fn name(&self) -> &str {
        "run"
    }

    fn signature(&self) -> Signature {
        Signature::new("run", "Generate code and start dev server for configured backends")
            .optional("target", "Target backend (vue, jet, tauri, etc.)")
            .flag("release", "Run in release mode")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let target = args.positionals.get(0).map(|s| s.as_str());
        let _release = args.has_flag("release");

        // TODO: Implement run logic
        // 1. Read pac.at config
        // 2. Parse backend config
        // 3. Generate code for each backend
        // 4. Start dev servers

        match target {
            Some(t) => Ok(PipelineData::from_text(format!("Running target: {}", t))),
            None => Ok(PipelineData::from_text("Running all backends".to_string())),
        }
    }
}
