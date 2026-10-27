//! PLAN-823 T-10 harness crate root — the same-source seam.
//!
//! `closures` is compiled from the PRODUCTION file
//! `experimental/ac-av-probe/core/src/closures.rs` (relative #[path]
//! below). Since the T-02 core window, core's lib.rs ALSO registers the
//! module (`pub mod closures;`) — this harness keeps compiling the very
//! same FILE standalone against the re-exported auto-av-core public
//! modules, proving the single-source property (the deep clone is
//! DELEGATED to `av::clone_owned_value`, so the harness re-exports `av`).

pub use auto_av_core::{av, contract, mir, owned_text, ownership, place};

#[path = "../../../core/src/closures.rs"]
pub mod closures;

// V-14 module-subset cases (contract-decision.md §5.1, r2). The fault seam
// is counted over the ARENA's global attempt sequence: every `alloc` a
// test does for its sources is attempt 1..k, so `set_fault_at` must target
// the clone's absolute attempt number.
#[cfg(test)]
mod tests {
    use crate::av::CancelFlag;
    use crate::closures::{
        deep_clone_value, CaptureMode, CaptureSpec, CloneContext, EnvCommitError, EnvLimits,
        EnvPlan, EnvPrepareError, MAX_ENV_FIELDS, MODULE_SUBSET,
    };
    use crate::contract::{
        ListHandle, ListPlaceV, MutRefV, RecordValue, StrView, StrViewSrc, Value, AV_DATA_PROFILE,
        AV_OWNED_PROFILE, AV_PROFILE, MAX_AV_INSTR_STEPS, MAX_AV_SLOT_BYTES, MAX_AV_WALL,
        MIR_OWNED_SCHEMA,
    };
    use crate::mir::{MirModule, OwnerScope};
    use crate::owned_text::StrArena;
    use std::rc::Rc;

    fn spec<'a>(mode: CaptureMode, src: &'a Value) -> CaptureSpec<'a> {
        CaptureSpec { mode, src }
    }

    fn record(ty: u32, fields: Vec<Value>) -> Value {
        Value::Record(Rc::new(RecordValue { ty, fields }))
    }

    fn str_bytes(v: &Value) -> &[u8] {
        match v {
            Value::Str(h) => h.bytes(),
            other => panic!("not a Str: {}", other.type_name()),
        }
    }

    /// The module-subset clone context: no cancel, generous wall/fuel.
    fn ctx<'a>(cancel: &'a CancelFlag, fuel: &'a mut u64) -> CloneContext<'a> {
        CloneContext {
            cancel,
            wall_left: MAX_AV_WALL,
            fuel,
            max_fuel: MAX_AV_INSTR_STEPS,
        }
    }

    #[test]
    fn av_closures_823_t10_prepare_commit_mixed() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let s = Value::Str(arena.alloc(b"auto")); // id 0
        let rec = record(7, vec![Value::Str(arena.alloc(b"lang")), Value::I32(5)]); // id 1
        let n = Value::Char('x');
        let (plan, prep) = {
            let specs = [
                spec(CaptureMode::Clone, &s),
                spec(CaptureMode::Move, &rec),
                spec(CaptureMode::Move, &n),
            ];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .expect("prepare")
        };
        // Staging cloned exactly one buffer (the record moves as-is).
        assert_eq!(arena.successful_allocs(), 3); // s, rec's Str, clone
        assert_eq!(arena.live(), 3);
        assert!(prep.bytes > 0 && prep.fields > 0);
        // Commit hands ONLY the move sources; the clone source stays.
        let (env, _) = plan.commit(vec![rec, n]).expect("commit");
        assert_eq!(env.field_count(), 3);
        assert_eq!(str_bytes(env.field(0).unwrap()), b"auto");
        match env.field(1).unwrap() {
            Value::Record(r) => {
                assert_eq!(r.ty, 7);
                assert_eq!(str_bytes(&r.fields[0]), b"lang");
                assert_eq!(r.fields[1], Value::I32(5));
            }
            other => panic!("field 1 holds {}", other.type_name()),
        }
        drop(s); // creator releases the clone source
        assert_eq!(arena.live(), 2);
        drop(env);
        assert_eq!(arena.live(), 0);
        // Reverse capture order: the moved record (id 1) frees BEFORE the
        // staged clone (id 2).
        let frees = arena.free_events();
        assert_eq!(frees.len(), 3);
        assert_eq!((frees[1].id, frees[2].id), (1, 2));
    }

    #[test]
    fn av_closures_823_t10_env_drop_reverse_once() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let a = Value::Str(arena.alloc(b"a")); // id 0
        let b = Value::Str(arena.alloc(b"b")); // id 1
        let (plan, _) = {
            let specs = [spec(CaptureMode::Move, &a), spec(CaptureMode::Move, &b)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap()
        };
        let (env, _) = plan.commit(vec![a, b]).unwrap();
        assert_eq!(arena.live(), 2);
        drop(env);
        assert_eq!(arena.live(), 0);
        let frees = arena.free_events();
        // Reverse capture order: id 1 releases before id 0; each exactly once.
        assert_eq!((frees[0].id, frees[1].id), (1, 0));
        assert_eq!(frees.iter().filter(|e| e.id == 0).count(), 1);
        assert_eq!(frees.iter().filter(|e| e.id == 1).count(), 1);

        // Clone copies release independently, exactly once each: source and
        // clone are separate owners of separate buffers.
        let s = Value::Str(arena.alloc(b"src")); // id 2
        let (plan2, _) = {
            let specs = [spec(CaptureMode::Clone, &s)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap()
        };
        let (env2, _) = plan2.commit(Vec::new()).unwrap();
        assert_eq!(arena.live(), 2);
        drop(s);
        assert_eq!(arena.live(), 1);
        drop(env2);
        assert_eq!(arena.live(), 0);
        let all = arena.free_events();
        for id in 0..=2u64 {
            assert_eq!(all.iter().filter(|e| e.id == id).count(), 1, "id {id}");
        }
    }

    #[test]
    fn av_closures_823_t10_faulted_clone_sources_readable() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let s = Value::Str(arena.alloc(b"keep")); // attempt 1
        let t = Value::Str(arena.alloc(b"other")); // attempt 2
        arena.set_fault_at(3); // first clone attempt
        let specs = [spec(CaptureMode::Clone, &s), spec(CaptureMode::Clone, &t)];
        let err = EnvPlan::prepare(
            &arena,
            &specs,
            &EnvLimits::default(),
            &mut ctx(&cancel, &mut fuel),
            Vec::new(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            EnvPrepareError::FaultedClone { attempt: 3, .. }
        ));
        assert_eq!(err.code(), "env.capture.faulted-clone");
        // Attempt exists, successful_alloc unchanged (V-07 fault分态).
        assert_eq!(arena.attempts(), 3);
        assert_eq!(arena.successful_allocs(), 2);
        assert_eq!(arena.live(), 2);
        // BOTH sources stay readable and caller-owned.
        assert_eq!(str_bytes(&s), b"keep");
        assert_eq!(str_bytes(&t), b"other");

        // Fault on the SECOND clone: the first staged clone rolls back
        // (reverse-staging release visible in the free ledger). Recount on
        // this arena: attempts=3 so far; two fresh sources are attempts
        // 4,5; the first clone succeeds (6), the second faults (7).
        arena.set_fault_at(7);
        let s2 = Value::Str(arena.alloc(b"p1")); // id 3
        let t2 = Value::Str(arena.alloc(b"p2")); // id 4
        let staged_before = arena.successful_allocs();
        let specs2 = [spec(CaptureMode::Clone, &s2), spec(CaptureMode::Clone, &t2)];
        let err2 = EnvPlan::prepare(
            &arena,
            &specs2,
            &EnvLimits::default(),
            &mut ctx(&cancel, &mut fuel),
            Vec::new(),
        )
        .unwrap_err();
        assert!(matches!(
            err2,
            EnvPrepareError::FaultedClone { attempt: 7, .. }
        ));
        let staged_id = staged_before as u64; // the clone that succeeded = id 5
        assert_eq!(arena.successful_allocs(), staged_before + 1);
        assert_eq!(arena.live(), 4); // s,t,s2,t2 — staged clone rolled back
        let frees = arena.free_events();
        assert!(
            frees.iter().any(|e| e.id == staged_id),
            "staged clone released during rollback"
        );
        assert_eq!(str_bytes(&s2), b"p1");
        assert_eq!(str_bytes(&t2), b"p2");
    }

    #[test]
    fn av_closures_823_t10_move_source_preserved_on_failure() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let m = Value::Str(arena.alloc(b"mover")); // attempt 1
        let c = Value::Str(arena.alloc(b"clone-me")); // attempt 2
        arena.set_fault_at(3); // the clone faults
        let err = {
            let specs = [spec(CaptureMode::Move, &m), spec(CaptureMode::Clone, &c)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap_err()
        };
        assert!(matches!(
            err,
            EnvPrepareError::FaultedClone { attempt: 3, .. }
        ));
        // The to-move source was never consumed: prepare only borrows.
        assert_eq!(arena.live(), 2);
        assert_eq!(str_bytes(&m), b"mover");
        assert_eq!(str_bytes(&c), b"clone-me");
        // It remains USABLE: a clean retry without the fault succeeds.
        arena.set_fault_at(u64::MAX);
        let (plan, _) = {
            let specs = [spec(CaptureMode::Move, &m)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap()
        };
        let (env, _) = plan.commit(vec![m]).unwrap();
        assert_eq!(str_bytes(env.field(0).unwrap()), b"mover");
        drop(env);
        assert_eq!(arena.live(), 1, "the clone source c is still caller-owned");
    }

    #[test]
    fn av_closures_823_t10_borrow_kinds_rejected() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let s = Value::Str(arena.alloc(b"src")); // attempt 1
        let view = Value::StrView(StrView {
            src: StrViewSrc::Share(match &s {
                Value::Str(h) => h.clone(),
                _ => unreachable!(),
            }),
            start: 0,
            end: 3,
        });
        let token = Value::MutRef(MutRefV {
            frame: 0,
            slot: 1,
            gen: 2,
        });
        arena.set_fault_at(1); // must never fire: rejection precedes attempts
        let err = {
            let specs = [
                spec(CaptureMode::Clone, &view),
                spec(CaptureMode::Move, &token),
            ];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap_err()
        };
        assert_eq!(
            err,
            EnvPrepareError::UnsupportedKind { kind: "str" },
            "first bad kind rejects before the token is even walked"
        );
        assert_eq!(err.code(), "env.capture.unsupported-kind");
        assert_eq!(arena.attempts(), 1, "kind rejection added no attempts");
        assert_eq!(arena.live(), 1);
        // MutRef alone is rejected the same way (move mode included).
        let err2 = {
            let specs = [spec(CaptureMode::Move, &token)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap_err()
        };
        assert_eq!(err2, EnvPrepareError::UnsupportedKind { kind: "mut-ref" });
        assert_eq!(arena.attempts(), 1);
    }

    #[test]
    fn av_closures_823_t10_commit_drift_rejected() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let s = Value::Str(arena.alloc(b"staged")); // attempt 1
        let m = Value::Str(arena.alloc(b"move-src")); // attempt 2

        // (a) wrong source count — sources returned untouched.
        let (plan, _) = {
            let specs = [spec(CaptureMode::Clone, &s), spec(CaptureMode::Move, &m)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap()
        };
        let (err, returned) = plan.commit(Vec::new()).unwrap_err();
        assert_eq!(
            err,
            EnvCommitError::SourceCountMismatch {
                expected: 1,
                got: 0
            }
        );
        assert_eq!(err.code(), "env.commit.source-count");
        assert!(returned.is_empty());

        // (b) kind drift — the handed source comes back intact.
        let (plan, _) = {
            let specs = [spec(CaptureMode::Move, &m)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap()
        };
        let (err, returned) = plan.commit(vec![Value::I32(9)]).unwrap_err();
        assert_eq!(
            err,
            EnvCommitError::SourceKind {
                index: 0,
                expected: "Str",
                got: "i32"
            }
        );
        assert_eq!(err.code(), "env.commit.source-kind");
        assert_eq!(returned, vec![Value::I32(9)]);

        // (c) same-kind content drift over budget — source returned, and a
        // staged clone rolled back (live returns to source-only).
        let cell = std::mem::size_of::<Value>();
        let tight = EnvLimits {
            max_env_bytes: cell + 8, // exactly one small Str
            ..EnvLimits::default()
        };
        let tiny = Value::Str(arena.alloc(b"tiny")); // attempt 3
        let (plan, _) = {
            let specs = [spec(CaptureMode::Move, &tiny)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &tight,
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap()
        };
        let huge = Value::Str(arena.alloc(&vec![b'x'; 4096])); // same kind
        let (err, returned) = plan.commit(vec![huge]).unwrap_err();
        assert!(matches!(err, EnvCommitError::EnvBytesBudget { .. }));
        assert_eq!(err.code(), "env.commit.bytes-budget");
        assert_eq!(returned.len(), 1);
        assert_eq!(
            str_bytes(&returned[0]).len(),
            4096,
            "source unowned-but-intact"
        );
        drop(tiny);
        drop(s);
        drop(m);
        drop(returned);
        assert_eq!(arena.live(), 0, "commit-time budget map is gone");
    }

    #[test]
    fn av_closures_823_t10_budget_boundaries() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        // Field count: exactly at cap passes, cap+1 rejects (both sides).
        let scalars: Vec<Value> = (0..MAX_ENV_FIELDS as i32).map(Value::I32).collect();
        let limits = EnvLimits::default();
        {
            let specs: Vec<CaptureSpec> =
                scalars.iter().map(|v| spec(CaptureMode::Move, v)).collect();
            assert!(EnvPlan::prepare(
                &arena,
                &specs,
                &limits,
                &mut ctx(&cancel, &mut fuel),
                Vec::new()
            )
            .is_ok());
            let over: Vec<CaptureSpec> = scalars
                .iter()
                .chain(std::iter::once(&Value::Bool(true)))
                .map(|v| spec(CaptureMode::Move, v))
                .collect();
            assert_eq!(
                EnvPlan::prepare(
                    &arena,
                    &over,
                    &limits,
                    &mut ctx(&cancel, &mut fuel),
                    Vec::new()
                )
                .unwrap_err(),
                EnvPrepareError::TooManyFields {
                    got: MAX_ENV_FIELDS + 1,
                    max: MAX_ENV_FIELDS
                }
            );
        }
        // Bytes: two 2-byte Str captures land exactly on the cap.
        let cell = std::mem::size_of::<Value>();
        let s1 = Value::Str(arena.alloc(b"aa")); // attempt 1
        let s2 = Value::Str(arena.alloc(b"bb")); // attempt 2
        let exact = 2 * (cell + 2);
        let reject = EnvLimits {
            max_env_bytes: exact - 1,
            ..EnvLimits::default()
        };
        let attempts_before = arena.attempts();
        {
            let specs = [spec(CaptureMode::Clone, &s1), spec(CaptureMode::Move, &s2)];
            assert_eq!(
                EnvPlan::prepare(
                    &arena,
                    &specs,
                    &reject,
                    &mut ctx(&cancel, &mut fuel),
                    Vec::new()
                )
                .unwrap_err(),
                EnvPrepareError::EnvBytesBudget {
                    would: exact,
                    max: exact - 1
                }
            );
        }
        assert_eq!(
            arena.attempts(),
            attempts_before,
            "budget rejection precedes any allocation attempt"
        );
        let pass = EnvLimits {
            max_env_bytes: exact,
            ..EnvLimits::default()
        };
        let (plan, _) = {
            let specs = [spec(CaptureMode::Clone, &s1), spec(CaptureMode::Move, &s2)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &pass,
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap()
        };
        let (env, _) = plan.commit(vec![s2]).unwrap();
        assert_eq!(env.field_count(), 2);
    }

    #[test]
    fn av_closures_823_t10_unknown_profile_rejected() {
        // Version/window control: a module declaring av.closures/1 passes
        // the CURRENT verifier only under the closures schema/profile
        // pair; the schema mismatch axis is the honest boundary test here
        // (a foreign schema under the profile rejects).
        let mut m = MirModule::new_owned(OwnerScope {
            schema: MIR_OWNED_SCHEMA.to_string(),
            module: "plan823".to_string(),
        });
        m.profile = "av.not-a-profile/1".to_string();
        match auto_av_core::verify_mir(&m) {
            Err(auto_av_core::MirVerifyError::Unsupported { code, .. }) => {
                assert_eq!(code, "mir.profile");
            }
            other => panic!("expected mir.profile rejection, got {other:?}"),
        }
    }

    #[test]
    fn av_closures_823_t10_module_subset_identity() {
        assert_ne!(MODULE_SUBSET, AV_PROFILE);
        assert_ne!(MODULE_SUBSET, AV_DATA_PROFILE);
        assert_ne!(MODULE_SUBSET, AV_OWNED_PROFILE);
        assert_ne!(MODULE_SUBSET, "av.closures/1");
        let d = EnvLimits::default();
        assert_eq!(d.max_fields, MAX_ENV_FIELDS);
        assert_eq!(d.max_env_bytes, MAX_AV_SLOT_BYTES);
    }

    #[test]
    fn av_closures_823_t10_deep_clone_independent_buffers() {
        // Canonical-implementation axis: the delegated visitor gives a
        // fresh independent buffer per reachable owned Str (the same
        // implementation av.rs's CLONE rides — r2 convergence).
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let rec = record(3, vec![Value::Str(arena.alloc(b"x")), Value::I32(1)]);
        let (cloned, acc) = deep_clone_value(&rec, &arena, &mut ctx(&cancel, &mut fuel)).unwrap();
        assert_eq!(cloned, rec, "content-equal after deep clone");
        assert!(acc.bytes >= 2 * std::mem::size_of::<Value>() && acc.fields >= 3);
        assert_eq!(arena.live(), 2);
        drop(rec);
        assert_eq!(arena.live(), 1, "clone independent of the source");
        match &cloned {
            Value::Record(r) => assert_eq!(str_bytes(&r.fields[0]), b"x"),
            other => panic!("clone holds {}", other.type_name()),
        }
        drop(cloned);
        assert_eq!(arena.live(), 0);
        // Fault mid-clone reports the faulted attempt; source untouched.
        let s = Value::Str(arena.alloc(b"soon-fault"));
        arena.set_fault_at(arena.attempts() + 1); // the clone's attempt
        let err = deep_clone_value(&s, &arena, &mut ctx(&cancel, &mut fuel)).unwrap_err();
        match err {
            EnvPrepareError::FaultedClone { attempt, .. } => {
                assert_eq!(attempt, arena.attempts());
            }
            other => panic!("expected FaultedClone, got {other:?}"),
        }
        assert_eq!(str_bytes(&s), b"soon-fault");
    }

    /// T-02 (r2): List/Variant captures are expressible now that D822
    /// landed; Place tokens reject like the other borrow kinds.
    #[test]
    fn av_closures_823_t10_list_variant_place_kinds() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let list = Value::List(ListHandle::detached(vec![
            Value::Str(arena.alloc(b"e0")),
            Value::I32(42),
        ]));
        let var = Value::Variant(crate::contract::VariantValue {
            ty: 9,
            disc: 1,
            payload: Some(Rc::new(Value::Str(arena.alloc(b"vp")))),
        });
        // Clone-capture the list: the staged clone is an INDEPENDENT
        // ledgered buffer with equal content.
        let (plan, _) = {
            let specs = [
                spec(CaptureMode::Clone, &list),
                spec(CaptureMode::Move, &var),
            ];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .expect("list/variant captures are admissible")
        };
        let (env, _) = plan.commit(vec![var]).unwrap();
        match env.field(0).unwrap() {
            Value::List(h) => {
                let buf = h.rc.borrow();
                assert_eq!(buf.elems.len(), 2);
                assert_eq!(str_bytes(&buf.elems[0]), b"e0");
                assert!(buf.id > 0, "the staged clone is a fresh ledgered buffer");
            }
            other => panic!("field 0 holds {}", other.type_name()),
        }
        // Independent: drop the source list, the env copy stays readable.
        drop(list);
        match env.field(0).unwrap() {
            Value::List(h) => assert_eq!(h.rc.borrow().elems.len(), 2),
            other => panic!("{}", other.type_name()),
        }
        drop(env);
        assert_eq!(arena.live(), 0);

        // A Place token rejects at prepare BEFORE any attempt (the escape
        // law covers 822's frame-rooted element references).
        let place = Value::Place(ListPlaceV {
            frame: 0,
            slot: 3,
            buf_id: 1,
            epoch: 0,
            index: 0,
            elem_gen: 0,
        });
        let attempts_before = arena.attempts();
        let err = {
            let specs = [spec(CaptureMode::Clone, &place)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap_err()
        };
        assert_eq!(err, EnvPrepareError::UnsupportedKind { kind: "place" });
        assert_eq!(arena.attempts(), attempts_before);
    }

    /// T-03 (r3): the Borrow mode — the module-level staging seam. The
    /// executor mints the share view; the module validates the source
    /// domain (owned Str only) and adopts the staged value; a Borrow spec
    /// without its staged view is a caller contract violation.
    #[test]
    fn av_closures_823_t10_borrow_mode_staging() {
        let arena = StrArena::new();
        let cancel = CancelFlag::new(false);
        let mut fuel = 0u64;
        let s = Value::Str(arena.alloc(b"lend"));
        let view = Value::StrView(StrView {
            src: StrViewSrc::Share(match &s {
                Value::Str(h) => h.clone(),
                _ => unreachable!(),
            }),
            start: 0,
            end: 4,
        });

        // (a) A Borrow of a non-Str kind rejects BEFORE any attempt.
        let n = Value::I32(7);
        let attempts_before = arena.attempts();
        let err = {
            let specs = [spec(crate::closures::CaptureMode::Borrow, &n)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap_err()
        };
        assert_eq!(err, EnvPrepareError::UnsupportedKind { kind: "i32" });
        assert_eq!(arena.attempts(), attempts_before);

        // (b) The staged view lands as the env field; the source stays
        // with the creator and stays readable after the env's release
        // (the SHARE keeps the bytes — snapshot semantics).
        let (plan, _) = {
            let specs = [spec(crate::closures::CaptureMode::Borrow, &s)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                vec![view],
            )
            .unwrap()
        };
        let (env, _) = plan.commit(Vec::new()).unwrap();
        match env.field(0).unwrap() {
            Value::StrView(v) => assert_eq!((v.start, v.end), (0, 4)),
            other => panic!("borrow field holds {}", other.type_name()),
        }
        drop(s);
        match env.field(0).unwrap() {
            Value::StrView(v) => {
                let r = crate::place::resolve_share_view(v).expect("share resolves");
                assert_eq!(r.handle.bytes(), b"lend");
            }
            other => panic!("{}", other.type_name()),
        }
        drop(env);
        assert_eq!(arena.live(), 0, "the share released with the env");

        // (c) A Borrow spec without its staged view is a caller contract
        // violation (never a language behavior).
        let s2 = Value::Str(arena.alloc(b"again"));
        let err2 = {
            let specs = [spec(crate::closures::CaptureMode::Borrow, &s2)];
            EnvPlan::prepare(
                &arena,
                &specs,
                &EnvLimits::default(),
                &mut ctx(&cancel, &mut fuel),
                Vec::new(),
            )
            .unwrap_err()
        };
        assert!(matches!(err2, EnvPrepareError::BorrowStaging(_)));
        assert_eq!(err2.code(), "env.capture.borrow-staging");
    }
}
