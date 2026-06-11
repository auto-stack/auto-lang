//! `http delete` command - HTTP DELETE request
//!
//! Performs an HTTP DELETE request using `curl` as a fallback HTTP client.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};

pub struct HttpDeleteCommand;

impl Command for HttpDeleteCommand {
    fn name(&self) -> &str {
        "http delete"
    }

    fn signature(&self) -> Signature {
        Signature::new("http delete", "Perform an HTTP DELETE request")
            .required("url", "URL to delete")
            .flag("header", "Custom header")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let url = args.first().ok_or_else(|| miette::miette!("http delete: missing URL"))?;
        let output = curl_delete(url, args)?;
        Ok(PipelineData::from_text(output))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let url = args.first().ok_or_else(|| miette::miette!("http delete: missing URL"))?;
        let output = curl_delete(url, args)?;
        Ok(AtomPipeline::from_atom(Atom::new(
            Value::str(&output),
            AtomType::Text,
        )))
    }
}

fn curl_delete(url: &str, args: &ParsedArgs) -> Result<String> {
    let mut cmd = std::process::Command::new("curl");
    cmd.arg("-s").arg("-X").arg("DELETE").arg(url);

    if let Some(header) = args.named.get("header") {
        cmd.arg("-H").arg(header);
    }

    let output = cmd.output().into_diagnostic()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        miette::bail!("http delete: curl failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_delete_command_name() {
        let cmd = HttpDeleteCommand;
        assert_eq!(cmd.name(), "http delete");
    }

    #[test]
    fn test_http_delete_signature() {
        let cmd = HttpDeleteCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "http delete");
    }
}
