use crate::cmd::{Command, Signature};
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
        _input: Option<&str>,
        _shell: &mut Shell,
    ) -> Result<Option<String>> {
        Ok(Some(args.positionals.join(" ")))
    }
}
