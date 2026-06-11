//! `auto run` command
//!
//! Generate backend code and start development server

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
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
        match target {
            Some(t) => Ok(PipelineData::from_text(format!("Running target: {}", t))),
            None => Ok(PipelineData::from_text("Running all backends".to_string())),
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
            Some(t) => format!("Running target: {}", t),
            None => "Running all backends".to_string(),
        };
        Ok(AtomPipeline::from_atom(Atom::new(
            auto_val::Value::str(&msg), AtomType::RunResult,
        )))
    }
}
