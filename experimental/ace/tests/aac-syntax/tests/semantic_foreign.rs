#![cfg(windows)]

use ace_aac_syntax_conformance::run_semantic_foreign_case;
use std::{fs, path::PathBuf};

const RICH_BOOL: &str = "use.rs f0_rich\nfn main() bool { return rich_is_longer(\"hello\", 4) }";
const RICH_FALLIBLE: &str = "use.rs f0_rich\nfn main() int { return rich_find(\"hello\", 111) }";

fn prepared_page(environment: &str) -> (Vec<u8>, String) {
    let root = PathBuf::from(
        std::env::var_os(environment).unwrap_or_else(|| panic!("prepare fixture {environment}")),
    );
    let page = fs::read(root.join("source-declarations.json")).expect("normalized 967 SourceDecls");
    let authority = fs::read_to_string(root.join("authority.blake3"))
        .expect("independently prepared authority");
    assert_eq!(authority.trim(), blake3::hash(&page).to_hex().as_str());
    (page, authority.trim().to_owned())
}

#[test]
fn checks_source_calls_before_whole_hir_admission() {
    let (page, authority) = prepared_page("AAC1047_FOREIGN_FIXTURE");
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
    let (page, authority) = prepared_page("AAC1047_FOREIGN_FIXTURE");
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

#[test]
fn checks_c_provider_handle_credentials_and_callback_targets() {
    let (page, authority) = prepared_page("AAC1047_C_FIXTURE");
    let run =
        |source: &str| run_semantic_foreign_case(source, &page, &authority).expect("Auto frontend");
    let handle_flow = concat!(
        "use.c f0_c\nfn main() int {\n",
        "    let h int = f0c_counter_create(7)\n",
        "    let v int = f0c_counter_get(h)\n",
        "    let d int = f0c_counter_destroy(h)\n",
        "    return v + d\n}",
    );
    let callback = concat!(
        "use.c f0_c\n",
        "fn adjust(ctx int, item int) int { return ctx + item }\n",
        "fn main() int { return f0c_apply(adjust, 1, 2, 3) }\n",
    );
    for source in [handle_flow, callback] {
        let report = run(source);
        assert!(report.starts_with("aac.type-report/1\n"), "{report}");
        assert!(report.contains("PENDING 4 "), "{report}");
        assert!(!report.contains("hir("), "{report}");
    }

    let unissued = handle_flow.replace("f0c_counter_create(7)", "7");
    let site = unissued.find("get(h)").unwrap() + "get(".len();
    assert_eq!(
        run(&unissued),
        format!("ERROR foreign.handle-owner 0 {site}..{}\n", site + 1)
    );

    let misdeclared = callback
        .replace("item int", "item bool")
        .replace("ctx + item", "ctx");
    let site = misdeclared.find("apply(adjust").unwrap() + "apply(".len();
    assert_eq!(
        run(&misdeclared),
        format!("ERROR type.argument 0 {site}..{}\n", site + "adjust".len())
    );
    let indirect = callback.replace(
        "return f0c_apply(adjust",
        "let selected = adjust\n return f0c_apply(selected",
    );
    let site = indirect.find("apply(selected").unwrap() + "apply(".len();
    assert_eq!(
        run(&indirect),
        format!(
            "ERROR foreign.callback-target 0 {site}..{}\n",
            site + "selected".len()
        )
    );
}
