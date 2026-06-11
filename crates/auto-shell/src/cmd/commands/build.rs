//! `auto build` command
//!
//! Generate backend code and build for configured backends

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use miette::Result;

/// `auto build` command
pub struct BuildCommand;

impl Command for BuildCommand {
    fn name(&self) -> &str {
        "build"
    }

    fn signature(&self) -> Signature {
        Signature::new("build", "Generate code and build for configured backends")
            .optional("target", "Target backend (vue, jet, tauri, etc.)")
            .flag("release", "Build in release mode")
            .flag("watch", "Watch for changes and rebuild")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let target = args.positionals.get(0).map(|s| s.as_str());
        let _release = args.has_flag("release");
        let _watch = args.has_flag("watch");

        match target {
            Some(t) => Ok(PipelineData::from_text(format!("Building target: {}", t))),
            None => Ok(PipelineData::from_text("Building all backends".to_string())),
        }
    }

    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let target = args.positionals.get(0).map(|s| s.as_str());
        let msg = match target {
            Some(t) => format!("Building target: {}", t),
            None => "Building all backends".to_string(),
        };
        Ok(AtomPipeline::from_atom(Atom::new(
            auto_val::Value::str(&msg), AtomType::BuildResult,
        )))
    }
}
