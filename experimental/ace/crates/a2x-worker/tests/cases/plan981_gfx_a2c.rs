//! PLAN-981（A2C 宿主面，2026-11-18 用户裁定）：AGE gfx 宿主 op 的 **C 车道
//! 符号面**。
//!
//! 现状→交付：此前任何 gfx 宿主 intrinsic 在 A2X 视图层被一刀切
//! `intrinsic … outside the compute surface` 拒。本批沿 PLAN-1041 platform-op
//! 模式开面：
//! - 视图（`SemanticView`）按语言侧受检表（`age_resources::is_gfx_host_symbol`
//!   ——HostAlgorithm/HostIdentity 16 op）收 `gfx_defs` 符号面；
//! - **A2C**：标量结果 op 发射 `a2x_gfx_<callee>` **宿主 extern 调用**
//!   （实现/链接归 C 宿主，AC 车道 1068/T-04）；运行时头追加 extern 原型页
//!   （签名镜像受检表，str 走 `a2x_str_view` 内容视图）；
//! - **A2R**：具名拒（宿主车道归 AC/C 与 AV 会话——Rust 生成侧不伪造宿主实现）；
//! - **端口 record 3 op**（segment/sweep/image2d.png）：C 侧 record 返回 stub
//!   属端口 ABI 的 C 腿，本批**具名延后**（不伪造 record）。
//!
//! 真实 `cl` 链接腿（宿主 main.c + stub 实现）归 AC 车道消费方（1068/T-04）
//! ——本文件钉**合同面**（视图收面、发射文本、原型页、A2R/端口的具名边界）。

use ace_a2x_c::lower_rt::{lower_functions_rt, print_fragment_rt};
use ace_a2x_c::rt_runtime::record_text_runtime_source_with;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::lower_program;
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// 标量宿主 op 的最小作者源（age-package 面，无 node——宿主面直接走模块 fn，
/// 与 AV gfx_host 用例同一形）。**i32/bool/str 通道**（frame 的 str 走内容视图）；
/// C record-text 车道的 f32 常量通道是另一条具名腿，本用例不触。
const SCALAR_SRC: &str = "pub fn probe() int {\n    let f = render2d.frame(1, \"atlas_main\", 0, 1)\n    let d = render2d.despawn(2)\n    let p = input.pressed(1)\n    if p {\n        return f + d + input.space()\n    }\n    return f\n}\n\nfn main() int {\n    return probe()\n}\n";

/// 端口 record op 源（sweep——C 侧 record 返回 stub 具名延后；参数不降）。
const PORT_SRC: &str = "pub fn probe() int {\n    let q Sweep2D = query2d.sweep(1, true, 0.5, 0.0)\n    return q.node\n}\n\nfn main() int {\n    return probe()\n}\n";

fn input(src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    }
}

fn compile(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
    compile_age_package(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .unwrap_or_else(|ds| {
        panic!(
            "AGE package front door rejected the gfx source: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })
}

/// 视图按受检表收 gfx 符号面（不再是泛型 "outside the compute surface"），
/// 且 `gfx_symbols()` 给出去重集（运行时原型页的输入）。
#[test]
fn p981_gfx_view_admits_host_symbols_from_the_checked_table() {
    let compiled = compile(SCALAR_SRC);
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission: {r}"));
    assert!(view.has_gfx_ops(), "the gfx symbol face must be admitted");
    let syms = view.gfx_symbols();
    for want in [
        "age.input.pressed.v1",
        "age.input.space.v1",
        "age.render2d.despawn.v1",
        "age.render2d.frame.v1",
    ] {
        assert!(
            syms.contains(&want),
            "gfx_symbols() must carry `{want}`: {syms:?}"
        );
    }
    assert!(
        !syms.contains(&"age.frame2d.at.v1"),
        "value ctor is not a host op"
    );
}

/// A2C 发射：标量宿主 op → `a2x_gfx_<callee>` 调用（结果类型按受检表：
/// i32→int32_t、f32→float、bool→bool；str 实参走内容视图）。
#[test]
fn p981_gfx_a2c_emits_typed_host_extern_calls() {
    let compiled = compile(SCALAR_SRC);
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission: {r}"));
    let fns: Vec<_> = view.functions().iter().collect();
    let cfns =
        lower_functions_rt(&view, &fns).unwrap_or_else(|r| panic!("C record-text lowering: {r}"));
    let text: String = cfns.iter().map(print_fragment_rt).collect();
    // 逐 op 的调用文本（受检 callee → extern 名；frame 的 str 实参走内容视图）。
    assert!(
        text.contains("int32_t ") && text.contains("a2x_gfx_input_space()"),
        "input.space emits as an i32 host call: {text}"
    );
    assert!(
        text.contains("a2x_gfx_input_pressed("),
        "input.pressed emits as a bool host call: {text}"
    );
    assert!(
        text.contains("a2x_gfx_render2d_frame("),
        "render2d.frame emits with its str view + i32 args: {text}"
    );
    assert!(
        text.contains("a2x_gfx_render2d_despawn("),
        "render2d.despawn emits as a host call: {text}"
    );
    assert!(
        !text.contains("a2x_gfx_query2d_"),
        "port-record ops never emit a scalar host call"
    );
}

/// 运行时头追加 **extern 原型页**：宿主（AC 车道 1068/T-04）实现/链接；签名
/// 镜像受检表（`render2d.spawn(str,f32,f32)`、`row.place(i32,f32,f32)`）。
#[test]
fn p981_gfx_a2c_runtime_page_declares_host_externs() {
    let compiled = compile(SCALAR_SRC);
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission: {r}"));
    let page = record_text_runtime_source_with(&[], false, &view.gfx_symbols());
    assert!(
        page.contains("extern int32_t a2x_gfx_input_space(void);"),
        "the page declares the i32 host extern: {}",
        &page[page.len().saturating_sub(400)..]
    );
    assert!(
        page.contains("extern bool a2x_gfx_input_pressed(int32_t a);"),
        "bool result signature: {}",
        &page[page.len().saturating_sub(400)..]
    );
    assert!(
        page.contains(
            "extern int32_t a2x_gfx_render2d_frame(int32_t a, a2x_str_view a, int32_t a, int32_t a);"
        ),
        "frame signature mirrors the checked table (i32, str view, i32, i32): {}",
        &page[page.len().saturating_sub(600)..]
    );
    assert!(
        page.contains("extern int32_t a2x_gfx_render2d_despawn(int32_t a);"),
        "despawn signature mirrors the checked table: {}",
        &page[page.len().saturating_sub(400)..]
    );
    // 端口 record 结果的 3 op **不进原型页**（C record 返回 stub 具名延后）。
    assert!(!page.contains("a2x_gfx_query2d_"));
    // 无 gfx op 的模块不追加该页。
    let plain = record_text_runtime_source_with(&[], false, &[]);
    assert!(!plain.contains("PLAN-981 A2C host face"));
}

/// A2R 车道具名拒：符号面在视图收进后，Rust 生成侧仍**不伪造**宿主实现。
#[test]
fn p981_gfx_a2r_lane_named_rejects_host_ops() {
    let compiled = compile(SCALAR_SRC);
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission: {r}"));
    let err =
        lower_program(&view, "main").expect_err("the A2R lane must named-refuse gfx host ops");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("AGE graphics host operation") && msg.contains("age.render2d.frame.v1"),
        "named A2R boundary: {msg}"
    );
}

/// 端口 record op（query2d.sweep 等）在 C 车道**具名延后**：record 返回 stub 属
/// 端口 ABI 的 C 腿（标量宿主面先行；不伪造 record）。
#[test]
fn p981_gfx_a2c_port_record_ops_stay_named_deferred() {
    let compiled = compile(PORT_SRC);
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission: {r}"));
    let fns: Vec<_> = view.functions().iter().collect();
    let err = lower_functions_rt(&view, &fns)
        .expect_err("the C lane must named-defer port-record host ops");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("age.query2d.sweep.v1") && msg.contains("port-ABI C leg"),
        "named C deferral: {msg}"
    );
    // 原型页同样不替它伪造签名。
    let page = record_text_runtime_source_with(&[], false, &view.gfx_symbols());
    assert!(!page.contains("a2x_gfx_query2d_sweep"));
}
