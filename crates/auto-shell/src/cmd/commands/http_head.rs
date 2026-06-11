//! `http head` command - HTTP HEAD request
//!
//! Performs an HTTP HEAD request using `curl -I` as a fallback.
//! Returns headers as a Record (structured object).

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Obj};
use miette::{IntoDiagnostic, Result};

pub struct HttpHeadCommand;

impl Command for HttpHeadCommand {
    fn name(&self) -> &str {
        "http head"
    }

    fn signature(&self) -> Signature {
        Signature::new("http head", "Perform an HTTP HEAD request and return headers")
            .required("url", "URL to request headers for")
            .flag("header", "Custom header")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let url = args.first().ok_or_else(|| miette::miette!("http head: missing URL"))?;
        let obj = curl_head(url, args)?;
        Ok(PipelineData::from_value(Value::Obj(obj)))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let url = args.first().ok_or_else(|| miette::miette!("http head: missing URL"))?;
        let obj = curl_head(url, args)?;
        Ok(AtomPipeline::from_atom(Atom::new(
            Value::Obj(obj),
            AtomType::Record,
        )))
    }
}

fn curl_head(url: &str, args: &ParsedArgs) -> Result<Obj> {
    let mut cmd = std::process::Command::new("curl");
    cmd.arg("-s").arg("-I").arg(url);

    if let Some(header) = args.named.get("header") {
        cmd.arg("-H").arg(header);
    }

    let output = cmd.output().into_diagnostic()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        miette::bail!("http head: curl failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut obj = Obj::new();

    for line in stdout.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            // Store header name in lowercase for predictable access
            obj.set(key.to_lowercase(), Value::str(value));
        }
    }

    Ok(obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_head_command_name() {
        let cmd = HttpHeadCommand;
        assert_eq!(cmd.name(), "http head");
    }

    #[test]
    fn test_http_head_signature() {
        let cmd = HttpHeadCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "http head");
    }

    #[test]
    fn test_parse_headers() {
        let raw = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 1234\r\n";
        let mut obj = Obj::new();
        for line in raw.lines() {
            if let Some((key, value)) = line.split_once(':') {
                obj.set(key.trim().to_lowercase(), Value::str(value.trim()));
            }
        }
        assert_eq!(obj.get("content-type").unwrap(), Value::str("text/html"));
        assert_eq!(obj.get("content-length").unwrap(), Value::str("1234"));
    }
}
