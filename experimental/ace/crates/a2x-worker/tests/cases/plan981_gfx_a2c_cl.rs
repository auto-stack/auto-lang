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
fn run_cl_leg(tag: &str, src: &str, host_stub: &str) -> (String, String) {
    let age = compile_age_package(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: src.into(),
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
    let (stdout, stderr) = run_cl_leg("ok", MAIN, HOST_OK);
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
    let (stdout, stderr) = run_cl_leg("mutant", MAIN, HOST_MUTANT);
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
    let (stdout, stderr) = run_cl_leg("fault", MAIN, HOST_FAULT);
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

/// **image2d.png owned-Str out 约定（真 cl）**：宿主经 `a2x_stro_result*`
/// 写 owned 缓冲（stub 用运行时 `a2x_str_from_lit` 构造 sha），调用侧核
/// wrapper trap 后**移动**进 record 字段——调用方读 byte_length/width/height
/// 与 `sha256.len()` 判别 ⇒ `value:1`；宿主恰收一次调用。
const IMG_MAIN: &str = "fn main() int {\n    let img Image2D = image2d.png(\"assets/x.png\")\n    let n = img.byte_length\n    let w = img.width\n    let h = img.height\n    let sl = img.sha256.len()\n    if n == 16 && w == 64 && h == 32 && sl == 8 {\n        return 1\n    }\n    return 0\n}\n";

const HOST_PNG: &str = r#"
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include "a2x_c_rt_runtime.h"

int32_t a2x_gfx_image2d_png(a2x_str_view a0,
    a2x_stro_result* out_sha256, int32_t* out_byte_length,
    int32_t* out_width, int32_t* out_height) {
    (void)a0;
    fprintf(stderr, "host:png\n");
    a2x_str_view lit = { (const uint8_t*)"deadbeef", 8 };
    *out_sha256 = a2x_str_from_lit(lit);
    *out_byte_length = 16;
    *out_width = 64;
    *out_height = 32;
    return 0;
}
"#;

#[test]
fn p981_gfx_a2c_cl_image2d_owned_str_out_reaches_the_caller() {
    let (stdout, stderr) = run_cl_leg("png", IMG_MAIN, HOST_PNG);
    assert!(
        stdout.contains("value:1"),
        "the host-written owned-Str + dims reached the caller: stdout={stdout:?} stderr={stderr:?}"
    );
    assert_eq!(
        stderr.matches("host:png").count(),
        1,
        "the host receives exactly ONE image2d.png call"
    );
}

/// **宿主 Str wrapper trap 负控**：`a2x_str_from_lit` 分配失败不可注入，但
/// wrapper trap 通道可注入——stub 直接给 `out_sha256` 塞 trap ⇒ 生成侧
/// 折叠 typed trap、零效果（`trap:` 不打 `value:`），且**不再读其余字段**。
const HOST_PNG_TRAP: &str = r#"
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include "a2x_c_rt_runtime.h"

int32_t a2x_gfx_image2d_png(a2x_str_view a0,
    a2x_stro_result* out_sha256, int32_t* out_byte_length,
    int32_t* out_width, int32_t* out_height) {
    (void)a0;
    (void)out_byte_length; (void)out_width; (void)out_height;
    fprintf(stderr, "host:png\n");
    *out_sha256 = a2x_fwd_stro(A2X_TRAP_RESOURCE);
    return 0;
}
"#;

#[test]
fn p981_gfx_a2c_cl_image2d_str_wrapper_trap_folds_zero_effect() {
    let (stdout, stderr) = run_cl_leg("pngtrap", IMG_MAIN, HOST_PNG_TRAP);
    assert!(
        stdout.contains("trap:"),
        "a trapped owned-Str wrapper must fold into a typed trap: stdout={stdout:?}"
    );
    assert!(
        !stdout.contains("value:1"),
        "the trapped record has zero effect: stdout={stdout:?}"
    );
    assert_eq!(stderr.matches("host:png").count(), 1);
}

/// **C f32 值通道（真 cl）**：float **参数/局部（var）/返回**全链——
/// `step(dx f32) f32` 两步积分，`main` 读 f32 局部 + 调 f32 返回的 helper
/// ⇒ `value:1`；f32 常量走按位构造器。
const F32_MAIN: &str = "fn step(dx f32) f32 {\n    var v = dx * 2.0\n    v = v + 0.05\n    return v\n}\n\nfn main() int {\n    let d f32 = 0.25\n    let r f32 = step(d)\n    if r > 0.54 && r < 0.56 {\n        return 1\n    }\n    return 0\n}\n";

const HOST_NONE: &str = "/* no host ops called by the f32 value channel leg */\nint p981_f32_leg_host_placeholder = 0;\n";

#[test]
fn p981_gfx_a2c_cl_f32_value_channel_locals_params_returns() {
    let (stdout, stderr) = run_cl_leg("f32", F32_MAIN, HOST_NONE);
    assert!(
        stdout.contains("value:1"),
        "float locals/params/returns ran through real cl: stdout={stdout:?} stderr={stderr:?}"
    );
}
