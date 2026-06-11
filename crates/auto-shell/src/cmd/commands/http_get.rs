//! `http get` command - HTTP GET request
//!
//! Performs an HTTP GET request using `curl` as a fallback HTTP client.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};

pub struct HttpGetCommand;

impl Command for HttpGetCommand {
    fn name(&self) -> &str {
        "http get"
    }

    fn signature(&self) -> Signature {
        Signature::new("http get", "Perform an HTTP GET request")
            .required("url", "URL to request")
            .flag("header", "Custom header (can be repeated)")
            .flag("timeout", "Timeout in seconds")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let url = args.first().ok_or_else(|| miette::miette!("http get: missing URL"))?;
        let output = curl_get(url, args)?;
        Ok(PipelineData::from_text(output))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let url = args.first().ok_or_else(|| miette::miette!("http get: missing URL"))?;
        let output = curl_get(url, args)?;
        Ok(AtomPipeline::from_atom(Atom::new(
            Value::str(&output),
            AtomType::Text,
        )))
    }
}

fn curl_get(url: &str, args: &ParsedArgs) -> Result<String> {
    let mut cmd = std::process::Command::new("curl");
    cmd.arg("-s").arg("-X").arg("GET").arg(url);

    if let Some(header) = args.named.get("header") {
        cmd.arg("-H").arg(header);
    }

    if let Some(timeout) = args.named.get("timeout") {
        cmd.arg("--max-time").arg(timeout);
    }

    let output = cmd.output().into_diagnostic()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        miette::bail!("http get: curl failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_get_command_name() {
        let cmd = HttpGetCommand;
        assert_eq!(cmd.name(), "http get");
    }

    #[test]
    fn test_http_get_signature() {
        let cmd = HttpGetCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "http get");
    }
}
