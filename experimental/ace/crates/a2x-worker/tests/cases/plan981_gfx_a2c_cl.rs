//! PLAN-981（A2C 宿主面，第②项——用户裁定验收口径）：**真 cl 腿**。
//!
//! f32 常量（`NumericConst`→`a2x_f32_bits`）与端口 record 返回（`query2d.sweep`
//! /`segment` 的**具名 out 指针 ABI**，`AGE_PORT_FIELDS` 声明序）经
//! **cl 编译/链接/运行**取真值——cl 跑通证明的是 **A2C 的 C 宿主 ABI**：
//! 宿主 extern 与生成代码在同一链接里成立、宿主恰收一次调用、调用方把
//! 返回记录字段**读对**（正例 + 变异负控 + 宿主 fault 折叠）。
//!
//! 链路：packages 前门（`compile_package_product`，端口 record 由
//! `uses_age_ports` 注入）→ `PackageA2xTarget::cold` + `CTargetProvider`
//! → fileset（main.c/generated.c/运行时头）→ **宿主 stub .c** 一起进
//! `cl` → 运行 → stdout `value:N` / stderr `host:<op>` 计数。

use std::path::PathBuf;

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_core::source::adapter::CompiledSource;
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::PackageProduct;

/// 正例源：sweep 的记录字段（node=7、t∈(0.4,0.6)、start_overlap=false）与
/// segment 的（t∈(1.9,2.1)、hit=true）由宿主 stub 填入；调用方**段内**先读
/// （第二次查询前——语言侧 stale 走查如此要求），读出的判别落 bool 局部
/// （f32 只在表达式位置——C record-text 车道的 f32 **值通道**不在本批面）；
/// f32 字面量走 `a2x_f32_bits` 常量腿。
const MAIN: &str = "fn main() int {\n    let q Sweep2D = query2d.sweep(1, true, 0.5, 0.0)\n    let ok1 = q.node == 7 && q.t > 0.4 && q.t < 0.6 && q.start_overlap == false\n    let s Cast2D = query2d.segment(1, true, 0.0, 0.0, 1.0, 0.0)\n    let ok2 = s.t > 1.9 && s.t < 2.1 && s.hit == true\n    if ok1 && ok2 {\n        return 1\n    }\n    return 0\n}\n";

/// 宿主 stub：正例值（node=7、t=0.5、seg t=2.0）；逐派发打一行 stderr 计数。
const HOST_OK: &str = r#"
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>

int32_t a2x_gfx_query2d_sweep(int32_t a, bool b, float dx, float dy,
    bool* out_hit, int32_t* out_node, float* out_t,
    float* out_origin_x, float* out_origin_y,
    float* out_normal_x, float* out_normal_y, bool* out_start_overlap) {
    (void)a; (void)b; (void)dx; (void)dy;
    fprintf(stderr, "host:sweep\n");
    *out_hit = true; *out_node = 7; *out_t = 0.5f;
    *out_origin_x = 1.0f; *out_origin_y = 2.0f;
    *out_normal_x = 0.0f; *out_normal_y = 1.0f;
    *out_start_overlap = false;
    return 0;
}

int32_t a2x_gfx_query2d_segment(int32_t a, bool b, float ax, float ay, float bx, float by,
    bool* out_hit, int32_t* out_node, float* out_t,
    float* out_point_x, float* out_point_y,
    float* out_normal_x, float* out_normal_y) {
    (void)a; (void)b; (void)ax; (void)ay; (void)bx; (void)by;
    fprintf(stderr, "host:segment\n");
    *out_hit = true; *out_node = 3; *out_t = 2.0f;
    *out_point_x = 1.0f; *out_point_y = 2.0f;
    *out_normal_x = 0.0f; *out_normal_y = 1.0f;
    return 0;
}
"#;

/// 变异负控 stub：node=9（≠7）——调用方读到的记录**不是**这些值 ⇒ 判别失败。
const HOST_MUTANT: &str = r#"
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>

int32_t a2x_gfx_query2d_sweep(int32_t a, bool b, float dx, float dy,
    bool* out_hit, int32_t* out_node, float* out_t,
    float* out_origin_x, float* out_origin_y,
    float* out_normal_x, float* out_normal_y, bool* out_start_overlap) {
    (void)a; (void)b; (void)dx; (void)dy;
    fprintf(stderr, "host:sweep\n");
    *out_hit = true; *out_node = 9; *out_t = 0.5f;
    *out_origin_x = 1.0f; *out_origin_y = 2.0f;
    *out_normal_x = 0.0f; *out_normal_y = 1.0f;
    *out_start_overlap = false;
    return 0;
}

int32_t a2x_gfx_query2d_segment(int32_t a, bool b, float ax, float ay, float bx, float by,
    bool* out_hit, int32_t* out_node, float* out_t,
    float* out_point_x, float* out_point_y,
    float* out_normal_x, float* out_normal_y) {
    (void)a; (void)b; (void)ax; (void)ay; (void)bx; (void)by;
    fprintf(stderr, "host:segment\n");
    *out_hit = true; *out_node = 3; *out_t = 2.0f;
    *out_point_x = 1.0f; *out_point_y = 2.0f;
    *out_normal_x = 0.0f; *out_normal_y = 1.0f;
    return 0;
}
"#;

/// 宿主 fault stub：sweep 返回非 0 状态 ⇒ 生成侧折叠为 typed
/// `A2X_TRAP_RESOURCE`（零效果，wrapper 打 `trap:` 而非 `value:`）。
const HOST_FAULT: &str = r#"
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>

int32_t a2x_gfx_query2d_sweep(int32_t a, bool b, float dx, float dy,
    bool* out_hit, int32_t* out_node, float* out_t,
    float* out_origin_x, float* out_origin_y,
    float* out_normal_x, float* out_normal_y, bool* out_start_overlap) {
    (void)a; (void)b; (void)dx; (void)dy;
    (void)out_hit; (void)out_node; (void)out_t;
    (void)out_origin_x; (void)out_origin_y;
    (void)out_normal_x; (void)out_normal_y; (void)out_start_overlap;
    fprintf(stderr, "host:sweep\n");
    return 13;
}

int32_t a2x_gfx_query2d_segment(int32_t a, bool b, float ax, float ay, float bx, float by,
    bool* out_hit, int32_t* out_node, float* out_t,
    float* out_point_x, float* out_point_y,
    float* out_normal_x, float* out_normal_y) {
    (void)a; (void)b; (void)ax; (void)ay; (void)bx; (void)by;
    (void)out_hit; (void)out_node; (void)out_t;
    (void)out_point_x; (void)out_point_y;
    (void)out_normal_x; (void)out_normal_y;
    fprintf(stderr, "host:segment\n");
    return 0;
}
"#;

/// 整链：**age 包前门**（f32 字面量/端口 record 是 age-package 专属文法——
/// packages TextRecord 面无浮点字面量，实测具名拒）→ 绑定图随产物交出 →
/// 手工构造 `PackageProduct`（`CompiledSource` 全 pub 字段 + 同一 bound，
/// 不重跑绑定）→ `PackageA2xTarget::cold` + C provider → fileset →
/// **+宿主 stub** → 真 cl → 运行 → (stdout, stderr)。
fn run_cl_leg(tag: &str, host_stub: &str) -> (String, String) {
    let age = compile_age_package(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: MAIN.into(),
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .unwrap_or_else(|ds| {
        panic!(
            "age package compile failed: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });
    let product = PackageProduct {
        compiled: CompiledSource {
            profile: "age-package",
            input_hash: age.input_hash.clone(),
            atom_text: age.atom_text.clone(),
            checked: age.checked.clone(),
        },
        bound: age.bound,
    };

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/p981-gfx-cl/{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let mut target = PackageA2xTarget::cold(
        &root.join("store"),
        "p981-gfx-cl",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &product,
    )
    .unwrap_or_else(|e| panic!("C cold target: {e}"));
    let out = root.join("out");
    std::fs::create_dir_all(&out).unwrap();
    for (name, text) in &target.current.files {
        std::fs::write(out.join(name), text).unwrap();
    }
    std::fs::write(out.join("host_stub.c"), host_stub).unwrap();

    let exe = out.join("p981_gfx_cl.exe");
    ace_a2x_worker::crun::compile_c_fileset_with_sources(&out, &exe, &["host_stub.c"])
        .unwrap_or_else(|e| panic!("cl failed for {tag}: {e}"));
    let run = ace_a2x_worker::crun::run_exe(&exe).unwrap_or_else(|e| panic!("run {tag}: {e}"));
    (
        String::from_utf8_lossy(&run.stdout).to_string(),
        String::from_utf8_lossy(&run.stderr).to_string(),
    )
}

/// **正例**：宿主恰收一次 sweep/一次 segment；调用方把返回记录字段读对
/// （node=7、t∈(0.4,0.6)、start_overlap=false、seg t∈(1.9,2.1)、hit=true）
/// ⇒ `value:1`。f32 常量腿同跑（0.4/0.6/1.9/2.1 字面量）。
#[test]
fn p981_gfx_a2c_cl_sweep_and_segment_run_through_the_real_host_abi() {
    let (stdout, stderr) = run_cl_leg("ok", HOST_OK);
    assert!(
        stdout.contains("value:1"),
        "the caller read the host-provided record fields: stdout={stdout:?} stderr={stderr:?}"
    );
    assert_eq!(
        stderr.matches("host:sweep").count(),
        1,
        "the host receives exactly ONE sweep call"
    );
    assert_eq!(
        stderr.matches("host:segment").count(),
        1,
        "the host receives exactly ONE segment call"
    );
}

/// **变异负控**：宿主只改 node 7→9 ⇒ 判别失败 `value:0`——记录字段确实
/// 来自宿主 out 槽，不是常量折叠/打印端伪造。
#[test]
fn p981_gfx_a2c_cl_mutant_host_value_flips_the_result() {
    let (stdout, stderr) = run_cl_leg("mutant", HOST_MUTANT);
    assert!(
        stdout.contains("value:0"),
        "the mutated host record must flip the discriminant: stdout={stdout:?}"
    );
    assert_eq!(stderr.matches("host:sweep").count(), 1);
}

/// **宿主 fault 负控**：sweep 返回非 0 状态 ⇒ 生成侧折叠 typed
/// `A2X_TRAP_RESOURCE`（零效果）——wrapper 打 `trap:`，不打 `value:`。
#[test]
fn p981_gfx_a2c_cl_host_fault_folds_into_a_typed_trap() {
    let (stdout, stderr) = run_cl_leg("fault", HOST_FAULT);
    assert!(
        stdout.contains("trap:"),
        "a host fault must fold into a typed trap: stdout={stdout:?}"
    );
    assert!(
        !stdout.contains("value:1"),
        "the faulted call has zero effect: stdout={stdout:?}"
    );
    assert_eq!(
        stderr.matches("host:sweep").count(),
        1,
        "the faulting call still reached the host exactly once"
    );
    assert_eq!(stderr.matches("host:segment").count(), 0);
}

/// **image2d.png 具名边界**：端口含 owned-Str 字段（`Image2D.sha256`）——
/// 具名 out 槽 ABI 只承载标量字段；该 op 的 out 约定（宿主写 Str）另行
/// 联冻，本批在 C 车道**具名拒**（不猜、不伪造原型）。
#[test]
fn p981_gfx_a2c_image2d_port_stays_named_deferred() {
    use ace_a2x_c::lower_rt::{lower_functions_rt, print_fragment_rt};
    use ace_a2x_core::SemanticView;
    use ace_core::source::age_h0::LegacyWidgetAlias;
    use ace_core::source::age_package::compile_age_package;

    let compiled = compile_age_package(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: "pub fn probe() int {\n    let img Image2D = image2d.png(\"assets/x.png\")\n    return img.byte_length\n}\n\nfn main() int {\n    return probe()\n}\n".into(),
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("the age front door admits image2d.png");
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission: {r}"));
    let fns: Vec<_> = view.functions().iter().collect();
    let err = lower_functions_rt(&view, &fns)
        .expect_err("the C lane must named-defer the owned-Str port");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("owned-Str port fields are a separate frozen convention")
            || msg.contains("Image2D"),
        "named C deferral for image2d.png: {msg}"
    );
    let _ = print_fragment_rt; // silence unused-import lint in cfg variants
}
