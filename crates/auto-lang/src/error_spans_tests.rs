//! Error-span regression tests (jade gaps 17/18/29/37/46).
//!
//! Each test pins either (a) that a formerly-mislocated error now reports at
//! the offending token, or (b) that a formerly-broken syntax now parses
//! (self-healed by an earlier batch — kept as a regression guard).

use crate::ast::{Expr, Stmt, ViewNode, ViewPropValue};
use crate::error::AutoError;
use crate::parser::Parser;
use crate::session::CompilerSession;
use miette::Diagnostic;

fn parse_ui(code: &str) -> Result<crate::ast::Code, AutoError> {
    let mut parser = Parser::from(code).with_session(CompilerSession::ui());
    parser.parse()
}

/// Offset of the first miette label of an error (delegates into the first
/// inner error for MultipleErrors).
fn first_label_offset(e: &AutoError) -> Option<usize> {
    e.labels()
        .and_then(|mut it| it.next())
        .map(|l| l.offset())
}

fn widget_decl(code: &crate::ast::Code) -> &crate::ast::WidgetDecl {
    code.stmts
        .iter()
        .find_map(|s| match s {
            Stmt::WidgetDecl(w) => Some(w),
            _ => None,
        })
        .expect("widget decl")
}

/// jade gap 17: `msg Msg { OpenSource(map) }` — payload-typed variant in a
/// widget msg decl. Was "Expected term, got RBrace" deep inside the view
/// block; self-healed by Plan 043 M5 (multi-param msg payloads). Guard: the
/// payload type must land on the variant.
#[test]
fn gap17_msg_variant_payload_parses_with_type() {
    let code = concat!(
        "widget App {\n",
        "  msg Msg { OpenSource(map) }\n",
        "  model { count int = 0 }\n",
        "  view {\n",
        "    col {\n",
        "      text \"hello\"\n",
        "    }\n",
        "  }\n",
        "}\n"
    );
    let ast = parse_ui(code).expect("msg variant with payload type must parse");
    let w = widget_decl(&ast);
    assert_eq!(w.messages.len(), 1);
    assert_eq!(w.messages[0].variants.len(), 1);
    assert_eq!(w.messages[0].variants[0].name, "OpenSource");
    assert_eq!(w.messages[0].variants[0].payload.len(), 1);
}

/// jade gaps 18/29/34: reserved words (`link`, `task`) as for-loop variables
/// in a view. Was "Expected term, got RBrace" at the loop's closing brace;
/// self-healed by c5b5fecf (contextual/soft keywords usable as identifiers).
#[test]
fn gap18_29_reserved_words_as_loop_vars_parse() {
    for var in ["link", "task"] {
        let code = format!(
            concat!(
                "widget App {{\n",
                "  model {{ var items List<str> = [] }}\n",
                "  view {{\n",
                "    col {{\n",
                "      for {} in .items {{\n",
                "        text \"x\"\n",
                "      }}\n",
                "    }}\n",
                "  }}\n",
                "}}\n"
            ),
            var
        );
        let ast = parse_ui(&code).unwrap_or_else(|e| {
            panic!("loop variable `{}` must parse: {}", var, e);
        });
        let w = widget_decl(&ast);
        let view = w.view.as_ref().expect("view block");
        let children = match &view.root {
            ViewNode::Element { children, .. } => children,
            other => panic!("expected element root, got {:?}", other),
        };
        match &children[0] {
            ViewNode::ForLoop { var: v, .. } => assert_eq!(v, var),
            other => panic!("expected ForLoop, got {:?}", other),
        }
    }
}

/// jade gap 37a (the fix of this plan): `onclick.self: ."update:open"(false)`
/// — a bool literal as a view event argument is unsupported. The error must
/// be loud and point AT the offending `false` token, not at the view's
/// closing `}`.
#[test]
fn gap37a_event_literal_arg_error_points_at_offending_token() {
    let code = concat!(
        "widget App {\n",
        "  model { var open bool = false }\n",
        "  view {\n",
        "    col {\n",
        "      button { onclick.self: .\"update:open\"(false) }\n",
        "    }\n",
        "  }\n",
        "}\n"
    );
    let err = parse_ui(code).expect_err("bool literal event arg must fail");

    // The root-cause error must mention the offending token...
    let msg = err.to_string();
    assert!(
        msg.contains("unsupported event argument `false`"),
        "root-cause message must name the offending token, got: {}",
        msg
    );

    // ...and its span must be the `false` token, not a closing brace.
    let false_off = code.find("(false)").unwrap() + 1;
    let label_off = first_label_offset(&err).expect("error must carry a label");
    assert_eq!(
        label_off, false_off,
        "error span must point at the `false` literal (code[..] = {:?})",
        &code[false_off..false_off + 5]
    );
    assert_eq!(&code[label_off..label_off + 5], "false");
}

/// Same root cause, unquoted handler with a bool arg (`.Inc(false)`).
#[test]
fn gap37a_bool_arg_error_span_unquoted_handler() {
    let code = concat!(
        "widget App {\n",
        "  model { var x int = 0 }\n",
        "  view {\n",
        "    col {\n",
        "      button { onclick: .Inc(false) }\n",
        "    }\n",
        "  }\n",
        "}\n"
    );
    let err = parse_ui(code).expect_err("bool literal event arg must fail");
    let false_off = code.find("(false)").unwrap() + 1;
    let label_off = first_label_offset(&err).expect("error must carry a label");
    assert_eq!(label_off, false_off);
    assert!(err.to_string().contains("unsupported event argument `false`"));
}

/// jade gap 37b: `.Handler(a, e)` multi-param on-handlers — both params must
/// be in scope in the body (parse) and in the generated Vue handler
/// signature (codegen). Self-healed on current master; kept as a guard.
#[test]
fn gap37b_handler_multi_params_stay_in_scope() {
    use crate::ui_gen::{BackendGenerator, VueGenerator};

    let code = concat!(
        "widget App {\n",
        "  model { var x int = 0 }\n",
        "  msg Msg { Update }\n",
        "  view {\n",
        "    col {\n",
        "      button { onclick: .Update(.x, $event) }\n",
        "    }\n",
        "  }\n",
        "  on {\n",
        "    .Update(a, e) -> {\n",
        "      .x = a\n",
        "      log(e)\n",
        "    }\n",
        "  }\n",
        "}\n"
    );
    let ast = parse_ui(code).expect("multi-param handler must parse");
    let w = widget_decl(&ast);
    let on = w.on.as_ref().expect("on block");
    assert_eq!(on.handlers[0].params, vec!["a".to_string(), "e".to_string()]);

    let widget = crate::aura::extract_widget_from_decl(w).expect("extract widget");
    let sfc = VueGenerator::new().generate(&widget).expect("generate SFC");
    assert!(
        sfc.contains("function Update(a: any, e: any)"),
        "both handler params must reach the Vue signature, got:\n{}",
        sfc
    );
}

/// jade gap 46: `text nodeLabel(node)` — a Call expression as the text
/// primary prop. Was "Expected term, got RBrace" at the loop's closing
/// brace; self-healed by 34c49427 (text fn call goes through Expr::Call).
#[test]
fn gap46_text_fn_call_parses_as_call_expr() {
    let code = concat!(
        "widget App {\n",
        "  model { var nodes List<str> = [] }\n",
        "  view {\n",
        "    col {\n",
        "      for node in .nodes {\n",
        "        text nodeLabel(node)\n",
        "      }\n",
        "    }\n",
        "  }\n",
        "}\n"
    );
    let ast = parse_ui(code).expect("text fn call must parse");
    let w = widget_decl(&ast);
    let view = w.view.as_ref().expect("view block");
    let children = match &view.root {
        ViewNode::Element { children, .. } => children,
        other => panic!("expected element root, got {:?}", other),
    };
    let loop_body = match &children[0] {
        ViewNode::ForLoop { body, .. } => body,
        other => panic!("expected ForLoop, got {:?}", other),
    };
    let text_props = match &loop_body[0] {
        ViewNode::Element { props, .. } => props,
        other => panic!("expected text element, got {:?}", other),
    };
    let is_call = text_props.iter().any(|p| {
        matches!(&p.value, ViewPropValue::Expr(Expr::Call(_)))
    });
    assert!(is_call, "text primary prop must be a Call expr: {:?}", text_props);
}
