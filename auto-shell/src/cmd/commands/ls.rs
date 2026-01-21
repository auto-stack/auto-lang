use crate::cmd::{fs, Command, PipelineData, Signature};
use crate::shell::Shell;
use miette::Result;
use std::path::Path;

pub struct LsCommand;

impl Command for LsCommand {
    fn name(&self) -> &str {
        "ls"
    }

    fn signature(&self) -> Signature {
        Signature::new("ls", "List directory contents")
            .optional("path", "Path to list")
            .flag_with_short("all", 'a', "Show all files including hidden (starts with .)")
            .flag_with_short("long", 'l', "Long listing format (permissions, owner, size, time)")
            .flag_with_short("human-readable", 'h', "Human-readable file sizes (1K, 234M, 2G)")
            .flag_with_short("time", 't', "Sort by modification time (newest first)")
            .flag_with_short("reverse", 'r', "Reverse sort order")
            .flag_with_short("recursive", 'R', "List subdirectories recursively")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let path_arg = args.positionals.get(0).map(|s| s.as_str()).unwrap_or(".");
        let path = Path::new(path_arg);

        // Extract flags
        let all = args.has_flag("all");
        let long = args.has_flag("long");
        let human = args.has_flag("human-readable");
        let time = args.has_flag("time");
        let reverse = args.has_flag("reverse");
        let recursive = args.has_flag("recursive");

        let output = fs::ls_command(
            path,
            &shell.pwd(),
            all,
            long,
            human,
            time,
            reverse,
            recursive,
        )?;
        Ok(PipelineData::from_text(output))
    }
}
