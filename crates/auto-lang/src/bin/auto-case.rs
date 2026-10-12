//! `auto-case` — developer entry for the AutoDown TestCase protocol pilot
//! (Plan 768 T-07; proposed interface from the plan §6.2).
//!
//! Only meaningful with `--features test-cases` (without it this binary is
//! an empty stub). The runner exit code is the GATE code — 0 selected gate
//! satisfied, 1 assertion/coverage failure, 2 contract/infrastructure error
//! — and never replaces any tested program's own exit code.
//!
//! ```text
//! auto-case lint     --suite <pilot.suite.ad>
//! auto-case list     --suite <pilot.suite.ad> [--format console|json]
//! auto-case inspect  --suite <pilot.suite.ad> --case <case-id>
//! auto-case run      --suite <pilot.suite.ad> --out <dir>
//!                    [--case <id>] [--adapter avm|a2r] [--check <id>]
//!                    [--format console|json] [--json-out <file>]
//! auto-case proposal --suite <pilot.suite.ad> --out <dir> [--proposal-dir <dir>]
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
use std::path::PathBuf;

#[cfg(feature = "test-cases")]
use auto_test_case::admission::Limits;
#[cfg(feature = "test-cases")]
use auto_test_case::discovery::load_suite;
#[cfg(feature = "test-cases")]
use auto_test_case::report::{RunReport, Status};
#[cfg(feature = "test-cases")]
use auto_test_case::suite::apply_defaults;

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

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

fn run(args: &[String]) -> Result<i32, String> {
    let cmd = args.first().ok_or_else(|| usage())?;
    let suite = PathBuf::from(require(args, "--suite")?);
    match cmd.as_str() {
        "lint" => lint(&suite),
        "list" => list(&suite, flag(args, "--format").as_deref()),
        "inspect" => inspect(&suite, require(args, "--case")?),
        "run" => run_cmd(&suite, args),
        "proposal" => proposal_cmd(&suite, args),
        other => Err(format!("unknown command {:?}\n{}", other, usage())),
    }
}

fn usage() -> String {
    "usage: auto-case <lint|list|inspect|run|proposal> --suite <path> [flags]".to_string()
}

fn lint(suite: &std::path::Path) -> Result<i32, String> {
    let loaded = load_suite(suite, &Limits::defaults()).map_err(|e| e.to_string())?;
    let total: usize = loaded.cases.iter().map(|c| c.specs.len()).sum();
    println!(
        "suite ok: {} case file(s), {} case(s)",
        loaded.cases.len(),
        total
    );
    for e in &loaded.errors {
        println!("ERROR {}", e);
    }
    if loaded.errors.is_empty() {
        Ok(0)
    } else {
        Ok(2)
    }
}

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

fn inspect(suite: &std::path::Path, case_id: &str) -> Result<i32, String> {
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
            return Ok(0);
        }
    }
    Err(format!("case {:?} not found in suite", case_id))
}

fn fmt_entry(spec: &auto_test_case::contract::CaseSpec) -> String {
    use auto_test_case::contract::EntryMode;
    match spec.entry {
        EntryMode::Script => "script".into(),
        EntryMode::Expression => "expression".into(),
        EntryMode::Program => "program".into(),
        EntryMode::Module => "module".into(),
    }
}

fn fmt_source(spec: &auto_test_case::contract::CaseSpec) -> String {
    use auto_test_case::contract::PayloadSpec;
    match &spec.source.payload {
        PayloadSpec::Block(a) => format!("block:{}", a),
        PayloadSpec::File(f) => format!("file:{}", f),
        PayloadSpec::Text(_) => "text:<inline>".to_string(),
    }
}

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

fn run_cmd(suite: &std::path::Path, args: &[String]) -> Result<i32, String> {
    let out = PathBuf::from(require(args, "--out")?);
    let format = flag(args, "--format");
    let mut report = auto_lang::test_cases::run_suite(suite, &out, &Limits::defaults())?;

    // Selectors mark unselected checks NOT_RUN — a partial report never
    // counts as a full gate pass (required coverage still judged against
    // every declared check).
    if let Some(case) = flag(args, "--case") {
        mark_unselected(
            &mut report,
            |r| r.case_id == case,
            &format!("--case {}", case),
        );
    }
    if let Some(adapter) = flag(args, "--adapter") {
        mark_unselected(
            &mut report,
            |r| r.adapter == adapter,
            &format!("--adapter {}", adapter),
        );
    }
    if let Some(check) = flag(args, "--check") {
        mark_unselected(
            &mut report,
            |r| r.check_id == check,
            &format!("--check {}", check),
        );
    }

    let text = match format.as_deref() {
        Some("json") => report.to_json(),
        _ => report.to_console(),
    };
    println!("{}", text);
    if let Some(file) = flag(args, "--json-out") {
        std::fs::write(&file, report.to_json()).map_err(|e| format!("json-out: {}", e))?;
    }
    // Persist the canonical report next to the run scratch.
    let _ = std::fs::write(out.join("report.console.txt"), report.to_console());
    let _ = std::fs::write(out.join("report.json"), report.to_json());
    Ok(report.exit_code())
}

fn mark_unselected(
    report: &mut RunReport,
    keep: impl Fn(&auto_test_case::report::CheckOutcome) -> bool,
    why: &str,
) {
    for r in report.results.iter_mut() {
        if !keep(r) && r.status != Status::NotRun {
            r.status = Status::NotRun;
            r.detail = Some(format!("not selected ({})", why));
            r.mismatches.clear();
        }
    }
}

fn proposal_cmd(suite: &std::path::Path, args: &[String]) -> Result<i32, String> {
    let out = PathBuf::from(require(args, "--out")?);
    let proposal_dir = flag(args, "--proposal-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| out.join("proposals"));
    let report = auto_lang::test_cases::run_suite(suite, &out, &Limits::defaults())?;
    let proposals = auto_lang::test_cases::materialize::collect_proposals(&report);
    if proposals.is_empty() {
        println!("no proposals — nothing failed an artifact oracle");
        return Ok(report.exit_code());
    }
    let n = auto_lang::test_cases::materialize::write_proposals(&proposals, &proposal_dir)?;
    println!(
        "{} proposal(s) written under {} — candidates only; accepting is a separate reviewed action",
        n,
        proposal_dir.display()
    );
    Ok(report.exit_code())
}

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
