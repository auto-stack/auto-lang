//! PLAN-780 T-02 unit tests — grammar/static subset, dynamic classification
//! (no fake green), and error diagnosis. The old-vs-new corpus comparison
//! (V-02) lives in the auto-man probe adapter; these pin the model's own
//! contract.

use crate::*;
use auto_val::Value;

fn parse(text: &str) -> ProjectDescriptor {
    parse_document(text, SourceInfo::inline()).expect("parse should succeed")
}

// ── static grammar ─────────────────────────────────────────────────────

#[test]
fn flat_props_and_comments() {
    let d = parse(
        "// header\nname: \"notes\"\nversion: \"1.0.0\"\n/* block */\nrender: \"vue\" // trailing\n",
    );
    assert!(d.is_static_equivalent());
    assert_eq!(d.identity.name.as_deref(), Some("notes"));
    assert_eq!(d.identity.version.as_deref(), Some("1.0.0"));
    assert_eq!(d.render.render, RenderSpec::Single("vue".into()));
    assert_eq!(
        d.render.effective_backend.as_deref(),
        Some("vue"),
        "render wins over the lang-inferred backend"
    );
}

#[test]
fn dep_bare_name_node_with_path() {
    let d = parse("name: \"x\"\ndep bps {\n    path: \"../../packages/blueprints\"\n}\n");
    assert!(d.is_static_equivalent());
    assert_eq!(d.deps.len(), 1);
    assert_eq!(d.deps[0].name, "bps");
    assert_eq!(d.deps[0].name_form, DepNameForm::Bare);
    assert_eq!(d.deps[0].path.as_deref(), Some("../../packages/blueprints"));
    assert_eq!(d.deps[0].line, 2);
}

#[test]
fn dep_quoted_and_arg_forms_parse() {
    let d =
        parse("dep \"stylekit\" { path: \"../stylekit\" }\ndep(\"kernel\") { port: \"msvc\" }\n");
    assert!(d.is_static_equivalent());
    assert_eq!(d.deps[0].name_form, DepNameForm::Quoted);
    assert_eq!(d.deps[1].name_form, DepNameForm::Arg);
    assert_eq!(d.deps[1].name, "kernel");
}

#[test]
fn render_split_multi_and_legacy_unknown() {
    let split = parse("render: { front: \"vue\", back: \"rust\" }\n");
    assert_eq!(
        split.render.render,
        RenderSpec::Split {
            front: vec!["vue".into()],
            back: "rust".into()
        }
    );
    let multi = parse("backend: [\"vue\", \"arkts\"]\n");
    assert_eq!(
        multi.render.render,
        RenderSpec::Multi(vec!["vue".into(), "arkts".into()])
    );
    assert_eq!(
        multi.render.legacy_backend.as_deref(),
        Some("[\"vue\", \"arkts\"]")
    );
    // `backend: "rust-tauri"` (api-example/back): declared but not a known
    // backend name — oracle maps it to backend_config = None + legacy string.
    let unknown = parse("backend: \"rust-tauri\"\n");
    assert_eq!(
        unknown.render.render,
        RenderSpec::Unknown("rust-tauri".into())
    );
    assert_eq!(
        unknown.render.effective_backend.as_deref(),
        Some("rust-tauri")
    );
}

#[test]
fn lang_inferred_backend_fallback() {
    let d = parse("name: \"auto\"\nlang: \"rust\"\n");
    assert_eq!(d.render.effective_backend.as_deref(), Some("rust"));
    let c = parse("name: \"x\"\n");
    assert_eq!(c.render.effective_backend.as_deref(), Some("c"));
}

#[test]
fn members_array_and_scene() {
    let d = parse("scene: \"workspace\"\nmembers: [\"source/front\", \"source/back\"]\n");
    assert_eq!(d.runtime.scene, Some(SceneKind::Workspace));
    assert_eq!(
        d.members,
        Some(vec!["source/front".to_string(), "source/back".to_string()])
    );
}

#[test]
fn display_and_runtime_fields_normalized_like_pac() {
    let d = parse(
        "title: \"  Notes \"\ntheme: \"Dark\"\naccent: \"Indigo\"\ndesktop_render: \"Queue\"\nfront_port: 4046\nwindow: \"1440x900\"\nmedia_root: \"  \"\n",
    );
    assert_eq!(d.display.title.as_deref(), Some("Notes"));
    assert_eq!(d.display.theme.as_deref(), Some("dark"));
    assert_eq!(d.display.accent.as_deref(), Some("indigo"));
    assert_eq!(d.display.desktop_render.as_deref(), Some("queue"));
    assert_eq!(d.runtime.front_port, Some(4046));
    assert_eq!(d.runtime.window, Some(WindowSpec::Size(1440.0, 900.0)));
    assert_eq!(d.runtime.media_root, None, "blank media_root → None");
    let fit = parse("window: \"fit\"\n");
    assert_eq!(fit.runtime.window, Some(WindowSpec::Fit));
    // Pac bounds: below 200 or above 7680/4320 → None
    let bad = parse("window: \"100x9000\"\n");
    assert_eq!(bad.runtime.window, None);
    // unparseable port → None (Pac: parse().ok())
    let bad_port = parse("front_port: \"not-a-port\"\n");
    assert_eq!(bad_port.runtime.front_port, None);
}

#[test]
fn toggles_on_off_and_string_forms_agree_with_pac_rule() {
    let d = parse("shadcn: off\ndefault_classes: \"false\"\n");
    assert!(d.is_static_equivalent());
    assert!(matches!(&d.toggles.shadcn, ToggleValue::Str(s) if s == "off"));
    assert!(!d.toggles.shadcn.normalized());
    assert!(!d.toggles.default_classes.normalized());
    assert!(ToggleValue::Unset.normalized());
    assert!(ToggleValue::Str("on".into()).normalized());
    assert!(ToggleValue::Bool(true).normalized());
    assert!(!ToggleValue::Str("0".into()).normalized());
}

#[test]
fn external_backend_object_and_bare_string() {
    let obj = parse("back: { project: \"../ash-server\" }\n");
    assert_eq!(obj.external_backend.as_deref(), Some("../ash-server"));
    let bare = parse("back: \"../b\"\n");
    assert_eq!(bare.external_backend.as_deref(), Some("../b"));
    let obj_no_project = parse("back: { other: \"x\" }\n");
    assert_eq!(obj_no_project.external_backend, None);
}

#[test]
fn unmodeled_nodes_and_props_are_catalogued_not_dropped() {
    let d = parse(
        "description: \"kept verbatim\"\nnpm_deps: [\"marked@^12.0.0\"]\napp main {\n    at: \".\"\n    srcs: [\"main.at\"]\n}\nlib(\"auto\") {\n    recurse: true\n}\ntheme: { mode: \"dark\" }\n",
    );
    assert!(d.is_static_equivalent());
    let raw_keys: Vec<&str> = d.raw_props.iter().map(|p| p.key.as_str()).collect();
    assert!(raw_keys.contains(&"description"));
    assert!(raw_keys.contains(&"npm_deps"));
    assert!(
        raw_keys.contains(&"theme"),
        "theme block form is catalogued"
    );
    assert_eq!(d.raw_nodes.len(), 2, "app + lib nodes");
    let app = &d.raw_nodes[0];
    assert_eq!(app.head, "app");
    assert_eq!(app.name.as_deref(), Some("main"));
    assert_eq!(
        app.props.iter().find(|p| p.key == "srcs").map(|p| &p.value),
        Some(&Value::Array(auto_val::Array::from_vec(vec![Value::Str(
            "main.at".into()
        )])))
    );
    let lib = &d.raw_nodes[1];
    assert_eq!(lib.name.as_deref(), Some("auto"));
    assert_eq!(
        lib.props
            .iter()
            .find(|p| p.key == "recurse")
            .map(|p| &p.value),
        Some(&Value::Bool(true))
    );
}

#[test]
fn duplicate_keys_later_wins_and_recorded() {
    let d = parse("title: \"first\"\ntitle: \"second\"\n");
    assert_eq!(d.display.title.as_deref(), Some("second"));
    assert_eq!(d.duplicates.len(), 1);
    assert_eq!(d.duplicates[0].key, "title");
    assert_eq!(d.duplicates[0].line, 2);
}

#[test]
fn npm_deps_object_with_quoted_keys() {
    let d = parse("npm_deps: {\n  \"@autodown/engine\": \"link:D:/x\"\n}\n");
    assert!(d.is_static_equivalent());
    let npm = d
        .raw_props
        .iter()
        .find(|p| p.key == "npm_deps")
        .expect("npm_deps catalogued");
    assert!(matches!(npm.value, Some(Value::Obj(_))));
}

#[test]
fn provenance_records_field_lines() {
    let d = parse("name: \"a\"\nrender: \"vm\"\ndep x { path: \"y\" }\n");
    let name_line = d
        .provenance
        .iter()
        .find(|(f, _)| *f == "name")
        .map(|(_, l)| *l);
    assert_eq!(name_line, Some(1));
    assert_eq!(d.deps[0].line, 3);
}

// ── dynamic classification (V-03): never fake green ────────────────────

#[test]
fn interpolation_is_classified_dynamic() {
    let d = parse("name: \"${name}\"\n");
    assert!(!d.is_static_equivalent());
    assert_eq!(d.unsupported[0].kind, UnsupportedKind::Interpolation);
    assert_eq!(d.identity.name, None, "field is not faked");
    assert_eq!(d.unsupported[0].raw, "${name}", "raw text preserved");
}

#[test]
fn dollar_ident_and_backtick_and_fstring_are_dynamic() {
    for (text, kind) in [
        ("x: \"hi $name\"\n", UnsupportedKind::Interpolation),
        ("x: `tpl`\n", UnsupportedKind::FString),
        ("x: f\"tpl\"\n", UnsupportedKind::FString),
    ] {
        let d = parse(text);
        assert!(!d.is_static_equivalent(), "{text}");
        assert_eq!(d.unsupported[0].kind, kind, "{text}");
    }
    // plain $ without ident/{ stays literal (e.g. "cost $5")
    let d = parse("x: \"cost $5\"\n");
    assert!(d.is_static_equivalent());
}

#[test]
fn bare_ident_assignment_and_call_are_dynamic() {
    for (text, kind) in [
        ("x: undefined_thing\n", UnsupportedKind::BareIdentValue),
        ("x = 5\n", UnsupportedKind::Assignment),
        ("x: make_value(1)\n", UnsupportedKind::Call),
        ("theme: on\n", UnsupportedKind::OnOffOutsideToggle),
        ("x: 'single'\n", UnsupportedKind::SingleQuotedString),
    ] {
        let d = parse(text);
        assert!(!d.is_static_equivalent(), "{text}");
        assert_eq!(d.unsupported[0].kind, kind, "{text}");
    }
}

#[test]
fn interpolation_inside_structured_value_is_dynamic() {
    let d = parse("cfg: { front: \"${base}/src\" }\n");
    assert!(!d.is_static_equivalent());
    assert_eq!(d.unsupported[0].kind, UnsupportedKind::Interpolation);
    let d = parse("members: [\"${a}\"]\n");
    assert!(!d.is_static_equivalent());
}

#[test]
fn dep_with_unnameable_args_is_classified() {
    let d = parse("dep(\"a\", \"b\") { path: \"x\" }\n");
    assert!(!d.is_static_equivalent());
    assert_eq!(
        d.unsupported[0].kind,
        UnsupportedKind::DepNameUnrepresentable
    );
    assert_eq!(d.deps.len(), 0);
    assert_eq!(d.raw_nodes.len(), 1, "raw node preserved for audit");
}

#[test]
fn members_wrong_shape_is_type_mismatch_not_empty_vec() {
    let d = parse("members: \"front\"\n");
    assert!(!d.is_static_equivalent());
    assert_eq!(
        d.unsupported[0].kind,
        UnsupportedKind::TypeMismatch { field: "members" }
    );
    assert_eq!(d.members, None);
}

// ── malformed input → staged error (V-05) ──────────────────────────────

#[test]
fn malformed_inputs_error_with_stage_and_position() {
    // missing value
    let e = parse_document("name:\n", SourceInfo::inline()).unwrap_err();
    assert_eq!(e.stage, ErrorStage::Value);
    assert_eq!(e.line, 1);
    // missing closing brace in structured value
    let e = parse_document("x: [\"a\"\n", SourceInfo::inline()).unwrap_err();
    assert!(e.message.contains("unterminated"), "{}", e);
    assert_eq!(e.stage, ErrorStage::Value);
    // unterminated node body
    let e = parse_document("dep x {\n  path: \"y\"\n", SourceInfo::inline()).unwrap_err();
    assert_eq!(e.stage, ErrorStage::NodeBody);
    // trailing junk after value
    let e = parse_document("x: \"a\" \"b\"\n", SourceInfo::inline()).unwrap_err();
    assert_eq!(e.stage, ErrorStage::Value);
    assert!(e.message.contains("trailing"), "{}", e);
    // node without body
    let e = parse_document("dep x\n", SourceInfo::inline()).unwrap_err();
    assert_eq!(e.stage, ErrorStage::Entry);
    // source path flows into diagnostics
    let e = parse_document("x:\n", SourceInfo::from_path("some/pac.at")).unwrap_err();
    assert_eq!(
        e.source_path.as_deref(),
        Some(std::path::Path::new("some/pac.at"))
    );
    assert!(e.to_string().contains("some/pac.at"));
    assert!(e.to_string().contains("value"));
}

#[test]
fn escape_sequences_mirror_atom_parser() {
    let d = parse("x: \"a\\tb\\nc\\\"d\\\\e\\q\"\n");
    match &d.raw_props[0].value {
        Some(Value::Str(s)) => assert_eq!(s.as_str(), "a\tb\nc\"d\\e\\q"),
        other => panic!("expected Str, got {other:?}"),
    }
}

// ── R1 review regressions (F780-R1-01..05) — permanent controls ────────

/// F-01: bare identifiers in structured VALUE positions classify dynamic —
/// AtomParser would silently stringify them (`members: [port]` would fake
/// `["port"]` static while the oracle evaluates the injected arg).
#[test]
fn r1_structured_value_bare_idents_are_dynamic() {
    for (text, field) in [
        ("members: [port]\n", "array element"),
        ("back: { project: port }\n", "object value"),
        ("cfg: { front: make_path(1) }\n", "call in object value"),
        ("members: [on]\n", "on/off in container"),
        ("members: [dep_like { a: 1 }]\n", "node in value position"),
    ] {
        let d = parse(text);
        assert!(!d.is_static_equivalent(), "{field}: {text}");
        assert!(!d.unsupported.is_empty(), "{field} needs a reason: {text}");
        // the field itself never fakes a value
        if text.starts_with("members") {
            assert_eq!(d.members, None);
        }
    }
    // top-level object document: same rule at every value entry
    let d = parse("{ name: \"dynamic\", members: [port] }");
    assert!(!d.is_static_equivalent());
    assert_eq!(d.unsupported[0].kind, UnsupportedKind::BareIdentValue);
    // quoted static containers stay static (positive control)
    let ok = parse("members: [\"front\"]\nback: { project: \"back\" }\n");
    assert!(ok.is_static_equivalent());
    assert_eq!(
        ok.members,
        Some(vec!["front".to_string()]),
        "clean container still parses statically"
    );
    // interpolation inside structured values still classifies
    let d = parse("members: [\"${a}\"]\n");
    assert!(!d.is_static_equivalent());
    assert_eq!(d.unsupported[0].kind, UnsupportedKind::Interpolation);
}

/// F-02: duplicate dep body keys — LAST declaration wins (mirrors the
/// config VM accumulation), every occurrence stays recorded.
#[test]
fn r1_dep_duplicate_path_last_wins() {
    let d = parse("dep foo {\n path: \"first\"\n path: \"second\"\n}\n");
    assert!(d.is_static_equivalent());
    assert_eq!(d.deps[0].path.as_deref(), Some("second"));
    assert_eq!(d.deps[0].props.len(), 2, "both declarations preserved");
    assert_eq!(d.deps[0].prop_duplicates.len(), 1);
    assert_eq!(d.deps[0].prop_duplicates[0].key, "path");
    assert_eq!(d.deps[0].props[1].line, 3, "provenance line kept");
}

/// F-03: unmodeled node subtrees and args survive on the DTO.
#[test]
fn r1_unknown_node_children_and_args_preserved() {
    let d = parse("dep foo {\n child Extra { marker: \"__CHILD_SENTINEL__\" }\n}\n");
    assert!(d.is_static_equivalent());
    assert_eq!(d.deps.len(), 1);
    assert_eq!(d.deps[0].children.len(), 1, "dep child not dropped");
    assert_eq!(d.deps[0].children[0].head, "child");
    assert_eq!(d.deps[0].children[0].name.as_deref(), Some("Extra"));
    assert_eq!(
        d.deps[0].children[0].props[0].value,
        Value::Str("__CHILD_SENTINEL__".into())
    );

    let d = parse("custom(\"__ARG_ONE__\", \"__ARG_TWO__\") { tag: \"kept\" }\n");
    let repr = format!("{d:?}");
    assert!(repr.contains("__ARG_ONE__") && repr.contains("__ARG_TWO__"));
    assert_eq!(d.raw_nodes.len(), 1);
    assert!(d.raw_nodes[0].raw_args.is_some());
}

/// F-05: the legacy `root { … }` wrapper is explicitly excluded from the
/// frozen static subset — classified, node preserved, no fake equivalence.
#[test]
fn r1_root_wrapper_is_not_static() {
    let d = parse("root {\n name: \"wrapped\"\n render: \"vue\"\n}\n");
    assert!(!d.is_static_equivalent());
    assert_eq!(d.unsupported[0].kind, UnsupportedKind::RootWrapper);
    assert_eq!(d.raw_nodes.len(), 1, "wrapper node preserved verbatim");
    assert_eq!(d.raw_nodes[0].head, "root");
    assert_eq!(d.identity.name, None, "no hoisted fake value");
}

/// R1 difference control: a legal EMPTY file is a valid empty config on
/// both arms — distinct from an unreadable required input.
#[test]
fn r1_empty_file_is_legal_empty_config() {
    let d = parse("");
    assert!(d.is_static_equivalent());
    assert!(d.unsupported.is_empty());
    assert_eq!(d.identity.name, None);
}

/// R1 comment_eof control: unterminated block comment at EOF accepted
/// (mirrors the legacy lexer synthesizing CommentEnd).
#[test]
fn r1_unterminated_block_comment_eof_accepted() {
    let d = parse("name: \"ok\"\n/* missing close");
    assert!(d.is_static_equivalent());
    assert_eq!(d.identity.name.as_deref(), Some("ok"));
}

// ── R2 review regressions (F780-R2-01/02) — termination & boundary matrix ──

/// Deadline-protected parse (R2 F-01): a wedged scan fails the test within
/// the deadline instead of hanging the suite. Returns the parse outcome —
/// a syntax `Err` is a legitimate result (malformed static input); only a
/// timeout counts as non-termination. The worker thread is leaked on
/// timeout; process exit reaps it.
fn parse_outcome_with_deadline(
    text: &str,
    ms: u64,
) -> Option<Result<ProjectDescriptor, crate::ProjectModelError>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let owned = text.to_string();
    std::thread::spawn(move || {
        let _ = tx.send(crate::parse_document(&owned, SourceInfo::inline()));
    });
    rx.recv_timeout(std::time::Duration::from_millis(ms)).ok()
}

/// R2 F-01: adversarial value-position inputs must TERMINATE and classify —
/// the review's four timeout cases plus the boundary matrix (unknown tokens,
/// mismatched closers, missing colons, expressions) at every value entry
/// (array / object / top-level object document).
#[test]
fn r2_value_scanner_terminates_and_classifies() {
    let cases = [
        // review timeout repros
        ("members: [1 * 2]\n", "expression in array"),
        ("members: [1 : 2]\n", "stray separator in array"),
        ("members: [1}\n", "mismatched closer"),
        ("back: { @: 1 }\n", "unknown char in key position"),
        // boundary matrix extensions
        ("members: [*]\n", "operator element"),
        ("cfg: { a: 1 * 2 }\n", "expression in object value"),
        ("cfg: { a: : 1 }\n", "double colon"),
        ("cfg: { a 1 }\n", "missing colon"),
        (
            "{ \"members\": [1 * 2] }\n",
            "expression in top-level object doc",
        ),
        ("{ @: 1 }\n", "unknown key in top-level doc"),
        ("members: [1 2 3]\n", "adjacent numbers (no separator)"),
        ("cfg: { a: }\n", "missing value in object"),
    ];
    for (text, why) in cases {
        let Some(outcome) = parse_outcome_with_deadline(text, 2000) else {
            panic!("scanner failed to terminate within deadline — {why}: {text:?}");
        };
        match outcome {
            // classified dynamic — never a static claim without reasons
            Ok(d) => {
                assert!(
                    !d.is_static_equivalent(),
                    "{why}: {text:?} must not claim static equivalence"
                );
                assert!(
                    !d.unsupported.is_empty(),
                    "{why}: {text:?} needs a classification reason"
                );
            }
            // malformed static input — a staged syntax error is the honest
            // outcome (V-05), equally terminating
            Err(e) => assert!(
                matches!(
                    e.stage,
                    crate::ErrorStage::Value | crate::ErrorStage::Document
                ),
                "{why}: {text:?} unexpected error stage {:?}",
                e.stage
            ),
        }
    }
    // positive controls still parse statically within the deadline
    for text in [
        "members: [\"front\", \"back\"]\n",
        "back: { project: \"back\" }\n",
        "cfg: { a: 1, b: [2, 3] }\n",
    ] {
        let Some(Ok(d)) = parse_outcome_with_deadline(text, 2000) else {
            panic!("clean input failed to terminate or errored: {text:?}");
        };
        assert!(d.is_static_equivalent(), "{text:?}");
    }
}

// ── R3 review regressions (F780-R3-01) — trivia/separation matrix ─────

/// R3 F-01: legal frozen syntax (comments around values/separators/closers,
/// newline-separated builtins) must stay STATIC. Review cases 0–4 plus the
/// placement × value-kind × container matrix; same-line adjacency stays a
/// negative control (covered in r2 above).
#[test]
fn r3_trivia_separation_matrix_stays_static() {
    let legal = [
        // review failing cases (old arm Ok — review combinations.json)
        (
            "members: [\"a\" // comment\n, \"b\"]\n",
            Some(vec!["a", "b"]),
        ),
        (
            "members: [\"a\" /* block */ , \"b\"]\n",
            Some(vec!["a", "b"]),
        ),
        ("cfg: { a: true\nb: false }\n", None),
        ("cfg: [true\nfalse]\n", None),
        (
            "{ name: \"ok\", back: { project: \"b\" /* comment */ } }",
            None,
        ),
        // review passing cases (keep pinned)
        ("members: [\"a\", \"b\",]\n", Some(vec!["a", "b"])),
        ("members: [\n1\n2\n]\n", Some(vec!["1", "2"])),
        ("members: [ // leading comment\n \"a\"]\n", Some(vec!["a"])),
        ("cfg: [true, false]\n", None),
        // matrix extensions: comment placement × value kind × container
        ("members: [1 /* v */ , 2]\n", Some(vec!["1", "2"])),
        ("members: [true // c\n, false]\n", None),
        (
            "members: [\"a\" , // after comma\n \"b\"]\n",
            Some(vec!["a", "b"]),
        ),
        ("members: [\"a\" /* c */]\n", Some(vec!["a"])),
        ("members: [\"a\" // c\n]\n", Some(vec!["a"])),
        ("cfg: { a: \"x\" /* v */ , b: 2 }\n", None),
        ("cfg: { a: true /* v */ }\n", None),
        ("cfg: { /* k */ a: 1 }\n", None),
        ("{ \"members\": [\"a\" /* c */] }\n", Some(vec!["a"])),
        ("{ \"members\": [true\nfalse] }\n", None),
        // multi-line block comment separates
        (
            "members: [\"a\" /* span\nlines */ , \"b\"]\n",
            Some(vec!["a", "b"]),
        ),
    ];
    for (text, want_members) in legal {
        let d = parse(text);
        assert!(
            d.is_static_equivalent(),
            "legal input misclassified: {text:?} — {:?}",
            d.unsupported
        );
        if let Some(want) = want_members {
            assert_eq!(
                d.members,
                Some(want.into_iter().map(String::from).collect::<Vec<_>>()),
                "{text:?}"
            );
        }
    }
    // negative control: same-line adjacency still classifies (no separator,
    // no line break — even with a same-line block comment between)
    for text in [
        // object comma AFTER a passed break / doubled — VM oracle: Err
        "cfg: { a: 1\n, b: 2 }\n",
        "cfg: { a: 1 // c\n, b: 2 }\n",
        "cfg: { a: 1,, b: 2 }\n",
        // array comma at element position — VM oracle: Err
        "cfg: [\n, 1]\n",
        "members: [1 2]\n",
        "members: [\"a\" /* c */ \"b\"]\n",
        "cfg: { a: 1 b: 2 }\n",
    ] {
        let d = parse(text);
        assert!(!d.is_static_equivalent(), "{text:?}");
        assert!(!d.unsupported.is_empty(), "{text:?}");
    }
}

// ── R4 review regressions (F780-R4-01) — comment-internal newlines ────

/// R4 F-01: a newline inside a block comment is a source-position fact,
/// NOT a syntactic separator — the VM treats the whole comment as trivia.
/// The review's three blocking cases plus the paired real-newline vs
/// comment-newline matrix (legal forms assert static + values; VM-rejected
/// forms assert classification).
#[test]
fn r4_comment_newline_is_not_a_separator() {
    // (input, expected static)
    let cases: Vec<(&str, bool)> = vec![
        // review blocking case 0: legal, must stay static with the field
        ("back: { project: \"b\" /*multi\nline*/, extra: 1 }\n", true),
        // review blocking case 1: VM "Expected infix operator" → classify
        ("back: { project: \"b\" /*multi\nline*/ extra: 1 }\n", false),
        // review blocking case 2: VM rejects adjacency → classify
        ("members: [\"a\" /*multi\nline*/ \"b\"]\n", false),
        // paired real-newline vs comment-only-newline
        ("members: [\"a\" /*c*/ \"b\"]\n", false), // same-line w/ comment
        ("members: [\"a\"\n\"b\"]\n", true),       // real newline separates
        ("cfg: { a: true /*c*/ false }\n", false), // adjacency
        ("cfg: { a: true\nb: false }\n", true),    // real newline separates pairs (R4 id 3)
        ("cfg: [true /*multi\nline*/ false]\n", false), // comment-newline ≠ separation
        ("cfg: [true\nfalse]\n", true),            // real newline separates
        // CRLF real break (R4 matrix id 12/13)
        ("members: [\"a\"\r\n\"b\"]\n", true),
        ("back: { project: \"b\",\r\n extra: 1 }\n", true),
        // nested builtin newline inside object value (id 14)
        ("cfg: { a: true /*c*/, b: [false\ntrue] }\n", true),
        // single-line block comment forms (id 5/7/8/15)
        (
            "{ name: \"ok\" /*comment*/, back: {project: \"b\"} }\n",
            true,
        ),
        ("cfg: { a: true, /*c*/ b: false }\n", true),
        ("members: [\"a\" /*c*/ \"b\"]\n", false),
    ];
    for (text, want_static) in cases {
        let d = parse(text);
        assert_eq!(
            d.is_static_equivalent(),
            want_static,
            "{text:?} — unsupported: {:?}",
            d.unsupported
        );
    }
    // case 0 keeps its field (the R4 failure dropped external_backend)
    let d = parse("back: { project: \"b\" /*multi\nline*/, extra: 1 }\n");
    assert_eq!(d.external_backend.as_deref(), Some("b"));
    // top-object paired control
    let d = parse("{ \"members\": [\"a\" /*multi\nline*/ \"b\"] }");
    assert!(!d.is_static_equivalent());
    // `#` comments INSIDE structured values are VM-illegal (R4 oracle:
    // "Expected infix operator, got Hash" — matrix ids 3/4/11 all old-Err):
    // the model refuses consistently, never claims static.
    for text in [
        "cfg: { a: true # line\nb: false }\n",
        "members: [\"a\" # line\n, \"b\"]\n",
        "members: [\"a\", #c\n \"b\"]\n",
    ] {
        let outcome = crate::parse_document(text, SourceInfo::inline());
        match outcome {
            Ok(d) => assert!(!d.is_static_equivalent(), "{text:?}"),
            Err(e) => assert!(
                matches!(
                    e.stage,
                    crate::ErrorStage::Value | crate::ErrorStage::Document
                ),
                "{text:?}: unexpected stage {:?}",
                e.stage
            ),
        }
    }
}
