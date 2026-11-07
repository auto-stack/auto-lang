//! PLAN-950: actual restored module source -> per-function AV and Rust
//! consumers. These tests observe real code/fragment identity and outputs.
#![cfg(all(windows, target_arch = "x86_64"))]
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::{modules::ModuleA2xTarget, Assembly};
use ace_av::modules::ModuleAvTarget;
use ace_core::source::{
    cst::FrontBudget,
    modules::{ModuleInput, MODULES_CAPABILITY},
};
use ace_source_query::{
    modules::{compile_module_product, ModuleReceipt, ModuleSession, ModuleStore},
    SourceQuerySession,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
const B: FrontBudget = FrontBudget::WORK_OWNED_64MIB;
const QUOTA: u64 = 256 * 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let base = std::env::var_os("AUTO_PLAN_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        Self(base.join(format!(
            "module-consumers-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn source(local: bool) -> Vec<ModuleInput> {
    vec![
        ModuleInput {
            id: "std".into(),
            source: "pub fn answer() int { return 42 } fn helper() int { return 9 }".into(),
        },
        ModuleInput {
            id: "app".into(),
            source: format!(
                "use std.answer\nfn answer() int {{ return 7 }} fn main() int {{ return {} }}",
                if local { "answer()" } else { "std.answer()" }
            ),
        },
    ]
}
pub(super) fn rust_value(assembly: &Assembly, dir: &Path, expected: i32) {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in &assembly.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("module.exe");
    let mut cmd = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"])
        .arg(&exe)
        .current_dir(dir);
    let compile = auto_tool_execution::run_command_capture(
        cmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "{}",
        String::from_utf8_lossy(compile.stderr.as_deref().unwrap_or(&[]))
    );
    let mut cmd = Command::new(&exe);
    cmd.current_dir(dir);
    let run = auto_tool_execution::run_command_capture(
        cmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    assert_eq!(
        String::from_utf8_lossy(run.stdout.as_deref().unwrap()).trim(),
        format!("value:{expected}")
    );
}
fn targets(av: ModuleReceipt, rust: ModuleReceipt) -> BTreeMap<String, ModuleReceipt> {
    BTreeMap::from([("av".into(), av), ("a2x-rust".into(), rust)])
}
fn changed_target(
    path: &Path,
    receipt: &ModuleReceipt,
    change: impl FnOnce(&mut serde_json::Value),
) -> ModuleReceipt {
    let store = ace_artifacts::store::ManagedStore::open(path).unwrap();
    let original = store
        .get_artifact_by_id(&receipt.artifact)
        .unwrap()
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&original.payload).unwrap();
    change(&mut value);
    let payload = serde_json::to_vec(&value).unwrap();
    let digest = ace_core::runtime_contract::sha256_hex(&payload);
    let edges = store
        .required_targets(&receipt.artifact)
        .unwrap()
        .into_iter()
        .map(|r| (r, ace_artifacts::identity::RefKind::RequiredData))
        .collect();
    let artifact = store
        .put_artifact(ace_artifacts::store::PutRequest {
            layer: "NEG950",
            key: ace_artifacts::identity::QueryKey {
                layer: "NEG950",
                components: vec![("payload".into(), digest.clone())],
            },
            kind: ace_artifacts::codec::ChunkKind::SnapshotRecord,
            payload,
            edges,
            project: None,
        })
        .unwrap();
    ModuleReceipt { artifact, digest }
}
fn run_case(local: bool) {
    let temp = Temp::new();
    let store_path = temp.0.join("store");
    let caps = vec![MODULES_CAPABILITY.into()];
    let initial = source(local);
    let expected = if local { 7 } else { 42 };
    let mut store = ModuleStore::open(&store_path, "module-test").unwrap();
    let mut query = SourceQuerySession::new("pkg", "app");
    let product = compile_module_product(&initial, "app", &caps, &mut query, B).unwrap();
    let mut session = ModuleSession::from_product(&product, query).unwrap();
    let av = ModuleAvTarget::cold(&store_path, &product).unwrap();
    assert_eq!(av.run().unwrap(), expected);
    let mut rust = ModuleA2xTarget::cold(
        &store_path,
        "module-test",
        "pkg",
        QUOTA,
        RustTargetProvider::v1(),
        &product,
    )
    .unwrap();
    rust_value(rust.current(), &temp.0.join("cold"), expected);
    let old_files = rust.current().files.clone();
    let original = session
        .prepare_current_receipt(
            &mut store,
            &targets(av.receipt().unwrap(), rust.receipt().unwrap()),
        )
        .unwrap();
    store.publish(&original, None, false).unwrap();
    drop(product);
    drop(session);
    drop(av);
    drop(rust);
    drop(store);

    let mut store = ModuleStore::open(&store_path, "module-test").unwrap();
    let ((recovery, work), syntax) = ace_core::source::parser::capture_syntax_work(|| {
        ace_core::body_work::capture_body_work(|| {
            store.restore(&original, &initial, "app", &caps, "pkg", "app", B)
        })
    });
    let recovery = recovery.unwrap();
    assert!(work.is_empty());
    assert!(syntax.parsed_functions.is_empty() && syntax.tokenized_ranges.is_empty());
    assert_eq!(recovery.session.counters().solver_runs, 0);
    let bad_av = changed_target(&store_path, &recovery.targets["av"], |v| {
        v["schema"] = serde_json::json!("foreign-av")
    });
    assert!(
        ModuleAvTarget::restore(&store_path, &bad_av, &recovery.bound)
            .err()
            .unwrap()
            .contains("source/schema/capability")
    );
    let bad_rust = changed_target(&store_path, &recovery.targets["a2x-rust"], |v| {
        let fields = v["target_inputs"].as_array_mut().unwrap();
        let print = fields.iter_mut().find(|f| f[0] == "print_version").unwrap();
        print[1] = serde_json::json!("foreign-print");
    });
    assert!(ModuleA2xTarget::restore(
        &store_path,
        "module-test",
        "pkg",
        QUOTA,
        RustTargetProvider::v1(),
        &bad_rust,
        &recovery.bound
    )
    .err()
    .unwrap()
    .contains("producer/source/runtime"));
    let mut av =
        ModuleAvTarget::restore(&store_path, &recovery.targets["av"], &recovery.bound).unwrap();
    let mut rust = ModuleA2xTarget::restore(
        &store_path,
        "module-test",
        "pkg",
        QUOTA,
        RustTargetProvider::v1(),
        &recovery.targets["a2x-rust"],
        &recovery.bound,
    )
    .unwrap();
    assert_eq!(av.lowered, 0);
    assert_eq!((rust.stats().lower, rust.stats().print), (0, 0));
    assert_eq!(av.run().unwrap(), expected);
    assert_eq!(rust.current().files, old_files);
    rust_value(rust.current(), &temp.0.join("restored"), expected);
    let old_codes: Vec<_> = av.current().code().iter().cloned().collect();
    let old_fragments = rust.fragment_receipts();
    // A valid edit from a different source base cannot stamp its source hash
    // onto this target's unchanged functions.
    let mut foreign_inputs = initial.clone();
    let app = foreign_inputs.iter_mut().find(|m| m.id == "app").unwrap();
    app.source = app.source.replace("return 7", "return 8");
    let mut foreign_query = SourceQuerySession::new("pkg", "app");
    let foreign_product =
        compile_module_product(&foreign_inputs, "app", &caps, &mut foreign_query, B).unwrap();
    let mut foreign = ModuleSession::from_product(&foreign_product, foreign_query).unwrap();
    let foreign_edit = foreign
        .prepare("std", "answer", "pub fn answer() int { return 43 }", B)
        .unwrap();
    assert!(av
        .prepare(&foreign_edit)
        .err()
        .unwrap()
        .contains("source base"));
    assert!(rust
        .prepare(&foreign_edit)
        .err()
        .unwrap()
        .contains("source base"));
    assert_eq!(av.lowered, 0);
    assert_eq!((rust.stats().lower, rust.stats().print), (0, 0));
    let reopen_old = |label: &str| {
        let reopened = ModuleStore::open(&store_path, "module-test").unwrap();
        let old = reopened
            .restore(&original, &initial, "app", &caps, "pkg", "app", B)
            .unwrap();
        assert_eq!(old.session.counters().solver_runs, 0);
        let av = ModuleAvTarget::restore(&store_path, &old.targets["av"], &old.bound).unwrap();
        let rust = ModuleA2xTarget::restore(
            &store_path,
            "module-test",
            "pkg",
            QUOTA,
            RustTargetProvider::v1(),
            &old.targets["a2x-rust"],
            &old.bound,
        )
        .unwrap();
        assert_eq!(av.lowered, 0);
        assert_eq!((rust.stats().lower, rust.stats().print), (0, 0));
        assert_eq!(av.run().unwrap(), expected);
        assert_eq!(rust.current().files, old_files);
        rust_value(rust.current(), &temp.0.join(label), expected);
    };
    let mut session = ModuleSession::from_recovery(recovery);
    let ((edit, work), syntax) = ace_core::source::parser::capture_syntax_work(|| {
        ace_core::body_work::capture_body_work(|| {
            session.prepare("std", "answer", "pub fn answer() int { return 43 }", B)
        })
    });
    let mut edit = edit.unwrap();
    assert_eq!(work.len(), 1);
    assert_eq!(syntax.parsed_functions.len(), 1);
    let w = work["m3_std_answer"];
    // PLAN-938 (8l merge ruling): the warm edit runs the real TypeSupply
    // path, which does not invoke the legacy typecheck stage.
    assert_eq!((w.resolve, w.typecheck, w.plan, w.bind), (1, 0, 1, 1));
    assert_eq!(session.query.counters().solver_runs, 1);
    let av_edit = av.prepare(&edit).unwrap();
    let rust_edit = rust.prepare(&edit).unwrap();
    assert_eq!(av.lowered, 1);
    assert_eq!((rust.stats().lower, rust.stats().print), (1, 1));
    assert!(
        session
            .prepare_receipt_targets(&mut store, &mut edit, session.targets.clone())
            .is_err(),
        "old target receipts cannot be rebound to new source"
    );
    let next = session
        .prepare_receipt_targets(
            &mut store,
            &mut edit,
            targets(
                av.prepared_receipt(&av_edit).unwrap(),
                rust.prepared_receipt(&rust_edit).unwrap(),
            ),
        )
        .unwrap();
    assert_eq!(session.query.counters().solver_runs, 1);
    // The real source publication rejects cancellation after both targets are
    // ready. Current executable handles and emitted old bytes still run old.
    assert!(store.publish(&next, Some(&original), true).is_err());
    reopen_old("after-cancel");
    assert_eq!(av.run().unwrap(), expected);
    assert_eq!(rust.current().files, old_files);
    assert!(store
        .restore(&original, &initial, "app", &caps, "pkg", "app", B)
        .is_ok());
    let injector = ace_artifacts::store::ManagedStore::open(&store_path).unwrap();
    injector.db().execute_batch("CREATE TRIGGER fail_module_install BEFORE INSERT ON roots WHEN NEW.kind='source-modules-current' BEGIN SELECT RAISE(ABORT,'injected module install failure'); END;").unwrap();
    assert!(store
        .publish(&next, Some(&original), false)
        .unwrap_err()
        .contains("injected module install failure"));
    assert_eq!(av.run().unwrap(), expected);
    assert_eq!(rust.current().files, old_files);
    injector
        .db()
        .execute_batch("DROP TRIGGER fail_module_install")
        .unwrap();
    reopen_old("after-insert-failure");
    injector.db().execute_batch("CREATE TRIGGER fail_module_retire BEFORE DELETE ON roots WHEN OLD.kind='source-modules-current' BEGIN SELECT RAISE(ABORT,'injected module retire failure'); END;").unwrap();
    assert!(store
        .publish(&next, Some(&original), false)
        .unwrap_err()
        .contains("injected module retire failure"));
    assert_eq!(
        injector
            .root_artifacts("module-test", "source-modules-current")
            .unwrap(),
        vec![original.artifact.clone()]
    );
    injector
        .db()
        .execute_batch("DROP TRIGGER fail_module_retire")
        .unwrap();
    reopen_old("after-delete-failure");
    drop(injector);
    assert!(store
        .restore(&original, &initial, "app", &caps, "pkg", "app", B)
        .is_ok());
    session.validate(&edit).unwrap();
    av.validate(&av_edit).unwrap();
    rust.validate(&rust_edit).unwrap();
    store.publish(&next, Some(&original), false).unwrap();
    av.commit(av_edit).unwrap();
    rust.commit(rust_edit).unwrap();
    session.commit(edit).unwrap();
    let new_expected = if local { 7 } else { 43 };
    assert_eq!(av.run().unwrap(), new_expected);
    rust_value(rust.current(), &temp.0.join("edited"), new_expected);
    for code in &old_codes {
        let next = av.current().find_code(code.name()).unwrap();
        assert_eq!(
            Arc::ptr_eq(code, next),
            code.name() != "d_m3_std_answer",
            "{}",
            code.name()
        );
    }
    let new_fragments = rust.fragment_receipts();
    for (name, id) in old_fragments {
        assert_eq!(
            new_fragments[&name] == id,
            name != "m3_std_answer",
            "{name}"
        );
    }
    let changed: Vec<_> = session
        .bound
        .segments
        .iter()
        .map(|s| ModuleInput {
            id: s.module.clone(),
            source: s.source.clone(),
        })
        .collect();
    // A private replacement is a same-type visibility counterexample. It
    // never reaches either target; current source/results remain published.
    assert!(session
        .prepare("std", "answer", "fn answer() int { return 99 }", B)
        .is_err());
    assert_eq!(av.lowered, 1);
    assert_eq!((rust.stats().lower, rust.stats().print), (1, 1));
    assert_eq!(av.run().unwrap(), new_expected);
    drop(session);
    drop(av);
    drop(rust);
    drop(store);
    let store = ModuleStore::open(&store_path, "module-test").unwrap();
    let recovery = store
        .restore(&next, &changed, "app", &caps, "pkg", "app", B)
        .unwrap();
    assert_eq!(recovery.session.counters().solver_runs, 0);
    let av =
        ModuleAvTarget::restore(&store_path, &recovery.targets["av"], &recovery.bound).unwrap();
    let rust = ModuleA2xTarget::restore(
        &store_path,
        "module-test",
        "pkg",
        QUOTA,
        RustTargetProvider::v1(),
        &recovery.targets["a2x-rust"],
        &recovery.bound,
    )
    .unwrap();
    assert_eq!(av.lowered, 0);
    assert_eq!((rust.stats().lower, rust.stats().print), (0, 0));
    assert_eq!(av.run().unwrap(), new_expected);
    rust_value(rust.current(), &temp.0.join("next-recovery"), new_expected);
}
#[test]
fn modules_950_qualified_consumer_restore_first_edit_and_next_read() {
    run_case(false)
}
#[test]
fn modules_950_user_shadow_consumer_restore_first_edit_and_next_read() {
    run_case(true)
}
