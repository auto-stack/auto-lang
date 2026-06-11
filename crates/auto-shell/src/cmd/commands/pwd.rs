use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline};
use miette::Result;

pub struct PwdCommand;

impl Command for PwdCommand {
    fn name(&self) -> &str {
        "pwd"
    }

    fn signature(&self) -> Signature {
        Signature::new("pwd", "Print working directory")
    }

    fn run(
        &self,
        _args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let path_str = normalize_path(shell.pwd().display().to_string());
        Ok(PipelineData::from_text(path_str))
    }

    fn run_atom(
        &self,
        _args: &crate::cmd::parser::ParsedArgs,
        _input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let path_str = normalize_path(shell.pwd().display().to_string());
        Ok(AtomPipeline::from_atom(Atom::path(path_str)))
    }
}

fn normalize_path(mut path_str: String) -> String {
    if path_str.starts_with(r"\\?\") {
        path_str = path_str[4..].to_string();
    }
    path_str.replace('\\', "/")
}
