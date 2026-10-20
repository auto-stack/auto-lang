//! T-05 tier: scale/budget/GC/deep chains (PLAN-798 V-09/V-10 resource
//! legs) + the turnaround smoke (full six-scenario release bench runs via
//! scripts/verify-ac-av-798.ps1 -Scope turnaround).

use auto_av_core::contract::{ResourceKind, MAX_FRONTEND_NESTING, MAX_TABLE_ENTRIES};
use auto_av_core::mir::*;
use auto_av_core::verify_mir_with_deadline;
use auto_av_probe::driver::{front_door_admit, front_door_admit_atom, Driver};
use std::time::{Duration, Instant};

fn owner() -> OwnerScope {
    OwnerScope {
        schema: auto_av_core::contract::MIR_SCHEMA.to_string(),
        module: "big".into(),
    }
}

/// V-09: a 4096-block straight chain + a mixed CFG (branches AND a legal
/// loop) traverse/verify inside the frozen work-step/wall budgets.
#[test]
fn av_resources_4096_block_chain_within_budget() {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let tb = m.intern_type(MirType::Bool);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let c0 = m.intern_const(MirConst::I32(0));
    const N: u32 = 4096;
    // Chain: block i assigns r (slot 0) and jumps to i+1; last returns r.
    let mut blocks = Vec::with_capacity(N as usize + 1);
    for i in 0..N {
        blocks.push(MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: c0 }],
            term: Terminator::Jump { target: i + 1 },
        });
    }
    blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Return { value: 0 },
    });
    // Mixed: a diamond + a loop appended at the end of the chain. The
    // branch condition is a bool temp initialized in the last chain block.
    let c_true = m.intern_const(MirConst::Bool(true));
    blocks[N as usize].instrs.push(MirInstr::Const {
        dst: 1,
        value: c_true,
    });
    blocks[N as usize].term = Terminator::Branch {
        cond: 1,
        then: N + 1,
        else_: N + 2,
    };
    blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Jump { target: N + 3 },
    });
    blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Jump { target: N + 3 },
    });
    // loop header N+3 → body N+4 → back to N+3; exit N+5.
    blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Jump { target: N + 4 },
    });
    blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Branch {
            cond: 1,
            then: N + 3,
            else_: N + 5,
        },
    });
    blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Return { value: 0 },
    });
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: Some("r".into()),
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: tb,
                name: None,
            },
        ],
        blocks,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    m.entry = 0;
    assert!(m.table_entries() <= MAX_TABLE_ENTRIES);
    let started = Instant::now();
    let checked = verify_mir_with_deadline(&m, started + Duration::from_secs(5))
        .expect("4096 chain + diamond + loop verifies");
    let elapsed = started.elapsed();
    assert!(elapsed < Duration::from_secs(5), "verify wall {elapsed:?}");
    assert!(
        checked.stats().work_steps <= 2_000_000,
        "within the frozen step budget"
    );
}

/// V-09: the same shape OVER the table budget is a resource rejection
/// (never an illegal-syntax classification), checked before deep analysis.
#[test]
fn av_resources_over_table_budget_is_resource() {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let blocks: Vec<MirBlock> = (0..70_000)
        .map(|i: u32| MirBlock {
            instrs: vec![],
            term: Terminator::Jump {
                target: i.saturating_sub(1),
            },
        })
        .collect();
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![],
        blocks,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    m.entry = 0;
    let err = verify_mir_with_deadline(&m, Instant::now() + Duration::from_secs(5)).unwrap_err();
    assert!(matches!(
        err,
        auto_av_core::mir_verify::MirVerifyError::Resource {
            kind: ResourceKind::TableEntries,
            ..
        }
    ));
}

/// V-09: source front door — 128 nesting boundary BOTH sides; rejection is
/// a RESOURCE classification that fires BEFORE the (iterative) parser, and
/// the input is never handed to the frontend afterwards.
#[test]
fn av_resources_front_door_nesting_boundary() {
    // Just under: 120 nested parens around an expression.
    let ok_src = format!(
        "fn main() int {{\n    var x = {}1{}\n    return x\n}}\n",
        "(".repeat(120),
        ")".repeat(120)
    );
    assert!(front_door_admit(&ok_src).is_ok(), "120 nesting admits");
    let mut d = Driver::ephemeral();
    let r = d.request(&ok_src, "d_main", &[]);
    assert!(r.ok(), "under-budget deep parens execute: {:?}", r.diag);

    // Over: 4096 nested parens — resource rejection at the door.
    let deep_src = format!(
        "fn main() int {{\n    var x = {}1{}\n    return x\n}}\n",
        "(".repeat(4096),
        ")".repeat(4096)
    );
    let started = Instant::now();
    let err = front_door_admit(&deep_src).unwrap_err();
    assert_eq!(err.code, "av.resource.frontend-nesting");
    assert!(
        started.elapsed() < Duration::from_millis(500),
        "door is cheap"
    );
    // And through the driver: the SAME resource classification, before the
    // frontend (frontend_reject counter absent, admission_reject present).
    let r = d.request(&deep_src, "d_main", &[]);
    let diag = r.diag.expect("rejected");
    assert_eq!(diag.stage, "admission");
    assert_eq!(diag.code, "av.resource.frontend-nesting");
    assert_eq!(*r.counters.get("admission_reject").unwrap_or(&0), 1);
}

/// F798-R11-01: the CURRENT sibling COMPOSES internally — a deep call as
/// the FIRST OPERAND of the sibling's own left-assoc chain is ONE path
/// (child calls + serial chain add); only COMPLETED siblings fold by max.
#[test]
fn av_resources_composed_path_inside_one_argument() {
    let nested =
        |n: usize, inner: &str| format!("{}{}{}", "idfn(x: ".repeat(n), inner, ")".repeat(n));
    let with_chain = |inner_nest: usize| -> String {
        let value = format!(
            "{} + {}",
            nested(inner_nest, "1"),
            vec!["1"; 69].join(" + ")
        );
        format!(
            "fn idfn(x int) int {{
    return x
}}

fn main() int {{
    return idfn(x: {value})
}}
"
        )
    };
    let deep_only = |inner_nest: usize| -> String {
        format!(
            "fn idfn(x int) int {{
    return x
}}

fn main() int {{
    return idfn(x: {})
}}
",
            nested(inner_nest, "1")
        )
    };
    let mut d = Driver::ephemeral();
    // Legal low-depth controls: 3-nest + 70-chain (true ~74) passes with
    // value 70; 60-nest + single item, no chain (true ~62) passes with
    // value 1.
    for (src, tag, want) in [
        (with_chain(3), "3-nest+chain", 70i32),
        (deep_only(60), "60-nest-only", 1),
    ] {
        let _ = tag;
        let door = front_door_admit(&src);
        assert!(door.is_ok(), "{tag} control admits: {:?}", door.err());
        let r = d.request(&src, "d_main", &[]);
        assert!(r.ok(), "{tag} control executes: {:?}", r.diag);
        assert_eq!(r.value_i32(), Some(want));
    }
    // The R11 shape: 60-nest + 70-chain INSIDE ONE named argument — true
    // path 131 — must be rejected AT THE DOOR (was bypassed, Driver
    // returned 70).
    let over = with_chain(60);
    let err = front_door_admit(&over).unwrap_err();
    assert_eq!(err.code, "av.resource.frontend-nesting");
    let r = d.request(&over, "d_main", &[]);
    let diag = r.diag.clone().expect("rejected");
    assert_eq!(diag.stage, "admission");
    assert_eq!(diag.code, "av.resource.frontend-nesting");
    // The positional-argument equivalent was already rejected and stays.
    let positional = format!(
        "fn idfn(x int) int {{
    return x
}}

fn main() int {{
    return idfn({})
}}
",
        format!("{} + {}", nested(60, "1"), vec!["1"; 69].join(" + "))
    );
    assert!(front_door_admit(&positional).is_err());
}

/// F798-R10-01: two SIBLING ARGUMENTS' maximal paths never concatenate —
/// a 60-paren argument beside a 70-term chain argument stays legal in
/// either order and at various splits, while true composition (the chain
/// INSIDE the nesting) still hits the door.
#[test]
fn av_resources_sibling_max_paths_not_concatenated() {
    let parens = |n: usize| format!("{}1{}", "(".repeat(n), ")".repeat(n));
    let chain = |n: usize| vec!["1"; n].join(" + ");
    let two_arg = |pa: usize, ch: usize, swap: bool| {
        let (a, b) = if swap {
            (chain(ch), parens(pa))
        } else {
            (parens(pa), chain(ch))
        };
        format!(
            "fn first(a int, b int) int {{
    return a
}}

fn main() int {{
    return first(a: {a}, b: {b})
}}
"
        )
    };
    let mut d = Driver::ephemeral();
    for (pa, ch) in [(60usize, 70usize), (60, 69), (40, 90)] {
        for swap in [false, true] {
            let src = two_arg(pa, ch, swap);
            let door = front_door_admit(&src);
            assert!(
                door.is_ok(),
                "sibling maxima {pa}/{ch} (swap={swap}) never add: {:?}",
                door.err()
            );
            let r = d.request(&src, "d_main", &[]);
            assert!(
                r.ok(),
                "sibling {pa}/{ch} swap={swap} executes: {:?}",
                r.diag
            );
            // first returns a: the parens literal (1) or, swapped, the
            // chain's value.
            let want = if swap { ch as i32 } else { 1 };
            assert_eq!(r.value_i32(), Some(want));
        }
    }
    // The TRUE composition control stays rejected: the chain INSIDE the
    // nesting is one path of ~pa+ch levels.
    let composed = format!(
        "fn main() int {{
    return {}
}}
",
        format!("{}{}{}", "(".repeat(60), chain(70), ")".repeat(60))
    );
    assert!(
        front_door_admit(&composed).is_err(),
        "60 nesting around a 70-chain (~130 path) is still rejected"
    );
}

/// F798-R9-01: sequential STATEMENTS are independent expressions — 64
/// `let vI = (1 + 1)` lines stay legal (the metric is a max over paths,
/// never a sum), while serial/nested chains still hit the 128 door.
#[test]
fn av_resources_sequential_shallow_statements_not_depth_summed() {
    let stmts = |n: usize| {
        let mut src = String::from("fn main() int {\n");
        for i in 0..n {
            src.push_str(&format!("    let v{i} = (1 + 1)\n"));
        }
        src.push_str("    return v0\n}\n");
        src
    };
    let mut d = Driver::ephemeral();
    for n in [16usize, 63, 64, 65, 100] {
        let src = stmts(n);
        let door = front_door_admit(&src);
        assert!(
            door.is_ok(),
            "{n} shallow statements admit: {:?}",
            door.err()
        );
        let r = d.request(&src, "d_main", &[]);
        assert!(r.ok(), "{n} shallow statements execute: {:?}", r.diag);
        assert_eq!(r.value_i32(), Some(2), "{n} statements value");
    }
    // The deep rejections stay: serial chain and mixed nesting+chain.
    let serial = format!(
        "fn main() int {{\n    return {}\n}}\n",
        vec!["(1)"; 129].join(" + ")
    );
    assert!(
        front_door_admit(&serial).is_err(),
        "serial 129 still rejected"
    );
    // Mixed: a 70-deep nesting of calls each containing a 70-op chain —
    // the true max path (~140) must be rejected, not just one axis.
    let mut mixed = String::from("fn main() int {\n    return ");
    mixed.push_str(&"idfn(".repeat(0)); // built below via named chain instead
    let chain = vec!["1"; 70].join(" + ");
    let nested = format!("fn idfn(x int) int {{\n    return x\n}}\n\nfn main() int {{\n    return idfn(idfn(idfn({})))\n}}\n", chain);
    let _ = mixed;
    assert!(
        front_door_admit(&nested).is_ok(),
        "3-deep nesting of a 70-op chain (path ~74) is legal"
    );
    let deep_nested = format!(
        "fn idfn(x int) int {{\n    return x\n}}\n\nfn main() int {{\n    return {}\n}}\n",
        format!(
            "{}{}",
            "idfn(".repeat(60),
            format!("{}{}", chain, ")".repeat(60))
        )
    );
    assert!(
        front_door_admit(&deep_nested).is_err(),
        "60-deep nesting of a 70-op chain (path ~130) is rejected"
    );
}

/// F798-R8-01: sibling arguments of ONE call are separate expressions —
/// a shallow WIDE call (each argument a single add) stays legal at any
/// arity the wire allows, while serial chains still hit the 128 door.
#[test]
fn av_resources_wide_shallow_call_not_depth_rejected() {
    let wide = |n: usize| {
        let params = (0..n)
            .map(|i| format!("a{i} int"))
            .collect::<Vec<_>>()
            .join(", ");
        let args = (0..n)
            .map(|i| format!("a{i}: 1 + 0"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "fn first({params}) int {{\n    return a0\n}}\n\nfn main() int {{\n    return first({args})\n}}\n"
        )
    };
    let mut d = Driver::ephemeral();
    for n in [16usize, 128, 129, 200] {
        let src = wide(n);
        let door = front_door_admit(&src);
        assert!(
            door.is_ok(),
            "{n} shallow args admit at the door: {:?}",
            door.err()
        );
        let r = d.request(&src, "d_main", &[]);
        assert!(r.ok(), "{n} shallow args execute: {:?}", r.diag);
        assert_eq!(r.value_i32(), Some(1), "{n} args value");
    }
    // The serial-chain rejections from R7 stay.
    let deep = format!(
        "fn main() int {{\n    return {}\n}}\n",
        vec!["(1)"; 129].join(" + ")
    );
    assert!(
        front_door_admit(&deep).is_err(),
        "serial 129 still rejected"
    );
}

/// F798-R7-02: bool-literal operands and named-argument separators are
/// expression-internal — `true && true && …` and `idfn(x:1) + idfn(x:1) + …`
/// accumulate construction depth like any other chain and hit the same
/// 128 door.
#[test]
fn av_resources_bool_and_named_arg_chains_front_door() {
    let bool_chain = |n: usize| {
        format!(
            "fn main() int {{\n    var x = {}\n    return 0\n}}\n",
            vec!["true"; n].join(" && ")
        )
    };
    let named_chain = |n: usize| {
        format!(
            "fn idfn(x int) int {{\n    return x\n}}\n\nfn main() int {{\n    var x = {}\n    return x\n}}\n",
            vec!["idfn(x:1)"; n].join(" + ")
        )
    };
    let mut d = Driver::ephemeral();
    for n in [1usize, 2, 16] {
        let src = bool_chain(n);
        assert!(front_door_admit(&src).is_ok(), "bool chain {n} admits");
        let r = d.request(&src, "d_main", &[]);
        assert!(r.ok(), "bool chain {n} executes: {:?}", r.diag);
        let src = named_chain(n);
        assert!(front_door_admit(&src).is_ok(), "named chain {n} admits");
        let r = d.request(&src, "d_main", &[]);
        assert!(r.ok(), "named chain {n} executes: {:?}", r.diag);
        assert_eq!(r.value_i32(), Some(n as i32), "value proves evaluation");
    }
    for n in [129usize, 400, 600] {
        for (tag, src) in [("bool", bool_chain(n)), ("named", named_chain(n))] {
            let err = front_door_admit(&src).unwrap_err();
            assert_eq!(err.code, "av.resource.frontend-nesting", "{tag} n={n}");
            let r = d.request(&src, "d_main", &[]);
            let diag = r.diag.clone().expect("rejected");
            assert_eq!(diag.stage, "admission", "{tag} n={n}: at the door");
            assert_eq!(diag.code, "av.resource.frontend-nesting", "{tag} n={n}");
        }
    }
}

/// F798-R6-04: BRACKETED operands do not reset the construction run —
/// `(1) + (1) + …` nests exactly like the unbracketed chain and must hit
/// the same 128 front door, at the DOOR (never `mir.lower.depth` from
/// inside the recursive parser).
#[test]
fn av_resources_parenthesized_flat_chain_front_door_boundary() {
    let chain = |n: usize| {
        format!(
            "fn main() int {{
    var x = {}
    return x
}}
",
            vec!["(1)"; n].join(" + ")
        )
    };
    // Under budget: 100 bracketed terms execute.
    let ok_src = chain(100);
    assert!(
        front_door_admit(&ok_src).is_ok(),
        "100-term bracketed chain admits"
    );
    let mut d = Driver::ephemeral();
    let r = d.request(&ok_src, "d_main", &[]);
    assert!(
        r.ok(),
        "under-budget bracketed chain executes: {:?}",
        r.diag
    );
    assert_eq!(r.value_i32(), Some(100), "value proves full evaluation");

    // Over budget: the R6 replay numbers — 129/400/600 all rejected AT THE
    // DOOR as resource (600 previously slipped through to mir.lower.depth).
    for n in [129usize, 400, 600] {
        let over = chain(n);
        let err = front_door_admit(&over).unwrap_err();
        assert_eq!(err.code, "av.resource.frontend-nesting", "n={n}");
        assert_eq!(err.stage, "admission");
        let r = d.request(&over, "d_main", &[]);
        let diag = r.diag.clone().expect("rejected");
        assert_eq!(diag.stage, "admission", "n={n}");
        assert_eq!(diag.code, "av.resource.frontend-nesting", "n={n}");
    }
}

/// F798-R3-04: flat operator CHAINS are expression-construction depth —
/// a `1+1+…` run past the frozen budget is a front-door RESOURCE
/// rejection, not a `mir.lower.depth` unsupported raised from inside the
/// recursive parser at its own 512 limit.
#[test]
fn av_resources_flat_chain_front_door_boundary() {
    let chain = |n: usize| {
        format!(
            "fn main() int {{
    var x = 1{}
    return x
}}
",
            " + 1".repeat(n - 1)
        )
    };
    // Under budget: 100 terms execute.
    let ok_src = chain(100);
    assert!(front_door_admit(&ok_src).is_ok(), "100-term chain admits");
    let mut d = Driver::ephemeral();
    let r = d.request(&ok_src, "d_main", &[]);
    assert!(r.ok(), "under-budget chain executes: {:?}", r.diag);

    // Over budget: 200 terms — resource rejection at the door, before the
    // frontend (and never the parser's own depth error).
    let over_src = chain(200);
    let started = Instant::now();
    let err = front_door_admit(&over_src).unwrap_err();
    assert_eq!(err.code, "av.resource.frontend-nesting");
    assert_eq!(err.stage, "admission");
    assert!(
        started.elapsed() < Duration::from_millis(500),
        "door is cheap"
    );
    let r = d.request(&over_src, "d_main", &[]);
    let diag = r.diag.expect("rejected");
    assert_eq!(diag.stage, "admission");
    assert_eq!(diag.code, "av.resource.frontend-nesting");
    // A chain that would have sailed through the parser's 512 limit must
    // STILL be rejected at the 128 door (the R3 replay shape: 400 terms
    // used to succeed, 600 hit mir.lower.depth).
    let r400 = d.request(&chain(400), "d_main", &[]);
    let diag = r400.diag.expect("400-term chain rejected at the door");
    assert_eq!(diag.stage, "admission");
    assert_eq!(diag.code, "av.resource.frontend-nesting");
}

/// F798-R12-01: the construction path follows the REAL association — a
/// deep call as the LAST operand of a left-associative chain is a sibling
/// BRANCH (deepest left spine 70 + the call = true path 71, legal and
/// returns 70), while the same call as the FIRST operand sits ON the left
/// spine (true path 131, still a resource rejection). The pre-R12 door
/// summed the two branches into 192 and rejected both.
#[test]
fn av_resources_tail_operand_deep_call_not_concatenated() {
    let nested = |n: usize| format!("{}1{}", "idfn(x: ".repeat(n), ")".repeat(n));
    let src = |deep_first: bool| {
        let mut operands = vec!["1".to_string(); 70];
        operands[if deep_first { 0 } else { 69 }] = nested(60);
        format!(
            "fn idfn(x int) int {{\n    return x\n}}\n\nfn main() int {{\n    return idfn(x: {})\n}}\n",
            operands.join(" + ")
        )
    };
    let mut d = Driver::ephemeral();
    // Tail operand: the deep call and the arithmetic chain are branches of
    // one tree — true path 71, admits and executes.
    let tail = src(false);
    assert!(
        front_door_admit(&tail).is_ok(),
        "tail deep call is a sibling branch, not a concatenated path"
    );
    let r = d.request(&tail, "d_main", &[]);
    assert!(r.ok(), "tail deep call executes: {:?}", r.diag);
    assert_eq!(r.value_i32(), Some(70));
    // Head operand: the deep call is on the left spine — true path 131.
    let head = src(true);
    let err = front_door_admit(&head).unwrap_err();
    assert_eq!(err.code, "av.resource.frontend-nesting");
    let r = d.request(&head, "d_main", &[]);
    assert_eq!(r.diag.expect("rejected").stage, "admission");
    // Low-depth controls in both positions stay legal.
    let mut shallow_tail = vec!["1".to_string(); 70];
    shallow_tail[69] = nested(3);
    let ok = format!(
        "fn idfn(x int) int {{\n    return x\n}}\n\nfn main() int {{\n    return idfn(x: {})\n}}\n",
        shallow_tail.join(" + ")
    );
    assert!(front_door_admit(&ok).is_ok());
}

/// V-09: the recursion-critical ATOM front door — 4096-deep nested lists
/// are rejected before the recursive document parser runs.
#[test]
fn av_resources_atom_front_door_rejects_before_recursive_parser() {
    let ok = "[a: [1, [2, 3]], b: 4]";
    assert!(front_door_admit_atom(ok).is_ok());
    let mut deep = String::from("[");
    deep.push_str(&"[1,".repeat(4096));
    deep.push('1');
    deep.push_str(&"]".repeat(4100));
    let started = Instant::now();
    let err = front_door_admit_atom(&deep).unwrap_err();
    assert_eq!(err.code, "av.resource.frontend-nesting");
    assert!(started.elapsed() < Duration::from_millis(500));
    let _ = MAX_FRONTEND_NESTING;
}

/// V-09: cold-product scale — ≥32 MiB of REAL MIR/ABC product units in the
/// managed store (streamed as ≥1 MiB modules under the 4 MiB input cap),
/// random read-only unit access re-verifies, 512 local revisions on the
/// working module stay correct, and a GC pass converges the water level
/// with leases released. (Sustained store size assertions use the store's
/// own space report.)
#[test]
fn av_resources_32mib_product_random_read_and_churn() {
    let root = std::env::temp_dir().join(format!("av798-res-32m-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let store = auto_av_probe::cache::open_store(&root).unwrap();
    // 60 modules x ~31k single-instr blocks ≈ >32 MiB of ABC product
    // (each module stays under the 65_536 table-entry and 4 MiB input
    // budgets; the products stream through ≤256 KiB codec pages).
    let per_fn_blocks = 31_000u32;
    let mut total_abc = 0usize;
    let mut artifact_ids = Vec::new();
    for mi in 0..64 {
        let mut m = MirModule::unit(OwnerScope {
            schema: auto_av_core::contract::MIR_SCHEMA.to_string(),
            module: format!("big-{mi:02}"),
        });
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![ti],
            result: ti,
        }));
        let c = m.intern_const(MirConst::I32(mi as i32));
        let p = m.intern_type(MirType::I32);
        let _ = p;
        let mut blocks = Vec::with_capacity(per_fn_blocks as usize);
        for i in 0..per_fn_blocks {
            blocks.push(MirBlock {
                instrs: vec![MirInstr::Const { dst: 1, value: c }],
                term: if i + 1 == per_fn_blocks {
                    Terminator::Return { value: 1 }
                } else {
                    Terminator::Jump { target: i + 1 }
                },
            });
        }
        m.funcs.push(MirFunction {
            owner: m.owner.clone(),
            name: format!("d_bulk_{mi:02}"),
            sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Param,
                    ty: ti,
                    name: Some("x".into()),
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: ti,
                    name: Some("r".into()),
                },
            ],
            blocks,
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
        });
        m.entry = 0;
        let bytes = auto_av_core::encode_abc(&m).expect("encode");
        for pg in &bytes.pages {
            assert!(pg.len() <= auto_av_core::contract::MAX_CODEC_PAGE_BYTES);
        }
        total_abc += bytes.total_len;
        let key = auto_av_probe::cache::abc_unit_key(
            &format!("big-{mi:02}"),
            &format!("d_bulk_{mi:02}"),
            &format!("bulk-{mi:02}"),
            &auto_av_probe::cache::caps_fingerprint(&[]),
        );
        let id = auto_av_probe::cache::put_enveloped(
            &store,
            &key,
            "A09",
            &auto_av_probe::cache::EnvelopeIdentity::abc(&format!("big-{mi:02}")),
            &bytes.to_flat(),
            vec![],
            Some("bulk"),
        )
        .expect("put");
        artifact_ids.push((format!("big-{mi:02}"), id));
    }
    assert!(
        total_abc >= 32 * 1024 * 1024,
        "cold ABC product {} < 32 MiB",
        total_abc
    );

    // Random read-only: pick units at (deterministic) pseudo-random offsets,
    // load by artifact id, decode + verify.
    let mut seed = 0x798_5eedu64;
    for k in 0..64 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let (owner_name, art) = &artifact_ids[(seed as usize >> 33) % artifact_ids.len()];
        let payload = auto_av_probe::cache::get_by_artifact_id(
            &store,
            art,
            &auto_av_probe::cache::EnvelopeIdentity::abc(owner_name),
        )
        .expect("random read");
        let decoded = auto_av_core::decode_abc(&payload).expect("decode");
        let owner_scope = OwnerScope {
            schema: auto_av_core::contract::ABC_SCHEMA.to_string(),
            module: owner_name.clone(),
        };
        auto_av_core::verify_abc(decoded, &owner_scope).expect("re-verify");
        let _ = k;
    }

    // 512 local revisions on a working module — REAL per-round edits
    // (review F798-R1-05): round `rev` rewrites `return acc + (rev-1)` to
    // `return acc + rev` (round 0 seeds `return acc + 0`), so every round
    // the input hash changes, the edited unit's chain re-keys, and the
    // independent expected (10 + rev) is asserted against the actual value.
    let work_root = std::env::temp_dir().join(format!("av798-res-churn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work_root);
    let mut d = Driver::with_store(&work_root, "churn").unwrap();
    let mut src = String::from(
        "fn work(v int) int {
    var acc = v
    for acc > 10 {
        acc = acc - 1
    }
    return acc + 0
}

fn main() int {
    var r = work(37)
    return r
}
",
    );
    // Warm-up on the seeded source so every subsequent round misses ONLY
    // the edited unit's chain.
    let warm = d.request(&src, "d_main", &[]);
    assert!(warm.ok(), "warm-up: {:?}", warm.diag);
    assert_eq!(warm.value_i32(), Some(10));
    let mut last_hash = String::new();
    let mut edited_rounds = 0usize;
    for rev in 0usize..512 {
        let (from, to) = (
            format!("return acc + {}", rev),
            format!("return acc + {}", rev + 1),
        );
        let next = src.replace(&from, &to);
        assert!(
            next != src,
            "round {rev}: edit target `{}` absent — churn must actually change the input",
            from
        );
        src = next;
        let r = d.request(&src, "d_main", &[]);
        assert!(r.ok(), "rev {rev}: {:?}", r.diag);
        assert_eq!(
            r.value_i32(),
            Some(10 + rev as i32 + 1),
            "rev {rev}: value must match the independent expected (work(37)=10 + {})",
            rev + 1
        );
        assert!(
            r.source_hash != last_hash,
            "rev {rev}: input hash must change each round"
        );
        last_hash = r.source_hash.clone();
        // The edited unit's chain re-keys (one real lowering call, one MIR
        // persist + one ABC emit for the changed unit); the untouched main
        // unit stays a hit.
        assert_eq!(
            *r.counters.get("cache_miss_abc").unwrap_or(&0),
            1,
            "rev {rev}"
        );
        assert_eq!(
            *r.counters.get("cache_hit_abc").unwrap_or(&0),
            1,
            "rev {rev}"
        );
        assert_eq!(
            *r.counters.get("mir_lower_modules").unwrap_or(&0),
            1,
            "rev {rev}"
        );
        assert_eq!(
            *r.counters.get("mir_units_persisted").unwrap_or(&0),
            1,
            "rev {rev}"
        );
        assert_eq!(
            *r.counters.get("abc_units_emitted").unwrap_or(&0),
            1,
            "rev {rev}"
        );
        // Managed partitions (§3 r2, F798-R2-02): staging ≤16 MiB, hot
        // closure ≤8 MiB, AV live slots ≤8 MiB — per round, on the counters
        // finalized AFTER the closure codec roundtrip.
        assert!(
            *r.counters.get("peak_staging_bytes").unwrap_or(&0) <= 16 * 1024 * 1024,
            "rev {rev}: staging over budget"
        );
        assert!(
            *r.counters.get("peak_hot_bytes").unwrap_or(&0) <= 8 * 1024 * 1024,
            "rev {rev}: hot partition over budget"
        );
        assert!(
            *r.counters.get("av_peak_slot_bytes").unwrap_or(&0) <= 8 * 1024 * 1024,
            "rev {rev}: exec slot partition over budget"
        );
        assert!(
            *r.counters.get("peak_staging_bytes").unwrap_or(&0)
                >= *r.counters.get("peak_hot_bytes").unwrap_or(&0),
            "rev {rev}: staging peak must include the closure codec phase"
        );
        edited_rounds += 1;
        if rev % 64 == 0 {
            // GC on the SAME store that carries the cold product.
            let cfg = ac_artifact_probe::store::gc::gc_config_default();
            let _ = ac_artifact_probe::store::gc::run_gc(&store, cfg).expect("gc");
        }
    }
    assert_eq!(edited_rounds, 512, "all 512 rounds ran with real edits");

    // Water-level convergence on the PRESSURE store (cold product + churn
    // variants): unroot the churn project, GC, and require reclamation of
    // the accumulated churn garbage (old unit variants) — the live bulk
    // product stays pinned by its roots.
    let space_before = store.space();
    let churn_garbage = space_before
        .total
        .saturating_sub(total_abc as u64 + 8 * 1024 * 1024);
    store.remove_project_roots("churn").expect("unroot churn");
    let cfg = ac_artifact_probe::store::gc::gc_config_default();
    let receipt = ac_artifact_probe::store::gc::run_gc(&store, cfg).expect("final gc");
    let space_after = store.space();
    assert!(
        space_after.total <= space_before.total,
        "GC must not grow the store: {} -> {}",
        space_before.total,
        space_after.total
    );
    if churn_garbage > 1024 * 1024 {
        assert!(
            space_before.total - space_after.total > 0 || receipt.deleted_chunks > 0,
            "churn garbage ({:.1} MiB) must be reclaimed (receipt deleted: {})",
            churn_garbage as f64 / 1048576.0,
            receipt.deleted_chunks
        );
    }
    // The sentinel bulk artifacts survive the churn GC (roots + no live
    // overage): random re-read of the first unit still loads + verifies.
    let (owner_name, art) = &artifact_ids[0];
    let payload = auto_av_probe::cache::get_by_artifact_id(
        &store,
        art,
        &auto_av_probe::cache::EnvelopeIdentity::abc(owner_name),
    )
    .expect("sentinel survives churn GC");
    assert!(auto_av_core::decode_abc(&payload).is_ok());
    // Bounded managed footprint after convergence (783 policy inherits).
    assert!(
        space_after.total < 512 * 1024 * 1024,
        "managed footprint bounded: {}",
        space_after.total
    );
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&work_root);
}

/// F798-R2-02 counter-control: the peak sampler must observe a transient
/// memory spike that the endpoint delta cannot see (endpoint low, process
/// peak high). A ~96 MiB spike is allocated mid-scope and freed before the
/// end; the sampler's peak must catch it while mem_end - mem_start stays
/// low. This is the discriminating oracle for peak-vs-delta evidence.
#[test]
#[cfg(windows)]
fn av_resources_peak_sampler_distinguishes_transient_spike() {
    use auto_av_probe::mempeek::{sample_private_bytes, PeakSampler};
    let m0 = sample_private_bytes();
    let sampler = PeakSampler::start(5);
    let spike: Vec<Vec<u8>> = (0..96).map(|_| vec![7u8; 1024 * 1024]).collect();
    // Hold the spike long enough for at least one poll.
    std::thread::sleep(std::time::Duration::from_millis(120));
    let _ = spike.len();
    drop(spike);
    let peak = sampler.stop();
    let delta = sample_private_bytes().saturating_sub(m0);
    assert!(
        peak >= 64 * 1024 * 1024,
        "sampler must catch the transient spike (peak {} bytes)",
        peak
    );
    assert!(
        delta < 32 * 1024 * 1024,
        "endpoint delta must stay low ({delta} bytes) while the peak was {peak}"
    );
}

/// r5 §5.9 (F798-R12-04 closure): the worker memory budget is TWO
/// attributed HARD-gated lines — Line A the ac-core-owned frontend
/// transient (isolated peak ≤224 MiB), Line B the probe-owned residual
/// (overall minus Line A ≤128 MiB; missing attribution conservatively
/// charges the whole peak). Neither line is advisory.
#[test]
fn av_turnaround_two_line_memory_gate() {
    use auto_av_probe::bench::{validate_and_gate, BenchSummary};
    // Empty rows + 0 samples: every scenario count and edit-hash check
    // passes, so the failure list isolates the memory lines.
    let rows = [];
    let mk = |frontend: Option<u64>, overall: Option<u64>| BenchSummary {
        frontend_peak_bytes: frontend,
        private_bytes_peak: overall,
        ..BenchSummary::default()
    };
    // Both lines within budget: no failures.
    let ok = mk(Some(210 * 1024 * 1024), Some(216 * 1024 * 1024));
    assert!(
        validate_and_gate(&rows, &ok, 0).is_empty(),
        "both lines within budget"
    );
    // Line A breach: the frontend line fires, the residual stays quiet.
    let a_over = mk(
        Some(auto_av_probe::bench::BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES + 1),
        Some(auto_av_probe::bench::BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES + 5 * 1024 * 1024),
    );
    let failures = validate_and_gate(&rows, &a_over, 0);
    assert!(
        failures
            .iter()
            .any(|f| f.contains("frontend transient line")),
        "Line A breach reported: {failures:?}"
    );
    assert!(
        !failures.iter().any(|f| f.contains("probe-owned residual")),
        "residual within its own line: {failures:?}"
    );
    // Line B breach: frontend within Line A, overall exceeds frontend by
    // more than 128 MiB.
    let b_over = mk(
        Some(150 * 1024 * 1024),
        Some(150 * 1024 * 1024 + auto_av_probe::bench::BUDGET_WORKER_PRIVATE_PEAK_BYTES + 1),
    );
    let failures = validate_and_gate(&rows, &b_over, 0);
    assert!(
        failures.iter().any(|f| f.contains("probe-owned residual")),
        "Line B breach reported: {failures:?}"
    );
    assert!(
        !failures
            .iter()
            .any(|f| f.contains("frontend transient line")),
        "frontend within its own line: {failures:?}"
    );
    // Missing attribution: the WHOLE peak is charged to Line B
    // (conservative — no decomposition, no waiver).
    let no_attr = mk(None, Some(200 * 1024 * 1024));
    let failures = validate_and_gate(&rows, &no_attr, 0);
    assert!(
        failures.iter().any(|f| f.contains("probe-owned residual")),
        "unattributed peak charged to the probe line: {failures:?}"
    );
}

/// V-11 smoke: the six-scenario bench runs end-to-end on a scaled load
/// (3 samples) with correct results, real hit/miss counters and diagnosed
/// signature conflicts. The 30-sample release-grade run with the P95 hard
/// gate is driven by scripts/verify-ac-av-798.ps1 -Scope turnaround.
#[test]
fn av_turnaround_six_scenarios_smoke() {
    let root = std::env::temp_dir().join(format!("av798-turn-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    // Scaled: the debug smoke shrinks the generator's target directly
    // frozen full-size run belongs to the release script).
    let small = auto_av_probe::load256::generate_with_sized(0, 24 * 1024);
    let conflicted = auto_av_probe::load256::edit_signature_conflict(&small.source);
    let bench = auto_av_probe::bench::BenchInputs {
        load: small,
        conflict: conflicted,
        samples: 3,
        gate_ms: f64::INFINITY,
        self_exe: std::path::PathBuf::from(env!("CARGO_BIN_EXE_av_probe_worker")),
        revision_target: 24 * 1024,
    };
    let (rows, summary) = auto_av_probe::bench::run_six_scenarios(&bench);
    let scenarios: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.scenario).collect();
    assert_eq!(scenarios.len(), 6, "all six scenarios ran: {:?}", scenarios);
    for row in &rows {
        if row.scenario == "signature-conflict" {
            assert!(row.ok, "conflict diagnosed: {:?}", row.diag);
            assert!(row.diag.is_some());
        } else {
            assert!(
                row.ok,
                "scenario {} sample {} ok (value {:?}, expected {:?})",
                row.scenario, row.sample, row.value, row.expected
            );
        }
    }
    // The body-edit scenario: every sample is a DISTINCT revision with its
    // own input hash, and per sample only the edited unit's chain re-keys
    // (at most one whole-module lower + one ABC emission).
    let edit_rows: Vec<_> = rows
        .iter()
        .filter(|r| r.scenario == "single-body-edit")
        .collect();
    let distinct_hashes = {
        let mut hs: Vec<&str> = edit_rows.iter().map(|r| r.source_hash.as_str()).collect();
        hs.sort();
        hs.dedup();
        hs.len()
    };
    assert_eq!(
        distinct_hashes,
        edit_rows.len(),
        "each edit sample edits its own input"
    );
    assert!(
        edit_rows
            .iter()
            .all(|r| r.mir_lower_modules <= 1 && r.abc_units_emitted <= 1),
        "per-sample edit re-keys at most the edited chain: {:?}",
        edit_rows
            .iter()
            .map(|r| (r.mir_lower_modules, r.abc_units_emitted))
            .collect::<Vec<_>>()
    );
    // Row/sample validation must pass with a non-binding gate (INF): the
    // release-grade gate runs in the worker with the frozen 1000 ms bound.
    let mut s = summary.clone();
    s.gate_ms = f64::INFINITY;
    let failures = auto_av_probe::bench::validate_and_gate(&rows, &s, 3);
    assert!(failures.is_empty(), "validation failures: {failures:?}");
    let _ = std::fs::remove_dir_all(&root);
}
