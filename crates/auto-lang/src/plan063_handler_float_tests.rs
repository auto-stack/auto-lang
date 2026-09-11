//! PLAN-063（auto-down）T-04b：handler 参数槽 f64 二元算术/比较坍缩回归锁。
//!
//! 实测签名（2026-09-11，demo/auto 实机 + 最小探针矩阵）：经
//! `call_handler_for`（iced ScrollCallback/onclick 实参 → `push_value`）派发
//! 的 float 实参，凡**参与二元运算或比较**即得 `int 0`/恒假；plain 赋值
//! （参数→state）与 state 字段运算完全正常。病征与 058 的 on_with_input
//! 路径（字符串解码 payload）互斥——058 锁全绿而实机坏，故本文件走
//! `call_widget_handler`（Value 实参 → push_value，demo 同构）。
//!
//! 各测试红 = 缺陷在现行 master 复现（转介单 DEBTS 063）；绿 = 已修复，
//! 本文件降级为回归锁（plan058_engine_gap_tests 同款口径）。

#[cfg(all(test, feature = "ui-iced"))]
mod plan063_handler_float_arith {
    use auto_val::Value;

    fn f_state(
        comp: &crate::ui::dynamic::DynamicComponent,
        field: &str,
    ) -> f64 {
        match comp.read_state(field).expect(field) {
            Value::Float(v) => v,
            Value::Double(v) => v,
            Value::Int(i) => i as f64,
            other => panic!("field {field}: expected numeric, got {other:?}"),
        }
    }

    /// 参数-参数减法（OnLeftScroll 的 `h - c` 同构）：期望 1881.36，实测 0。
    #[test]
    fn handler_param_minus_param_via_value_dispatch() {
        let src = r#"widget App {
    model {
        var a float = 0.0
    }
    msg Msg { Tri(float, float, float) }
    view { col {} {} }
    on {
        .Tri(x, y, z) -> {
            .a = x - y
        }
    }
}
"#;
        let mut comp = crate::build_dynamic_component(src, None).expect("compile");
        comp.call_widget_handler(
            "App",
            "Tri",
            &[Value::Float(2653.36), Value::Float(772.0), Value::Float(600.0)],
        )
        .expect("dispatch");
        let v = f_state(&comp, "a");
        assert!(
            (v - 1881.36).abs() < 0.01,
            "x - y with x=2653.36 y=772.0: expected ≈1881.36, got {v}"
        );
    }

    /// `f64` 拼写（demo app.at 实际用型）与复合级联全式：
    /// `.right_top = sy / (h - c) * (.right_height - .right_client)` 同构。
    #[test]
    fn handler_f64_spelling_full_cascade() {
        let src = r#"widget App {
    model {
        var lh float = 2653.36
        var lc float = 772.0
        var rh float = 3088.04
        var rc float = 772.0
        var out float = 0.0
    }
    msg Msg { Tri(f64, f64, f64) }
    view { col {} {} }
    on {
        .Tri(h, c, sy) -> {
            if h > c && .rh > .rc {
                .out = sy / (h - c) * (.rh - .rc)
            }
        }
    }
}
"#;
        let mut comp = crate::build_dynamic_component(src, None).expect("compile");
        comp.call_widget_handler(
            "App",
            "Tri",
            &[Value::Float(2653.36), Value::Float(772.0), Value::Float(600.0)],
        )
        .expect("dispatch");
        let v = f_state(&comp, "out");
        let expected = 600.0 / (2653.36 - 772.0) * (3088.04 - 772.0);
        assert!(
            (v - expected).abs() < 0.01,
            "full cascade: expected ≈{expected}, got {v}"
        );
    }

    /// 参数 float 比较（CustomScrollbar Move 守卫 `.scrollHeight > .clientHeight`
    /// 同构：参数/实参参与 `>` 比较）——期望真臂可达。
    #[test]
    fn handler_param_float_comparison() {
        let src = r#"widget App {
    model {
        var hit int = 0
    }
    msg Msg { Cmp(float, float) }
    view { col {} {} }
    on {
        .Cmp(a, b) -> {
            if a > b {
                .hit = 1
            }
        }
    }
}
"#;
        let mut comp = crate::build_dynamic_component(src, None).expect("compile");
        comp.call_widget_handler("App", "Cmp", &[Value::Float(2653.36), Value::Float(772.0)])
            .expect("dispatch");
        let v = f_state(&comp, "hit");
        assert_eq!(v, 1.0, "a > b with a=2653.36 b=772.0 must hold");
    }
}
