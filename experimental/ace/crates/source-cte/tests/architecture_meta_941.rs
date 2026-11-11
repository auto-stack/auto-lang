//! Real meta arguments/absence dependencies, with no new executor or store.
use ace_av_core::{CancelFlag, ExecLimits};
use ace_core::atom_text::Stage;
use ace_source_cte::envelope::EnvelopeError;
use ace_source_cte::{
    compile_scalar_ct, run_entry_on_av, CtEnvelopeSink, CtExecutedV1, CtProduct, CtProductError,
    CtService, ExecKey, MemoryEnvelopeSink, PlanError,
};
use std::{cell::RefCell, io::Write, rc::Rc};

const FIELD2: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta2.at");
const FIELD3: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta3.at");
const ABSENT: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/has-absent.at"
);
const PRESENT: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/has-present.at"
);
const LOCAL: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/meta-only.at"
);

#[derive(Default)]
struct Effects {
    probes: usize,
    keys: Vec<ExecKey>,
}
struct Sink {
    inner: MemoryEnvelopeSink,
    effects: Rc<RefCell<Effects>>,
}
impl CtEnvelopeSink for Sink {
    fn probe(&self, key: &ExecKey) -> Result<Option<CtExecutedV1>, EnvelopeError> {
        self.effects.borrow_mut().probes += 1;
        self.inner.probe(key)
    }
    fn publish(&mut self, key: &ExecKey, page: &CtExecutedV1) -> Result<(), EnvelopeError> {
        self.inner.publish(key, page)?;
        self.effects.borrow_mut().keys.push(key.clone());
        Ok(())
    }
}
fn service() -> (CtService<Sink>, Rc<RefCell<Effects>>) {
    let effects = Rc::new(RefCell::new(Effects::default()));
    (
        CtService::new(Sink {
            inner: MemoryEnvelopeSink::new(),
            effects: effects.clone(),
        }),
        effects,
    )
}
fn compile(svc: &mut CtService<Sink>, src: &str, meta: bool) -> Result<CtProduct, CtProductError> {
    let mut caps = vec!["ac.source.cte.scalar/v1".into()];
    if meta {
        caps.push("cte.meta.fields/v1".into());
    }
    compile_scalar_ct(svc, src, "main.at", &caps, "plan941", "meta-architecture")
}
fn value(p: &CtProduct) -> i32 {
    let (value, facts) = run_entry_on_av(
        &p.checked,
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
fn snapshot(svc: &CtService<Sink>, effects: &Rc<RefCell<Effects>>) -> String {
    let c = svc.counters();
    let log = svc.exec_log().iter().map(|e| format!(
        "{{\"key_hash\":\"{}\",\"attempt\":{},\"outcome_hex\":\"{}\",\"steps\":{},\"wall_ms\":{},\"from_cache\":{}}}",
        e.key_hash,e.attempt,hex(e.outcome.as_bytes()),e.steps,e.wall_ms,e.from_cache)).collect::<Vec<_>>().join(",");
    format!("{{\"ct_execute\":{},\"ct_hits\":{},\"ct_freeze_publishes\":{},\"ct_rejections\":{},\"probes\":{},\"exec_log\":[{}]}}",
        c.ct_execute,c.ct_hits,c.ct_freeze_publishes,c.ct_rejections,effects.borrow().probes,log)
}
fn keys(effects: &Rc<RefCell<Effects>>) -> String {
    effects.borrow().keys.iter().map(|k| format!(
        "{{\"key_hash\":\"{}\",\"callee_hex\":\"{}\",\"logical\":\"{}\",\"signature\":\"{}\",\"body\":\"{}\",\"args_hex\":\"{}\"}}",
        k.hash(),hex(k.callee.as_bytes()),k.callee_logical,k.callee_sig_version,k.callee_body_version,hex(k.args_canonical.as_bytes()))).collect::<Vec<_>>().join(",")
}
fn axis(before: &str, after: &str, expected: [i32; 3], args: [&str; 2]) -> String {
    let (mut svc, effects) = service();
    let cold_product = compile(&mut svc, before, true).unwrap();
    let cold_value = value(&cold_product);
    assert_eq!(cold_product.served.meta_queries, 1);
    let cold = snapshot(&svc, &effects);
    svc.set_forbid_av(true);
    let hit_product = compile(&mut svc, before, true).unwrap();
    let hit_value = value(&hit_product);
    assert_eq!(hit_product.served.meta_queries, 1);
    let hit = snapshot(&svc, &effects);
    let control_code = match compile(&mut svc, after, true).unwrap_err() {
        CtProductError::Plan(PlanError::Demand { err, .. }) => {
            assert!(!err.av_reached);
            assert_eq!(err.code, "cte.infra/av-forbidden");
            err.code
        }
        other => panic!("{other:?}"),
    };
    let control = snapshot(&svc, &effects);
    svc.set_forbid_av(false);
    let edited_product = compile(&mut svc, after, true).unwrap();
    let edited_value = value(&edited_product);
    assert_eq!(edited_product.served.meta_queries, 1);
    assert_eq!([cold_value, hit_value, edited_value], expected);
    assert_eq!(svc.counters().ct_execute, 2);
    assert_eq!(svc.counters().ct_hits, 1);
    assert_eq!(svc.counters().ct_freeze_publishes, 2);
    assert_eq!(svc.counters().ct_rejections, 1);
    assert_eq!(effects.borrow().keys.len(), 2);
    let k = effects.borrow();
    assert_eq!(k.keys[0].callee_logical, k.keys[1].callee_logical);
    assert_eq!(k.keys[0].callee_sig_version, k.keys[1].callee_sig_version);
    assert_eq!(k.keys[0].callee_body_version, k.keys[1].callee_body_version);
    assert_ne!(k.keys[0].hash(), k.keys[1].hash());
    assert_eq!(k.keys[0].args_canonical, args[0]);
    assert_eq!(k.keys[1].args_canonical, args[1]);
    drop(k);
    format!("{{\"values\":[{cold_value},{hit_value},{edited_value}],\"meta_queries\":[{},{},{}],\"control_code_hex\":\"{}\",\"cold\":{cold},\"hit\":{hit},\"control\":{control},\"edited\":{},\"published_keys\":[{}]}}",
        cold_product.served.meta_queries,hit_product.served.meta_queries,edited_product.served.meta_queries,hex(control_code.as_bytes()),snapshot(&svc,&effects),keys(&effects))
}
fn closed(src: &str) -> String {
    let (mut svc, effects) = service();
    let error = compile(&mut svc, src, false).unwrap_err();
    let CtProductError::Diagnostics(ds) = error else {
        panic!("{error:?}")
    };
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].stage, Stage::Bind);
    assert_eq!(ds[0].code, "source.cte.bind");
    assert_eq!(svc.counters(), &Default::default());
    assert_eq!(effects.borrow().probes, 0);
    assert!(effects.borrow().keys.is_empty());
    assert!(svc.exec_log().is_empty());
    format!(
        "{{\"stage\":\"{}\",\"code_hex\":\"{}\",\"state\":{}}}",
        ds[0].stage.as_str(),
        hex(ds[0].code.as_bytes()),
        snapshot(&svc, &effects)
    )
}

#[test]
fn ordinary_source_meta_arguments_and_absence_941() {
    assert_eq!(
        ABSENT.replace("    y int\n", "    y int\n    z int\n"),
        PRESENT
    );
    let field = axis(
        FIELD2,
        FIELD3,
        [42, 42, 43],
        ["i32:2,i32:40", "i32:3,i32:40"],
    );
    let has = axis(ABSENT, PRESENT, [42, 42, 40], ["bool:false", "bool:true"]);
    let field_closed = closed(FIELD2);
    let has_closed = closed(ABSENT);
    let (mut svc, effects) = service();
    let local = compile(&mut svc, LOCAL, true).unwrap();
    let local_value = value(&local);
    assert_eq!(local_value, 2);
    assert_eq!(local.served.meta_queries, 1);
    assert_eq!(svc.counters(), &Default::default());
    assert_eq!(effects.borrow().probes, 0);
    assert!(effects.borrow().keys.is_empty());
    if let Some(path) = std::env::var_os("AUTO_PLAN_941_CT_META_OBSERVATIONS") {
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
        let payload=format!("{{\"schema\":\"ace.m1.ct-meta-arguments/1\",\"sources_hex\":[\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"],\"field\":{field},\"has\":{has},\"field_closed\":{field_closed},\"has_closed\":{has_closed},\"local\":{{\"value\":{local_value},\"meta_queries\":{},\"state\":{}}}}}\n",
            hex(FIELD2.as_bytes()),hex(FIELD3.as_bytes()),hex(ABSENT.as_bytes()),hex(PRESENT.as_bytes()),hex(LOCAL.as_bytes()),local.served.meta_queries,snapshot(&svc,&effects));
        assert!(payload.len() <= 16 << 10);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();
        file.write_all(payload.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
}
