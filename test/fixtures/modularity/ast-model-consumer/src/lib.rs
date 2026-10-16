//! PLAN-799 V01 外部 AST 模型消费方（AC-01）。
//!
//! 本 crate 的 manifest 是验收证据：唯一依赖是 `auto-ast`（path）——构建并
//! 运行这些测试证明 AST 模型可在没有 auto-lang（Parser/TypeStore/推断/
//! VM/转译）/auto-man/UI 的依赖树里真实构造、检查、序列化，不借主库
//! rlib。V02 投影保持的 golden 重放在 tests/v02_replay.rs。

use auto_ast::ast::{
    AtomWriter, AutoStr, Body, Closure, ClosureParam, Code, Expr, Fn, FnKind, GenericInstance,
    ModulePath, Name, OnEvents, Op, Pair, PathPrefix, Stmt, ToAtom, ToAtomStr, ToNode, Type, Use,
    UseKind, Value, WidgetDecl,
};

#[test]
fn consumer_constructs_core_and_ui_nodes_without_compiler_stack() {
    // Core：语句/表达式/泛型类型构造 + 序列化三投影
    let mut code = Code::new();
    code.stmts.push(Stmt::Expr(Expr::Int(42)));
    code.file_attrs.push(Name::from("tool"));

    let list_int = Type::GenericInstance(GenericInstance {
        base_name: Name::from("List"),
        args: vec![Type::Int],
        source: None,
    });

    let node = code.to_node();
    assert_eq!(node.name.as_str(), "code");
    assert_eq!(code.to_atom().as_str(), "(code 42)");
    assert!(format!("{}", code).starts_with("(code"));

    let ty_atom = list_int.to_atom();
    assert_eq!(ty_atom.as_str(), "List<int>");

    // UI 兼容节点构造（无 renderer 依赖的共享模型面）
    let widget = WidgetDecl {
        name: Name::from("Counter"),
        ..empty_widget()
    };
    assert_eq!(widget.name.as_str(), "Counter");
}

#[test]
fn consumer_module_path_and_import_model() {
    let mp = ModulePath {
        prefix: PathPrefix::Pac,
        segments: vec![Name::from("db"), Name::from("users")],
        items: vec![Name::from("find")],
    };
    assert!(mp.display().contains("pac.db.users"));

    let use_stmt = Stmt::Use(Use {
        kind: UseKind::Auto,
        module_path: Some(mp),
        paths: Vec::new(),
        items: Vec::new(),
        is_pub: false,
        is_wildcard: false,
    });
    assert!(format!("{}", use_stmt).starts_with("(use"));
}

#[test]
fn consumer_traits_and_type_identity() {
    // Name = AutoStr 同一身份（pub type 别名，非 newtype 转换副本）；经
    // auto-ast 同源 re-export 直接命名 AutoStr 互证。
    let n: Name = "x".into();
    let s: AutoStr = n.clone();
    assert_eq!(s.as_str(), "x");
    let m: Name = s;
    assert_eq!(m.as_str(), "x");

    // ToAtomStr blanket（AtomWriter → AutoStr）与 AtomWriter 直写
    let expr = Expr::Bina(Box::new(Expr::Int(1)), Op::Add, Box::new(Expr::Int(2)));
    let mut buf = Vec::new();
    expr.write_atom(&mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "bina(+, 1, 2)");
    assert_eq!(expr.to_atom_str().as_str(), "bina(+, 1, 2)");

    // 闭包/函数/事件模型方法可构造可投影
    let closure = Expr::Closure(Closure {
        params: vec![ClosureParam {
            name: Name::from("a"),
            ty: Some(Type::Int),
        }],
        ret: None,
        body: Box::new(Expr::Ident(Name::from("a"))),
        is_move: false,
    });
    assert!(format!("{}", closure).starts_with("(closure"));

    let mut f = Fn {
        kind: FnKind::Function,
        name: Name::from("main"),
        ..empty_fn()
    };
    f.body = Body::new();
    assert!(format!("{}", f).starts_with("(fn"));

    let on = OnEvents {
        branches: Vec::new(),
    };
    assert_eq!(on.to_atom_str().as_str(), "on { }");

    // ToNode 产物是 auto_val::Node（同源 re-export 为 AutoNode）——消费方
    // 无需直接依赖 auto-val 即可检查。
    let v = Value::Int(7);
    assert_eq!(v.to_string(), "7");

    let pair = Pair {
        key: auto_ast::ast::Key::StrKey(Name::from("k")),
        value: Box::new(Expr::Int(1)),
    };
    assert_eq!(pair.to_atom_str().as_str(), "pair(\"k\", 1)");
}

// —— 构造脚手架（字段全集见模型定义；此处只填最小面）——

fn empty_widget() -> WidgetDecl {
    WidgetDecl {
        name: Name::new(),
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
    }
}

fn empty_fn() -> Fn {
    Fn {
        kind: FnKind::Function,
        name: Name::new(),
        parent: None,
        params: Vec::new(),
        body: Body::new(),
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
