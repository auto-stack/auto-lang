//! PLAN-881 T-05 performance gate (`av_method_881_perf`): the frozen
//! development-period budget — every fresh/hit/restart/body-edit wall
//! clock on the method sessions stays under the P90<=2000ms gate. The
//! numbers are ASSERTED (the gate is real) and printed for the plan's
//! evidence record.

use ace_av::data_session::DataIncrementalSession;
use ace_av_core::contract::Value;
use std::time::Instant;

const GATE_MS: f64 = 2000.0;

const MIXED: &str = "\
type Counter {
    value int
    static fn new(n int) Counter { return Counter(n) }
    fn read() int { return .value }
    fn bump() int { return .value + 1 }
}
fn free_read(c Counter) int { return c.value }
fn main() int {
    let x Counter = Counter.new(42)
    return x.read()
}
";

#[test]
fn method_perf_gate_2000ms() {
    let mut samples: Vec<(String, f64)> = Vec::new();

    // fresh (cold warmup: whole frontend + every unit + link + run)
    let t = Instant::now();
    let mut sess = DataIncrementalSession::warmup(MIXED, "d_main", &[]).expect("warmup");
    samples.push(("fresh".into(), t.elapsed().as_secs_f64() * 1000.0));

    // body-edit hit (single-method channel) — several rounds
    let mut src = MIXED.to_string();
    for i in 0..5 {
        let next = if i % 2 == 0 {
            src.replace(
                "fn read() int { return .value }",
                "fn read() int { return .value + 1 }",
            )
        } else {
            src.replace(
                "fn read() int { return .value + 1 }",
                "fn read() int { return .value }",
            )
        };
        let t = Instant::now();
        sess.edit_body(&next, "m.Counter.read").expect("edit");
        samples.push((format!("body-edit-{i}"), t.elapsed().as_secs_f64() * 1000.0));
        src = next;
    }
    // one interface round (type shape change) — the wider channel
    let grown = MIXED.replace(
        "type Counter {\n    value int",
        "type Counter {\n    value int\n    extra int",
    );
    let grown = grown.replace(
        "static fn new(n int) Counter { return Counter(n) }",
        "static fn new(n int) Counter { return Counter { value: n, extra: 0 } }",
    );
    let t = Instant::now();
    sess.apply_source(&grown).expect("interface apply");
    samples.push(("interface-edit".into(), t.elapsed().as_secs_f64() * 1000.0));

    // restart (a FRESH session over the current source — full re-decode)
    let t = Instant::now();
    let mut s2 = DataIncrementalSession::warmup(&grown, "d_main", &[]).expect("restart warmup");
    samples.push(("restart".into(), t.elapsed().as_secs_f64() * 1000.0));
    assert_eq!(s2.run().expect("restart run"), Value::I32(42));

    // The gate: every sample under the frozen budget; report the P90.
    let mut walls: Vec<f64> = samples.iter().map(|(_, w)| *w).collect();
    walls.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p90 = walls[(walls.len() as f64 * 0.9).ceil() as usize - 1];
    for (name, w) in &samples {
        assert!(
            *w < GATE_MS,
            "{name} took {w:.0}ms — above the {}ms gate",
            GATE_MS
        );
    }
    eprintln!(
        "method-881 perf: samples {:?} p90={p90:.0}ms gate={GATE_MS}ms",
        samples
    );
    assert!(p90 < GATE_MS, "P90 {p90:.0}ms above the gate");
}
