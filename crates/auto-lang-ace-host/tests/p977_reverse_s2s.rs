//! PLAN-977 B-REVERSE-V: s2s 改写源在 ACE 公开前门上的真实执行证据。
//!
//! 链：Auto 源 → `auto_lang_core::trans::auto_s2s::lower_source_for_ace`
//! （规则重写 + ACE 规范形 + 行级 source map）→ `auto_lang_core::run_ace`
//! （ACE AV 执行器 = M2 唯一执行器）→ 独立手算值。
//!
//! 判别性（不接受"产物能跑"一句口号）：
//! - 历史方言（签名带 `->`）在同一门上必须是**具名**语法诊断——证明规范
//!   方言不是装饰，随便喂历史产物过不了；
//! - 诊断位必须经 source map 回到**原源行**（含单行块内的精确行），而不是
//!   停在"产物行号"或笼统的语句起点；
//! - 旧引擎名（autovm/vm/evaluator）在同一门上是具名拒绝——本条链上没有
//!   旧执行器回退。

use std::time::Instant;

use auto_lang_core::trans::auto_s2s::{lower_source, lower_source_for_ace};
use auto_lang_core::trans::emit::source_line_for_output;
use auto_lang_core::{
    run_ace, run_ace_named, AceEngineChoice, AceOutcome, AceRequest, AceRunReport, AceValue,
};

/// 本包 §5/§6 的具体输入：递归 fib，`main` 返回 fib(10)。
const FIB_SRC: &str = "fn fib(n int) -> int {\n    if n < 2 { return n }\n    return fib(n - 1) + fib(n - 2)\n}\nfn main() -> int {\n    return fib(10)\n}\n";

/// 诊断位用例：`main` 第 6 行调用 `add(1)`（少一个实参），第 1 行是 add。
const BROKEN_SRC: &str = "fn add(a int, b int) -> int {\n    return a + b\n}\n\nfn main() -> int {\n    return add(1)\n}\n";

fn install() {
    auto_lang_ace_host::install().expect("the host executor installs");
}

fn expect_success_i32(report: &AceRunReport, expected: i32) {
    match &report.outcome {
        AceOutcome::Success {
            value: AceValue::I32(v),
            steps,
        } => assert_eq!(*v, expected, "ACE success value (steps={steps})"),
        other => panic!("expected Success(I32({expected})), got {other:?}"),
    }
}

/// 独立 oracle：手算的斐波那契（不是"与旧 VM 相等"）。
fn fib_reference(n: i32) -> i32 {
    if n < 2 {
        n
    } else {
        fib_reference(n - 1) + fib_reference(n - 2)
    }
}

/// B-REVERSE-V 主链：s2s 改写产物在 ACE AV 上真实执行，值对独立手算；
/// 历史方言与旧引擎名在同一门上具名拒绝。
#[test]
fn p977_reverse_s2s_rewritten_source_runs_on_ace() {
    let started = Instant::now();
    install();

    let (source, map) = lower_source_for_ace(FIB_SRC).expect("the script rewrites");
    assert!(
        !source.contains("->"),
        "ACE 规范形不得带签名箭头:\n{source}"
    );
    assert_eq!(
        map.len(),
        source.lines().count(),
        "source map 必须覆盖每条产物行"
    );

    let report = run_ace(AceEngineChoice::AceAV, &AceRequest::simple(&source));
    expect_success_i32(&report, fib_reference(10));
    assert_eq!(fib_reference(10), 55, "独立手算 oracle");

    // 历史方言（公开命令的现状产物）在 ACE 上是具名语法拒绝：规范方言
    // 是可执行的前提，不是装饰性差异。
    let legacy = lower_source(FIB_SRC).expect("the legacy emission is unchanged");
    assert!(legacy.contains("->"), "历史方言仍写 `->`:\n{legacy}");
    match run_ace(AceEngineChoice::AceAV, &AceRequest::simple(&legacy)).outcome {
        AceOutcome::Diagnostic { stage, code, .. } => {
            assert_eq!(stage, "text", "历史方言在文本阶段就失败");
            assert_eq!(code, "source.syntax", "具名语法拒绝，不是运行错");
        }
        other => panic!("历史方言必须被 ACE 具名拒绝，got {other:?}"),
    }

    // 同一门不接受旧引擎名 = 本链没有旧执行器回退。
    for legacy_engine in ["autovm", "vm", "evaluator"] {
        let report = run_ace_named(legacy_engine, &AceRequest::simple(&source));
        assert!(
            matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
            "legacy engine `{legacy_engine}` must be a named refusal, got {:?}",
            report.outcome
        );
    }

    println!(
        "p977_reverse_s2s_rewritten_source_runs_on_ace elapsed: {:.3}s",
        started.elapsed().as_secs_f64()
    );
}

/// B-REVERSE-V 诊断映射：ACE 报出的产物内 span → source map → 原源行
/// （精确到 `return add(1)` 那一行，而不是笼统的语句起点）。
#[test]
fn p977_reverse_s2s_source_map_maps_diagnostics_to_original_line() {
    let started = Instant::now();
    install();

    let (source, map) = lower_source_for_ace(BROKEN_SRC).expect("the broken script rewrites");
    let report = run_ace(AceEngineChoice::AceAV, &AceRequest::simple(&source));
    let (stage, code, span, message) = match &report.outcome {
        AceOutcome::Diagnostic {
            stage,
            code,
            span,
            message,
        } => (
            stage.clone(),
            code.clone(),
            span.expect("the bind diagnostic carries the call-site span"),
            message.clone(),
        ),
        other => panic!("wrong arity must be a diagnosed source, got {other:?}"),
    };
    assert_eq!(stage, "bind", "错误实参个数在绑定阶段诊断: {message}");
    assert!(
        code.starts_with("source."),
        "source-family diagnostic expected, got {code}: {message}"
    );

    let output_line = source[..span.start as usize].matches('\n').count() + 1;
    let mapped = source_line_for_output(&map, output_line)
        .unwrap_or_else(|| panic!("产物行 {output_line} 必须映射回原源"));
    assert_eq!(
        mapped, 6,
        "诊断必须回到原源第 6 行（`return add(1)`），产物行号 {output_line}"
    );
    let original_line = BROKEN_SRC
        .lines()
        .nth(mapped - 1)
        .expect("the mapped original line exists");
    assert!(
        original_line.contains("add(1)"),
        "映射行必须是调用发生处，got {original_line:?}"
    );

    println!(
        "p977_reverse_s2s_source_map_maps_diagnostics_to_original_line elapsed: {:.3}s",
        started.elapsed().as_secs_f64()
    );
}

/// 产物行映射的总性与原点性：每条产物行有映射、原行号落在源内，且
/// 未改写的调用行映射到**它自己的原行**（不是笼统的语句起点）。
#[test]
fn p977_reverse_s2s_map_is_total_and_points_at_the_origin_line() {
    let started = Instant::now();
    for (name, src, needle, expected_line) in [
        ("fib", FIB_SRC, "return fib(10)", 6usize),
        ("broken", BROKEN_SRC, "return add(1)", 6usize),
    ] {
        let (source, map) = lower_source_for_ace(src).expect("source rewrites");
        assert_eq!(map.len(), source.lines().count(), "{name}: 每条产物行");
        for entry in &map {
            assert!(
                entry.source_line >= 1 && entry.source_line <= src.lines().count(),
                "{name}: 原行号越界 {}",
                entry.source_line
            );
        }
        let emitted_line = source
            .lines()
            .position(|line| line.trim() == needle)
            .unwrap_or_else(|| panic!("{name}: 产物里必须有 {needle:?}"))
            + 1;
        let mapped = source_line_for_output(&map, emitted_line)
            .unwrap_or_else(|| panic!("{name}: 产物行 {emitted_line} 必须映射回原源"));
        assert_eq!(
            mapped, expected_line,
            "{name}: 产物行 {emitted_line} 必须回到原行 {expected_line}"
        );
        assert!(
            src.lines()
                .nth(mapped - 1)
                .is_some_and(|line| line.contains(needle)),
            "{name}: 映射行必须是该调用的原行"
        );
    }
    println!(
        "p977_reverse_s2s_map_is_total_and_points_at_the_origin_line elapsed: {:.3}s",
        started.elapsed().as_secs_f64()
    );
}
