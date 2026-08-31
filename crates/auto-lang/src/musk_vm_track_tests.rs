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
