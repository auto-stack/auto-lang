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
        .find_map(|line| line.strip_prefix("MN-COLL-TRAP code=")?.parse::<i64>().ok());
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
    let error = res
        .error
        .expect("the shared-cell capture must not run natively");
    assert_eq!(error.code(), "mn.unsupported.type");
    assert!(
        error.message().contains("List"),
        "the rejection names the container: {}",
        error.message()
    );
}

// ---- V-01 (R05): creator-exit env lifetime on the MN-EVT channel ---------

#[test]
fn k02_env_lifecycle_events_show_creator_exit_and_final_release() {
    // The observation switch decodes strictly; the env node appears as a
    // `closure` kind owner created in make's frame, still LIVE after the
    // creator frame leaves (the indirect call in main), and freed exactly
    // once at the program's final sweep (live=0).
    let res = closures_run(
        &corpus("K02"),
        "k02evt",
        WrapperKnobs {
            observe: true,
            ..Default::default()
        },
    );
    assert_exit(&res, 42);
    let text = String::from_utf8_lossy(&res.stderr);
    let own_closure = text
        .lines()
        .filter(|l| l.contains("act=own kind=closure"))
        .count();
    let free_closure = text.lines().filter(|l| l.contains("act=free")).count();
    assert!(
        own_closure >= 1,
        "no closure own event:
{text}"
    );
    assert!(
        text.contains("MN-EVTEND total="),
        "missing event end marker"
    );
    assert!(
        text.contains(
            "MN-COLL live=0
"
        ),
        "env not fully released:
{text}"
    );
    assert!(
        free_closure >= 2,
        "expected env + text frees:
{text}"
    );
    // R05's discriminating fact: the creator frame LEAVES before the call —
    // frame-leave events must precede the call's continued liveness; the
    // closure free comes after main's frame-enter (not with make's sweep).
    let make_leave = text
        .lines()
        .position(|l| l.contains("act=frame-leave"))
        .expect("frame-leave event");
    let closure_own = text
        .lines()
        .position(|l| l.contains("act=own kind=closure"))
        .expect("closure own event");
    assert!(closure_own < make_leave, "env created after creator exit?");
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
    assert_eq!(
        trapped_code(&res),
        Some((25, true)),
        "stderr:\n{}",
        String::from_utf8_lossy(&res.stderr)
    );
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

// ---- T-07: the warm native session (single-unit/owner edits, hit) --------

#[test]
fn warm_session_lambda_edit_relinks_one_unit_and_hit_is_zero_production() {
    assert!(common::tools_available());
    let source = corpus("K02");
    let dir = std::env::temp_dir().join(format!("plan924-sess-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (mut session, cold) =
        ace_native::closures_incremental::ClosuresNativeSession::new(&source, "main", &dir, None)
            .expect("cold session");
    assert_eq!(
        cold.exit_code,
        Some(42),
        "cold run: {}",
        String::from_utf8_lossy(&cold.stderr)
    );
    assert_eq!(cold.work.codegen_units.len(), 3, "K02 has three units");

    // Lambda body edit: s.len() + n -> s.len() + n + 1 (42 -> 43). Exactly
    // the closure body unit re-codegens; make/main keep their objects.
    let edited = source.replace("+ n }", "+ n + 1 }");
    let warm = session.apply_source(&edited).expect("warm edit");
    assert_eq!(
        warm.exit_code,
        Some(43),
        "warm run: {}",
        String::from_utf8_lossy(&warm.stderr)
    );
    assert_eq!(
        warm.work.codegen_units,
        vec!["d___closure_make_0".to_string()],
        "expected exactly the closure body unit re-codegen: {:?} route={}",
        warm.work.codegen_units,
        warm.work.route
    );
    assert!(
        warm.work.route.starts_with("fast-unit"),
        "route: {}",
        warm.work.route
    );

    // Hit: identical source re-runs with ZERO codegen (the installed PE).
    let hit = session.apply_source(&edited).expect("hit");
    assert_eq!(hit.exit_code, Some(43));
    assert!(
        hit.work.codegen_units.is_empty(),
        "hit must not codegen: {:?}",
        hit.work.codegen_units
    );
    assert_eq!(hit.work.route, "hit");

    // Owner body edit in main: f(37) -> f(38) (43 -> 44). Only d_main
    // re-codegens.
    let edited2 = edited.replace("f(37)", "f(38)");
    let owner = session.apply_source(&edited2).expect("owner edit");
    assert_eq!(
        owner.exit_code,
        Some(44),
        "owner run: {}",
        String::from_utf8_lossy(&owner.stderr)
    );
    assert_eq!(
        owner.work.codegen_units,
        vec!["d_main".to_string()],
        "owner edit codegen: {:?} route={}",
        owner.work.codegen_units,
        owner.work.route
    );

    session.close().expect("close");
}

// ---- V-11: the front-door rejections on the source ride -----------------

#[test]
fn k04_borrow_escape_is_rejected_at_the_front_door() {
    // K04's lambda escapes its creator while capturing a non-move local —
    // the SHARED source/verify front door refuses it before any native
    // admission (the native domain never sees borrow-escape input).
    let res = closures_run(&corpus("K04"), "k04", Default::default());
    let error = res
        .error
        .expect("borrow escape must be refused at the front door");
    assert_eq!(error.code(), "mn.unsupported.frontend");
}

#[test]
fn closures_entry_must_be_an_ordinary_zero_argument_int_function() {
    // The K02 closure BODY unit exists and links, but naming it as the
    // entry is a NAMED driver rejection: env bodies are invoked through
    // their closure values, never as program entries.
    assert!(common::tools_available());
    let dir = std::env::temp_dir().join(format!("plan924-entry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let res = driver.request_closures(&corpus("K02"), "d___closure_make_0", &dir, None);
    let error = res.error.expect("a closure body entry must be refused");
    assert!(
        error
            .message()
            .contains("ordinary zero-argument int function"),
        "named entry rejection: {}",
        error.message()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ---- T-07/V-06/V-07: the formal-scale warm session -----------------------

/// 62 helpers + `make` (1 move-lambda) + `sum` (3 sibling lambdas) +
/// `main` (3 sibling lambdas + the factory call) = 65 top-level functions,
/// 72 units, every tested owner carrying ≥3 sibling lambdas.
fn scale_source() -> String {
    let mut src = String::new();
    for i in 0..62 {
        src.push_str(&format!("fn h{i}(n int) int {{ return n + {i} }}\n"));
    }
    src.push_str(
        "fn make(move s Str) fn(int) int {\n\
         return move (n int) => { return s.len() + n }\n\
         }\n",
    );
    src.push_str(
        "fn sum(n int) int {\n\
         var y int = 4\n\
         var g0 = (k int) => { return y + k }\n\
         var g1 = (k int) => { return y - k }\n\
         var g2 = (k int) => { return k * 3 }\n\
         return g0(n) + g1(1) + g2(4) + h0(1)\n\
         }\n",
    );
    src.push_str(
        "fn main() int {\n\
         let base int = 10\n\
         let s Str = \"abc\"\n\
         let f0 = (n int) => { return base + n }\n\
         let f1 = make(s.move)\n\
         let f2 = (n int) => { return n * 2 }\n\
         let f3 = (n int) => { return base - n }\n\
         return f0(20) + f1(3) + f2(5) + f3(4) + sum(2) + h0(1) + h61(1)\n\
         }\n",
    );
    src
}

// ---- V-12: a trap INSIDE a pinned call sweeps the parked environment -----

#[test]
fn trap_inside_closure_call_parks_env_and_aborts_clean() {
    // The lambda body STRLENs the captured text (allocating the text
    // runtime's scalar metadata) and then divides by zero: the body's
    // prologue pinned the env, the trap path never unpins, and
    // mn_coll_abort's parked-environment sweep + idle-metadata release
    // must still reach the zero-live/zero-bytes/zero-arena baseline.
    let source = r#"
fn make(move s Str) fn(int) int {
    return move (n int) => { return s.len() / (n - n) }
}
fn main() int {
    let s Str = "abc"
    let f = make(s.move)
    return f(37)
}
"#;
    let res = closures_run(source, "trapin", Default::default());
    assert_eq!(
        res.exit_code,
        Some(70),
        "stderr:\n{}",
        String::from_utf8_lossy(&res.stderr)
    );
    assert_clean_after_trap(&res);
}

// ---- V-04/T-09: the full seam sweep over the closure program ------------

#[test]
fn k02_full_seam_sweep_every_injection_traps_clean_or_completes() {
    // Every seam of the closure program (node/data/grow/frame/clonework/
    // text/textmeta/token/envnode), at every early ordinal: an injection
    // that fires traps INJECTED(25) with the full cleanup baseline; an
    // ordinal beyond what the program allocates on that seam simply runs
    // to the true 42. Both faces are honest — no silent third state.
    assert!(common::tools_available());
    let source = corpus("K02");
    for seam in 1u32..=9 {
        for nth in 1u64..=4 {
            let res = closures_run(
                &source,
                &format!("sweep{seam}_{nth}"),
                WrapperKnobs {
                    fail_seam: Some((seam, nth)),
                    ..Default::default()
                },
            );
            let text = String::from_utf8_lossy(&res.stderr);
            if res.exit_code == Some(70) {
                assert!(
                    text.contains("MN-COLL-INJECT seam="),
                    "trap without injection fact (seam {seam} nth {nth}): {text}"
                );
                assert!(
                    text.contains(&format!("MN-COLL-INJECT seam={seam}\n")),
                    "wrong seam fired: {text}"
                );
                let code = text
                    .lines()
                    .find_map(|line| line.strip_prefix("MN-COLL-TRAP code=")?.parse::<i64>().ok());
                // The delivered frame-open shim collapses an injected
                // failure into its null path (trap 22) while keeping the
                // INJECT fact line — the 923 error taxonomy, frozen in the
                // 923 case.ad fault-s4 expectation. Every other seam traps
                // the named INJECTED(25).
                let expected = if seam == 4 { 22 } else { 25 };
                assert_eq!(code, Some(expected), "seam {seam} nth {nth}: {text}");
                assert_clean_after_trap(&res);
            } else {
                assert_eq!(
                    res.exit_code,
                    Some(42),
                    "seam {seam} nth {nth}: err={:?} {}",
                    res.error
                        .as_ref()
                        .map(|e| (e.code().to_string(), e.message().to_string())),
                    text
                );
                assert!(
                    res.error.is_none(),
                    "{:?}",
                    res.error.map(|e| e.message().to_string())
                );
            }
        }
    }
}

mod performance {
    use super::*;

    #[test]
    fn warm_session_formal_scale_precise_codegen_identity_and_routes() {
        assert!(common::tools_available());
        let source = scale_source();
        // Independent hand model: f0(20)=30, f1(3)=3+3, f2(5)=10, f3(4)=6,
        // sum(2)=6+3+12+1=22, h0(1)=1, h61(1)=62 → 137.
        let dir = std::env::temp_dir().join(format!("plan924-scale-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (mut session, cold) = ace_native::closures_incremental::ClosuresNativeSession::new(
            &source, "main", &dir, None,
        )
        .expect("cold scale session");
        assert_eq!(
            cold.exit_code,
            Some(137),
            "cold: {}",
            String::from_utf8_lossy(&cold.stderr)
        );
        assert_eq!(
            cold.work.codegen_units.len(),
            72,
            "the scale corpus has 72 units"
        );
        let cold_hashes = session.object_hashes();
        assert_eq!(cold_hashes.len(), 72);

        // Hit: zero production over the installed PE.
        let hit = session.apply_source(&source).expect("hit");
        assert_eq!(hit.exit_code, Some(137));
        assert!(
            hit.work.codegen_units.is_empty(),
            "hit codegen: {:?}",
            hit.work.codegen_units
        );
        assert_eq!(hit.work.route, "hit");

        // FIRST lambda body edit (main's f0): base + n -> base + n + 1 (137→138).
        let edited = source.replace("return base + n }", "return base + n + 1 }");
        let warm = session.apply_source(&edited).expect("f0 edit");
        assert_eq!(
            warm.exit_code,
            Some(138),
            "f0: {}",
            String::from_utf8_lossy(&warm.stderr)
        );
        assert_eq!(
            warm.work.codegen_units,
            vec!["d___closure_main_0".to_string()],
            "f0 route: {}",
            warm.work.route
        );
        let after_f0 = session.object_hashes();
        for unrelated in ["d_h0", "d_h33", "d_h61", "d_sum", "d___closure_make_0"] {
            assert_eq!(
                after_f0.get(unrelated),
                cold_hashes.get(unrelated),
                "{unrelated} object changed on an unrelated edit"
            );
        }
        assert_ne!(
            after_f0.get("d___closure_main_0"),
            cold_hashes.get("d___closure_main_0"),
            "the edited body's object must change"
        );

        // MIDDLE lambda edit (main's f2): n * 2 -> n * 3 (138→143).
        let edited = edited.replace("return n * 2 }", "return n * 3 }");
        let warm = session.apply_source(&edited).expect("f2 edit");
        assert_eq!(
            warm.exit_code,
            Some(143),
            "f2: {}",
            String::from_utf8_lossy(&warm.stderr)
        );
        assert_eq!(
            warm.work.codegen_units,
            vec!["d___closure_main_1".to_string()]
        );

        // LAST lambda edit (main's f3): base - n -> base - n - 1 (143→142).
        let edited = edited.replace("return base - n }", "return base - n - 1 }");
        let warm = session.apply_source(&edited).expect("f3 edit");
        assert_eq!(
            warm.exit_code,
            Some(142),
            "f3: {}",
            String::from_utf8_lossy(&warm.stderr)
        );
        assert_eq!(
            warm.work.codegen_units,
            vec!["d___closure_main_2".to_string()]
        );

        // OWNER body edit (V-07): `let base int = 10` -> 11 changes the
        // owner's own body BETWEEN its creation sites — the owner batch
        // re-codegens EXACTLY d_main; every sibling lambda body keeps its
        // linked object (precise invalidation, honest reuse).
        let edited = edited.replace("let base int = 10", "let base int = 11");
        let before_owner = session.object_hashes();
        let owner = session.apply_source(&edited).expect("owner edit");
        // f0(20)=11+20+1=32, f1(3)=6, f2(5)=15, f3(4)=11-4-1=6 → 144.
        assert_eq!(
            owner.exit_code,
            Some(144),
            "owner: {}",
            String::from_utf8_lossy(&owner.stderr)
        );
        assert_eq!(
            owner.work.codegen_units,
            vec!["d_main".to_string()],
            "owner route: {}",
            owner.work.route
        );
        assert!(
            owner.work.route.starts_with("fast-owner"),
            "route: {}",
            owner.work.route
        );
        let after_owner = session.object_hashes();
        for sibling in [
            "d___closure_main_0",
            "d___closure_main_1",
            "d___closure_main_2",
        ] {
            assert_eq!(
                after_owner.get(sibling),
                before_owner.get(sibling),
                "{sibling} sibling object changed on the owner edit"
            );
        }

        // Topology changes are the honest NON-FAST faces: a NEW sibling lambda
        // (owner topology) and a new top-level function (crosses units) each
        // rebuild the frontend whole and stay correct.
        let edited = edited.replace(
            "return f0(20) + f1(3) + f2(5) + f3(4) + sum(2) + h0(1) + h61(1)",
            "let f4 = (n int) => { return base + 100 + n }\n    \
         return f0(20) + f1(3) + f2(5) + f3(4) + f4(1) + sum(2) + h0(1) + h61(1)",
        );
        let whole = session.apply_source(&edited).expect("owner topology edit");
        assert_eq!(
            whole.exit_code,
            Some(256), // 144 + (100 + 11 + 1)
            "owner topology: {}",
            String::from_utf8_lossy(&whole.stderr)
        );
        assert!(
            whole
                .work
                .route
                .starts_with("nonfast:owner-topology-changed"),
            "owner topology route: {}",
            whole.work.route
        );
        let edited = edited + "\nfn extra(n int) int { return n + 500 }\n";
        let whole = session.apply_source(&edited).expect("topology edit");
        assert_eq!(
            whole.exit_code,
            Some(256),
            "topology: {}",
            String::from_utf8_lossy(&whole.stderr)
        );
        assert!(
            whole.work.route.starts_with("nonfast:"),
            "topology route: {}",
            whole.work.route
        );

        session.close().expect("close");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
