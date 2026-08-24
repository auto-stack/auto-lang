/// Plan 437 Phase 2：chart 几何的语言能力验证 + 端到端几何回归
///
/// 覆盖本计划修复的六个 VM/codegen 缺陷（详见 Plan 437 §0.6.C）：
/// EQ_F 族 VALID 白名单、GET_ELEM 动态 record 字段（d[field_name]）、
/// extract 的 tag-first、Bina float 类型记录、float 存储强转、
/// f32 算术的 tag 驱动操作数。端到端用例（line_path/path_probe）
/// 验证 records → scale → path `d` 字符串的完整链路。
///
/// 纪律：float 中间量显式声明 `float`；int→float 转换经存储强转或
/// 与 float 字面量混合（运行时 tag 驱动，两路均已修复）。

use crate::run;

// ---------- 能力验证 ----------

#[test]
fn test_geo_dynamic_record_field() {
    // d[name]：按字符串名访问 record 字段（charts 的 index/categories 语义）
    let code = r#"
        var d = { month: "Jan", desktop: 186 }
        var name = "desktop"
        d[name]
    "#;
    let result = run(code);
    match result {
        Ok(v) => assert_eq!(v, "186", "d[name] 应取出 desktop 字段"),
        Err(e) => panic!("动态字段访问不支持: {:?}", e),
    }
}

#[test]
fn test_geo_fstr_float_format() {
    // f-string 的 float 插值应为常规十进制（path d 字符串的基础）
    let code = r#"
        var x = 10.5
        var y = 20.25
        f"${x},${y}"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10.5,20.25");
}

#[test]
fn test_geo_string_accum_in_loop() {
    // 循环内累积字符串（line path 的 M/L 拼接）
    let code = r#"
        var pts List<float> = [10.0, 20.0, 30.0]
        var acc = "M"
        for p in pts {
            acc = acc + f" ${p}"
        }
        acc
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "M 10 20 30");
}

#[test]
fn test_geo_manual_enumeration() {
    // 手动计数枚举（x 坐标 = i * step）
    let code = r#"
        var vals List<int> = [3, 5, 2]
        var total = 0
        var i = 0
        for v in vals {
            total = total + v * i
            i = i + 1
        }
        total
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "9"); // 3*0 + 5*1 + 2*2（此前断言算错，VM 行为正确）
}

#[test]
fn test_geo_counter_scope_micro() {
    // 计数器微测试：for 体内对外层 var 的累加是否跨迭代保持
    let code = r#"
        var i = 0
        var s = 0
        for v in [10, 20, 30] {
            s = s + i
            i = i + 1
        }
        f"${i}-${s}"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3-3"); // s=0+1+2（此前断言算错，VM 行为正确）
}

#[test]
fn test_geo_min_max_over_records() {
    // record 列表上的动态字段 min/max（y 轴 scale 的域计算）
    let code = r#"
        var data = [
            { month: "Jan", v: 186 },
            { month: "Feb", v: 305 },
            { month: "Mar", v: 73 }
        ]
        var vmax = 0
        for d in data {
            var val = d["v"]
            if val > vmax {
                vmax = val
            }
        }
        vmax
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "305");
}

// ---------- 几何原语（d3 风格，v1 最小集）----------

#[test]
fn test_geo_linear_scale() {
    // 线性尺度：域 [0,100] → 值域 [40,560]（含 padding 的图表内区）
    let code = r#"
        fn lin(v float, d0 float, d1 float, r0 float, r1 float) -> float {
            if d1 == d0 {
                return r0
            }
            r0 + (v - d0) / (d1 - d0) * (r1 - r0)
        }
        f"[${lin(0.0, 0.0, 100.0, 40.0, 560.0)},${lin(50.0, 0.0, 100.0, 40.0, 560.0)},${lin(100.0, 0.0, 100.0, 40.0, 560.0)}]"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "[40,300,560]");
}

#[test]
fn test_geo_line_path_from_records() {
    // 端到端：records + index/categories → SVG path d 字符串。
    // 纪律（见 Plan 437 §0.6.C）：float 中间量一律显式声明 `float`——
    // 混合 int/f32 算术的运行时值正确但静态推断会标成 Int（f-string
    // part tag 随之错），显式声明让 var_types 记对类型。
    let code = r#"
        fn scale_y(v float, vmax float, top float, bottom float) -> float {
            if vmax == 0.0 { return bottom }
            bottom - v / vmax * (bottom - top)
        }
        var data = [
            { m: "Jan", v: 30 },
            { m: "Feb", v: 80 },
            { m: "Mar", v: 10 }
        ]
        var vmax int = 0
        for d in data {
            var val = d["v"]
            if val > vmax {
                vmax = val
            }
        }
        var vmaxf float = vmax * 1.0
        var dstr = ""
        var i = 0
        for d in data {
            var vf float = d["v"] * 1.0
            var x float = 40.0 + i * 200.0
            var y float = scale_y(vf, vmaxf, 20.0, 260.0)
            if i == 0 {
                dstr = f"M ${x} ${y}"
            } else {
                dstr = dstr + f" L ${x} ${y}"
            }
            i = i + 1
        }
        dstr
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "M 40 170 L 240 20 L 440 230"); // y=260-v/80*240（首版期望值手算错，VM 正确）
}

// ---------- 调用约定探针（诊断 path 测试全 260 的根因）----------

#[test]
fn test_geo_callconv_f32_literal() {
    let code = r#"
        fn f(x float) -> float { x * 2.0 }
        f"${f(3.0)}"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "6");
}

#[test]
fn test_geo_fn_identity() {
    let code = r#"
        fn h(x float) -> float { x }
        f"${h(3.0)}"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_geo_fn_noargs() {
    let code = r#"
        fn k() -> float { 3.0 * 2.0 }
        f"${k()}"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "6");
}

#[test]
fn test_geo_fn_two_args() {
    let code = r#"
        fn m(x float, y float) -> float { x * y }
        f"${m(3.0, 2.0)}"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "6");
}

#[test]
fn test_geo_fn_if_branch() {
    let code = r#"
        fn q(x float) -> float {
            if x == 0.0 { return 99.0 }
            x * 2.0
        }
        f"${q(3.0)}-${q(0.0)}"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "6-99");
}

#[test]
fn test_geo_path_probe() {
    let code = r#"
        var data = [
            { m: "Jan", v: 30 },
            { m: "Feb", v: 80 },
            { m: "Mar", v: 10 }
        ]
        var vmax int = 0
        for d in data {
            var val = d["v"]
            if val > vmax {
                vmax = val
            }
        }
        var vmaxf float = vmax * 1.0
        var vf float = data[1]["v"] * 1.0
        var y float = 260.0 - vf / vmaxf * 240.0
        f"[${vmaxf}|${vf}|${y}]"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "[80|80|20]");
}

#[test]
fn test_geo_math_trig() {
    // donut 圆弧所需三角函数：sin/cos 调用语法与精度
    let code = r#"
        var pi float = 3.14159265358979
        var half float = pi / 2.0
        var s float = math.sin(half)
        var c float = math.cos(0.0)
        f"${s}-${c}"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1-1");
}



// ---------- donut 几何（复算 /donut-chart 页 Init 的完整计算）----------

#[test]
fn test_geo_donut_paths() {
    // 与 pages/donut-chart.at 的 Init 逐行同构（顶层直线形态，§0.6.H）：
    // 4 片圆环路径 + 图例。断言结构性质（起点/弧参数/大弧标志/闭合）。
    let code = r#"
        fn dc(x float) -> float { math.cos(x) }
        fn ds(x float) -> float { math.sin(x) }
        var a0_0 float = -1.5707963267948966
        var a1_0 float = 1.8849555921538763
        var a0_1 float = 1.8849555921538763
        var a1_1 float = 3.455751918948773
        var a0_2 float = 3.455751918948773
        var a1_2 float = 4.209734155810323
        var a0_3 float = 4.209734155810323
        var a1_3 float = 4.71238898038469

        var c00 float = dc(a0_0)
        var s00 float = ds(a0_0)
        var c01 float = dc(a1_0)
        var s01 float = ds(a1_0)
        var x00 float = 280.0 + 100.0 * c00
        var y00 float = 150.0 + 100.0 * s00
        var x01 float = 280.0 + 100.0 * c01
        var y01 float = 150.0 + 100.0 * s01
        var u00 float = 280.0 + 62.0 * c00
        var v00 float = 150.0 + 62.0 * s00
        var u01 float = 280.0 + 62.0 * c01
        var v01 float = 150.0 + 62.0 * s01
        var p0 = f"M${x00} ${y00} A100 100 0 1 1 ${x01} ${y01} L${u01} ${v01} A62 62 0 1 0 ${u00} ${v00} Z"

        var c10 float = dc(a0_1)
        var s10 float = ds(a0_1)
        var c11 float = dc(a1_1)
        var s11 float = ds(a1_1)
        var x10 float = 280.0 + 100.0 * c10
        var y10 float = 150.0 + 100.0 * s10
        var x11 float = 280.0 + 100.0 * c11
        var y11 float = 150.0 + 100.0 * s11
        var u10 float = 280.0 + 62.0 * c10
        var v10 float = 150.0 + 62.0 * s10
        var u11 float = 280.0 + 62.0 * c11
        var v11 float = 150.0 + 62.0 * s11
        var p1 = f"M${x10} ${y10} A100 100 0 0 1 ${x11} ${y11} L${u11} ${v11} A62 62 0 0 0 ${u10} ${v10} Z"

        var c20 float = dc(a0_2)
        var s20 float = ds(a0_2)
        var c21 float = dc(a1_2)
        var s21 float = ds(a1_2)
        var x20 float = 280.0 + 100.0 * c20
        var y20 float = 150.0 + 100.0 * s20
        var x21 float = 280.0 + 100.0 * c21
        var y21 float = 150.0 + 100.0 * s21
        var u20 float = 280.0 + 62.0 * c20
        var v20 float = 150.0 + 62.0 * s20
        var u21 float = 280.0 + 62.0 * c21
        var v21 float = 150.0 + 62.0 * s21
        var p2 = f"M${x20} ${y20} A100 100 0 0 1 ${x21} ${y21} L${u21} ${v21} A62 62 0 0 0 ${u20} ${v20} Z"

        var c30 float = dc(a0_3)
        var s30 float = ds(a0_3)
        var c31 float = dc(a1_3)
        var s31 float = ds(a1_3)
        var x30 float = 280.0 + 100.0 * c30
        var y30 float = 150.0 + 100.0 * s30
        var x31 float = 280.0 + 100.0 * c31
        var y31 float = 150.0 + 100.0 * s31
        var u30 float = 280.0 + 62.0 * c30
        var v30 float = 150.0 + 62.0 * s30
        var u31 float = 280.0 + 62.0 * c31
        var v31 float = 150.0 + 62.0 * s31
        var p3 = f"M${x30} ${y30} A100 100 0 0 1 ${x31} ${y31} L${u31} ${v31} A62 62 0 0 0 ${u30} ${v30} Z"

        var traffic = [
            { l: "Desktop", v: 55 },
            { l: "Mobile", v: 25 },
            { l: "Tablet", v: 12 },
            { l: "Wearable", v: 8 }
        ]
        var lg = ""
        for d in traffic {
            var vf = d["v"]
            var lbl = d["l"]
            if lg == "" {
                lg = f"${lbl} ${vf}%"
            } else {
                lg = lg + f"   ${lbl} ${vf}%"
            }
        }
        p0 + "|" + p1 + "|" + p2 + "|" + p3 + "~" + lg
    "#;
    let result = run(code).unwrap();
    let (paths, legend) = result.split_once('~').unwrap();
    let slices: Vec<&str> = paths.split('|').collect();
    assert_eq!(slices.len(), 4, "4 slices: {}", paths);

    // 片 0（55%，span>π）：从正顶 (280,50) 起，大弧标志 1
    let s0 = slices[0];
    assert!(s0.starts_with("M280 50 A100 100 0 1 1 "), "s0 = {}", s0);
    assert!(s0.ends_with("A62 62 0 1 0 280 88 Z"), "s0 tail = {}", s0);

    // 小弧片：laf=0
    assert!(slices[1].contains(" A100 100 0 0 1 "), "s1: {}", slices[1]);
    assert!(slices[2].contains(" A100 100 0 0 1 "), "s2: {}", slices[2]);
    assert!(slices[3].contains(" A100 100 0 0 1 "), "s3: {}", slices[3]);

    // 图例（含 {lbl} 字面量 bug 的回归防护）
    assert_eq!(legend, "Desktop 55%   Mobile 25%   Tablet 12%   Wearable 8%");
}



// §0.6.H 回归：CALL_NAT 直接在 for 循环体内 → 后续浮点算术损坏
// （最小复现：循环内 math.cos 后 cx+c 得位模式垃圾；用户函数包装幸存）。
// 根因修复后（见下方 test_geo_callnat_in_loop_regression），wrapper 与
// direct 两版语义一致，此测试保留双路径等价性断言。
#[test]
fn test_geo_callnat_in_loop_workaround() {
    let code = r#"
        fn dc(x float) -> float { math.cos(x) }
        var cx float = 280.0
        var data = [{ v: 55 }]
        var out = ""
        for d in data {
            var a float = -1.57079632679
            var c float = dc(a)
            var x float = cx + 100.0 * c
            out = f"${x}"
        }
        out
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "280");
}

// §0.6.H 根因回归（Plan 437 Charts 阶段收尾）：
// 三层缺陷修复——
//   ① decode_i64_full/u64_full 兜底 tag-first：f32/f64 tag 不再位读
//     （f32(280.0)=0x438C0000 被旧 decode_i32 读成 1133248512，
//      I64_TO_F64 后得位模式垃圾 1133248511.9999957）；
//   ② math 族目录返回类型 Void→Float：推断不再 Unknown 回退 int 路径；
//   ③ "math" 补入静态模块白名单：qualified 调用不再发射 const.i32 0
//     receiver 占位（shim 不消费 → 每次调用泄漏 +1 槽）。
// 顺带修复：f-string 内联 math.* 的 part tag 落 Int 缺口（tags=[1,3]）。

// ①引擎层单测：decode_i64_full 对 float tag 按数值截断，不做位读。
#[test]
fn test_geo_decode_i64_tag_first() {
    use auto_val::{encode_f32, encode_f64, encode_i32};
    // 需要一个 AutoVM 实例（BIGINT 分支才用堆；此处不触发）。
    let vm = crate::vm::engine::AutoVM::new(crate::vm::virt_memory::VirtualFlash::new(64), 64);
    assert_eq!(
        crate::vm::ffi::decode_i64_full(&vm, encode_f32(280.0)),
        280,
        "f32(280.0) 不得被位读为 0x438C0000=1133248512"
    );
    assert_eq!(
        crate::vm::ffi::decode_i64_full(&vm, encode_f64(-3.7)),
        -3,
        "f64 负值数值截断"
    );
    assert_eq!(
        crate::vm::ffi::decode_i64_full(&vm, encode_i32(42)),
        42,
        "i32 兼容路径不变"
    );
}

// ②+③端到端：循环体内直接 math.cos（无用户函数包装）——原失败用例。
#[test]
fn test_geo_callnat_in_loop_regression() {
    let code = r#"
        var cx float = 280.0
        var data = [{ v: 55 }]
        var out = ""
        for d in data {
            var a float = -1.57079632679
            var c float = math.cos(a)
            var x float = cx + 100.0 * c
            out = f"${x}"
        }
        out
    "#;
    assert_eq!(run(code).unwrap(), "280");
}

// ③泄漏回归：每次迭代多次原生调用 + 多迭代，sp 不得漂移累积。
#[test]
fn test_geo_callnat_multi_call_multi_iter() {
    let code = r#"
        var total = 0
        var last = ""
        for i in 0..200 {
            var c float = math.cos(0.0)
            var s float = math.sin(0.0)
            total = total + 1
            last = f"${c}|${s}"
        }
        f"${total}|${last}"
    "#;
    assert_eq!(run(code).unwrap(), "200|1|0");
}

// ②顺带修复：f-string 内联 math.*（原 known gap：part tag 落 Int，需 var 中转）。
#[test]
fn test_geo_fstr_inline_math() {
    let code = r#"f"cos0=${math.cos(0.0)} sin0=${math.sin(0.0)}""#;
    assert_eq!(run(code).unwrap(), "cos0=1 sin0=0");
}

// §0.6.H 残余症状核销：if 分支重赋值在循环内（原观察"corrupts ints"）。
#[test]
fn test_geo_if_reassign_in_loop() {
    let code = r#"
        var total = 0
        var data = [{ v: 1 }, { v: 2 }, { v: 3 }]
        for d in data {
            var n = d["v"]
            if n > 1 {
                n = n * 10
            }
            total = total + n
        }
        var acc float = 0.0
        for i in 0..3 {
            var x float = 1.5
            if i > 0 {
                x = x * 2.0
            }
            acc = acc + x
        }
        f"${total}|${acc}"
    "#;
    assert_eq!(run(code).unwrap(), "51|7.5");
}
