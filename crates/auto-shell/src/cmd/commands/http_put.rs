//! `http put` command - HTTP PUT request
//!
//! Performs an HTTP PUT request using `curl` as a fallback HTTP client.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};

pub struct HttpPutCommand;

impl Command for HttpPutCommand {
    fn name(&self) -> &str {
        "http put"
    }

    fn signature(&self) -> Signature {
        Signature::new("http put", "Perform an HTTP PUT request")
            .required("url", "URL to put to")
            .optional("body", "Request body")
            .flag("header", "Custom header")
            .flag("content-type", "Content-Type header (default: application/json)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let url = args.first().ok_or_else(|| miette::miette!("http put: missing URL"))?;
        let output = curl_put(url, args)?;
        Ok(PipelineData::from_text(output))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let url = args.first().ok_or_else(|| miette::miette!("http put: missing URL"))?;
        let output = curl_put(url, args)?;
        Ok(AtomPipeline::from_atom(Atom::new(
            Value::str(&output),
            AtomType::Text,
        )))
    }
}

fn curl_put(url: &str, args: &ParsedArgs) -> Result<String> {
    let mut cmd = std::process::Command::new("curl");
    cmd.arg("-s").arg("-X").arg("PUT").arg(url);

    let content_type = args
        .named
        .get("content-type")
        .map(|s| s.as_str())
        .unwrap_or("application/json");
    cmd.arg("-H").arg(format!("Content-Type: {}", content_type));

    if let Some(header) = args.named.get("header") {
        cmd.arg("-H").arg(header);
    }

    if let Some(body) = args.second() {
        cmd.arg("-d").arg(body);
    }

    let output = cmd.output().into_diagnostic()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        miette::bail!("http put: curl failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_put_command_name() {
        let cmd = HttpPutCommand;
        assert_eq!(cmd.name(), "http put");
    }

    #[test]
    fn test_http_put_signature() {
        let cmd = HttpPutCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "http put");
    }
}
