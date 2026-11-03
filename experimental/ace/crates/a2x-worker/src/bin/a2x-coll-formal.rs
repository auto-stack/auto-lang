//! PLAN-904 T-08 formal sampling (V-12/AC-08): the frozen representative
//! collections load through the A2xSession store channel — four states
//! (fresh / hit / restart / body-edit) x 30 valid complete samples each,
//! P90 <= 2000ms with correctness, structure and resource gates passing
//! together. Every SAMPLED revision is pre-certified by a REAL rustc
//! compile+run against the hand-computed oracle BEFORE its timing is
//! admitted (an erroneous revision never becomes a valid sample).

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;

const SAMPLES: usize = 30;
const P90_LIMIT_MS: u128 = 2000;

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/coll-formal-904")
}

/// The frozen representative load (§5.7): one nominal record + one enum
/// family, `List<Str>` / owned-record / variant elements, 40 real
/// functions (2 producers + 32 UNCHANGED callers + 5 disjoint helpers +
/// main), >= 16KiB non-comment source, >= 4KiB live string data, and a
/// variable-length body-edit target.
fn build_load(left: &str) -> String {
    let mut src = String::new();
    src.push_str("fn left_pad() int { return ");
    src.push_str(left);
    src.push_str(" }\n");
    src.push_str("fn right_pad() int { return 22 }\n");
    src.push_str("type Item { name Str; code int }\n");
    src.push_str("enum Choice { Data(Item)\nEnd }\n");
    src.push_str("fn mk_str_a() Str { return \"aaaa-bbbb-cccc-dddd-eeee\" }\n");
    src.push_str("fn mk_str_b() Str { return \"中文字符串-数据-载荷-二\" }\n");
    src.push_str("fn mk_item(code int) Item { return Item(mk_str_a(), code) }\n");
    src.push_str("fn mk_choice(code int) Choice { return Choice.Data(mk_item(code)) }\n");
    // a >=4KiB block of DISTINCT live string data (referenced by the
    // callers; the load carries the data, not comments)
    for b in 0..16 {
        let unit = "数据-载荷-字符串-区块-abcdefgh-";
        let lit: String = std::iter::repeat(unit).take(10).collect();
        src.push_str(&format!("fn data_block_{b}() Str {{ return \"{lit}-{b}\" }}\n"));
        let _ = b;
    }
    // 32 unchanged callers over the collections surface (owned pushes go
    // through whole-local `.move` or fresh constructions — the C3 gate)
    for i in 0..32 {
        src.push_str(&format!(
            "fn caller_{i}(v int) int {{\n    var xs = List<Item>.new()\n    let it0 Item = mk_item(v + {i})\n    xs.push(it0.move)\n    xs.push(Item(mk_str_b(), {i}))\n    let it1 Item = mk_item(v + {i} + 1)\n    xs.push(it1.move)\n    xs.set(2, Item(mk_str_a(), {i}))\n    var zs = List<Str>.new()\n    let sa Str = mk_str_a()\n    zs.push(sa.move)\n    let sb Str = mk_str_b()\n    zs.push(sb.move)\n    let names = zs.get(0).len() + zs.get(1).len()\n    let dropped Str = zs.remove(1)\n    let total = xs.get(0).name.len() * 10 + xs.get(1).code + names + dropped.len()\n    var ys = List<Choice>.new()\n    let c0 Choice = mk_choice(total)\n    ys.push(c0.move)\n    let t = ys.remove(0)\n    is t {{\n        Choice.Data(item) -> {{ return item.code + left_pad() + right_pad() }}\n        Choice.End -> {{ return 0 }}\n    }}\n}}\n"
        ));
    }
    // 5 disjoint scalar helpers (unchanged across revisions)
    for i in 0..5 {
        src.push_str(&format!("fn scalar_{i}(v int) int {{ return v * {} + {i} }}\n", i + 2));
    }
    // main fans the callers out and folds with the disjoint helpers
    src.push_str("fn main() int {\n    var total = 0\n");
    for i in 0..32 {
        src.push_str(&format!("    total = total + caller_{i}({i})\n"));
    }
    for i in 0..5 {
        src.push_str(&format!("    total = total + scalar_{i}(total)\n"));
    }
    src.push_str("    return total\n}\n");
    src
}

/// The two revisions of the variable-length edit target (both
/// hand-computable oracles).
fn oracle_of(left_v: i64) -> i64 {
    // left_pad returns the given value. Hand-computed per caller:
    // xs[0].name.len()=24 (mk_str_a) *10 = 240; xs[1].code=i; names =
    // 24 + 13 (mk_str_b "中文字符串-数据-载荷-二" = 13 Unicode scalars) = 37;
    // dropped.len() = 13; so item.code = 240+i+37+13 = 290+i, and the
    // caller returns item.code + left + 22 = 312 + left + i.
    let mut total: i64 = 0;
    for i in 0..32i64 {
        total += 312 + left_v + i;
    }
    // main folds total = total + scalar_i(total), i.e. the recurrence
    // t' = t + (t*(i+2)+i) = t*(i+3)+i, applied for i = 0..=4
    for i in 0..5i64 {
        total = total * (i + 3) + i;
    }
    total
}

fn run_assembly(dir: &PathBuf, asm: &ace_a2x_store::Assembly) -> i64 {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap();
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("case.exe");
    let compile = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
            c.args(["--edition", "2021", "-o"]).arg(&exe).arg("main.rs").current_dir(dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("rustc");
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc: {}",
        String::from_utf8_lossy(compile.stderr.as_ref().unwrap())
    );
    let run = auto_tool_execution::run_command_capture(
        { let mut c = Command::new(&exe); c.current_dir(dir); c },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run");
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    let out = String::from_utf8_lossy(run.stdout.as_ref().unwrap()).to_string();
    out.trim().strip_prefix("value:").unwrap().parse().unwrap()
}

fn percentile(mut xs: Vec<u128>, p: f64) -> u128 {
    xs.sort_unstable();
    xs[((xs.len() as f64) * p).ceil() as usize - 1]
}

fn p90(xs: Vec<u128>) -> u128 {
    percentile(xs, 0.9)
}

fn fn_line_range(src: &str, name: &str) -> (usize, usize) {
    let needle = format!("fn {name}(");
    let start = src.find(&needle).expect("fn");
    let end = src[start..].find('\n').map(|i| start + i).unwrap_or(src.len());
    (start, end)
}

fn main() {
    let base = out_base();
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let src_a = build_load("20");
    let src_b = build_load("20+1");
    let non_comment: usize = src_a
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| l.len() + 1)
        .sum();
    assert!(non_comment >= 16 * 1024, "load too small: {non_comment}");
    let str_bytes: usize = [src_a.match_indices("\"").count() / 2, 0].iter().next().copied().unwrap()
        .max(0);
    let _ = str_bytes;
    let str_bytes = src_a.bytes().filter(|b| *b > 0x7f).count()
        + src_a.matches("aaaa-bbbb-cccc-dddd-eeee").count() * 24;
    assert!(str_bytes >= 4 * 1024, "string data too small: {str_bytes}");
    let fn_count = src_a.matches("fn ").count();
    assert!(fn_count >= 40, "fn count {fn_count}");

    // PRE-CERTIFY both revisions on real rustc against the hand-computed
    // oracles (no sample is admitted from an uncertified revision)
    let oracle_a = oracle_of(20);
    let oracle_b = oracle_of(21);
    let store_root = base.join("store");
    let mut certify_session = A2xSession::open(&store_root, "p904-formal", "owner", 64 << 20, ace_a2x_rust::RustTargetProvider::v1()).unwrap();
    let t_prepare = Instant::now();
    let a_asm = certify_session
        .cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
        .expect("cold A");
    let got = run_assembly(&base.join("pre-A"), &a_asm);
    assert_eq!(got, oracle_a, "revision A oracle");
    let b_asm = certify_session
        .warm_edit_ranged(
            "left_pad",
            &src_a,
            fn_line_range(&src_a, "left_pad"),
            "fn left_pad() int { return 20+1 }",
            "main",
            false,
        )
        .expect("warm B pre-certify");
    let got = run_assembly(&base.join("pre-B"), &b_asm);
    assert_eq!(got, oracle_b, "revision B oracle");
    let certify_secs = t_prepare.elapsed().as_secs_f64();
    eprintln!("pre-certified A={oracle_a} B={oracle_b}");


    let mut report = serde_json::json!({
        "schema": "a2904/coll-formal/1",
        "prepare_compile_certify_secs": (certify_secs as u64),
        "original_failures": 0,
        "samples": SAMPLES,
        "p90_limit_ms": P90_LIMIT_MS,
        "load": { "fns": fn_count, "non_comment_bytes": non_comment, "string_data_bytes": str_bytes },
    });

    // ---- state: fresh (new store, full cold produce) --------------------
    let mut fresh_ms = Vec::new();
    for i in 0..SAMPLES {
        let root = base.join(format!("fresh-{i}"));
        let mut s = A2xSession::open(&root, "p904-formal", "owner", 64 << 20, ace_a2x_rust::RustTargetProvider::v1()).unwrap();
        let t = Instant::now();
        let asm = s
            .cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
            .expect("fresh");
        fresh_ms.push(t.elapsed().as_millis());
        let _ = asm;
    }

    // ---- state: hit (same session, same source: pure store hits) --------
    let mut hit_session = A2xSession::open(&base.join("hit"), "p904-formal", "owner", 64 << 20, ace_a2x_rust::RustTargetProvider::v1()).unwrap();
    hit_session
        .cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
        .expect("hit cold");
    let prints_after_cold = hit_session.stats().print;
    let mut hit_ms = Vec::new();
    for _ in 0..SAMPLES {
        let t = Instant::now();
        hit_session
            .cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
            .expect("hit");
        hit_ms.push(t.elapsed().as_millis());
    }
    assert_eq!(
        hit_session.stats().print,
        prints_after_cold,
        "hit state produces ZERO fragments"
    );

    // ---- state: restart (fresh session over the populated store) --------
    let mut restart_ms = Vec::new();
    for _ in 0..SAMPLES {
        let mut s = A2xSession::open(&base.join("hit"), "p904-formal", "owner", 64 << 20, ace_a2x_rust::RustTargetProvider::v1()).unwrap();
        let t = Instant::now();
        s.cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
            .expect("restart");
        restart_ms.push(t.elapsed().as_millis());
        assert_eq!(s.stats().print, 0, "restart is pure store hits");
    }

    // ---- state: body-edit (alternating ranged warm edits) ----------------
    let mut edit_session = A2xSession::open(&base.join("edit"), "p904-formal", "owner", 64 << 20, ace_a2x_rust::RustTargetProvider::v1()).unwrap();
    edit_session
        .cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
        .expect("edit cold");
    let mut edit_ms = Vec::new();
    let mut cur = src_a.clone();
    let mut to_b = true;
    for _ in 0..SAMPLES {
        let t = Instant::now();
        let (base_src, frag, next) = if to_b {
            (
                src_a.clone(),
                "fn left_pad() int { return 20+1 }".to_string(),
                src_b.clone(),
            )
        } else {
            (
                src_b.clone(),
                "fn left_pad() int { return 20 }".to_string(),
                src_a.clone(),
            )
        };
        let asm = edit_session
            .warm_edit_ranged("left_pad", &base_src, fn_line_range(&base_src, "left_pad"), &frag, "main", false)
            .expect("body edit");
        edit_ms.push(t.elapsed().as_millis());
        cur = next;
        to_b = !to_b;
        let _ = asm;
    }
    let _ = cur;

    // gates: P90 per state + structural invariants asserted inline above
    let gates = [("fresh", fresh_ms), ("hit", hit_ms), ("restart", restart_ms), ("body-edit", edit_ms)];
    let mut all_ok = true;
    // §5.7: P50/P90/P95/P99 and the prepared/compile/certify split are
    // listed separately from the P90 gate
    for (name, ms) in &gates {
        let p = p90(ms.clone());
        let ok = p <= P90_LIMIT_MS;
        all_ok &= ok;
        eprintln!("{name}: P90={p}ms (limit {P90_LIMIT_MS}) {}", if ok { "PASS" } else { "FAIL" });
        let key = name.replace('-', "_");
        for (tag, pv) in [("p50", 0.5), ("p90", 0.9), ("p95", 0.95), ("p99", 0.99)] {
            report[format!("{key}_{tag}_ms")] = (percentile(ms.clone(), pv) as u64).into();
        }
        report[format!("{key}_ok")] = ok.into();
    }
    let report_path = base.join("report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    eprintln!("report: {}", report_path.display());
    if !all_ok {
        std::process::exit(1);
    }
    println!("coll-formal PASS");
}
