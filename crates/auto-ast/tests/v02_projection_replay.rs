//! PLAN-799 V02 投影保持的永久重放（crate 内，日常档自动执行）。
//!
//! golden 冻结自旧实现（auto-lang@143aff0a2 抽取前，36 case ×
//! Display/ToAtom/ToNode 三投影；生成记录 docs/plans/attachments/799/，
//! SHA256 1e74a8df…）——期望不得由新实现再造，逐字节相等即投影保持。
//! 独立消费方形态（manifest 仅依赖 auto-ast）另由
//! test/fixtures/modularity/ast-model-consumer 钉住（V01）。

use auto_ast::ast::*;

fn name(s: &str) -> Name {
    AutoStr::from(s)
}

fn body(stmts: Vec<Stmt>) -> Body {
    Body {
        stmts,
        has_new_line: false,
        source_lines: Vec::new(),
    }
}

fn int(i: i32) -> Expr {
    Expr::Int(i)
}

fn ident(s: &str) -> Expr {
    Expr::Ident(name(s))
}

fn call(fname: &str, args: Vec<Arg>) -> Call {
    Call {
        name: Box::new(ident(fname)),
        args: Args { args },
        ret: Type::Void,
        type_args: Vec::new(),
        generic_args: Vec::new(),
        pos: None,
    }
}

fn store(kind: StoreKind, n: &str, ty: Type, e: Expr) -> Stmt {
    Stmt::Store(Store {
        kind,
        name: name(n),
        ty,
        expr: e,
        attrs: Vec::new(),
        is_pub: false,
    })
}

fn fn_decl(fname: &str, fbody: Body) -> Fn {
    Fn {
        kind: FnKind::Function,
        name: name(fname),
        parent: None,
        params: Vec::new(),
        body: fbody,
        ret: Type::Void,
        ret_name: None,
        is_static: false,
        is_pub: false,
        is_mut: false,
        is_test: false,
        type_params: Vec::new(),
        const_params: Vec::new(),
        doc: None,
        span: None,
        api_attrs: None,
        attrs: Vec::new(),
        export_abi: None,
    }
}

thread_local! {
    static OUT: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// 单 case 的三投影输出（与探针 println 版逐字符同构）。
fn project(label: &str, v: &(impl std::fmt::Display + ToAtom + ToNode)) {
    OUT.with(|o| {
        let mut o = o.borrow_mut();
        o.push_str(&format!("### {}\n", label));
        o.push_str("--Display--\n");
        o.push_str(&format!("{}\n", v));
        o.push_str("--ToAtom--\n");
        o.push_str(&format!("{}\n", v.to_atom()));
        o.push_str("--ToNode--\n");
        o.push_str(&format!("{:?}\n", v.to_node()));
        o.push('\n');
    });
}

fn v02_core_statements_and_exprs() {
    // Core：let/if/for/try/is + 表达式全家桶 + 嵌套
    let let_x = store(
        StoreKind::Let,
        "x",
        Type::Int,
        Expr::Bina(Box::new(int(1)), Op::Add, Box::new(int(2))),
    );

    let if_stmt = Stmt::If(If {
        branches: vec![Branch {
            cond: Expr::Bina(Box::new(ident("x")), Op::Gt, Box::new(int(10))),
            body: body(vec![store(
                StoreKind::Var,
                "y",
                Type::StrOwned,
                Expr::Str(name("big")),
            )]),
        }],
        else_: Some(body(vec![Stmt::Expr(ident("small"))])),
    });

    let for_stmt = Stmt::For(For {
        iter: Iter::Indexed(name("i"), name("items")),
        range: ident("items"),
        body: body(vec![Stmt::Expr(Expr::Call(call(
            "push",
            vec![Arg::Pos(ident("i"))],
        )))]),
        new_line: false,
        init: None,
    });

    let try_stmt = Stmt::Try(Try {
        body: body(vec![Stmt::Expr(ident("risky"))]),
        catch_param: Some("e".to_string()),
        catch_body: body(vec![Stmt::Expr(Expr::Str(name("caught")))]),
        finally_body: Some(body(vec![Stmt::Expr(ident("cleanup"))])),
        new_line: false,
    });

    let is_stmt = Stmt::Is(Is {
        target: ident("res"),
        branches: vec![
            IsBranch::EqBranch(
                vec![Expr::Ok(Box::new(ident("v")))],
                body(vec![Stmt::Return(Box::new(ident("v")))]),
            ),
            IsBranch::ElseBranch(body(vec![Stmt::Break])),
        ],
    });

    let mut code = Code::new();
    code.stmts = vec![
        let_x,
        if_stmt,
        for_stmt,
        try_stmt,
        is_stmt,
        Stmt::Return(Box::new(Expr::Tuple(vec![int(1), int(2)]))),
    ];
    code.source_lines = vec![1, 2, 5, 9, 14, 16];

    project("core-code", &code);
}

fn v02_expr_variants() {
    // 表达式变体：闭包/FStr/切片索引/对象/cast/async-await
    let closure = Expr::Closure(Closure {
        params: vec![ClosureParam {
            name: name("a"),
            ty: Some(Type::Int),
        }],
        ret: Some(Type::Int),
        body: Box::new(Expr::Bina(Box::new(ident("a")), Op::Mul, Box::new(int(2)))),
        is_move: false,
    });

    let fstr = Expr::FStr(FStr::new(vec![
        Expr::Str(name("n=")),
        ident("n"),
        Expr::Str(name("!")),
    ]));

    let idx = Expr::Index(
        Box::new(ident("arr")),
        Box::new(Expr::Range(Range {
            start: Box::new(int(0)),
            end: Box::new(Expr::Nil),
            eq: false,
        })),
    );

    let obj = Expr::Object(vec![
        Pair {
            key: Key::StrKey(name("width")),
            value: Box::new(int(320)),
        },
        Pair {
            key: Key::StrKey(name("title")),
            value: Box::new(Expr::Str(name("demo"))),
        },
    ]);

    let cast = Expr::Cast {
        expr: Box::new(ident("raw")),
        target_type: Type::Uint,
    };

    let compound = Expr::AsyncBlock {
        body: body(vec![Stmt::Expr(Expr::Await {
            expr: Box::new(Expr::Call(call(
                "fetch",
                vec![Arg::Pos(Expr::Str(name("url")))],
            ))),
        })]),
        return_type: Some(Type::StrSlice),
    };

    project("expr-closure", &closure);
    project("expr-fstr", &fstr);
    project("expr-index-slice", &idx);
    project("expr-object", &obj);
    project("expr-cast", &cast);
    project("expr-async-await", &compound);
}

fn v02_generics_and_decls() {
    // 泛型：GenericInstance 类型 + TypeDecl 泛型参数 + 泛型 TypeAlias
    let list_int = Type::GenericInstance(GenericInstance {
        base_name: name("List"),
        args: vec![Type::Int],
        source: None,
    });

    let type_decl = Stmt::TypeDecl(TypeDecl {
        name: name("Box"),
        kind: TypeDeclKind::UserType,
        parent: None,
        has: Vec::new(),
        specs: Vec::new(),
        spec_impls: Vec::new(),
        generic_params: vec![GenericParam::Type(TypeParam {
            name: name("T"),
            constraint: Vec::new(),
        })],
        members: vec![Member {
            name: name("value"),
            ty: Type::GenericInstance(GenericInstance {
                base_name: name("T"),
                args: Vec::new(),
                source: None,
            }),
            value: None,
            attrs: Vec::new(),
        }],
        delegations: Vec::new(),
        methods: Vec::new(),
        consts: Vec::new(),
        attrs: Vec::new(),
        impl_attrs: Vec::new(),
        doc: None,
        is_pub: false,
    });

    let alias_decl = Stmt::TypeAlias(TypeAlias {
        name: name("Ints"),
        params: Vec::new(),
        target: list_int.clone(),
    });

    let enum_item = |n: &str, v: Option<i32>| EnumItem {
        name: name(n),
        scalar_value: v,
        value_explicit: v.is_some(),
        payload_type: None,
        payload_types: Vec::new(),
        fields: Vec::new(),
        attrs: Vec::new(),
    };

    let enum_decl = Stmt::EnumDecl(EnumDecl {
        name: name("Color"),
        items: vec![enum_item("Red", Some(0)), enum_item("Green", None)],
        kind: EnumKind::Scalar {
            repr_type: Some(Type::Uint),
        },
        doc: None,
        is_pub: false,
        attrs: Vec::new(),
    });

    project("generic-type", &list_int);
    project("decl-type", &type_decl);
    project("decl-typealias", &alias_decl);
    project("decl-enum", &enum_decl);
}

fn v02_patterns_and_covers() {
    // 模式：TagCover/TagUncover + Option/Result/Struct pattern
    let cover = Expr::Cover(Cover::Tag(TagCover {
        kind: name("Shape"),
        tag: name("Circle"),
        bindings: vec![name("r")],
    }));

    let uncover = Expr::Uncover(TagUncover {
        src: name("shape"),
        cover: TagCover {
            kind: name("Shape"),
            tag: name("Rect"),
            bindings: vec![name("w"), name("h")],
        },
        binding: name("area"),
    });

    let some_pat = Expr::OptionPattern(OptionCover {
        variant: OptionVariant::Some,
        binding: Some(name("v")),
    });

    let err_pat = Expr::ResultPattern(ResultCover {
        variant: ResultVariant::Err,
        binding: Some(name("e")),
        inner: None,
    });

    let struct_pat = Expr::StructPattern(StructCover {
        type_name: name("Point"),
        variant: Some(name("Cartesian")),
        fields: vec![
            FieldBinding {
                field: name("x"),
                binding: name("x"),
            },
            FieldBinding {
                field: name("y"),
                binding: name("dy"),
            },
        ],
    });

    project("pattern-cover-tag", &cover);
    project("pattern-uncover", &uncover);
    project("pattern-option-some", &some_pat);
    project("pattern-result-err", &err_pat);
    project("pattern-struct", &struct_pat);
}

fn v02_task_and_events() {
    // 任务：TaskDef + OnEvents/Arrow 事件族
    let task_def = Stmt::TaskDef(TaskDef {
        name: name("sync"),
        attrs: vec![TaskAttr::Single],
        state: vec![(name("count"), true, int(0))],
        start_hook: Some(fn_decl("start", body(vec![Stmt::Expr(ident("init"))]))),
        stop_hook: None,
        on_block: TaskOnBlock {
            context_param: Some(name("ctx")),
            handlers: vec![(
                TaskMsgPattern::Simple(name("Tick")),
                None,
                body(vec![Stmt::Expr(ident("tick"))]),
            )],
            else_handler: None,
            pos: Pos {
                line: 3,
                at: 20,
                pos: 20,
                len: 13,
            },
        },
        pos: Pos {
            line: 1,
            at: 0,
            pos: 0,
            len: 4,
        },
    });

    let on_events = OnEvents {
        branches: vec![
            Event::Arrow(Arrow {
                src: Some(int(5)),
                dest: Some(int(7)),
                with: Some(int(10)),
            }),
            Event::CondArrow(CondArrow {
                src: Some(ident("btn")),
                cond: Expr::Bool(true),
                subs: vec![Arrow {
                    src: Some(int(9)),
                    dest: Some(int(10)),
                    with: None,
                }],
            }),
        ],
    };

    let on_stmt = Stmt::OnEvents(OnEvents {
        branches: on_events.branches.clone(),
    });

    project("task-def", &task_def);
    project("on-events", &on_stmt);
}

fn v02_imports_and_paths() {
    // 导入：Use（Auto/Rust）+ ModulePath/PathPrefix + Dep
    let use_auto = Stmt::Use(Use {
        kind: UseKind::Auto,
        module_path: Some(ModulePath {
            prefix: PathPrefix::None,
            segments: vec![name("math"), name("trig")],
            items: vec![name("sin"), name("cos")],
        }),
        paths: Vec::new(),
        items: Vec::new(),
        is_pub: false,
        is_wildcard: false,
    });

    let use_rust = Stmt::Use(Use {
        kind: UseKind::Rust,
        module_path: Some(ModulePath {
            prefix: PathPrefix::Pac,
            segments: vec![name("serde"), name("json")],
            items: Vec::new(),
        }),
        paths: Vec::new(),
        items: Vec::new(),
        is_pub: true,
        is_wildcard: false,
    });

    let dep = Stmt::Dep(DepStmt {
        name: name("iced"),
        version: Some(name("0.13")),
        features: vec![name("canvas")],
        path: None,
        git: None,
        git_ref: None,
    });

    project("import-use-auto", &use_auto);
    project("import-use-rust", &use_rust);
    project("import-dep", &dep);
}

fn v02_ui_compat_nodes() {
    // UI 兼容节点：WidgetDecl/StoreDecl/ViewFragmentDecl/StyleRecipeDecl/
    // ActionsDecl/MsgDecl/ModelBlock/ViewBlock/SceneDecl/UseWeb
    let widget = Stmt::WidgetDecl(WidgetDecl {
        name: name("Counter"),
        messages: Vec::new(),
        model: None,
        computed: None,
        view: None,
        named_views: Vec::new(),
        on: None,
        bind: None,
        props: Vec::new(),
        routes: None,
        lifecycle: Vec::new(),
        style: None,
        ext_imports: Vec::new(),
        watch: Vec::new(),
        setup: None,
        expose: Vec::new(),
        actions: None,
        timer: None,
    });

    let store_decl = Stmt::StoreDecl(ui::StoreDecl {
        name: name("counter"),
        messages: Vec::new(),
        model: None,
        computed: None,
        on: None,
        watch: Vec::new(),
        timer: None,
    });

    let view_frag = Stmt::ViewFragmentDecl(ui::ViewFragmentDecl {
        name: name("row"),
        params: vec![(name("n"), "int".to_string())],
        body: ui::ViewNode::Element {
            tag: "col".to_string(),
            props: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
            span: None,
        },
    });

    let style_recipe = Stmt::StyleRecipeDecl(ui::StyleRecipeDecl {
        name: name("btn-primary"),
        params: Vec::new(),
        body: Expr::Int(1),
        is_pub: false,
        doc: None,
    });

    let actions = Stmt::ActionsDecl(ui::ActionsBlock {
        actions: Vec::new(),
        menubar: None,
        toolbar: None,
    });

    let use_web = Stmt::UseWeb(vec![ui::ExtImport {
        kind: ui::ExtImportKind::Composable,
        symbols: vec![name("useCounter")],
        path: name("@/composables/counter"),
        call_args: Vec::new(),
        ref_fields: Vec::new(),
    }]);

    let model_block = Stmt::ModelBlock(ui::ModelBlock { fields: Vec::new() });

    let view_block = Stmt::ViewBlock(ui::ViewBlock {
        root: ui::ViewNode::Element {
            tag: "button".to_string(),
            props: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
            span: None,
        },
    });

    let scene = Stmt::SceneDecl(ui::SceneDecl {
        name: name("Main"),
        node_type: name("Control"),
        props: Vec::new(),
        script: None,
        children: Vec::new(),
        connections: Vec::new(),
        signals: Vec::new(),
    });

    let msg_decl = Stmt::MsgDecl(MsgDecl {
        variants: vec![MsgVariant {
            name: name("Increment"),
            quoted: false,
            payload: Vec::new(),
            payload_names: Vec::new(),
        }],
    });

    project("ui-widget-decl", &widget);
    project("ui-store-decl", &store_decl);
    project("ui-view-fragment", &view_frag);
    project("ui-style-recipe", &style_recipe);
    project("ui-actions-decl", &actions);
    project("ui-use-web", &use_web);
    project("ui-model-block", &model_block);
    project("ui-view-block", &view_block);
    project("ui-scene-decl", &scene);
    project("ui-msg-decl", &msg_decl);
}

fn v02_nested_deep() {
    // 嵌套 AST：多层 body/expr/lambda/hold/grid/node 深嵌套 + comptime
    let nested = Stmt::If(If {
        branches: vec![Branch {
            cond: Expr::Bina(
                Box::new(Expr::Call(call("check", vec![Arg::Pos(ident("deep"))]))),
                Op::And,
                Box::new(Expr::Bool(true)),
            ),
            body: body(vec![Stmt::Fn({
                let mut f = fn_decl("inner", Body::new());
                f.params = vec![Param {
                    name: name("z"),
                    ty: Type::Float,
                    default: None,
                    mode: ParamMode::View,
                    destructure: None,
                }];
                f.body = body(vec![Stmt::Expr(Expr::Hold(Hold {
                    path: Box::new(Expr::Dot(Box::new(ident("ctx")), name("res"))),
                    name: name("h"),
                    body: body(vec![Stmt::Expr(Expr::Yield(Box::new(ident("h"))))]),
                    span: None,
                }))]);
                f
            })]),
        }],
        else_: None,
    });

    let node_expr = Expr::Node(Node::new("Button"));
    let grid = Expr::Grid(Grid {
        head: Args::new(),
        data: vec![vec![int(1), int(2)], vec![int(3), int(4)]],
    });

    let macro_call = Stmt::MacroCall(MacroCall {
        name: name("debug"),
        args: vec![ident("x")],
    });

    let comptime = Stmt::HashIf(HashIf {
        cond: Expr::Bool(true),
        then_block: body(vec![macro_call.clone()]),
        else_block: Some(HashIfElse::Block(body(vec![Stmt::Expr(ident("fallback"))]))),
    });

    project("nested-deep-if-fn-hold", &nested);
    project("expr-node", &node_expr);
    project("expr-grid", &grid);
    project("comptime-macro", &macro_call);
    project("comptime-hash-if", &comptime);
}

#[test]
fn v02_golden_replay_auto_ast() {
    v02_core_statements_and_exprs();
    v02_expr_variants();
    v02_generics_and_decls();
    v02_imports_and_paths();
    v02_nested_deep();
    v02_patterns_and_covers();
    v02_task_and_events();
    v02_ui_compat_nodes();
    let produced = OUT.with(|o| std::mem::take(&mut *o.borrow_mut()));
    let golden = include_str!("v02-golden.txt");
    assert!(
        produced.trim_end() == golden.trim_end(),
        "auto_ast projection drifted from frozen old-implementation golden;
--- first diff around ---
{}",
        first_diff(&produced, golden)
    );
}

fn first_diff(a: &str, b: &str) -> String {
    let al: Vec<&str> = a.lines().collect();
    let bl: Vec<&str> = b.lines().collect();
    for i in 0..al.len().max(bl.len()) {
        let x = al.get(i).copied().unwrap_or("<eof>");
        let y = bl.get(i).copied().unwrap_or("<eof>");
        if x != y {
            return format!("line {}:\n  new: {}\n  old: {}", i + 1, x, y);
        }
    }
    "no diff".to_string()
}
