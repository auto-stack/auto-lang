use crate::cmd::{Command, Signature};
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
        args: &[String],
        _input: Option<&str>,
        shell: &mut Shell,
    ) -> Result<Option<String>> {
        let path = args.get(0).map(|s| s.as_str()).unwrap_or("~");
        shell.cd(path)?;
        Ok(None)
    }
}
