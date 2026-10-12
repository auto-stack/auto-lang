//! Suite loading pipeline integration (V-01/V-07 core): real files on disk,
//! discovery inventory complete, duplicate identities localized, empty
//! selection hard-fails, sidecar payloads resolve.

use std::path::PathBuf;

use auto_test_case::admission::Limits;
use auto_test_case::discovery::load_suite;
use auto_test_case::ir::resolve_case;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-suite-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const SUITE: &str = concat!(
    "# Pilot suite\n",
    "```test-suite\n",
    "suite { schema: \"autolang.suite/1\", id: \"pilot\", ",
    "include: [\"a.case.ad\", \"b.case.ad\", \"broken.case.ad\"], ",
    "defaults: { tier: \"vm-corpus\" }, ",
    "required: { adapters: [\"avm\"] } }\n",
    "```\n",
);

const CASE_A: &str = concat!(
    "# A\n",
    "## Src ^src\n",
    "```auto\n",
    "1 + 2\n",
    "```\n",
    "## Contract\n",
    "```test-case\n",
    "case { schema: \"autolang.case/1\", id: \"pilot.a\", ",
    "source: { block: \"src\", syntax: \"auto\", path: \"main.at\" }, ",
    "entry: { mode: \"expression\" }, ",
    "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", ",
    "expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ] }\n",
    "```\n",
);

const CASE_B: &str = concat!(
    "# B — sidecar source\n",
    "```test-case\n",
    "case { schema: \"autolang.case/1\", id: \"pilot.b\", ",
    "source: { file: \"b.at\", syntax: \"auto\", path: \"main.at\" }, ",
    "entry: { mode: \"expression\" }, ",
    "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", ",
    "expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"8\" } } } ] }\n",
    "```\n",
);

const CASE_BROKEN: &str = "# broken\n```test-case\ncase { schema: \"autolang.case/1\", id: \"\", source: { text: \"x\", syntax: \"auto\", path: \"m.at\" } }\n```\n";

#[test]
fn loads_suite_with_inventory_and_errors() {
    let root = temp_root("ok");
    std::fs::write(root.join("pilot.suite.ad"), SUITE).unwrap();
    std::fs::write(root.join("a.case.ad"), CASE_A).unwrap();
    std::fs::write(root.join("b.case.ad"), CASE_B).unwrap();
    std::fs::write(root.join("b.at"), b"3 + 5\n").unwrap();
    std::fs::write(root.join("broken.case.ad"), CASE_BROKEN).unwrap();

    let loaded = load_suite(&root.join("pilot.suite.ad"), &Limits::defaults()).unwrap();
    assert_eq!(loaded.suite.id, "pilot");
    assert_eq!(
        loaded.cases.len(),
        3,
        "broken file still appears with its specs (inventory complete)"
    );
    let total: usize = loaded.cases.iter().map(|c| c.specs.len()).sum();
    assert_eq!(total, 2, "only the two valid specs decode");
    assert_eq!(
        loaded.errors.len(),
        1,
        "broken case error is kept, not dropped"
    );
    assert!(
        loaded.errors[0].detail.contains("case fence"),
        "{}",
        loaded.errors[0]
    );

    // Resolve payloads: anchored fence + sidecar file.
    for case_file in &loaded.cases {
        for spec in &case_file.specs {
            let rc = resolve_case(
                spec.clone(),
                &case_file.doc,
                case_file.path.parent().unwrap(),
                &Limits::defaults(),
            )
            .unwrap();
            assert_eq!(rc.files.len(), 1);
            assert_eq!(rc.files[0].virtual_path, "main.at");
        }
    }
    let b = loaded
        .cases
        .iter()
        .flat_map(|c| c.specs.iter())
        .find(|s| s.id == "pilot.b")
        .unwrap();
    let b_file = loaded
        .cases
        .iter()
        .find(|c| c.specs.iter().any(|s| s.id == "pilot.b"))
        .unwrap();
    let rc = resolve_case(
        b.clone(),
        &b_file.doc,
        b_file.path.parent().unwrap(),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(rc.files[0].bytes, b"3 + 5\n".to_vec(), "sidecar bytes");
}

#[test]
fn duplicate_case_ids_across_files_are_reported() {
    let root = temp_root("dup");
    std::fs::write(
        root.join("pilot.suite.ad"),
        SUITE.replace("broken.case.ad", "c.case.ad"),
    )
    .unwrap();
    std::fs::write(root.join("a.case.ad"), CASE_A).unwrap();
    std::fs::write(
        root.join("b.case.ad"),
        CASE_A
            .replace("## Src ^src", "## Src ^src2")
            .replace("block: \"src\"", "block: \"src2\""),
    )
    .unwrap();
    std::fs::write(root.join("c.case.ad"), CASE_B).unwrap();
    std::fs::write(root.join("b.at"), b"1\n").unwrap();

    let loaded = load_suite(&root.join("pilot.suite.ad"), &Limits::defaults()).unwrap();
    assert!(
        loaded
            .errors
            .iter()
            .any(|e| e.detail.contains("duplicate case id")),
        "errors: {:?}",
        loaded.errors
    );
}

#[test]
fn zero_cases_is_a_hard_error() {
    let root = temp_root("empty");
    let suite = SUITE.replace(
        "include: [\"a.case.ad\", \"b.case.ad\", \"broken.case.ad\"]",
        "include: [\"missing.case.ad\"]",
    );
    std::fs::write(root.join("pilot.suite.ad"), suite).unwrap();
    let err = load_suite(&root.join("pilot.suite.ad"), &Limits::defaults()).unwrap_err();
    assert!(err.detail.contains("zero cases"), "{}", err);
}

#[test]
fn suite_defaults_apply_tier_and_options() {
    use auto_test_case::suite::apply_defaults;
    let root = temp_root("defaults");
    std::fs::write(root.join("pilot.suite.ad"), concat!(
        "```test-suite\n",
        "suite { schema: \"autolang.suite/1\", id: \"d\", ",
        "include: [\"a.case.ad\"], ",
        "defaults: { tier: \"vm-corpus\", options: { a2r: { emit: { module_name: \"auto\" } } } } }\n",
        "```\n",
    )).unwrap();
    std::fs::write(root.join("a.case.ad"), CASE_A).unwrap();
    let loaded = load_suite(&root.join("pilot.suite.ad"), &Limits::defaults()).unwrap();
    let spec = &loaded.cases[0].specs[0];
    assert!(spec.checks[0].tier.is_none(), "case declares no tier");
    let (merged, origins) = apply_defaults(&spec.checks[0], &loaded.suite.defaults);
    assert_eq!(merged.tier.as_deref(), Some("vm-corpus"));
    assert!(origins
        .iter()
        .any(|o| o.field == "tier" && o.source == "suite-default"));
}
