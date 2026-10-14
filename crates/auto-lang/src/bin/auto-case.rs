//! `auto-case` — developer entry for the AutoDown TestCase protocol pilot
//! (Plan 768 T-07; proposed interface from the plan §6.2).
//!
//! Only meaningful with `--features test-cases` (without it this binary is
//! an empty stub). The runner exit code is the GATE code — 0 selected gate
//! satisfied, 1 assertion/coverage failure, 2 contract/infrastructure error
//! — and never replaces any tested program's own exit code.
//!
//! Plan 773 wiring: selectors validate and apply BEFORE execution (F-04);
//! every write port (`--out`, `--json-out`, `--proposal-dir`) is
//! path-domain audited against the suite/fixture roots BEFORE anything
//! runs (F-05); lint/inspect run the full reference admission shared with
//! the runner (F-09); proposals bind old/new identity with a real diff and
//! persist the run report (F-11); `stale` re-checks an old report's
//! identity axes against the current tree (F-06).
//!
//! ```text
//! auto-case lint     --suite <pilot.suite.ad>
//! auto-case list     --suite <pilot.suite.ad> [--format console|json]
//! auto-case inspect  --suite <pilot.suite.ad> --case <case-id>
//! auto-case run      --suite <pilot.suite.ad> --out <dir>
//!                    [--case <id>] [--adapter avm|a2r] [--check <id>]
//!                    [--format console|json] [--json-out <file>]
//! auto-case proposal --suite <pilot.suite.ad> --out <dir> [--proposal-dir <dir>]
//! auto-case stale    --suite <pilot.suite.ad> --report <old report.json>
//! ```

#[cfg(not(feature = "test-cases"))]
fn main() {
    eprintln!("auto-case: this binary needs `--features test-cases` (empty stub build)");
}

#[cfg(feature = "test-cases")]
fn main() {
    let code = match real_main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("auto-case: {}", e);
            2
        }
    };
    std::process::exit(code);
}

#[cfg(feature = "test-cases")]
fn real_main() -> Result<i32, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    run(&args)
}

#[cfg(feature = "test-cases")]
use std::path::{Path, PathBuf};

#[cfg(feature = "test-cases")]
use auto_test_case::admission::{
    check_no_reparse_ancestors, validate_output_domain, validate_output_file, Limits,
};
#[cfg(feature = "test-cases")]
use auto_test_case::discovery::load_suite;
#[cfg(feature = "test-cases")]
use auto_test_case::suite::apply_defaults;

#[cfg(feature = "test-cases")]
fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

#[cfg(feature = "test-cases")]
fn require<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    match args
        .iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
    {
        Some(v) => Ok(v.as_str()),
        None => Err(format!("missing required flag {}", name)),
    }
}

#[cfg(feature = "test-cases")]
fn run(args: &[String]) -> Result<i32, String> {
    let cmd = args.first().ok_or_else(|| usage())?;
    let suite = PathBuf::from(require(args, "--suite")?);
    match cmd.as_str() {
        "lint" => lint(&suite),
        "list" => list(&suite, flag(args, "--format").as_deref()),
        "inspect" => inspect(&suite, require(args, "--case")?),
        "run" => run_cmd(&suite, args),
        "proposal" => proposal_cmd(&suite, args),
        "stale" => stale_cmd(&suite, require(args, "--report")?),
        other => Err(format!("unknown command {:?}\n{}", other, usage())),
    }
}

#[cfg(feature = "test-cases")]
fn usage() -> String {
    "usage: auto-case <lint|list|inspect|run|proposal|stale> --suite <path> [flags]".to_string()
}

/// Protected input/oracle roots for the suite: its directory and the
/// fixture base every `file:` reference resolves against.
#[cfg(feature = "test-cases")]
fn protected_roots(suite: &Path) -> Result<Vec<PathBuf>, String> {
    let loaded = load_suite(suite, &Limits::defaults()).map_err(|e| e.to_string())?;
    let mut roots = Vec::new();
    if let Some(parent) = loaded.suite_path.parent() {
        roots.push(parent.to_path_buf());
    }
    roots.push(loaded.fixture_base.clone());
    Ok(roots)
}

/// Audit one output directory (created first so canonicalization resolves
/// the real Windows form); on rejection, remove the directory only if this
/// call created it empty (F-05).
#[cfg(feature = "test-cases")]
fn audit_output_dir(out: &Path, protected: &[PathBuf]) -> Result<(), String> {
    let existed = out.exists();
    std::fs::create_dir_all(out).map_err(|e| format!("create {}: {}", out.display(), e))?;
    check_no_reparse_ancestors(out)
        .and_then(|()| validate_output_domain(out, protected))
        .map_err(|e| {
            if !existed {
                let _ = std::fs::remove_dir(out);
            }
            format!("output path rejected: {}", e)
        })
}

/// Audit a single output FILE target: parent must exist (created on
/// demand), no reparse point on the chain, and the file must stay outside
/// the protected roots (F-05).
#[cfg(feature = "test-cases")]
fn audit_output_file(target: &Path, protected: &[PathBuf]) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {}", parent.display(), e))?;
    }
    check_no_reparse_ancestors(target)
        .and_then(|()| validate_output_file(target, protected))
        .map_err(|e| format!("output path rejected: {}", e))
}

#[cfg(feature = "test-cases")]
fn lint(suite: &std::path::Path) -> Result<i32, String> {
    // F-09: full admission — references (payload sidecars AND expected
    // artifacts) resolve before lint can claim the contract is sound.
    let errors = auto_lang::test_cases::materialize::lint_resolve(suite, &Limits::defaults())?;
    let loaded = load_suite(suite, &Limits::defaults()).map_err(|e| e.to_string())?;
    let total: usize = loaded.cases.iter().map(|c| c.specs.len()).sum();
    println!(
        "suite ok: {} case file(s), {} case(s), full reference admission {}",
        loaded.cases.len(),
        total,
        if errors.is_empty() { "passed" } else { "FAILED" }
    );
    for e in &errors {
        println!("ERROR {}", e);
    }
    if errors.is_empty() {
        Ok(0)
    } else {
        Ok(2)
    }
}

#[cfg(feature = "test-cases")]
fn list(suite: &std::path::Path, format: Option<&str>) -> Result<i32, String> {
    let loaded = load_suite(suite, &Limits::defaults()).map_err(|e| e.to_string())?;
    let json = matches!(format, Some("json"));
    if !json {
        println!("suite {} (sha256 {})", loaded.suite.id, loaded.suite_sha256);
    }
    let mut lines: Vec<String> = Vec::new();
    for file in &loaded.cases {
        for spec in &file.specs {
            for check in &spec.checks {
                let (effective, origins) = apply_defaults(check, &loaded.suite.defaults);
                lines.push(format!(
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    spec.id,
                    check.id,
                    effective.adapter.as_str(),
                    effective.action.as_str(),
                    effective.tier.as_deref().unwrap_or(""),
                    if check.skip.is_some() || spec.status_skip.is_some() {
                        "SKIP"
                    } else {
                        "RUN"
                    },
                ));
                let _ = origins;
            }
        }
    }
    if json {
        let arr: Vec<String> = lines
            .iter()
            .map(|l| {
                let c: Vec<&str> = l.split('\t').collect();
                format!(
                    "{{ \"case\": {}, \"check\": {}, \"adapter\": {}, \"action\": {}, \"tier\": {}, \"state\": {} }}",
                    jstr(c[0]),
                    jstr(c[1]),
                    jstr(c[2]),
                    jstr(c[3]),
                    if c[4].is_empty() { "null".to_string() } else { jstr(c[4]) },
                    jstr(c[5])
                )
            })
            .collect();
        println!("[{}]", arr.join(",\n"));
    } else {
        for l in lines {
            println!("{}", l);
        }
    }
    for e in &loaded.errors {
        eprintln!("ERROR {}", e);
    }
    if loaded.errors.is_empty() {
        Ok(0)
    } else {
        Ok(2)
    }
}

#[cfg(feature = "test-cases")]
fn inspect(suite: &std::path::Path, case_id: &str) -> Result<i32, String> {
    // F-09: inspect shares the full admission — a broken reference is
    // reported here too, not only at run time.
    let errors = auto_lang::test_cases::materialize::lint_resolve(suite, &Limits::defaults())?;
    let loaded = load_suite(suite, &Limits::defaults()).map_err(|e| e.to_string())?;
    for file in &loaded.cases {
        for spec in &file.specs {
            if spec.id != case_id {
                continue;
            }
            println!("case {} ({})", spec.id, file.path.display());
            println!(
                "  entry: {:?}, source: {:?}, files: {}",
                fmt_entry(&spec),
                fmt_source(&spec),
                spec.files.len()
            );
            if let Some(skip) = &spec.status_skip {
                println!("  case-level skip: {} (issue {})", skip.reason, skip.issue);
            }
            for check in &spec.checks {
                let (effective, origins) = apply_defaults(check, &loaded.suite.defaults);
                println!(
                    "  check {} [{}::{}] tier={} skip={}",
                    effective.id,
                    effective.adapter.as_str(),
                    effective.action.as_str(),
                    effective.tier.as_deref().unwrap_or("-"),
                    effective.skip.is_some()
                );
                for o in &origins {
                    println!("    field {} <- {}", o.field, o.source);
                }
                println!(
                    "    completion: {}",
                    fmt_completion(&effective.expect.completion)
                );
                for a in &effective.expect.artifacts {
                    println!("    artifact {} <- {} ({:?})", a.path, a.file, a.compare);
                }
            }
            for e in &errors {
                if e.contains(&file.path.display().to_string()) {
                    println!("ERROR {}", e);
                }
            }
            return Ok(0);
        }
    }
    Err(format!("case {:?} not found in suite", case_id))
}

#[cfg(feature = "test-cases")]
fn fmt_entry(spec: &auto_test_case::contract::CaseSpec) -> String {
    use auto_test_case::contract::EntryMode;
    match spec.entry {
        EntryMode::Script => "script".into(),
        EntryMode::Expression => "expression".into(),
        EntryMode::Program => "program".into(),
        EntryMode::Module => "module".into(),
    }
}

#[cfg(feature = "test-cases")]
fn fmt_source(spec: &auto_test_case::contract::CaseSpec) -> String {
    use auto_test_case::contract::PayloadSpec;
    match &spec.source.payload {
        PayloadSpec::Block(a) => format!("block:{}", a),
        PayloadSpec::File(f) => format!("file:{}", f),
        PayloadSpec::Text(_) => "text:<inline>".to_string(),
    }
}

#[cfg(feature = "test-cases")]
fn fmt_completion(c: &auto_test_case::contract::Completion) -> String {
    match c {
        auto_test_case::contract::Completion::Success => "success".to_string(),
        auto_test_case::contract::Completion::Error {
            stage,
            code,
            message,
        } => format!(
            "error@{} code={} message={}",
            stage.as_str(),
            code.as_deref().unwrap_or("-"),
            match message {
                Some(auto_test_case::contract::MessageMatcher::Exact(s)) =>
                    format!("exact {:?}", s),
                Some(auto_test_case::contract::MessageMatcher::Contains(s)) =>
                    format!("contains {:?}", s),
                None => "-".to_string(),
            }
        ),
    }
}

#[cfg(feature = "test-cases")]
fn run_cmd(suite: &std::path::Path, args: &[String]) -> Result<i32, String> {
    let out = PathBuf::from(require(args, "--out")?);
    let format = flag(args, "--format");
    let protected = protected_roots(suite)?;

    // F-05: every write port is audited BEFORE execution.
    audit_output_dir(&out, &protected)?;
    if let Some(file) = flag(args, "--json-out") {
        let file = PathBuf::from(file);
        audit_output_file(&file, &protected)?;
    }

    // F-04: the selection is parsed, validated, and applied INSIDE the
    // runner — unselected checks never execute; typo/zero-match selectors
    // are contract errors before any adapter call.
    let selection = auto_lang::test_cases::Selection {
        case: flag(args, "--case"),
        adapter: flag(args, "--adapter"),
        check: flag(args, "--check"),
    };
    let mut report =
        auto_lang::test_cases::run_suite_selected(suite, &out, &Limits::defaults(), &selection)?;

    let text = match format.as_deref() {
        Some("json") => report.to_json(),
        _ => report.to_console(),
    };
    println!("{}", text);
    if let Some(file) = flag(args, "--json-out") {
        std::fs::write(&file, report.to_json()).map_err(|e| format!("json-out: {}", e))?;
    }
    // Persist the canonical report next to the run scratch. Failure to save
    // either format is an infrastructure error, never a green run.
    persist_reports(&out, &report.to_console(), &report.to_json())?;
    let _ = &mut report;
    Ok(report.exit_code())
}

#[cfg(feature = "test-cases")]
fn persist_reports(out: &Path, console: &str, json: &str) -> Result<(), String> {
    std::fs::write(out.join("report.console.txt"), console)
        .map_err(|e| format!("write report.console.txt: {}", e))?;
    std::fs::write(out.join("report.json"), json)
        .map_err(|e| format!("write report.json: {}", e))?;
    Ok(())
}

#[cfg(feature = "test-cases")]
fn proposal_cmd(suite: &std::path::Path, args: &[String]) -> Result<i32, String> {
    let out = PathBuf::from(require(args, "--out")?);
    let proposal_dir = flag(args, "--proposal-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| out.join("proposals"));
    let protected = protected_roots(suite)?;

    // F-05: both write ports audited before execution.
    audit_output_dir(&out, &protected)?;
    audit_output_dir(&proposal_dir, &protected)?;

    let report = auto_lang::test_cases::run_suite(suite, &out, &Limits::defaults())?;
    // F-11: the run report is ALWAYS persisted — a proposal without its
    // source report is not reviewable evidence.
    persist_reports(&out, &report.to_console(), &report.to_json())?;
    let report_ref = out.join("report.json").display().to_string();

    let proposals = auto_lang::test_cases::materialize::collect_proposals(
        &report,
        suite,
        &Limits::defaults(),
    )?;
    if proposals.is_empty() {
        println!("no proposals — nothing failed an artifact oracle");
        return Ok(report.exit_code());
    }
    let n = auto_lang::test_cases::materialize::write_proposals(
        &proposals,
        &proposal_dir,
        Some(Path::new(&report_ref)),
    )?;
    println!(
        "{} proposal(s) written under {} — candidates only (old/new sha256 + diff + shared consumers in PROPOSALS.md); accepting is a separate reviewed action",
        n,
        proposal_dir.display()
    );
    Ok(report.exit_code())
}

/// F-06: re-check an old report against the CURRENT tree via the library
/// staleness check (`test_cases::stale`). Exit 0 = all fresh, 1 = stale
/// findings, 2 = errors.
#[cfg(feature = "test-cases")]
fn stale_cmd(suite: &std::path::Path, report_path: &str) -> Result<i32, String> {
    let raw = std::fs::read_to_string(report_path)
        .map_err(|e| format!("reading report {}: {}", report_path, e))?;
    let old: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("parsing report {}: {}", report_path, e))?;
    let verdicts = auto_lang::test_cases::stale::check(suite, &old)?;
    let mut stale_count = 0usize;
    for v in &verdicts {
        match v {
            auto_lang::test_cases::stale::RowVerdict::Fresh { case, check } => {
                println!("FRESH {}:{}", case, check)
            }
            auto_lang::test_cases::stale::RowVerdict::Stale {
                case,
                check,
                drifted_axes,
            } => {
                println!("STALE {}:{} — {}", case, check, drifted_axes.join(", "));
                stale_count += 1;
            }
        }
    }
    println!(
        "{} result(s) checked, {} stale; axes: suite/case_file/source/oracle/contract + toolchain",
        verdicts.len(),
        stale_count
    );
    if stale_count == 0 {
        Ok(0)
    } else {
        Ok(1)
    }
}

#[cfg(feature = "test-cases")]
fn jstr(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(all(test, feature = "test-cases"))]
mod tests {
    use super::persist_reports;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn report_write_failure_is_reported() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let out = std::env::temp_dir().join(format!(
            "auto-case-report-write-{}-{}",
            std::process::id(),
            nonce
        ));
        assert!(!out.exists());
        std::fs::create_dir_all(&out).unwrap();
        std::fs::create_dir(out.join("report.console.txt")).unwrap();

        let error = persist_reports(&out, "console", "json").unwrap_err();
        assert!(error.contains("report.console.txt"), "{error}");

        std::fs::remove_dir_all(out).unwrap();
    }
}
