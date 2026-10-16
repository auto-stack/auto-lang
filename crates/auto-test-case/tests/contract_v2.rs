//! PLAN-788 v2 contract surface (V-01): strict version dispatch, the ac/
//! syntax adapter face, typed stages, budgets, native channels — and the
//! byte-identical v1 accept/reject sets.

use auto_test_case::admission::Limits;
use auto_test_case::atom::parse_contract;
use auto_test_case::contract::{decode_case, AcProfile, CacheMode, CASE_SCHEMA, CASE_SCHEMA_V2};
use auto_test_case::suite::{apply_defaults, decode_suite};

fn decode(
    src: &str,
) -> Result<auto_test_case::contract::CaseSpec, auto_test_case::contract::ContractError> {
    let v = parse_contract(src).unwrap();
    decode_case(&v, &Limits::defaults())
}

fn v2_case(check: &str) -> String {
    format!(
        concat!(
            "case {{ schema: \"autolang.case/2\", id: \"ok.ac\", ",
            "source: {{ text: \"fn main() int {{ return 3 }}\", syntax: \"auto\", path: \"main.at\" }}, ",
            "entry: {{ mode: \"program\" }}, ",
            "checks: [ {} ] }}"
        ),
        check
    )
}

fn ac_check(action: &str, expect: &str, options: &str) -> String {
    format!(
        "{{ id: \"ac\", adapter: \"ac\", action: \"{}\", {} {} }}",
        action, options, expect
    )
}

#[test]
fn contract_v2_minimal_ac_check_decodes() {
    let spec = decode(&v2_case(&ac_check(
        "check",
        "expect: { completion: \"success\" }",
        "",
    )))
    .unwrap();
    assert_eq!(spec.schema, CASE_SCHEMA_V2);
    assert_eq!(spec.checks[0].options.effective_profile(), AcProfile::Core);
    assert_eq!(spec.checks[0].options.effective_entry(), "main");
    assert_eq!(spec.checks[0].options.effective_cache(), CacheMode::Reuse);
    assert_eq!(
        spec.checks[0].options.effective_target(),
        "x86_64-pc-windows-msvc"
    );
    assert_eq!(spec.checks[0].options.effective_budget().build_ms, None);
}

#[test]
fn contract_v2_declared_options_resolve() {
    let spec = decode(&v2_case(&ac_check(
        "run",
        "expect: { completion: \"success\", exit: 3 }",
        "options: { profile: \"scalar\", entry: \"start\", cache: \"fresh\", budget: { build_ms: 5000 } },",
    )))
    .unwrap();
    let o = &spec.checks[0].options;
    assert_eq!(o.effective_profile(), AcProfile::Scalar);
    assert_eq!(o.effective_entry(), "start");
    assert_eq!(o.effective_cache(), CacheMode::Fresh);
    assert_eq!(o.effective_budget().build_ms, Some(5000));
    assert_eq!(o.effective_budget().run_ms, None, "undeclared = hard cap");
    assert_eq!(spec.checks[0].expect.exit, Some(3));
}

#[test]
fn contract_v2_v1_checks_behave_identically() {
    // A v2 document may still carry avm/run checks with the v1 field set.
    let src = v2_case(
        "{ id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } }",
    );
    let spec = decode(&src).unwrap();
    assert_eq!(spec.checks[0].adapter.as_str(), "avm");
    assert_eq!(spec.checks[0].action.as_str(), "run");
    assert!(spec.checks[0].options.ac_profile.is_none());
}

#[test]
fn contract_v2_v1_rejects_v2_face() {
    // v1 + ac adapter stays the v1 rejection (byte-identical accept set).
    let v1 = concat!(
        "case { schema: \"autolang.case/1\", id: \"ok.min\", ",
        "source: { text: \"1\", syntax: \"auto\", path: \"main.at\" }, ",
        "entry: { mode: \"expression\" }, ",
        "checks: [ { id: \"ac\", adapter: \"ac\", action: \"check\", expect: { completion: \"success\" } } ] }"
    );
    let err = decode(v1).unwrap_err();
    assert!(
        err.path.ends_with("adapter") && err.message.contains("not available"),
        "{}",
        err
    );

    // v1 + hir-atom syntax: unknown syntax for v1.
    let v1_hir = concat!(
        "case { schema: \"autolang.case/1\", id: \"ok.min\", ",
        "source: { text: \"hir {}\", syntax: \"hir-atom\", path: \"main.at\" }, ",
        "entry: { mode: \"expression\" }, ",
        "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ] }"
    );
    let err = decode(v1_hir).unwrap_err();
    assert!(
        err.path.ends_with("syntax") && err.message.contains("unknown syntax"),
        "{}",
        err
    );

    // v1 + exit field: located unknown-field error (v1 never gains native
    // channels — parsed-then-ignored is forbidden).
    let v1_exit = concat!(
        "case { schema: \"autolang.case/1\", id: \"ok.min\", ",
        "source: { text: \"1\", syntax: \"auto\", path: \"main.at\" }, ",
        "entry: { mode: \"expression\" }, ",
        "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" }, exit: 3 } } ] }"
    );
    let err = decode(v1_exit).unwrap_err();
    assert!(err.path.contains("expect.exit"), "{}", err);
    assert!(err.message.contains("unknown field"), "{}", err);
}

#[test]
fn contract_v2_unknown_field_is_located() {
    let err = decode(&v2_case(&ac_check(
        "check",
        "expect: { completion: \"success\", perf: {}, bogus: 1 }",
        "",
    )))
    .unwrap_err();
    assert!(err.path.contains("expect.bogus"), "{}", err);
}

#[test]
fn contract_v2_hir_atom_syntax_path_rules() {
    let hir_ok = concat!(
        "case { schema: \"autolang.case/2\", id: \"ok.hir\", ",
        "source: { text: \"hir(schema: \\\"auto.hir.core.draft\\\") {}\", syntax: \"hir-atom\", path: \"main.atom\" }, ",
        "entry: { mode: \"program\" }, ",
        "checks: [ { id: \"ac\", adapter: \"ac\", action: \"check\", expect: { completion: \"success\" } } ] }"
    );
    let spec = decode(hir_ok).unwrap();
    assert_eq!(spec.source.syntax.as_str(), "hir-atom");

    let err = decode(&hir_ok.replace("main.atom", "main.at")).unwrap_err();
    assert!(err.message.contains("does not match syntax"), "{}", err);
}

#[test]
fn contract_v2_adapter_action_matrix() {
    for action in ["check", "build", "run"] {
        let expect = if action == "run" {
            "expect: { completion: \"success\", exit: 0 }"
        } else {
            "expect: { completion: \"success\" }"
        };
        decode(&v2_case(&ac_check(action, expect, ""))).unwrap_or_else(|e| panic!("{action}: {e}"));
    }
    for action in ["parse", "edit"] {
        decode(&v2_case(&format!(
            "{{ id: \"sy\", adapter: \"syntax\", action: \"{}\", expect: {{ completion: \"success\" }} }}",
            action
        )))
        .unwrap_or_else(|e| panic!("{action}: {e}"));
    }
    let err = decode(&v2_case(&ac_check(
        "emit",
        "expect: { completion: \"success\" }",
        "",
    )))
    .unwrap_err();
    assert!(err.message.contains("illegal combination"), "{}", err);
    let err = decode(&v2_case(&format!(
        "{{ id: \"sy\", adapter: \"syntax\", action: \"run\", expect: {{ completion: \"success\" }} }}"
    )))
    .unwrap_err();
    assert!(err.message.contains("illegal combination"), "{}", err);
}

#[test]
fn contract_v2_option_value_validation() {
    let bad = |options: &str, needle: &str| {
        let err = decode(&v2_case(&ac_check(
            "check",
            "expect: { completion: \"success\" }",
            &format!("options: {},", options),
        )))
        .unwrap_err();
        assert!(err.message.contains(needle), "{} in {}", needle, err);
    };
    bad("{ profile: \"text\" }", "not imported in this batch");
    bad("{ profile: \"record\" }", "not imported in this batch");
    bad("{ target: \"x86_64-unknown-linux-gnu\" }", "not supported");
    bad("{ entry: \"../x\" }", "entry must be a function-name");
    bad("{ cache: \"warm\" }", "unknown cache mode");
    bad("{ budget: { build_ms: 60001 } }", "frozen hard cap");
    bad("{ budget: { run_ms: 5001 } }", "frozen hard cap");
    bad("{ budget: { total_ms: 0 } }", "frozen hard cap");
    bad("{ budget: { total_ms: 70001 } }", "frozen hard cap");
    // At-cap declarations are legal (may only tighten, never widen).
    decode(&v2_case(&ac_check(
        "check",
        "expect: { completion: \"success\" }",
        "options: { budget: { build_ms: 60000, run_ms: 5000, total_ms: 70000 } },",
    )))
    .unwrap();
}

#[test]
fn contract_v2_exit_channel_rules() {
    // exit on ac/check: not a run observation.
    let err = decode(&v2_case(&ac_check(
        "check",
        "expect: { completion: \"success\", exit: 0 }",
        "",
    )))
    .unwrap_err();
    assert!(err.message.contains("ac/run observation"), "{}", err);

    // exit with an error completion: a trap exit IS success + exit code.
    let err = decode(&v2_case(&ac_check(
        "run",
        "expect: { completion: { stage: \"verify\", code: \"verify.x\" }, exit: 70 }",
        "",
    )))
    .unwrap_err();
    assert!(
        err.message.contains("require completion \"success\""),
        "{}",
        err
    );

    // exit on a v2 avm check: the avm face is unchanged — `exit` is not a
    // v1/avm field at all (located unknown-field, never parsed-then-ignored).
    let err = decode(&v2_case(
        "{ id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", exit: 3 } }",
    ))
    .unwrap_err();
    assert!(
        err.path.contains("expect.exit") && err.message.contains("unknown field"),
        "{}",
        err
    );
}

#[test]
fn contract_v2_stream_rules_for_ac() {
    let err = decode(&v2_case(&ac_check(
        "check",
        "expect: { completion: \"success\", stdout: { text: \"\" } }",
        "",
    )))
    .unwrap_err();
    assert!(
        err.message.contains("ac/check exposes no process streams"),
        "{}",
        err
    );
    decode(&v2_case(&ac_check(
        "run",
        "expect: { completion: \"success\", exit: 0, stdout: { text: \"\" } }",
        "",
    )))
    .unwrap();
}

#[test]
fn contract_v2_perf_face() {
    decode(&v2_case(&ac_check(
        "build",
        "expect: { completion: \"success\", perf: { total_ms_max: 1000 } }",
        "",
    )))
    .unwrap();
    let err = decode(&v2_case(&ac_check(
        "build",
        "expect: { completion: \"success\", perf: { total_ms_max: 0 } }",
        "",
    )))
    .unwrap_err();
    assert!(err.message.contains("total_ms_max must be > 0"), "{}", err);
    let err = decode(&v2_case(
        "{ id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", perf: {} } }",
    ))
    .unwrap_err();
    assert!(
        err.path.contains("expect.perf") && err.message.contains("unknown field"),
        "{}",
        err
    );
}

#[test]
fn contract_v2_artifacts_hir_only() {
    decode(&v2_case(&ac_check(
        "build",
        "expect: { completion: \"success\", artifacts: [ { path: \"hir.atom\", file: \"expect.atom\" } ] }",
        "",
    )))
    .unwrap();
    let err = decode(&v2_case(&ac_check(
        "build",
        "expect: { completion: \"success\", artifacts: [ { path: \"app.exe\", file: \"expect.exe\" } ] }",
        "",
    )))
    .unwrap_err();
    assert!(err.message.contains("manifest-bound"), "{}", err);
}

#[test]
fn contract_v2_typed_stages_per_adapter() {
    for stage in [
        "resolve",
        "typecheck",
        "bind",
        "verify",
        "capability",
        "backend",
        "link",
        "run",
        "parse",
    ] {
        decode(&v2_case(&ac_check(
            "check",
            &format!(
                "expect: {{ completion: {{ stage: \"{}\", code: \"x.y\" }} }}",
                stage
            ),
            "",
        )))
        .unwrap_or_else(|e| panic!("{stage}: {e}"));
    }
    // avm keeps the v1 stage set — even inside a v2 document.
    let err = decode(&v2_case(
        "{ id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"verify\", code: \"x.y\" } } }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("unknown or unsupported stage"),
        "{}",
        err
    );
    // syntax negatives are parse-stage diagnostics only until T-06.
    decode(&v2_case(
        "{ id: \"sy\", adapter: \"syntax\", action: \"parse\", expect: { completion: { stage: \"parse\", code: \"lex.bad\" } } }",
    ))
    .unwrap();
    let err = decode(&v2_case(
        "{ id: \"sy\", adapter: \"syntax\", action: \"parse\", expect: { completion: { stage: \"verify\", code: \"x.y\" } } }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("unknown or unsupported stage"),
        "{}",
        err
    );
}

#[test]
fn contract_v2_syntax_channels_closed_until_t06() {
    let err = decode(&v2_case(
        "{ id: \"sy\", adapter: \"syntax\", action: \"parse\", expect: { completion: \"success\", stdout: { text: \"\" } } }",
    ))
    .unwrap_err();
    assert!(err.message.contains("not an observable channel"), "{}", err);
    // completion-only syntax checks decode (execution still ERRORs until
    // D-787 — the executor's admission, not a decode SKIP).
    decode(&v2_case(
        "{ id: \"sy\", adapter: \"syntax\", action: \"parse\", expect: { completion: \"success\" } }",
    ))
    .unwrap();
}

#[test]
fn contract_v2_suite_dispatch_and_ac_defaults() {
    let suite_v2 = concat!(
        "suite { schema: \"autolang.suite/2\", id: \"t2\", include: [\"a.case.ad\"], ",
        "defaults: { tier: \"native\", options: { ac: { run: { profile: \"scalar\", budget: { run_ms: 3000 } } } } }, ",
        "required: { adapters: [\"ac\"] } }"
    );
    let s = decode_suite(&parse_contract(suite_v2).unwrap()).unwrap();
    assert_eq!(s.required_adapters.len(), 1);
    assert_eq!(
        s.defaults.options.get("ac").unwrap().get("run").is_some(),
        true
    );

    // v1 suite: ac defaults key and ac required adapter stay rejected.
    let v1_ac_default = concat!(
        "suite { schema: \"autolang.suite/1\", id: \"t1\", include: [\"a.case.ad\"], ",
        "defaults: { options: { ac: { run: { profile: \"scalar\" } } } } }"
    );
    let err = decode_suite(&parse_contract(v1_ac_default).unwrap()).unwrap_err();
    assert!(err.message.contains("unknown field"), "{}", err);
    let v1_required = concat!(
        "suite { schema: \"autolang.suite/1\", id: \"t1\", include: [\"a.case.ad\"], ",
        "required: { adapters: [\"ac\"] } }"
    );
    let err = decode_suite(&parse_contract(v1_required).unwrap()).unwrap_err();
    assert!(err.message.contains("unknown adapter"), "{}", err);

    // v2 suite default value errors are located at the suite.
    let bad_default = concat!(
        "suite { schema: \"autolang.suite/2\", id: \"t2\", include: [\"a.case.ad\"], ",
        "defaults: { options: { ac: { run: { profile: \"text\" } } } } }"
    );
    let err = decode_suite(&parse_contract(bad_default).unwrap()).unwrap_err();
    assert!(
        err.message.contains("not imported in this batch"),
        "{}",
        err
    );
    let over_cap = concat!(
        "suite { schema: \"autolang.suite/2\", id: \"t2\", include: [\"a.case.ad\"], ",
        "defaults: { options: { ac: { run: { budget: { total_ms: 99999 } } } } } }"
    );
    let err = decode_suite(&parse_contract(over_cap).unwrap()).unwrap_err();
    assert!(err.message.contains("frozen hard cap"), "{}", err);
}

#[test]
fn contract_v2_suite_default_fills_undeclared_ac_field() {
    let suite_v2 = concat!(
        "suite { schema: \"autolang.suite/2\", id: \"t2\", include: [\"a.case.ad\"], ",
        "defaults: { options: { ac: { run: { profile: \"scalar\" } } } } }"
    );
    let s = decode_suite(&parse_contract(suite_v2).unwrap()).unwrap();
    let spec = decode(&v2_case(&ac_check(
        "run",
        "expect: { completion: \"success\", exit: 3 }",
        "",
    )))
    .unwrap();
    let (merged, _) = apply_defaults(&spec.checks[0], &s.defaults);
    assert_eq!(merged.options.effective_profile(), AcProfile::Scalar);
    // A case-declared value wins over the suite default.
    let spec = decode(&v2_case(&ac_check(
        "run",
        "expect: { completion: \"success\", exit: 3 }",
        "options: { profile: \"core\" },",
    )))
    .unwrap();
    let (merged, _) = apply_defaults(&spec.checks[0], &s.defaults);
    assert_eq!(merged.options.effective_profile(), AcProfile::Core);
    // Without any suite default, undeclared stays implicit (core).
    let spec2 = decode(&v2_case(&ac_check(
        "run",
        "expect: { completion: \"success\", exit: 3 }",
        "",
    )))
    .unwrap();
    assert_eq!(spec2.checks[0].options.effective_profile(), AcProfile::Core);
}

#[test]
fn contract_v2_report_native_record_renders() {
    use auto_test_case::report::{CheckOutcome, IdentityHashes, NativeRecord, RunReport, Status};
    let mut r = RunReport::new("s", "h", &[]);
    r.schema = auto_test_case::report::REPORT_SCHEMA_V2.to_string();
    let mut o = CheckOutcome {
        suite_id: "s".into(),
        case_id: "c".into(),
        check_id: "ac".into(),
        execution_key: "s:h:c:ac".into(),
        adapter: "ac".into(),
        action: "run".into(),
        tier: None,
        status: Status::Pass,
        mismatches: vec![],
        detail: None,
        identity: IdentityHashes::default(),
        artifacts: vec![],
        duration_ms: 42,
        native: Some(NativeRecord {
            exit_code: Some(3),
            cache_mode: Some("reuse".into()),
            cache_hits: vec![("object".into(), "hit".into())],
            counters: vec![("parse".into(), 0), ("link".into(), 1)],
            stage_spans: vec![("link".into(), 120)],
            toolchain: vec![("lld".into(), "abc".into())],
            gaps: vec!["cpu_ms".into(), "peak_rss".into()],
        }),
    };
    o.status = Status::Pass;
    r.results = vec![o];
    let json = r.to_json();
    assert!(json.contains("autolang.test-report/2"), "{}", json);
    assert!(json.contains("\"exit_code\": 3"));
    assert!(json.contains("\"native\""));
    assert!(r.to_console().contains("exit=3"), "console native line");
    assert_eq!(r.exit_code(), 0, "native record never changes the gate");
}

// ---- PLAN-788 R1 repairs: JSON legality + cache-declaration gate ------

#[test]
fn contract_v2_report_json_round_trips_through_standard_parser() {
    use auto_test_case::report::{CheckOutcome, IdentityHashes, NativeRecord, RunReport, Status};
    let mk = |native: Option<NativeRecord>, detail: Option<String>| {
        let mut r = RunReport::new("s", "h", &["ac".into()]);
        r.schema = auto_test_case::report::REPORT_SCHEMA_V2.to_string();
        let mut o = CheckOutcome {
            suite_id: "s".into(),
            case_id: "c".into(),
            check_id: "ac".into(),
            execution_key: "s:h:c:ac".into(),
            adapter: "ac".into(),
            action: "run".into(),
            tier: None,
            status: Status::Pass,
            mismatches: vec![],
            detail,
            identity: IdentityHashes::default(),
            artifacts: vec![],
            duration_ms: 7,
            native,
        };
        o.status = Status::Pass;
        r.results = vec![o];
        r
    };
    // Full native record: standard parser must round-trip (F06: the old
    // emitter produced a missing comma after duration_ms and a trailing
    // comma after native — both illegal).
    let full = mk(
        Some(NativeRecord {
            exit_code: Some(3),
            cache_mode: Some("reuse".into()),
            cache_hits: vec![("hir".into(), "hit(restored)".into())],
            counters: vec![("parse".into(), 0), ("run".into(), 1)],
            stage_spans: vec![("frontend".into(), 12), ("run".into(), 34)],
            toolchain: vec![("lld-sha256".into(), "dead\\bee\"f".into())],
            gaps: vec!["cpu_ms".into()],
        }),
        None,
    );
    let v: serde_json::Value = serde_json::from_str(&full.to_json()).expect("legal JSON");
    assert_eq!(v["results"][0]["native"]["exit_code"], 3);
    assert_eq!(v["results"][0]["native"]["stage_spans"][1][1], 34);
    assert_eq!(v["results"][0]["native"]["gaps"][0], "cpu_ms");
    // No-native and multi-line detail rows stay legal too.
    let none = mk(None, Some("multi\nline — detail \u{2014} ok".into()));
    let v: serde_json::Value = serde_json::from_str(&none.to_json()).expect("legal JSON");
    assert!(v["results"][0].get("native").is_none());
    // v1 reports are unchanged and legal.
    let mut v1 = mk(None, None);
    v1.schema = auto_test_case::report::REPORT_SCHEMA.to_string();
    let json = v1.to_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("v1 legal");
    assert_eq!(v["schema"], "autolang.test-report/1");
}

#[test]
fn contract_v2_cache_declaration_requires_participation() {
    // fresh on ac/check: the frontend-observing action never touches the
    // store — declaring a behavioral mode is an admission error.
    let err = decode(&v2_case(&ac_check(
        "check",
        "expect: { completion: \"success\" }",
        "options: { cache: \"fresh\" },",
    )))
    .unwrap_err();
    assert!(
        err.path.contains("options.cache") && err.message.contains("participation"),
        "{}",
        err
    );
    // read-only on a raw-HIR run: the driver path needs auto-source input.
    let hir = concat!(
        "case { schema: \"autolang.case/2\", id: \"ok.hir\", ",
        "source: { text: \"hir {}\", syntax: \"hir-atom\", path: \"main.atom\" }, ",
        "entry: { mode: \"program\" }, ",
        "checks: [ { id: \"ac\", adapter: \"ac\", action: \"run\", options: { cache: \"read-only\" }, expect: { completion: \"success\", exit: 0 } } ] }"
    );
    let err = decode(hir).unwrap_err();
    assert!(err.message.contains("participation"), "{}", err);
    // The default (undeclared) and explicit reuse stay legal everywhere.
    decode(&v2_case(&ac_check(
        "check",
        "expect: { completion: \"success\" }",
        "",
    )))
    .unwrap();
    decode(&v2_case(&ac_check(
        "check",
        "expect: { completion: \"success\" }",
        "options: { cache: \"reuse\" },",
    )))
    .unwrap();
}

#[test]
fn contract_v2_syntax_options_and_edits_bounds() {
    // T-06: grammar selector and bounded single-edit steps decode; the
    // wrong grammar, over-long sequences, inverted spans and oversized
    // inserts are located admission errors.
    let syn_case = |options: &str| {
        format!(
            concat!(
                "case {{ schema: \"autolang.case/2\", id: \"ok.syn\", ",
                "source: {{ text: \"fn main() int {{ return 3 }}\", syntax: \"auto\", path: \"main.at\" }}, ",
                "entry: {{ mode: \"program\" }}, ",
                "checks: [ {{ id: \"sy\", adapter: \"syntax\", action: \"parse\", options: {}, ",
                "expect: {{ completion: \"success\" }} }} ] }}"
            ),
            options
        )
    };
    decode(&syn_case("{ grammar: \"interface-mini-v1\" }")).unwrap();
    decode(&syn_case("{ grammar: \"auto-core\" }")).unwrap();
    let err = decode(&syn_case("{ grammar: \"nope\" }")).unwrap_err();
    assert!(err.message.contains("unknown grammar"), "{}", err);

    let edit_case = |edits: &str| {
        format!(
            concat!(
                "case {{ schema: \"autolang.case/2\", id: \"ok.syn\", ",
                "source: {{ text: \"fn main() int {{ return 3 }}\", syntax: \"auto\", path: \"main.at\" }}, ",
                "entry: {{ mode: \"program\" }}, ",
                "checks: [ {{ id: \"sy\", adapter: \"syntax\", action: \"edit\", options: {{ grammar: \"auto-core\", edits: {} }}, ",
                "expect: {{ completion: \"success\" }} }} ] }}"
            ),
            edits
        )
    };
    decode(&edit_case("[{ start: 0, end: 0, insert: \"x\" }]")).unwrap();
    let err = decode(&edit_case("[]")).unwrap_err();
    assert!(err.message.contains("1..=64"), "{}", err);
    let err = decode(&edit_case("[{ start: 5, end: 2, insert: \"x\" }]")).unwrap_err();
    assert!(err.message.contains("half-open"), "{}", err);
    // syntax artifacts are restricted to the three logical projections.
    let err = decode(&syn_case("{}").replace(
        "expect: { completion: \"success\" }",
        "expect: { completion: \"success\", artifacts: [ { path: \"app.exe\", file: \"x\" } ] }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("syntax artifact byte-oracles"),
        "{}",
        err
    );
}
