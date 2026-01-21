use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use miette::Result;

pub struct CdCommand;

impl Command for CdCommand {
    fn name(&self) -> &str {
        "cd"
    }

    fn signature(&self) -> Signature {
        Signature::new("cd", "Change directory").optional("path", "Directory path")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let path = args.positionals.get(0).map(|s| s.as_str()).unwrap_or("~");
        shell.cd(path).map(|_| PipelineData::empty())
    }
}
