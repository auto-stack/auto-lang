#![cfg(windows)]

use ace_aac_syntax_conformance::run_semantic_foreign_case;
use std::{fs, path::PathBuf};

const RICH_BOOL: &str = "use.rs f0_rich\nfn main() bool { return rich_is_longer(\"hello\", 4) }";
const RICH_FALLIBLE: &str = "use.rs f0_rich\nfn main() int { return rich_find(\"hello\", 111) }";

fn prepared_page() -> (Vec<u8>, String) {
    let root = PathBuf::from(
        std::env::var_os("AAC1047_FOREIGN_FIXTURE").expect("prepare the 967 foreign fixture"),
    );
    let page = fs::read(root.join("source-declarations.json")).expect("normalized 967 SourceDecls");
    let authority = fs::read_to_string(root.join("authority.blake3"))
        .expect("independently prepared authority");
    assert_eq!(authority.trim(), blake3::hash(&page).to_hex().as_str());
    (page, authority.trim().to_owned())
}

#[test]
fn checks_source_calls_before_whole_hir_admission() {
    let (page, authority) = prepared_page();
    let run =
        |source: &str| run_semantic_foreign_case(source, &page, &authority).expect("Auto frontend");
    for source in [RICH_BOOL, RICH_FALLIBLE] {
        let report = run(source);
        assert!(report.starts_with("aac.type-report/1\n"), "{report}");
        assert!(report.contains("PENDING 4 "), "{report}");
        assert!(!report.contains("hir("), "{report}");
    }
    let wrong_arg = RICH_BOOL.replace(", 4)", ", true)");
    let start = wrong_arg.find("true").unwrap();
    assert_eq!(
        run(wrong_arg.as_str()),
        format!("ERROR type.argument 0 {start}..{}\n", start + 4)
    );
    let wrong_result = RICH_BOOL.replace("() bool", "() int");
    let start = wrong_result.find("rich_is_longer").unwrap();
    let end = wrong_result.find(") }").unwrap() + 1;
    assert_eq!(
        run(wrong_result.as_str()),
        format!("ERROR type.return 0 {start}..{end}\n")
    );
}

#[test]
fn rejects_substituted_physical_layout_and_unknown_provider() {
    let (page, authority) = prepared_page();
    let mut changed: serde_json::Value = serde_json::from_slice(&page).unwrap();
    changed[0]["physical_table_digest"] = serde_json::Value::String("0".repeat(64));
    let altered = serde_json::to_vec(&changed).unwrap();
    assert_eq!(
        run_semantic_foreign_case(RICH_BOOL, &altered, &authority).expect("Auto frontend"),
        "ERROR foreign.authority 0 7..14\n"
    );
    let unknown = RICH_BOOL.replace("f0_rich", "unknown");
    assert_eq!(
        run_semantic_foreign_case(&unknown, &page, &authority).expect("Auto frontend"),
        "ERROR foreign.provider 0 7..14\n"
    );
}
