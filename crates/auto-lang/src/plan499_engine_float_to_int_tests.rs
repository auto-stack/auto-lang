/// Plan 499 M3 引擎修复回归:浮点接收者 `.to_int()`(CALL_SPEC 布局)。
/// 修复前:f64/f32 nanbox 接收者 type_name = "<unknown_nv:…>" 不匹配
/// 6872 分支前缀 → "CALL_SPEC: no function …to_int" 运行时错。
/// 经 build_dynamic_component 装配 → 随 498 模式挂 ui-iced 门
/// (裸 `cargo t`/`tf` 默认特性下模块整体配置出编译单元)。
#[cfg(feature = "ui-iced")]
#[test]
fn plan499_float_receiver_to_int_via_call_spec() {
    let src = r#"
widget W {
    msg { Init, Snap(float) }
    model {
        idx int = 0
        shown str = ""
    }
    view { col { text .shown {} } }
    on {
        .Init -> { }
        .Snap(x) -> {
            var fi float = x / 510.0 * 5.0 + 0.5
            var i int = fi.to_int()
            .idx = i
            .shown = f"i=${i}"
        }
    }
}
"#;
    let mut dc = crate::build_dynamic_component(src, None).expect("build");
    dc.fire_init();
    let (view, _, _) = dc.view_with_debug_gated(true);
    let _ = format!("{:?}", view); // 子 Init 经视图构建重放
    dc.on_with_input("Snap\u{1F}f\u{1F}145", None);
    let idx = dc.bridge_mut().read_state("idx").expect("idx");
    let shown = dc.bridge_mut().read_state("shown").expect("shown");
    // 145/510*5+0.5 = 1.92 → 截断 1
    assert!(
        matches!(idx, auto_val::Value::Int(i) if i == 1),
        "float .to_int() must truncate to 1, got {:?}",
        idx
    );
    assert!(
        matches!(shown, auto_val::Value::Str(ref s) if s.as_str() == "i=1"),
        "interpolation of converted int, got {:?}",
        shown
    );
}
