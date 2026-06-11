//! `date` command - Display current date and time
//!
//! Shows the current date/time with configurable formatting.
//! Supports local time, UTC, and unix timestamp output.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use chrono::{Local, Utc, TimeZone};
use miette::Result;

pub struct DateCommand;

impl Command for DateCommand {
    fn name(&self) -> &str {
        "date"
    }

    fn signature(&self) -> Signature {
        Signature::new("date", "Display current date and time")
            .flag("format", "strftime format string (default: \"%Y-%m-%d %H:%M:%S\")")
            .flag("utc", "Use UTC time instead of local")
            .flag("unix", "Show unix timestamp")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let value = build_date_value(args);
        Ok(PipelineData::from_value(value))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let value = build_date_value(args);
        Ok(AtomPipeline::from_atom(Atom::new(value, AtomType::Record)))
    }
}

fn build_date_value(args: &ParsedArgs) -> Value {
    if args.has_flag("unix") {
        let ts = if args.has_flag("utc") {
            Utc::now().timestamp()
        } else {
            Local::now().timestamp()
        };
        return Value::I64(ts);
    }

    let fmt = args
        .named
        .get("format")
        .map(|s| s.as_str())
        .unwrap_or("%Y-%m-%d %H:%M:%S");

    let formatted = if args.has_flag("utc") {
        Utc::now().format(fmt).to_string()
    } else {
        Local::now().format(fmt).to_string()
    };

    Value::str(&formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_command_name() {
        let cmd = DateCommand;
        assert_eq!(cmd.name(), "date");
    }

    #[test]
    fn test_date_signature() {
        let cmd = DateCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "date");
    }

    #[test]
    fn test_date_default_format() {
        let now = Local::now();
        let formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
        // Verify format pattern: YYYY-MM-DD HH:MM:SS
        assert_eq!(formatted.len(), 19);
        assert_eq!(&formatted[4..5], "-");
        assert_eq!(&formatted[7..8], "-");
        assert_eq!(&formatted[10..11], " ");
        assert_eq!(&formatted[13..14], ":");
        assert_eq!(&formatted[16..17], ":");
    }

    #[test]
    fn test_date_utc_vs_local() {
        let local_ts = Local::now().timestamp();
        let utc_ts = Utc::now().timestamp();
        // Should be within 2 seconds of each other
        assert!((local_ts - utc_ts).abs() <= 2);
    }

    #[test]
    fn test_date_custom_format() {
        let now = Local::now();
        let formatted = now.format("%Y/%m/%d").to_string();
        assert_eq!(formatted.len(), 10);
        assert_eq!(&formatted[4..5], "/");
    }

    #[test]
    fn test_unix_timestamp_is_i64() {
        let ts = Local::now().timestamp();
        assert!(ts > 1_700_000_000); // Sometime after 2023
    }
}
