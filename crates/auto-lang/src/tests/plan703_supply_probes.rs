// PLAN-703 供⑤ supply-endpoint probes (the PLAN-701 time-族 probe shape:
// minimal VM bridge + .at fn calling the native + envelope JSON assertions).
//
// AC-07: natives 9915/9916/9917 registered; envelope field-identical to the
// downstream replacement-seam contracts (auto-edit diff-view.md SD-01/
// PLAN-011/012): hunks 0-based half-open, rows 12 fields, adds/dels,
// truncated=false, degraded=false (engine era), err form for missing files
// (values not raises), CR tolerated.

#[cfg(test)]
mod plan703_diff_supply {
    use crate::parser::Parser;

    /// Minimal VM bridge (plan701 build_probe_bridge 同款骨架).
    fn build_probe_bridge(src: &str) -> crate::ui::vm_bridge::VmBridge {
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|st| match st {
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .expect("decl");
        let fns: Vec<crate::ast::Stmt> = ast
            .stmts
            .iter()
            .filter(|st| matches!(st, crate::ast::Stmt::Fn(_)))
            .cloned()
            .collect();
        let widget = crate::aura::extract_widget_from_decl(&decl).expect("extract");
        crate::ui::vm_bridge::VmBridge::new_with_imports(&widget, fns).expect("bridge")
    }

    struct TempFixture(std::path::PathBuf);
    impl TempFixture {
        fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!("p703_probe_{tag}_{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            TempFixture(p)
        }
        fn write(&self, name: &str, content: &str) -> String {
            let path = self.0.join(name);
            std::fs::write(&path, content).unwrap();
            // Forward slashes — no .at escape hazards on Windows paths.
            path.to_string_lossy().replace('\\', "/")
        }
        fn dir(&self) -> String {
            self.0.to_string_lossy().replace('\\', "/")
        }
    }
    impl Drop for TempFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn probe_bridge_with(body: &str) -> crate::ui::vm_bridge::VmBridge {
        let src = format!(
            concat!(
                "widget T703Probe {{\n",
                "    view {{\n",
                "        col {{\n",
                "            text \"probe\"\n",
                "        }}\n",
                "    }}\n",
                "}}\n",
                "{}\n"
            ),
            body
        );
        build_probe_bridge(&src)
    }

    fn call_str(bridge: &crate::ui::vm_bridge::VmBridge, name: &str) -> String {
        match bridge.call_vm_fn(name, &[]) {
            Ok(auto_val::Value::Str(s)) => s.to_string(),
            other => panic!("{name} must return Str, got {other:?}"),
        }
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn diff_files_envelope_fields_and_three_segment_marks() {
        let fx = TempFixture::new("files");
        let pa = fx.write("a.txt", "alpha\nbeta\ngamma\n");
        let pb = fx.write("b.txt", "alpha\nbeta2\ngamma\n");
        let bridge = probe_bridge_with(&format!(
            "fn probe_diff_files() str {{\n    return diff_files(\"{pa}\", \"{pb}\", 1)\n}}\n"
        ));
        let json = call_str(&bridge, "probe_diff_files");
        // Field-by-field (parse without serde — the envelope is small and
        // the assertions are positional; a tiny extractor keeps the probe
        // dependency-free).
        assert!(json.contains("\"hunks\":[{\"a1\":0,\"a2\":3,\"b1\":0,\"b2\":3}]"), "hunk field form (ctx=1 clamps a1 to 0): {json}");
        assert!(json.contains("\"adds\":1"), "adds: {json}");
        assert!(json.contains("\"dels\":1"), "dels: {json}");
        assert!(json.contains("\"truncated\":false"), "truncated: {json}");
        assert!(json.contains("\"degraded\":false"), "degraded (engine era): {json}");
        assert!(json.contains("\"err\":\"\""), "no error: {json}");
        // Paired row: beta → beta2 with three-segment marking (pre="beta",
        // mid "2" on the r side; 1-based lo/ro).
        assert!(
            json.contains("\"lo\":2,\"ro\":2,\"ln\":\"beta\",\"rn\":\"beta2\",\"lk\":\"del\",\"rk\":\"add\""),
            "paired row: {json}"
        );
        assert!(json.contains("\"lpre\":\"beta\",\"lmid\":\"\",\"lpost\":\"\""), "l side (empty mid): {json}");
        assert!(json.contains("\"rpre\":\"beta\",\"rmid\":\"2\",\"rpost\":\"\""), "r three-segment: {json}");
        // Ctx rows carry the full text in lpre.
        assert!(json.contains("\"lk\":\"ctx\",\"rk\":\"ctx\",\"lpre\":\"alpha\""), "ctx row: {json}");
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn diff_files_cr_tolerance() {
        let fx = TempFixture::new("crlf");
        let pa = fx.write("a.txt", "one\r\ntwo\r\n");
        let pb = fx.write("b.txt", "one\ntwo\n");
        let bridge = probe_bridge_with(&format!(
            "fn probe_diff_cr() str {{\n    return diff_files(\"{pa}\", \"{pb}\", 3)\n}}\n"
        ));
        let json = call_str(&bridge, "probe_diff_cr");
        assert!(json.contains("\"hunks\":[],\"rows\":[]"), "CRLF == LF → empty diff: {json}");
        assert!(json.contains("\"adds\":0,\"dels\":0"), "zero changes: {json}");
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn diff_files_missing_file_is_err_form_not_raise() {
        let fx = TempFixture::new("missing");
        let pa = fx.write("a.txt", "x\n");
        let bridge = probe_bridge_with(&format!(
            "fn probe_missing() str {{\n    return diff_files(\"{pa}\", \"{}/nope.txt\", 3)\n}}\n",
            fx.dir()
        ));
        let json = call_str(&bridge, "probe_missing");
        assert!(json.contains("\"hunks\":[],\"rows\":[]"), "err form empties hunks/rows: {json}");
        assert!(json.contains("\"err\":\"文件不存在"), "err carries the message: {json}");
        assert!(json.contains("\"degraded\":false"), "degraded stays false: {json}");
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn diff_dirs_envelope_states_and_counts() {
        let fx = TempFixture::new("dirs");
        std::fs::write(fx.0.join("mod.txt"), "old").unwrap();
        std::fs::write(fx.0.join("same.txt"), "keep").unwrap();
        std::fs::create_dir_all(fx.0.join("target")).unwrap();
        std::fs::write(fx.0.join("target/skip.js"), "a").unwrap();
        std::fs::create_dir_all(fx.0.join("btree")).unwrap();
        std::fs::write(fx.0.join("btree/new.txt"), "new").unwrap();
        std::fs::write(fx.0.join("btree/mod.txt"), "new-content").unwrap();

        let da = fx.0.join("a");
        let db = fx.0.join("b");
        std::fs::create_dir_all(&da).unwrap();
        std::fs::create_dir_all(&db).unwrap();
        std::fs::rename(fx.0.join("mod.txt"), da.join("mod.txt")).unwrap();
        std::fs::rename(fx.0.join("same.txt"), da.join("same.txt")).unwrap();
        std::fs::rename(fx.0.join("target"), da.join("target")).unwrap();
        std::fs::rename(fx.0.join("btree/new.txt"), db.join("new.txt")).unwrap();
        std::fs::rename(fx.0.join("btree/mod.txt"), db.join("mod.txt")).unwrap();
        // same.txt on BOTH sides (the "same" state fixture).
        std::fs::write(db.join("same.txt"), "keep").unwrap();
        let (da, db) = (
            da.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
            db.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
        );

        let bridge =
            probe_bridge_with(&format!("fn probe_dirs() str {{\n    return diff_dirs(\"{da}\", \"{db}\")\n}}\n"));
        let json = call_str(&bridge, "probe_dirs");
        // Five states: same / added / modified; skip-list hides target/.
        assert!(json.contains("\"rel\":\"same.txt\",\"status\":\"same\""), "{json}");
        assert!(json.contains("\"rel\":\"new.txt\",\"status\":\"added\""), "{json}");
        assert!(json.contains("\"rel\":\"mod.txt\",\"status\":\"modified\""), "{json}");
        assert!(!json.contains("target"), "skip-list hides build dirs: {json}");
        assert!(json.contains("\"counts\":{\"same\":1,\"added\":1,\"deleted\":0,\"modified\":1,\"binary\":0}"), "counts: {json}");
        assert!(json.contains("\"truncated\":false"), "{json}");
        assert!(json.contains("\"err\":\"\""), "{json}");
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn diff_dirs_missing_root_err_not_silent() {
        let fx = TempFixture::new("dirs_missing");
        let da = format!("{}/a", fx.dir());
        let db = format!("{}/b", fx.dir());
        std::fs::create_dir_all(&da).unwrap();
        let bridge =
            probe_bridge_with(&format!("fn probe_dirs_missing() str {{\n    return diff_dirs(\"{da}\", \"{db}\")\n}}\n"));
        let json = call_str(&bridge, "probe_dirs_missing");
        assert!(json.contains("\"entries\":[]"), "{json}");
        assert!(json.contains("\"err\":\"目录不存在"), "{json}");
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn diff_snapshots_reads_buffer_registry() {
        // Two live editors with distinct contents; the native reads the
        // registry snapshots directly (zero full-text VM transit). Headless
        // registry setup: default font-system install (idempotent) + the
        // registry test lock, mirroring the core test posture.
        use crate::ui::code_editor::{code_editor, code_editor_dispose, core, storage_key, CodeEditorConfig};
        core::ensure_font_system_call();
        let _guard = core::REGISTRY_TEST_LOCK.lock().unwrap();
        // The registry convention: `code_editor()` inserts under the key as
        // GIVEN; payload accessors look up `storage_key(key)` — register
        // under the storage key (core-test posture).
        let key_a = storage_key("p703-snap-a");
        let key_b = storage_key("p703-snap-b");
        code_editor_dispose(&key_a);
        code_editor_dispose(&key_b);
        let cfg = CodeEditorConfig::default();
        let core_a = code_editor(&key_a, &cfg);
        let core_b = code_editor(&key_b, &cfg);
        core_a.doc_replace("alpha\nbeta\ngamma\n");
        core_b.doc_replace("alpha\nbeta\ngamma\ndelta\n");

                // Direct Rust-side call first: the registry face must work without
        // the VM in the loop.
        let direct = crate::ui::code_editor::diff::envelope::diff_snapshots_envelope("p703-snap-a", "p703-snap-b");
        assert!(direct.contains("\"adds\":1"), "direct registry face: {direct}");
        let bridge = probe_bridge_with(
            "fn probe_snapshots() str {
    return diff_snapshots(\"p703-snap-a\", \"p703-snap-b\")
}
",
        );
        let json = call_str(&bridge, "probe_snapshots");
        assert!(json.contains("\"hunks\":[{\"a1\":0,\"a2\":4,\"b1\":0,\"b2\":5}]"), "append hunk (ctx=3 absorbs the 4-line doc): {json}");
        assert!(json.contains("\"adds\":1,\"dels\":0"), "counts: {json}");
        assert!(json.contains("\"err\":\"\""), "{json}");
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn diff_snapshots_missing_key_err_form() {
        let bridge = probe_bridge_with(
            "fn probe_snap_missing() str {\n    return diff_snapshots(\"p703-nope-a\", \"p703-nope-b\")\n}\n",
        );
        let json = call_str(&bridge, "probe_snap_missing");
        assert!(json.contains("\"hunks\":[],\"rows\":[]"), "{json}");
        assert!(json.contains("\"err\":\"编辑器不存在"), "{json}");
    }
}
