//! PLAN-915 R02/R03 scenario fixtures: the per-allocation-point fault sweep
//! over the frozen corpus programs and the typed stale-Place sequence.
//!
//! These are the crate-side controls for the formal scenario recipe
//! (`ac-av/resource-run-v1`): they drive the SAME `resource_915` fixtures the
//! worker drives, so a break in the fixture or in the executor seams fails
//! here first.

use ace_av::resource_915::{self, PlaceMutation, SourceFrontend};

const L04: &str = r#"type Item {
    name Str
    code int
}

fn main() int {
    var xs = List<Item>.new()
    let x Item = Item("hello", 37)
    xs.push(x.move)
    return xs.get(0).name.len() + xs.get(0).code
}
"#;

const K02: &str = r#"fn make(move s Str) fn(int) int {
    return move (n int) => {
        return s.len() + n
    }
}

fn main() int {
    let s Str = "hello"
    let f = make(s.move)
    return f(37)
}
"#;

/// R02: every arena attempt of the real L04 program faults exactly once,
/// leaks nothing, reports its own ordinal, and the N+1 control still
/// produces the frozen value.
#[test]
fn resource_915_r02_l04_arena_sweep_is_complete() {
    let facts = resource_915::source_fault_sweep(L04, "main", SourceFrontend::Collections)
        .expect("L04 sweep runs");
    assert_eq!(facts["source_value"], 42, "{facts:#}");
    // FROZEN (T-02): the L04 program makes exactly two owned-arena attempts
    // (the "hello" buffer + the list buffer) and six value allocations.
    assert_eq!(facts["arena_attempts"], 2, "{facts:#}");
    assert_eq!(facts["value_attempts"], 6, "{facts:#}");
    let points = facts["arena_points"].as_array().expect("points");
    assert_eq!(
        points.len() as u64,
        facts["arena_attempts"].as_u64().unwrap()
    );
    for (i, p) in points.iter().enumerate() {
        assert_eq!(p["attempt"], (i + 1) as u64, "{facts:#}");
        assert_eq!(p["faulted"], true, "arena point {} must fault: {p}", i + 1);
        assert_eq!(p["live"], 0, "arena point {} leaked: {p}", i + 1);
        let msg = p["message"].as_str().unwrap_or_default();
        assert!(
            msg.contains("fault-injected at owned allocation attempt #"),
            "arena point {} message: {msg}",
            i + 1
        );
    }
    assert_eq!(facts["arena_control"]["value"], 42, "{facts:#}");
    assert_eq!(facts["after"]["value"], 42, "{facts:#}");
    assert_eq!(facts["after"]["live"], 0, "{facts:#}");
    assert_eq!(facts["after_recompile"]["value"], 42, "{facts:#}");
    assert_eq!(facts["after_recompile"]["live"], 0, "{facts:#}");
}

/// R02: the value-allocation seam (slot writes incl. list growth) sweeps the
/// same program point by point with zero live owned bytes each time.
#[test]
fn resource_915_r02_l04_value_sweep_is_complete() {
    let facts = resource_915::source_fault_sweep(L04, "main", SourceFrontend::Collections)
        .expect("L04 sweep runs");
    let attempts = facts["value_attempts"].as_u64().unwrap_or(0);
    assert_eq!(attempts, 6, "{facts:#}");
    let points = facts["value_points"].as_array().expect("points");
    assert_eq!(points.len() as u64, attempts);
    for (i, p) in points.iter().enumerate() {
        assert_eq!(p["faulted"], true, "value point {} must fault: {p}", i + 1);
        assert_eq!(p["live"], 0, "value point {} leaked: {p}", i + 1);
    }
    assert_eq!(facts["value_control"]["value"], 42, "{facts:#}");
}

/// R02: the closure-capture program (K02 domain) sweeps through the real
/// closures frontend as well — the same recipe covers the capture clone /
/// env materialization path instead of assuming it.
#[test]
fn resource_915_r02_k02_closure_sweep_is_complete() {
    let arena = resource_915::source_fault_sweep(K02, "main", SourceFrontend::Closures)
        .expect("K02 sweep runs");
    assert_eq!(arena["source_value"], 42, "{arena:#}");
    // FROZEN (T-02): K02 makes one owned-arena attempt and five value
    // allocations through the closures frontend.
    assert_eq!(arena["arena_attempts"], 1, "{arena:#}");
    assert_eq!(arena["value_attempts"], 5, "{arena:#}");
    let points = arena["arena_points"].as_array().expect("points");
    assert!(!points.is_empty(), "{arena:#}");
    for p in points {
        assert_eq!(p["faulted"], true, "{p}");
        assert_eq!(p["live"], 0, "{p}");
    }
    assert_eq!(arena["arena_control"]["value"], 42, "{arena:#}");
}

/// R03: each structural mutation retires the old token — the use traps
/// `av.trap.stale-place` with NO buffer access counted for that use, while
/// the legal pre-mutation read and the fresh-token control did read.
#[test]
fn resource_915_r03_stale_place_sequence() {
    for mutation in [
        PlaceMutation::Push,
        PlaceMutation::Insert,
        PlaceMutation::Remove,
        PlaceMutation::Replace,
    ] {
        let seq = resource_915::place_sequence(mutation)
            .unwrap_or_else(|e| panic!("{}: {e}", mutation.as_str()));
        let stale = &seq["stale"];
        assert_eq!(
            stale["outcome"],
            "trap",
            "{} must trap: {seq:#}",
            mutation.as_str()
        );
        assert_eq!(
            stale["trap"],
            "av.trap.stale-place",
            "{}: {seq:#}",
            mutation.as_str()
        );
        // mints=1, the legal read happened (get_ok=1, buffer_reads=1), then
        // the stale use was rejected without touching the buffer.
        assert_eq!(stale["facts"]["mints"], 1, "{seq:#}");
        assert_eq!(stale["facts"]["get_ok"], 1, "{seq:#}");
        assert_eq!(stale["facts"]["get_stale"], 1, "{seq:#}");
        assert_eq!(
            stale["facts"]["buffer_reads"], 1,
            "the stale use must not read the buffer: {seq:#}"
        );
        assert_eq!(stale["live"], 0, "{seq:#}");

        let fresh = &seq["fresh"];
        assert_eq!(fresh["outcome"], "success", "{seq:#}");
        assert_eq!(fresh["facts"]["get_ok"], 1, "{seq:#}");
        assert_eq!(fresh["facts"]["buffer_reads"], 1, "{seq:#}");

        let legal = &seq["legal"];
        assert_eq!(legal["outcome"], "success", "{seq:#}");
        assert_eq!(legal["value"], 10, "{seq:#}");
        assert_eq!(legal["facts"]["get_stale"], 0, "{seq:#}");
    }
}

/// The gated counter really is gated: without `observe_places` the trace
/// carries no place facts at all.
#[test]
fn resource_915_place_facts_are_opt_in() {
    let module = resource_915::place_module(Some(PlaceMutation::Push));
    let plain = resource_915::run_place_module(&module, false).expect("plain run");
    assert_eq!(plain["outcome"], "trap");
    assert!(plain["facts"].is_null(), "{plain:#}");
    let observed = resource_915::run_place_module(&module, true).expect("observed run");
    assert_eq!(observed["facts"]["buffer_reads"], 1, "{observed:#}");
}

const L05: &str = r#"type Item { name Str; code int }
fn main() int {
    var xs = List<Item>.new()
    xs.push(Item("hello", 37))
    let x Item = xs.remove(0)
    return x.name.len() + x.code + xs.len()
}
"#;

const LSTR: &str = r#"fn main() int {
    var xs = List<Str>.new()
    xs.push("hello")
    xs.push("中")
    let removed Str = xs.remove(0)
    return removed.len() * 10 + xs.get(0).len()
}
"#;

const V05: &str = r#"type Item { name Str; code int }
enum Token {
    Data(Item)
    End
}
fn main() int {
    let t = Token.Data(Item("hello", 37))
    is t {
        Token.Data(v) -> { return v.name.len() + v.code }
        Token.End -> { return 1 }
    }
}
"#;

/// R02: the remaining frozen corpus programs (L05 move-out of an owned
/// value, V05 owned variant payload) sweep the same way with their own
/// frozen attempt counts.
#[test]
fn resource_915_r02_l05_v05_sweeps() {
    for (name, src, arena_want, value_want) in [("l05", L05, 2u64, 5u64), ("v05", V05, 1u64, 6u64)]
    {
        let facts = resource_915::source_fault_sweep(src, "main", SourceFrontend::Collections)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(facts["source_value"], 42, "{name}: {facts:#}");
        assert_eq!(facts["arena_attempts"], arena_want, "{name}: {facts:#}");
        assert_eq!(facts["value_attempts"], value_want, "{name}: {facts:#}");
        for point in facts["arena_points"].as_array().unwrap() {
            assert_eq!(point["faulted"], true, "{name} arena: {point}");
            assert_eq!(point["live"], 0, "{name} arena: {point}");
        }
        for point in facts["value_points"].as_array().unwrap() {
            assert_eq!(point["faulted"], true, "{name} value: {point}");
            assert_eq!(point["live"], 0, "{name} value: {point}");
        }
        assert_eq!(facts["arena_control"]["value"], 42, "{name}: {facts:#}");
        assert_eq!(facts["value_control"]["value"], 42, "{name}: {facts:#}");
        assert_eq!(facts["after"]["value"], 42, "{name}: {facts:#}");
        assert_eq!(facts["after"]["live"], 0, "{name}: {facts:#}");
    }
}

/// R02 typed control: every capture-clone attempt fails as the typed
/// `env.capture.faulted-clone`, the source stays readable and owned, and the
/// post-fault control clones successfully.
#[test]
fn resource_915_r02_capture_clone_sweep() {
    let facts = resource_915::capture_clone_sweep().expect("capture sweep");
    assert_eq!(facts["clean"]["outcome"], "success", "{facts:#}");
    assert_eq!(facts["clean"]["live"], 2, "{facts:#}");
    let points = facts["points"].as_array().expect("points");
    assert_eq!(points.len(), 2, "{facts:#}");
    for (i, point) in points.iter().enumerate() {
        assert_eq!(point["outcome"], "faulted-clone", "{facts:#}");
        assert_eq!(point["fault_attempt"], (i as u64) + 3, "{facts:#}");
        assert_eq!(point["code"], "env.capture.faulted-clone", "{facts:#}");
        assert_eq!(point["live"], 2, "the source must stay owned: {facts:#}");
        assert_eq!(point["source_readable"], true, "{facts:#}");
    }
    assert_eq!(facts["control"]["outcome"], "success", "{facts:#}");
    assert_eq!(facts["control"]["live"], 2, "{facts:#}");
}
