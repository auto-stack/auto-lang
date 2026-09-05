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

    /// PLAN-054 T2 (R2)：内容子树按钮 icon 子组件——R2 全集逐图标回归锁。
    /// 09-01 对拍 A2/A10 现场声明的丢失集（Plus/Trash2/Send/Folder/
    /// ChevronDown/Settings）+ 对照组（Search/Info）。P-051 P2-①（模块级
    /// use.web 注册）+ P-053-6（registry 守卫）修复后桥路已通，本测锁死
    /// "button 内容子树 + title" 形态下每个图标都以 lucide:{kebab} 进视图。
    #[test]
    fn icon_component_child_renders_in_button_content_subtree() {
        crate::ui::aura_view_builder::register_imported_components(vec![
            "Plus".to_string(),
            "Trash2".to_string(),
            "Search".to_string(),
            "Info".to_string(),
            "Send".to_string(),
            "Folder".to_string(),
            "ChevronDown".to_string(),
            "Settings".to_string(),
        ]);
        // (组件名, 期望 kebab glyph)——R2 丢失集在前，对照组在后。
        let cases: &[(&str, &str)] = &[
            ("Plus", "lucide:plus"),
            ("Trash2", "lucide:trash-2"),
            ("Send", "lucide:send"),
            ("Folder", "lucide:folder"),
            ("ChevronDown", "lucide:chevron-down"),
            ("Settings", "lucide:settings"),
            ("Search", "lucide:search"),
            ("Info", "lucide:info"),
        ];
        for (tag, want) in cases {
            let src = format!(
                "widget Root54i {{\n    view {{\n        button {{\n            title: \"t-{tag}\"\n            {tag} {{ size: 14 }}\n        }}\n    }}\n}}\n"
            );
            let session = crate::session::CompilerSession::ui();
            let mut parser = Parser::from(src.as_str()).with_session(session);
            let ast = parser.parse().unwrap_or_else(|e| panic!("{tag}: parse {e}"));
            let decls: Vec<crate::ast::WidgetDecl> = ast
                .stmts
                .iter()
                .filter_map(|st| match st {
                    crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                    _ => None,
                })
                .collect();
            let root_widget =
                crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
            let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
                &decls[0],
                &decls[1..],
                &root_widget,
                crate::ui::widget_registry::WidgetRegistry::new(),
                vec![],
                &std::collections::HashMap::new(),
                false,
            )
            .expect("component");
            let (view, _, _) = comp.view_with_debug_gated(false);
            fn first_image(
                view: &View<crate::ui::interpreter::DynamicMessage>,
                img: &mut Option<String>,
            ) {
                if img.is_some() {
                    return;
                }
                match view {
                    View::Image { src, .. } => *img = Some(src.clone()),
                    View::Button { content: Some(c), .. } => first_image(c, img),
                    View::Row { children, .. } | View::Column { children, .. } => {
                        for c in children {
                            first_image(c, img);
                        }
                    }
                    View::Container { child, .. } => first_image(child, img),
                    _ => {}
                }
            }
            let mut got = None;
            first_image(&view, &mut got);
            assert_eq!(
                got.as_deref(),
                Some(*want),
                "{tag} 必须以 {want} 渲染进 button 内容子树"
            );
        }
        crate::ui::aura_view_builder::clear_imported_components();
    }

    /// title 仍以 EE03 尾段进 Button.label（renderer tooltip 通道不回退），
    /// 但任何 View::Text 的可见内容不得携带 EE03（PUA 字形不落文本流）。
    #[test]
    fn title_ee03_stays_out_of_visible_text_stream() {
        crate::ui::aura_view_builder::clear_imported_components();
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


/// PLAN-054 手动探针（#[ignore]，需 MUSK_APP_PATH 指向真实 musk app.at）：
/// 走生产管线 build_dynamic_component（与 auto run --render=vm 同装载路径，
/// 含 register_imported_components 三路注册）渲染一帧，倒出全部 Image src
/// 与 Text 内容——裁定图标丢失发生在注册面/视图面/渲染面哪一层。
/// 运行：MUSK_APP_PATH=D:/autostack/auto-musk/src/front/app.at \
///   cargo nextest run -p auto-lang --lib --features ui-iced musk_runtime_icon -- --ignored --nocapture
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_runtime_probe {
    #[test]
    #[ignore = "requires MUSK_APP_PATH pointing at real musk checkout"]
    fn musk_runtime_icon_and_text_dump() {
        let app = std::path::PathBuf::from(
            std::env::var("MUSK_APP_PATH").expect("MUSK_APP_PATH required"),
        );
        let code = std::fs::read_to_string(&app).expect("read app.at");
        let mut dc = crate::build_dynamic_component(&code, Some(app.to_str().unwrap()))
            .expect("production loader build");
        // 过 auth guard（app.at: authenticated => token != None computed）。
        let fields: Vec<String> = dc.state_fields().iter().map(|f| format!("{:?}", f)).collect();
        eprintln!("[P054-probe] state fields: {:?}", fields);
        let authed = dc.write_state("token", auto_val::Value::Str("probe-token".into()));
        authed.expect("write token");
        // T3 R3 勘察：给 .current 填真值，看 ${currentName}/${currentTitle}
        // 字面量是否被求值（workspace_selector computed 形态）。
        let ws = auto_val::Obj::new()
            .with("id", auto_val::Value::str("ws-1"))
            .with("name", auto_val::Value::str("musk-demo"))
            .with("path", auto_val::Value::str("D:\\autostack\\auto-musk"));
        dc.write_state("current", auto_val::Value::Obj(ws))
            .expect("write current");
        // T3 R3 勘察续：VM flash 里 computed 是否编成同名 fn + call_vm_fn 直调。
        let bridge = dc.bridge();
        let names: Vec<String> = bridge
            .vm()
            .flash
            .exports_by_name
            .keys()
            .filter(|k| k.contains("current") || k.contains("Name") || k.contains("Title"))
            .cloned()
            .collect();
        eprintln!("[P054-probe] vm fn exports matching current/Name/Title: {:?}", names);
        for cand in ["currentName", "currentTitle", "WorkspaceSelector.currentName"] {
            match bridge.call_vm_fn(cand, &[]) {
                Ok(v) => eprintln!("[P054-probe] call_vm_fn({cand}) = {:?}", v),
                Err(e) => eprintln!("[P054-probe] call_vm_fn({cand}) ERR: {:?}", e),
            }
        }
        for cand in ["ws_load_current", "ws_load_recent"] {
            match bridge.call_vm_fn(cand, &[]) {
                Ok(v) => eprintln!("[P054-probe] call_vm_fn({cand}) = {:?}", v),
                Err(e) => eprintln!("[P054-probe] call_vm_fn({cand}) ERR: {:?}", e),
            }
        }
        eprintln!(
            "[P054-probe] root read_state(current) pre-view = {:?}",
            bridge.read_state("current")
        );
        let (view, _, _) = dc.view_with_debug_gated(false);
        eprintln!(
            "[P054-probe] root read_state(current) post-view = {:?}",
            dc.bridge().read_state("current")
        );
        fn walk(
            view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
            imgs: &mut Vec<String>,
            texts: &mut Vec<String>,
            depth: usize,
        ) {
            use crate::ui::view::View;
            if depth > 40 {
                return;
            }
            match view {
                View::Image { src, .. } => imgs.push(src.clone()),
                View::Text { content, .. } => texts.push(content.clone()),
                View::Button { label, content, .. } => {
                    texts.push(format!("[btn label={:?}]", label));
                    if let Some(c) = content {
                        walk(c, imgs, texts, depth + 1);
                    }
                }
                View::Row { children, .. } | View::Column { children, .. } => {
                    for c in children {
                        walk(c, imgs, texts, depth + 1);
                    }
                }
                View::Container { child, .. } => walk(child, imgs, texts, depth + 1),
                _ => {}
            }
        }
        let mut imgs = Vec::new();
        let mut texts = Vec::new();
        walk(&view, &mut imgs, &mut texts, 0);
        eprintln!("[P054-probe] image srcs ({}): {:?}", imgs.len(), imgs);
        eprintln!("[P054-probe] texts ({}):", texts.len());
        for t in &texts {
            eprintln!("  text: {:?}", t);
        }
    }
}

/// PLAN-054 T3 (R3/R4): 文本插值 computed 形态求值 + i18n {'x'} 转义。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t3_interp_i18n {
    use crate::parser::Parser;

    fn build(src: &str) -> crate::ui::dynamic::DynamicComponent {
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

    fn texts_of(comp: &crate::ui::dynamic::DynamicComponent) -> Vec<String> {
        let (view, _, _) = comp.view_with_debug_gated(false);
        fn walk(view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>, out: &mut Vec<String>) {
            use crate::ui::view::View;
            match view {
                View::Text { content, .. } => out.push(content.clone()),
                View::Button { label, content, .. } => {
                    out.push(format!("[label {:?}]", label));
                    if let Some(c) = content {
                        walk(c, out);
                    }
                }
                View::Row { children, .. } | View::Column { children, .. } => {
                    for c in children {
                        walk(c, out);
                    }
                }
                View::Container { child, .. } => walk(child, out),
                _ => {}
            }
        }
        let mut out = Vec::new();
        walk(&view, &mut out);
        out
    }

    /// 根 widget 形态：computed if（.current != None → .current.name）。
    /// workspace_selector.at:23 的 currentName 同款。此前显示字面
    /// "${currentName}"（read_state Err 兜底臂）。
    #[test]
    fn computed_if_text_resolves_in_root_widget() {
        let src = concat!(
            "widget Root54c {\n",
            "    model { var current obj = None }\n",
            "    computed {\n",
            "        currentName => if .current != None { .current.name } else { \"选择工作目录\" }\n",
            "    }\n",
            "    view {\n",
            "        col {\n",
            "            text .currentName\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let comp = build(src);
        let texts = texts_of(&comp);
        eprintln!("[P054-T3] root texts: {:?}", texts);
        assert!(
            texts.iter().any(|t| t == "选择工作目录"),
            "computed if 必须求值（current=None → else 分支）, got: {:?}",
            texts
        );
    }

    /// 子 widget 形态：computed 定义在子 widget，父视图实例化。
    /// 此前子 builder 的 computed 链路断裂 → 字面 "${currentName}"。
    #[test]
    fn computed_if_text_resolves_in_child_widget() {
        let src = concat!(
            "widget Selector54c {\n",
            "    computed {\n",
            "        currentName => if .current != None { .current.name } else { \"选择工作目录\" }\n",
            "        currentTitle => if .current != None { .current.path } else { \"选择工作目录\" }\n",
            "    }\n",
            "    model { var current obj = None }\n",
            "    view {\n",
            "        col {\n",
            "            text .currentName\n",
            "            text .currentTitle\n",
            "        }\n",
            "    }\n",
            "}\n",
            "widget Root54c2 {\n",
            "    view {\n",
            "        col {\n",
            "            Selector54c {}\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let comp = build(src);
        let texts = texts_of(&comp);
        eprintln!("[P054-T3] child texts: {:?}", texts);
        assert!(
            texts.iter().filter(|t| !t.starts_with("[label")).count() >= 2,
            "子 widget 两个 computed text 都要渲染, got: {:?}",
            texts
        );
        assert!(
            texts.iter().any(|t| t == "选择工作目录"),
            "子 widget computed if 必须求值, got: {:?}",
            texts
        );
        assert!(
            !texts.iter().any(|t| t.contains("${current")),
            "不得残留 ${{...}} 字面量, got: {:?}",
            texts
        );
    }
}

/// PLAN-054 T3 (R3) 生产装载面：`var current obj = None` 的 None 初始值
/// 经 VmBridge::new 状态种入必须是 Nil——此前 eval_expr_to_value 的 Ident
/// 臂把 None 解析为 Int(0)（"unresolved ident 零占位"），子作用域 computed
/// 链 `.current != None` 恒真 → `.current.path` 作用 Int(0) → None →
/// workspace 行显示字面 "${currentName}/${currentTitle}"（A4）。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t3_none_initial {
    use crate::parser::Parser;

    #[test]
    fn obj_model_none_initial_seeds_nil_in_vm_state() {
        let src = concat!(
            "widget Root54n {\n",
            "    model {\n",
            "        var current obj = None\n",
            "        var flag bool = false\n",
            "    }\n",
            "    view {\n",
            "        col {\n",
            "            text \"x\"\n",
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
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .expect("decl");
        let widget = crate::aura::extract_widget_from_decl(&decl).expect("extract");
        let bridge = crate::ui::vm_bridge::VmBridge::new(&widget).expect("bridge");
        let current = bridge.read_state("current").expect("read current");
        eprintln!("[P054-T3] current initial = {:?}", current);
        assert_eq!(
            current,
            auto_val::Value::Nil,
            "obj = None 初始值必须种入 Nil(此前 Int(0) 令 computed != None 恒真), got {:?}",
            current
        );
        let flag = bridge.read_state("flag").expect("read flag");
        assert_eq!(flag, auto_val::Value::Bool(false), "bool 初始值不变");
    }
}

/// PLAN-054 T3 (R3) VM 返回值形态：`.at` fn 体里裸 `return None` 必须以
/// NV nil 返回（call_vm_fn 解码为 Value::Nil）——P-053-2 只修了 null/nil
/// 字面量，大写 None（musk 惯用）经 Ident/未解析臂落 Int(0)，沿
/// `.current = ws_load_current()` 写进 state → `!= None` 恒真 →
/// `.current.path` 作用 Int(0) → workspace 行显示字面 ${currentTitle}（A4）。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t3_none_return {
    use crate::parser::Parser;

    pub(super) fn build_bridge(src: &str) -> crate::ui::vm_bridge::VmBridge {
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

    #[test]
    fn vm_fn_bare_none_return_is_nil_not_int0() {
        let src = concat!(
            "widget Root54nr {\n",
            "    view {\n",
            "        col {\n",
            "            text \"x\"\n",
            "        }\n",
            "    }\n",
            "}\n",
            "fn load() Value {\n",
            "    return None\n",
            "}\n",
        );
        let bridge = build_bridge(src);
        let v = bridge
            .call_vm_fn("load", &[])
            .expect("call load");
        eprintln!("[P054-T3] return None => {:?}", v);
        assert_eq!(
            v,
            auto_val::Value::Nil,
            "裸 return None 必须解出 Nil, got {:?}（Int(0) 沿状态写入扩散成 != None 恒真）",
            v
        );
    }
}

/// PLAN-054 T3 (R3)：computed if 的兜底语义——then 体求值失败（如 x 为
/// 零默认 Int(0) 时 `x.f` 解析 None）不得整链报废出 "${name}" 字面量，
/// 必须落到 else 兜底（`if x != None { x.f } else { fallback }` 意图）。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t3_if_fallback {
    use crate::parser::Parser;

    #[test]
    fn computed_if_falls_back_when_then_body_unresolvable() {
        let src = concat!(
            "widget Root54fb {\n",
            "    model { var current obj = None }\n",
            "    computed {\n",
            "        currentName => if .current != None { .current.name } else { \"选择工作目录\" }\n",
            "    }\n",
            "    view {\n",
            "        col {\n",
            "            text .currentName\n",
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
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .expect("decl");
        let widget = crate::aura::extract_widget_from_decl(&decl).expect("extract");
        let mut comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decl,
            &[],
            &widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component");
        // 模拟生产现场的零默认垃圾态（VM 装载/async 句柄写回 Int(0)）。
        comp.bridge_mut()
            .write_state("current", auto_val::Value::Int(0))
            .expect("write garbage current");
        let (view, _, _) = comp.view_with_debug_gated(false);
        fn texts(view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>, out: &mut Vec<String>) {
            use crate::ui::view::View;
            match view {
                View::Text { content, .. } => out.push(content.clone()),
                View::Row { children, .. } | View::Column { children, .. } => {
                    for c in children {
                        texts(c, out);
                    }
                }
                View::Container { child, .. } => texts(child, out),
                _ => {}
            }
        }
        let mut out = Vec::new();
        texts(&view, &mut out);
        eprintln!("[P054-T3] fallback texts: {:?}", out);
        assert!(
            out.iter().any(|t| t == "选择工作目录"),
            "then 体不可解析时必须落 else 兜底, got: {:?}",
            out
        );
        assert!(
            !out.iter().any(|t| t.contains("${currentName}")),
            "不得残留 ${{...}} 字面量, got: {:?}",
            out
        );
    }
}

/// PLAN-054 T4 (A9) 勘察：会话卡片样式链解析产物。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t4_style_probe {
    #[test]
    #[ignore = "manual probe"]
    fn session_card_style_parse_dump() {
        let selected = crate::ui::style::Style::parse(
            "session-item relative h-auto w-full flex flex-col items-start justify-start gap-0.5 text-left py-2.5 px-3 mb-1.5 rounded-lg border border-primary/25 bg-primary/10 text-primary",
        );
        let unselected = crate::ui::style::Style::parse(
            "session-item relative h-auto w-full flex flex-col items-start justify-start gap-0.5 text-left py-2.5 px-3 mb-1.5 rounded-lg border border-transparent bg-card hover:border-border hover:bg-accent text-foreground",
        );
        for (name, s) in [("selected", selected), ("unselected", unselected)] {
            let Ok(s) = s else { eprintln!("[P054-T4] {name}: PARSE FAIL"); continue };
            let is = crate::ui::style::iced_adapter::IcedStyle::from_style(&s);
            eprintln!("[P054-T4] {name}: border={} width={:?} color={:?} bg={:?}",
                is.border, is.border_width, is.border_color, is.background_color);
        }
    }
}

/// PLAN-054 T4 (A6/A9/A11) 回归锁：会话卡片/消息行样式链。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t4_styles {
    use crate::ui::style::{Style, StyleClass};
    use crate::ui::style::iced_adapter::IcedStyle;

    /// A9 锁①：border-primary/25 带解析（选中卡片淡描边）。
    /// 09-01 对拍"border alpha 未支持/描边过重"在当前 master 不成立,
    /// 此测钉死解析面(alpha 0.25 语义色)。
    #[test]
    fn border_primary_25_parses_with_alpha() {
        let s = Style::parse("border border-primary/25").expect("parse");
        let is = IcedStyle::from_style(&s);
        assert!(is.border, "border 宽度类必须在");
        let c = is.border_color.expect("border color");
        assert!((c.a - 0.25).abs() < 0.02, "alpha 必须为 25%, got {:?}", c);
    }

    /// A9 锁②：bg-card 语义解析 = musk dark --card(222.2 47% 10%) =
    /// rgb(13,21,38)（Plan 448 对齐,两轨一致）。
    /// Plan 518 stella 重校:dark Surface 翻精修蓝黑 #1a2235 = (26,34,53)。
    #[test]
    fn bg_card_matches_musk_dark_token() {
        let s = Style::parse("bg-card").expect("parse");
        let is = IcedStyle::from_style(&s);
        let c = is.background_color.expect("bg");
        assert_eq!((c.r * 255.0).round() as u8, 26, "r");
        assert_eq!((c.g * 255.0).round() as u8, 34, "g");
        assert_eq!((c.b * 255.0).round() as u8, 53, "b");
    }

    /// A11：图标组件 class prop 下传——ml-auto 贴行右端 + muted 着色,
    /// size 像素保持。
    #[test]
    fn icon_component_class_prop_carries_ml_auto_and_tint() {
        use crate::parser::Parser;
        use crate::ui::view::View;
        let src = concat!(
            "widget Root54t4 {\n",
            "    view {\n",
            "        row {\n",
            "            style: \"flex items-center gap-1 w-full\"\n",
            "            text \"N 条\"\n",
            "            Info { size: 11, class: \"text-muted-foreground shrink-0 ml-auto\" }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        crate::ui::aura_view_builder::register_imported_components(vec!["Info".to_string()]);
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
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract");
        let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0], &decls[1..], &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![], &std::collections::HashMap::new(), false,
        )
        .expect("component");
        let (view, _, _) = comp.view_with_debug_gated(false);
        fn find_image(view: &View<crate::ui::interpreter::DynamicMessage>) -> Option<&crate::ui::style::Style> {
            match view {
                View::Image { style, .. } => style.as_ref(),
                View::Row { children, .. } | View::Column { children, .. } => {
                    children.iter().find_map(find_image)
                }
                View::Container { child, .. } => find_image(child),
                _ => None,
            }
        }
        let img_style = find_image(&view).expect("icon image");
        let is = IcedStyle::from_style(img_style);
        assert!(is.margin_left_auto, "ml-auto 必须进图标样式(A11 此前整串丢弃)");
        assert!(is.text_color.is_some(), "text-muted-foreground 着色必须在");
        assert!(
            img_style.classes.iter().any(|c| matches!(c,
                StyleClass::Width(crate::ui::style::SizeValue::Pixels(px)) if *px == 11.0)),
            "size 像素必须保持"
        );
        crate::ui::aura_view_builder::clear_imported_components();
    }

    /// A6：self-end / items-end 进 IcedStyle（渲染层列臂消费）。
    #[test]
    fn message_row_self_end_items_end_reach_iced_style() {
        let s = Style::parse("flex flex-col gap-[3px] max-w-[85%] self-end items-end").expect("parse");
        let is = IcedStyle::from_style(&s);
        assert!(
            matches!(is.align_self, Some(crate::ui::style::iced_adapter::IcedAlign::End)),
            "self-end 必须进 align_self(此前仅降级告警)"
        );
        assert!(
            matches!(is.align_items, Some(crate::ui::style::iced_adapter::IcedAlign::End)),
            "items-end 必须进 align_items"
        );
    }
}

/// PLAN-054 T5 (A7)：Date.format 宿主桥端到端——musk forge_helpers.at 的
/// msgTimeLabel 同款形态（epoch 秒 ×1000 → "HH:mm:ss"），此前 Date.format
/// 未桥接返回垃圾（时间标签缺失现场）。宿主臂收口 KNOWN-DEBT 051。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t5_date_format {
    use super::musk_vm_track_p054_t3_none_return::build_bridge;

    #[test]
    fn date_format_bridge_yields_hhmmss_label() {
        let src = concat!(
            "widget Root54d {\n",
            "    view {\n",
            "        col {\n",
            "            text \"x\"\n",
            "        }\n",
            "    }\n",
            "}\n",
            "fn msg_time_label(createdAt int) str {\n",
            "    if createdAt == 0 { return \"\" }\n",
            "    return Date.format(createdAt * 1000, \"HH:mm:ss\")\n",
            "}\n",
        );
        let bridge = build_bridge(src);
        // 固定历元（本地时区只影响时分秒数值,不影响形态）。
        let out = bridge
            .call_vm_fn("msg_time_label", &[auto_val::Value::Int(86401)])
            .expect("call msg_time_label");
        eprintln!("[P054-T5] msg_time_label(86401) = {:?}", out);
        let s = match &out {
            auto_val::Value::Str(s) => s.to_string(),
            auto_val::Value::String(s) => s.to_string(),
            other => panic!("必须返回字符串, got {:?}", other),
        };
        assert_eq!(
            s.split(':').count(),
            3,
            "HH:MM:SS 形态(两个冒号三段数字), got {:?}",
            s
        );
        for part in s.split(':') {
            assert!(
                !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()),
                "时间段必须是数字, got {:?}",
                s
            );
        }
        // 零契约：0 → 空串（web 轨同款守卫）。
        let zero = bridge
            .call_vm_fn("msg_time_label", &[auto_val::Value::Int(0)])
            .expect("call zero");
        assert_eq!(zero, auto_val::Value::Str("".into()), "createdAt=0 必须空串");
    }
}

/// T5 勘察：Date.now 同形态对照。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p054_t5_date_probe {
    use super::musk_vm_track_p054_t3_none_return::build_bridge;

    #[test]
    #[ignore = "manual probe"]
    fn date_now_routing_probe() {
        let src = concat!(
            "widget Root54p {\n",
            "    view {\n",
            "        col {\n",
            "            text \"x\"\n",
            "        }\n",
            "    }\n",
            "}\n",
            "fn now_ms() int {\n",
            "    return Date.now()\n",
            "}\n",
            "fn fmt(ms int) str {\n",
            "    return Date.format(ms, \"HH:mm:ss\")\n",
            "}\n",
            "fn fmt_now() str {\n",
            "    return Date.format(Date.now(), \"HH:mm:ss\")\n",
            "}\n",
        );
        let bridge = build_bridge(src);
        for (name, f) in [("now_ms", 0), ("fmt", 1), ("fmt_now", 2)] {
            let _ = f;
            match bridge.call_vm_fn(name, &[]) {
                Ok(v) => eprintln!("[P054-T5P] {} = {:?}", name, v),
                Err(e) => eprintln!("[P054-T5P] {} ERR: {:?}", name, e),
            }
        }
        match bridge.call_vm_fn("fmt", &[auto_val::Value::Int(86401000)]) {
            Ok(v) => eprintln!("[P054-T5P] fmt(86401000) = {:?}", v),
            Err(e) => eprintln!("[P054-T5P] fmt ERR: {:?}", e),
        }
    }
}

/// PLAN-055 勘察探针（#[ignore]）：复刻 musk chat_message/chats_view 层级，
/// dump 动态管线产出的 View 树 + 每节点样式类——定位用户气泡 self-end/
/// bg-primary 丢失、会话卡 bg-primary/10、canvas gap-6 的断层在哪一层。
/// 运行：cargo nextest run -p auto-lang --lib --features ui-iced p055_dump -- --ignored --nocapture
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p055_dump {
    #[test]
    #[ignore = "manual probe"]
    fn p055_chat_hierarchy_style_dump() {
        use crate::parser::Parser;
        use crate::ui::view::View;
        let src = concat!(
            "widget P55Root {\n",
            "    computed {\n",
            "        rowClass => \"flex flex-col gap-[3px] self-end items-end\"\n",
            "        headerClass => \"flex items-center gap-2 px-1 justify-end\"\n",
            "    }\n",
            "    view {\n",
            "        col {\n",
            "            key: \"m1\"\n",
            "            row { style: \"flex gap-1 mb-[2px]\" }\n",
            "            col {\n",
            "                class: .rowClass\n",
            "                row {\n",
            "                    style: .headerClass\n",
            "                    text \"You\" { style: \"text-[13.6px] font-semibold text-primary\" }\n",
            "                }\n",
            "                col {\n",
            "                    style: \"msg-bubble-user px-[14px] py-[10px] rounded-xl text-[15px] bg-primary text-primary-foreground\"\n",
            "                    P55User(content: \"你好\")\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
            "widget P55User(content: str) {\n",
            "    view {\n",
            "        text .content\n",
            "    }\n",
            "}\n",
        );
        crate::ui::aura_view_builder::register_imported_components(vec!["P55User".to_string()]);
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
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract");
        let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0], &decls[1..], &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![], &std::collections::HashMap::new(), false,
        )
        .expect("component");
        let (view, _, _) = comp.view_with_debug_gated(false);
        fn style_summary(style: &Option<crate::ui::style::Style>) -> String {
            match style {
                None => "None".into(),
                Some(s) => {
                    let names: Vec<String> = s.classes.iter().map(|c| format!("{:?}", c).split('(').next().unwrap_or("").to_string()).collect();
                    format!("{:?}", names)
                }
            }
        }
        fn walk(view: &View<crate::ui::interpreter::DynamicMessage>, depth: usize) {
            let ind = "  ".repeat(depth);
            match view {
                View::Column { children, style, .. } => {
                    eprintln!("{}Column style={}", ind, style_summary(style));
                    for c in children { walk(c, depth + 1); }
                }
                View::Row { children, style, .. } => {
                    eprintln!("{}Row style={}", ind, style_summary(style));
                    for c in children { walk(c, depth + 1); }
                }
                View::Button { label, style, content, .. } => {
                    eprintln!("{}Button label={:?} style={}", ind, label, style_summary(style));
                    if let Some(c) = content { walk(c, depth + 1); }
                }
                View::Text { content, style, .. } => {
                    eprintln!("{}Text {:?} style={}", ind, content, style_summary(style));
                }
                View::Container { child, style, .. } => {
                    eprintln!("{}Container style={}", ind, style_summary(style));
                    walk(child, depth + 1);
                }
                _other => eprintln!("{}other", ind),
            }
        }
        walk(&view, 0);
        crate::ui::aura_view_builder::clear_imported_components();
    }
}

/// PLAN-055 勘察探针②：if 分支内 div{html:} 是否触发 else 兜底（musk
/// 用户消息进 AI 分支的根因裁定）。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p055_dump2 {
    use super::musk_vm_track_p054_t3_none_return::build_bridge;

    #[test]
    #[ignore = "manual probe"]
    fn p055_html_branch_fallback() {
        use crate::parser::Parser;
        use crate::ui::view::View;
        let src = concat!(
            "widget P55BRoot {
",
            "    computed {
",
            "        isUser => true
",
            "    }
",
            "    view {
",
            "        col {
",
            "            if .isUser {
",
            "                col {
",
            "                    style: \"bg-primary px-[14px] self-end\"
",
            "                    div {
",
            "                        html: \"<span>@x</span> 你好\"
",
            "                        style: \"text-primary-foreground\"
",
            "                    }
",
            "                }
",
            "            } else {
",
            "                col {
",
            "                    style: \"border-t border-b\"
",
            "                    text \"ELSE-BRANCH\"
",
            "                }
",
            "            }
",
            "        }
",
            "    }
",
            "}
",
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
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract");
        let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0], &decls[1..], &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![], &std::collections::HashMap::new(), false,
        )
        .expect("component");
        let (view, _, _) = comp.view_with_debug_gated(false);
        fn walk(view: &View<crate::ui::interpreter::DynamicMessage>, depth: usize) {
            let ind = "  ".repeat(depth);
            match view {
                View::Column { children, style, .. } => {
                    let n = style.as_ref().map(|s| s.classes.len()).unwrap_or(0);
                    eprintln!("{}Column classes={}", ind, n);
                    for c in children { walk(c, depth + 1); }
                }
                View::Row { children, .. } => {
                    eprintln!("{}Row", ind);
                    for c in children { walk(c, depth + 1); }
                }
                View::Text { content, .. } => eprintln!("{}Text {:?}", ind, content),
                _other => eprintln!("{}other", ind),
            }
        }
        walk(&view, 0);
    }

    /// Math/算术微探针：定位 est 恒 1 的剩余环节。
    #[test]
    #[ignore = "manual probe"]
    fn p055_math_micro() {
        let src = concat!(
            "widget P55MathRoot {
",
            "    view {
",
            "        text \"x\"
",
            "    }
",
            "}
",
            "fn f_max() int {
",
            "    return Math.max(1, 15)
",
            "}
",
            "fn f_round() int {
",
            "    return Math.round(4.25)
",
            "}
",
            "fn f_intfloat() float {
",
            "    let cjk = 17
",
            "    return cjk * 0.9
",
            "}
",
            "fn f_sum() float {
",
            "    let cjk = 17
",
            "    let non = 0
",
            "    return cjk * 0.9 + non * 0.25
",
            "}
",
            "fn f_full() int {
",
            "    let cjk = 17
",
            "    let non = 0
",
            "    return Math.max(1, Math.round(cjk * 0.9 + non * 0.25))
",
            "}
",
        );
        let bridge = build_bridge(src);
        for name in ["f_max", "f_round", "f_intfloat", "f_sum", "f_full"] {
            match bridge.call_vm_fn(name, &[]) {
                Ok(v) => eprintln!("[P055-Math] {} = {:?}", name, v),
                Err(e) => eprintln!("[P055-Math] {} ERR: {:?}", name, e),
            }
        }
    }

    /// estimateTokens v3 预验：纯整数定点（绕开 VM float 管线）。
    #[test]
    fn p055_est_v3() {
        let src = concat!(
            "widget P55Est3Root {
",
            "    view {
",
            "        text \"x\"
",
            "    }
",
            "}
",
            "fn est3(text str) int {
",
            "    if text == None { return 0 }
",
            "    if text == \"\" { return 0 }
",
            "    var total = 0
",
            "    for c in text {
",
            "        total = total + 1
",
            "    }
",
            "    var cjk = 0
",
            "    for i in 0..total {
",
            "        let code = text.char_code_at(i)
",
            "        if (code >= 19968 && code <= 40959) || (code >= 12352 && code <= 12543) || (code >= 44032 && code <= 55215) {
",
            "            cjk = cjk + 1
",
            "        }
",
            "    }
",
            "    let nonCjk = total - cjk
",
            "    let scaled = cjk * 9000 + nonCjk * 2500
",
            "    return Math.max(1, (scaled + 5000) / 10000)
",
            "}
",
        );
        let bridge = build_bridge(src);
        // 回归锁：musk estimateTokens 整数定点形态——双轨同值
        // （Vue: round(cjk*0.9 + non*0.25)）。
        let get = |sample: &str| {
            bridge.call_vm_fn("est3", &[auto_val::Value::str(sample)]).ok()
        };
        assert_eq!(get("用户打招呼，询问有什么可以帮忙的。"), Some(auto_val::Value::Int(14)));
        assert_eq!(get("abcdefgh"), Some(auto_val::Value::Int(2)));
        assert_eq!(get("hi 你好"), Some(auto_val::Value::Int(3)));
    }

    /// estimateTokens 重写形态预验：range 循环 + 索引式 char_code_at。
    #[test]
    #[ignore = "manual probe"]
    fn p055_est_v2() {
        let src = concat!(
            "widget P55Est2Root {
",
            "    view {
",
            "        text \"x\"
",
            "    }
",
            "}
",
            "fn est2(text str) int {
",
            "    if text == None { return 0 }
",
            "    if text == \"\" { return 0 }
",
            "    var total = 0
",
            "    for c in text {
",
            "        total = total + 1
",
            "    }
",
            "    var cjk = 0
",
            "    for i in 0..total {
",
            "        let code = text.char_code_at(i)
",
            "        if (code >= 19968 && code <= 40959) || (code >= 12352 && code <= 12543) || (code >= 44032 && code <= 55215) {
",
            "            cjk = cjk + 1
",
            "        }
",
            "    }
",
            "    let nonCjk = total - cjk
",
            "    return Math.max(1, Math.round(cjk * 0.9 + nonCjk * 0.25))
",
            "}
",
        );
        let bridge = build_bridge(src);
        for (label, sample) in [("cjk17", "用户打招呼，询问有什么可以帮忙的。"), ("ascii8", "abcdefgh"), ("mixed", "hi 你好")] {
            match bridge.call_vm_fn("est2", &[auto_val::Value::str(sample)]) {
                Ok(v) => eprintln!("[P055-E2] {} = {:?}", label, v),
                Err(e) => eprintln!("[P055-E2] {} ERR: {:?}", label, e),
            }
        }
    }

    /// 微探针：.length / for-in 迭代数 / char_code_at 逐项核验。
    #[test]
    fn p055_str_micro() {
        let src = concat!(
            "widget P55MicroRoot {
",
            "    view {
",
            "        text \"x\"
",
            "    }
",
            "}
",
            "fn len_of(text str) int {
",
            "    return text.length
",
            "}
",
            "fn iter_count(text str) int {
",
            "    var n = 0
",
            "    for c in text {
",
            "        n = n + 1
",
            "    }
",
            "    return n
",
            "}
",
            "fn code_of_first(text str) int {
",
            "    for c in text {
",
            "        return c.char_code_at(0)
",
            "    }
",
            "    return -1
",
            "}
",
            "fn code_via_index(text str) int {
",
            "    return text.char_code_at(0)
",
            "}
",
        );
        let bridge = build_bridge(src);
        let sample = "用户打招呼，询问有什么可以帮忙的。";
        // 回归锁：.length = 字符数（JS 语义）、for-in 计数、索引式 char_code_at。
        let get = |name: &str| {
            bridge.call_vm_fn(name, &[auto_val::Value::str(sample)]).ok()
        };
        assert_eq!(get("len_of"), Some(auto_val::Value::Int(17)), ".length 必须为字符数");
        assert_eq!(get("iter_count"), Some(auto_val::Value::Int(17)), "for-in over str 必须逐字符迭代");
        assert_eq!(get("code_via_index"), Some(auto_val::Value::Int(0x7528)), "str.char_code_at 索引式");
    }

    /// estimateTokens 同款 for-in over str 在 VM 的行为（Vue=5 vs VM=1 根因）。
    #[test]
    #[ignore = "manual probe"]
    fn p055_str_iteration_est() {
        let src = concat!(
            "widget P55EstRoot {
",
            "    view {
",
            "        text \"x\"
",
            "    }
",
            "}
",
            "fn est(text str) int {
",
            "    if text == None { return 0 }
",
            "    if text == \"\" { return 0 }
",
            "    var cjk = 0
",
            "    for c in text {
",
            "        let code = c.char_code_at(0)
",
            "        if (code >= 19968 && code <= 40959) || (code >= 12352 && code <= 12543) {
",
            "            cjk = cjk + 1
",
            "        }
",
            "    }
",
            "    let nonCjk = text.length - cjk
",
            "    return Math.max(1, Math.round(cjk * 0.9 + nonCjk * 0.25))
",
            "}
",
        );
        let bridge = build_bridge(src);
        let sample = "用户打招呼，询问有什么可以帮忙的。";
        match bridge.call_vm_fn("est", &[auto_val::Value::str(sample)]) {
            Ok(v) => eprintln!("[P055-2] est({} chars) = {:?}", sample.chars().count(), v),
            Err(e) => eprintln!("[P055-2] est ERR: {:?}", e),
        }
    }
}

/// PLAN-055 勘察探针③：if 分支兜底 bisect——条件求值 vs 分支体失败。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p055_dump3 {
    use crate::parser::Parser;
    use crate::ui::view::View;

    fn render(src: &str) -> String {
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
        let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract");
        let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0], &decls[1..], &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![], &std::collections::HashMap::new(), false,
        )
        .expect("component");
        let (view, _, _) = comp.view_with_debug_gated(false);
        fn walk(view: &View<crate::ui::interpreter::DynamicMessage>, out: &mut Vec<String>) {
            match view {
                View::Text { content, .. } => out.push(format!("T({})", content)),
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children { walk(c, out); }
                }
                _ => out.push("?".into()),
            }
        }
        let mut out = Vec::new();
        walk(&view, &mut out);
        out.join(",")
    }

    #[test]
    fn p055_if_bisect() {
        // 回归锁：裸 computed 条件真值判定（此前恒假 → musk 用户消息
        // 恒走 chat_message else 臂、hasTime 恒隐藏）。
        let ra = render("widget P55A {
    computed {
        isUser => true
    }
    view {
        col {
            if .isUser {
                text \"IF\"
            } else {
                text \"ELSE\"
            }
        }
    }
}
");
        assert_eq!(ra, "T(IF)", "裸 computed 真值必须取 then 臂");
        // (b) computed true 条件 + div{html:} 混合分支体
        eprintln!("[P055-3b] {}",
            render("widget P55B {
    computed {
        isUser => true
    }
    view {
        col {
            if .isUser {
                div {
                    html: \"<b>x</b> 你好\"
                }
                text \"IF\"
            } else {
                text \"ELSE\"
            }
        }
    }
}
"));
        // (c) 字面量 true 条件
        eprintln!("[P055-3c] {}",
            render("widget P55C {
    view {
        col {
            if true {
                text \"IF\"
            } else {
                text \"ELSE\"
            }
        }
    }
}
"));
        // (d) 无 else 臂（if 独立）
        eprintln!("[P055-3d] {}",
            render("widget P55D {
    view {
        col {
            if true {
                text \"IF-NOELSE\"
            }
            text \"TAIL\"
        }
    }
}
"));
        // (e) musk 同款条件形态 .role == "user"
        eprintln!("[P055-3e] {}",
            render("widget P55E {
    model {
        var role str = \"user\"
    }
    computed {
        isUser => .role == \"user\"
    }
    view {
        col {
            if .isUser {
                text \"IF\"
            } else {
                text \"ELSE\"
            }
        }
    }
}
"));
    }
}

/// PLAN-055 T8(⑥)：input 通道 `$event` 实参的运行期文本替换回归。
/// 链路：convert_input 冻结字面 "$event" 实参 → encode_payload 随事件串携带
/// → render_dynamic_view Input 臂 on_input 携 input_value:Some(text) →
/// on_with_input_for 的 U2 替换（dynamic.rs Plan 446 批五）把 "$event" 前缀
/// 实参换成输入文本。此前诊断（2026-09-01 musk 搜索框失效）怀疑 VM 侧断链；
/// 复测现行 master：动态路径链路完整——本用例固化该行为防回退。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p055_input_event_text {
    #[test]
    fn input_event_arg_replaced_with_typed_text() {
        let src = concat!(
            "widget QSearch {\n",
            "    model {\n",
            "        var q str = \"\"\n",
            "    }\n",
            "    msg Msg { SetQ(str) }\n",
            "    on {\n",
            "        .SetQ(v) -> {\n",
            "            .q = v\n",
            "        }\n",
            "    }\n",
            "    view {\n",
            "        col {\n",
            "            input {\n",
            "                placeholder: \"搜索消息…\"\n",
            "                oninput: .SetQ($event)\n",
            "            }\n",
            "            text .q\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src).with_session(session);
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
        let mut dc = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component");
        let _ = dc.view_with_debug_gated(false);
        // 派发形态与实机一致：事件串携带 payload 编码的字面 "$event" 实参
        // （event_to_message_with 冻结 + encode_payload 嵌入），input_value 为
        // 用户键入文本。
        dc.on_with_input_for("QSearch", "SetQ\u{1F}s\u{1F}$event", Some("你好".to_string()));
        assert_eq!(
            match dc.read_state("q").expect("q readable") {
                auto_val::Value::Str(s) => s.as_str().to_string(),
                v => format!("{:?}", v),
            },
            "你好",
            "input 通道 $event 实参应被替换为输入文本（musk 搜索框 VM 侧链路）"
        );
    }
}

/// PLAN-055 T12（④）：pre/code 转换臂——此前落 unknown fallback 成
/// style:None 的 Column（类串整体丢弃）。现进容器臂：think 展开区类串
/// （py-[9px]/px-[12px]/my-0/border-t/max-h-[240px]）应完整解析进 View 样式。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p055_pre_code_arm {
    use crate::parser::Parser;
    use crate::ui::view::View;

    fn build_view(src: &str) -> View<crate::ui::interpreter::DynamicMessage> {
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
        let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component");
        let (v, _, _) = comp.view_with_debug_gated(false);
        v
    }

    /// 容器视图递归找首个带样式容器（pre 的转换产物：单子容器臂产
    /// View::Container，多子时为 Column——两形态都收）。
    fn first_styled_container<'a>(
        v: &'a View<crate::ui::interpreter::DynamicMessage>,
    ) -> Option<&'a View<crate::ui::interpreter::DynamicMessage>> {
        match v {
            View::Container { style, .. } if style.is_some() => Some(v),
            View::Column { style, .. } if style.is_some() => Some(v),
            View::Row { children, .. } | View::Column { children, .. } => {
                children.iter().find_map(first_styled_container)
            }
            _ => None,
        }
    }

    #[test]
    fn pre_class_string_parses_padding_border_and_max_h() {
        let v = build_view(concat!(
            "widget PreProbe {\n",
            "    view {\n",
            "        pre {\n",
            "            style: \"think-content my-0 py-[9px] px-[12px] text-[13.5px] max-h-[240px] border-t border-border\"\n",
            "            text \"思考内容\"\n",
            "        }\n",
            "    }\n",
            "}\n",
        ));
        let col = first_styled_container(&v).expect("pre 应转换为带样式的容器（此前 fallback 丢类串）");
        let style = match col {
            View::Container { style, .. } | View::Column { style, .. } => {
                style.as_ref().expect("style 必须在")
            }
            _ => unreachable!(),
        };
        use crate::ui::style::StyleClass;
        use crate::ui::style::iced_adapter::IcedStyle;
        // 类串解析面：border-top 类 + padding/max-h 适配值三键齐备。
        assert!(
            style.classes.iter().any(|c| matches!(c, StyleClass::BorderTop)),
            "border-t 应解析，got {:?}",
            style.classes
        );
        let is = IcedStyle::from_style(style);
        assert!(
            is.padding_y.unwrap_or(0.0) >= 8.0 && is.padding_x.unwrap_or(0.0) >= 10.0,
            "py-[9px]/px-[12px] 应进 padding，got y={:?} x={:?}",
            is.padding_y,
            is.padding_x
        );
        assert!(
            is.max_height.unwrap_or(0.0) >= 200.0,
            "max-h-[240px] 应解析，got {:?}",
            is.max_height
        );
    }
}

/// PLAN-055 ①/T14：一级导航 active 布尔表达式求值——musk app.at 声明
/// `active: .current_view == "chats"`，断链嫌疑在 extract_bool_expr →
/// resolve_expr_to_value 的 Eq 臂（state 字段 vs 字面量）。active=true 必须
/// 把 nav_contract::ITEM_ACTIVE（bg-primary/10 text-primary font-medium）
/// 拼进按钮类串。
#[cfg(all(test, feature = "ui-iced"))]
mod musk_vm_track_p055_nav_active {
    use crate::parser::Parser;
    use crate::ui::view::View;

    fn build_nav(src: &str) -> View<crate::ui::interpreter::DynamicMessage> {
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
        let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component");
        let (v, _, _) = comp.view_with_debug_gated(false);
        v
    }

    fn nav_src(view_value: &str) -> String {
        format!(
            concat!(
                "widget NavProbe {{\n",
                "    model {{\n",
                "        var current_view str = \"{v}\"\n",
                "    }}\n",
                "    view {{\n",
                "        col {{\n",
                // musk app.at 实际形态：括号属性式（Plan 482 nav-item 接线现场）。
                "            nav-item (label: \"会话\", icon: \"message-circle\", active: .current_view == \"chats\")\n",
                "        }}\n",
                "    }}\n",
                "}}\n",
            ),
            v = view_value
        )
    }

    /// 找 nav-item 产物按钮的样式类串文本。
    fn nav_button_classes(
        v: &View<crate::ui::interpreter::DynamicMessage>,
    ) -> Option<String> {
        match v {
            View::Button { style, .. } => style.as_ref().map(|s| {
                s.classes
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join(",")
            }),
            View::Row { children, .. } | View::Column { children, .. } => {
                children.iter().find_map(nav_button_classes)
            }
            View::Container { child, .. } => nav_button_classes(child),
            _ => None,
        }
    }

    #[test]
    fn nav_active_eq_expr_yields_item_active_classes() {
        // current_view == "chats" → active=true → ITEM_ACTIVE（BgPrimary 透明度
        // 0.1 + TextPrimary 语义色）必须出现在按钮类集。
        let v = build_nav(&nav_src("chats"));
        let classes = nav_button_classes(&v).expect("nav-item 按钮样式");
        eprintln!("[T14-DBG] active classes = {}", classes);
        // ITEM_ACTIVE = bg-primary/10 text-primary font-medium——适配层的
        // Debug 形态：BackgroundColor(Rgba … a≈0.1×255) + TextColor(Primary)
        // + FontMedium。
        assert!(
            classes.contains("BackgroundColor"),
            "active 时 bg-primary/10 应在，got {}",
            classes
        );
        assert!(
            classes.contains("TextColor(Primary)"),
            "active 时 text-primary 应在，got {}",
            classes
        );
        assert!(classes.contains("FontMedium"), "font-medium 应在，got {}", classes);
    }

    #[test]
    fn nav_inactive_eq_expr_omits_item_active() {
        // current_view == "plans" → active=false → 无 ITEM_ACTIVE。
        let v = build_nav(&nav_src("plans"));
        let classes = nav_button_classes(&v).expect("nav-item 按钮样式");
        assert!(
            !classes.contains("BackgroundColor"),
            "inactive 时 bg-primary 不应在，got {}",
            classes
        );
    }
}

/// PLAN-057 T2：SET_FIELD 新键插入语义（等价性缺陷族①——Block 全家福 a1
/// 整条空白的直接死因）。TS 基准：对象是开放 dict，`obj.newKey = v` 合法插入
/// （ObjectData 底层本就是开放 HashMap，types.rs set=insert）。
/// 根修仅放开 ObjectData 臂的 Str 新键；Int/Bool 键格式与
/// GenericInstanceData（typed instance）维持 Plan 118 报错（类型严格性）。
#[cfg(test)]
mod musk_vm_track_p057_setfield_newkey {
    fn run_code(code: &str) -> Result<String, String> {
        match crate::run_with_capture(code) {
            Ok((_, stdout)) => Ok(stdout),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    /// 空对象字面量加新键（此前 RuntimeError 中止——case_setfield_newkey A）。
    #[test]
    fn setfield_empty_literal_newkey_inserts() {
        let out = run_code("var o = {}\no.a = 1\nprint(o.a)");
        assert!(
            matches!(&out, Ok(s) if s.contains('1')),
            "expected inserted value 1, got: {:?}",
            out
        );
    }

    /// 既有键赋值仍走更新（对照——case_setfield_newkey B）。
    #[test]
    fn setfield_existing_key_updates() {
        let out = run_code(concat!(
            "let o = { name: \"a\", status: \"pending\" }\n",
            "o.status = \"completed\"\n",
            "print(o.status)",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains("completed")),
            "expected completed, got: {:?}",
            out
        );
    }

    /// for-in 元素加新键（messageBlocks 现场——case_setfield_newkey C）。
    #[test]
    fn setfield_forin_element_newkey_inserts() {
        let out = run_code(concat!(
            "let calls = [{ name: \"x\" }, { name: \"y\" }]\n",
            "for raw in calls {\n",
            "    var status = \"done\"\n",
            "    raw.status = status\n",
            "}\n",
            "print(calls[0].status)",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains("done")),
            "expected done, got: {:?}",
            out
        );
    }

    /// typed instance（type 声明的泛型实例）加新键仍不落键——Plan 118
    /// 类型严格性保留（GenericInstanceData 分支不动）。实测形态：写侧编译
    /// 期告警+静默跳过（不中止），新键**读回**时 RuntimeError（engine.rs
    /// GET_FIELD typed-instance 臂）。钉住可观测不变量：typed 写不创建键、
    /// 读回应中止——若未来被"修成"插入语义，本测试转红。
    #[test]
    fn setfield_typed_instance_still_errs_on_readback() {
        let out = run_code(concat!(
            "type Point { x int, y int }\n",
            "let p = Point { x: 1, y: 2 }\n",
            "p.z = 3\n",
            "print(p.z)\n",
            "print(\"BOOM\")",
        ));
        let leaked = matches!(&out, Ok(s) if s.contains("BOOM") || s.contains('3'))
            || matches!(&out, Err(e) if e.contains("BOOM"));
        assert!(
            !leaked,
            "typed instance 新键不应落键（读回应中止，哨兵 BOOM/值 3 不应出现），got: {:?}",
            out
        );
    }
}

/// PLAN-057 T3：for-in Call 源通道泛化（等价性缺陷族②）。TS 基准：`for x in f()`
/// 迭代 f() 返回数组的全部元素；VM 现状=任意直接调用源恒 0 次静默迭代
/// （codegen.rs Plan 454 E5b 索引通道仅覆盖 .values/.keys，其余 Call 源落
/// 默认迭代器通道——List 句柄≠iterator，auto.iterator.next 立即判尽）。
/// 泛化后索引通道承接全部 Call 源；迭代器协议族保留原通道（.iter()/.take()
/// 等适配链——通道选择由 vm_types 编译形态测试钉住，运行期 .iter() 本身
/// 零迭代是既有独立债，非本计划范围；sse_get_stream 流式源惰性拉帧）。
#[cfg(test)]
mod musk_vm_track_p057_forin_call {
    fn run_code(code: &str) -> Result<String, String> {
        match crate::run_with_capture(code) {
            Ok((_, stdout)) => Ok(stdout),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    /// obj 返回注解的直接调用源——计数（case_forin_call A）。
    #[test]
    fn forin_call_obj_return_iterates() {
        let out = run_code(concat!(
            "fn g() obj {\n",
            "    return [{ n: 1 }, { n: 2 }, { n: 3 }]\n",
            "}\n",
            "var n = 0\n",
            "for x in g() {\n",
            "    n = n + 1\n",
            "}\n",
            "print(n)",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains('3')),
            "expected 3 iterations, got: {:?}",
            out
        );
    }

    /// list 返回注解的直接调用源——求和（case_forin_call D）。
    #[test]
    fn forin_call_int_list_sums() {
        let out = run_code(concat!(
            "fn g() list {\n",
            "    return [10, 20, 30]\n",
            "}\n",
            "var sum = 0\n",
            "for v in g() {\n",
            "    sum = sum + v\n",
            "}\n",
            "print(sum)",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains("60")),
            "expected sum 60, got: {:?}",
            out
        );
    }

    /// 泛化回归面：`.values` 既有 E5b 形态不因泛化回归（索引通道承接）。
    #[test]
    fn forin_values_call_still_iterates() {
        let out = run_code(concat!(
            "let o = { a: 1, b: 2, c: 3 }\n",
            "var n = 0\n",
            "for v in Object.values(o) {\n",
            "    n = n + v\n",
            "}\n",
            "print(n)",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains('6')),
            "expected 6 via Object.values, got: {:?}",
            out
        );
    }
}

/// PLAN-057 T4：实参含直接调用的参数槽错位（等价性缺陷族⑤）。现场：
/// case_web_builtins A/B/C 标签乱码（4000000/None/hi）——实参为未解析调用
/// （CALL_SPEC 静态兜底）时，str/List/unknown 接收者臂的未知方法兜底
/// **只压 None 不弹 receiver+实参**，栈失衡 +1 使后续调用的参数槽整体
/// 平移。根修=三处兜底臂配平（pop 0..=arg_count 再压 None）。
/// 用合成名（Foo.bar/xs.frobnicate）锁未解析路径——不在 Math/JSON/Object/
/// Array 命名空间，免疫 T7 编译期门禁。
#[cfg(test)]
mod musk_vm_track_p057_arg_stack {
    fn run_code(code: &str) -> Result<String, String> {
        match crate::run_with_capture(code) {
            Ok((_, stdout)) => Ok(stdout),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    fn src(body: &str) -> String {
        format!(
            "fn f3(label str, v obj) {{\n    print(f\"${{label}}|${{v}}\")\n}}\nfn main() {{\n{}\n}}\n",
            body
        )
    }

    /// 未解析静态调用（str 接收者臂）：标签必须完整（case A/B/C 形态）。
    #[test]
    fn unresolved_static_call_keeps_sibling_label() {
        let out = run_code(&src(concat!(
            "    let o = { a: 1 }\n",
            "    f3(\"lblA\", Foo.bar(o))\n",
            "    f3(\"lblC\", \"tail\")",
        )));
        assert!(
            matches!(&out, Ok(s) if s.contains("lblA|") && s.contains("lblC|tail")),
            "labels must stay intact across unresolved static call, got: {:?}",
            out
        );
    }

    /// 未知 list 方法（List 接收者臂）：同上。
    #[test]
    fn unknown_list_method_keeps_sibling_label() {
        let out = run_code(&src(concat!(
            "    let xs = [1, 2]\n",
            "    f3(\"lblB\", xs.frobnicate(1))\n",
            "    f3(\"lblC\", \"tail\")",
        )));
        assert!(
            matches!(&out, Ok(s) if s.contains("lblB|") && s.contains("lblC|tail")),
            "labels must stay intact across unknown list method, got: {:?}",
            out
        );
    }

    /// 未解析调用的值语义：恒 None（静默桩，缺陷族③运行期面），配平后仍是。
    #[test]
    fn unresolved_call_value_is_none() {
        let out = run_code(&src(concat!(
            "    let o = { a: 1 }\n",
            "    f3(\"lblA\", Foo.bar(o))",
        )));
        assert!(
            matches!(&out, Ok(s) if s.contains("lblA|None")),
            "unresolved call value should be None, got: {:?}",
            out
        );
    }
}

/// PLAN-057 T5：for-in 裸字符接收者分派重定位（等价性缺陷族④）。
/// for-in over str 经 GET_ELEM 产出 i32 码点——`c.char_code_at(0)` 接收者
/// 是码点不是字符串，落 CALL_SPEC 的 `<unknown:` 接收者臂（整型字面量方法
/// 族）被静默吞 None；PLAN-055 加的 Char 恒等臂（engine.rs 7213 一带）
/// 排在该臂之后永不命中。根修=恒等臂提升到 `<unknown:` 臂之前。
#[cfg(test)]
mod musk_vm_track_p057_char_receiver {
    fn run_code(code: &str) -> Result<String, String> {
        match crate::run_with_capture(code) {
            Ok((_, stdout)) => Ok(stdout),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    /// for-in 字符接收者逐字符码点求和（"你好ab"=20320+22909+97+98=43424）。
    #[test]
    fn forin_char_receiver_char_code_at_sums() {
        let out = run_code(concat!(
            "let text = \"你好ab\"\n",
            "var sum = 0\n",
            "for c in text {\n",
            "    sum = sum + c.char_code_at(0)\n",
            "}\n",
            "print(sum)",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains("43424")),
            "expected codepoint sum 43424, got: {:?}",
            out
        );
    }

    /// 控制组：单字符字符串接收者（str 臂）不回归。
    #[test]
    fn single_char_string_receiver_still_works() {
        let out = run_code(concat!(
            "let one = \"你\"\n",
            "print(one.char_code_at(0))",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains("20320")),
            "expected 20320, got: {:?}",
            out
        );
    }
}

/// PLAN-057 T6：web 内建 natives 补齐（等价性缺陷族③——未实现者运行期
/// 静默 None）。TS 基准：Array.isArray/JSON.stringify（含缩进参）/
/// JSON.parse 数组形态 `.length`/Math.trunc（int 恒等不回绕）/Math.imul
/// （i32 回绕乘）。trunc/imul 为 T1 探针实证的 census 漏计项（待澄清⑤），
/// 与 T7 编译期门禁连贯性要求一并落地。
#[cfg(test)]
mod musk_vm_track_p057_web_natives {
    fn run_code(code: &str) -> Result<String, String> {
        match crate::run_with_capture(code) {
            Ok((_, stdout)) => Ok(stdout),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    fn assert_out(code: &str, needle: &str) {
        let out = run_code(code);
        assert!(
            matches!(&out, Ok(s) if s.contains(needle)),
            "expected stdout containing {:?}, got: {:?}",
            needle, out
        );
    }

    /// Array.isArray：真列表 true；对象/字符串 false；parse 数组 true。
    #[test]
    fn is_array_js_semantics() {
        assert_out(
            concat!(
                "let l = [1, 2]\n",
                "let o = { a: 1 }\n",
                "var n = 0\n",
                "if Array.isArray(l) { n = n + 1 }\n",
                "if !Array.isArray(o) { n = n + 1 }\n",
                "if !Array.isArray(\"hi\") { n = n + 1 }\n",
                "if Array.isArray(JSON.parse(\"[1]\")) { n = n + 1 }\n",
                "print(n)",
            ),
            "4",
        );
    }

    /// JSON.stringify 单参：紧凑 JSON 文本。
    #[test]
    fn json_stringify_compact() {
        assert_out(
            concat!(
                "let o = { a: 1, b: \"x\" }\n",
                "let s = JSON.stringify(o)\n",
                "if s == \"{\\\"a\\\":1,\\\"b\\\":\\\"x\\\"}\" {\n",
                "    print(\"OK\")\n",
                "} else {\n",
                "    print(f\"got=${s}\")\n",
                "}",
            ),
            "OK",
        );
    }

    /// JSON.stringify 三参（v, null, 2）：缩进美化（多行）。
    #[test]
    fn json_stringify_pretty() {
        assert_out(
            concat!(
                "let o = { a: 1, b: 2 }\n",
                "let s = JSON.stringify(o, null, 2)\n",
                "let compact = JSON.stringify(o)\n",
                "if s != None && s.length > compact.length {\n",
                "    print(\"PRETTY\")\n",
                "} else {\n",
                "    print(\"FLAT\")\n",
                "}",
            ),
            "PRETTY",
        );
    }

    /// JSON.parse 数组形态：.length=3、元素 [0]=10（case H）。
    #[test]
    fn json_parse_array_length() {
        assert_out(
            concat!(
                "let pa = JSON.parse(\"[10,20,30]\")\n",
                "if pa.length == 3 && pa[0] == 10 {\n",
                "    print(\"OK\")\n",
                "} else {\n",
                "    print(f\"len=${pa.length} e0=${pa[0]}\")\n",
                "}",
            ),
            "OK",
        );
    }

    /// Math.trunc：int 表达式恒等（不 32 位回绕——wl_probe2 实证原值
    /// -2147483647 垃圾）；float 截断。
    #[test]
    fn math_trunc_int_identity_and_float() {
        assert_out(
            concat!(
                "let d = 1756812345678\n",
                "let t = Math.trunc(d / 1000)\n",
                "let f = Math.trunc(2.9)\n",
                "if t == 1756812345 && f == 2 {\n",
                "    print(\"OK\")\n",
                "} else {\n",
                "    print(f\"t=${t} f=${f}\")\n",
                "}",
            ),
            "OK",
        );
    }

    /// Math.imul：小值直乘 + i32 回绕语义（hash 链依赖）。
    /// 123456789×1000 = 123456789000；mod 2^32 = 3197704712 > 2^31 →
    /// 有符号 = 3197704712 − 4294967296 = −1097262584。
    #[test]
    fn math_imul_wrapping_semantics() {
        assert_out(
            concat!(
                "let h = Math.imul(123456789, 1000)\n",
                "let m = Math.imul(3, 4)\n",
                "if h == -1097262584 && m == 12 {\n",
                "    print(\"OK\")\n",
                "} else {\n",
                "    print(f\"h=${h} m=${m}\")\n",
                "}",
            ),
            "OK",
        );
    }
}

/// PLAN-057 T7：未解析 web 内建编译期报错（等价性缺陷族③的门禁面）。
/// VM 轨对 Math.*/JSON.*/Object.*/Array.* 命名空间内**解析失败**的调用
/// 从「运行期静默 None 桩」升「编译期报错」；豁免=调用点同行或上一行
/// `// vm-safe-allow <原因>`（与 musk 侧 scripts/vm-safe-lint.mjs 同机制）。
/// 非四命名空间的未解析调用（Foo.bar 等）维持静默兜底（T4 语义面）。
#[cfg(test)]
mod musk_vm_track_p057_compile_gate {
    fn run_code(code: &str) -> Result<String, String> {
        match crate::run_with_capture(code) {
            Ok((_, stdout)) => Ok(stdout),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    /// 未实现内建 → 编译失败（错误信息点名豁免机制）。
    #[test]
    fn unresolved_web_builtin_is_compile_error() {
        let out = run_code("fn main() {\n    let x = Array.foo({ a: 1 })\n    print(x)\n}");
        assert!(
            matches!(&out, Err(e) if e.contains("web 内建") && e.contains("vm-safe-allow")),
            "expected compile error mentioning vm-safe-allow, got: {:?}",
            out
        );
    }

    /// `// vm-safe-allow` 豁免后编译通过（运行期仍静默 None——豁免≠实现）。
    #[test]
    fn vm_safe_allow_exempts_compile_error() {
        let out = run_code(concat!(
            "fn main() {\n",
            "    // vm-safe-allow 测试豁免\n",
            "    let x = Array.foo({ a: 1 })\n",
            "    if x == None { print(\"NONE\") }\n",
            "}",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains("NONE")),
            "expected exempted call to compile and yield None, got: {:?}",
            out
        );
    }

    /// 控制组：非四命名空间的未解析静态调用维持静默（T4 语义面不回归）。
    #[test]
    fn non_web_namespace_stays_silent() {
        let out = run_code("fn main() {\n    let x = Foo.bar({ a: 1 })\n    if x == None { print(\"NONE\") }\n}");
        assert!(
            matches!(&out, Ok(s) if s.contains("NONE")),
            "non-web unresolved call should stay silent, got: {:?}",
            out
        );
    }

    /// 控制组：已实现内建不受门禁影响。
    #[test]
    fn implemented_builtins_unaffected() {
        let out = run_code("fn main() {\n    print(Math.round(2.5))\n}");
        assert!(
            matches!(&out, Ok(s) if s.contains('3')),
            "Math.round should still work, got: {:?}",
            out
        );
    }
}

/// PLAN-057 T11 实机补遗：嵌套 compound 字段的 stringify。实机现场：
/// 工具卡展开 Arguments 显示裸堆句柄数字（"4067504"）——嵌套 VmRef 字段读
/// （GET_FIELD ObjectData 臂 rc_push_id）以裸 int 句柄（≥HEAP_ID_BASE 约定）
/// 出栈，nv_to_vm_value 的 i32 标量臂先于堆判定吞掉句柄。修=解码序前移。
#[cfg(test)]
mod musk_vm_track_p057_nested_stringify {
    fn run_code(code: &str) -> Result<String, String> {
        match crate::run_with_capture(code) {
            Ok((_, stdout)) => Ok(stdout),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    /// 对象字段嵌套对象的 stringify（实机 tc.arguments 形态）。
    #[test]
    fn stringify_nested_object_field() {
        let out = run_code(concat!(
            "fn main() {\n",
            "    let tc = { name: \"x\", arguments: { a: 1, b: \"s\" } }\n",
            "    let s = JSON.stringify(tc.arguments, null, 2)\n",
            "    if s == \"{\n  \\\"a\\\": 1,\n  \\\"b\\\": \\\"s\\\"\n}\" {\n",
            "        print(\"OK\")\n",
            "    } else {\n",
            "        print(f\"got=${s}\")\n",
            "    }\n",
            "}",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains("OK")),
            "expected pretty JSON of nested field, got: {:?}",
            out
        );
    }

    /// 列表元素嵌字段的 stringify（实机 blocks[i].tc.arguments 形态）。
    #[test]
    fn stringify_nested_list_elem_field() {
        let out = run_code(concat!(
            "fn main() {\n",
            "    var arr = []\n",
            "    arr.push({ kind: \"tool\", tc: { name: \"y\", arguments: { c: 2 } } })\n",
            "    let s = JSON.stringify(arr[0].tc.arguments)\n",
            "    if s == \"{\\\"c\\\":2}\" {\n",
            "        print(\"OK\")\n",
            "    } else {\n",
            "        print(f\"got=${s}\")\n",
            "    }\n",
            "}",
        ));
        assert!(
            matches!(&out, Ok(s) if s.contains("OK")),
            "expected compact JSON of nested list elem, got: {:?}",
            out
        );
    }

    /// 控制组：真小整数不被误判为堆句柄（<HEAP_ID_BASE 走标量）。
    #[test]
    fn stringify_plain_int_untouched() {
        let out = run_code("fn main() {\n    let s = JSON.stringify(42)\n    print(s)\n}");
        assert!(
            matches!(&out, Ok(s) if s.contains("42")),
            "expected 42, got: {:?}",
            out
        );
    }
}

/// PLAN-062 T1: retain 泄漏 soak——恒写 timer 拍 × 整树重建，VM 堆
/// live_heap 增量必须有界。
///
/// 现场（2026-09-05 实机定罪）：每次视图重建经 call_vm_fn/
/// call_computed_fn 求值 computed，`retain_heap_result` 对返回堆引用
/// +1 后无配对释放（KD-051 ⑤）——每拍泄漏整棵当帧 computed 输出树
/// （musk 空闲实测 0.85–2.2 MB/s，30–60 分钟达 4GB）。语料
/// `test/ui/plan062_memleak`（BeatTick 恒写 + `items => build_items(20)`
/// 表达式体 computed）同构 musk PollStream × filteredMessages 链。
#[cfg(feature = "ui-interpreter")]
mod musk_vm_track_p062_heap_soak {
    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan062_memleak/src/front/app.at";
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    /// 40 拍脏重建后 live_heap 增量 ≤ 64（warmup 10 拍后计基线）。
    /// 现状红：每拍保留 1 list + 20 obj ⇒ +840 量级。
    /// 修复后绿：帧账本换代释放，残差仅常数（字符串池 dedup 命中零增长）。
    #[test]
    fn musk_vm_track_heap_soak() {
        let Some(path) = locate_corpus() else {
            eprintln!("plan062: SKIPPED — corpus not found");
            return;
        };
        let mut dc = match crate::plan370_test_support::build_component_from_app(&path) {
            Some(c) => c,
            None => {
                eprintln!("plan062: SKIPPED — component build failed");
                return;
            }
        };
        // 镜像 renderer dirty 分支契约：build → 缓存写回 → commit_dirty_frame
        // （PLAN-062 F2 帧账本换代）。
        // 镜像 renderer dirty 分支契约：build → 缓存写回 → commit_dirty_frame
        // （PLAN-062 F2 帧账本换代）。
        let tick_rebuild = |dc: &mut crate::ui::dynamic::DynamicComponent| {
            dc.fire_timer("App", "BeatTick");
            let _ = dc.view_with_debug_gated(false);
            dc.commit_dirty_frame();
            dc.clear_dirty();
        };
        // ── 相 A：空闲（零状态写拍 × 40）——零求值零增长 ──
        // F1 契约：no-op 拍不置脏 → renderer 走缓存分支不重建 → 无
        // computed 求值、无 retain。此相断言 0 增长（musk 实机空闲泄漏
        // 0.85–2.2 MB/s 的根治面）。
        for _ in 0..10 {
            let _ = dc.fire_timer("App", "IdleTick");
        }
        dc.clear_dirty();
        let idle_base = dc.heap_live_objects();
        for _ in 0..40 {
            let fired = dc.fire_timer("App", "IdleTick");
            assert!(fired, "ungated entry still dispatches");
            assert!(!dc.is_dirty(), "idle tick must not dirty (F1)");
        }
        let idle_after = dc.heap_live_objects();
        eprintln!(
            "[P062-soak-A:idle] live_heap {} -> {} (+{})",
            idle_base,
            idle_after,
            idle_after.saturating_sub(idle_base)
        );
        assert_eq!(
            idle_after, idle_base,
            "idle phase (no-op ticks, no rebuilds) must be zero-growth: {} -> {}",
            idle_base, idle_after
        );

        // ── 相 B：脏重建（恒写拍 × 40）——速率绊线 ──
        // 残留已知债：每 call_vm_fn 有 1 个未定位归属的存量 stake（非
        // 任务帧槽/非全局表/非结果槽——canary 实证各清账路径均不安全），
        // 钉住当帧 computed 树 ⇒ ~+21 obj/rebuild。帧账本已归还宿主份额
        // （42→21/拍，隐式重建移除 + F2），根修归上游 RC 槽位记账专项
        // （KD-051⑤ 续行）。绊线口径：≤ +24/rebuild，回归到双倍泄漏
        // （+42/rebuild，隐式重建复发或账本失效）即红。
        for _ in 0..10 {
            tick_rebuild(&mut dc);
        }
        let base = dc.heap_live_objects();
        for _ in 0..40 {
            tick_rebuild(&mut dc);
        }
        let after = dc.heap_live_objects();
        eprintln!(
            "[P062-soak-B:rebuild] live_heap {} -> {} (+{})",
            base,
            after,
            after.saturating_sub(base)
        );
        assert!(
            after <= base + 24 * 40,
            "rebuild leak-rate tripped: base={} after={} (+{}/40 rebuilds, > 24/rebuild)",
            base,
            after,
            after.saturating_sub(base)
        );
    }
}
