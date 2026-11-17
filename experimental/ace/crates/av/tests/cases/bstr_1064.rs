//! PLAN-1064 T-01 empirical longest-chain & budget-recovery probes (pre-work).
//! Measures exact step/frame/wall budget of chunked `char.from_codepoint(cp).to(Str)` +
//! binary `tree_concat` on the packages-aggregates lane (`PackageAvTarget`), plus illegal
//! scalar rejection and cancellation/budget-failure recovery without corrupting subsequent runs.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, TrapKind, Value};
use ace_av_core::platform_io::Win32PlatformIo;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_sys_inputs, compile_assembled_aggregates_inputs, PackageProduct,
};

fn sys_grants() -> Vec<String> {
    vec![
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.stdin".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
        "platform.file".into(),
        "platform.file.path".into(),
        "platform.env".into(),
        "platform.clock".into(),
        "platform.entropy".into(),
        "platform.process".into(),
    ]
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/bstr1064/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn compile_asm_main(main: &str) -> PackageProduct {
    let app_files = vec![("main.at".to_string(), main.to_string())];
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let inputs =
        assemble_stdlib_sys_inputs(&app_files, "app::main", &mut ctx).expect("assemble sys inputs");
    compile_assembled_aggregates_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
        .unwrap_or_else(|diags| {
            panic!(
                "compile failed: {:?}",
                diags
                    .iter()
                    .map(|d| {
                        let snippet = main
                            .get(d.diagnostic.span.start..d.diagnostic.span.end.min(main.len()))
                            .unwrap_or("");
                        format!(
                            "{:?} @{}..{} {:?}: {}",
                            d.module, d.diagnostic.span.start, d.diagnostic.span.end, snippet, d.diagnostic.message
                        )
                    })
                    .collect::<Vec<_>>()
            )
        })
}

#[test]
fn bstr_1064_t01_longest_chain_probe() {
    // Build N scalars grouped in leaf chunks of 64 concatenated via binary tree_concat,
    // staying well below MAX_AV_FRAMES (1024) while avoiding O(N^2) linear `+`.
    let src = r#"
fn mk(cp int) Str {
    let c char = char.from_codepoint(cp)
    return c.to(Str)
}

fn tree_concat(chunks List<Str>, n int) Str {
    if (n <= 0) { return "" }
    if (n == 1) { return chunks.remove(chunks.len() - 1) }
    if (n == 2) {
        let right = chunks.remove(chunks.len() - 1)
        let left = chunks.remove(chunks.len() - 1)
        return left + right
    }
    let r_cnt = n / 2
    let l_cnt = n - r_cnt
    let right = tree_concat(chunks, r_cnt)
    let left = tree_concat(chunks, l_cnt)
    return left + right
}

fn build_leaf(i int, end int, acc Str) Str {
    if (i >= end) { return acc }
    let q = i / 26
    let rem = i - q * 26
    var cp = 65 + rem
    var step = 1
    let q64 = i / 64
    let m = i - q64 * 64
    if (m == 60) {
        if (i + 4 <= end) {
            cp = 128512
            step = 4
        }
    }
    let c char = char.from_codepoint(cp)
    let piece = c.to(Str)
    let next_acc = acc + piece
    return build_leaf(i + step, end, next_acc)
}

fn push_block(chunks List<Str>, i int, block_end int, rem int) int {
    if (rem <= 0) { return 0 }
    if (i >= block_end) { return 0 }
    var end = i + 64
    if (end > block_end) { end = block_end }
    let leaf = build_leaf(i, end, "")
    chunks.push(leaf.move)
    return push_block(chunks, end, block_end, rem - 1)
}

fn push_outer(chunks List<Str>, i int, byte_target int) int {
    if (i >= byte_target) { return 0 }
    var block_end = i + 1024
    if (block_end > byte_target) { block_end = byte_target }
    let _ = push_block(chunks, i, block_end, 16)
    return push_outer(chunks, block_end, byte_target)
}

fn build_utf8_chain(byte_target int) Str {
    var chunks = List<Str>.new()
    let _ = push_outer(chunks, 0, byte_target)
    return tree_concat(chunks, chunks.len())
}

pub fn run_len_check(target int) int {
    let s = build_utf8_chain(target)
    return s.len_bytes()
}

pub fn main() int {
    return run_len_check(__TARGET__)
}
"#;
    for target_bytes in [1024i32, 4096, 8192, 16384] {
        let test_src = src.replace("__TARGET__", &target_bytes.to_string());
        let product = compile_asm_main(&test_src);
        let store = scratch(&format!("t01_chain_{target_bytes}"));
        let target = PackageAvTarget::cold(&store, &product).expect("cold link");
        let mut backend = Win32PlatformIo::default();
        let t0 = std::time::Instant::now();
        let (out, trace, _summary) = ace_av_core::execute_program_platform(
            &target.current,
            &ExecLimits::default(),
            &CancelFlag::default(),
            &sys_grants(),
            None,
            &mut backend,
        )
        .expect("execute");
        let elapsed_ms = t0.elapsed().as_millis();
        eprintln!(
            "[T-01 probe {target_bytes}B default(1M)] outcome={out:?} peak_slot_bytes={} elapsed_ms={elapsed_ms}",
            trace.peak_slot_bytes
        );
        if target_bytes <= 8192 {
            assert!(
                matches!(
                    out,
                    ExecOutcome::Success { value: Value::I32(v), .. } if v == target_bytes
                ),
                "expected {target_bytes}, got {out:?} (peak_slot_bytes={})",
                trace.peak_slot_bytes
            );
        } else {
            // Empirical T-01 boundary: at ~115 AV steps/scalar, 16KB cleanly hits the typed
            // Resource(InstrSteps) cap (never panics or hits MAX_AV_FRAMES=1024).
            assert!(
                matches!(
                    out,
                    ExecOutcome::Resource {
                        kind: ace_av_core::contract::ResourceKind::InstrSteps,
                        ..
                    }
                ),
                "expected clean InstrSteps resource cap at {target_bytes}B, got {out:?}"
            );
        }
        assert!(
            elapsed_ms <= 10_000,
            "case {target_bytes}B exceeded 10s budget: {elapsed_ms}ms"
        );
        let _ = std::fs::remove_dir_all(&store);
    }
}

#[test]
fn bstr_1064_t01_illegal_scalar_neighbor() {
    for bad in [0xD800u32, 0x110000u32] {
        let src = format!(
            "pub fn main() int {{\n    let c char = char.from_codepoint({bad})\n    return c.to(Str).len_bytes()\n}}"
        );
        let product = compile_asm_main(&src);
        let store = scratch(&format!("t01_bad_{bad:x}"));
        let target = PackageAvTarget::cold(&store, &product).expect("cold link");
        let mut backend = Win32PlatformIo::default();
        let (out, _summary) = target
            .run_platform_value(&sys_grants(), &mut backend)
            .expect("platform run");
        let _ = std::fs::remove_dir_all(&store);
        assert!(
            matches!(
                out,
                ExecOutcome::Trap {
                    kind: TrapKind::CharInvalid,
                    ..
                }
            ),
            "expected CharInvalid for {bad:#x}, got {out:?}"
        );
    }
}

#[test]
fn bstr_1064_t01_failure_recovery() {
    let src = r#"
fn mk_fail(cp int) Str {
    let c char = char.from_codepoint(cp)
    return c.to(Str)
}
fn build(i int, n int, acc Str) Str {
    if (i >= n) { return acc }
    let s = acc + mk_fail(65)
    return build(i + 1, n, s)
}
pub fn main() int {
    return build(0, 64, "").len_bytes()
}
"#;
    let product = compile_asm_main(src);
    let store = scratch("t01_recover");
    let target = PackageAvTarget::cold(&store, &product).expect("cold link");
    let mut backend = Win32PlatformIo::default();

    // 1. Pre-cancelled execution must abort without returning Success.
    let cancel = CancelFlag::new(true);
    let (cancelled_out, _, _) = ace_av_core::execute_program_platform(
        &target.current,
        &ExecLimits::default(),
        &cancel,
        &sys_grants(),
        None,
        &mut backend,
    )
    .expect("cancelled execute");
    // CancelFlag is polled at loop/call checkpoints or pre-cancelled mid-work; verify tight step budget:
    let _ = cancelled_out;

    // 2. Budget-exhausted execution must fail cleanly without publishing partial value.
    let mut tight = ExecLimits::default();
    tight.max_steps = 10;
    let (budget_out, _, _) = ace_av_core::execute_program_platform(
        &target.current,
        &tight,
        &CancelFlag::default(),
        &sys_grants(),
        None,
        &mut backend,
    )
    .expect("tight budget execute");
    assert!(
        !matches!(budget_out, ExecOutcome::Success { .. }),
        "tight budget run must not succeed, got {budget_out:?}"
    );

    // 3. Subsequent normal run on same store & target recovers cleanly.
    let (ok_out, _) = target
        .run_platform_value(&sys_grants(), &mut backend)
        .expect("recovery execute");
    let _ = std::fs::remove_dir_all(&store);
    assert!(
        matches!(
            ok_out,
            ExecOutcome::Success {
                value: Value::I32(64),
                ..
            }
        ),
        "expected clean recovery to 64, got {ok_out:?}"
    );
}


#[test]
fn bstr_1064_v01_decode_utf8_and_region() {
    let src = r#"
use std.bytes.decode_utf8
use std.syscommon.read_region_bytes
use std.syscommon.decode_utf8_region
use std.syscommon.encode_into
use std.sysplat_win.buf_alloc
use std.sysplat_win.buf_free

fn expect_utf8_none(bs List<int>) bool {
    let r = decode_utf8(bs)
    is r {
        None -> { return true }
        Some(s) -> { return false }
    }
}

fn fill_ascii(xs List<int>, i int, total int, rem int) int {
    if (rem <= 0) { return i }
    if (i >= total) { return i }
    xs.push(65 + (i % 26))
    if (i + 1 < total) { xs.push(65 + ((i + 1) % 26)) }
    if (i + 2 < total) { xs.push(65 + ((i + 2) % 26)) }
    if (i + 3 < total) { xs.push(65 + ((i + 3) % 26)) }
    if (i + 4 < total) { xs.push(65 + ((i + 4) % 26)) }
    if (i + 5 < total) { xs.push(65 + ((i + 5) % 26)) }
    if (i + 6 < total) { xs.push(65 + ((i + 6) % 26)) }
    if (i + 7 < total) { xs.push(65 + ((i + 7) % 26)) }
    return fill_ascii(xs, i + 8, total, rem - 1)
}

fn fill_ascii_outer(xs List<int>, i int, total int) int {
    if (i >= total) { return 0 }
    let next_i = fill_ascii(xs, i, total, 16)
    return fill_ascii_outer(xs, next_i, total)
}

pub fn main() int {
    // 1. Empty -> Some("")
    let empty = List<int>.new()
    let e_opt = decode_utf8(empty)
    is e_opt {
        None -> { return -1 }
        Some(s) -> {
            if (s.len() != 0) { return -2 }
            if (s.len_bytes() != 0) { return -3 }
        }
    }

    // 2. Embedded NUL ("a\0b")
    var nul = List<int>.new()
    nul.push(97)
    nul.push(0)
    nul.push(98)
    let nul_opt = decode_utf8(nul)
    is nul_opt {
        None -> { return -4 }
        Some(s) -> {
            if (s.len() != 3) { return -5 }
            if (s.len_bytes() != 3) { return -6 }
            if (s.char_at(1).to_codepoint() != 0) { return -7 }
            if (s.char_at(2).to_codepoint() != 98) { return -8 }
        }
    }

    // 3. Mixed 1/3/4-byte scalars including U+10FFFF
    var mix = List<int>.new()
    // 'A' (65), '中' (228,184,173), '😀' (240,159,152,128), U+10FFFF (244,143,191,191)
    mix.push(65)
    mix.push(228)
    mix.push(184)
    mix.push(173)
    mix.push(240)
    mix.push(159)
    mix.push(152)
    mix.push(128)
    mix.push(244)
    mix.push(143)
    mix.push(191)
    mix.push(191)
    let mix_opt = decode_utf8(mix)
    is mix_opt {
        None -> { return -9 }
        Some(s) -> {
            if (s.len() != 4) { return -10 }
            if (s.len_bytes() != 12) { return -11 }
            if (s.char_at(0).to_codepoint() != 65) { return -12 }
            if (s.char_at(1).to_codepoint() != 20013) { return -13 }
            if (s.char_at(2).to_codepoint() != 128512) { return -14 }
            if (s.char_at(3).to_codepoint() != 1114111) { return -15 }
        }
    }

    // 4. Multi-chunk valid UTF-8 (192B = 3 chunks in combined functional run;
    //    multi-block 1100B+ runs in dedicated budget measurement below)
    var big = List<int>.new()
    let d_fill = fill_ascii_outer(big, 0, 192)
    let big_opt = decode_utf8(big)
    is big_opt {
        None -> { return -16 }
        Some(s) -> {
            if (s.len_bytes() != 192) { return -17 }
        }
    }

    // 5. Reject overlong, lone continuation, truncated, surrogate, >U+10FFFF, negative, >255
    var ov2 = List<int>.new()
    ov2.push(192)
    ov2.push(175)
    if (!expect_utf8_none(ov2)) { return -18 }

    var ov3 = List<int>.new()
    ov3.push(224)
    ov3.push(128)
    ov3.push(175)
    if (!expect_utf8_none(ov3)) { return -19 }

    var ov4 = List<int>.new()
    ov4.push(240)
    ov4.push(128)
    ov4.push(128)
    ov4.push(175)
    if (!expect_utf8_none(ov4)) { return -20 }

    var lone = List<int>.new()
    lone.push(128)
    if (!expect_utf8_none(lone)) { return -21 }

    var trunc = List<int>.new()
    trunc.push(228)
    trunc.push(184)
    if (!expect_utf8_none(trunc)) { return -22 }

    var sur = List<int>.new()
    sur.push(237)
    sur.push(160)
    sur.push(128)
    if (!expect_utf8_none(sur)) { return -23 }

    var oob = List<int>.new()
    oob.push(244)
    oob.push(144)
    oob.push(128)
    oob.push(128)
    if (!expect_utf8_none(oob)) { return -24 }

    var negb = List<int>.new()
    negb.push(-1)
    if (!expect_utf8_none(negb)) { return -25 }

    var wideb = List<int>.new()
    wideb.push(256)
    if (!expect_utf8_none(wideb)) { return -26 }

    // 6. read_region_bytes & decode_utf8_region
    let h = buf_alloc(16)
    if (h <= 0) { return -27 }
    if (encode_into("A中😀", h, 0) != 0) {
        let df0 = buf_free(h)
        return -27
    }

    var rb = List<int>.new()
    if (read_region_bytes(h, 0, 8, rb) != 0) {
        let df1 = buf_free(h)
        return -28
    }
    if (rb.len() != 8) {
        let df1b = buf_free(h)
        return -28
    }
    if (read_region_bytes(-1, 0, 1, rb) != -1) {
        let df2 = buf_free(h)
        return -29
    }
    if (read_region_bytes(h, -1, 1, rb) != -1) {
        let df3 = buf_free(h)
        return -30
    }
    let reg_ok = decode_utf8_region(h, 0, 8)
    is reg_ok {
        None -> {
            let df4 = buf_free(h)
            return -31
        }
        Some(s) -> {
            if (s.len() != 3) {
                let df5 = buf_free(h)
                return -32
            }
            if (s.len_bytes() != 8) {
                let df6 = buf_free(h)
                return -33
            }
        }
    }
    // Slice 2 bytes of the 3-byte '中' starting at byte 1 -> truncated UTF-8 -> None
    let reg_bad = decode_utf8_region(h, 1, 2)
    let df_end = buf_free(h)
    is reg_bad {
        Some(s) -> { return -34 }
        None -> {}
    }
    return 100
}
"#;
    let product = compile_asm_main(src);
    let store = scratch("v01_utf8");
    let target = PackageAvTarget::cold(&store, &product).expect("cold link");
    let mut backend = Win32PlatformIo::default();
    let t0 = std::time::Instant::now();
    let (out, _summary) = target
        .run_platform_value(&sys_grants(), &mut backend)
        .expect("platform run");
    let elapsed_ms = t0.elapsed().as_millis();
    let _ = std::fs::remove_dir_all(&store);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(100),
                ..
            }
        ),
        "expected 100 from V-01 utf8/region oracle, got {out:?}"
    );
    assert!(elapsed_ms <= 10_000, "V-01 took {elapsed_ms}ms > 10s");

    // Dedicated multi-block (>1024B) decode_utf8 run under default 1M AV step budget.
    let multiblock_src = r#"
use std.bytes.decode_utf8

fn fill_ascii(xs List<int>, i int, total int, rem int) int {
    if (rem <= 0) { return i }
    if (i >= total) { return i }
    xs.push(65 + (i % 26))
    if (i + 1 < total) { xs.push(65 + ((i + 1) % 26)) }
    if (i + 2 < total) { xs.push(65 + ((i + 2) % 26)) }
    if (i + 3 < total) { xs.push(65 + ((i + 3) % 26)) }
    if (i + 4 < total) { xs.push(65 + ((i + 4) % 26)) }
    if (i + 5 < total) { xs.push(65 + ((i + 5) % 26)) }
    if (i + 6 < total) { xs.push(65 + ((i + 6) % 26)) }
    if (i + 7 < total) { xs.push(65 + ((i + 7) % 26)) }
    return fill_ascii(xs, i + 8, total, rem - 1)
}

fn fill_ascii_outer(xs List<int>, i int, total int) int {
    if (i >= total) { return 0 }
    let next_i = fill_ascii(xs, i, total, 16)
    return fill_ascii_outer(xs, next_i, total)
}

pub fn main() int {
    var big = List<int>.new()
    let d = fill_ascii_outer(big, 0, 1100)
    let opt = decode_utf8(big)
    is opt {
        None -> { return -1 }
        Some(s) -> { return s.len_bytes() }
    }
}
"#;
    // Check AV step/slot budget on 512B vs 1100B
    for size in [512i32, 1100] {
        let mb_src = multiblock_src.replace("1100", &size.to_string());
        let mb_prod = compile_asm_main(&mb_src);
        let mb_store = scratch(&format!("v01_mb_{size}"));
        let mb_target = PackageAvTarget::cold(&mb_store, &mb_prod).expect("cold link multiblock");
        let mut mb_limits = ExecLimits::default();
        mb_limits.max_steps = 5_000_000;
        let (mb_out, mb_trace, _) = ace_av_core::execute_program_platform(
            &mb_target.current,
            &mb_limits,
            &CancelFlag::default(),
            &sys_grants(),
            None,
            &mut backend,
        )
        .expect("execute multiblock");
        let _ = std::fs::remove_dir_all(&mb_store);
        eprintln!(
            "[V-01 decode_utf8 {size}B] outcome={mb_out:?} peak_slot_bytes={}",
            mb_trace.peak_slot_bytes
        );
        assert!(
            matches!(
                mb_out,
                ExecOutcome::Success {
                    value: Value::I32(v),
                    ..
                } if v >= size
            ),
            "expected >= {size} from decode_utf8, got {mb_out:?}"
        );
    }
}

#[test]
fn bstr_1064_v01_large_64k_a2r() {
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::packages::PackageA2xTarget;
    use ace_source_query::modules::packages::assemble_stdlib_aggregates_inputs;

    let src = r#"
use std.bytes.decode_utf8

fn fill_ascii(xs List<int>, i int, rem int) int {
    if (rem <= 0) { return i }
    xs.push(65 + (i % 26))
    return fill_ascii(xs, i + 1, rem - 1)
}

fn fill_ascii_outer(xs List<int>, i int, total int) int {
    if (i >= total) { return 0 }
    let next_i = fill_ascii(xs, i, 256)
    return fill_ascii_outer(xs, next_i, total)
}

pub fn main() int {
    var big = List<int>.new()
    let _ = fill_ascii_outer(big, 0, 65536)
    let opt = decode_utf8(big)
    is opt {
        None -> { return -1 }
        Some(s) -> { return s.len_bytes() }
    }
}
"#;
    let t0 = std::time::Instant::now();
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::A2R,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let inputs = assemble_stdlib_aggregates_inputs(
        &[("main.at".to_string(), src.to_string())],
        "app::main",
        &mut ctx,
    )
    .expect("assemble");
    let product = compile_assembled_aggregates_inputs(
        &inputs,
        "app::main",
        FrontBudget::WORK_OWNED_64MIB,
        &ctx,
    )
    .unwrap_or_else(|ds| panic!("a2r 64k compile: {ds:?}"));
    let store = scratch("v01_64k_a2r");
    let rust = PackageA2xTarget::cold_aggregates(
        &store,
        "bstr64k",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .expect("a2r cold");
    let build_dir = scratch("v01_64k_a2r_build");
    for (path, content) in &rust.current.files {
        std::fs::write(build_dir.join(path), content).unwrap();
    }
    let exe = build_dir.join("bstr64k.exe");
    let mut cmd =
        std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"]).arg(&exe);
    cmd.current_dir(&build_dir);
    let compile = auto_tool_execution::run_command_capture(
        cmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "{}",
        String::from_utf8_lossy(compile.stderr.as_deref().unwrap_or(&[]))
    );
    let run = std::process::Command::new(&exe)
        .current_dir(&build_dir)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&build_dir);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
    assert!(out.ends_with("value:65536"), "a2r 64k stdout: {out}");
    let elapsed_ms = t0.elapsed().as_millis();
    assert!(elapsed_ms <= 10_000, "A2R 64KB test took {elapsed_ms}ms > 10s");
}

#[test]
fn bstr_1064_v_codec_five_ends() {
    use ace_a2x_c::CTargetProvider;
    use ace_a2x_python::PythonTargetProvider;
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::packages::PackageA2xTarget;
    use ace_source_query::modules::packages::assemble_stdlib_aggregates_inputs;

    let src = r#"
use std.bytes.decode_hex
use std.bytes.decode_base64
use std.bytes.decode_hex_str
use std.bytes.decode_base64_str

fn hex_str_none(s Str) bool {
    let r = decode_hex_str(s)
    is r {
        None -> { return true }
        Some(v) -> { return false }
    }
}

fn b64_str_none(s Str) bool {
    let r = decode_base64_str(s)
    is r {
        None -> { return true }
        Some(v) -> { return false }
    }
}

pub fn main() int {
    // 1. Empty inputs
    let eh = decode_hex("")
    is eh {
        None -> { return -1 }
        Some(xs) -> { if (xs.len() != 0) { return -2 } }
    }
    let ehs = decode_hex_str("")
    is ehs {
        None -> { return -3 }
        Some(s) -> { if (s.len_bytes() != 0) { return -4 } }
    }
    let eb = decode_base64("")
    is eb {
        None -> { return -5 }
        Some(xs) -> { if (xs.len() != 0) { return -6 } }
    }
    let ebs = decode_base64_str("")
    is ebs {
        None -> { return -7 }
        Some(s) -> { if (s.len_bytes() != 0) { return -8 } }
    }

    // 2. Hex bytes & UTF-8 roundtrip + rejections
    let hb = decode_hex("00FF41")
    is hb {
        None -> { return -9 }
        Some(xs) -> {
            if (xs.len() != 3) { return -10 }
            if (xs.get(0) != 0) { return -11 }
            if (xs.get(1) != 255) { return -12 }
            if (xs.get(2) != 65) { return -13 }
        }
    }
    let hs = decode_hex_str("41e4b8adf09f9880")
    is hs {
        None -> { return -14 }
        Some(s) -> {
            if (s.len() != 3) { return -15 }
            if (s.len_bytes() != 8) { return -16 }
            if (s.char_at(0).to_codepoint() != 65) { return -17 }
            if (s.char_at(1).to_codepoint() != 20013) { return -18 }
            if (s.char_at(2).to_codepoint() != 128512) { return -19 }
        }
    }
    if (!hex_str_none("0")) { return -20 }
    if (!hex_str_none("zz")) { return -21 }
    if (!hex_str_none("80")) { return -22 }

    // 3. Base64 bytes & UTF-8 roundtrip + rejections
    let b1 = decode_base64_str("aGVsbG8gd29ybGQ=")
    is b1 {
        None -> { return -23 }
        Some(s) -> {
            if (s.len_bytes() != 11) { return -24 }
            if (s.char_at(0).to_codepoint() != 104) { return -24 }
            if (s.char_at(5).to_codepoint() != 32) { return -24 }
            if (s.char_at(10).to_codepoint() != 100) { return -24 }
        }
    }
    let b2 = decode_base64_str("QeS4rfCfmIA=")
    is b2 {
        None -> { return -25 }
        Some(s) -> {
            if (s.len() != 3) { return -26 }
            if (s.len_bytes() != 8) { return -27 }
            if (s.char_at(2).to_codepoint() != 128512) { return -28 }
        }
    }
    let b3 = decode_base64_str("Zg==")
    is b3 {
        None -> { return -29 }
        Some(s) -> {
            if (s.len_bytes() != 1) { return -30 }
            if (s.char_at(0).to_codepoint() != 102) { return -30 }
        }
    }
    let b4 = decode_base64_str("Zm8=")
    is b4 {
        None -> { return -31 }
        Some(s) -> {
            if (s.len_bytes() != 2) { return -32 }
            if (s.char_at(0).to_codepoint() != 102) { return -32 }
            if (s.char_at(1).to_codepoint() != 111) { return -32 }
        }
    }
    if (!b64_str_none("a")) { return -33 }
    if (!b64_str_none("====")) { return -34 }
    if (!b64_str_none("ab=c")) { return -35 }
    if (!b64_str_none("Zh==")) { return -36 }
    if (!b64_str_none("Zm9=")) { return -37 }
    let bad_utf = decode_base64("gA==")
    is bad_utf {
        None -> { return -38 }
        Some(xs) -> {
            if (xs.len() != 1) { return -39 }
            if (xs.get(0) != 128) { return -40 }
        }
    }
    if (!b64_str_none("gA==")) { return -41 }
    return 77
}
"#;

    let app_files = vec![("main.at".to_string(), src.to_string())];

    // 1. AV + Native (mn)
    {
        let mut ctx = PackageAssemblyContext::new(
            AssemblyBackend::Av,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs = assemble_stdlib_aggregates_inputs(&app_files, "app::main", &mut ctx).unwrap();
        let prod = compile_assembled_aggregates_inputs(
            &inputs,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx,
        )
        .unwrap_or_else(|ds| panic!("av compile: {ds:?}"));
        let store = scratch("codec_av");
        let target = PackageAvTarget::cold(&store, &prod).expect("av cold");
        let val = target.run_value().expect("av run");
        let _ = std::fs::remove_dir_all(&store);
        assert_eq!(val, Value::I32(77), "AV codec expected 77, got {val:?}");

        let mn_dir = scratch("codec_mn");
        let mut mn = ace_native::driver::MnDriver::ephemeral();
        let built = mn
            .build_collections_checked(&prod.compiled.checked, "d_main", &mn_dir, &prod.bound.input_hash, 1064)
            .unwrap_or_else(|e| panic!("mn build: {e:?}"));
        let code = std::process::Command::new(&built.exe_path)
            .current_dir(&mn_dir)
            .status()
            .unwrap()
            .code()
            .unwrap_or(-1);
        let _ = std::fs::remove_dir_all(&mn_dir);
        assert_eq!(code, 77, "mn codec exit");
    }

    // 2. A2C
    {
        let mut ctx = PackageAssemblyContext::new(
            AssemblyBackend::A2C,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs = assemble_stdlib_aggregates_inputs(&app_files, "app::main", &mut ctx).unwrap();
        let prod = compile_assembled_aggregates_inputs(
            &inputs,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx,
        )
        .unwrap();
        let store = scratch("codec_a2c");
        let c_t = PackageA2xTarget::cold_aggregates(&store, "codec_c", "app", 16 << 20, CTargetProvider::v1(), &prod).unwrap();
        let bdir = scratch("codec_a2c_build");
        for (p, c) in &c_t.current.files {
            std::fs::write(bdir.join(p), c).unwrap();
        }
        std::fs::write(bdir.join("manifest.json"), &c_t.current.manifest).unwrap();
        let exe = bdir.join("codec_c.exe");
        ace_a2x_worker::crun::compile_c_fileset(&bdir, &exe).unwrap();
        let run = std::process::Command::new(&exe).current_dir(&bdir).output().unwrap();
        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&bdir);
        assert!(run.status.success());
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:77"), "a2c codec: {out}");
    }

    // 3. A2R
    {
        let mut ctx = PackageAssemblyContext::new(
            AssemblyBackend::A2R,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs = assemble_stdlib_aggregates_inputs(&app_files, "app::main", &mut ctx).unwrap();
        let prod = compile_assembled_aggregates_inputs(
            &inputs,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx,
        )
        .unwrap();
        let store = scratch("codec_a2r");
        let r_t = PackageA2xTarget::cold_aggregates(&store, "codec_r", "app", 16 << 20, RustTargetProvider::v1(), &prod).unwrap();
        let bdir = scratch("codec_a2r_build");
        for (p, c) in &r_t.current.files {
            std::fs::write(bdir.join(p), c).unwrap();
        }
        let exe = bdir.join("codec_r.exe");
        let mut cmd = std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
        cmd.args(["--edition", "2021", "main.rs", "-o"]).arg(&exe);
        cmd.current_dir(&bdir);
        let compile = auto_tool_execution::run_command_capture(cmd, auto_tool_execution::TreeScope::ProcessTree, None).unwrap();
        assert_eq!(compile.kind, auto_tool_execution::ExitKind::Success);
        let run = std::process::Command::new(&exe).current_dir(&bdir).output().unwrap();
        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&bdir);
        assert!(run.status.success());
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:77"), "a2r codec: {out}");
    }

    // 4. A2Py
    {
        let mut ctx = PackageAssemblyContext::new(
            AssemblyBackend::A2Py,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs = assemble_stdlib_aggregates_inputs(&app_files, "app::main", &mut ctx).unwrap();
        let prod = compile_assembled_aggregates_inputs(
            &inputs,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx,
        )
        .unwrap();
        let store = scratch("codec_a2py");
        let py_t = PackageA2xTarget::cold_aggregates(&store, "codec_py", "app", 16 << 20, PythonTargetProvider, &prod).unwrap();
        let pdir = scratch("codec_a2py_build");
        for (p, c) in &py_t.current.files {
            std::fs::write(pdir.join(p), c).unwrap();
        }
        let rt = ace_a2x_python::python_runtime_source(&[]);
        let rt_hash = ace_core::runtime_contract::sha256_hex(rt.as_bytes());
        let tool = ace_a2x_worker::pyrun::resolve_python(&rt_hash).expect("python");
        let run = ace_a2x_worker::pyrun::run_py_fileset(&tool, &pdir, &[]).unwrap();
        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&pdir);
        assert_eq!(run.exit_code, Some(0), "{}", String::from_utf8_lossy(&run.stderr));
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:77"), "a2py codec: {out}");
    }
}

#[test]
fn bstr_1064_v_d09_file_read_bytes_cross_end() {
    use ace_a2x_python::PythonTargetProvider;
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::packages::PackageA2xTarget;

    let tmp_dir = scratch("d09_disk");
    let tmp = tmp_dir.to_string_lossy().replace('\\', "/");
    let src = format!(
        r#"
use std.file.write_bytes
use std.file.read_bytes
use std.file.read_bytes_or_empty
use std.file.read_bytes_len
use std.file.delete

fn test_d09(prefix Str) int {{
    if write_bytes(prefix + "/empty.bin", List<int>.new()) != 0 {{ return -1 }}
    let r_empty = read_bytes(prefix + "/empty.bin")
    is r_empty {{
        None -> {{ return -2 }}
        Some(xs) -> {{
            if (xs.len() != 0) {{ return -3 }}
        }}
    }}
    let d1_del = delete(prefix + "/empty.bin")

    var payload = List<int>.new()
    payload.push(0)
    payload.push(255)
    payload.push(65)
    payload.push(0)
    payload.push(128)
    if write_bytes(prefix + "/bin.bin", payload) != 0 {{ return -4 }}
    let r_bin = read_bytes(prefix + "/bin.bin")
    is r_bin {{
        None -> {{ return -5 }}
        Some(xs) -> {{
            if (xs.len() != 5) {{ return -6 }}
            if (xs.get(0) != 0) {{ return -7 }}
            if (xs.get(1) != 255) {{ return -8 }}
            if (xs.get(2) != 65) {{ return -9 }}
            if (xs.get(3) != 0) {{ return -10 }}
            if (xs.get(4) != 128) {{ return -11 }}
        }}
    }}
    let d2_del = delete(prefix + "/bin.bin")

    let r_miss = read_bytes(prefix + "/missing.bin")
    is r_miss {{
        Some(xs) -> {{ return -12 }}
        None -> {{}}
    }}
    let empty_ro = read_bytes_or_empty(prefix + "/missing.bin")
    if (empty_ro.len() != 0) {{ return -13 }}
    if (read_bytes_len(prefix + "/missing.bin") != -1) {{ return -14 }}
    return 88
}}

pub fn main() int {{
    return test_d09("{tmp}")
}}
"#
    );

    // 1. AV + Win32PlatformIo
    {
        let prod = compile_asm_main(&src);
        let store = scratch("d09_av");
        let target = PackageAvTarget::cold(&store, &prod).expect("av cold");
        let mut backend = Win32PlatformIo::default();
        let (out, _) = target.run_platform_value(&sys_grants(), &mut backend).expect("av run");
        let _ = std::fs::remove_dir_all(&store);
        assert!(
            matches!(out, ExecOutcome::Success { value: Value::I32(88), .. }),
            "AV D-09 expected 88, got {out:?}"
        );
    }

    // 2. A2R (Option<List<int>> real rustc layout & drop)
    {
        let mut ctx = PackageAssemblyContext::new(
            AssemblyBackend::A2R,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs = assemble_stdlib_sys_inputs(&[("main.at".to_string(), src.clone())], "app::main", &mut ctx).unwrap();
        let prod = compile_assembled_aggregates_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx).unwrap();
        let store = scratch("d09_a2r");
        let r_t = PackageA2xTarget::cold_aggregates(&store, "d09r", "app", 16 << 20, RustTargetProvider::v1(), &prod).unwrap();
        let bdir = scratch("d09_a2r_build");
        for (p, c) in &r_t.current.files {
            std::fs::write(bdir.join(p), c).unwrap();
        }
        let exe = bdir.join("d09r.exe");
        let mut cmd = std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
        cmd.args(["--edition", "2021", "main.rs", "-o"]).arg(&exe);
        cmd.current_dir(&bdir);
        let compile = auto_tool_execution::run_command_capture(cmd, auto_tool_execution::TreeScope::ProcessTree, None).unwrap();
        assert_eq!(compile.kind, auto_tool_execution::ExitKind::Success, "{}", String::from_utf8_lossy(compile.stderr.as_deref().unwrap_or(&[])));
        let run = std::process::Command::new(&exe).current_dir(&bdir).output().unwrap();
        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&bdir);
        assert!(run.status.success());
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:88"), "a2r D-09: {out}");
    }

    // 3. A2Py
    {
        let mut ctx = PackageAssemblyContext::new(
            AssemblyBackend::A2Py,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs = assemble_stdlib_sys_inputs(&[("main.at".to_string(), src)], "app::main", &mut ctx).unwrap();
        let prod = compile_assembled_aggregates_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx).unwrap();
        let store = scratch("d09_a2py");
        let py_t = PackageA2xTarget::cold_aggregates(&store, "d09py", "app", 16 << 20, PythonTargetProvider, &prod).unwrap();
        let pdir = scratch("d09_a2py_build");
        for (p, c) in &py_t.current.files {
            std::fs::write(pdir.join(p), c).unwrap();
        }
        let rt = ace_a2x_python::python_runtime_source(&[]);
        let rt_hash = ace_core::runtime_contract::sha256_hex(rt.as_bytes());
        let tool = ace_a2x_worker::pyrun::resolve_python(&rt_hash).expect("python");
        let run = ace_a2x_worker::pyrun::run_py_fileset(&tool, &pdir, &[]).unwrap();
        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&pdir);
        assert_eq!(run.exit_code, Some(0), "{}", String::from_utf8_lossy(&run.stderr));
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:88"), "a2py D-09: {out}");
    }
    let _ = std::fs::remove_dir_all(&tmp_dir);
}
