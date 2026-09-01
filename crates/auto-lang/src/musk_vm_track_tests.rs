//! PLAN-053 (auto-musk VM 上游跟踪伞): musk VM 轨实测暴露的 auto-lang
//! 运行时缺陷回归锚。沿 planNNN_tests.rs 模块惯例，lib.rs 注册。
//!
//! - P-053-2: nil/None 语义不等值——`.store.current_gate != None` 守卫拦不
//!   住 null 态（GateCard/ReportCard 常显）。null 家族字面量（`null` →
//!   CONST_I32 -1、`None` → PUSH_NIL encode_null）与 JSON null 读回值在
//!   EQ/NE 比较臂必须等值。
//! - P-053-1: computed 经 `use.web.fn` helper 链读扁平 store 字段产出空
//!   （musk `filteredMessages` 消息列表恒空）。见下方 p1 模块。

/// P-053-2: null 家族等值语义（脚本级：真实 codegen + 执行链）。
#[cfg(test)]
mod musk_vm_track_p053_2_null_equality {
    use crate::run_with_capture;

    fn run_code(code: &str) -> String {
        match run_with_capture(code) {
            Ok((_, stdout)) => stdout,
            Err(e) => panic!("run failed: {:?}", e),
        }
    }

    /// `null` 字面量与 `None` 字面量必须等值（musk store 字段
    /// `var current_gate Value = null` 初始化 vs 守卫 `!= None`）。
    #[test]
    fn null_literal_eq_none_literal() {
        let out = run_code("print(null == None)");
        eprintln!("[P053-2] null == None => [{}]", out);
        assert!(out.contains("true"), "expected true, got: [{}]", out);
    }

    /// 守卫形态：null 初始化的变量 `!= None` 必须为 false（GateCard
    /// 显隐的现场形态）。
    #[test]
    fn null_var_guard_ne_none_is_false() {
        let out = run_code("let g = null\nif g != None {\n    print(\"BAD\")\n} else {\n    print(\"GOOD\")\n}");
        eprintln!("[P053-2] null guard => [{}]", out);
        assert!(out.contains("GOOD"), "expected GOOD (guard blocked), got: [{}]", out);
    }

    /// JSON null 字段与 `None` 守卫（musk 后端桥回填形态）。
    #[test]
    fn json_null_field_eq_none() {
        let out = run_code(
            "let js = \"{\\\"gate\\\":null}\"\nlet obj = Json.to_value(js)\nprint(obj.gate == None)",
        );
        eprintln!("[P053-2] json null == None => [{}]", out);
        assert!(out.contains("true"), "expected true, got: [{}]", out);
    }

    /// 控制组：None 与 None 等值（本应正确，钉住防回归）。
    #[test]
    fn none_eq_none_control() {
        let out = run_code("print(None == None)");
        eprintln!("[P053-2] None == None => [{}]", out);
        assert!(out.contains("true"), "expected true, got: [{}]", out);
    }

    /// 控制组：null 与非空值不等。
    #[test]
    fn null_ne_int_control() {
        let out = run_code("let g = null\nprint(g == 5)");
        eprintln!("[P053-2] null == 5 => [{}]", out);
        assert!(out.contains("false"), "expected false, got: [{}]", out);
    }

    /// `null ?? default`：null 字面量侧也必须落到 default（musk
    /// `ev.run_id ?? ""` 家族；NULL_COALESCE 只认 is_null，i32(-1) 漏过）。
    #[test]
    fn null_literal_coalesces_to_default() {
        let out = run_code("let g = null\nprint(g ?? \"dflt\")");
        eprintln!("[P053-2] null ?? dflt => [{}]", out);
        assert!(out.contains("dflt"), "expected dflt, got: [{}]", out);
    }
}

/// P-053-6: web 生态 Regex 静态形态（musk forge_helpers/mention_helpers 的
/// 消息渲染链全死于此——`CALL_SPEC: no function 'Regex.replace' for type
/// 'Regex'`，Call 臂 swallowed-Err 静默）。
#[cfg(test)]
mod musk_vm_track_p053_6_regex_static {
    use crate::run_with_capture;

    fn run_code(code: &str) -> String {
        match run_with_capture(code) {
            Ok((_, stdout)) => stdout,
            Err(e) => panic!("run failed: {:?}", e),
        }
    }

    /// `Regex.replace(text, pattern, repl, "g")` 全局替换（stripQuestionnaire
    /// / render_mentions_default 的 HTML 转义链现场形态）。
    #[test]
    fn regex_replace_global() {
        let out = run_code(
            "print(Regex.replace(Regex.replace(\"a<b>c\", \"<\", \"&lt;\", \"g\"), \">\", \"&gt;\", \"g\"))",
        );
        eprintln!("[P053-6] replace g => [{}]", out);
        assert!(
            out.contains("a&lt;b&gt;c"),
            "expected a&lt;b&gt;c, got: [{}]", out
        );
    }

    /// `Regex.replace` 无 g 标志只替换首处（web 生态默认）。
    #[test]
    fn regex_replace_first() {
        let out = run_code("print(Regex.replace(\"a-a-a\", \"a\", \"b\", \"\"))");
        eprintln!("[P053-6] replace first => [{}]", out);
        assert!(out.contains("b-a-a"), "expected b-a-a, got: [{}]", out);
    }

    /// `Regex.test(text, pattern) -> bool`（stripQuestionnaire 的
    /// 问卷探测现场形态）。
    #[test]
    fn regex_test_bool() {
        let out = run_code("print(Regex.test(\"hello 123\", \"\\\\d+\"))");
        eprintln!("[P053-6] test => [{}]", out);
        assert!(out.contains("true"), "expected true, got: [{}]", out);
    }

    /// 控制组：不匹配返回 false。
    #[test]
    fn regex_test_no_match() {
        let out = run_code("print(Regex.test(\"hello\", \"\\\\d+\"))");
        eprintln!("[P053-6] test-neg => [{}]", out);
        assert!(out.contains("false"), "expected false, got: [{}]", out);
    }

    /// stripQuestionnaire 链最小同构：Regex.replace → trimEnd → obj 字段落值。
    /// 现场症状：blocks 元素 {kind:"text", text:Nil}（正文丢成 null）。
    #[test]
    fn strip_questionnaire_chain_keeps_text() {
        let out = run_code(concat!(
            "fn stripQ(text str, streaming bool) -> str {\n",
            "    if text == \"\" { return text }\n",
            "    var out = Regex.replace(text, \"```json[\\\\s\\\\S]*\", \"\", \"g\")\n",
            "    return out.trimEnd()\n",
            "}\n",
            "var b = { kind: \"text\", text: stripQ(\"reply with one short sentence\", false) }\n",
            "print(b.kind + \"|\" + b.text)\n",
        ));
        eprintln!("[P053-6] stripQ chain => [{}]", out);
        assert!(
            out.contains("text|reply with one short sentence"),
            "expected text|reply..., got: [{}]",
            out
        );
    }

    /// 切分探针 A：纯 trimEnd。
    #[test]
    fn probe_trim_end_alone() {
        let out = run_code("fn f(s str) -> str { return s.trimEnd() }\nprint(f(\"abc \"))");
        eprintln!("[P053-6] trimEnd alone => [{}]", out);
        assert!(out.contains("abc"), "expected abc, got: [{}]", out);
    }

    /// 切分探针 B：Regex.replace 结果直存局部再读。
    #[test]
    fn probe_replace_local_roundtrip() {
        let out = run_code(
            "fn f(t str) -> str { var out = Regex.replace(t, \"zzz\", \"\", \"g\"); return out }\nprint(f(\"reply\"))",
        );
        eprintln!("[P053-6] replace local => [{}]", out);
        assert!(out.contains("reply"), "expected reply, got: [{}]", out);
    }

    /// 接收者泄漏探针：函数体内静态 native 调用后，后续局部/字段读取
    /// 不得被残留的接收者槽污染（现场：blocks 元素 text 落成 "0"）。
    #[test]
    fn probe_static_native_receiver_leak_in_fn() {
        // 变体 A：fn 内 replace 后直接返回（无 obj）。
        let a = run_code(concat!(
            "fn mk(s str) -> str {\n",
            "    var out = Regex.replace(s, \"zzz\", \"\", \"g\")\n",
            "    return out\n",
            "}\n",
            "fn main() {\n",
            "    print(mk(\"reply with one short sentence\"))\n",
            "}\n",
        ));
        eprintln!("[P053-6] leak-A fn-replace => [{}]", a);
        // 变体 B：fn 内 obj 字面量（无 native）。
        let b = run_code(concat!(
            "fn mk(s str) -> obj {\n",
            "    return { kind: \"text\", text: s }\n",
            "}\n",
            "fn main() {\n",
            "    let b = mk(\"reply\")\n",
            "    print(b.kind + \"|\" + b.text)\n",
            "}\n",
        ));
        eprintln!("[P053-6] leak-B fn-obj => [{}]", b);
        // 变体 C：完整形态。
        let c = run_code(concat!(
            "fn mk(s str) -> obj {\n",
            "    var out = Regex.replace(s, \"zzz\", \"\", \"g\")\n",
            "    return { kind: \"text\", text: out }\n",
            "}\n",
            "fn main() {\n",
            "    let b = mk(\"reply with one short sentence\")\n",
            "    print(b.kind + \"|\" + b.text)\n",
            "}\n",
        ));
        eprintln!("[P053-6] leak-C full => [{}]", c);
        assert!(
            a.contains("reply with one short sentence"),
            "A: fn 内 replace 返回, got: [{}]", a
        );
        assert!(b.contains("text|reply"), "B: fn 内 obj 字面量, got: [{}]", b);
        assert!(
            c.contains("text|reply with one short sentence"),
            "C: 完整形态, got: [{}]", c
        );
    }

    /// 消息链字符串方法矩阵（musk forge/mention helpers 实际使用面）。
    #[test]
    fn message_chain_str_methods_matrix() {
        let cases: &[(&str, &str)] = &[
            ("trimEnd", "fn f(s str) -> str { return s.trimEnd() }\nprint(f(\"a \"))|a"),
            ("to_lower", "fn f(s str) -> str { return s.to_lower() }\nprint(f(\"AbC\"))|abc"),
            ("to_upper", "fn f(s str) -> str { return s.to_upper() }\nprint(f(\"aBc\"))|ABC"),
            ("includes", "fn f(s str) -> bool { return s.includes(\"ell\") }\nprint(f(\"hello\"))|true"),
            ("lastIndexOf", "fn f(s str) -> int { return s.lastIndexOf(\"l\") }\nprint(f(\"hello\"))|3"),
            ("indexOf", "fn f(s str) -> int { return s.indexOf(\"e\") }\nprint(f(\"hello\"))|1"),
            ("substring", "fn f(s str) -> str { return s.substring(1, 3) }\nprint(f(\"hello\"))|el"),
            ("char_code_at", "fn f(s str) -> int { return s.char_code_at(0) }\nprint(f(\"A\"))|65"),
            ("slice", "fn f(s str) -> str { return s.slice(0, 2) }\nprint(f(\"hello\"))|he"),
        ];
        let mut misses: Vec<&str> = Vec::new();
        for (name, spec) in cases.iter() {
            let (code, expect) = spec.split_once('|').unwrap();
            let out = run_code(code);
            let ok = out.trim().contains(expect);
            eprintln!("[P053-6] str-method {} => [{}] expect contains [{}] {}", name, out.trim(), expect, if ok { "OK" } else { "MISS" });
            if !ok {
                misses.push(name);
            }
        }
        assert!(misses.is_empty(), "消息链字符串方法缺口: {:?}", misses);
    }
}

/// P-053-6 widget 级：子组件 computed 调 Regex helper 渲染消息正文
/// （UserMessage `html => render_mentions_default(.content)` 的最小同构）。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p053_6_widget_content {
    use crate::parser::Parser;

    fn count_text_nodes(view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>, label: &str) -> usize {
        use crate::ui::view::View;
        match view {
            View::Text { content, .. } => usize::from(content == label),
            View::Column { children, .. } => children.iter().map(|c| count_text_nodes(c, label)).sum(),
            View::Row { children, .. } => children.iter().map(|c| count_text_nodes(c, label)).sum(),
            _ => 0,
        }
    }

    #[test]
    fn widget_child_computed_regex_helper_content() {
        let src = concat!(
            // P-053-6 续: use.web component 形态 + registry 已注册同名适配器
            // widget（renderer.vm.at Markdown 纯文本降级的现场形态）——图标
            // 臂不得遮蔽注册组件。
            "use.web component Msg53c from \"src/front/ports/renderer.at\"\n",
            // render_mentions_default 的最小同构：HTML 转义链(Regex.replace)+
            // 词汇探测(Regex.test)——现场死因的浓缩。
            "fn esc(text str) -> str {\n",
            "    let a = Regex.replace(text, \"&\", \"&amp;\", \"g\")\n",
            "    let b = Regex.replace(a, \"<\", \"&lt;\", \"g\")\n",
            "    let c = Regex.replace(b, \">\", \"&gt;\", \"g\")\n",
            "    if Regex.test(c, \"q\") { return c + \"[Q]\" }\n",
            "    return c\n",
            "}\n",
            "widget Root53c {\n",
            "    model { var messages []Value = [] }\n",
            "    view { col { for m in .messages { Msg53c { content: .m.content } } } }\n",
            "}\n",
            "widget Msg53c(content: str) {\n",
            "    computed { html => esc(.content) }\n",
            "    view { col { text .html {} } }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut decls: Vec<crate::ast::WidgetDecl> = vec![];
        let mut import_stmts: Vec<crate::ast::Stmt> = vec![];
        for st in &ast.stmts {
            match st {
                crate::ast::Stmt::WidgetDecl(d) => decls.push(d.clone()),
                crate::ast::Stmt::Fn(_) | crate::ast::Stmt::UseWeb(_) => import_stmts.push(st.clone()),
                _ => {}
            }
        }
        let root_widget =
            crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        let child_widget =
            crate::aura::extract_widget_from_decl(&decls[1]).expect("extract child");
        registry.register(child_widget);

        let mut comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            registry,
            import_stmts,
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component");
        comp.write_state_vec(
            "messages",
            vec![auto_val::Value::Obj(
                auto_val::Obj::new().with("content", auto_val::Value::str("hello & <world>")),
            )],
        )
        .unwrap();
        let (view, _, _) = comp.view_with_debug_gated(false);
        fn dump_texts(v: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>, out: &mut Vec<String>) {
            use crate::ui::view::View;
            match v {
                View::Text { content, .. } => out.push(content.clone()),
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children { dump_texts(c, out); }
                }
                _ => {}
            }
        }
        let mut texts = Vec::new();
        dump_texts(&view, &mut texts);
        eprintln!("[P053-6w] text nodes: {:?}", texts);
        let rows = count_text_nodes(&view, "hello &amp; &lt;world&gt;");
        assert_eq!(rows, 1, "子组件 computed 经 Regex helper 的正文必须渲染");
    }

    /// P-053-6 续(Obj 实参物化)：computed 把 obj 值(state 字段)传给
    /// helper，helper 内读字段——原编组落 push_i32(0) 占位，msg 变 Int(0)，
    /// `.content` 全读 0（musk 消息正文 "0" 的终因）。
    #[test]
    fn widget_obj_arg_field_read_in_helper() {
        let src = concat!(
            "fn contentOf(msg obj) -> str {\n",
            "    return msg.content\n",
            "}\n",
            "widget Root53d {\n",
            "    model { var current obj = {} }\n",
            "    view { col { Msg53d {} } }\n",
            "}\n",
            "widget Msg53d {\n",
            "    computed { body => contentOf(.store.current) }\n",
            "    view { col { text .body {} } }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut decls: Vec<crate::ast::WidgetDecl> = vec![];
        let mut import_stmts: Vec<crate::ast::Stmt> = vec![];
        for st in &ast.stmts {
            match st {
                crate::ast::Stmt::WidgetDecl(d) => decls.push(d.clone()),
                crate::ast::Stmt::Fn(_) => import_stmts.push(st.clone()),
                _ => {}
            }
        }
        let root_widget =
            crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        let child_widget =
            crate::aura::extract_widget_from_decl(&decls[1]).expect("extract child");
        registry.register(child_widget);

        let mut comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            registry,
            import_stmts,
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component");
        comp.write_state(
            "current",
            auto_val::Value::Obj(
                auto_val::Obj::new().with("content", auto_val::Value::str("hello obj world")),
            ),
        )
        .unwrap();
        let (view, _, _) = comp.view_with_debug_gated(false);
        let rows = count_text_nodes(&view, "hello obj world");
        assert_eq!(rows, 1, "obj 实参经 helper 的字段读取必须成立");
    }
}

/// P-053-1: computed + use.web.fn helper 链（占位——复现测试落地于步骤 4）。
#[cfg(test)]
mod musk_vm_track_p053_1_computed_helper_chain {
    use crate::run_with_capture;

    fn run_code(code: &str) -> String {
        match run_with_capture(code) {
            Ok((_, stdout)) => stdout,
            Err(e) => panic!("run failed: {:?}", e),
        }
    }

    /// helper 内对 `obj` 参数取 `.length`（chatActivePath 现场形态:
    /// `let n = messages.length`）——不得因 GET_FIELD 接收者是 TAG_LIST
    /// 而产出垃圾/0。
    #[test]
    fn helper_obj_param_dot_length() {
        let out = run_code(
            "fn n(messages obj) -> int {\n    return messages.length\n}\nprint(n([1, 2, 3]))",
        );
        eprintln!("[P053-1] helper .length => [{}]", out);
        assert!(out.contains("3"), "expected 3, got: [{}]", out);
    }

    /// helper 内 `obj` 参数按索引取元素（messages[i].id 家族的最小形态）。
    #[test]
    fn helper_obj_param_index() {
        let out = run_code(
            "fn at(messages obj, i int) -> int {\n    return messages[i]\n}\nprint(at([10, 20, 30], 1))",
        );
        eprintln!("[P053-1] helper index => [{}]", out);
        assert!(out.contains("20"), "expected 20, got: [{}]", out);
    }

    /// 对照组:同数据直连(不经 helper 参数)。
    #[test]
    fn direct_dot_length_control() {
        let out = run_code("let xs = [1, 2, 3]\nprint(xs.length)");
        eprintln!("[P053-1] direct .length => [{}]", out);
        assert!(out.contains("3"), "expected 3, got: [{}]", out);
    }

    /// `[]Value` 类型参数取 `.length`（mention_professions_list 现场形态:
    /// `prop_prof []Value` + `prop_prof.length`）。
    #[test]
    fn helper_value_list_param_dot_length() {
        let out = run_code(
            "fn n(prop_prof []Value) -> int {\n    return prop_prof.length\n}\nprint(n([1, 2, 3]))",
        );
        eprintln!("[P053-1] []Value .length => [{}]", out);
        assert!(out.contains("3"), "expected 3, got: [{}]", out);
    }

    /// `for x in list` 迭代 `[]Value` 参数（chatSearchFilter 现场形态）。
    #[test]
    fn helper_value_list_for_iter() {
        let out = run_code(
            "fn n(configs []Value) -> int {\n    var c = 0\n    for x in configs {\n        c = c + 1\n    }\n    return c\n}\nprint(n([{}, {}, {}]))",
        );
        eprintln!("[P053-1] []Value for => [{}]", out);
        assert!(out.contains("3"), "expected 3, got: [{}]", out);
    }
}

/// P-053-1 widget 级:computed 链式 helper 调用 + `.store.` 扁平化实参
/// (musk filteredMessages => chatSearchFilter(chatActivePath(.store.messages,
/// .store.active_leaf), .chat_search) 的最小同构)。此前实机产出空列表。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p053_1_widget_computed {
    use crate::parser::Parser;

    fn count_text_nodes(view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>, label: &str) -> usize {
        use crate::ui::view::View;
        match view {
            View::Text { content, .. } => usize::from(content == label),
            View::Column { children, .. } => children.iter().map(|c| count_text_nodes(c, label)).sum(),
            View::Row { children, .. } => children.iter().map(|c| count_text_nodes(c, label)).sum(),
            _ => 0,
        }
    }

    /// musk chats_view 同构:helper 参数 `obj` + `.length` + 索引;computed
    /// 在子 widget,实参走 `.store.messages` 扁平化;for 以 computed 为源。
    /// active_leaf 非空(走 `.length`+索引路径),消息为对象(带 .id 字段,
    /// 与 musk messages[i].id 同构)。
    #[test]
    fn widget_computed_store_arg_helper_chain() {
        let src = concat!(
            "fn chatActivePath(messages obj, leaf str) -> obj {\n",
            "    if leaf == None || leaf == \"\" { return messages }\n",
            "    let n = messages.length\n",
            "    var i = 0\n",
            "    var out list = []\n",
            "    while i < n {\n",
            "        if messages[i].id == leaf {\n",
            "            out.push(messages[i])\n",
            "        }\n",
            "        i = i + 1\n",
            "    }\n",
            "    return out\n",
            "}\n",
            "fn chatSearchFilter(messages obj, q str) -> obj {\n",
            "    if q == None { return messages }\n",
            "    if q.trim() == \"\" { return messages }\n",
            "    return messages\n",
            "}\n",
            "widget Root53a {\n",
            "    model { var messages []Value = []\n    var active_leaf str = \"\"\n    var chat_search str = \"\" }\n",
            "    view { col { List53a {} } }\n",
            "}\n",
            "widget List53a {\n",
            "    model { var n int = 0 }\n",
            "    computed { filtered => chatSearchFilter(chatActivePath(.store.messages, .store.active_leaf), .store.chat_search) }\n",
            "    view { col { for m in .filtered { text \"row\" } } }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut decls: Vec<crate::ast::WidgetDecl> = vec![];
        let mut import_stmts: Vec<crate::ast::Stmt> = vec![];
        for st in &ast.stmts {
            match st {
                crate::ast::Stmt::WidgetDecl(d) => decls.push(d.clone()),
                crate::ast::Stmt::Fn(_) => import_stmts.push(st.clone()),
                _ => {}
            }
        }
        assert_eq!(decls.len(), 2, "root + child");
        let root_widget =
            crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        let child_widget =
            crate::aura::extract_widget_from_decl(&decls[1]).expect("extract child");
        registry.register(child_widget);

        let mut comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            registry,
            import_stmts,
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component");
        let mk = |id: &str, role: &str| {
            auto_val::Value::Obj(
                auto_val::Obj::new()
                    .with("id", auto_val::Value::str(id))
                    .with("role", auto_val::Value::str(role)),
            )
        };
        comp.write_state_vec(
            "messages",
            vec![mk("m1", "user"), mk("m2", "assistant"), mk("m3", "user")],
        )
        .unwrap();
        // 非空 leaf:强制走 `.length` + `messages[i].id` 全路径。
        comp.write_state("active_leaf", auto_val::Value::str("m3")).unwrap();
        let (view, _, _) = comp.view_with_debug_gated(false);
        let rows = count_text_nodes(&view, "row");
        assert_eq!(rows, 1, "computed 链式 helper(.length+索引+.id)必须解出 1 行(leaf=m3)");
    }

    /// P-053-1 续(悬垂透传):computed 透传链多帧重估——call_vm_fn 编组
    /// 堆引用实参原裸 push_nv 无 stake,RET 释放逐帧烧 state 份额 → 列表
    /// 对象回收成悬垂 id,消息气泡整体空(实机 [VM-IDX] no-heap-object)。
    /// 多次 view 重估(leaf 空 → helper 直通)后列表仍须可迭代。
    #[test]
    fn widget_computed_passthrough_survives_reeval() {
        let src = concat!(
            "fn chatSearchFilter(messages obj, q str) -> obj {\n",
            "    if q == None { return messages }\n",
            "    if q.trim() == \"\" { return messages }\n",
            "    return messages\n",
            "}\n",
            "widget Root53b {\n",
            "    model { var messages []Value = []\n    var chat_search str = \"\" }\n",
            "    view { col { List53b {} } }\n",
            "}\n",
            "widget List53b {\n",
            "    model { var n int = 0 }\n",
            "    computed { filtered => chatSearchFilter(.store.messages, .store.chat_search) }\n",
            "    view { col { for m in .filtered { text \"row\" } } }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut decls: Vec<crate::ast::WidgetDecl> = vec![];
        let mut import_stmts: Vec<crate::ast::Stmt> = vec![];
        for st in &ast.stmts {
            match st {
                crate::ast::Stmt::WidgetDecl(d) => decls.push(d.clone()),
                crate::ast::Stmt::Fn(_) => import_stmts.push(st.clone()),
                _ => {}
            }
        }
        let root_widget =
            crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        let child_widget =
            crate::aura::extract_widget_from_decl(&decls[1]).expect("extract child");
        registry.register(child_widget);

        let mut comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            registry,
            import_stmts,
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component");
        comp.write_state_vec(
            "messages",
            vec![auto_val::Value::str("m1"), auto_val::Value::str("m2")],
        )
        .unwrap();
        // 多帧重估(设备每帧重算 computed 的形态):透传链不得烧穿引用计数。
        let mut rows_last = 0;
        for _ in 0..8 {
            let (view, _, _) = comp.view_with_debug_gated(false);
            rows_last = count_text_nodes(&view, "row");
        }
        assert_eq!(rows_last, 2, "computed 透传链 8 帧重估后仍须解出 2 行(悬垂引用回归)");
    }
}

/// P-053-4: merged 模式下 #[api] no-op 显式告警。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p053_4_merged_api_warning {
    use crate::parser::Parser;

    #[test]
    fn merged_mode_api_call_emits_warn_opcode() {
        let src = concat!(
            "#[api(method = \"GET\", path = \"/api/chats/sessions\")]\n",
            "fn chats_list_sessions() SessionListResponse { return None }\n",
            "widget App {\n",
            "    model { var count int = 0 }\n",
            "    msg Msg { Fetch }\n",
            "    on { .Fetch -> { let r = chats_list_sessions(); } }\n",
            "    view { text \"app\" }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut decls: Vec<crate::ast::WidgetDecl> = vec![];
        let mut import_stmts: Vec<crate::ast::Stmt> = vec![];
        for st in &ast.stmts {
            match st {
                crate::ast::Stmt::WidgetDecl(d) => decls.push(d.clone()),
                crate::ast::Stmt::Fn(_) => import_stmts.push(st.clone()),
                _ => {}
            }
        }
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
        let (module, _) = crate::ui::handler_codegen::synthesize_widget_module(
            &root_widget,
            &[],
            import_stmts,
            &std::collections::HashMap::new(),
            false, // merged mode (api_over_http = false)
        )
        .expect("synthesize");

        // Bytecode must contain CALL_NAT 3142 (auto.vm.warn_api_noop)
        let has_warn_call = module.code.windows(4).any(|w| {
            w[0] == crate::vm::opcode::OpCode::CALL_NAT as u8 && u16::from_le_bytes([w[1], w[2]]) == 3142
        });
        assert!(has_warn_call, "merged mode #[api] call must emit CALL_NAT 3142");
    }
}

/// P-053-5: localStorage.getItem 字符串入池在 debug 模式下不踩 RC canary。
#[cfg(test)]
mod musk_vm_track_p053_5_localstorage_rc_canary {
    use crate::run_with_capture;

    #[test]
    fn localstorage_get_item_canary_safe() {
        let code = r#"
localStorage.setItem("test_key", "test_value_123")
let v = localStorage.getItem("test_key")
print(v)
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "localStorage get_item should run without canary panic: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        assert!(stdout.contains("test_value_123"), "expected test_value_123, got: [{}]", stdout);
    }
}

/// P-053-7: Widget .Init 内 bare Sibling() 调用与 .Sibling() 均正确转译派发。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p053_7_sibling_handler_calls {
    use crate::parser::Parser;

    #[test]
    fn store_init_bare_and_dot_sibling_calls_rewritten() {
        let src = concat!(
            "widget SiblingWidget {\n",
            "    model { var loaded bool = false }\n",
            "    msg Msg { Init, DoLoad }\n",
            "    on {\n",
            "        .Init -> { DoLoad() }\n",
            "        .DoLoad -> { .loaded = true }\n",
            "    }\n",
            "    view { text \"app\" }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut decls: Vec<crate::ast::WidgetDecl> = vec![];
        for st in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(d) = st {
                decls.push(d.clone());
            }
        }
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");

        let (module, _) = crate::ui::handler_codegen::synthesize_widget_module(
            &root_widget,
            &[],
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("synthesize");

        // Synthesized module contains handler_SiblingWidget_DoLoad export
        let has_handler = module.exports.iter().any(|(name, _)| {
            name.contains("handler_SiblingWidget_DoLoad")
        });
        assert!(has_handler, "expected handler_SiblingWidget_DoLoad in module exports: {:?}", module.exports.keys().collect::<Vec<_>>());
    }
}

/// P-053-M1: 失败响应与成功响应在 `resp != None && resp.session != None` 守卫下的行为。
#[cfg(test)]
mod musk_vm_track_p053_m1_guard_behavior {
    use crate::run_with_capture;

    #[test]
    fn error_object_fails_session_guard() {
        // 404 error response object
        let code = r#"
let resp = Json.to_value("{\"error\":\"HTTP 404\",\"status\":404}")
let guard = (resp != None) && (resp.session != None)
if guard {
    print("GUARD_PASSED")
} else {
    print("GUARD_BLOCKED")
}
"#;
        let (_code_res, stdout) = run_with_capture(code).expect("run");
        assert!(stdout.contains("GUARD_BLOCKED"), "404 error object must be blocked by guard, got: [{}]", stdout);
    }

    #[test]
    fn success_object_passes_session_guard() {
        // 200 success response object
        let code = r#"
let resp = Json.to_value("{\"session\":{\"id\":\"s123\",\"messages\":[]}}")
let guard = (resp != None) && (resp.session != None)
if guard {
    print("GUARD_PASSED")
} else {
    print("GUARD_BLOCKED")
}
"#;
        let (_code_res, stdout) = run_with_capture(code).expect("run");
        assert!(stdout.contains("GUARD_PASSED"), "200 success object must pass guard, got: [{}]", stdout);
    }
}

/// PLAN-053 批4: 普通 button 的 `title` prop → EE03 PUA tooltip 通道。
/// 现场：musk 会话侧栏 `button { title: .s.id ... }`——vue 轨 title 映射原生
/// 属性，VM 轨此前静默丢弃（EE03 只有 toolbar 合成按钮在埋）。接线后
/// renderer Button 臂剥 EE03 包 iced tooltip；snapshot 侧剥离为独立 title prop。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p053_b4_title_tooltip {
    use crate::parser::Parser;
    use crate::ui::view::View;

    fn build_root() -> crate::ui::dynamic::DynamicComponent {
        let src = concat!(
            "widget Root53t {\n",
            "    view {\n",
            "        col {\n",
            "            button {\n",
            "                title: \"sess-053-id\"\n",
            "                text \"你好\"\n",
            "            }\n",
            "            button {\n",
            "                text \"无提示\"\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decls: Vec<crate::ast::WidgetDecl> = ast
            .stmts
            .iter()
            .filter_map(|st| match st {
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .collect();
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
        crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component")
    }

    fn collect_buttons<'a>(
        view: &'a View<crate::ui::interpreter::DynamicMessage>,
        out: &mut Vec<&'a View<crate::ui::interpreter::DynamicMessage>>,
    ) {
        match view {
            View::Button { .. } => out.push(view),
            View::Column { children, .. } | View::Row { children, .. } => {
                for c in children {
                    collect_buttons(c, out);
                }
            }
            _ => {}
        }
    }

    /// 带 title 的按钮：label 必须以 EE03+title 收尾（renderer 剥离后包
    /// iced tooltip 的载体约定）。
    #[test]
    fn title_prop_rides_ee03_marker_in_label() {
        let comp = build_root();
        let (view, _, _) = comp.view_with_debug_gated(false);
        let mut buttons = Vec::new();
        collect_buttons(&view, &mut buttons);
        assert_eq!(buttons.len(), 2, "两个 button 都要转出, got {}", buttons.len());
        let labeled = buttons
            .iter()
            .map(|b| match b {
                View::Button { label, .. } => label.clone(),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        eprintln!("[P053-b4] button labels: {:?}", labeled);
        assert!(
            labeled.contains(&format!("你好\u{EE03}sess-053-id")),
            "title prop 必须以 EE03 尾段进 label(iced tooltip 通道), got: {:?}",
            labeled
        );
    }

    /// 无 title 的按钮：label 不得携带 EE03（控制组，防误埋）。
    #[test]
    fn button_without_title_has_no_ee03() {
        let comp = build_root();
        let (view, _, _) = comp.view_with_debug_gated(false);
        let mut buttons = Vec::new();
        collect_buttons(&view, &mut buttons);
        let plain = buttons
            .iter()
            .find_map(|b| match b {
                View::Button { label, .. } if label.starts_with("无提示") => Some(label.clone()),
                _ => None,
            })
            .expect("plain button");
        assert_eq!(plain, "无提示", "无 title 的 label 保持纯净, got: {:?}", plain);
    }

    /// snapshot：EE03 剥离为独立 title prop（MCP 断言面直接可读）。
    #[test]
    fn snapshot_exposes_title_prop_and_clean_label() {
        let comp = build_root();
        let (view, id_map, _) = comp.view_with_debug_gated(false);
        let state = std::collections::HashMap::new();
        let snap = crate::ui::snapshot_builder::SnapshotBuilder::build(
            "Root53t", &state, &view, &id_map,
        );
        // UiNode 树递归找 Button 节点的 props。
        fn walk(
            node: &crate::ui::mcp_types::UiNode,
            out: &mut Vec<(String, Vec<(String, String)>)>,
        ) {
            if node.kind == "Button" {
                out.push((
                    node.props
                        .iter()
                        .find(|(k, _)| k == "label")
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default(),
                    node.props.clone(),
                ));
            }
            for c in &node.children {
                walk(c, out);
            }
        }
        let mut found = Vec::new();
        walk(&snap.tree, &mut found);
        eprintln!("[P053-b4] snapshot buttons: {:?}", found);
        let with_title = found
            .iter()
            .find(|(label, _)| label == "你好")
            .expect("titled button in snapshot");
        assert!(
            with_title
                .1
                .iter()
                .any(|(k, v)| k == "title" && v == "sess-053-id"),
            "snapshot 必须暴露 title prop, got: {:?}",
            with_title.1
        );
        assert!(
            !found.iter().any(|(label, _)| label.contains('\u{EE03}')),
            "snapshot label 不得残留 EE03 标记, got: {:?}",
            found
        );
        assert_eq!(found.len(), 2, "两个按钮都进快照");
    }

    /// vue 轨 codegen 对照：button 的 `title` 必须落到模板属性——表达式
    /// `title: .s.id` → `:title="s.id"`，字面量 → 绑定常量（与 variant/size
    /// 臂同型，Vue 语义等价）。此前 shadcn Button 臂静默丢弃 title（原生
    /// span 路径透传，仅 Button 丢），web 轨所有 button tooltip 失效。
    #[test]
    fn vue_codegen_emits_title_attr_on_button() {
        use crate::ui_gen::{BackendGenerator, VueGenerator};
        let src = concat!(
            "widget Root53v {\n",
            "    model { var session_list []Value = [] }\n",
            "    view {\n",
            "        col {\n",
            "            button {\n",
            "                title: \"sess-literal\"\n",
            "                text \"静态\"\n",
            "            }\n",
            "            for s in .session_list {\n",
            "                button {\n",
            "                    key: .s.id\n",
            "                    title: .s.id\n",
            "                    text .s.name\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|st| match st {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract");
        let sfc = VueGenerator::new().generate(&widget).expect("generate");
        eprintln!("[P053-b4] vue SFC:\n{}", sfc);
        assert!(
            sfc.contains(":title=\"'sess-literal'\""),
            "静态 title 必须落 title 属性(绑定常量形态), SFC:\n{}", sfc
        );
        assert!(
            sfc.contains(":title=\"s.id\""),
            "表达式 title 必须落 :title 绑定, SFC:\n{}", sfc
        );
    }
}


/// PLAN-054 T1 (R1): EE03 title 标记不得落可见文本流——内容子树按钮的
/// leading Text 泄漏。现场（musk chats_view.at A1/A2）：`button { title: …
/// span { text .s.name } }`——子文本是 child node 而非 text prop，
/// extract_children_text 落 None → label_from_prop=true，而 EE03 后缀先行
/// 拼进 label → leading Text("EE03+title") 常显（卡片首行 "Y<id>"、
/// "Y新建会话"、"Y设置" 的根因）。修复后：内容子树用干净 label，
/// EE03 只留在 Button.label 走 renderer tooltip 通道。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t1_title_content_subtree {
    use crate::parser::Parser;
    use crate::ui::view::View;

    fn build_root() -> crate::ui::dynamic::DynamicComponent {
        let src = concat!(
            "widget Root54t {\n",
            "    view {\n",
            "        col {\n",
            "            button {\n",
            "                title: \"sess-054-id\"\n",
            "                span {\n",
            "                    text \"你好\"\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decls: Vec<crate::ast::WidgetDecl> = ast
            .stmts
            .iter()
            .filter_map(|st| match st {
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .collect();
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
        crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component")
    }

    /// title 仍以 EE03 尾段进 Button.label（renderer tooltip 通道不回退），
    /// 但任何 View::Text 的可见内容不得携带 EE03（PUA 字形不落文本流）。
    #[test]
    fn title_ee03_stays_out_of_visible_text_stream() {
        let comp = build_root();
        let (view, _, _) = comp.view_with_debug_gated(false);
        // 递归收集 (Button labels, Text contents)。
        fn walk(
            view: &View<crate::ui::interpreter::DynamicMessage>,
            labels: &mut Vec<String>,
            texts: &mut Vec<String>,
        ) {
            match view {
                View::Button { label, content, .. } => {
                    labels.push(label.clone());
                    if let Some(c) = content {
                        walk(c, labels, texts);
                    }
                }
                View::Text { content, .. } => texts.push(content.clone()),
                View::Row { children, .. } | View::Column { children, .. } => {
                    for c in children {
                        walk(c, labels, texts);
                    }
                }
                View::Container { child, .. } => walk(child, labels, texts),
                _ => {}
            }
        }
        let mut labels = Vec::new();
        let mut texts = Vec::new();
        walk(&view, &mut labels, &mut texts);
        eprintln!("[P054-T1] button labels: {:?}, text contents: {:?}", labels, texts);
        assert!(
            labels.iter().any(|l| l.ends_with(&format!("\u{EE03}sess-054-id"))),
            "Button.label 必须保留 EE03+title 尾段(tooltip 通道), got: {:?}",
            labels
        );
        assert!(
            texts.iter().all(|t| !t.contains('\u{EE03}')),
            "可见文本流不得携带 EE03 PUA 标记, got: {:?}",
            texts
        );
        assert!(
            texts.iter().any(|t| t == "你好"),
            "span 子节点文本必须照常渲染, got: {:?}",
            texts
        );
    }
}


/// P-053-8: 二级导航点击会话实参漂移——UI 事件层携带正确 id
/// （encode `Pick\u{1f}s\u{1f}<id>`），handler 体内参数却读到会话名
/// （"你好"）。实机日志（plan053-batch4-vm.log）：
///   [VM_HANDLER_CALL] args=[Str("8f20138…")]  ← 正确
///   [VM_EXEC]          args=[Str("8f20138…")]  ← 正确
///   [ChatsView.SelectSession] ENTER id=你好    ← 跑偏
/// 且逐项确定：13e16… 两次点击均正确，8f20…（名"你好"）两次均漂移。
/// 嫌疑：字符串池索引 u16 截断回绕（add_string 2026-08-22 注释登记的
/// 引擎债）或 NV 负 i32 编码途经 i32 通道后的索引偏移。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p053_8_click_arg_drift {
    use crate::parser::Parser;

    fn build_root() -> crate::ui::dynamic::DynamicComponent {
        let src = concat!(
            "widget Root53r {\n",
            "    model {\n",
            "        var session_list []Value = []\n",
            "        var got str = \"\"\n",
            "    }\n",
            "    msg Msg { Pick(str), Churn }\n",
            "    on {\n",
            "        .Pick(id) -> {\n",
            "            .got = id\n",
            "        }\n",
            "        .Churn -> {\n",
            "            var s = \"\"\n",
            "            for i in 0..70000 {\n",
            "                s = \"x\" + i\n",
            "            }\n",
            "            .got = s\n",
            "        }\n",
            "    }\n",
            "    view {\n",
            "        col {\n",
            "            for s in .session_list {\n",
            "                button {\n",
            "                    key: .s.id\n",
            "                    onclick: .Pick(.s.id)\n",
            "                    text .s.name\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decls: Vec<crate::ast::WidgetDecl> = ast
            .stmts
            .iter()
            .filter_map(|st| match st {
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .collect();
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
        crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component")
    }

    fn seed_sessions(dc: &mut crate::ui::dynamic::DynamicComponent) {
        dc.write_state_vec(
            "session_list",
            vec![
                auto_val::Value::Obj(
                    auto_val::Obj::new()
                        .with("id", auto_val::Value::str("8f20138cab63f0c24832d3fb"))
                        .with("name", auto_val::Value::str("你好")),
                ),
                auto_val::Value::Obj(
                    auto_val::Obj::new()
                        .with("id", auto_val::Value::str("13e16478f80c91da604b87e7"))
                        .with("name", auto_val::Value::str("alpha")),
                ),
            ],
        )
        .unwrap();
    }

    fn got_of(dc: &crate::ui::dynamic::DynamicComponent) -> String {
        match dc.read_state("got").expect("got readable") {
            auto_val::Value::Str(s) => s.as_str().to_string(),
            v => format!("{:?}", v),
        }
    }

    /// 最小复现：池低位时点击实参必须原样到达 handler。
    #[test]
    fn pick_arg_intact_small_pool() {
        let mut dc = build_root();
        seed_sessions(&mut dc);
        let _ = dc.view_with_debug_gated(false);
        dc.on_with_input_for("Root53r", "Pick\u{1F}s\u{1F}8f20138cab63f0c24832d3fb", None);
        assert_eq!(got_of(&dc), "8f20138cab63f0c24832d3fb");
        dc.on_with_input_for("Root53r", "Pick\u{1F}s\u{1F}13e16478f80c91da604b87e7", None);
        assert_eq!(got_of(&dc), "13e16478f80c91da604b87e7");
    }

    /// 池膨胀复现（u16 回绕假说）：池条目超 65535 后实参必须仍原样到达。
    #[test]
    fn pick_arg_intact_after_pool_churn() {
        let mut dc = build_root();
        seed_sessions(&mut dc);
        let _ = dc.view_with_debug_gated(false);
        dc.on_with_input_for("Root53r", "Churn", None);
        dc.on_with_input_for("Root53r", "Pick\u{1F}s\u{1F}8f20138cab63f0c24832d3fb", None);
        assert_eq!(
            got_of(&dc),
            "8f20138cab63f0c24832d3fb",
            "池膨胀后点击实参漂移（P-053-8 现场：id 变会话名）"
        );
    }
}

/// P-053-8 语料级复现：test/ui/plan053_p8_click_arg（生产路径构建，
/// store 兄弟调用 + 扁平化同名状态 + 循环按钮带参 onclick 全保真）。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p053_8_corpus {
    fn build() -> Option<crate::ui::dynamic::DynamicComponent> {
        let rel = "test/ui/plan053_p8_click_arg/src/front/app.at";
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

    fn seed(dc: &mut crate::ui::dynamic::DynamicComponent) {
        dc.write_state_vec(
            "session_list",
            vec![
                auto_val::Value::Obj(
                    auto_val::Obj::new()
                        .with("id", auto_val::Value::str("8f20138cab63f0c24832d3fb"))
                        .with("name", auto_val::Value::str("你好")),
                ),
                auto_val::Value::Obj(
                    auto_val::Obj::new()
                        .with("id", auto_val::Value::str("13e16478f80c91da604b87e7"))
                        .with("name", auto_val::Value::str("alpha")),
                ),
            ],
        )
        .unwrap();
    }

    fn str_state(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
        match dc.read_state(field).expect(field) {
            auto_val::Value::Str(s) => s.as_str().to_string(),
            v => format!("{:?}", v),
        }
    }

    /// 点击「你好」（id 8f20…）：handler 实参必须是 id，不是会话名。
    #[test]
    fn click_arg_is_id_not_name() {
        let Some(mut dc) = build() else {
            eprintln!("P053-8 corpus: SKIPPED — app.at not found");
            return;
        };
        seed(&mut dc);
        let _ = dc.view_with_debug_gated(false);
        dc.on_with_input_for("Child53", "Pick\u{1F}s\u{1F}8f20138cab63f0c24832d3fb", None);
        assert_eq!(
            str_state(&dc, "debug_click_id"),
            "8f20138cab63f0c24832d3fb",
            "P-053-8: 点击实参漂移成会话名"
        );
        assert_eq!(
            str_state(&dc, "session_id"),
            "8f20138cab63f0c24832d3fb",
            "P-053-8: store 兄弟调用实参同源漂移"
        );
    }

    /// 对照组：点击「alpha」（id 13e1…）。
    #[test]
    fn click_arg_control_second_item() {
        let Some(mut dc) = build() else {
            eprintln!("P053-8 corpus: SKIPPED — app.at not found");
            return;
        };
        seed(&mut dc);
        let _ = dc.view_with_debug_gated(false);
        dc.on_with_input_for("Child53", "Pick\u{1F}s\u{1F}13e16478f80c91da604b87e7", None);
        assert_eq!(str_state(&dc, "debug_click_id"), "13e16478f80c91da604b87e7");
    }
}

/// P-053-8 根因回归:字符串池 dedup 残键指向已复用槽——add_string 命中侧
/// 内容校验必须把它转为干净重内化。现场(POOLLOG 实测 #222→#223):键
/// "8f20…"→槽 2348 的条目存活(rc=1)时槽被幻影 freelist 条目复用覆写为
/// "你好"而旧键未删,后续 add_string("8f20…") 残键命中返回 2348,点击
/// 会话实参由 id 漂移成会话名。
#[cfg(test)]
mod musk_vm_track_p053_8_stale_key_selfheal {
    use crate::vm::engine::AutoVM;
    use crate::vm::virt_memory::VirtualFlash;

    #[test]
    fn stale_dedup_key_reinterns_cleanly() {
        let vm = AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024);
        let hello = vm.add_string("你好".as_bytes().to_vec());
        // 人为注入残键: "8f20…" → 你好所在槽(幻影 freelist 复用覆写后
        // 旧键未删的现场形态)。
        vm.string_dedup
            .lock()
            .unwrap()
            .insert("8f20138cab63f0c24832d3fb".as_bytes().to_vec(), hello);
        let idx = vm.add_string("8f20138cab63f0c24832d3fb".as_bytes().to_vec());
        let got = vm.get_string(idx as u32).expect("slot readable");
        assert_eq!(
            got,
            "8f20138cab63f0c24832d3fb".as_bytes(),
            "残键命中必须重内化为内容一致的新槽,不得返回它串的槽"
        );
        assert_ne!(idx, hello, "重内化必须离开被污染槽");
        // 残键已被重内化 insert 覆盖:同字节再次内化命中新槽且内容一致。
        let again = vm.add_string("8f20138cab63f0c24832d3fb".as_bytes().to_vec());
        assert_eq!(again, idx, "自愈后同键内化应稳定命中新槽");
        assert_eq!(
            vm.get_string(again as u32).unwrap(),
            "8f20138cab63f0c24832d3fb".as_bytes()
        );
    }
}

/// P-053-8 续:幻影 freelist 条目清扫——rc>0 的槽是存活槽,freelist 弹出
/// 时必须丢弃该条目并跳过,绝不复用(复用=覆写活内容+清零 rc→孤儿
/// release 下溢风暴自续;实测槽 49299 rc=5 被复用后 rc=4294967295,
/// musk store 兄弟调用实参读到后落的 404 JSON)。
#[cfg(test)]
mod musk_vm_track_p053_8_phantom_freelist {
    use crate::vm::engine::AutoVM;
    use crate::vm::virt_memory::VirtualFlash;

    #[test]
    fn phantom_entry_dropped_live_slot_never_stolen() {
        let vm = AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024);
        let live = vm.add_string("你好".as_bytes().to_vec());
        vm.pool_retain(live);
        vm.pool_retain(live); // rc=2:两个存活持有
        // 人为注入幻影条目(现场形态:存活槽进入 freelist)。
        vm.pool_state.write().unwrap().freelist.push(live);
        // 后续内化不得偷该槽。
        let other = vm.add_string("other".as_bytes().to_vec());
        assert_ne!(other, live, "幻影条目必须被丢弃,存活槽不得复用");
        assert_eq!(
            vm.get_string(live as u32).unwrap(),
            "你好".as_bytes(),
            "存活槽内容不得被覆写"
        );
        // 幻影条目已被清扫:freelist 不再含该槽。
        assert!(
            !vm.pool_state.read().unwrap().freelist.contains(&live),
            "幻影条目应被消费丢弃"
        );
        // 同字节内化仍命中存活槽。
        let again = vm.add_string("你好".as_bytes().to_vec());
        assert_eq!(again, live);
    }
}
