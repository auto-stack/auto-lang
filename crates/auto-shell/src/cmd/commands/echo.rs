use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use miette::Result;

pub struct EchoCommand;

impl Command for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }

    fn signature(&self) -> Signature {
        Signature::new("echo", "Print arguments")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        Ok(PipelineData::from_text(args.positionals.join(" ")))
    }
}
