// Plan 768 T-01 whole-path feasibility probe.
//
// Scratch evidence collector (deleted after T-01): proves the parse → raw-bytes →
// real-adapter path against
//   - autodown-core (external checkout, read-only path dep; vendoring candidate)
//   - auto-lang worktree crate with --no-default-features (no GUI)
//
// Sections: [FP] source fingerprints, [FENCE] raw fence locator vs block tree,
// [AVM] run_with_capture_and_path observations, [A2R] emit observations.

use std::collections::HashMap;
use std::path::Path;

use autodown_core::block_model::{attrGet, blockText, BlockNode, BlockType, Value};
use autodown_core::markdown_parser::parse_blocks;

const WT: &str = "D:/autostack/.wt/lang-768/auto-lang/crates/auto-lang";
const EXT: &str = "D:/autostack/auto-down/autodown/packages/engine/rust";

#[derive(Default)]
struct Counter {
    pass: usize,
    fail: usize,
}

impl Counter {
    fn check(&mut self, name: &str, cond: bool, detail: &str) {
        if cond {
            self.pass += 1;
            println!("[PASS] {} — {}", name, detail);
        } else {
            self.fail += 1;
            println!("[FAIL] {} — {}", name, detail);
        }
    }
    fn info(&self, name: &str, detail: &str) {
        println!("[INFO] {} — {}", name, detail);
    }
}

// ---------------------------------------------------------------- sha256 --

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bitlen = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ------------------------------------------------------------- fingerprints --

fn source_fingerprints(c: &mut Counter) {
    println!("== [FP] vendoring-closure source fingerprints ==");
    let expect: &[(&str, &str)] = &[
        (
            "markdown_parser.rs",
            "4c41853517d09f4740ed32f3a6645f3fedd8bd7ad69ac0fd3ece16f21427ffc9",
        ),
        (
            "block_model.rs",
            "8e99920ff0f078706ea7d8e8b2f351e3fe90c5220d57c791d6ca0e1000b04c36",
        ),
    ];
    for (name, want) in expect {
        let p = format!("{}/src/{}", EXT, name);
        match std::fs::read(&p) {
            Ok(bytes) => {
                let got = hex(&sha256(&bytes));
                c.check(
                    &format!("fp:{}", name),
                    got == *want,
                    &format!("sha256={}", got),
                );
            }
            Err(e) => c.check(&format!("fp:{}", name), false, &format!("read failed: {}", e)),
        }
    }
    let ial = format!("{}/src/ial.rs", EXT);
    match std::fs::read(&ial) {
        Ok(bytes) => c.info("fp:ial.rs", &format!("sha256={} (not pre-recorded in plan)", hex(&sha256(&bytes)))),
        Err(e) => c.info("fp:ial.rs", &format!("read failed: {}", e)),
    }
}

// ------------------------------------------------------------- fence probe --

struct RawFence {
    info: String,
    payload_start: usize,
    payload_end: usize,
    closed: bool,
}

/// Raw fence locator prototype: byte-range scan of the ORIGINAL document bytes,
/// replicating the tree parser's fence open/close rules (indent < 4; trimmed
/// opener line starts with a 3+ run of ` or ~; closer = trimmed line is exactly
/// the same-length marker run; anything else inside is payload).
fn locate_fences(bytes: &[u8]) -> Vec<RawFence> {
    // Line table: (content_start, content_end_excl_eol) — EOL is the bytes
    // between content_end and the next line's content_start.
    let mut lines: Vec<(usize, usize)> = Vec::new();
    let mut start = 0usize;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            let mut e = i;
            if e > start && bytes[e - 1] == b'\r' {
                e -= 1;
            }
            lines.push((start, e));
            start = i + 1;
        }
    }
    if start < bytes.len() {
        lines.push((start, bytes.len()));
    }

    let mut out: Vec<RawFence> = Vec::new();
    let mut li = 0usize;
    while li < lines.len() {
        let (s, e) = lines[li];
        let lossy = String::from_utf8_lossy(&bytes[s..e]).to_string();
        let indent = lossy.chars().take_while(|ch| *ch == ' ').count();
        let t = lossy.trim();
        let marker = match t.chars().next() {
            Some('`') if t.starts_with("```") => '`',
            Some('~') if t.starts_with("~~~") => '~',
            _ => '\0',
        };
        if marker == '\0' || indent >= 4 {
            li += 1;
            continue;
        }
        let run: String = t.chars().take_while(|ch| *ch == marker).collect();
        let info = t
            .chars()
            .skip(run.chars().count())
            .collect::<String>()
            .trim()
            .to_string();
        let mut j = li + 1;
        let mut closed = false;
        while j < lines.len() {
            let (cs, ce) = lines[j];
            let cline = String::from_utf8_lossy(&bytes[cs..ce]).to_string();
            let cindent = cline.chars().take_while(|ch| *ch == ' ').count();
            let ct = cline.trim();
            if cindent < 4
                && ct.chars().count() == run.chars().count()
                && ct.chars().all(|ch| ch == marker)
            {
                closed = true;
                break;
            }
            j += 1;
        }
        let payload_start = if li + 1 < lines.len() { lines[li + 1].0 } else { bytes.len() };
        let payload_end = if closed { lines[j].0 } else { bytes.len() };
        out.push(RawFence { info, payload_start, payload_end, closed });
        if closed {
            li = j + 1;
        } else {
            break;
        }
    }
    out
}

fn collect_tree(b: &BlockNode, fences: &mut Vec<(String, String)>, heads: &mut Vec<(i64, String, Option<String>)>) {
    match b.kind {
        BlockType::Fence => {
            let lang = match attrGet(b.attrs.clone(), "language") {
                Some(Value::Str(s)) => s,
                _ => String::new(),
            };
            fences.push((lang, blockText(b.clone())));
        }
        BlockType::Heading => {
            let level = match attrGet(b.attrs.clone(), "level") {
                Some(Value::Int(i)) => i,
                _ => 0,
            };
            let anchor = match attrGet(b.attrs.clone(), "anchor") {
                Some(Value::Str(s)) => Some(s),
                _ => None,
            };
            heads.push((level, blockText(b.clone()), anchor));
        }
        _ => {}
    }
    for child in &b.children {
        collect_tree(child, fences, heads);
    }
}

fn normalize_eols(s: &[u8]) -> String {
    let a: String = String::from_utf8_lossy(s).split("\r\n").map(|x| x.to_string()).collect::<Vec<_>>().join("\n");
    a.split('\r').map(|x| x.to_string()).collect::<Vec<_>>().join("\n")
}

fn fence_probe(c: &mut Counter) {
    println!("\n== [FENCE] raw locator vs pinned block tree ==");
    let doc: &str = &concat!(
        "# Title\r\n",
        "\r\n",
        "Intro with unicode \u{1F600} and CJK \u{4E2D}\u{6587}.\r\n",
        "\r\n",
        "## Payloads ^auto-src\r\n",
        "\r\n",
        "````auto\r\n",
        "fn main() {\r\n",
        "    print(\"nested ``` inside\")\n",
        "}\r\n",
        "````\r\n",
        "\r\n",
        "```test-case\r\n",
        "case { id: \"probe.1\" }\r\n",
        "```\n",
        "\r\n",
        "```\n",
        "```\n",
        "\r\n",
        "```  spaced-info  \n",
        "payload with trailing spaces   \n",
        "```   \n",
        "\r\n",
        "    ```not-a-fence\n",
        "\r\n",
        "not a fence: ``` inline\r\n",
    );
    let bytes = doc.as_bytes();

    let tree = parse_blocks(doc, true);
    let mut fences: Vec<(String, String)> = Vec::new();
    let mut heads: Vec<(i64, String, Option<String>)> = Vec::new();
    collect_tree(&tree, &mut fences, &mut heads);

    let raw = locate_fences(bytes);
    c.check("fence:count", fences.len() == 4 && raw.len() == 4,
        &format!("tree={} raw={} (expect 4/4)", fences.len(), raw.len()));

    let want_lang = ["auto", "test-case", "", "spaced-info"];
    for i in 0..raw.len().min(fences.len()) {
        c.check(&format!("fence:{}:lang", i), fences[i].0 == raw[i].info && raw[i].info == want_lang[i],
            &format!("tree='{}' raw='{}'", fences[i].0, raw[i].info));
    }

    let want_payload: [&[u8]; 4] = [
        b"fn main() {\r\n    print(\"nested ``` inside\")\n}\r\n",
        b"case { id: \"probe.1\" }\r\n",
        b"",
        b"payload with trailing spaces   \n",
    ];
    for (i, w) in want_payload.iter().enumerate() {
        if i < raw.len() {
            let got = &bytes[raw[i].payload_start..raw[i].payload_end];
            c.check(&format!("fence:{}:payload-bytes", i), got == *w,
                &format!("len={} sha256={}", got.len(), hex(&sha256(got))));
        }
    }

    // Tree <-> raw-range correspondence: tree fence text must equal the
    // EOL-normalized raw payload for every closed fence.
    for i in 0..raw.len().min(fences.len()) {
        if raw[i].closed {
            let norm = normalize_eols(&bytes[raw[i].payload_start..raw[i].payload_end]);
            c.check(&format!("fence:{}:tree-matches-raw", i), norm == fences[i].1,
                &format!("tree_len={} norm_len={}", fences[i].1.len(), norm.len()));
        }
    }

    c.check("fence:headings", heads.len() == 2,
        &format!("heads={:?}", heads.iter().map(|h| (h.0, h.1.clone(), h.2.clone())).collect::<Vec<_>>()));
    if heads.len() == 2 {
        c.check("fence:anchor", heads[1].2.as_deref() == Some("auto-src"),
            &format!("level={} text='{}' anchor={:?}", heads[1].0, heads[1].1, heads[1].2));
    }

    // Unclosed fence: locator reports closed=false, payload runs to EOF; tree
    // exists with loading=false (isFinal=true). Divergence in trailing bytes is
    // expected for unclosed fences — the protocol rejects them as ERROR anyway.
    let doc2 = "```rust\nlet x = 1\n";
    let tree2 = parse_blocks(doc2, true);
    let mut f2: Vec<(String, String)> = Vec::new();
    let mut h2: Vec<(i64, String, Option<String>)> = Vec::new();
    collect_tree(&tree2, &mut f2, &mut h2);
    let raw2 = locate_fences(doc2.as_bytes());
    let unclosed_payload = &doc2.as_bytes()[raw2[0].payload_start..raw2[0].payload_end];
    c.info("fence:unclosed",
        &format!("raw closed={} payload={:?} tree_lang='{}' tree_text={:?}",
            raw2[0].closed, String::from_utf8_lossy(unclosed_payload), f2[0].0, f2[0].1));
}

// --------------------------------------------------------------- AVM probe --

use auto_lang::error::AutoError;
use auto_lang::run_with_capture_and_path;

fn stage_of(e: &AutoError) -> &'static str {
    match e {
        AutoError::Lexer(_) | AutoError::LexerWithSource(_) | AutoError::Syntax(_) | AutoError::SyntaxWithSource(_) => "parse",
        AutoError::Type(_) | AutoError::Name(_) | AutoError::Comptime(_) => "type",
        AutoError::Runtime(_) => "run(structured)",
        AutoError::Msg(m) => {
            if m.starts_with("auto_gate_") { "gate" } else { "run(msg)" }
        }
        AutoError::MsgWithSource(_) => "link",
        AutoError::Io(_) => "io",
        AutoError::Warning(_) => "warning",
        AutoError::MultipleErrors { errors, .. } => match errors.first() {
            Some(e) => stage_of(e),
            None => "multiple-empty",
        },
    }
}

fn run_avm(c: &mut Counter, label: &str, code: &str, path: &str) -> Option<(String, String)> {
    match std::panic::catch_unwind(|| run_with_capture_and_path(code, path)) {
        Ok(Ok((result, stdout))) => {
            c.info(label, &format!("Ok result={:?} stdout={:?}", result, stdout));
            Some((result, stdout))
        }
        Ok(Err(e)) => {
            c.info(label, &format!("Err stage={} display={}", stage_of(&e), e));
            None
        }
        Err(_) => {
            c.info(label, "PANIC (escaped through join())");
            None
        }
    }
}

fn avm_probe(c: &mut Counter) {
    println!("\n== [AVM] run_with_capture_and_path observations ==");

    // 1. Real script fixture: result oracle identical to legacy runner.
    let fx = format!("{}/test/vm/01_basics/002_arithmetic", WT);
    let src = std::fs::read_to_string(format!("{}/arithmetic.at", fx)).unwrap();
    let expected_result = std::fs::read_to_string(format!("{}/arithmetic.expected.result", fx)).unwrap();
    if let Some((result, _)) = run_avm(c, "avm:fixture-arithmetic", &src, &format!("{}/arithmetic.at", fx)) {
        c.check("avm:fixture-arithmetic:result", result == expected_result,
            &format!("got={:?} want={:?}", result, expected_result));
    }

    // 2. Program entry: does fn main's return value surface as result?
    let prog = "fn main() int { return 2 + 3 * 4 }";
    if let Some((result, _)) = run_avm(c, "avm:program-main", prog, "main.at") {
        c.check("avm:program-main:result-14", result == "14", &format!("result={:?}", result));
    }

    // 3. Parse error classification.
    run_avm(c, "avm:parse-error", "fn (", "bad.at");

    // 4. Type error classification (Auto type syntax: `var x int = ...`).
    run_avm(c, "avm:type-error", "var x int = \"s\"", "t.at");

    // 4b. Typed-context mismatch: fn return type vs actual value.
    run_avm(c, "avm:type-error-fn", "fn f() int { return \"s\" }\nprint(f())", "t2.at");

    // 4b2. Ill-typed operand / undefined name classification.
    run_avm(c, "avm:ill-typed-add", "print(1 + \"s\")", "t3.at");
    run_avm(c, "avm:undefined-name", "print(undefined_var)", "t4.at");

    // 4c. Real weak-oracle fixture: out-of-bounds assignment trap message.
    let fx_oob = format!("{}/test/vm/15_nested_mutation/014_out_of_bounds_error", WT);
    let src_oob = std::fs::read_to_string(format!("{}/out_of_bounds_error.at", fx_oob)).unwrap();
    run_avm(c, "avm:out-of-bounds(weak)", &src_oob, &format!("{}/out_of_bounds_error.at", fx_oob));

    // 5. Runtime trap classification.
    let rt = "var a = 6\nvar b = 0\nprint(a / b)";
    if let Some(_) = run_avm(c, "avm:runtime-div0", rt, "r.at") {
        c.check("avm:runtime-div0", false, "expected Err but got Ok");
    }

    // 6. .as lowering through the path-carrying API (real fixture; python-free).
    let fx_as = format!("{}/test/vm/99_script_err/01_propagate", WT);
    let src_as = std::fs::read_to_string(format!("{}/propagate.as", fx_as)).unwrap();
    let expected_out = std::fs::read_to_string(format!("{}/propagate.expected.out", fx_as)).unwrap();
    if let Some((_, stdout)) = run_avm(c, "avm:as-propagate", &src_as, &format!("{}/propagate.as", fx_as)) {
        c.check("avm:as-propagate:stdout", stdout == expected_out,
            &format!("got={:?} want={:?}", stdout, expected_out));
    }

    // 6b. .as + weak oracle (expected.error only — legacy-any-error shape).
    let fx_unc = format!("{}/test/vm/99_script_err/03_uncaught", WT);
    let src_unc = std::fs::read_to_string(format!("{}/uncaught.as", fx_unc)).unwrap();
    run_avm(c, "avm:as-uncaught(weak)", &src_unc, &format!("{}/uncaught.as", fx_unc));

    // 6c. catch_value needs `python` feature; recorded, not a pilot candidate.
    let fx_cv = format!("{}/test/vm/99_script_err/02_catch_value", WT);
    let src_cv = std::fs::read_to_string(format!("{}/catch_value.as", fx_cv)).unwrap();
    run_avm(c, "avm:as-catch-value(python-gated)", &src_cv, &format!("{}/catch_value.as", fx_cv));

    // 6d. Replicate the LEGACY path for the same fixture: pre-lower via
    // auto_s2s, then run path-less — records whether use.py survives lowering.
    let lowered = auto_lang::trans::auto_s2s::lower_source(&src_as);
    match lowered {
        Ok(low) => {
            let head: Vec<&str> = low.lines().take(6).collect();
            c.info("avm:as-propagate:lowered-head", &format!("{:?}", head));
            match std::panic::catch_unwind(|| auto_lang::run_with_capture(&low)) {
                Ok(Ok((result, stdout))) => {
                    c.check("avm:as-propagate-legacy:stdout", stdout == expected_out,
                        &format!("result={:?} stdout={:?}", result, stdout));
                }
                Ok(Err(e)) => c.info("avm:as-propagate-legacy", &format!("Err stage={} display={}", stage_of(&e), e)),
                Err(_) => c.info("avm:as-propagate-legacy", "PANIC"),
            }
        }
        Err(e) => c.info("avm:as-propagate:lower-failed", &format!("{} {}", stage_of(&e), e)),
    }

    // 7. `use auto.*` module fixture: legacy wrapper is #[ignore]d — record
    // actual behavior through the production path API (expected: module not found).
    let fx_use = format!("{}/test/vm/17_modules/001_use_fn", WT);
    let src_use = std::fs::read_to_string(format!("{}/use_fn.at", fx_use)).unwrap();
    let expected_use = std::fs::read_to_string(format!("{}/use_fn.expected.out", fx_use)).unwrap();
    if let Some((_, stdout)) = run_avm(c, "avm:use-fn", &src_use, &format!("{}/use_fn.at", fx_use)) {
        c.check("avm:use-fn:stdout", stdout == expected_use,
            &format!("got={:?} want={:?}", stdout, expected_use));
    }
}

// --------------------------------------------------------------- a2r probe --

use auto_lang::trans::rust::{transpile_rust_project, transpile_rust_with_source_dir};

fn a2r_probe(c: &mut Counter) {
    println!("\n== [A2R] emit observations ==");

    // 1. Basic fixture: byte-identical to legacy golden.
    let fx = format!("{}/test/a2r/01_basics/001_hello", WT);
    let src = std::fs::read_to_string(format!("{}/hello.at", fx)).unwrap();
    let expected = std::fs::read(format!("{}/hello.expected.rs", fx)).unwrap();
    let name = "hello".to_string();
    let mut sink = transpile_rust_with_source_dir(Path::new(&fx), &name, &src).expect("emit hello");
    let got: Vec<u8> = sink.done().expect("sink done").clone();
    c.check("a2r:hello:bytes", got == expected,
        &format!("len={} sha256={}", got.len(), hex(&sha256(&got))));

    // 2. c-abi static manifest fixture: emit-safe, byte-identical golden.
    let fxc = format!("{}/test/a2r/27_c_abi/003_use_c_static", WT);
    let srcc = std::fs::read_to_string(format!("{}/use_c_static.at", fxc)).unwrap();
    let expectedc = std::fs::read(format!("{}/use_c_static.expected.rs", fxc)).unwrap();
    let namec = "use_c_static".to_string();
    let mut sinkc = transpile_rust_with_source_dir(Path::new(&fxc), &namec, &srcc).expect("emit use_c_static");
    let gotc: Vec<u8> = sinkc.done().expect("sink done").clone();
    c.check("a2r:use-c-static:bytes", gotc == expectedc,
        &format!("len={} sha256={}", gotc.len(), hex(&sha256(&gotc))));

    // 2b. Same source but manifest NOT resolvable (empty source_dir): record
    // whether emit silently changes output instead of erroring.
    let empty_dir = std::env::temp_dir().join("probe768-empty-srcdir");
    std::fs::create_dir_all(&empty_dir).unwrap();
    let mut sinkc2 = transpile_rust_with_source_dir(&empty_dir, &namec, &srcc).expect("emit use_c_static (no manifest)");
    let gotc2: Vec<u8> = sinkc2.done().expect("sink done").clone();
    c.info("a2r:use-c-static:no-manifest",
        &format!("len={} (with manifest {}) differs={} — silent-ignore caveat",
            gotc2.len(), gotc.len(), gotc2 != gotc));

    // 3. Multi-module project: filename-keyed multi-artifact map.
    let fxp = format!("{}/test/a2r/14_modules/005_multi_file", WT);
    let entry = format!("{}/main.at", fxp);
    match transpile_rust_project(&entry) {
        Ok(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            c.check("a2r:project:keys",
                keys.iter().any(|k| k.as_str() == "main.rs")
                    && keys.iter().any(|k| k.as_str() == "db.rs")
                    && keys.iter().any(|k| k.as_str() == "api/mod.rs")
                    && keys.iter().any(|k| k.as_str() == "api/handlers.rs")
                    && keys.iter().any(|k| k.as_str() == "Cargo.toml"),
                &format!("keys={:?}", keys));
            for key in ["main.rs", "db.rs", "api/mod.rs", "api/handlers.rs"] {
                let stem = key.trim_end_matches(".rs");
                let exp_path = format!("{}/{}.expected.rs", fxp, stem);
                let expected_p = std::fs::read(&exp_path).unwrap();
                let got_p = &map[key].0;
                c.check(&format!("a2r:project:{}:bytes", key), got_p == &expected_p,
                    &format!("len={} sha256={}", got_p.len(), hex(&sha256(got_p))));
            }
            if let Some((cargo, _)) = map.get("Cargo.toml") {
                c.info("a2r:project:cargo-toml", &format!("len={} head={:?}", cargo.len(),
                    String::from_utf8_lossy(&cargo[..cargo.len().min(120)])));
            }
        }
        Err(e) => c.check("a2r:project", false, &format!("project emit failed: {}", e)),
    }
}

fn main() {
    // a2r transpile paths can be stack-hungry in debug builds; run the whole
    // probe on a large dedicated stack (mirrors auto-lang's own sync wrappers).
    let handle = std::thread::Builder::new()
        .stack_size(128 * 1024 * 1024)
        .spawn(run_all)
        .expect("spawn probe thread");
    match handle.join() {
        Ok(code) => std::process::exit(code),
        Err(_) => {
            eprintln!("probe thread panicked");
            std::process::exit(3);
        }
    }
}

fn run_all() -> i32 {
    let mut c = Counter::default();
    println!("== probe-t01 (Plan 768 T-01 whole-path feasibility) ==");
    source_fingerprints(&mut c);
    fence_probe(&mut c);
    avm_probe(&mut c);
    a2r_probe(&mut c);
    println!("\n== summary: {} pass, {} fail ==", c.pass, c.fail);
    if c.fail == 0 { 0 } else { 1 }
}
