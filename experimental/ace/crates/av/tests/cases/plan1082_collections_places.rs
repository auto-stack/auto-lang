//! PLAN-1082 T-01 feasibility probes: the collections/places language face
//! through the ORDINARY Auto source profile (`compile_language_source`) —
//! the same front door the `ac-av/language-source-run-v1` recipe runs.
//!
//! Phase 1 (T-01): one OBSERVATION test prints, for each §6 face variant,
//! the current chain outcome (success value / first diagnostic stage+code)
//! under `--nocapture`. The asserted expectations record the T-01 observed
//! baseline; T-02..04 flip them to the §6 contract (42 / named rejections)
//! as the faces land. Observation list (source snippets are §6 verbatim or
//! minimal one-face deltas):
//!   O1 `[19,20]` under `let`  O2 same under `var`
//!   O3 bare read `xs[0]`      O4 plain index write `xs[0] = v`
//!   O5 compound `xs[0] += 1`  O6 chained place `r.xs[1] += 3`
//!   O7 `List.new(19, 23)`     O8 HOF filter/map/reduce
//!   O9 empty `[]` literal

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::AbcBytes;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};

/// Compile and admit one Auto-profile program (the recipe chain up to the
/// executable module).
fn compile_and_load(src: &str) -> ace_av_core::abc_verify::LoadedModule {
    let compiled = ace_core::source::compile_language_source(src, "p1082.at", &[])
        .unwrap_or_else(|d| panic!("Auto-profile compile failed: {d:?}"));
    let mir = ace_lowering::lower_checked_auto_language(&compiled.checked, "d_main", 0x1082)
        .expect("auto-language lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire: AbcBytes = ace_av_core::abc::encode_abc(&mir).expect("AVAD encode");
    abc_verify::verify_abc_bytes(&wire.to_flat(), &mir.owner).expect("AVAD admits")
}

fn run(src: &str) -> (ExecOutcome, ConsoleCapture) {
    let mut capture = ConsoleCapture::default();
    let loaded = compile_and_load(src);
    let (out, _trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut capture,
    );
    (out, capture)
}

fn ok_i32(src: &str) -> i32 {
    match run(src).0 {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected an i32 success, got {other:?}"),
    }
}

/// Human-readable current outcome of one source: either the executed i32
/// or the first compile diagnostic (stage+code).
fn observe(src: &str) -> String {
    match ace_core::source::compile_language_source(src, "p1082.at", &[]) {
        Ok(_) => match run(src).0 {
            ExecOutcome::Success {
                value: Value::I32(v),
                ..
            } => format!("ok:{v}"),
            other => format!("exec:{other:?}"),
        },
        Err(ds) => {
            let d = ds.first().expect("at least one diagnostic");
            format!("diag:{:?}:{}", d.stage, d.code).to_lowercase()
        }
    }
}

/// T-01 observation matrix — prints each face's current outcome and pins
/// the OBSERVED baseline (updated per face as T-02..04 land them).
#[test]
fn t01_observe_face_matrix() {
    let faces: &[(&str, &str)] = &[
        (
            "O1 array-lit let",
            "fn main() int { let xs = [19, 20]; return xs[0] + xs[1] }\n",
        ),
        (
            "O2 array-lit var",
            "fn main() int { var xs = [19, 20]; return xs[0] + xs[1] }\n",
        ),
        (
            "O3 bare read",
            "fn main() int { let xs = [19, 20]; return xs[1] }\n",
        ),
        (
            "O4 plain index write",
            "fn main() int { let xs = [19, 20]\n xs[1] = 22\n return xs[0] + xs[1] }\n",
        ),
        (
            "O5 compound index write",
            "fn main() int { let xs = [19, 20]\n xs[0] += 1\n return xs[0] + xs[1] }\n",
        ),
        (
            "O6 chained field+index place",
            concat!(
                "type Row { xs List<int> }\n",
                "fn main() int {\n",
                " var r = Row([19, 20])\n",
                " r.xs[1] += 3\n",
                " return r.xs[0] + r.xs[1]\n",
                "}\n"
            ),
        ),
        (
            "O7 legacy List.new varargs",
            "fn main() int { let xs = List.new(19, 23); return xs[0] + xs[1] }\n",
        ),
        (
            "O8 HOF filter/map/reduce",
            concat!(
                "fn main() int {\n",
                " let xs = [1, 2, 3, 4]\n",
                " let ys = xs.filter((x int) => x % 2 == 0).map((x int) => x * 3)\n",
                " return ys.reduce(24, (acc int, x int) => acc + x)\n",
                "}\n"
            ),
        ),
        (
            "O9 empty array literal",
            "fn main() int { let xs = []\n xs.push(42)\n return xs[0] }\n",
        ),
        (
            "O10 OOB read xs.get(5)",
            "fn main() int { let xs = [19, 20]; return xs.get(5) }\n",
        ),
        (
            "O11 negative read xs[-1]",
            "fn main() int { let xs = [19, 20]; return xs[-1] }\n",
        ),
        (
            "O12 OOB bare read xs[5]",
            "fn main() int { let xs = [19, 20]; return xs[5] }\n",
        ),
        (
            "O13 OOB write xs[5] = 1",
            "fn main() int { let xs = [19, 20]\n xs[5] = 1\n return xs[0] }\n",
        ),
        (
            "O14 tuple lit + .0",
            "fn main() int { let t = (10, 20, 30)\n return t.0 + t.1 + t.2 }\n",
        ),
        (
            "O15 empty via List<int>.new()",
            "fn main() int { let xs = List<int>.new()\n return xs.len() }\n",
        ),
        (
            "O16 nested index read matrix[0][1]",
            "fn main() int { let m = [[1, 2]]\n return m[0][1] }\n",
        ),
        (
            "O17 index-then-field read arr[0].x",
            "type P { x int }\nfn main() int { let a = [P(1), P(2)]\n return a[0].x }\n",
        ),
        (
            "O18 field+index place write obj.items[0] = 10",
            "fn main() int { let obj = { items: [1, 2, 3] }\n obj.items[0] = 10\n return obj.items[0] }\n",
        ),
        (
            "O19 index+field place write arr[0].x = 10",
            "type P { x int }\nfn main() int { let a = [P(1), P(2)]\n a[0].x = 10\n return a[0].x }\n",
        ),
        (
            "O20 chained index place write m[0][1] = 20",
            "fn main() int { let m = [[1, 2]]\n m[0][1] = 20\n return m[0][1] }\n",
        ),
        (
            "O21 for-in over list",
            "fn main() int { var s = 0\n for x in [1, 2, 3, 4, 5] { s = s + x }\n return s }\n",
        ),
        (
            "O22 record list field ctor arg",
            "type Row { xs List<int> }\nfn main() int { var r = Row([19, 20])\n return r.xs[0] + r.xs[1] }\n",
        ),
        (
            "O23 record method self field push",
            concat!(
                "type Holder {\n items List<int>\n",
                " mut fn add(x int) void { self.items.push(x) }\n",
                " fn count() int { return self.items.len() }\n",
                "}\n",
                "fn main() int { var h = Holder { items: List<int>.new() }\n",
                " h.add(10)\n h.add(20)\n return h.count() }\n"
            ),
        ),
        (
            "O24 obj literal field read obj.items[0]",
            "fn main() int { let obj = { items: [1, 2, 3] }\n return obj.items[0] }\n",
        ),
        (
            "O25 record list field push via method",
            concat!(
                "type Row { xs List<int> }\n",
                "fn main() int {\n",
                " var r = Row(List<int>.new())\n",
                " r.xs.push(19)\n r.xs.push(20)\n",
                " return r.xs[0] + r.xs[1]\n",
                "}\n"
            ),
        ),
        (
            "O26 record list field set via method",
            concat!(
                "type Row { xs List<int> }\n",
                "fn main() int {\n",
                " var r = Row(List<int>.new())\n",
                " r.xs.push(19)\n r.xs.push(20)\n",
                " r.xs.set(1, 22)\n",
                " return r.xs[0] + r.xs[1]\n",
                "}\n"
            ),
        ),
        (
            "O27 tuple literal alone",
            "fn main() int { let t = (10, 20, 30)\n return 1 }\n",
        ),
        (
            "O28 tuple destructure read",
            "fn main() int { let t = (10, 20, 30)\n let (a, b, c) = t\n return a + b + c }\n",
        ),
        (
            "O29 positional tuple access t.0",
            "fn main() int { let t = (10, 20, 30)\n return t.0 + t.1 + t.2 }\n",
        ),
        (
            "O30 compound write value readback",
            "fn main() int { let xs = [19, 20]\n xs[0] += 1\n return xs[0] }\n",
        ),
        (
            "O31 scalar compound a += 1",
            "fn main() int { var a = 19\n a += 1\n return a }\n",
        ),
        (
            "O32 manual compound spelling",
            "fn main() int { let xs = [19, 20]\n xs[0] = xs[0] + 1\n return xs[0] + xs[1] }\n",
        ),
    ];
    for (name, src) in faces {
        println!("PLAN1082-OBS {name} => {}", observe(src));
    }
}

// ---- §6 contract probes (red until T-02..04 land the faces) ----------------

#[test]
fn probe_bracket_read_write() {
    assert_eq!(
        ok_i32(concat!(
            "fn main() int {\n",
            " var xs = [19, 20]\n",
            " xs[0] += 1\n",
            " xs[1] = xs[1] + 2\n",
            " return xs[0] + xs[1]\n",
            "}\n"
        )),
        42,
        "§6 bracket-read-write: bare index read, plain and compound write"
    );
}

#[test]
fn probe_legacy_list_new() {
    assert_eq!(
        ok_i32("fn main() int { let xs = List.new(19, 23); return xs[0] + xs[1] }\n"),
        42,
        "§6 legacy-list-new: old varargs constructor + bare index reads"
    );
}

#[test]
fn probe_hof_real_call() {
    assert_eq!(
        ok_i32(concat!(
            "fn main() int {\n",
            " let xs = [1, 2, 3, 4]\n",
            " let ys = xs.filter((x int) => x % 2 == 0).map((x int) => x * 3)\n",
            " return ys.reduce(24, (acc int, x int) => acc + x)\n",
            "}\n"
        )),
        42,
        "§6 hof-real-call: filter/map/reduce as the real 972 public methods"
    );
}

#[test]
fn probe_nested_place() {
    assert_eq!(
        ok_i32(concat!(
            "type Row { xs List<int> }\n",
            "fn main() int {\n",
            " var r = Row([19, 20])\n",
            " r.xs[1] += 3\n",
            " return r.xs[0] + r.xs[1]\n",
            "}\n"
        )),
        42,
        "§6 nested-place: chained field+index place with compound write-back"
    );
}

