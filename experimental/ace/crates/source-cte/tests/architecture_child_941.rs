//! The real checked-child version axis; no second instance/cache producer.
use ace_av_core::{CancelFlag, ExecLimits};
use ace_source_cte::envelope::EnvelopeError;
use ace_source_cte::{
    compile_scalar_ct, run_entry_on_av, CtEnvelopeSink, CtExecutedV1, CtProduct, CtService,
    ExecKey, MemoryEnvelopeSink,
};
use std::{cell::RefCell, io::Write, rc::Rc};

const TRUE: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/pick40.at");
const FALSE: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/pick42.at"
);

struct RecordingSink {
    inner: MemoryEnvelopeSink,
    published: Rc<RefCell<Vec<ExecKey>>>,
}
impl CtEnvelopeSink for RecordingSink {
    fn probe(&self, key: &ExecKey) -> Result<Option<CtExecutedV1>, EnvelopeError> {
        self.inner.probe(key)
    }
    fn publish(&mut self, key: &ExecKey, env: &CtExecutedV1) -> Result<(), EnvelopeError> {
        self.inner.publish(key, env)?;
        self.published.borrow_mut().push(key.clone());
        Ok(())
    }
}
fn compile(svc: &mut CtService<RecordingSink>, source: &str) -> CtProduct {
    compile_scalar_ct(
        svc,
        source,
        "main.at",
        &[
            "ac.source.cte.scalar/v1".into(),
            "ac.source.instances.scalar/v1".into(),
        ],
        "plan941",
        "child-version",
    )
    .expect("original ordinary-source child chain compiles")
}
fn value(product: &CtProduct) -> i32 {
    let (value, facts) = run_entry_on_av(
        &product.checked,
        "d_main",
        941,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .unwrap();
    assert!(facts.steps > 0);
    match value {
        ace_av_core::contract::Value::I32(n) => n,
        other => panic!("{other:?}"),
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn instance(product: &CtProduct) -> String {
    assert_eq!(product.instances.len(), 1);
    let i = &product.instances[0];
    format!("{{\"logical\":\"{}\",\"key\":\"{}\",\"template_hex\":\"{}\",\"concrete_hex\":\"{}\",\"signature\":\"{}\",\"body\":\"{}\",\"demands\":{}}}",
        i.logical, i.key, hex(i.template.as_bytes()), hex(i.concrete_name.as_bytes()), i.sig_version, i.body_version, i.demands)
}
fn snapshot(svc: &CtService<RecordingSink>) -> String {
    let c = svc.counters();
    let log = svc.exec_log().iter().map(|e| format!(
        "{{\"key_hash\":\"{}\",\"attempt\":{},\"outcome_hex\":\"{}\",\"steps\":{},\"wall_ms\":{},\"from_cache\":{}}}",
        e.key_hash,e.attempt,hex(e.outcome.as_bytes()),e.steps,e.wall_ms,e.from_cache)).collect::<Vec<_>>().join(",");
    format!("{{\"ct_execute\":{},\"ct_hits\":{},\"ct_freeze_publishes\":{},\"ct_rejections\":{},\"exec_log\":[{}]}}",
        c.ct_execute,c.ct_hits,c.ct_freeze_publishes,c.ct_rejections,log)
}

#[test]
fn ordinary_source_child_effective_versions_941() {
    // Only flag's checked body changes. The instance declaration and its owner
    // call site are exactly the same bytes, fixed before counters are read.
    assert_eq!(TRUE.replace("return true", "return false"), FALSE);
    let published = Rc::new(RefCell::new(Vec::new()));
    let mut svc = CtService::new(RecordingSink {
        inner: MemoryEnvelopeSink::new(),
        published: published.clone(),
    });
    let before = compile(&mut svc, TRUE);
    let first = value(&before);
    assert_eq!(first, 40);
    assert_eq!(svc.counters().ct_execute, 2);
    assert_eq!(svc.counters().ct_freeze_publishes, 2);
    let first_instance = instance(&before);
    let cold = snapshot(&svc);
    let after = compile(&mut svc, FALSE);
    let second = value(&after);
    assert_eq!(second, 42);
    assert_eq!(svc.counters().ct_execute, 4);
    assert_eq!(svc.counters().ct_hits, 0);
    assert_eq!(svc.counters().ct_freeze_publishes, 4);
    // Template identity is stable; the effective checked execution is not.
    assert_eq!(before.instances, after.instances);
    let second_instance = instance(&after);
    let edited = snapshot(&svc);
    svc.set_forbid_av(true);
    let hit = compile(&mut svc, FALSE);
    let third = value(&hit);
    assert_eq!(third, 42);
    assert_eq!(svc.counters().ct_execute, 4);
    assert_eq!(svc.counters().ct_hits, 2);
    assert_eq!(svc.counters().ct_freeze_publishes, 4);
    let cached = snapshot(&svc);
    let keys = published.borrow();
    assert_eq!(keys.len(), 4);
    assert_eq!(
        (&keys[0].callee, &keys[1].callee),
        (&"flag".to_string(), &"pick_i32".to_string())
    );
    assert_eq!(keys[0].callee_logical, keys[2].callee_logical);
    assert_eq!(keys[1].callee_logical, keys[3].callee_logical);
    assert_eq!(keys[0].callee_sig_version, keys[2].callee_sig_version);
    assert_eq!(keys[1].callee_sig_version, keys[3].callee_sig_version);
    assert_ne!(keys[0].callee_body_version, keys[2].callee_body_version);
    assert_ne!(keys[1].callee_body_version, keys[3].callee_body_version);
    for (index, key) in keys.iter().enumerate() {
        assert_eq!(svc.exec_log()[index].key_hash, key.hash());
        assert!(!svc.exec_log()[index].from_cache);
        assert!(svc.exec_log()[index].steps > 0);
    }
    if let Some(path) = std::env::var_os("AUTO_PLAN_941_CT_CHILD_OBSERVATIONS") {
        let path = std::path::PathBuf::from(path);
        let scope = (std::env::var_os("AUTO_PLAN_941_SCOPE_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new("D:/autostack/plan-artifacts/plan-941").to_path_buf()))
            .canonicalize()
            .unwrap();
        assert!(path
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(scope));
        let keys = keys.iter().map(|k| format!(
            "{{\"key_hash\":\"{}\",\"callee_hex\":\"{}\",\"logical\":\"{}\",\"signature\":\"{}\",\"body\":\"{}\",\"args_hex\":\"{}\"}}",
            k.hash(),hex(k.callee.as_bytes()),k.callee_logical,k.callee_sig_version,k.callee_body_version,hex(k.args_canonical.as_bytes())
        )).collect::<Vec<_>>().join(",");
        let payload = format!("{{\"schema\":\"ace.m1.ct-child-versions/1\",\"source_true_hex\":\"{}\",\"source_false_hex\":\"{}\",\"owner_values\":[{},{},{}],\"instance_before\":{},\"instance_after\":{},\"cold\":{},\"edited\":{},\"hit\":{},\"published_keys\":[{}]}}\n",
            hex(TRUE.as_bytes()),hex(FALSE.as_bytes()),first,second,third,first_instance,second_instance,cold,edited,cached,keys);
        assert!(payload.len() <= 16 * 1024);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();
        file.write_all(payload.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
}
