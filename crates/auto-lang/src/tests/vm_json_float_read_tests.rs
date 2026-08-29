//! Plan 474: VM 轨 `__json_object` 浮点字段消费误码回归（plan011④）
//!
//! 根因（已修，worktree commit d55f98b0e）：engine.rs CALL_SPEC 一元/二元数学
//! 分支的 `read_i32/push_i32/pop_i32` i32 化石把浮点 nv 位型当整数值——
//! 裸 f64（encode_f64=原始位）被读成低 32 位（54.16 → 0xE147AE14 →
//! -515396076）、TAG_F32 被读成 payload 位型（54.16f32 → 0x4258A3D7 →
//! 1113105367）。修复 = 接收者/结果按 NanoValue 透传（shim 的 VMConvertible
//! f64 pop 自带 TAG_F32→f64 提升与裸 f64 直读）。
//!
//! 分层钉死：
//!   - 脚本级（run_with_capture）：端到端基线（基础链本就无辜，11 用例）；
//!   - 位级：单条 GET_FIELD 直执行，位级断言；
//!   - widget handler 级：`.floor()` 等 CALL_SPEC 数学族在合成 handler 内
//!     的回归（④ 的现场形态——脚本路径走 CALL_NAT，掩盖此 bug）。

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

    #[test]
    fn json_missing_key_reads_null() {
        // Plan 044：__json_object 缺键读 null（Option.unwrap_or 链依赖，
        // engine.rs GET_FIELD __json_object 分支）。实测基线：print 形态为
        // `None`（encode_null 的显示形态）。
        let stdout = run_code(&format!("{PRELUDE}print(obj.no_such_field)"));
        eprintln!("[P474] missing-key = [{}]", stdout);
        assert!(
            stdout.contains("None") || stdout.contains("null") || stdout.contains("nil"),
            "expected None/null/nil for missing key, got: [{}]",
            stdout
        );
    }

    #[test]
    fn json_bool_field_compare() {
        // GET_FIELD bool 臂 push encode_bool（Plan 402 §13.10）——比较语义回归。
        // 待澄清#3 已修（print is_bool 臂）：bool 打印 true/false 形态。
        let stdout = run_code(&format!("{PRELUDE}print(obj.ok == true)"));
        eprintln!("[P474] bool-compare = [{}]", stdout);
        assert!(
            stdout.contains("true"),
            "expected true, got: [{}]",
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
        // 待澄清#3（已修）：print shim 原缺 is_bool 臂，TAG_BOOL 哨兵 payload
        // 被按 i32 解码特判打印 "1"/"0"（且真整数 i32::MIN 同路误打）；修后
        // tag 守卫优先，bool 一律 true/false。
        let stdout = run_code(&format!("{PRELUDE}print(obj.ok)"));
        eprintln!("[P474] bool-field = [{}]", stdout);
        assert!(
            stdout.contains("true"),
            "expected true, got: [{}]",
            stdout
        );
    }

    #[test]
    fn bool_literal_print_form() {
        // 待澄清#3 直测：字面量 bool 的 print 形态（原同印 1，修后 true）
        let stdout = run_code("let b = false\nprint(b)");
        eprintln!("[P474] bool-literal = [{}]", stdout);
        assert!(
            stdout.contains("false"),
            "expected false, got: [{}]",
            stdout
        );
    }

    #[test]
    fn int_min_prints_numerically() {
        // 待澄清#3 顺修：真整数 i32::MIN 不再被 bool 哨兵特判误打为 "1"
        let stdout = run_code("let n = -2147483648\nprint(n)");
        eprintln!("[P474] int-min = [{}]", stdout);
        assert!(
            stdout.contains("-2147483648"),
            "expected -2147483648, got: [{}]",
            stdout
        );
    }

    #[test]
    fn json_null_field_set_preserves_nil() {
        // 待澄清#4 直测：decode_tagged_nv 补 is_null 臂——null nv 写字段
        // 存 Value::Nil（原落 _ => Int(0) 兜底失真）。经缺键读 null → 写入
        // 对象字面量字段 → 读回仍为 null 形态。
        let stdout = run_code(&format!(
            "{PRELUDE}let o = {{\"f\": 0}}\no.f = obj.no_such_field\nprint(o.f)"
        ));
        eprintln!("[P474] null-field = [{}]", stdout);
        assert!(
            stdout.contains("None") || stdout.contains("null") || stdout.contains("nil"),
            "expected None/null/nil preserved through SET_FIELD, got: [{}]",
            stdout
        );
    }

    #[test]
    fn json_float_fstring_interpolation() {
        // 待澄清#5 钉正：f-string 插值语法是 `${expr}` / `$ident`（parser
        // fstr 文法），`{expr}` 是字面文本——原用例语法写错，非引擎缺陷。
        // 覆盖「json 浮点 → 局部 → 插值」消费链。
        let stdout = run_code(&format!(
            "{PRELUDE}let x = obj.storage_free_gb\nprint(f\"value=${{x}}\")"
        ));
        eprintln!("[P474] f-string = [{}]", stdout);
        assert!(
            stdout.contains("54.16"),
            "expected 54.16 in f-string, got: [{}]",
            stdout
        );
    }

    // ── handler 形态变体（活体复现判定：CALL_SPEC 数学族是损坏点）────────

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

/// Widget 语境（plan011④ 现场形态）：Init handler 内 json 浮点字段的
/// CALL_SPEC 数学族消费。④ 根因即在此语境——脚本路径的 `.floor()` 编译为
/// CALL_NAT（marshalling 正确），而合成 handler 内编译为 CALL_SPEC，其内联
/// 数学分发曾以 read_i32/push_i32/pop_i32 处理浮点 nv（位型当整数）。
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
        var probe_d float = 0.0
        var probe_e float = 0.0
        var probe_f float = 0.0
        var probe_g float = 0.0
    }
    on {
        .Init -> {
            var r = Json.to_value("{\"storage_free_gb\":54.16}")
            .probe_a = r.storage_free_gb
            .probe_b = r.storage_free_gb.floor()
            .probe_c = 54.16.floor()
            .probe_d = r.storage_free_gb.ceil()
            .probe_e = r.storage_free_gb.round()
            .probe_f = r.storage_free_gb.sqrt()
            .probe_g = r.storage_free_gb.powf(2.0)
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
        Some(comp)
    }

    fn as_f64(v: &auto_val::Value) -> Option<f64> {
        match v {
            auto_val::Value::Float(f) => Some(*f as f64),
            auto_val::Value::Double(d) => Some(*d),
            _ => None,
        }
    }

    /// CALL_SPEC 数学族回归：一元（floor/ceil/round/sqrt）× 接收者两形态
    /// （json 读出的裸 f64 nv / 字面量 TAG_F32 nv）+ 二元（powf）。
    /// 修复前实测：floor(json) = Int(-515396076)（= 0xE147AE14，54.16 裸
    /// f64 低 32 位按 i32 读）、floor(字面量) = Int(1113105367)（=
    /// 0x4258A3D7，f32(54.16) 位型按 i32 读）。
    #[test]
    fn json_float_callspec_math_family_in_init_handler() {
        let comp = match build_probe_widget() {
            Some(c) => c,
            None => {
                eprintln!("[P474] widget build failed — see stderr above");
                panic!("probe widget must build");
            }
        };
        let state = comp.read_all_state();

        let check = |name: &str, expect: f64, tol: f64| {
            let got = state.get(name).and_then(as_f64);
            eprintln!("[P474] {} = {:?} (expect ~{})", name, state.get(name), expect);
            let v = got.unwrap_or_else(|| {
                panic!("{} 应为数值，got {:?}", name, state.get(name))
            });
            assert!(
                (v - expect).abs() < tol,
                "{} 应为 ~{}, got {:?}",
                name,
                expect,
                state.get(name)
            );
        };

        check("probe_a", 54.16, 1e-3); // Dot 读 → state 赋值（基线）
        check("probe_b", 54.0, 1e-9); // floor(json)
        check("probe_c", 54.0, 1e-9); // floor(字面量)
        check("probe_d", 55.0, 1e-9); // ceil(json)
        check("probe_e", 54.0, 1e-9); // round(json)
        check("probe_f", 7.36, 1e-2); // sqrt(54.16) ≈ 7.3600…
        check("probe_g", 2933.3, 0.5); // powf(54.16, 2) ≈ 2933.31
    }
}
