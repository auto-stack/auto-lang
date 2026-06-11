//! `sleep` command - Sleep for a duration
//!
//! Pauses execution for the specified duration.
//! Supports suffixes: s (seconds, default), m (minutes), h (hours).

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use miette::Result;

pub struct SleepCommand;

impl Command for SleepCommand {
    fn name(&self) -> &str {
        "sleep"
    }

    fn signature(&self) -> Signature {
        Signature::new("sleep", "Sleep for a duration (s=seconds, m=minutes, h=hours)")
            .required("duration", "Duration to sleep (e.g. 5, 2s, 1m, 1h)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let duration_str = args
            .first()
            .ok_or_else(|| miette::miette!("sleep: missing duration"))?;
        let millis = parse_duration(duration_str)?;
        std::thread::sleep(std::time::Duration::from_millis(millis));
        Ok(PipelineData::empty())
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let duration_str = args
            .first()
            .ok_or_else(|| miette::miette!("sleep: missing duration"))?;
        let millis = parse_duration(duration_str)?;
        std::thread::sleep(std::time::Duration::from_millis(millis));
        Ok(AtomPipeline::empty())
    }
}

/// Parse a duration string like "5", "2s", "1m", "1h" into milliseconds.
/// Supports: s (seconds), m (minutes), h (hours). Default is seconds.
fn parse_duration(input: &str) -> Result<u64> {
    let input = input.trim();

    if input.is_empty() {
        miette::bail!("sleep: empty duration");
    }

    // Split into numeric part and suffix
    let (num_str, suffix) = if let Some(pos) = input.find(|c: char| !c.is_ascii_digit() && c != '.') {
        (&input[..pos], &input[pos..])
    } else {
        (input, "s")
    };

    let num: f64 = num_str
        .parse()
        .map_err(|_| miette::miette!("sleep: invalid number '{}'", num_str))?;

    if num < 0.0 {
        miette::bail!("sleep: duration must be positive");
    }

    let millis = match suffix {
        "s" | "" => (num * 1000.0) as u64,
        "m" => (num * 60.0 * 1000.0) as u64,
        "h" => (num * 3600.0 * 1000.0) as u64,
        _ => miette::bail!("sleep: unknown suffix '{}'. Use s, m, or h", suffix),
    };

    Ok(millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleep_command_name() {
        let cmd = SleepCommand;
        assert_eq!(cmd.name(), "sleep");
    }

    #[test]
    fn test_sleep_signature() {
        let cmd = SleepCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "sleep");
    }

    #[test]
    fn test_parse_seconds() {
        assert_eq!(parse_duration("5").unwrap(), 5000);
        assert_eq!(parse_duration("5s").unwrap(), 5000);
    }

    #[test]
    fn test_parse_minutes() {
        assert_eq!(parse_duration("1m").unwrap(), 60_000);
        assert_eq!(parse_duration("2m").unwrap(), 120_000);
    }

    #[test]
    fn test_parse_hours() {
        assert_eq!(parse_duration("1h").unwrap(), 3_600_000);
    }

    #[test]
    fn test_parse_fractional() {
        assert_eq!(parse_duration("0.5s").unwrap(), 500);
        assert_eq!(parse_duration("1.5m").unwrap(), 90_000);
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("-5s").is_err());
        assert!(parse_duration("5x").is_err());
    }

    #[test]
    fn test_parse_zero() {
        assert_eq!(parse_duration("0").unwrap(), 0);
        assert_eq!(parse_duration("0s").unwrap(), 0);
    }
}
