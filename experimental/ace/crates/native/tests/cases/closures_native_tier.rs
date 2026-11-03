//! PLAN-924 closure tiers: the corpus closure sources through the REAL
//! native closures pipeline (source → shared closures MIR units → native
//! admit/CLIF/COFF → PE link/run), the limited-domain named rejections
//! (V-11), the dual-target indirect-dispatch control (V-03) and the S9
//! env-node fault sweep with N+1 positive controls (V-04 core).

#[path = "../common/mod.rs"]
mod common;

use ace_native::collections_lower::WrapperKnobs;

fn corpus(name: &str) -> String {
    let path = common::repo_root()
        .join("crates/auto-lang/test/cases/ace-m1/src")
        .join(format!("{name}.at"));
    std::fs::read_to_string(&path).expect("corpus source")
}

fn closures_run(source: &str, tag: &str, knobs: WrapperKnobs) -> ace_native::driver::MnRequest {
    assert!(
        common::tools_available(),
        "native tools are required; no skip pass"
    );
    let dir = std::env::temp_dir().join(format!("plan924-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    driver.request_closures_resource(source, "main", &dir, None, knobs)
}

fn assert_exit(res: &ace_native::driver::MnRequest, expected: i32) {
    assert!(
        res.ok,
        "request failed: {:?}\n{}",
        res.error,
        String::from_utf8_lossy(&res.stderr)
    );
    assert_eq!(
        res.exit_code,
        Some(expected),
        "stderr:\n{}",
        String::from_utf8_lossy(&res.stderr)
    );
}

fn trapped_code(res: &ace_native::driver::MnRequest) -> Option<(i64, bool)> {
    if res.exit_code != Some(70) {
        return None;
    }
    let text = String::from_utf8_lossy(&res.stderr);
    let code = text
        .lines()
        .find_map(|line| line.strip_prefix("MN-COLL-TRAP code=")?
            .parse::<i64>()
            .ok());
    let inject = text
        .lines()
        .any(|line| line.starts_with("MN-COLL-INJECT seam=9"));
    code.map(|c| (c, inject))
}

fn assert_clean_after_trap(res: &ace_native::driver::MnRequest) {
    let text = String::from_utf8_lossy(&res.stderr);
    for field in [
        "MN-COLL live=0",
        "MN-COLL bytes=0",
        "MN-COLL invalid=0",
        "MN-COLL arena_live=0",
        "MN-COLL arena_invalid=0",
    ] {
        assert!(
            text.contains(&format!("{field}\n")),
            "missing {field}: {text}"
        );
    }
}

// ---- V-01: the corpus closure programs ----------------------------------

#[test]
fn k02_creator_exit_native_indirect_call_returns_42() {
    let res = closures_run(&corpus("K02"), "k02", Default::default());
    assert_exit(&res, 42);
}

#[test]
fn k01_copy_capture_native_returns_42() {
    let res = closures_run(&corpus("K01"), "k01", Default::default());
    assert_exit(&res, 42);
}

#[test]
fn k05_deep_clone_closure_both_callable_returns_42() {
    let res = closures_run(&corpus("K05"), "k05", Default::default());
    assert_exit(&res, 42);
}

// ---- V-03: two same-signature targets with different bodies/envs --------

#[test]
fn dual_target_same_signature_dispatches_independently() {
    let source = r#"
fn makeAdd(move s Str) fn(int) int {
    return move (n int) => { return s.len() + n }
}
fn makeMul(move t Str) fn(int) int {
    return move (n int) => { return t.len() * n }
}
fn main() int {
    let a Str = "hello"
    let b Str = "hi!"
    let f = makeAdd(a.move)
    let g = makeMul(b.move)
    return f(37) + g(4)
}
"#;
    // 5 + 37 = 42 and 3 * 4 = 12 → 54: two REAL indirect dispatches to two
    // different bodies over two different envs; folding either factory (or
    // dispatching both to one target) cannot produce this.
    let res = closures_run(source, "dual", Default::default());
    assert_exit(&res, 54);
}

// ---- V-11: the native limited domain is a NAMED rejection ----------------
// (the shared/AV contracts keep their own wider support — these corpus
// programs stay green in the AV closures suites.)

#[test]
fn k03_non_escaping_scalar_capture_returns_42() {
    // K03's lambda captures only an i32 local: the lowering stages it as a
    // by-value capture (no Borrow kind ever forms for non-escaping Copy
    // scalars), so the native domain covers it — a positive case.
    let res = closures_run(&corpus("K03"), "k03", Default::default());
    assert_exit(&res, 42);
}

#[test]
fn k06_shared_cell_capture_is_a_named_native_rejection() {
    // K06's shared mutable capture lowers to a MOVE of the source-front's
    // single-element List cell — a container capture, outside the native
    // closure domain: a NAMED type-face rejection (the AV suites keep it
    // green; the shared-cell/K06 domain belongs to a future scope).
    let res = closures_run(&corpus("K06"), "k06", Default::default());
    let error = res.error.expect("the shared-cell capture must not run natively");
    assert_eq!(error.code(), "mn.unsupported.type");
    assert!(
        error.message().contains("List"),
        "the rejection names the container: {}",
        error.message()
    );
}

// ---- V-04 core: the S9 env-node fault sweep with N+1 controls ------------

#[test]
fn k02_env_node_seam_sweeps_with_positive_control() {
    // K02 stages exactly ONE env node (S9); nth=1 traps INJECTED, nth=2 is
    // the N+1 positive control returning the true 42.
    let res = closures_run(
        &corpus("K02"),
        "k02s9a",
        WrapperKnobs {
            fail_seam: Some((9, 1)),
            ..Default::default()
        },
    );
    assert_eq!(trapped_code(&res), Some((25, true)), "stderr:\n{}", String::from_utf8_lossy(&res.stderr));
    assert_clean_after_trap(&res);

    let res = closures_run(
        &corpus("K02"),
        "k02s9b",
        WrapperKnobs {
            fail_seam: Some((9, 2)),
            ..Default::default()
        },
    );
    assert_exit(&res, 42);
}

#[test]
fn k05_env_clone_seam_sweeps_two_allocations() {
    // K05 stages the original env AND the deep-clone env (two S9 blocks);
    // both injected points trap INJECTED with a clean runtime and the N+1
    // control returns 42.
    for nth in [1u64, 2] {
        let res = closures_run(
            &corpus("K05"),
            &format!("k05s9{nth}"),
            WrapperKnobs {
                fail_seam: Some((9, nth)),
                ..Default::default()
            },
        );
        assert_eq!(
            trapped_code(&res),
            Some((25, true)),
            "nth={nth} stderr:\n{}",
            String::from_utf8_lossy(&res.stderr)
        );
        assert_clean_after_trap(&res);
    }
    let res = closures_run(
        &corpus("K05"),
        "k05s9n",
        WrapperKnobs {
            fail_seam: Some((9, 3)),
            ..Default::default()
        },
    );
    assert_exit(&res, 42);
}
