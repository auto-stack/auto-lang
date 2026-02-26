use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
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
        let path = shell.pwd();
        let mut path_str = path.display().to_string();

        // 1. Remove UNC prefix on Windows
        if path_str.starts_with(r"\\?\") {
            path_str = path_str[4..].to_string();
        }

        // 2. Unify separators to forward slash
        path_str = path_str.replace('\\', "/");

        Ok(PipelineData::from_text(path_str))
    }
}
