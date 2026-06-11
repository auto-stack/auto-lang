use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use miette::Result;

pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn signature(&self) -> Signature {
        Signature::new("help", "Display help information for commands")
            .optional("command", "Specific command to show help for")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = self.build_help_text(args, shell)?;
        Ok(PipelineData::from_text(text))
    }

    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let text = self.build_help_text(args, shell)?;
        Ok(AtomPipeline::from_atom(Atom::new(
            auto_val::Value::str(&text), AtomType::HelpInfo,
        )))
    }
}

impl HelpCommand {
    fn build_help_text(&self, args: &crate::cmd::parser::ParsedArgs, shell: &Shell) -> Result<String> {
        let registry = shell.registry();

        if let Some(cmd_name) = args.positionals.get(0) {
            if let Some(cmd) = registry.get(cmd_name) {
                let sig = cmd.signature();
                let mut help = format!("Command: {}\nDescription: {}\n", sig.name, sig.description);
                if !sig.arguments.is_empty() {
                    help.push_str("Arguments:\n");
                    for arg in sig.arguments {
                        let req = if arg.required { "required" } else { "optional" };
                        help.push_str(&format!(
                            "  {:<12} ({}) - {}\n",
                            arg.name, req, arg.description
                        ));
                    }
                }
                return Ok(help);
            } else {
                return Ok(format!("Command '{}' not found.", cmd_name));
            }
        }

        let mut signatures = registry.params();
        signatures.sort_by(|a, b| a.name.cmp(&b.name));

        let mut output = String::from("Available Commands:\n");
        for sig in signatures {
            output.push_str(&format!("  {:<10} {}\n", sig.name, sig.description));
        }
        Ok(output)
    }
}
