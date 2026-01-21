use crate::cmd::{fs, Command, Signature};
use crate::shell::Shell;
use miette::Result;
use std::path::Path;

pub struct LsCommand;

impl Command for LsCommand {
    fn name(&self) -> &str {
        "ls"
    }

    fn signature(&self) -> Signature {
        Signature::new("ls", "List directory contents").optional("path", "Path to list")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: Option<&str>,
        shell: &mut Shell,
    ) -> Result<Option<String>> {
        let path_arg = args.positionals.get(0).map(|s| s.as_str()).unwrap_or(".");
        let path = Path::new(path_arg);

        let output = fs::ls_command(path, &shell.pwd())?;
        Ok(Some(output))
    }
}
