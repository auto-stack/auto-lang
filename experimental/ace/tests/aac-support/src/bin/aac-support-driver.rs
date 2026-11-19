//! PLAN-1056 real stage-driver entry: the Gen0 (Rust-frontend) toolchain
//! compiles the frozen Auto support package (`experimental/ace/aac/support/`)
//! and the AAC support algorithms run on the AV executor. Each case prints
//! `case\t<observation>`; the observation is the byte-exact owned-Str result
//! (hex digest / framed bytes / outcome-code sequence), so evidence lines
//! carry real exit codes and real values instead of a pass/fail proxy.
//!
//! Usage:
//!   aac-support-driver --list
//!   aac-support-driver --case <name>
//!   aac-support-driver --all

use ace_aac_support_conformance::{run_program, str_bytes};
use std::process::ExitCode;

/// One frozen driver case: `(name, driver.at function, optional argument)`.
const CASES: &[(&str, &str, &str)] = &[
    ("sha256:empty", "sha256_case", "empty"),
    ("sha256:abc", "sha256_case", "abc"),
    ("sha256:nul", "sha256_case", "nul"),
    ("sha256:utf8", "sha256_case", "utf8"),
    ("sha256:block64", "sha256_case", "block64"),
    ("sha256:split", "sha256_case", "split"),
    ("blake3:empty", "blake3_case", "empty"),
    ("blake3:abc", "blake3_case", "abc"),
    ("blake3:nul", "blake3_case", "nul"),
    ("blake3:utf8", "blake3_case", "utf8"),
    ("blake3:chunk1024", "blake3_case", "chunk1024"),
    ("blake3:chunk1025", "blake3_case", "chunk1025"),
    ("identity:snapshot", "identity_case", "snapshot"),
    ("identity:artifact", "identity_case", "artifact"),
    ("identity:family-mismatch", "identity_case", "family-mismatch"),
    ("manifest:encode-unsorted", "manifest_case", "encode-unsorted"),
    ("manifest:encode-sorted", "manifest_case", "encode-sorted"),
    ("manifest:reencode-decoded", "manifest_case", "reencode-decoded"),
    ("manifest:anchor-min", "manifest_case", "anchor-min"),
    ("manifest:anchor-max", "manifest_case", "anchor-max"),
    (
        "manifest:anchor-min-roundtrip",
        "manifest_case",
        "anchor-min-roundtrip",
    ),
    ("frame:report", "frame_report", ""),
    ("frame:taxonomy", "frame_case_codes", ""),
    ("manifest:taxonomy", "manifest_codes", ""),
    ("admission:taxonomy", "admission_case_codes", ""),
];

fn program(func: &str, arg: &str) -> String {
    if arg.is_empty() {
        format!("use app.driver.{func}\nfn main() Str {{ return {func}() }}")
    } else {
        format!("use app.driver.{func}\nfn main() Str {{ return {func}(\"{arg}\") }}")
    }
}

fn run_case(name: &str, func: &str, arg: &str) -> Result<String, String> {
    let value = run_program(name, &program(func, arg))?;
    let bytes = str_bytes(&value)?;
    String::from_utf8(bytes).map_err(|e| format!("non-utf8 observation: {e}"))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--list") => {
            for (name, _, _) in CASES {
                println!("{name}");
            }
            ExitCode::SUCCESS
        }
        Some("--all") => {
            let mut failures = 0;
            for (name, func, arg) in CASES {
                match run_case(name, func, arg) {
                    Ok(obs) => println!("{name}\t{obs}"),
                    Err(e) => {
                        failures += 1;
                        println!("{name}\terror:{}", e.replace('\n', " "));
                    }
                }
            }
            if failures == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("--case") => match args.get(1) {
            None => {
                eprintln!("--case needs a case name");
                ExitCode::from(2)
            }
            Some(want) => match CASES.iter().find(|(name, _, _)| name == want) {
                None => {
                    eprintln!("unknown case `{want}`");
                    ExitCode::from(2)
                }
                Some((name, func, arg)) => match run_case(name, func, arg) {
                    Ok(obs) => {
                        println!("{name}\t{obs}");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("{name}\terror:{e}");
                        ExitCode::from(1)
                    }
                },
            },
        },
        _ => {
            eprintln!("usage: aac-support-driver --list | --case <name> | --all");
            ExitCode::from(2)
        }
    }
}
