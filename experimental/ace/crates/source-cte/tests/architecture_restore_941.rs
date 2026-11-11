//! Actual S936 disk restore observations. These do not claim the other owners'
//! source/facts/instance/foreign restore obligations are satisfied.
use ace_av_core::{CancelFlag, ExecLimits};
use ace_source_cte::store::ManagedEnvelopeSink;
use ace_source_cte::{
    compile_scalar_ct, run_entry_on_av, CtProduct, CtProductError, CtService, PlanError,
};
use std::{io::Write, path::Path};

const GOOD: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/sum42.at");
const ARGS: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/restore-args.at"
);
const BAD: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/restore-trap.at"
);
const REPAIR: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/restore-repair.at"
);
const SECOND: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/restore-second.at"
);

fn service(path: &Path) -> CtService<ManagedEnvelopeSink> {
    CtService::new(ManagedEnvelopeSink::open(path, Some("plan941-ct-restore")).unwrap())
}
fn compile(
    svc: &mut CtService<ManagedEnvelopeSink>,
    src: &str,
) -> Result<CtProduct, CtProductError> {
    compile_scalar_ct(
        svc,
        src,
        "main.at",
        &["ac.source.cte.scalar/v1".into()],
        "plan941",
        "restore",
    )
}
fn value(p: &CtProduct) -> i32 {
    let (v, facts) = run_entry_on_av(
        &p.checked,
        "d_main",
        941,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .unwrap();
    assert!(facts.steps > 0);
    match v {
        ace_av_core::contract::Value::I32(v) => v,
        other => panic!("{other:?}"),
    }
}
fn observe(values: &mut Vec<i32>, product: &CtProduct) -> i32 {
    let v = value(product);
    values.push(v);
    v
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn snapshot(svc: &CtService<ManagedEnvelopeSink>) -> String {
    let c = svc.counters();
    let (probes, bytes, decodes) = svc.r_window();
    let log = svc.exec_log().iter().map(|e| format!(
        "{{\"key_hash\":\"{}\",\"attempt\":{},\"outcome_hex\":\"{}\",\"steps\":{},\"wall_ms\":{},\"from_cache\":{}}}",
        e.key_hash, e.attempt, hex(e.outcome.as_bytes()), e.steps, e.wall_ms, e.from_cache
    )).collect::<Vec<_>>().join(",");
    format!("{{\"ct_execute\":{},\"ct_hits\":{},\"ct_freeze_publishes\":{},\"ct_rejections\":{},\"read_probes\":{probes},\"read_bytes\":{bytes},\"read_decodes\":{decodes},\"exec_log\":[{log}]}}",
        c.ct_execute, c.ct_hits, c.ct_freeze_publishes, c.ct_rejections)
}
fn counts(svc: &CtService<ManagedEnvelopeSink>, expected: [u64; 4]) {
    let c = svc.counters();
    assert_eq!(
        [
            c.ct_execute,
            c.ct_hits,
            c.ct_freeze_publishes,
            c.ct_rejections
        ],
        expected
    );
}
fn demand_error(error: CtProductError) -> (String, bool) {
    match error {
        CtProductError::Plan(PlanError::Demand { err, .. }) => (err.code, err.av_reached),
        other => panic!("{other:?}"),
    }
}

#[test]
fn ordinary_source_disk_restore_first_edits_and_repair_941() {
    let mut owner_values = Vec::new();
    let scope = (std::env::var_os("AUTO_PLAN_941_SCOPE_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new("D:/autostack/plan-artifacts/plan-941").to_path_buf()))
        .canonicalize()
        .unwrap();
    let root = scope.join(format!(
        "ct-restore-store-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let disk = root.join("good");
    let mut svc = service(&disk);
    let old = compile(&mut svc, GOOD).unwrap();
    assert_eq!(observe(&mut owner_values, &old), 42);
    counts(&svc, [1, 0, 1, 0]);
    let cold = snapshot(&svc);
    let good_key = svc.exec_log()[0].key_hash.clone();
    drop(svc);

    // The first demand after reopen is a changed source, without a warm hit.
    let mut svc = service(&disk);
    let edited = compile(&mut svc, ARGS).unwrap();
    assert_eq!(observe(&mut owner_values, &edited), 42);
    counts(&svc, [1, 0, 1, 0]);
    assert_ne!(svc.exec_log()[0].key_hash, good_key);
    assert!(!svc.exec_log()[0].from_cache);
    let first_edit = snapshot(&svc);
    drop(svc);

    // Both persisted versions must answer with AV explicitly forbidden.
    let mut svc = service(&disk);
    svc.set_forbid_av(true);
    assert_eq!(
        observe(&mut owner_values, &compile(&mut svc, GOOD).unwrap()),
        42
    );
    assert_eq!(
        observe(&mut owner_values, &compile(&mut svc, ARGS).unwrap()),
        42
    );
    counts(&svc, [0, 2, 0, 0]);
    let (p, b, d) = svc.r_window();
    assert_eq!((p, d), (2, 2));
    assert!(b > 0);
    assert!(svc.exec_log().iter().all(|e| e.from_cache));
    let restored = snapshot(&svc);
    drop(svc);

    let disk = root.join("repair");
    let mut svc = service(&disk);
    let old = compile(&mut svc, GOOD).unwrap();
    counts(&svc, [1, 0, 1, 0]);
    drop(svc);
    // First demand after restart traps. No replacement envelope is published.
    let mut svc = service(&disk);
    let (trap, reached) = demand_error(compile(&mut svc, BAD).unwrap_err());
    assert!(trap.starts_with("cte.trap/"));
    assert!(reached);
    counts(&svc, [1, 0, 0, 1]);
    assert_eq!(observe(&mut owner_values, &old), 42);
    let failed = snapshot(&svc);
    svc.set_forbid_av(true);
    assert_eq!(
        observe(&mut owner_values, &compile(&mut svc, GOOD).unwrap()),
        42
    );
    counts(&svc, [1, 1, 0, 1]);
    let preserved = snapshot(&svc);
    svc.set_forbid_av(false);
    assert_eq!(
        observe(&mut owner_values, &compile(&mut svc, REPAIR).unwrap()),
        43
    );
    counts(&svc, [2, 1, 1, 1]);
    let repaired = snapshot(&svc);
    drop(svc);
    let mut svc = service(&disk);
    // Again, the FIRST demand is the next edit, not a prewarm of the repair.
    assert_eq!(
        observe(&mut owner_values, &compile(&mut svc, SECOND).unwrap()),
        44
    );
    counts(&svc, [1, 0, 1, 0]);
    let second_edit = snapshot(&svc);
    drop(svc);
    let mut svc = service(&disk);
    svc.set_forbid_av(true);
    assert_eq!(
        observe(&mut owner_values, &compile(&mut svc, REPAIR).unwrap()),
        43
    );
    assert_eq!(
        observe(&mut owner_values, &compile(&mut svc, SECOND).unwrap()),
        44
    );
    counts(&svc, [0, 2, 0, 0]);
    let repair_restore = snapshot(&svc);
    drop(svc);

    // Missing pages are a real miss, never a nominal restore hit.
    let mut svc = service(&root.join("empty"));
    svc.set_forbid_av(true);
    let (missing_code, reached) = demand_error(compile(&mut svc, GOOD).unwrap_err());
    assert_eq!(missing_code, "cte.infra/av-forbidden");
    assert!(!reached);
    counts(&svc, [0, 0, 0, 1]);
    assert_eq!(svc.r_window(), (0, 0, 0));
    let missing = snapshot(&svc);
    svc.set_forbid_av(false);
    assert_eq!(
        observe(&mut owner_values, &compile(&mut svc, GOOD).unwrap()),
        42
    );
    counts(&svc, [1, 0, 1, 1]);
    let fallback = snapshot(&svc);
    drop(svc);
    if let Some(path) = std::env::var_os("AUTO_PLAN_941_CT_RESTORE_OBSERVATIONS") {
        let path = std::path::PathBuf::from(path);
        assert!(path
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(&scope));
        let sources = [GOOD, ARGS, BAD, REPAIR, SECOND]
            .iter()
            .map(|s| format!("\"{}\"", hex(s.as_bytes())))
            .collect::<Vec<_>>()
            .join(",");
        let payload = format!("{{\"schema\":\"ace.m1.ct-disk-restore/1\",\"sink\":\"ManagedEnvelopeSink/S936\",\"owner_values\":{owner_values:?},\"sources_hex\":[{sources}],\"store_root_hex\":\"{}\",\"trap_code_hex\":\"{}\",\"missing_code_hex\":\"{}\",\"cold\":{cold},\"first_edit\":{first_edit},\"restored\":{restored},\"failed\":{failed},\"preserved\":{preserved},\"repaired\":{repaired},\"second_edit\":{second_edit},\"repair_restore\":{repair_restore},\"missing\":{missing},\"fallback\":{fallback}}}\n", hex(root.to_str().unwrap().as_bytes()), hex(trap.as_bytes()), hex(missing_code.as_bytes()));
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
