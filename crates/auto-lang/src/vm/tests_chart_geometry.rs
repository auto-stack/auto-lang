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
