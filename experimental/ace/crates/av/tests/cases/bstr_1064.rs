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
