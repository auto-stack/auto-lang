//! Plan 492 M5: 包组件编译失败的显式诊断——组件名+原因,替换静默回落。
//!
//! 两个静默面(484 现场排查成本高的根源):
//! ①装载层: `parse_package_widgets` per-file try-parse 失败仅存
//!   `LoadedPackage.parse_warnings`,两个消费方(lib.rs VM / api.rs vue)均
//!   不看——组件整文件无声消失;
//! ②合成层: handler `compile_stmt` 失败仅 eprintln(stderr,UI 运行期不可见)
//!   ——handler 不存在,调用期 "handler not found",组件回落 model 默认值。
//! 修复后两层均可查询/告警(log::warn + take_synth_failures)。

#[cfg(test)]
mod m5_pkg_diagnostics {
    /// 装载层: 坏文件(裸 prop 名 RHS → undefined variable)进包,
    /// load_package 成功但 parse_warnings 必须含文件名与原因,好组件照常注册。
    #[test]
    fn load_layer_parse_warnings_surface_bad_file() {
        let tmp = std::env::temp_dir().join("plan492-m5-pkg");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        // 坏组件: 裸 prop 名在赋值 RHS 位(M4 定案的硬错形态)。
        std::fs::write(
            tmp.join("bad_widget.at"),
            r##"
widget BadWidget (curve: str = "linear") {
    msg { Init }
    model { mark str = "" }
    on {
        .Init -> {
            .mark = curve
        }
    }
    view { col { text .mark { } } }
}
"##,
        )
        .unwrap();
        // 好组件。
        std::fs::write(
            tmp.join("good_widget.at"),
            r##"
widget GoodWidget {
    msg { Init }
    model { mark str = "" }
    on {
        .Init -> { .mark = "ok" }
    }
    view { col { text .mark { } } }
}
"##,
        )
        .unwrap();
        let mut reg = crate::ui_gen::widget::ComponentRegistry::new();
        let pkg = reg
            .load_package(&tmp, &tmp)
            .expect("package loads (good file present)");
        assert!(
            !pkg.parse_warnings.is_empty(),
            "bad file must surface in parse_warnings: {:?}",
            pkg.parse_warnings
        );
        let joined = pkg.parse_warnings.join("; ");
        assert!(
            joined.contains("bad_widget.at"),
            "warning must name the failing file: {joined}"
        );
        assert!(
            joined.to_lowercase().contains("undefined"),
            "warning must carry the reason: {joined}"
        );
        assert!(
            pkg.widgets.contains_key("goodwidget"),
            "good widget must still register: {:?}",
            pkg.widgets
        );
    }

    /// 合成/链接层: 包组件 Init 调用未定义函数 → 构建必须 FATAL 且错误带
    /// 模块名与符号名(响亮,不静默回落);synth 层诊断通道可取走(空=无
    /// per-handler 静默失败)。
    #[cfg(feature = "ui-iced")]
    #[test]
    fn synth_and_link_layer_failures_are_loud_and_named() {
        let front = crate::plan370_test_support::locate_example_app_at("charts-gallery")
            .expect("gallery")
            .parent()
            .expect("front")
            .to_path_buf();
        let tmp = std::env::temp_dir().join("plan492-m5-synth");
        let _ = std::fs::remove_dir_all(&tmp);
        crate::plan492_tests::pkg_harness::copy_tree_for_test(&front, &tmp).unwrap();
        let bar = tmp.join("components/bar_chart.at");
        let code = std::fs::read_to_string(&bar).unwrap();
        let patched = code.replace(
            ".Init -> {",
            ".Init -> {
            undefined_fn_xyz(1)
",
        );
        assert_ne!(patched, code, "canary anchor found");
        std::fs::write(&bar, patched).unwrap();
        let app = tmp.join("app.at");
        let app_code = std::fs::read_to_string(&app).unwrap();
        let build_err = crate::build_dynamic_component(&app_code, Some(app.to_str().unwrap()))
            .expect_err("undefined fn must fail the build loudly (not silent fallback)");
        let msg = format!("{build_err}");
        println!("build error: {msg}");
        assert!(
            msg.contains("undefined_fn_xyz"),
            "error must name the offending symbol: {msg}"
        );
        assert!(
            msg.contains("App"),
            "error must name the module: {msg}"
        );
        // synth 层诊断通道可取走(此向量死在 link 层,应为空)。
        let failures = crate::ui::handler_codegen::take_synth_failures();
        assert!(
            failures.iter().all(|f| !f.contains("undefined_fn_xyz")),
            "vector died at link layer; synth failures unexpected: {failures:?}"
        );
    }
}
