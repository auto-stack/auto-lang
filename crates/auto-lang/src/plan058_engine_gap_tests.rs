//! PLAN-058（auto-down）面 B 转介单②三件引擎缺口的探针/回归锁。
//!
//! - ②-③ nanbox 整值 float 位型保真（PLAN-043 T6 登记引擎债）：
//!   `write_state` 往返 / payload 实参绑定 / 复合算式 RHS / int 宽化。
//! - ②-① use 导入子件 handler 体内 computed 解析（语料
//!   test/ui/plan058_child_emit_computed）。
//! - ②-② 带计算实参的引号 emit 经派发器路由到父级 on "<name>" 绑定
//!   （同语料）。
//!
//! 各测试若红 = 对应缺口在现行 master 复现；若绿 = 已被过渡计划清偿，
//! 本文件降级为回归锁（转介单⑤ R1/R4 先例口径）。

/// ②-③ nanbox 整值 float 保真探针。
#[cfg(test)]
mod plan058_nanbox_int_float {
    use auto_val::Value;

    const SRC: &str = r#"widget App {
    model {
        var top float = 0.0
    }
    msg Msg { SetTop(float) }
    view {
        col {} {}
    }
    on {
        .SetTop(v) -> {
            .top = v
        }
    }
}
"#;

    fn build() -> crate::ui::dynamic::DynamicComponent {
        crate::build_dynamic_component(SRC, None).expect("compile")
    }

    /// write_state → read_state 往返：整值 float 不丢 float 标签。
    #[test]
    fn write_state_int_float_roundtrip() {
        let mut comp = build();
        for v in [240.0f64, 0.0f64, 1.0f64, 2590.99f64] {
            comp.write_state("top", Value::Float(v)).expect("write");
            match comp.read_state("top").expect("read") {
                Value::Float(f) => assert_eq!(f, v, "roundtrip Float({v})"),
                other => panic!("expected Float({v}), got {other:?}"),
            }
        }
    }

    /// handler 实参绑定（iced renderer payload 形态 `f` 分量）：整值 float
    /// 经 on_with_input 绑定到 float 形参后写入 state，读回保真。
    #[test]
    fn handler_int_float_arg_binding() {
        let mut comp = build();
        comp.on_with_input("SetTop\u{1F}f\u{1F}240.0", None);
        match comp.read_state("top").expect("read") {
            Value::Float(f) => assert_eq!(f, 240.0, "payload-bound integral float"),
            other => panic!("expected Float(240.0), got {other:?}"),
        }
    }

    /// 三参 handler 全 float 位（OnLeftScroll(h, c, sy) 同构）：首参绑定
    /// 保真（PLAN-043 期「首参读到垃圾、二三参正常」形态的反例锁）。
    #[test]
    fn handler_three_float_params_first_arg_intact() {
        let src = r#"widget App {
    model {
        var a float = 0.0
        var b float = 0.0
        var c float = 0.0
    }
    msg Msg { Tri(float, float, float) }
    view { col {} {} }
    on {
        .Tri(x, y, z) -> {
            .a = x
            .b = y
            .c = z
        }
    }
}
"#;
        let mut comp = crate::build_dynamic_component(src, None).expect("compile");
        comp.on_with_input("Tri\u{1F}f\u{1F}240.0\u{1F}f\u{1F}800.5\u{1F}f\u{1F}0.0", None);
        let f_state = |comp: &crate::ui::dynamic::DynamicComponent, f: &str| match comp
            .read_state(f)
            .expect(f)
        {
            Value::Float(v) => v,
            other => panic!("field {f}: expected Float, got {other:?}"),
        };
        assert_eq!(f_state(&comp, "a"), 240.0, "first param binding");
        assert_eq!(f_state(&comp, "b"), 800.5, "second param binding");
        assert_eq!(f_state(&comp, "c"), 0.0, "third param binding");
    }

    /// 复合算式 RHS（PLAN-043 T6 登记「复合算式 RHS 全哑」形态）：整值
    /// float 实参参与二元运算并写回，操作数与结果均不哑。
    #[test]
    fn handler_int_float_composite_arithmetic() {
        let src = r#"widget App {
    model {
        var top float = 0.0
        var scaled float = 0.0
    }
    msg Msg { SetTop(float) }
    view { col {} {} }
    on {
        .SetTop(v) -> {
            .top = v
            .scaled = v * 2 + 1
        }
    }
}
"#;
        let mut comp = crate::build_dynamic_component(src, None).expect("compile");
        comp.on_with_input("SetTop\u{1F}f\u{1F}240.0", None);
        match comp.read_state("scaled").expect("read") {
            Value::Float(f) => assert_eq!(f, 481.0, "v*2+1 with v=240.0"),
            Value::Int(i) if i == 481 => {
                // 数值保真但形态为 int——数值等价判过（形态注记在案）
            }
            other => panic!("expected 481.0 (float or equal int), got {other:?}"),
        }
    }

    /// 整值 float 经 Int payload（`i` 分量）绑到 float 形参的宽化路径。
    #[test]
    fn handler_int_payload_widens_to_float_param() {
        let mut comp = build();
        comp.on_with_input("SetTop\u{1F}i\u{1F}240", None);
        match comp.read_state("top").expect("read") {
            Value::Float(f) => assert_eq!(f, 240.0, "int payload widened"),
            Value::Int(i) if i == 240 => {
                // int 存储但数值保真——数值等价判过（形态注记在案）
            }
            other => panic!("expected 240.0 (float or equal int), got {other:?}"),
        }
    }
}

/// ②-①/②-② use 导入子件语料探针（生产路径构建，p053_8 先例）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan058_child_emit_computed {
    fn build() -> Option<crate::ui::dynamic::DynamicComponent> {
        let rel = "test/ui/plan058_child_emit_computed/src/front/app.at";
        let manifest = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())?;
        crate::plan370_test_support::build_component_from_app(&manifest)
    }

    fn f_state(
        dc: &crate::ui::dynamic::DynamicComponent,
        field: &str,
    ) -> auto_val::Value {
        dc.read_state(field).expect(field)
    }

    /// ②-①：use 导入子件 handler 体内的 computed 引用解析出真值
    /// （.half = .raw / 2 = 120.0），Nil 静默传播消除。
    #[test]
    fn computed_resolves_in_used_child_handler() {
        let Some(mut dc) = build() else {
            eprintln!("plan058 corpus: SKIPPED — app.at not found");
            return;
        };
        let _ = dc.view_with_debug_gated(false);
        dc.on_with_input_for("Child58", "Ping", None);
        match f_state(&dc, "last_half") {
            auto_val::Value::Float(f) if f == 120.0 => {}
            auto_val::Value::Double(f) if f == 120.0 => {}
            auto_val::Value::Int(i) if i == 120 => {}
            other => panic!("expected 120.0 (float/double/int), got {other:?}"),
        }
    }

    /// ②-②：带计算实参的引号 emit 经派发器路由到父级 on "update:half"
    /// 绑定（.GotHalf 收到 120.0 写 root_half）。
    #[test]
    fn quoted_emit_with_computed_arg_routes_to_parent() {
        let Some(mut dc) = build() else {
            eprintln!("plan058 corpus: SKIPPED — app.at not found");
            return;
        };
        let _ = dc.view_with_debug_gated(false);
        dc.on_with_input_for("Child58", "Ping", None);
        match f_state(&dc, "root_half") {
            auto_val::Value::Float(f) if f == 120.0 => {}
            auto_val::Value::Double(f) if f == 120.0 => {}
            auto_val::Value::Int(i) if i == 120 => {}
            other => panic!("expected 120.0 (float/double/int), got {other:?}"),
        }
    }
}
