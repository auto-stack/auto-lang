//! `version` command - Show shell version info
//!
//! Returns a Record with version, name, and description fields.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Obj};
use miette::Result;

pub struct VersionCommand;

const VERSION: &str = "0.1.0";
const NAME: &str = "auto-shell";
const DESCRIPTION: &str = "A modern shell environment using AutoLang as the scripting language";

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }

    fn signature(&self) -> Signature {
        Signature::new("version", "Show shell version information")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let obj = build_version_record();
        Ok(PipelineData::from_value(Value::Obj(obj)))
    }

    fn run_atom(
        &self,
        _args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let obj = build_version_record();
        Ok(AtomPipeline::from_atom(Atom::new(
            Value::Obj(obj),
            AtomType::Record,
        )))
    }
}

fn build_version_record() -> Obj {
    let mut obj = Obj::new();
    obj.set("version", Value::str(VERSION));
    obj.set("name", Value::str(NAME));
    obj.set("description", Value::str(DESCRIPTION));
    obj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_command_name() {
        let cmd = VersionCommand;
        assert_eq!(cmd.name(), "version");
    }

    #[test]
    fn test_version_signature() {
        let cmd = VersionCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "version");
    }

    #[test]
    fn test_build_version_record() {
        let obj = build_version_record();
        assert_eq!(obj.get("version").unwrap(), Value::str("0.1.0"));
        assert_eq!(obj.get("name").unwrap(), Value::str("auto-shell"));
        assert_eq!(
            obj.get("description").unwrap(),
            Value::str("A modern shell environment using AutoLang as the scripting language")
        );
    }

    #[test]
    fn test_version_constants() {
        assert_eq!(VERSION, "0.1.0");
        assert_eq!(NAME, "auto-shell");
    }
}
