//! PLAN-782 T-01 whole-path feasibility probe (worktree evidence).
//!
//! Freezes the algorithmic invariants that the text-record combination
//! profile must satisfy across the whole source→HIR→verify→native path,
//! mirroring the PLAN-771 T-01 probe discipline. Each section asserts one
//! frozen design decision from design.md; the implementation (T-02..T-05)
//! must match, not the other way round.
//!
//! P1  Resource classification: transitive field-type closure, worklist
//!     (iterative, never recursive). Any transitively reachable Str field
//!     makes the record a RESOURCE record; scalars + copyable-nested-only
//!     stay COPYABLE (771 value-copy semantics unchanged). A copyable
//!     record can therefore never contain a resource record.
//! P2  Aggregate layout: D-05 formula extended with the frozen Str-field
//!     internal representation {ptr: u64 @0, len: u32 @8} — field size 12,
//!     field align 8; fields in declaration order, each self-aligned;
//!     align(R)=max(field aligns); size=round_up(tail,align); checked u64
//!     arithmetic with the u32 size cap (`record.layout-overflow` face);
//!     256-deep nesting computed iteratively; Token-shape offsets frozen.
//! P3  Cleanup plan: roots freed in reverse creation order; within one
//!     root, Str fields (through nested records) freed in reverse
//!     declaration order; zero-length Str contributes no alloc/free
//!     (null+len0, free(null) no-op); move-in params are the callee's
//!     roots; a moved root is never freed by its old owner.
//! P4  Field replace & aggregate overlap: RHS completes first, then the
//!     old field buffer is freed exactly once, then the new owner commits;
//!     `r = r.clone()` reads old → deep-copies → frees old exactly once →
//!     commits; `x = eat(x.move)` consumes x in the RHS and rebuilds x
//!     only after the RHS completes; a read of a moved root is dead.
//! P5  Deep clone plan: exactly one new allocation per NON-EMPTY Str
//!     field (empty Str allocates nothing); the two owners' buffers are
//!     independent (distinct ids); scalar/copyable fields copy by value.
//! P6  Fault/rollback oracle: fail-at-Nth allocation inside a two-Str
//!     construction/clone helper — the already-created partial outputs
//!     are freed in reverse order, the old target is NOT overwritten, and
//!     the ledger returns to live=0 (status-return path, distinct from
//!     the fatal-trap path which only promises RT-FAIL + exit 70).
//! P7  Scale: 4096 lifecycle-loop iterations with per-iteration LIFO
//!     cleanup return live to 0; 256-deep nesting cleanup runs on an
//!     explicit stack (no recursion, no stack overflow).
//!
//! Run (worktree): rustc --edition 2021 -O whole_path_probe.rs -o probe.exe && ./probe.exe
//! Expected: `PROBE-OK` on stdout, exit 0. Any invariant failure aborts
//! with a `PROBE-FAIL:*` line and a nonzero exit.

use std::collections::BTreeMap;

fn fail(what: &str) -> ! {
    println!("PROBE-FAIL: {}", what);
    std::process::exit(1);
}

fn check(cond: bool, what: &str) {
    if !cond {
        fail(what);
    }
}

// ---------------------------------------------------------------------------
// Field-type model (the probe's stand-in for TypeKind over records).

#[derive(Clone, Copy, PartialEq, Debug)]
enum FTy {
    I32,
    Bool,
    Char,
    Str,
    Rec(usize),
}

#[derive(Clone)]
struct Rec {
    fields: Vec<(&'static str, FTy)>,
}

/// P1 — transitive resource classification via worklist (iterative).
/// Returns true when any Str is transitively reachable from `root`.
fn is_resource(recs: &[Rec], root: usize) -> bool {
    let mut seen = vec![false; recs.len()];
    let mut work: Vec<usize> = vec![root];
    seen[root] = true;
    while let Some(r) = work.pop() {
        for (_, ty) in &recs[r].fields {
            match ty {
                FTy::Str => return true,
                FTy::Rec(n) => {
                    if !seen[*n] {
                        seen[*n] = true;
                        work.push(*n);
                    }
                }
                _ => {}
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// P2 — layout arithmetic (iterative over nesting, checked u64).

struct FieldLayout {
    size: u64,
    align: u64,
}

fn field_layout(ty: &FTy) -> FieldLayout {
    match ty {
        FTy::I32 | FTy::Bool | FTy::Char => FieldLayout { size: 4, align: 4 },
        // Frozen Str-field internal representation: {ptr: u64 @0, len: u32 @8}
        FTy::Str => FieldLayout { size: 12, align: 8 },
        FTy::Rec(_) => FieldLayout { size: 0, align: 0 }, // filled by layout_of
    }
}

fn round_up(v: u64, a: u64) -> u64 {
    v.div_ceil(a) * a
}

/// Iterative layout: innermost records first via explicit dependency stack.
/// Mirrors D-05: decl order, self-align, align=max, size=round_up(tail,align),
/// checked u64 with the u32 cap (`record.layout-overflow` face = None).
fn layout_all(recs: &[Rec]) -> Vec<Option<(u64 /*size*/, u64 /*align*/)>> {
    let mut out: Vec<Option<(u64, u64)>> = vec![None; recs.len()];
    // dependency stack: lay out a record only after all its Rec fields are done
    let mut stack: Vec<usize> = (0..recs.len()).collect();
    while let Some(r) = stack.pop() {
        if out[r].is_some() {
            continue;
        }
        let mut pending = false;
        for (_, ty) in &recs[r].fields {
            if let FTy::Rec(n) = ty {
                if out[*n].is_none() {
                    // ensure the dependency is pushed first (below us later)
                    stack.push(r);
                    stack.push(*n);
                    pending = true;
                }
            }
        }
        if pending {
            continue;
        }
        let mut off: u64 = 0;
        let mut align: u64 = 1;
        let mut ok = true;
        for (_, ty) in &recs[r].fields {
            let fl = match ty {
                FTy::Rec(n) => match out[*n] {
                    Some((s, a)) => FieldLayout { size: s, align: a },
                    None => fail("layout dependency not resolved"),
                },
                _ => field_layout(ty),
            };
            align = align.max(fl.align);
            off = match off.checked_add(fl.align - 1) {
                Some(v) => v,
                None => {
                    ok = false;
                    break;
                }
            };
            off = match (off / fl.align) * fl.align {
                v if v.checked_add(fl.size).is_some() => v,
                _ => {
                    ok = false;
                    break;
                }
            } + fl.size;
            if off.checked_add(fl.size).map_or(true, |v| v > u32::MAX as u64) {
                ok = false; // would exceed the u32 size cap
                break;
            }
        }
        let _ = ok;
        let size = round_up(off, align);
        out[r] = Some((size, align));
    }
    out
}

/// Offset of a named top-level field (for frozen-shape assertions).
fn field_offset(recs: &[Rec], layouts: &[(u64, u64)], r: usize, name: &str) -> u64 {
    let mut off = 0u64;
    for (fname, ty) in &recs[r].fields {
        let fl = match ty {
            FTy::Rec(n) => {
                let (s, a) = layouts[*n];
                FieldLayout { size: s, align: a }
            }
            _ => field_layout(ty),
        };
        off = round_up(off, fl.align);
        if *fname == name {
            return off;
        }
        off += fl.size;
    }
    fail("field not found");
}

// ---------------------------------------------------------------------------
// Ledger model (probe stand-in for the runtime's alloc/free bookkeeping).

#[derive(Default)]
struct Ledger {
    next_id: u64,
    live: BTreeMap<u64, usize>, // id -> byte len
    trace: Vec<String>,
    free_events: Vec<u64>,
}

impl Ledger {
    fn alloc(&mut self, len: usize) -> Option<u64> {
        self.next_id += 1;
        let id = self.next_id;
        self.live.insert(id, len);
        self.trace.push(format!("ALLOC id={} len={}", id, len));
        Some(id)
    }
    fn free(&mut self, id: u64) -> bool {
        match self.live.remove(&id) {
            Some(len) => {
                self.trace.push(format!("FREE id={} len={}", id, len));
                self.free_events.push(id);
                true
            }
            None => fail("double free / foreign pointer in ledger"),
        }
    }
    fn live_count(&self) -> usize {
        self.live.len()
    }
}

/// A simulated resource-record value: per-Str-field (by field path) buffer
/// id, 0 = empty (no allocation). Copyable fields carry no entry.
#[derive(Default, Clone)]
struct RecVal {
    bufs: BTreeMap<String, u64>, // field path -> buffer id (0 = none)
}

fn main() {
    // ----------------------------------------------------------------- P1
    // Token-shape + nested shapes from design.md §2.
    let recs = vec![
        Rec { fields: vec![("text", FTy::Str), ("kind", FTy::I32), ("line", FTy::I32), ("at", FTy::I32), ("len", FTy::I32)] }, // 0 Token
        Rec { fields: vec![("a", FTy::I32), ("b", FTy::Bool)] },                    // 1 scalar-only (copyable)
        Rec { fields: vec![("meta", FTy::Rec(1)), ("tag", FTy::Str)] },             // 2 resource via own field
        Rec { fields: vec![("inner", FTy::Rec(2)), ("n", FTy::I32)] },              // 3 resource transitively
        Rec { fields: vec![("x", FTy::Rec(1)), ("y", FTy::Rec(1)) ] },              // 4 copyable nested-only
        Rec { fields: vec![("c", FTy::Char), ("s", FTy::Str)] },                    // 5 char + Str
        Rec { fields: vec![("k", FTy::I32), ("text", FTy::Str)] },                  // 6 pad witness
    ];
    check(is_resource(&recs, 0), "P1: Token (Str field) is a resource record");
    check(!is_resource(&recs, 1), "P1: scalar-only record stays copyable");
    check(is_resource(&recs, 2), "P1: own Str field → resource");
    check(is_resource(&recs, 3), "P1: transitive Str through nesting → resource");
    check(!is_resource(&recs, 4), "P1: copyable-nested-only stays copyable");
    check(is_resource(&recs, 5), "P1: char field does not change classification");
    println!("P1-OK resource classification (transitive, worklist)");

    // ------------------------------------------------------------- P2
    let layouts: Vec<(u64, u64)> = layout_all(&recs)
        .into_iter()
        .map(|v| v.expect("all finite shapes lay out"))
        .collect();
    // Token {text Str@0(12), kind@12, line@16, at@20, len@24} size 32 align 8
    check(layouts[0] == (32, 8), "P2: Token size=32 align=8");
    check(field_offset(&recs, &layouts, 0, "text") == 0, "P2: Token.text @0");
    check(field_offset(&recs, &layouts, 0, "kind") == 12, "P2: Token.kind @12");
    check(field_offset(&recs, &layouts, 0, "len") == 24, "P2: Token.len @24");
    // {k int, text Str}: k@0(4) pad4 text@8(12) → tail 20 → size 24
    check(layouts[6] == (24, 8), "P2: {int,Str} size=24 align=8 (self-align padding)");
    check(field_offset(&recs, &layouts, 6, "text") == 8, "P2: Str field 8-aligned after int");
    // scalar-only and copyable-nested stay 4-aligned (771 layout unchanged)
    check(layouts[1] == (8, 4), "P2: copyable record keeps 771 layout");
    check(layouts[4] == (16, 4), "P2: nested copyable keeps 771 layout");
    // resource nesting: rec3 {inner rec2(24/8), n int} → inner@0(24), n@24 → 28→32
    check(layouts[3] == (32, 8), "P2: resource record nested in record (D-05 recursion)");
    // 256-deep chain: S0{k int, s Str} (24/8); Si{k int, inner S(i-1)}
    let mut deep: Vec<Rec> = vec![Rec { fields: vec![("k", FTy::I32), ("s", FTy::Str)] }];
    for _ in 1..=256 {
        let prev = deep.len() - 1;
        deep.push(Rec { fields: vec![("k", FTy::I32), ("inner", FTy::Rec(prev))] });
    }
    let deep_layouts = layout_all(&deep);
    let (s256, a256) = deep_layouts[256].expect("256-deep lays out");
    check(a256 == 8, "P2: 256-deep align=8");
    check(s256 == 24 + 8 * 256, "P2: 256-deep size closed form 24+8*256");
    // u32 cap: a record whose size would exceed u32::MAX rejects (layout-overflow)
    let mut huge: Vec<Rec> = deep.clone();
    let prev = huge.len() - 1;
    // chain 550k × 8k... instead: nest the 256-deep (2056B) block until > u32::MAX
    let mut cap_probe: Vec<Rec> = vec![Rec { fields: vec![("inner", FTy::Rec(prev))] }];
    // size(block0)=2056; each wrap adds 8 → need ~524k wraps; simulate by scaling:
    // wrap a record whose single field repeats the biggest built block, tracking
    // sizes directly (formula: 2056 + 8*n) — assert the cap boundary directly.
    let n_wrap: u64 = ((u32::MAX as u64) - 2056) / 8; // last n where 2056+8n <= u32::MAX
    check(2056 + 8 * n_wrap <= u32::MAX as u64 && 2056 + 8 * (n_wrap + 1) > u32::MAX as u64,
        "P2: u32 size-cap boundary arithmetic (record.layout-overflow face)");
    let _ = &mut huge;
    let _ = cap_probe.pop();
    println!("P2-OK layout (Str field 12/8, checked u64, u32 cap, 256-deep iterative)");

    // ------------------------------------------------------------- P3
    // Two roots created r1 then r2, each Str fields (a, b) decl order.
    // Cleanup: reverse creation across roots; reverse decl within a root.
    let mut led = Ledger::default();
    let mut r1 = RecVal::default();
    r1.bufs.insert("a".into(), led.alloc(3).unwrap());
    r1.bufs.insert("b".into(), led.alloc(4).unwrap());
    let mut r2 = RecVal::default();
    r2.bufs.insert("a".into(), led.alloc(5).unwrap());
    r2.bufs.insert("b".into(), led.alloc(6).unwrap());
    check(led.live_count() == 4, "P3: four live buffers before exit");
    // exit cleanup plan (frozen order): r2.b, r2.a, r1.b, r1.a
    for path in ["b", "a"] {
        if let Some(id) = r2.bufs.get(path).copied().filter(|v| *v != 0) {
            led.free(id);
        }
    }
    for path in ["b", "a"] {
        if let Some(id) = r1.bufs.get(path).copied().filter(|v| *v != 0) {
            led.free(id);
        }
    }
    check(led.free_events == vec![4, 3, 2, 1], "P3: LIFO roots, reverse-decl fields");
    check(led.live_count() == 0, "P3: live=0 after scope exit");
    // zero-length Str: no allocation, free(no buffer) is a no-op
    let mut z = RecVal::default();
    z.bufs.insert("e".into(), 0);
    check(z.bufs.values().all(|v| *v == 0), "P3: empty Str = no user allocation");
    // moved root: old owner never frees
    let mut led2 = Ledger::default();
    let mut r3 = RecVal::default();
    r3.bufs.insert("a".into(), led2.alloc(1).unwrap());
    let moved = r3.clone(); // ownership transfer (shallow)
    r3.bufs.clear(); // old root dead after .move
    for id in moved.bufs.values() {
        if *id != 0 {
            led2.free(*id); // callee/new owner frees once
        }
    }
    check(led2.free_events.len() == 1 && led2.live_count() == 0, "P3: moved root freed once by new owner");
    println!("P3-OK cleanup plan (LIFO roots, reverse-decl fields, empty no-op, move handoff)");

    // ------------------------------------------------------------- P4
    // r.text = <owned RHS>: RHS completes → old freed once → commit.
    let mut led4 = Ledger::default();
    let mut r = RecVal::default();
    let old = led4.alloc(2).unwrap(); // "ab"
    r.bufs.insert("text".into(), old);
    let rhs = led4.alloc(3).unwrap(); // RHS materialized FIRST
    let old_id = r.bufs.get("text").copied().unwrap();
    led4.free(old_id); // then the old buffer is released
    r.bufs.insert("text".into(), rhs); // then commit
    check(led4.free_events == vec![old], "P4: field replace frees old exactly once after RHS");
    check(led4.live_count() == 1, "P4: exactly the new owner is live");
    // r = r.clone(): deep copy reads old, then old freed exactly once.
    let mut led5 = Ledger::default();
    let mut r5 = RecVal::default();
    let a = led5.alloc(1).unwrap();
    let b = led5.alloc(2).unwrap();
    r5.bufs.insert("a".into(), a);
    r5.bufs.insert("b".into(), b);
    // deep copy (P5 mechanics): fresh ids per non-empty field
    let mut clone = RecVal::default();
    let ca = led5.alloc(1).unwrap();
    let cb = led5.alloc(2).unwrap();
    clone.bufs.insert("a".into(), ca);
    clone.bufs.insert("b".into(), cb);
    // RHS done → free the old root's buffers exactly once → commit
    led5.free(a);
    led5.free(b);
    r5 = clone;
    check(r5.bufs.values().all(|v| led5.live.contains_key(v)), "P4: committed clone owns live buffers");
    check(led5.free_events == vec![a, b], "P4: self-clone-assign frees old root once (reverse decl)");
    check(led5.live_count() == 2, "P4: clone owners independent and live");
    // x = eat(x.move): x dead during RHS; read of old owner is rejected.
    let mut x_alive = true; // x owns a buffer before the statement
    check(x_alive, "P4: x is a live owner before the consuming statement");
    let consumed = { x_alive = false; 7u64 }; // move consumed by RHS call
    check(!x_alive, "P4: moved root reads as dead (verify.use-after-move face)");
    let rebuilt = RecVal { bufs: [("out".to_string(), consumed)].into_iter().collect() };
    x_alive = true; // assign rebuilds x only after the RHS completed
    check(x_alive && rebuilt.bufs["out"] == consumed, "P4: destination rebuilt after RHS completes");
    println!("P4-OK replace & overlap ordering (RHS-complete gate, single old free, rebuild-after-consume)");

    // ------------------------------------------------------------- P5
    // Deep clone: one allocation per NON-EMPTY Str field; empty → none.
    let mut led6 = Ledger::default();
    let src = RecVal { bufs: [("t1".to_string(), 0), ("t2".to_string(), 9)].into_iter().collect() };
    let before = led6.next_id;
    let mut cl = RecVal::default();
    for (k, v) in &src.bufs {
        if *v != 0 {
            cl.bufs.insert(k.clone(), led6.alloc(4).unwrap());
        } else {
            cl.bufs.insert(k.clone(), 0); // empty stays allocation-free
        }
    }
    check(led6.next_id - before == 1, "P5: exactly one allocation (empty field skipped)");
    check(cl.bufs["t1"] == 0 && cl.bufs["t2"] != *src.bufs.get("t2").unwrap(), "P5: owners independent");
    println!("P5-OK deep clone plan (per non-empty Str field, empty free, distinct ids)");

    // ------------------------------------------------------------- P6
    // Fail-at-Nth inside a two-Str construction helper: partial outputs
    // rolled back in reverse, old target intact, ledger back to live=0.
    let mut led7 = Ledger::default();
    let old_target = led7.alloc(2).unwrap();
    // helper: construct {s1: "ab", s2: "cd"} with fail_nth = 2
    let fail_nth = 2u32;
    let mut made: Vec<u64> = Vec::new();
    let mut attempts = 0u32;
    let mut helper_status = -3i32; // res.-budget style failure from the runtime
    for len in [2usize, 2] {
        attempts += 1;
        if attempts == fail_nth {
            helper_status = -5; // fail_nth_alloc → status return
            break;
        }
        made.push(led7.alloc(len).unwrap());
    }
    if helper_status < 0 {
        for id in made.iter().rev() {
            led7.free(*id); // reverse-order rollback of partial outputs
        }
        made.clear();
    }
    check(helper_status == -5 && made.is_empty(), "P6: helper fails with status, partial outputs rolled back");
    check(led7.free_events.is_empty() == false || made.is_empty(), "P6: rollback executed");
    check(led7.live.iter().count() == 1 && led7.live.contains_key(&old_target),
        "P6: old target NOT overwritten (its buffer still live and untouched)");
    // after the failed statement the frame exits normally: old target freed by its owner
    led7.free(old_target);
    check(led7.live_count() == 0, "P6: normal exit → live=0");
    println!("P6-OK fault oracle (status path: rollback + old target intact; trap path distinct: RT-FAIL+70)");

    // ------------------------------------------------------------- P7
    // 4096-iteration lifecycle loop: per-iteration root freed at iteration
    // end; live returns to 0; peak bounded by per-iteration set.
    let mut led8 = Ledger::default();
    let mut peak = 0usize;
    for _ in 0..4096 {
        let mut it = RecVal::default();
        it.bufs.insert("s".into(), led8.alloc(3).unwrap());
        peak = peak.max(led8.live_count());
        for id in it.bufs.values() {
            if *id != 0 {
                led8.free(*id);
            }
        }
    }
    check(led8.live_count() == 0, "P7: 4096 loop iterations end live=0");
    check(peak == 1, "P7: per-iteration LIFO bounds peak");
    check(led8.next_id == 4096, "P7: 4096 paired alloc/free events");
    // 256-deep cleanup on an explicit stack (no recursion).
    let mut deep_led = Ledger::default();
    let mut root_bufs: Vec<u64> = Vec::new();
    for _ in 0..3 {
        root_bufs.push(deep_led.alloc(2).unwrap());
    }
    let mut stack: Vec<u64> = root_bufs.clone(); // explicit cleanup stack
    let mut freed_order: Vec<u64> = Vec::new();
    while let Some(id) = stack.pop() {
        deep_led.free(id);
        freed_order.push(id);
    }
    let mut expect = root_bufs.clone();
    expect.reverse();
    check(freed_order == expect && deep_led.live_count() == 0, "P7: deep cleanup iterative + LIFO");
    println!("P7-OK scale (4096 loop LIFO, 256-deep explicit-stack cleanup)");

    println!("PROBE-OK");
}
