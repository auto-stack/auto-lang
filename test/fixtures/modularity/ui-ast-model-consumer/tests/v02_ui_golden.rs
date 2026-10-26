use auto_ast::ast::{Name, ToAtom, ToNode};
use auto_ui_dialect::ast::*;
use auto_ui_dialect::UiStmt as Stmt;

fn name(s: &str) -> Name {
    Name::from(s)
}

fn project(out: &mut String, label: &str, value: &(impl std::fmt::Display + ToAtom + ToNode)) {
    out.push_str(&format!("### {label}\n"));
    out.push_str("--Display--\n");
    out.push_str(&format!("{value}\n"));
    out.push_str("--ToAtom--\n");
    out.push_str(&format!("{}\n", value.to_atom()));
    out.push_str("--ToNode--\n");
    out.push_str(&format!("{:?}\n\n", value.to_node()));
}

fn ui_golden() -> String {
    let golden = include_str!("../../ast-model-consumer/tests/v02-golden.txt");
    let labels = [
        "ui-widget-decl",
        "ui-store-decl",
        "ui-view-fragment",
        "ui-style-recipe",
        "ui-actions-decl",
        "ui-use-web",
        "ui-model-block",
        "ui-view-block",
        "ui-msg-decl",
    ];
    let mut selected = String::new();
    for section in golden.split("### ").skip(1) {
        if labels.iter().any(|label| section.starts_with(&format!("{label}\n"))) {
            selected.push_str("### ");
            selected.push_str(section);
        }
    }
    selected
}

#[test]
fn v02_golden_replay_auto_ui_dialect() {
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
    let store_decl = Stmt::StoreDecl(StoreDecl {
        name: name("counter"),
        messages: Vec::new(),
        model: None,
        computed: None,
        on: None,
        watch: Vec::new(),
        timer: None,
    });
    let view_frag = Stmt::ViewFragmentDecl(ViewFragmentDecl {
        name: name("row"),
        params: vec![(name("n"), "int".to_string())],
        body: ViewNode::Element {
            tag: "col".to_string(),
            props: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
            span: None,
        },
    });
    let style_recipe = Stmt::StyleRecipeDecl(StyleRecipeDecl {
        name: name("btn-primary"),
        params: Vec::new(),
        body: auto_ast::ast::Expr::Int(1),
        is_pub: false,
        doc: None,
    });
    let actions = Stmt::ActionsDecl(ActionsBlock {
        actions: Vec::new(),
        menubar: None,
        toolbar: None,
    });
    let use_web = Stmt::UseWeb(vec![ExtImport {
        kind: ExtImportKind::Composable,
        symbols: vec![name("useCounter")],
        path: name("@/composables/counter"),
        call_args: Vec::new(),
        ref_fields: Vec::new(),
    }]);
    let model_block = Stmt::ModelBlock(ModelBlock { fields: Vec::new() });
    let view_block = Stmt::ViewBlock(ViewBlock {
        root: ViewNode::Element {
            tag: "button".to_string(),
            props: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
            span: None,
        },
    });
    let msg_decl = Stmt::MsgDecl(MsgDecl {
        variants: vec![MsgVariant {
            name: name("Increment"),
            quoted: false,
            payload: Vec::new(),
            payload_names: Vec::new(),
        }],
    });

    let mut produced = String::new();
    project(&mut produced, "ui-widget-decl", &widget);
    project(&mut produced, "ui-store-decl", &store_decl);
    project(&mut produced, "ui-view-fragment", &view_frag);
    project(&mut produced, "ui-style-recipe", &style_recipe);
    project(&mut produced, "ui-actions-decl", &actions);
    project(&mut produced, "ui-use-web", &use_web);
    project(&mut produced, "ui-model-block", &model_block);
    project(&mut produced, "ui-view-block", &view_block);
    project(&mut produced, "ui-msg-decl", &msg_decl);
    assert_eq!(produced.trim_end(), ui_golden().trim_end());
}
