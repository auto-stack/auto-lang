//! Plan 474: VM 轨 `__json_object` 浮点字段 Dot 读误码回归（plan011④）
//!
//! 症状（auto-os-config 现场，worktree plan-011-dev）：JSON 浮点字段 54.16
//! 单跳 Dot 读出 -1073741824（0xC0000000 = -2.0f32 位型按 i32 重解释），
//! `.floor()` 得 0；整数（as_i64→Int）与字符串字段正常。
//! 写入侧三级排除见 docs/plans/KNOWN-DEBT-AND-RISKS.md p1(plan011④)：
//!   1. stdlib `json_to_vm_value(_inner)` Number 分支产 `Value::Double`；
//!   2. 对象字段直存 GenericInstanceData（无栈往返）；
//!   3. GET_FIELD GenericInstanceData Double 臂 `push_f64`。
//!
//! 两层钉死：
//!   - 脚本级（run_with_capture，真实编译+VM 执行链）——端到端形态；
//!   - 字节码级（单条 GET_FIELD 直执行）——位级断言，钉死读取链本身。

#[cfg(test)]
mod vm_json_float_read_tests {
    use crate::run_with_capture;

    const PRELUDE: &str = r#"
let js = "{\"storage_free_gb\":54.16,\"n_cpu\":8,\"host\":\"abc\",\"ok\":true}"
let obj = Json.to_value(js)
"#;

    fn run_code(code: &str) -> String {
        match run_with_capture(code) {
            Ok((_, stdout)) => stdout,
            Err(e) => panic!("run failed: {:?}", e),
        }
    }

    // ── 脚本级：端到端（真实 codegen + 执行链）────────────────────────

    #[test]
    fn json_float_field_dot_read_print() {
        let stdout = run_code(&format!("{PRELUDE}print(obj.storage_free_gb)"));
        eprintln!("[P474] dot-read print = [{}]", stdout);
        assert!(stdout.contains("54.16"), "expected 54.16, got: [{}]", stdout);
    }

    #[test]
    fn json_float_field_local_then_print() {
        // os-config 现场形态：Dot 读结果经局部槽 store/load 后再消费
        let stdout = run_code(&format!("{PRELUDE}let x = obj.storage_free_gb\nprint(x)"));
        eprintln!("[P474] local-then-print = [{}]", stdout);
        assert!(stdout.contains("54.16"), "expected 54.16, got: [{}]", stdout);
    }

    #[test]
    fn json_float_field_floor() {
        let stdout = run_code(&format!("{PRELUDE}print(obj.storage_free_gb.floor())"));
        eprintln!("[P474] floor = [{}]", stdout);
        assert!(stdout.contains("54"), "expected 54, got: [{}]", stdout);
    }

    #[test]
    fn json_float_field_arith() {
        let stdout = run_code(&format!("{PRELUDE}print(obj.storage_free_gb * 100)"));
        eprintln!("[P474] arith(*100) = [{}]", stdout);
        // f64 精度下 54.16*100 可能印 5415.99…，都算「值正确」
        assert!(
            stdout.contains("5416") || stdout.contains("5415.99"),
            "expected ~5416, got: [{}]",
            stdout
        );
    }

    // ── handler 形态变体（活体复现判定：`.floor()` 是唯一损坏点）────────

    #[test]
    fn json_float_floor_into_object_field() {
        // os-config 形态：<json 浮点>.floor() 结果写入对象字段（state 赋值同构）
        let stdout = run_code(&format!(
            "{PRELUDE}let o = {{\"f\": 0.0}}\no.f = obj.storage_free_gb.floor()\nprint(o.f)"
        ));
        eprintln!("[P474] floor→obj-field = [{}]", stdout);
        assert!(
            stdout.contains("54"),
            "expected 54, got: [{}]",
            stdout
        );
    }

    #[test]
    fn json_float_floor_stored_local() {
        let stdout = run_code(&format!(
            "{PRELUDE}let y = obj.storage_free_gb.floor()\nprint(y)"
        ));
        eprintln!("[P474] floor→local = [{}]", stdout);
        assert!(stdout.contains("54"), "expected 54, got: [{}]", stdout);
    }

    #[test]
    fn json_float_floor_arith_consumer() {
        let stdout = run_code(&format!("{PRELUDE}print(obj.storage_free_gb.floor() + 0.0)"));
        eprintln!("[P474] floor+0.0 = [{}]", stdout);
        assert!(
            stdout.contains("54"),
            "expected 54, got: [{}]",
            stdout
        );
    }

    // 控制组：同实例的整数/字符串/bool 字段（既有正确路径不应受影响）
    #[test]
    fn json_int_field_dot_read() {
        let stdout = run_code(&format!("{PRELUDE}print(obj.n_cpu)"));
        eprintln!("[P474] int-field = [{}]", stdout);
        assert!(stdout.contains("8"), "expected 8, got: [{}]", stdout);
    }

    #[test]
    fn json_str_field_dot_read() {
        let stdout = run_code(&format!("{PRELUDE}print(obj.host)"));
        eprintln!("[P474] str-field = [{}]", stdout);
        assert!(stdout.contains("abc"), "expected abc, got: [{}]", stdout);
    }

    #[test]
    fn json_bool_field_dot_read() {
        // 实测基线（S2 观测，2026-08-29）：json bool 字段 print 印出 `1` 而非
        // 字面量形态的 `true`（`let b = true; print(b)` 印 true）——显示/类型
        // 提示路径的旁支不一致，非 ④ 值损坏；已登记 plan 474 待澄清#3。
        let stdout = run_code(&format!("{PRELUDE}print(obj.ok)"));
        eprintln!("[P474] bool-field = [{}]", stdout);
        assert!(
            stdout.contains("1"),
            "expected 1 (observed baseline), got: [{}]",
            stdout
        );
    }

    // ── 字节码级：单条 GET_FIELD 位级断言 ──────────────────────────────

    #[test]
    fn json_float_field_get_field_bitexact() {
        use crate::vm::engine::AutoVM;
        use crate::vm::ffi::stdlib;
        use crate::vm::generic_registry::GenericInstanceData;
        use crate::vm::opcode::OpCode;
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;
        use auto_val::Value;
        use std::sync::Arc;

        // ① 写侧物化：json → __json_object（GenericInstanceData）
        let mut vm = AutoVM::new(VirtualFlash::new(16), 1024);
        let mut task = AutoTask::new(0, 1024, 0);
        let json: serde_json::Value =
            serde_json::from_str("{\"storage_free_gb\":54.16}").unwrap();
        stdlib::json_to_vm_value(&mut task, &vm, &json, 0).unwrap();
        let obj_nv = task.ram.pop_nv();

        // ② 写侧钉死：fields[0] 是 Value::Double(54.16)（若写侧回归此处先红）
        let obj_id = auto_val::decode_object(obj_nv) as u64;
        let heap_ref = vm.heap_objects.get(&obj_id).expect("heap obj missing");
        let heap_obj = heap_ref.read().unwrap();
        let inst = heap_obj
            .as_any()
            .downcast_ref::<GenericInstanceData>()
            .expect("__json_object instance");
        assert_eq!(inst.mono_name, "__json_object");
        assert_eq!(inst.field_names[0], "storage_free_gb");
        match inst.get_field(0) {
            Some(Value::Double(d)) => assert_eq!(*d, 54.16, "写侧字段值"),
            other => panic!("写侧字段非 Double: {:?}", other),
        }
        drop(heap_obj);
        drop(heap_ref);

        // ③ 读侧：flash 换单条 GET_FIELD，receiver 预推进栈（rc_push 平衡
        //    arm 末尾的 Plan 419 stake 释放）
        let field_idx = vm.add_string(b"storage_free_gb".to_vec()) as u32;
        let mut code = vec![OpCode::GET_FIELD as u8];
        code.extend_from_slice(&field_idx.to_le_bytes());
        vm.flash = Arc::new(VirtualFlash::new_with_code(code));
        task.ip = 0;
        vm.rc_push(&mut task, obj_nv);
        let frame = vm.execute_single_frame(&mut task, 1);
        let result_nv = task.ram.pop_nv();
        eprintln!(
            "[P474] GET_FIELD bitexact: frame={:?} read=0x{:016X} expect=0x{:016X}",
            frame,
            result_nv,
            auto_val::encode_f64(54.16)
        );
        assert_eq!(
            result_nv, auto_val::encode_f64(54.16),
            "GET_FIELD 位级不等——注入点在读取链"
        );
    }
}

/// Widget 语境（plan011④ 现场形态）：Init handler 内 json 浮点字段消费。
///
/// 活体复现（os-config plan-011-dev，2026-08-29）判定：基础链 Dot 读/算术/
/// int 槽赋值全部正确，唯一损坏点是 **`.floor()` 方法派发**——
/// `r.storage_free_gb.floor()` → -536870912（0xE0000000，int），值相关错解码
/// （用户现场 54.16 → -1073741824/0xC0000000），fb06cd8b2「CALL_NAT 损坏浮点」
/// 族的方法形态残留。此处在本仓以最小 widget 源级钉死。
///
/// 门控与 plan370_*_tests 一致：widget 测试需要 `ui-iced` feature。
#[cfg(all(test, feature = "ui-iced"))]
mod vm_json_float_widget_tests {
    use crate::ui::dynamic::DynamicComponent;
    use crate::ui::widget_registry::WidgetRegistry;
    use std::collections::HashMap;

    fn build_probe_widget() -> Option<DynamicComponent> {
        let code = r#"
widget App {
    model {
        var probe_a float = 0.0
        var probe_b float = 0.0
        var probe_c float = 0.0
    }
    on {
        .Init -> {
            var r = Json.to_value("{\"storage_free_gb\":54.16}")
            .probe_a = r.storage_free_gb
            .probe_b = r.storage_free_gb.floor()
            .probe_c = 54.16.floor()
        }
    }
    view {
        col {
            text (text: "probe") {}
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code).with_session(session);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[P474] probe widget parse error: {:?}", e);
                return None;
            }
        };
        let root_decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
            _ => None,
        })?;
        let widget = crate::aura::extract_widget_from_decl(&root_decl).ok()?;
        let mut comp = DynamicComponent::with_registry_and_imports_from_decls(
            &root_decl,
            &[],
            &widget,
            WidgetRegistry::new(),
            vec![],
            &HashMap::new(),
            false,
        )
        .ok()?;
        comp.fire_init();
        // [P474] 临时探针：dump 合成 handler 字节码（S3 二分证据，随修摘除）
        {
            use crate::vm::disasm::Disassembler;
            let bridge = comp.bridge();
            let vm = bridge.vm();
            let dis = Disassembler::new(&vm.flash);
            let strings: Vec<String> = vm
                .strings
                .read()
                .unwrap()
                .iter()
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .collect();
            let mut names: Vec<&String> = vm.flash.exports_by_name.keys().collect();
            names.sort();
            eprintln!("[P474-disasm] exports = {:?}", names);
            if let Some(&addr) = vm.flash.exports_by_name.get("handler_App_Init") {
                for line in dis.disassemble_range(addr as usize, addr as usize + 140) {
                    eprintln!(
                        "[P474-disasm] {:04} {} {}",
                        line.offset, line.mnemonic, line.operands
                    );
                }
            }
            eprintln!("[P474-disasm] strings = {:?}", strings);
        }
        Some(comp)
    }

    fn as_f64(v: &auto_val::Value) -> Option<f64> {
        match v {
            auto_val::Value::Float(f) => Some(*f as f64),
            auto_val::Value::Double(d) => Some(*d),
            _ => None,
        }
    }

    #[test]
    fn json_float_consumption_in_init_handler() {
        let comp = match build_probe_widget() {
            Some(c) => c,
            None => {
                eprintln!("[P474] widget build failed — see stderr above");
                panic!("probe widget must build");
            }
        };
        let state = comp.read_all_state();
        for name in ["probe_a", "probe_b", "probe_c"] {
            eprintln!(
                "[P474] {} = {:?} ({:?})",
                name,
                state.get(name),
                state.get(name).map(|v| std::mem::discriminant(v))
            );
        }
        let a = state.get("probe_a").and_then(as_f64).unwrap_or(f64::NAN);
        assert!(
            (a - 54.16).abs() < 1e-3,
            "probe_a（Dot 读→state 赋值）应为 ~54.16，got {:?}",
            state.get("probe_a")
        );
        let b = state.get("probe_b").and_then(as_f64).unwrap_or(f64::NAN);
        assert!(
            (b - 54.0).abs() < 1e-9,
            "probe_b（json 浮点 .floor()）应为 54，got {:?}",
            state.get("probe_b")
        );
        let c = state.get("probe_c").and_then(as_f64).unwrap_or(f64::NAN);
        assert!(
            (c - 54.0).abs() < 1e-9,
            "probe_c（字面量 .floor()）应为 54，got {:?}",
            state.get("probe_c")
        );
    }
}
