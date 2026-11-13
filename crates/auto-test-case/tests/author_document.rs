//! PLAN-1002 document-style authoring decode battery (V-01): the committed
//! four-case pilot decodes to the exact legacy identities (ids, recipes,
//! check order, byte-identical sources and goldens), new-writing defaults
//! are computable, and every malformed shape is a LOCATED error.

use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, is_document_case, AuthorError};
use auto_test_case::contract::{Adapter, Completion, PayloadSpec, Stage};
use auto_test_case::discovery::{load_suite, CaseFileFormat};
use auto_test_case::document::CaseDocument;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn corpus(rel: &str) -> Vec<u8> {
    std::fs::read(Path::new(ROOT).join("../auto-lang/test/cases").join(rel))
        .expect("repo corpus file present")
}

fn pilot_doc() -> CaseDocument {
    CaseDocument::parse(&corpus("ace-m2/document-pilot-1002/pilot.case.ad")).unwrap()
}

fn decode(src: &str, rel: &str) -> Result<Vec<auto_test_case::author::AuthorCase>, AuthorError> {
    let doc = CaseDocument::parse(src.as_bytes()).unwrap();
    decode_document(&doc, rel, &Limits::defaults())
}

const AUTO_SECTION: &str = "### Auto\n\n```auto\nfn main() int { return 1 }\n```\n";

fn case_doc(body: &str) -> String {
    format!("## Case: a\n\n{AUTO_SECTION}{body}")
}

#[test]
fn pilot_decodes_four_cases_seven_checks_with_legacy_identities() {
    let doc = pilot_doc();
    assert!(is_document_case(&doc));
    let cases = decode_document(
        &doc,
        "ace-m2/document-pilot-1002/pilot.case.ad",
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(cases.len(), 4);

    let ids: Vec<&str> = cases.iter().map(|c| c.spec.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "ace.m1.b01",
            "ace.m1.o03",
            "ace.m1.ia920.b18",
            "ace.m1.ia920.nb18"
        ]
    );

    // B01: file tags + case tags merge in order, links survive.
    let b01 = &cases[0];
    assert_eq!(b01.spec.tags, vec!["ace-m1", "four-path", "scalar"]);
    assert_eq!(b01.spec.links, vec!["PLAN-852"]);
    assert_eq!(b01.spec.source.path, "main.at");
    assert_eq!(
        b01.spec.source.payload,
        PayloadSpec::Text(String::from_utf8(corpus("ace-m1/src/B01.at").to_vec()).unwrap()),
        "inline source bytes must equal the old src/B01.at"
    );

    // B01 checks keep the legacy order and wiring.
    let checks: Vec<(&str, Adapter)> = b01
        .spec
        .checks
        .iter()
        .map(|c| (c.id.as_str(), c.adapter))
        .collect();
    assert_eq!(
        checks,
        vec![
            ("ac-native", Adapter::Probe),
            ("av", Adapter::Probe),
            ("autovm", Adapter::Avm),
            ("a2r-snapshot", Adapter::A2r),
        ]
    );
    let ac = &b01.spec.checks[0];
    assert_eq!(
        ac.options.probe_recipe.as_deref(),
        Some("ac/mir-native-scalar-v1")
    );
    assert_eq!(ac.options.probe_mn.as_ref().unwrap().entry, "d_main");
    assert_eq!(ac.expect.exit, Some(14));
    let av = &b01.spec.checks[1];
    assert_eq!(
        av.options.probe_recipe.as_deref(),
        Some("ac-av/core-run-v1")
    );
    assert_eq!(av.options.probe_av.as_ref().unwrap().entry, "d_main");
    assert_eq!(av.expect.exit, Some(14));
    // module-name rides ONLY the a2r emit check.
    assert!(b01.spec.checks[0].options.module_name.is_none());
    assert!(b01.spec.checks[1].options.module_name.is_none());
    assert!(b01.spec.checks[2].options.module_name.is_none());
    assert_eq!(
        b01.spec.checks[3].options.module_name.as_deref(),
        Some("m1_b01")
    );
    // VM result text is exactly "14" (final-newline stripped).
    assert_eq!(
        b01.spec.checks[2].expect.result.as_ref().unwrap().text,
        "14"
    );
    // Inline golden bytes equal the old oracle file byte-for-byte.
    let golden = &b01.spec.checks[3].expect.artifacts[0];
    assert_eq!(golden.path, "m1_b01.rs");
    match &golden.payload {
        auto_test_case::contract::ArtifactPayload::Inline {
            bytes, open_line, ..
        } => {
            assert_eq!(bytes, &corpus("ace-m1/oracle/m1_b01.rs"));
            assert!(*open_line > 0, "golden carries its fence origin");
        }
        other => panic!("document golden must be inline, got {:?}", other),
    }

    // O03: one console probe, dual streams, exact UTF-8 bytes.
    let o03 = &cases[1];
    assert_eq!(o03.spec.checks.len(), 1);
    let console = &o03.spec.checks[0];
    assert_eq!(console.id, "av-console");
    assert_eq!(
        console.options.probe_recipe.as_deref(),
        Some("ac-av/console-run-v1")
    );
    let avb = console.options.probe_av.as_ref().unwrap();
    assert_eq!(avb.entry, "d_main");
    assert_eq!(avb.caps, vec!["console.stdout", "console.stderr"]);
    assert_eq!(console.expect.exit, Some(0));
    assert_eq!(
        console.expect.stdout.as_ref().unwrap().text.as_bytes(),
        "你好\n\n".as_bytes()
    );
    assert_eq!(
        console.expect.stderr.as_ref().unwrap().text.as_bytes(),
        "é\n".as_bytes()
    );
    assert_eq!(
        o03.spec.source.payload,
        PayloadSpec::Text(
            String::from_utf8(corpus("ace-m1/console-863/src/O03.at").to_vec()).unwrap()
        )
    );

    // B18: staged typed diagnostic; NB18: success result 2.
    let b18 = &cases[2];
    assert_eq!(b18.spec.checks.len(), 1);
    match &b18.spec.checks[0].expect.completion {
        Completion::Error { stage, code, .. } => {
            assert_eq!(*stage, Stage::Type);
            assert_eq!(code.as_deref(), Some("auto_name_E0203"));
        }
        other => panic!("B18 must be a typed error, got {:?}", other),
    }
    let nb18 = &cases[3];
    assert_eq!(
        nb18.spec.checks[0].expect.result.as_ref().unwrap().text,
        "2"
    );
    assert_eq!(
        nb18.spec.source.payload,
        PayloadSpec::Text(String::from_utf8(corpus("ace-m1/src/NB18.at").to_vec()).unwrap())
    );

    // Document spans: per-check section positions exist and are ascending.
    for c in &cases {
        assert!(c.heading_line >= 1);
        assert_eq!(c.check_spans.len(), c.spec.checks.len());
        let mut lines: Vec<usize> = c.check_spans.iter().map(|s| s.section_line).collect();
        lines.dedup();
        assert!(
            lines.windows(2).all(|w| w[0] < w[1]),
            "section lines ascend"
        );
    }
}

#[test]
fn pilot_loads_through_the_suite_pipeline() {
    let suite =
        Path::new(ROOT).join("../auto-lang/test/cases/ace-m2/document-pilot-1002/pilot.suite.ad");
    let loaded = load_suite(&suite, &Limits::defaults()).unwrap();
    assert!(loaded.errors.is_empty());
    assert_eq!(loaded.cases.len(), 1);
    assert_eq!(loaded.cases[0].format, CaseFileFormat::Document);
    assert_eq!(loaded.cases[0].specs.len(), 4);
    assert_eq!(loaded.cases[0].doc_spans.len(), 4);
    let total_checks: usize = loaded.cases.iter().map(|c| c.specs.len()).sum::<usize>();
    assert_eq!(total_checks, 4);
    let checks: usize = loaded
        .cases
        .iter()
        .flat_map(|c| c.specs.iter().map(|s| s.checks.len()))
        .sum();
    assert_eq!(checks, 7, "the pilot is exactly 4 cases / 7 checks");
}

#[test]
fn hello_defaults_without_any_pairs() {
    let src = concat!(
        "## Case: hello\n",
        "\n",
        "### Auto\n",
        "\n",
        "```auto\n",
        "print(\"hello, world!\")\n",
        "```\n",
        "\n",
        "### Rust\n",
        "\n",
        "```rust\n",
        "// Auto-generated by a2r transpiler\n",
        "\n",
        "fn main() {\n",
        "    println!(\"hello, world!\");\n",
        "}\n",
        "```\n",
    );
    let cases = decode(src, "basics/printing.case.ad").unwrap();
    assert_eq!(cases.len(), 1);
    let spec = &cases[0].spec;
    assert_eq!(
        spec.id, "basics/printing#hello",
        "default ID is rel-path#key"
    );
    assert_eq!(spec.source.path, "hello.at", "default virtual source name");
    assert!(spec.tags.is_empty() && spec.links.is_empty());
    assert_eq!(spec.checks.len(), 1);
    assert_eq!(spec.checks[0].id, "rust", "default Rust check id");
    assert!(spec.checks[0].options.module_name.is_none());
    let art = &spec.checks[0].expect.artifacts[0];
    assert_eq!(art.path, "hello.rs", "default single-artifact logical name");
}

#[test]
fn located_rejection_battery() {
    // Case-sensitive heading vocabulary.
    let lower = "## Case: a\n\n### auto\n\n```auto\nx\n```\n";
    assert!(
        decode(lower, "x.case.ad").is_err(),
        "### auto (lowercase) is unknown"
    );
    // Unwired target vocabulary is a located error, never a skip.
    let e = decode(&case_doc("\n### VM\n\n```text\n1\n```\n"), "x.case.ad").unwrap_err();
    assert!(e.message.contains("未在本批接通"), "{e:?}");
    // Wrong fence language under Rust.
    let c_fence = case_doc("\n### Rust\n\n```c\nint main() {}\n```\n");
    assert!(
        decode(&c_fence, "x.case.ad").is_err(),
        "c fence under Rust is rejected"
    );
    // Duplicate case key.
    let dup = format!("## Case: a\n\n{AUTO_SECTION}\n## Case: a\n\n{AUTO_SECTION}");
    assert!(decode(&dup, "x.case.ad").is_err(), "duplicate key rejected");
    // Duplicate check id.
    let dupcheck = case_doc(
        "\n### Probe p\nrecipe: \"ac-av/core-run-v1\"\nentry: \"d_main\"\n\n### Probe p\nrecipe: \"ac-av/core-run-v1\"\nentry: \"d_main\"\n",
    );
    assert!(
        decode(&dupcheck, "x.case.ad").is_err(),
        "duplicate Probe id rejected"
    );
    // Success and failure on one VM action.
    let conflict = case_doc(
        "\n### VM result\n\n```text\n1\n```\n\n### VM diagnostic\nstage: \"type\"\ncode: \"c1\"\n",
    );
    assert!(
        decode(&conflict, "x.case.ad").is_err(),
        "success+failure conflict"
    );
    // Missing stage.
    let nostage = case_doc("\n### VM diagnostic\ncode: \"c1\"\n");
    assert!(decode(&nostage, "x.case.ad").is_err(), "stage required");
    // Unknown attribute key.
    let badattr = format!("## Case: a\nadaptor: \"x\"\n\n{AUTO_SECTION}");
    assert!(
        decode(&badattr, "x.case.ad").is_err(),
        "unknown case attribute"
    );
    // Attributes after prose are not silently swallowed.
    let afterprose = format!("## Case: a\n\nintro line\n\nid: \"x.y\"\n\n{AUTO_SECTION}");
    assert!(
        decode(&afterprose, "x.case.ad").is_err(),
        "attrs after prose rejected"
    );
    // Console recipe needs both streams.
    let nostream =
        case_doc("\n### Probe c\nrecipe: \"ac-av/console-run-v1\"\nentry: \"d_main\"\nexit: 0\n");
    assert!(
        decode(&nostream, "x.case.ad").is_err(),
        "console needs stdout+stderr"
    );
    // Console entry must be d_main.
    let badentry = case_doc(
        "\n### Probe c\nrecipe: \"ac-av/console-run-v1\"\nentry: \"main\"\nexit: 0\n\n#### stdout\n\n```text\n1\n```\n\n#### stderr\n\n```text\n```\n",
    );
    assert!(
        decode(&badentry, "x.case.ad").is_err(),
        "console entry is d_main"
    );
    // Unwired recipe is a located error.
    let recipe = case_doc("\n### Probe r\nrecipe: \"ac-av/resident-run-v1\"\nentry: \"d_main\"\n");
    let e = decode(&recipe, "x.case.ad").unwrap_err();
    assert!(e.message.contains("未接入文档式试点"), "{e:?}");
    // Empty Auto section (no fence) is rejected.
    let nofence = "## Case: a\n\n### Auto\n";
    assert!(decode(nofence, "x.case.ad").is_err(), "Auto needs a fence");
    // Rust with neither a direct fence nor subsections is rejected.
    let bare = case_doc("\n### Rust\ncheck-id: \"rust\"\n");
    assert!(decode(&bare, "x.case.ad").is_err(), "Rust needs payload");
}

#[test]
fn final_newline_and_empty_stream_rules() {
    // Empty stream fence = zero bytes; console success may ride exit only.
    let doc = case_doc(
        "\n### Probe c\nrecipe: \"ac-av/console-run-v1\"\nentry: \"d_main\"\nexit: 0\n\n#### stdout\n\n```text\n```\n\n#### stderr\n\n```text\n```\n",
    );
    let cases = decode(&doc, "x.case.ad").unwrap();
    let p = &cases[0].spec.checks[0];
    assert_eq!(p.expect.stdout.as_ref().unwrap().text, "");
    assert_eq!(p.expect.stderr.as_ref().unwrap().text, "");
    // final-newline: false removes exactly one logical newline.
    let vm = case_doc("\n### VM result\nfinal-newline: false\n\n```text\n14\n```\n");
    let cases = decode(&vm, "x.case.ad").unwrap();
    assert_eq!(
        cases[0].spec.checks[0].expect.result.as_ref().unwrap().text,
        "14"
    );
    let vm_keep = case_doc("\n### VM result\n\n```text\n14\n```\n");
    let cases = decode(&vm_keep, "x.case.ad").unwrap();
    assert_eq!(
        cases[0].spec.checks[0].expect.result.as_ref().unwrap().text,
        "14\n"
    );
}

#[test]
fn mixed_format_file_is_rejected_at_discovery() {
    let root = std::env::temp_dir().join(format!("atc-mixed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("s.suite.ad"),
        "# s\n\n```test-suite\nsuite { schema: \"autolang.suite/1\", id: \"s\", include: [\"m.case.ad\"] }\n```\n",
    )
    .unwrap();
    std::fs::write(
        root.join("m.case.ad"),
        concat!(
            "## Case: a\n",
            "\n### Auto\n",
            "\n```auto\n",
            "1\n",
            "```\n",
            "\n### VM result\n",
            "\n```text\n1\n```\n",
            "\n```test-case\n",
            "case { schema: \"autolang.case/1\", id: \"old.x\", source: { text: \"1\", syntax: \"auto\", path: \"m.at\" }, checks: [{ id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\" } }] }\n",
            "```\n"
        ),
    )
    .unwrap();
    let err = load_suite(&root.join("s.suite.ad"), &Limits::defaults())
        .expect_err("mixed formats reject the whole load");
    assert!(err.detail.contains("mixed formats"), "{:?}", err);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn legacy_files_without_any_entry_still_report_clearly() {
    let root = std::env::temp_dir().join(format!("atc-empty-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("s.suite.ad"),
        "# s\n\n```test-suite\nsuite { schema: \"autolang.suite/1\", id: \"s\", include: [\"n.case.ad\"] }\n```\n",
    )
    .unwrap();
    std::fs::write(root.join("n.case.ad"), "# nothing here\nprose only\n").unwrap();
    let err = load_suite(&root.join("s.suite.ad"), &Limits::defaults())
        .expect_err("zero discovered cases is a hard load error");
    assert!(
        err.detail
            .contains("no `test-case` fence and no `## Case:` heading"),
        "{:?}",
        err
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn heading_shaped_payload_lines_are_payload() {
    // Fences containing heading/attr-shaped lines must not confuse the walk.
    let doc = concat!(
        "## Case: a\n",
        "\n### Auto\n",
        "\n````auto\n",
        "## not a case\n",
        "### not a section\n",
        "tags: [\"x\"]\n",
        "````\n",
        "\n### VM result\n",
        "\n```text\n1\n```\n",
    );
    let cases = decode(doc, "x.case.ad").unwrap();
    assert_eq!(cases.len(), 1);
    assert!(cases[0].spec.tags.is_empty(), "payload attr line is data");
}

#[test]
fn crlf_documents_normalize_to_logical_lf_payloads() {
    let doc = "## Case: a\r\n\r\n### Auto\r\n\r\n```auto\r\nfn main() int { return 1 }\r\n```\r\n\r\n### VM result\r\n\r\n```text\r\n7\r\n```\r\n";
    let cases = decode(doc, "x.case.ad").unwrap();
    assert_eq!(
        cases[0].spec.source.payload,
        PayloadSpec::Text("fn main() int { return 1 }\n".to_string()),
        "CRLF source normalizes to logical LF"
    );
    assert_eq!(
        cases[0].spec.checks[0].expect.result.as_ref().unwrap().text,
        "7\n",
        "default keeps the final logical newline"
    );
}
