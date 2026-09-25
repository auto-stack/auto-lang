// PLAN-701 供④/供⑥ supply-endpoint probes.
//
// 供④ time 族三面对拍（GOAL-003）：catalog 1200/1201/1205 已登记但长期
// 无运行时 shim（裸/use 两形态均返 0，m1-supply §9 登记）——本探针钉
// VM 轨实装语义与 a2r-std/src/time.rs 一致（Unix epoch 钟值，非单调钟；
// .at stdlib time.at/time.rs.at 文档注释口径）。tolerance=5s（两腿分别
// 取时，进程内串行远小于此）。
//
// 供⑥ shell_add_recent：签名面冒烟（empty path → false，零 shell 副作
// 用面）；真调用冒烟属下游 E2E（跳转列表为用户桌面面，测试不污染用户
// Recent 列表——probe_jumplist_report 裁定节的动态面同口径）。

#[cfg(test)]
mod plan701_time_supply {
    /// Minimal VM bridge (musk_vm_track build_bridge 同款骨架).
    fn build_probe_bridge(src: &str) -> crate::ui::vm_bridge::VmBridge {
        use crate::parser::Parser;
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

    const PROBE_SRC: &str = concat!(
        "widget T701Probe {\n",
        "    view {\n",
        "        col {\n",
        "            text \"probe\"\n",
        "        }\n",
        "    }\n",
        "}\n",
        // 全宽毫秒值经字符串出口对拍（桥面 Value::Int=i32，且 .at 层
        // TAG_I64 算术不可靠[预存]——下游 bench 用值为字符串/状态透传形，
        // 不涉 .at 内算术，与本探针同形）。
        "fn probe_ms() i64 {\n",
        "    return time.now_ms()\n",
        "}\n",
        "fn probe_sec() int {\n",
        "    return time.now_sec()\n",
        "}\n",
        "fn probe_now() str {\n",
        "    return time.now()\n",
        "}\n",
    );

    /// Bridge-side Int|I64 acceptor (now_sec fits i32 today but is pushed
    /// via the i64 lane — the tag, not the magnitude, decides the variant).
    fn as_int_i64(v: auto_val::Value) -> i64 {
        match v {
            auto_val::Value::Int(i) => i as i64,
            auto_val::Value::I64(i) => i,
            other => panic!("expected integer value, got {other:?}"),
        }
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn vm_time_now_ms_full_width_matches_a2r_std() {
        let bridge = build_probe_bridge(PROBE_SRC);
        let vm_ms = as_int_i64(
            bridge
                .call_vm_fn("probe_ms", &[])
                .expect("call probe_ms"),
        );
        let a2r_ms = a2r_std::time::now_ms();
        assert!(
            vm_ms >= 1_672_531_200_000,
            "time.now_ms() must return a real epoch-ms value, got {vm_ms} (pre-fix state = 0)"
        );
        assert!(
            (vm_ms - a2r_ms).abs() < 5_000,
            "VM track must agree with a2r-std::time::now_ms (GOAL-003): vm={vm_ms} a2r={a2r_ms}"
        );
    }

    /// .at 层 i64 算术面（预存缺口勘定）：now_ms()/1000 vs now_sec()。
    /// 实测 TAG_I64 操作数的 DIV 解码错位（now_ms()/1000 → 0，skew =
    /// −now_sec 实证）——.at 内 i64 算术不可用是**预存平台缺口**，不在本
    /// 计划清偿面（下游 bench 用值为字符串/状态透传形，不涉 .at 内算术；
    /// unified print 的 TAG_I64 臂在档，BENCH 标记行可直出毫秒值）。本
    /// 测试#[ignore]留档勘定，待平台 i64 算术面单独立件清偿后翻正。
    #[test]
    #[cfg(feature = "ui-iced")]
    #[ignore = "预存缺口留档：.at 层 TAG_I64 算术（DIV）解码错位，平台级另件"]
    fn vm_time_now_ms_agrees_with_now_sec_at_level() {
        let src = format!(
            "{PROBE_SRC}fn probe_ms_sec_agree() int {{\n    return ((time.now_ms() / 1000) - time.now_sec())\n}}\n"
        );
        let bridge = build_probe_bridge(&src);
        let v = as_int_i64(
            bridge
                .call_vm_fn("probe_ms_sec_agree", &[])
                .expect("call probe_ms_sec_agree"),
        );
        assert!(
            v.abs() <= 1,
            "now_ms()/1000 must agree with now_sec() inside the VM, got skew {v}"
        );
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn a2r_std_time_agrees_with_vm_shim_clock() {
        // Host-side 对拍: the a2r leg's clock source vs the VM shim's (both
        // SystemTime epoch — same value class, GOAL-003).
        let bridge = build_probe_bridge(PROBE_SRC);
        let vm_sec = as_int_i64(
            bridge
                .call_vm_fn("probe_sec", &[])
                .expect("call probe_sec"),
        );
        assert!(
            vm_sec >= 1_672_531_200,
            "time.now_sec() must return a real epoch seconds value, got {vm_sec}"
        );
        let a2r_sec = a2r_std::time::now_sec();
        assert!(
            (vm_sec - a2r_sec).abs() < 5,
            "VM track must agree with a2r-std::time::now_sec: vm={vm_sec} a2r={a2r_sec}"
        );
    }

    #[test]
    #[cfg(feature = "ui-iced")]
    fn vm_time_now_is_epoch_seconds_string() {
        let bridge = build_probe_bridge(PROBE_SRC);
        let s = match bridge.call_vm_fn("probe_now", &[]) {
            Ok(auto_val::Value::Str(s)) => s.to_string(),
            other => panic!("time.now() must return Str, got {other:?}"),
        };
        assert!(
            !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()),
            "time.now() must be a decimal epoch-seconds string, got {s:?}"
        );
        let parsed: i64 = s.parse().expect("digits");
        assert!(parsed >= 1_672_531_200, "time.now() = {s:?} below epoch floor");
    }
}

#[cfg(test)]
mod plan701_shell_recent_supply {
    /// Empty path → false without touching the shell. (The real
    /// SHAddToRecentDocs smoke belongs to the downstream E2E — a test-run
    /// call would pollute the user's real Recent items.)
    #[test]
    fn shell_add_recent_empty_path_is_false() {
        assert!(!crate::vm::native::shell_add_recent(""));
    }
}

#[cfg(test)]
mod plan701_vue_strict_supply {
    /// PLAN-701 供⑤b: print 发射改道 globalThis.console.log——store 名为
    /// `console` 的 state 字段（Ref）遮蔽裸 console 的 TS2339 解
    /// （m1-supply §9 附记；auto-edit useEditorStore BENCH ×4 实证；
    /// 下游 regen_vue.py 遮蔽缓解件随之可退役）。
    #[test]
    fn print_emits_globalthis_console_log() {
        use crate::trans::{Sink, Trans, typescript::TypeScriptTrans};
        let _scope = crate::scope_manager::ScopeManager::new();
        let mut parser = crate::parser::Parser::from("print(\"BENCH x\")\n");
        let ast = parser.parse().expect("parse");
        let mut sink = Sink::new("p701_print".into());
        let mut trans = TypeScriptTrans::new("p701_print".into());
        trans.trans(ast, &mut sink).expect("trans");
        let out = String::from_utf8_lossy(&sink.done().expect("sink")).into_owned();
        assert!(
            out.contains("globalThis.console.log("),
            "print must emit globalThis.console.log (store console-field shadowing): {out}"
        );
        let bare_left = out.replace("globalThis.console.log(", "");
        assert!(
            !bare_left.contains("console.log("),
            "bare console.log must not appear: {out}"
        );
    }

    /// PLAN-701 供⑤a: dash 形 menubar-sub 族必须过 strict codegen
    /// validation（S002 unknown-element 门）。下游 2046 钉版工具链的
    /// **构建期内嵌 schema** 早于 PLAN-695 的 schema 吸收 → S002 误红
    /// （二进制内 0 命中实证）；master 树 schema 已在档——本测试钉当前
    /// 树的解析面，防回归 + 供下游 T-00② 形探针复核。
    #[test]
    fn schema_resolves_dash_form_menubar_sub_family() {
        let schema = crate::aura::load_default_schema().expect("embedded schema loads");
        for tag in ["menubar-sub", "menubar-sub-trigger", "menubar-sub-content"] {
            assert!(
                schema.resolve_tag(tag).is_some(),
                "dash-form <{tag}> must resolve through the schema (S002 guard)"
            );
        }
        // 下游用法面：trigger 的 text prop 在档（S001 漂移防线）。
        let (_, def) = schema
            .resolve_tag("menubar-sub-trigger")
            .expect("trigger resolves");
        assert!(def.get_prop("text").is_some(), "trigger text prop declared");
    }
}
