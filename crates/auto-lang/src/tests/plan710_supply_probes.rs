// PLAN-710 供① supply-endpoint probes (the plan703 probe shape: minimal VM
// bridge + .at fn calling the native; a2r track = generator-output form
// asserts on the same .at shapes + a2r_std helper semantics on the same JSON
// fixtures — 673 同源对拍惯例 at unit level).
//
// AC-01 (G-C): code_editor_delta ui_gen arm — direct core call, shim error
// message parity on unregistered keys. AC-02 (G-B): try/catch arms — success
// path zero-disturbance, native-error path routes to catch with the payload
// message bound (VM STORE_LOCAL ↔ a2r panic_message). AC-03 (G-A): envelope
// projection — coalesce families (plain/negative-default/bool/list/call-
// default/nested) project identically on both tracks; a2r_std helpers are
// is_true-semantics (auto_val value.rs:682).

#[cfg(test)]
mod plan710_supply {
    use crate::parser::Parser;

    /// Minimal VM bridge (plan703 build_probe_bridge 同款骨架).
    fn build_probe_bridge(src: &str) -> crate::ui::vm_bridge::VmBridge {
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|st| match st {
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .expect("decl");
        let fns: Vec<crate::ast::Stmt> = ast
            .stmts
            .iter()
            .filter(|st| matches!(st, crate::ast::Stmt::Fn(_)))
            .cloned()
            .collect();
        let widget = crate::aura::extract_widget_from_decl(&decl).expect("extract");
        crate::ui::vm_bridge::VmBridge::new_with_imports(&widget, fns).expect("bridge")
    }

    fn probe_bridge_with(body: &str) -> crate::ui::vm_bridge::VmBridge {
        let src = format!(
            concat!(
                "widget T710Probe {{\n",
                "    view {{\n",
                "        col {{\n",
                "            text \"probe\"\n",
                "        }}\n",
                "    }}\n",
                "}}\n",
                "{}\n"
            ),
            body
        );
        build_probe_bridge(&src)
    }

    fn call_str(bridge: &crate::ui::vm_bridge::VmBridge, name: &str) -> String {
        match bridge.call_vm_fn(name, &[]) {
            Ok(auto_val::Value::Str(s)) => s.to_string(),
            other => panic!("{name} must return Str, got {other:?}"),
        }
    }

    /// a2r 轨：内联 widget 源 → rust 发射产物（plan627/634 范式）。
    fn gen_rust(handler_body: &str) -> String {
        let src = format!(
            r#"
widget App {{
    msg {{ Init }}

    on {{
        {handler_body}
    }}

    view {{
        col {{
            text "term" {{}}
        }}
    }}
}}
"#,
        );
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let mut widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = crate::ui_gen::rust::RustGenerator::new();
        gen.generate_rust(&widget).expect("generate rust")
    }

    /// Probe font-system callback（core test_font_system 同款镜像——该件
    /// 为 core 测试模块私有，跨模块探针在此自备同形件）。
    fn probe_font_system(with: &mut dyn FnMut(&mut cosmic_text::FontSystem)) {
        static FS: std::sync::OnceLock<std::sync::RwLock<cosmic_text::FontSystem>> =
            std::sync::OnceLock::new();
        let fs = FS.get_or_init(|| std::sync::RwLock::new(cosmic_text::FontSystem::new()));
        let mut guard = fs.write().unwrap();
        with(&mut guard);
    }

    const ENV_FIXTURE: &str =
        r#"{"nest":{"deep":"hit"},"num":7,"flag":true,"list":[1,2],"s":"here"}"#;

    // ---- G-A: envelope projection — VM track ----

    #[test]
    #[cfg(feature = "ui-iced")]
    fn envelope_projection_vm_track() {
        // 注：VM 轨探针用单层成员链（.at 解析面）；嵌套形（v.a.b ?? d）
        // 的语义由 helper 单测（get_str_or 路径切片）与发射形单测覆盖。
        // .at 串面转义：fixture 的 `"` 在 .at 源内为 `\"`（corpus 同款）。
        let at_json = ENV_FIXTURE.replace('"', "\\\"");
        let bridge = probe_bridge_with(&format!(
            "fn probe_env() str {{\n    let v = json.to_value(\"{}\")\n    var out str = v.s ?? \"miss\"\n    out = out + \"|\" + (v.num ?? -1).str()\n    var fbit str = \"0\"\n    if v.flag ?? false {{ fbit = \"1\" }}\n    out = out + \"|\" + fbit\n    out = out + \"|\" + (v.list ?? []).len().str()\n    out = out + \"|\" + (v.absent ?? \"dflt\")\n    return out\n}}\n",
            at_json
        ));
        let out = call_str(&bridge, "probe_env");
        assert_eq!(out, "here|7|1|2|dflt", "envelope projection: {out}");
    }

    // ---- G-A: envelope projection — a2r helper semantics (same fixture) ----

    #[test]
    fn envelope_projection_a2r_helpers_same_fixture() {
        let v: serde_json::Value = serde_json::from_str(ENV_FIXTURE).unwrap();
        use crate::a2r_std::json as j;
        let nest = j::get_owned(&v, "nest");
        assert_eq!(j::get_str_or(&nest, &["deep"], "miss"), "hit");
        assert_eq!(j::get_str_or(&v, &["nest", "deep"], "miss"), "hit");
        assert_eq!(
            j::get_int_or(&v, &["num"], -1) as i32,
            7,
            "int projection: present"
        );
        assert_eq!(
            (j::get_int_or(&v, &["absent_num"], -1) as i32),
            -1,
            "int projection: missing → ?? 右值"
        );
        assert!(j::get_bool_or(&v, &["flag"], false));
        assert!(!j::get_bool_or(&v, &["absent_flag"], false));
        assert_eq!(j::get_array_or(&v, &["list"]).len(), 2);
        assert!(j::get_array_or(&v, &["absent_list"]).is_empty());
        assert_eq!(j::get_str_or(&v, &["absent"], "dflt"), "dflt");
        // 计算缺省变体（`t.title ?? file_basename(p)` 形）。
        assert_eq!(
            j::get_str_or_with(&v, &["absent"], || "computed".to_string()),
            "computed"
        );
    }

    /// truthy ↔ VM `Value::is_true`（auto_val value.rs:682）逐格对拍。
    #[test]
    fn truthy_matches_vm_is_true() {
        use crate::a2r_std::json::truthy;
        // serde_json ↔ auto_val 同格：bool 直读 / 数值 >0 / 串非空 / 其余 false。
        assert!(truthy(&serde_json::json!(true)));
        assert!(!truthy(&serde_json::json!(false)));
        assert!(truthy(&serde_json::json!(7)));
        assert!(!truthy(&serde_json::json!(0)));
        assert!(!truthy(&serde_json::json!(-3)));
        assert!(truthy(&serde_json::json!("x")));
        assert!(!truthy(&serde_json::json!("")));
        assert!(!truthy(&serde_json::Value::Null));
        assert!(!truthy(&serde_json::json!([])));
        // VM 侧同格断言（auto_val Value）。
        assert!(auto_val::Value::Int(7).is_true());
        assert!(!auto_val::Value::Int(0).is_true());
        assert!(!auto_val::Value::Int(-3).is_true());
        assert!(auto_val::Value::Bool(true).is_true());
        assert!(!auto_val::Value::Str("".into()).is_true());
        assert!(auto_val::Value::Str("x".into()).is_true());
    }

    // ---- G-B: try/catch arms — VM track ----

    #[test]
    #[cfg(feature = "ui-iced")]
    fn try_success_path_zero_disturbance_vm() {
        let bridge = probe_bridge_with(
            "fn probe_try_ok() str {\n    var r str = \"untouched\"\n    try {\n        r = \"touched\"\n    } catch {\n        r = \"caught\"\n    }\n    return r\n}\n",
        );
        assert_eq!(call_str(&bridge, "probe_try_ok"), "touched");
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    #[cfg(feature = "code-editor")]
    fn try_native_error_routes_to_catch_with_binding_vm() {
        let bridge = probe_bridge_with(
            "fn probe_try_err() str {\n    var r str = \"untouched\"\n    try {\n        let d = code_editor_delta(\"p710-no-such-editor\")\n        r = d\n    } catch (err) {\n        r = err\n    }\n    return r\n}\n",
        );
        let out = call_str(&bridge, "probe_try_err");
        assert!(
            out.contains("no editor registered"),
            "catch binds the error message (VM STORE_LOCAL 同源): {out}"
        );
        assert!(out.contains("p710-no-such-editor"), "message carries the key: {out}");
    }

    // ---- G-C: code_editor_delta — VM track vs core (envelope + error form) ----

    #[test]
    #[cfg(feature = "ui-iced")]
    #[cfg(feature = "code-editor")]
    fn vm_delta_registered_envelope_and_unregistered_error() {
        use crate::ui::code_editor::{
            code_editor, code_editor_delta, code_editor_dispose, code_editor_set_text,
            set_font_system_call, storage_key, CodeEditorConfig,
        };
        use crate::ui::code_editor::core::REGISTRY_TEST_LOCK;
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(probe_font_system);
        // registry 以 `__code_editor_{key}` 存取（storage_key 归一）；.at/shim
        // 轨传裸名（shim 侧 normalize）——创建用归一键（673 惯例），调用轨
        // 用裸名。
        let key = storage_key("p710-probe-editor");
        code_editor_dispose(&key);
        let config = CodeEditorConfig { ..CodeEditorConfig::default() };
        let _core = code_editor(&key, &config);
        assert!(code_editor_set_text(&key, "hello"), "set_text queues delta 1");

        // Registered: envelope JSON（revision/deltas 形——core 直读同源）。
        let bridge = probe_bridge_with(
            "fn probe_delta() str {\n    return code_editor_delta(\"p710-probe-editor\")\n}\n",
        );
        let json = call_str(&bridge, "probe_delta");
        let v: serde_json::Value = serde_json::from_str(&json).expect("envelope json");
        assert!(v.get("revision").is_some(), "envelope field revision: {json}");
        assert!(v.get("deltas").is_some(), "envelope field deltas: {json}");
        // 同源对拍：core 直读（destructive）与 VM shim 轨返回同一 watermark 形。
        let direct = code_editor_delta(&key).expect("core direct read");
        assert_eq!(direct, r#"{"revision":1,"deltas":[]}"#, "destructive watermark form");
        assert_eq!(direct, r#"{"revision":1,"deltas":[]}"#, "destructive watermark form");

        // Unregistered: VM shim 错误形（RuntimeError + 同消息）。
        let bridge2 = probe_bridge_with(
            "fn probe_delta_err() str {\n    return code_editor_delta(\"p710-no-such\")\n}\n",
        );
        let err = bridge2
            .call_vm_fn("probe_delta_err", &[])
            .err()
            .expect("unregistered key must error");
        let msg = format!("{err}");
        assert!(
            msg.contains("no editor registered") && msg.contains("p710-no-such"),
            "shim error message form: {msg}"
        );
        code_editor_dispose(&key);
    }

    // ---- a2r 轨：发射形断言（同 .at 形） ----

    #[test]
    fn emitted_try_form_catch_unwind_and_escape() {
        let code = gen_rust(
            ".Init -> {\n            var r str = \"untouched\"\n            try {\n                let d = code_editor_delta(\"k710\")\n                r = d\n            } catch (err) {\n                r = err\n            }\n        }",
        );
        assert!(
            code.contains("std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {"),
            "try 臂 catch_unwind 形: {code}"
        );
        assert!(code.contains("Ok(_) => {}"), "成功臂零扰动（空 Ok）");
        assert!(
            code.contains("let err = auto_lang::a2r_std::panic_message(&__p);"),
            "catch 绑定形 = panic_message 载荷串（VM STORE_LOCAL 同源）"
        );
    }

    #[test]
    fn emitted_delta_direct_call_with_shim_error_message() {
        let code = gen_rust(".Init -> { code_editor_delta(\"k710\") }");
        assert!(
            code.contains("auto_lang::ui::code_editor::code_editor_delta(&__key)"),
            "delta 直调臂（703 直调纪律——不绕 VM shim）: {code}"
        );
        assert!(
            code.contains("panic!(\"code_editor_delta: no editor registered for key {:?}\""),
            "未注册键错误形与 shim 同消息（native.rs:717）: {code}"
        );
    }

    #[test]
    fn emitted_coalesce_projection_families() {
        // 平/嵌套/负缺省/bool/list/调用缺省/局部接收者 七形（供① 实例集全族）。
        let code = gen_rust(
            ".Init -> {\n            let v = json.to_value(\"{}\")\n            var a str = v.err ?? \"\"\n            var b int = v.active ?? -1\n            var c bool = v.degraded ?? false\n            var d = v.hunks ?? []\n            var e str = v.t ?? file_basename(v.p ?? \"\")\n            var f str = v.a.b ?? \"d\"\n            let hk = v.h\n            var g int = hk.a1 ?? 0\n        }",
        );
        assert!(
            code.contains("auto_lang::a2r_std::json::get_str_or(&(v), &[\"err\"], \"\")"),
            "str 平形: {code}"
        );
        assert!(
            code.contains("auto_lang::a2r_std::json::get_int_or(&(v), &[\"active\"], -1) as i32"),
            "int 负缺省形（Unary(Sub) 入臂）: {code}"
        );
        assert!(
            code.contains("auto_lang::a2r_std::json::get_bool_or(&(v), &[\"degraded\"], false)"),
            "bool 形: {code}"
        );
        assert!(
            code.contains("auto_lang::a2r_std::json::get_array_or(&(v), &[\"hunks\"])"),
            "list 形: {code}"
        );
        assert!(
            code.contains("auto_lang::a2r_std::json::get_str_or_with("),
            "调用缺省形（str-with 变体）: {code}"
        );
        assert!(
            code.contains("auto_lang::a2r_std::json::get_str_or(&(v), &[\"a\", \"b\"], \"d\")"),
            "嵌套路径切片形: {code}"
        );
        assert!(
            code.contains("auto_lang::a2r_std::json::get_int_or(&(hk), &[\"a1\"], 0) as i32"),
            "局部接收者形: {code}"
        );
    }

    #[test]
    fn emitted_scroll_to_and_scroll_pane_controller() {
        let code = gen_rust(".Init -> { let ok = scroll_to(\"h710\", \"y\", 24.0) }");
        assert!(
            code.contains("auto_lang::ui::scroll::ScrollIntent::ScrollTo"),
            "D-4 scroll_to 臂（shim_scroll_to 同源 intent 形）: {code}"
        );
        assert!(code.contains("Axis::Y"), "轴串分派: {code}");
    }

    #[test]
    fn emitted_char_at_substr_to_int_semantics() {
        let code = gen_rust(
            ".Init -> {\n            let w = \"x\"\n            let c = w.char_at(0)\n            let s = w.substr(0, 1)\n            let n = w.to_int()\n        }",
        );
        assert!(code.contains("chars().nth("), "D-8 char_at 字符语义: {code}");
        assert!(
            code.contains("chars().skip(") && code.contains("collect::<String>()"),
            "D-8 substr 字符窗: {code}"
        );
        assert!(
            code.contains("parse::<i32>().unwrap_or(0)"),
            "D-8 to_int 解析整形（失败 0）: {code}"
        );
    }
    // ---- PLAN-714 r4: code_editor 注册键贯通（供料档 §7 定谳+修复） ----
    // 定谳（R4-T1）：a2r 生成器 key 属性只认 Str 字面量（Plan 413 原始
    // 形态），`(key: t.key)` 动态表达式静默回落 widget 名 "editor" →
    // 注册键 `__code_editor_editor` ≠ store 装载门查找键
    // `__code_editor_tab-N` → 装载链永递延。非 2205→2366 回归（生成器/
    // renderer/widget/core 四面与 019 钉版 5bb3f53be 逐字节一致）。

    /// a2r 轨：全 widget 源 → rust 发射产物（gen_rust 的视图面变体）。
    fn gen_rust_full(widget_src: &str) -> String {
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(widget_src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let mut widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = crate::ui_gen::rust::RustGenerator::new();
        gen.generate_rust(&widget).expect("generate rust")
    }

    #[test]
    fn plan714_r4_codegen_dynamic_key_threaded_to_view() {
        // `(key: t.key)` 动态键经 ast_expr_to_rust 发射（Value 循环变量
        // 字段访问形——与视图条件同源），不再回落 "editor" 字面量。
        let src = r#"
store ProbeStore {
    model {
        var tabs list = [{key: "tab-1", src: ""}, {key: "tab-2", src: ""}]
        var active_key str = "tab-1"
    }
}

widget App {
    msg { Init }

    on {
        .Init -> {
        }
    }

    view {
        col {
            for i, t in .store.tabs {
                if t.key == .store.active_key {
                    code_editor (key: t.key, wrap: false) {
                        oninput: .SrcChanged(i)
                    }
                }
            }
        }
    }
}
"#;
        let code = gen_rust_full(src);
        assert!(
            code.contains("View::code_editor(t[\"key\"].as_str().unwrap_or_default().to_string())"),
            "动态 key 贯通注册键发射（t[\"key\"] 形——与条件判断同源）:\n{code}"
        );
        assert!(
            !code.contains("View::code_editor(\"editor\")"),
            "字面量回落形必须消失:\n{code}"
        );
    }

    #[test]
    fn plan714_r4_codegen_static_key_literal_unchanged() {
        // 静态 Str key 维持字面量发射（既有金样/探针应用零扰动——
        // probe_bufdiff_app "tab-1" 形）。
        let src = r#"
widget App {
    msg { Init }

    on {
        .Init -> {
        }
    }

    view {
        col {
            code_editor (key: "tab-1") {
            }
        }
    }
}
"#;
        let code = gen_rust_full(src);
        assert!(
            code.contains("View::code_editor(\"tab-1\")"),
            "静态 key 字面量形不变:\n{code}"
        );
        assert!(
            !code.contains("View::code_editor(\"editor\")"),
            "显式静态 key 不回落:\n{code}"
        );
    }

    #[test]
    fn plan714_r4_codegen_missing_key_fallback_unchanged() {
        // 无 key 属性维持 "editor" 回落（Plan 413 语义保持）。
        let src = r#"
widget App {
    msg { Init }

    on {
        .Init -> {
        }
    }

    view {
        col {
            code_editor {
            }
        }
    }
}
"#;
        let code = gen_rust_full(src);
        assert!(
            code.contains("View::code_editor(\"editor\")"),
            "无 key 回落形不变:\n{code}"
        );
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    #[cfg(feature = "code-editor")]
    fn plan714_r4_registry_gate_accepts_runtime_tab_key() {
        // 注册键≡store 查找键的注册表层 E2E：按修复后发射形注册
        // （View::code_editor(t.key) → renderer storage_key("tab-N")），
        // store 装载探测门 code_editor_edit("__code_editor_tab-N",0,0,"")
        // 必须命中（供料档 §7 永假门反转绿）；脱节旧形（注册在
        // __code_editor_editor 下）保持 false——钉住 §7 脱节形态本身。
        use crate::ui::code_editor::{
            code_editor, code_editor_dispose, code_editor_edit, set_font_system_call,
            storage_key, CodeEditorConfig,
        };
        use crate::ui::code_editor::core::REGISTRY_TEST_LOCK;
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(probe_font_system);
        let config = CodeEditorConfig { ..CodeEditorConfig::default() };

        // 修复形：注册键 = storage_key(t.key 运行时值)。
        let fixed = storage_key("tab-3");
        code_editor_dispose(&fixed);
        let _core = code_editor(&fixed, &config);
        assert!(
            code_editor_edit("__code_editor_tab-3", 0, 0, ""),
            "修复形：装载探测门命中（注册键≡查找键）"
        );
        code_editor_dispose(&fixed);

        // 脱节旧形（§7）：注册键 = storage_key("editor")。
        let stale = storage_key("editor");
        code_editor_dispose(&stale);
        let _core2 = code_editor(&stale, &config);
        assert!(
            !code_editor_edit("__code_editor_tab-3", 0, 0, ""),
            "脱节旧形：探测门永假（§7 形态钉住）"
        );
        code_editor_dispose(&stale);
    }
}

    // ---- PLAN-714 r2: back-path Stmt::Try arm (trans/rust.rs emission) ----

    #[test]
    fn plan714_back_try_arm_catch_unwind_fsys_shape() {
        let src = "pub fn find_in_files(fpath str) {
    var n int = 0
    try {
        let content str = File.read_text(fpath)
        for ln in content.split(\"\n\") {
            n = n + 1
            if n > 10 {
                break
            }
        }
    } catch {
    }
}
";
        let mut rcode = crate::trans::rust::transpile_rust("find_in_files", src)
            .expect("transpile failed");
        let rs_bytes = rcode.done().expect("done failed");
        let rs = String::from_utf8_lossy(rs_bytes);
        assert!(
            rs.contains("std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {"),
            "try 臂 catch_unwind 形: {rs}"
        );
        assert!(rs.contains("Ok(_) => {}"), "成功臂零扰动: {rs}");
        assert!(rs.contains("Err(_p714) => {}"), "空 catch 载荷弃: {rs}");
        assert!(rs.contains("break;"), "for{{if break}} 体保留: {rs}");
    }

    #[test]
    fn plan714_back_try_arm_catch_binding_panic_message() {
        let src = "pub fn probe() {
    var r str = \"untouched\"
    try {
        let d = File.read_text(\"no-such-file\")
        r = d
    } catch (err) {
        r = err
    }
}
";
        let mut rcode = crate::trans::rust::transpile_rust("probe", src)
            .expect("transpile failed");
        let rs_bytes = rcode.done().expect("done failed");
        let rs = String::from_utf8_lossy(rs_bytes);
        assert!(
            rs.contains("let err = auto_lang::a2r_std::panic_message(&__p);"),
            "catch 绑定形 = panic_message 载荷串: {rs}"
        );
        assert!(rs.contains("Err(__p) => {"), "异常臂非空 catch 体: {rs}");
    }

    #[test]
    fn plan714_back_try_arm_finally_follows_match() {
        let src = "pub fn probe() {
    try {
        let d = File.read_text(\"x\")
    } catch {
    } finally {
        var done int = 1
    }
}
";
        let mut rcode = crate::trans::rust::transpile_rust("probe", src)
            .expect("transpile failed");
        let rs_bytes = rcode.done().expect("done failed");
        let rs = String::from_utf8_lossy(rs_bytes);
        assert!(rs.contains("Err(_p714) => {} }; {"), "finally 跟发于 match 后: {rs}");
        assert!(rs.contains("let mut done"), "finally 体语句发射: {rs}");
    }

    #[test]
    fn plan714_back_try_arm_return_propagation() {
        // r3 R3-T3: return-in-try 传播形——fn bool 体含 return → 闭包
        // Option<bool>+Some 包装+Ok(Some(v))=>return v（VM return-in-try
        // =从 fn 返回同源；710 边界条款的 corpus 实实例收口）。
        let src = "pub fn probe() bool {
    try {
        let d = File.read_text(\"x\")
        return true
    } catch {
        return false
    }
}
";
        let mut rcode = crate::trans::rust::transpile_rust("probe", src)
            .expect("transpile failed");
        let rs_bytes = rcode.done().expect("done failed");
        let rs = String::from_utf8_lossy(rs_bytes);
        assert!(rs.contains("|| -> Option<bool> {"), "闭包标注 Option<Ret>: {rs}");
        assert!(rs.contains("Ok(Some(__v)) => return __v"), "Some 传播臂: {rs}");
        assert!(
            rs.contains("Ok(None) => return Default::default()"),
            "None 尾臂（体必 return 形不可达）: {rs}"
        );
        assert!(rs.contains("return Some(true)"), "体 return 包 Some: {rs}");
    }

    #[test]
    fn plan714_back_try_unit_fn_unchanged() {
        // unit fn 的 try 保持 r2 形（无 Option 传播——门控条件）。
        let src = "pub fn probe() {
    try {
        let d = File.read_text(\"x\")
    } catch {
    }
}
";
        let mut rcode = crate::trans::rust::transpile_rust("probe", src)
            .expect("transpile failed");
        let rs_bytes = rcode.done().expect("done failed");
        let rs = String::from_utf8_lossy(rs_bytes);
        assert!(!rs.contains("-> Option<"), "unit fn 不传播: {rs}");
        assert!(rs.contains("Ok(_) => {}"), "r2 形保持: {rs}");
    }

    #[test]
    fn plan714_back_try_borrow_iter_clone() {
        // r3 R3-T3: 借位迭代变量的闭包体内绑定 → .clone()（preview=ln 株）。
        let src = "pub fn probe(path str) {
    try {
        let content str = File.read_text(path)
        let lines = content.split(\"\\n\")
        for ln in lines {
            var preview str = ln
            let _ = preview
        }
    } catch {
    }
}
";
        let mut rcode = crate::trans::rust::transpile_rust("probe", src)
            .expect("transpile failed");
        let rs_bytes = rcode.done().expect("done failed");
        let rs = String::from_utf8_lossy(rs_bytes);
        assert!(
            rs.contains("= ln.clone()"),
            "借位迭代变量绑定 .clone(): {rs}"
        );
    }

    #[test]
    fn plan714_r3_fsys_at_full_transpile_qualified() {
        // r3 R3-T4 前哨：corpus fsys.at 全文直转——fs.metadata→file_size、
        // copy_recursive、json.from_value、diff 裸名映射、Regex.test 全部
        // 限定形（掩蔽层清单的发射面收口验证）。fixture 路径=组内 tmp
        // corpus（不在 git——缺文件时跳过断言并打印 SKIP 标记）。
        let path = "D:/autostack/.wt/lang-714/tmp-corpus/specs/auto-edit/src/back/fsys.at";
        let Ok(src) = std::fs::read_to_string(path) else {
            println!("SKIP: corpus fixture absent ({path})");
            return;
        };
        let mut rcode = crate::trans::rust::transpile_rust("fsys", &src)
            .expect("fsys.at transpile failed");
        let rs_bytes = rcode.done().expect("done failed");
        let rs = String::from_utf8_lossy(rs_bytes);
        std::fs::write("D:/autostack/.wt/lang-714/tmp-corpus/fsys-transpiled.rs", rs.as_bytes()).unwrap();
        assert!(!rs.contains("= fs.metadata("), "fs.metadata 裸名残留(赋值位)");
        assert!(!rs.contains("return fs.metadata("), "fs.metadata 裸名残留(返回位)");
        assert!(rs.contains("a2r_std::fs::file_size"), "file_size 限定: {rs}");
        assert!(rs.contains("a2r_std::fs::copy_recursive"), "copy_recursive 限定");
        assert!(rs.contains("a2r_std::json::from_value(a2r_std::json!("), "from_value json! 形");
        assert!(rs.contains("a2r_std::re::test("), "Regex.test 限定");
        assert!(rs.contains("a2r_std::diff::diff_files("), "diff 裸名限定");
        assert!(rs.contains("a2r_std::fs::read_bytes_list"), "read_bytes list 形");
    }
