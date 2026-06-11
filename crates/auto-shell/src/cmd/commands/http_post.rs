//! `http post` command - HTTP POST request
//!
//! Performs an HTTP POST request using `curl` as a fallback HTTP client.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};

pub struct HttpPostCommand;

impl Command for HttpPostCommand {
    fn name(&self) -> &str {
        "http post"
    }

    fn signature(&self) -> Signature {
        Signature::new("http post", "Perform an HTTP POST request")
            .required("url", "URL to post to")
            .optional("body", "Request body")
            .flag("header", "Custom header")
            .flag("content-type", "Content-Type header (default: application/json)")
            .flag("data", "Request body (alternative to positional)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let url = args.first().ok_or_else(|| miette::miette!("http post: missing URL"))?;
        let output = curl_post(url, args)?;
        Ok(PipelineData::from_text(output))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let url = args.first().ok_or_else(|| miette::miette!("http post: missing URL"))?;
        let output = curl_post(url, args)?;
        Ok(AtomPipeline::from_atom(Atom::new(
            Value::str(&output),
            AtomType::Text,
        )))
    }
}

fn curl_post(url: &str, args: &ParsedArgs) -> Result<String> {
    let mut cmd = std::process::Command::new("curl");
    cmd.arg("-s").arg("-X").arg("POST").arg(url);

    let content_type = args
        .named
        .get("content-type")
        .map(|s| s.as_str())
        .unwrap_or("application/json");
    cmd.arg("-H").arg(format!("Content-Type: {}", content_type));

    if let Some(header) = args.named.get("header") {
        cmd.arg("-H").arg(header);
    }

    // Body: --data flag takes priority, then positional arg
    let body = args
        .named
        .get("data")
        .cloned()
        .or_else(|| args.second().map(|s| s.to_string()));
    if let Some(data) = body {
        cmd.arg("-d").arg(data);
    }

    let output = cmd.output().into_diagnostic()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        miette::bail!("http post: curl failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_post_command_name() {
        let cmd = HttpPostCommand;
        assert_eq!(cmd.name(), "http post");
    }

    #[test]
    fn test_http_post_signature() {
        let cmd = HttpPostCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "http post");
    }
}
