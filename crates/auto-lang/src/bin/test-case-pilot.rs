//! Explicit developer-only A/B pilot and independent per-case replay.
#[cfg(not(feature = "test-case-batch"))]
fn main() {
    eprintln!("test-case-pilot requires --features test-case-batch");
    std::process::exit(2);
}

#[cfg(feature = "test-case-batch")]
fn main() {
    match enabled::run() {
        Ok((report, code)) => {
            println!("{}", serde_json::to_string(&report).unwrap());
            std::process::exit(code);
        }
        Err(error) => {
            eprintln!("test-case-pilot: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(feature = "test-case-batch")]
mod enabled {
    use auto_lang::test_cases::ac_av::{
        self,
        test_batch::{self, pilot},
    };
    use auto_lang::test_cases::avm::Verdict;
    use auto_test_case::report::Status;
    use serde_json::{json, Value};
    use std::time::{Duration, Instant};

    fn observed(operation: impl FnOnce() -> Result<Value, String>) -> Result<Value, String> {
        let (result, observations) = ac_av::parent_observations::capture(operation);
        let mut report = result?;
        if report["worker_starts"].as_u64() != Some(observations.worker_pids.len() as u64)
            || observations.worker_pids.len() != 1
        {
            return Err(
                "reported worker count differs from actual successful parent spawns".into(),
            );
        }
        if let Some(pid) = report["worker_pid"].as_u64() {
            if pid != observations.worker_pids[0] as u64 {
                return Err("reported worker PID differs from actual spawn".into());
            }
        }
        report["worker_pid"] = json!(observations.worker_pids[0]);
        report["parent_observations"] =
            serde_json::to_value(observations).map_err(|e| e.to_string())?;
        Ok(report)
    }

    fn row(
        bound: &pilot::Bound,
        status: Status,
        verdict: Option<&Verdict>,
        detail: Option<&str>,
    ) -> Value {
        let native = verdict.and_then(|v| v.native.as_ref());
        let receipt_path = bound.root.join("probe-receipt.json");
        // This is the original parent's admitted observation, never an oracle
        // sent to the worker. It retains actual counters and floating spans.
        let receipt = std::fs::read(&receipt_path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
        json!({"case":bound.case.spec.id,"check":bound.check.id,"status":status.as_str(),
            "origin":bound.origin,"source_sha256":bound.case.source_sha256,
            "oracle_sha256":bound.oracle.oracle_sha256,
            "expected":bound.oracle.exit,"actual":native.and_then(|n|n.exit_code),
            "detail":detail,"mismatches":verdict.map(|v|v.mismatches.iter().map(|m|
                json!({"observation":m.observation,"kind":m.kind,"expected":m.expected,"actual":m.actual})
            ).collect::<Vec<_>>()).unwrap_or_default(),
            "receipt":receipt,"receipt_path":receipt_path,
            "replay_argv":[std::env::current_exe().ok(),Some(std::path::PathBuf::from("--mode")),
                Some(std::path::PathBuf::from("independent")),Some(std::path::PathBuf::from("--case")),
                Some(std::path::PathBuf::from(&bound.case.spec.id))]})
    }

    pub fn run() -> Result<(Value, i32), String> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        if args.len() % 2 != 0 {
            return Err("flags require values".into());
        }
        let mut flags = std::collections::BTreeMap::new();
        for pair in args.chunks_exact(2) {
            if ![
                "--mode",
                "--case",
                "--group-size",
                "--repeat",
                "--parallelism",
                "--set",
            ]
            .contains(&pair[0].as_str())
                || flags.insert(pair[0].as_str(), pair[1].as_str()).is_some()
            {
                return Err(format!("unknown or duplicate flag {}", pair[0]));
            }
        }
        // Promotion is limited to this fixed, proven pilot allowlist. A
        // single-case invocation remains the original independent replay.
        let mode = *flags
            .get("--mode")
            .unwrap_or(&if flags.contains_key("--case") {
                "independent"
            } else {
                "group"
            });
        if !["independent", "group", "combined"].contains(&mode) {
            return Err("mode must be independent, group or combined".into());
        }
        let case_set = flags
            .get("--set")
            .copied()
            .unwrap_or(if mode == "combined" { "d3" } else { "pilot15" });
        if !["d3", "pilot15"].contains(&case_set) || (mode == "combined" && case_set != "d3") {
            return Err("set must be d3 or pilot15; combined is limited to d3".into());
        }
        if std::env::var_os("AC_AV_CASE_WORKER")
            .map(std::path::PathBuf::from)
            .filter(|p| p.is_file())
            .is_none()
        {
            return Err(
                "pilot requires an explicitly prebuilt AC_AV_CASE_WORKER; build costs are separate"
                    .into(),
            );
        }
        let number = |name, default| -> Result<usize, String> {
            flags
                .get(name)
                .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
                .unwrap_or(Ok(default))
        };
        let k = number("--group-size", 10)?;
        let repeats = number("--repeat", 1)?;
        let parallelism = number("--parallelism", 1)?;
        if !(1..=10).contains(&k) || !(1..=10).contains(&repeats) || !(1..=2).contains(&parallelism)
        {
            return Err("group-size 1..10, repeat 1..10 and parallelism 1..2 required".into());
        }
        let selected = flags.get("--case").copied();
        let started = Instant::now();
        let mut input_load_ms = 0.0;
        let mut rounds = Vec::new();
        let mut code = 0;
        if mode == "combined" {
            if selected.is_some() {
                return Err(
                    "C requires all D3 expressions; replay a single case with independent mode"
                        .into(),
                );
            }
            let mut invocations = Vec::new();
            for repeat in 0..repeats {
                let loading = Instant::now();
                let mut values = pilot::load(&format!("cli-c-{repeat}"))?;
                input_load_ms += loading.elapsed().as_secs_f64() * 1000.0;
                values.retain(|b| b.case.spec.id.starts_with("gov.d."));
                invocations.push(values);
            }
            for wave in invocations.chunks(parallelism) {
                let reports = std::thread::scope(|scope| {
                    let handles: Vec<_> = wave
                        .iter()
                        .map(|values| {
                            scope.spawn(|| {
                                observed(|| {
                                    test_batch::combined::run(values, Duration::from_secs(60))
                                })
                            })
                        })
                        .collect();
                    handles
                        .into_iter()
                        .map(|h| h.join().map_err(|_| "combined task panicked".to_owned())?)
                        .collect::<Result<Vec<_>, String>>()
                })?;
                for mut report in reports {
                    report["repeat"] = json!(rounds.len());
                    for item in report["items"].as_array_mut().ok_or("missing C items")? {
                        item["replay_argv"][0] =
                            json!(std::env::current_exe().map_err(|e| e.to_string())?);
                    }
                    for item in report["items"].as_array().ok_or("missing C items")? {
                        if item["status"] != "PASS" {
                            code = 1;
                        }
                    }
                    rounds.push(report);
                }
            }
            return Ok((
                json!({"schema":"test-case-pilot/1","mode":mode,"set":case_set,
                "parallelism":parallelism,"group_size":3,"repeat":repeats,
                "wall_ms":started.elapsed().as_secs_f64()*1000.0,"input_load_ms":input_load_ms,"parent_pid":std::process::id(),"groups":rounds,"exit":code}),
                code,
            ));
        }
        for repeat in 0..repeats {
            let loading = Instant::now();
            let mut bindings = pilot::load(&format!("cli-{repeat}"))?;
            input_load_ms += loading.elapsed().as_secs_f64() * 1000.0;
            if case_set == "d3" {
                bindings.retain(|b| b.case.spec.id.starts_with("gov.d."));
            }
            if let Some(id) = selected {
                bindings.retain(|b| b.case.spec.id == id);
            }
            if bindings.is_empty() {
                return Err("case filter selected zero original obligations".into());
            }
            let chunks: Vec<_> = bindings
                .chunks(if mode == "independent" { 1 } else { k })
                .collect();
            for wave in chunks.chunks(parallelism) {
                let reports = std::thread::scope(|scope| {
                    let handles: Vec<_> = wave.iter().map(|chunk|scope.spawn(move || observed(|| {
                        let begin = Instant::now();
                        if mode=="independent" {
                            let b = &chunk[0];
                            let verdict = ac_av::run_check(&b.case,&b.check,&b.oracle,&b.root,&b.root,&b.root)?;
                            return Ok(json!({"repeat":repeat,"mode":mode,"worker_starts":1,
                                "wall_ms":begin.elapsed().as_secs_f64()*1000.0,
                                "items":[row(b,verdict.status,Some(&verdict),verdict.detail.as_deref())]}));
                        }
                        let items = pilot::items(chunk);
                        let report = test_batch::run_group(&items,Duration::from_secs(60))?;
                        Ok(json!({"repeat":repeat,"mode":mode,"group":report.group,
                            "worker_starts":report.worker_starts,"worker_pid":report.worker_pid,
                            "wall_ms":report.wall_ms,"error":report.error,"cleanup_complete":report.cleanup_complete,
                            "worker_sha256":report.worker_exe_sha256,
                            "items":chunk.iter().zip(&report.items).map(|(b,r)|row(b,r.status,r.verdict.as_ref(),r.detail.as_deref())).collect::<Vec<_>>()}))
                    }))).collect();
                    handles
                        .into_iter()
                        .map(|h| h.join().map_err(|_| "pilot task panicked".to_owned())?)
                        .collect::<Result<Vec<_>, String>>()
                })?;
                for report in reports {
                    if !report["error"].is_null() {
                        code = 2;
                    }
                    for item in report["items"].as_array().ok_or("missing item report")? {
                        match item["status"].as_str() {
                            Some("PASS") => (),
                            Some("FAIL") => code = code.max(1),
                            _ => code = 2,
                        }
                        if item["status"] == "PASS" && item["receipt"].is_null() {
                            code = 2;
                        }
                    }
                    rounds.push(report);
                }
            }
        }
        Ok((
            json!({"schema":"test-case-pilot/1","mode":mode,"set":case_set,"parallelism":parallelism,
            "group_size":if mode=="independent" {1}else{k},"repeat":repeats,
            "wall_ms":started.elapsed().as_secs_f64()*1000.0,"input_load_ms":input_load_ms,"parent_pid":std::process::id(),"groups":rounds,"exit":code}),
            code,
        ))
    }
}
