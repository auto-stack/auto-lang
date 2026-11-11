//! Observe the existing ordinary-source CT service, without a second executor.
//! Optional evidence export is for PLAN-941's strict parent reader. Assertions
//! run normally even when export is disabled; no library/protocol is changed.
use ace_av_core::{CancelFlag, ExecLimits};
use ace_source_cte::{
    compile_scalar_ct, run_entry_on_av, CtProduct, CtProductError, CtService, MemoryEnvelopeSink,
    PlanError,
};
use std::io::Write;

const SOURCE42: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/sum42.at");
const SOURCE43: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/sum43.at");

fn compile(
    svc: &mut CtService<MemoryEnvelopeSink>,
    src: &str,
) -> Result<CtProduct, CtProductError> {
    compile_scalar_ct(
        svc,
        src,
        "main.at",
        &["ac.source.cte.scalar/v1".to_string()],
        "plan941",
        "ct-architecture",
    )
}

fn owner(product: &CtProduct) -> i32 {
    let (value, facts) = run_entry_on_av(
        &product.checked,
        "d_main",
        941,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .expect("served checked owner executes on real AV");
    assert!(facts.steps > 0);
    match value {
        ace_av_core::contract::Value::I32(value) => value,
        other => panic!("unexpected owner value {other:?}"),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn snapshot(svc: &CtService<MemoryEnvelopeSink>) -> String {
    let c = svc.counters();
    let log = svc
        .exec_log()
        .iter()
        .map(|e| {
            // Every string is hex-encoded or a boolean; the exporter does not
            // hand-escape arbitrary source/message strings into JSON.
            format!(
                "{{\"key_hash\":\"{}\",\"attempt\":{},\"outcome_hex\":\"{}\",\"steps\":{},\"wall_ms\":{},\"from_cache\":{}}}",
                e.key_hash, e.attempt, hex(e.outcome.as_bytes()), e.steps, e.wall_ms, e.from_cache
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"ct_execute\":{},\"ct_hits\":{},\"ct_freeze_publishes\":{},\"ct_rejections\":{},\"exec_log\":[{}]}}",
        c.ct_execute, c.ct_hits, c.ct_freeze_publishes, c.ct_rejections, log
    )
}

fn forbidden(error: CtProductError) -> String {
    match error {
        CtProductError::Plan(PlanError::Demand { err, .. }) => {
            assert_eq!(err.code, "cte.infra/av-forbidden");
            assert!(!err.av_reached);
            hex(err.code.as_bytes())
        }
        other => panic!("forbidden miss did not reach the typed control: {other:?}"),
    }
}

#[test]
fn ordinary_source_cold_hit_and_callee_edit_observations_941() {
    // These are the original frozen sources. Their call-site suffix is byte
    // identical; the edit is in the callee alone, before any measurement.
    assert_eq!(
        SOURCE42.split_once("fn main").unwrap().1,
        SOURCE43.split_once("fn main").unwrap().1
    );
    let mut denied = CtService::new(MemoryEnvelopeSink::new());
    denied.set_forbid_av(true);
    let cold_error = forbidden(compile(&mut denied, SOURCE42).unwrap_err());
    assert_eq!(denied.counters().ct_execute, 0);
    assert_eq!(denied.counters().ct_freeze_publishes, 0);
    assert_eq!(denied.counters().ct_rejections, 1);
    assert!(denied.exec_log().is_empty());
    let cold_control = snapshot(&denied);

    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let cold = compile(&mut svc, SOURCE42).expect("cold original source");
    let cold_value = owner(&cold);
    assert_eq!(cold_value, 42);
    assert_eq!(cold.served.values.len(), 1);
    assert_eq!(svc.counters().ct_execute, 1);
    assert_eq!(svc.counters().ct_hits, 0);
    assert_eq!(svc.counters().ct_freeze_publishes, 1);
    assert_eq!(svc.exec_log().len(), 1);
    assert!(!svc.exec_log()[0].from_cache);
    assert!(svc.exec_log()[0].steps > 0);
    let cold_snapshot = snapshot(&svc);

    svc.set_forbid_av(true);
    let hit = compile(&mut svc, SOURCE42).expect("forbidden AV still permits genuine hit");
    let hit_value = owner(&hit);
    assert_eq!(hit_value, 42);
    assert_eq!(svc.counters().ct_execute, 1);
    assert_eq!(svc.counters().ct_hits, 1);
    assert_eq!(svc.counters().ct_freeze_publishes, 1);
    assert_eq!(svc.exec_log().len(), 2);
    assert!(svc.exec_log()[1].from_cache);
    assert_eq!(svc.exec_log()[0].key_hash, svc.exec_log()[1].key_hash);
    let hit_snapshot = snapshot(&svc);

    let edit_error = forbidden(compile(&mut svc, SOURCE43).unwrap_err());
    assert_eq!(svc.counters().ct_execute, 1);
    assert_eq!(svc.counters().ct_freeze_publishes, 1);
    assert_eq!(svc.counters().ct_rejections, 1);
    assert_eq!(svc.exec_log().len(), 2);
    let edit_control = snapshot(&svc);

    svc.set_forbid_av(false);
    let edited = compile(&mut svc, SOURCE43).expect("callee edit requires new real execution");
    let edit_value = owner(&edited);
    assert_eq!(edit_value, 43);
    assert_eq!(edited.served.values.len(), 1);
    assert_eq!(svc.counters().ct_execute, 2);
    assert_eq!(svc.counters().ct_hits, 1);
    assert_eq!(svc.counters().ct_freeze_publishes, 2);
    assert_eq!(svc.exec_log().len(), 3);
    assert!(!svc.exec_log()[2].from_cache);
    assert!(svc.exec_log()[2].steps > 0);
    assert_ne!(svc.exec_log()[0].key_hash, svc.exec_log()[2].key_hash);
    let edited_snapshot = snapshot(&svc);

    if let Some(path) = std::env::var_os("AUTO_PLAN_941_CT_OBSERVATIONS") {
        let path = std::path::PathBuf::from(path);
        let scope = (std::env::var_os("AUTO_PLAN_941_SCOPE_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new("D:/autostack/plan-artifacts/plan-941").to_path_buf()))
            .canonicalize()
            .unwrap();
        let parent = path.parent().unwrap().canonicalize().unwrap();
        assert!(
            parent.starts_with(scope),
            "PLAN-941 export must stay in its registered scope"
        );
        let payload = format!(
            "{{\"schema\":\"ace.m1.ct-source-mechanisms/1\",\"sink\":\"fresh-memory-envelope\",\"source42_hex\":\"{}\",\"source43_hex\":\"{}\",\"cold_control_code_hex\":\"{}\",\"edit_control_code_hex\":\"{}\",\"cold_control\":{},\"cold\":{},\"hit\":{},\"edit_control\":{},\"edited\":{},\"owner_values\":[{},{},{}]}}\n",
            hex(SOURCE42.as_bytes()), hex(SOURCE43.as_bytes()), cold_error, edit_error,
            cold_control, cold_snapshot, hit_snapshot, edit_control, edited_snapshot,
            cold_value, hit_value, edit_value
        );
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
