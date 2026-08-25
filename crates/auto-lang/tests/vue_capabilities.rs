//! Plan 012 batch C — production-verified capability regression locks.
//!
//! Each test locks one capability that was verified in production by the
//! jade-garden (front/auto/README.md "新验证能力" sections) and
//! @autodown/editor (src/auto/README.md) migrations but had no
//! compiler-side regression test. Every test goes through the REAL parse
//! pipeline (widget source → Parser → extract_widget_from_decl →
//! VueGenerator) — no hand-built AST.
//!
//! Mechanism: fragment assertions on the generated SFC, NOT insta
//! snapshots. Two reasons:
//! 1. The existing insta infra (plan 362, tests/ui_snapshots.rs) snapshots
//!    AuraWidget *extraction* and embeds CARGO_MANIFEST_DIR-absolute paths,
//!    so those tests fail in worktrees (known environmental exclusion).
//! 2. SFC emission order is HashMap-unstable across builds (jade README
//!    gap 56), so full-file snapshots of SFC output would be brittle.
//!    Single-fragment substring assertions are immune to both.

use auto_lang::aura::extract_widget_from_decl;
use auto_lang::ast::Stmt;
use auto_lang::parser::Parser;
use auto_lang::session::CompilerSession;
use auto_lang::ui_gen::{BackendGenerator, VueGenerator};

/// Parse a widget source and generate its Vue SFC (full real pipeline).
fn gen_sfc(src: &str) -> String {
    let session = CompilerSession::ui();
    let mut parser = Parser::from(src).with_session(session);
    let ast = parser.parse().expect("widget source must parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = extract_widget_from_decl(decl).expect("extract widget");
    VueGenerator::new().generate(&widget).expect("generate SFC")
}

/// Plan 028: parse a store source and generate its composable (real pipeline).
fn gen_store(src: &str) -> String {
    let session = CompilerSession::ui();
    let mut parser = Parser::from(src).with_session(session);
    let ast = parser.parse().expect("store source must parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            Stmt::StoreDecl(d) => Some(d),
            _ => None,
        })
        .expect("store decl");
    let store = auto_lang::aura::extract_store_from_decl(decl).expect("extract store");
    VueGenerator::generate_store_composable(&store)
}

/// jade 5.3a (FileTreeNode): a widget may render ITSELF in its own view —
/// codegen emits a self import from `@/components/<Name>.vue`, and an
/// explicit `key:` prop overrides the automatic constant key.
#[test]
fn cap_recursive_widget_self_reference() {
    let sfc = gen_sfc(
        r#"
widget FileTreeNode {
    model { var children list = [] }
    view {
        col {
            for child in .children {
                FileTreeNode(key: child.path)
            }
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("import FileTreeNode from '@/components/FileTreeNode.vue'"),
        "recursive widget self-import:\n{sfc}"
    );
    assert!(
        sfc.contains(":key=\"child.path\""),
        "explicit key on recursive instance:\n{sfc}"
    );
}

/// jade 5.3a/5.3d: a file-top `use file_tree_node: FileTreeNode` sibling
/// widget reference emits a direct `@/components` import (no ext re-export
/// needed).
#[test]
fn cap_sibling_widget_use_direct_reference() {
    let sfc = gen_sfc(
        r#"
use file_tree_node: FileTreeNode

widget App {
    view {
        col {
            FileTreeNode { }
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("import FileTreeNode from '@/components/FileTreeNode.vue'"),
        "sibling widget direct import:\n{sfc}"
    );
    assert!(sfc.contains("<FileTreeNode"), "sibling widget rendered:\n{sfc}");
}

/// editor README / jade batch 3: `.Init` → onMounted, `.Destroy` →
/// onUnmounted, with handler bodies intact (`.x = 1` → `x.value = 1`).
#[test]
fn cap_lifecycle_init_destroy() {
    let sfc = gen_sfc(
        r#"
widget W {
    msg Msg { Init, Destroy }
    model { var x int = 0 }
    view { col { text "hi" } }
    on {
        .Init -> { .x = 1 }
        .Destroy -> { .x = 0 }
    }
}
"#,
    );
    assert!(sfc.contains("onMounted(() => {"), ".Init → onMounted:\n{sfc}");
    assert!(sfc.contains("onUnmounted(() => {"), ".Destroy → onUnmounted:\n{sfc}");
    assert!(sfc.contains("x.value = 1"), "init body:\n{sfc}");
    assert!(sfc.contains("x.value = 0"), "destroy body:\n{sfc}");
}

/// jade 5.3c (EditorTab): `on "ns:evt".window:` registers a window-level
/// CustomEvent listener with a sanitized wrapper, torn down in
/// onUnmounted (no leak). Must be `.window` for window.dispatchEvent.
#[test]
fn cap_window_custom_event_listener_with_cleanup() {
    let sfc = gen_sfc(
        r#"
widget W {
    msg Msg { OnScrollToBlock }
    model { var x int = 0 }
    view {
        col {
            on "jade-scroll-to-block".window: .OnScrollToBlock($event)
        }
    }
    on {
        .OnScrollToBlock(e) -> { .x = 1 }
    }
}
"#,
    );
    assert!(
        sfc.contains("window.addEventListener('jade-scroll-to-block', __auto_gl_jade_scroll_to_block_OnScrollToBlock)"),
        "window listener registered:\n{sfc}"
    );
    assert!(
        sfc.contains("window.removeEventListener('jade-scroll-to-block', __auto_gl_jade_scroll_to_block_OnScrollToBlock)"),
        "window listener cleaned up:\n{sfc}"
    );
}

/// jade batch 3 (CreatePagePrompt): `onclick.self:` → `@click.self`
/// (modifier passes through verbatim).
#[test]
fn cap_click_self_modifier() {
    let sfc = gen_sfc(
        r#"
widget W {
    msg Msg { Cancel }
    model { var x int = 0 }
    view {
        col {
            onclick.self: .Cancel
        }
    }
    on {
        .Cancel -> { .x = 0 }
    }
}
"#,
    );
    assert!(sfc.contains("@click.self=\"Cancel\""), ".self modifier:\n{sfc}");
}

/// jade gap 37 / batch 4 (CommandPalette/QuickSwitcher): a map literal is
/// the standard way to pass multiple args to a view event handler —
/// `oninput: .H({ entry: entry, evt: $event })` passes through verbatim,
/// loop variable included.
#[test]
fn cap_map_literal_event_arg() {
    let sfc = gen_sfc(
        r#"
widget W {
    msg Msg { H }
    model { var items list = [] }
    view {
        col {
            for entry in .items {
                input {
                    oninput: .H({ entry: entry, evt: $event })
                }
            }
        }
    }
    on {
        .H(args) -> { }
    }
}
"#,
    );
    assert!(
        sfc.contains("@input=\"H({ entry: entry, evt: $event })\""),
        "map literal event arg verbatim:\n{sfc}"
    );
    assert!(sfc.contains("function H(args: any)"), "handler param:\n{sfc}");
}

/// jade batch 3 (CreatePagePrompt/LeftSidebar/TabStrip): a widget whose
/// view root is a conditional renders a `<template v-if>` fragment root.
#[test]
fn cap_root_if_fragment() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var open bool = false }
    view {
        if .open {
            div { text "hi" }
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("<template v-if=\"open\">"),
        "root-level v-if fragment:\n{sfc}"
    );
}

/// jade 5.3b (GraphPage): optional widget props via default-value syntax —
/// `loading: bool = false` → `loading?: boolean`,
/// `centerPath: ?str = None` → `centerPath?: string | null`.
#[test]
fn cap_optional_props_with_defaults() {
    let sfc = gen_sfc(
        r#"
widget W(loading: bool = false, centerPath: ?str = None) {
    view { col { text "hi" } }
}
"#,
    );
    assert!(sfc.contains("loading?: boolean"), "optional bool prop:\n{sfc}");
    assert!(
        sfc.contains("centerPath?: string | null"),
        "optional nullable str prop:\n{sfc}"
    );
}

/// jade 5.3b (GraphView): quoted LOWERCASE emit variant — child side.
/// `msg Msg { "open"(str) }` + empty handler → `defineEmits<{ open:
/// [string] }>` and the auto-appended `emit('open', p)`.
#[test]
fn cap_quoted_lowercase_emit_child() {
    let sfc = gen_sfc(
        r#"
widget Child {
    msg Msg { "open"(str) }
    view {
        col {
            button "x" { onclick: ."open" }
        }
    }
    on {
        ."open"(p) -> { }
    }
}
"#,
    );
    assert!(sfc.contains("open: [string]"), "lowercase emit signature:\n{sfc}");
    assert!(sfc.contains("emit('open', p)"), "verbatim lowercase emit:\n{sfc}");
}

/// jade 5.3b: parent side of the lowercase emit contract —
/// `Child { onopen: .OpenPage }` → `@open="OpenPage"`.
#[test]
fn cap_quoted_lowercase_emit_parent_wiring() {
    let sfc = gen_sfc(
        r#"
widget App {
    msg Msg { OpenPage }
    model { var x int = 0 }
    view {
        col {
            Child { onopen: .OpenPage }
        }
    }
    on {
        .OpenPage(p) -> { .x = 1 }
    }
}
"#,
    );
    assert!(
        sfc.contains("@open=\"OpenPage\""),
        "parent listens to lowercase emit:\n{sfc}"
    );
}

/// plan 013 Phase 2 (shell elimination): quoted msg variants are
/// CONTRACTUAL emit names — they must be declared in defineEmits even when
/// no template binding references a handler (bridge/extension code emits
/// them via getCurrentInstance().emit; undeclared listeners would fall
/// through as native DOM listeners on the root element).
#[test]
fn cap_quoted_emit_declared_without_view_reference() {
    let sfc = gen_sfc(
        r#"
widget Shell {
    msg Msg { "update"(str), "blur", Internal }
    view {
        col {
            button "x" { onclick: .Internal }
        }
    }
    on {
        .Internal -> { }
    }
}
"#,
    );
    assert!(
        sfc.contains("update: [string]"),
        "handler-less quoted emit declared:\n{sfc}"
    );
    assert!(sfc.contains("blur: []"), "unit quoted emit declared:\n{sfc}");
}

/// plan 013 Phase 2: the computed-payload relay — a view-bound (or
/// exposed) handler computes the payload, then self-calls the quoted
/// handler which trailing-emits it (`emit('save', md)`), even though the
/// quoted handler itself is never template-bound.
#[test]
fn cap_quoted_emit_self_called_relay() {
    let sfc = gen_sfc(
        r#"
widget Shell {
    msg Msg { "save"(str) }
    view {
        col {
            button "save" { onclick: .HandleSave }
        }
    }
    on {
        .HandleSave -> { ."save"("md") }
        ."save"(md) -> { }
    }
}
"#,
    );
    assert!(
        sfc.contains("save: [string]"),
        "self-called quoted emit declared:\n{sfc}"
    );
    assert!(
        sfc.contains("emit('save', md)"),
        "self-called quoted handler trailing-emits the payload:\n{sfc}"
    );
    assert!(
        sfc.contains("save('md')"),
        "the relay call passes the computed payload:\n{sfc}"
    );
}

/// plan 015 (probe): a quoted msg variant whose name is NOT a valid JS
/// identifier (`"update:open"`), self-called from another on-block handler
/// (`let _ = ."update:open"(false)`), must (a) be emitted as a function
/// under its sanitized name (`update_open` — the used_handlers filter
/// previously dropped it because the raw quoted name never matched), and
/// (b) be called under that same sanitized name (the ts_adapter self-call
/// path previously emitted the raw `update:open(false)` — invalid JS).
/// The colon-containing name stays verbatim only in defineEmits/emit().
#[test]
fn cap_quoted_emit_self_called_non_ident_name() {
    let sfc = gen_sfc(
        r#"
widget Kid {
    msg Msg { "update:open"(bool), Close }
    view {
        col {
            button "x" { onclick: .Close }
        }
    }
    on {
        .Close -> { let _ = ."update:open"(false) }
        ."update:open"(v) -> { }
    }
}
"#,
    );
    assert!(
        sfc.contains("function update_open("),
        "self-called quoted handler emitted under sanitized name:\n{sfc}"
    );
    assert!(
        sfc.contains("update_open(false)"),
        "call site uses the sanitized name:\n{sfc}"
    );
    assert!(
        !sfc.contains("update:open(false)"),
        "raw quoted name must not leak into JS code:\n{sfc}"
    );
    assert!(
        sfc.contains("'update:open': [boolean]"),
        "quoted emit name stays verbatim in defineEmits:\n{sfc}"
    );
}

/// plan 013 follow-up (probe 09): the parser drops explicit parentheses
/// when building the `Bina` tree, so emitters must re-derive them from
/// precedence/associativity — `(a+b)*c` used to silently come out as
/// `a + b * c`, `(x||y)&&z` as `x || y && z` (both WRONG).
#[test]
fn cap_bina_parens_restored_in_computed() {
    let sfc = gen_sfc(
        r#"
widget P {
    model {
        var a int = 1
        var b int = 2
        var c int = 3
        var x bool = true
        var y bool = false
        var z bool = false
    }
    computed {
        arith => (.a + .b) * .c
        logic => (.x || .y) && .z
        plain => .a + .b * .c
        neg => !(.x && .y)
    }
    view {
        col {
            text f"${.arith} ${.logic} ${.plain} ${.neg}"
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("(a.value + b.value) * c.value"),
        "lower-precedence left child re-parenthesized:\n{sfc}"
    );
    assert!(
        sfc.contains("(x.value || y.value) && z.value"),
        "|| under && re-parenthesized:\n{sfc}"
    );
    assert!(
        sfc.contains("a.value + b.value * c.value"),
        "natural precedence stays paren-free:\n{sfc}"
    );
    assert!(
        sfc.contains("!(x.value && y.value)"),
        "unary ! keeps the Bina operand grouped:\n{sfc}"
    );
}

/// Right-child associativity: all DSL binops are left-associative, so an
/// equal-precedence right child must be re-parenthesized —
/// `a - (b - c)` ≠ `a - b - c`.
#[test]
fn cap_bina_parens_right_child_regroup() {
    let sfc = gen_sfc(
        r#"
widget P {
    model {
        var a int = 8
        var b int = 5
        var c int = 2
    }
    computed {
        sub => .a - (.b - .c)
        add => .a + (.b - .c)
    }
    view {
        col {
            text f"${.sub} ${.add}"
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("a.value - (b.value - c.value)"),
        "right child of `-` re-parenthesized:\n{sfc}"
    );
    assert!(
        sfc.contains("a.value + b.value - c.value"),
        "`a + (b - c)` safely flattens (left-assoc equivalent):\n{sfc}"
    );
}

/// The same regrouping applies in handler bodies (ts_adapter) and in view
/// bindings (expr_to_vue_bound_value).
#[test]
fn cap_bina_parens_in_handler_and_view_binding() {
    let sfc = gen_sfc(
        r#"
widget P {
    model {
        var a int = 1
        var b int = 2
        var c int = 3
        var r int = 0
    }
    view {
        col {
            div { title: (.a + .b) * .c }
            button "go" { onclick: .Go }
        }
    }
    on {
        .Go -> { .r = (.a + .b) * .c }
    }
}
"#,
    );
    assert!(
        sfc.contains(".r.value = (a.value + b.value) * c.value")
            || sfc.contains("r.value = (a.value + b.value) * c.value"),
        "handler body keeps the grouping:\n{sfc}"
    );
    assert!(
        sfc.contains(":title=\"(a + b) * c\""),
        "view binding keeps the grouping:\n{sfc}"
    );
}

/// plan 013 follow-up (probe 14): `||`/`&&` yield one of the OPERANDS in
/// JS, so `.a || "b"` must infer `computed<string>`, not
/// `computed<boolean>` (which forced the strOr/orNull extension helpers).
#[test]
fn cap_logical_computed_infers_operand_type() {
    let sfc = gen_sfc(
        r#"
widget P {
    model {
        var a str = ""
        var x bool = true
        var y bool = false
    }
    computed {
        or_str => .a || "b"
        or_bool => .x || .y
    }
    view {
        col {
            text f"${.or_str} ${.or_bool}"
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("const or_str = computed<string>(() => a.value || 'b')"),
        "|| over strings infers computed<string>:\n{sfc}"
    );
    assert!(
        sfc.contains("const or_bool = computed<boolean>(() => x.value || y.value)"),
        "|| over booleans stays computed<boolean>:\n{sfc}"
    );
}

/// plan 013 follow-up: a binop/unary method-call RECEIVER keeps its
/// parens too — `(a + b).toUpperCase()`, not `a + b.toUpperCase()`
/// (the AST drops explicit parens; receivers are Dot/Call children, a
/// different path from Bina operands).
#[test]
fn cap_bina_parens_on_call_receiver() {
    let sfc = gen_sfc(
        r#"
widget P {
    model {
        var first str = "a"
        var last str = "b"
    }
    computed {
        shout => (.first + " " + .last).to_upper()
    }
    view {
        col {
            text f"${.shout}"
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("(first.value + ' ' + last.value).toUpperCase()"),
        "binop receiver re-parenthesized:\n{sfc}"
    );
}

/// editor README note 3 / jade batch 2 (CodeBlockMenu, jade first use):
/// v-model FOLD — `value: .query` + `oninput: .QueryInput($event)` on a
/// state field collapses to `v-model="query"`; the handler function is
/// still emitted AND still wired via `@input`.
///
/// Plan 399 Phase 12 (9e9faaf2) pinned this contract: v-model only owns the
/// two-way value binding; arbitrary handler side effects (e.g. typing-signal
/// InputChanged) still need the explicit `@input` listener. Vue 3 merges
/// `v-model` + `@input` into one onInput handler array, so both run — no
/// double value binding. (The pre-Phase-12 behavior silently dropped @input
/// and side-effect handlers never fired.)
#[test]
fn cap_vmodel_fold() {
    let sfc = gen_sfc(
        r#"
widget W {
    msg Msg { QueryInput }
    model { var query str = "" }
    view {
        col {
            input { value: .query, oninput: .QueryInput($event) }
        }
    }
    on {
        .QueryInput(v) -> { .query = "" }
    }
}
"#,
    );
    assert!(sfc.contains("v-model=\"query\""), "value+oninput folds to v-model:\n{sfc}");
    assert!(
        sfc.contains("function QueryInput(v: any)"),
        "handler still emitted:\n{sfc}"
    );
    assert!(
        sfc.contains("@input=\"QueryInput($event)\""),
        "handler side effects still wired via @input alongside v-model:\n{sfc}"
    );
}

/// jade 5.3d (WhiteboardPage): a widget model `var doc map = {}` keeps
/// its map-literal initial value — `ref<any>({})` (contrast store-side,
/// which inits ref(null)).
#[test]
fn cap_widget_map_model_init() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var doc map = {} }
    view { col { text "hi" } }
}
"#,
    );
    assert!(sfc.contains("const doc = ref<any>({})"), "map init preserved:\n{sfc}");
}

/// jade gap 33 / batch 3: view-if conditions support comparison and
/// negation expressions directly (`if .x != ""`, `if !.flag`) — no
/// pre-declared computed needed.
#[test]
fn cap_view_if_comparison_and_negation() {
    let sfc = gen_sfc(
        r#"
widget W {
    model {
        var x str = ""
        var open bool = false
    }
    view {
        col {
            if .x != "" { text "nonempty" }
            if !.open { text "closed" }
        }
    }
}
"#,
    );
    assert!(sfc.contains("v-if=\"x != ''\""), "comparison condition:\n{sfc}");
    // NOTE: codegen emits `! open` (with a space) — locked as-is.
    assert!(sfc.contains("v-if=\"! open\""), "negation condition:\n{sfc}");
}

/// jade 5.3a (FileTreeNode NodeIcon): dyn components accept dot-chain
/// expression props — `dyn (.NodeIcon) { is_dir: .node.is_dir }` →
/// `:is_dir="node.is_dir"`.
#[test]
fn cap_dyn_component_dot_chain_props() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var node map = {} }
    view {
        col {
            dyn (.NodeIcon) { is_dir: .node.is_dir, expanded: .node.expanded }
        }
    }
}
"#,
    );
    assert!(sfc.contains(":is=\"(NodeIcon) as any\""), "dyn :is:\n{sfc}");
    assert!(sfc.contains(":is_dir=\"node.is_dir\""), "dot-chain prop:\n{sfc}");
    assert!(sfc.contains(":expanded=\"node.expanded\""), "second dot-chain prop:\n{sfc}");
}

/// jade 5.3a (LeftSidebar): style_obj values mix state/computed refs and
/// string literals — `style_obj: { marginLeft: .indent_left, marginRight:
/// "6px" }` → `:style="({ marginLeft: indent_left, marginRight: '6px' })"`.
#[test]
fn cap_style_obj_mixed_values() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var indent_left str = "4px" }
    view {
        col {
            style_obj: { marginLeft: .indent_left, marginRight: "6px" }
        }
    }
}
"#,
    );
    assert!(
        sfc.contains(":style=\"({ marginLeft: indent_left, marginRight: '6px' } as any)\""),
        "mixed style_obj values:\n{sfc}"
    );
}

/// jade batch 3 (WorkspaceOpener): `disabled: .busy` on a state field →
/// `:disabled="busy"` boolean attr binding.
#[test]
fn cap_disabled_state_binding() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var busy bool = false }
    view {
        col {
            button "go" { disabled: .busy, class: "x" }
        }
    }
}
"#,
    );
    assert!(sfc.contains(":disabled=\"busy\""), "disabled binding:\n{sfc}");
}

// ============================================================================
// Plan 012 P2 — reserved-word contextualization (jade gaps 18/27/29/34/43/53).
// `link`/`task` are element/keyword tokens but legitimate identifier names;
// `type:`/`to:` are keyword tokens but legitimate prop keys; `map` is a
// built-in prop type.
// ============================================================================

/// jade gap 18/29: a view for-loop variable may be named `link` or `task`.
/// Previously `text link.title` misparsed as a router-link element + garbage
/// sibling nodes.
#[test]
fn cap_loop_var_named_link_and_task() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var items list = [] }
    view {
        col {
            for link in .items {
                div { text link.title }
            }
            for task in .items {
                div { text task.name }
            }
        }
    }
}
"#,
    );
    assert!(sfc.contains(r#"v-for="link in items""#), "link loop:\n{sfc}");
    assert!(sfc.contains("{{ link.title }}"), "link field access:\n{sfc}");
    assert!(sfc.contains(r#"v-for="task in items""#), "task loop:\n{sfc}");
    assert!(sfc.contains("{{ task.name }}"), "task field access:\n{sfc}");
    assert!(!sfc.contains("router-link"), "no router-link garbage:\n{sfc}");
}

/// jade gap 18 (handler side): a handler local may be named `link`/`task`
/// and used in conditions and field access.
#[test]
fn cap_handler_local_named_link() {
    let sfc = gen_sfc(
        r#"
widget W {
    msg Msg { Go }
    model {
        var items list = []
        var n int = 0
    }
    view {
        col {
            button "go" { onclick: .Go }
        }
    }
    on {
        .Go -> {
            var link = .items.find(l => l.id == 1)
            if link != null { .n = link.id }
        }
    }
}
"#,
    );
    assert!(sfc.contains("function Go("), "handler emitted:\n{sfc}");
    assert!(sfc.contains("link"), "link local survives:\n{sfc}");
}

/// jade gap 53: keyword-token prop keys in BRACE form — `button { type:
/// "button" }` now emits a real `:type` binding instead of garbage
/// `<div>button</div>` child nodes (previously only the paren form worked).
#[test]
fn cap_keyword_prop_keys_brace_form() {
    let sfc = gen_sfc(
        r#"
widget W {
    view {
        col {
            button {
                type: "button"
                class: "x"
                text "hi"
            }
        }
    }
}
"#,
    );
    assert!(sfc.contains(r#":type="'button'"#), "type prop emitted:\n{sfc}");
    assert!(!sfc.contains("<div>button</div>"), "no garbage child:\n{sfc}");
}

/// jade gap 27: `to:` on a dyn block is a normal prop — `:to="'body'"`, no
/// garbage `<div>body</div>` child nodes.
#[test]
fn cap_dyn_keyword_to_prop() {
    let sfc = gen_sfc(
        r#"
widget W {
    view {
        col {
            dyn (.Teleport) {
                to: "body"
                text "overlay"
            }
        }
    }
}
"#,
    );
    assert!(sfc.contains(r#":to="'body'"#), "to prop emitted:\n{sfc}");
    assert!(!sfc.contains("<div>body</div>"), "no garbage child:\n{sfc}");
}

/// jade gap 34: `path` is NOT actually reserved for computed/locals (only
/// SVG-path element position) — a computed named `path` compiles. Lock the
/// verified-good behavior.
#[test]
fn cap_computed_named_path() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var active_path str = "" }
    computed {
        path => .active_path
    }
    view { col { text .path } }
}
"#,
    );
    assert!(sfc.contains("{{ path }}"), "computed path in template:\n{sfc}");
}

/// jade gap 43: `map` is a legal widget prop type — emits `any` and, unlike
/// before, no broken `import type { map } from '@/lib/api'`.
#[test]
fn cap_map_prop_type_is_any() {
    let sfc = gen_sfc(
        r#"
widget W(settings: map) {
    view { col { text "x" } }
}
"#,
    );
    assert!(sfc.contains("settings: any"), "map prop → any:\n{sfc}");
    assert!(!sfc.contains("import type { map }"), "no broken import:\n{sfc}");
}

/// Regression lock: the router-link VIEW element (`link (to: ...)`) is
/// unaffected by `link` becoming a contextual identifier.
#[test]
fn cap_router_link_element_unchanged() {
    let sfc = gen_sfc(
        r#"
widget W {
    view {
        col {
            link (to: "/home") { text "Home" }
        }
    }
}
"#,
    );
    assert!(sfc.contains("router-link"), "router-link still emitted:\n{sfc}");
    assert!(sfc.contains(r#"to="/home""#), "router-link target:\n{sfc}");
}

// ============================================================================
// Plan 015 P1#8 — R016 hard-keyword collision (view AST validator)
// ============================================================================

/// Parse a widget source and extract its aura widget (real pipeline), then
/// run the R016 keyword-collision rule on the view tree.
fn r016_warnings(src: &str) -> Vec<auto_lang::ui_gen::validators::ValidationWarning> {
    let session = CompilerSession::ui();
    let mut parser = Parser::from(src).with_session(session);
    let ast = parser.parse().expect("widget source must parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = extract_widget_from_decl(decl).expect("extract widget");
    auto_lang::ui_gen::validators::r016_keyword_collision(&widget.view_tree, &widget.name)
}

/// jade gap 18/29: an element named `view` collides with the hard keyword —
/// it parses silently and degrades to a `<div>`. R016 must flag it.
#[test]
fn cap_keyword_element_view_flagged() {
    let ws = r016_warnings(
        r#"
widget W {
    view {
        col {
            view { text "inner" }
        }
    }
}
"#,
    );
    assert!(
        ws.iter().any(|w| w.rule == "R016" && w.message.contains("view")),
        "view element must be flagged: {ws:?}"
    );
}

/// jade gap 18/29 follow-up: an element named `task` collides the same way.
#[test]
fn cap_keyword_element_task_flagged() {
    let ws = r016_warnings(
        r#"
widget W {
    view {
        col {
            task { text "x" }
        }
    }
}
"#,
    );
    assert!(
        ws.iter().any(|w| w.rule == "R016" && w.message.contains("task")),
        "task element must be flagged: {ws:?}"
    );
}

/// jade gap 34/53: `link` without `to:` silently becomes
/// `<router-link to="">`. R016 must flag it.
#[test]
fn cap_keyword_link_without_to_flagged() {
    let ws = r016_warnings(
        r#"
widget W {
    view {
        col {
            link { text "click" }
        }
    }
}
"#,
    );
    assert!(
        ws.iter().any(|w| w.rule == "R016" && w.message.contains("router-link")),
        "link without to: must be flagged: {ws:?}"
    );
}

/// The legal router-link form (`link (to: ...)`) must NOT be flagged.
#[test]
fn cap_keyword_link_with_to_clean() {
    let ws = r016_warnings(
        r#"
widget W {
    view {
        col {
            link (to: "/home") { text "Home" }
        }
    }
}
"#,
    );
    assert!(ws.is_empty(), "link (to:) must not warn: {ws:?}");
}

/// Probe-verified legal keyword-adjacent forms must NOT be flagged:
/// paren `type:` prop, block `type:` prop, block `as:`/`to:` props
/// (Plan 012 P2 contextualized them), and `path` as model/computed/loop
/// variable names.
#[test]
fn cap_keyword_legal_forms_clean() {
    let ws = r016_warnings(
        r#"
widget W {
    model { var path str = "/a" }
    computed { full_path => .path + "/b" }
    view {
        col {
            button "ok" (type: "button") { }
            input { type: "text" }
            div { as: "x", to: "/y" }
            text .path
        }
    }
}
"#,
    );
    assert!(ws.is_empty(), "legal keyword-adjacent forms must not warn: {ws:?}");
}

/// A model field named `view` referenced as `text .view` lexes `.view` into
/// the View keyword token, leaking a garbage `view` element node into the
/// view tree — caught by R016's view-element check.
#[test]
fn cap_model_field_named_view_ref_flagged() {
    let ws = r016_warnings(
        r#"
widget W {
    model { var view str = "a" }
    view {
        text .view
    }
}
"#,
    );
    assert!(
        ws.iter().any(|w| w.rule == "R016" && w.message.contains("view")),
        "leaked view element from .view ref must be flagged: {ws:?}"
    );
}

// ============================================================================
// Plan 028 — a2ts capability gaps for the Block migration (F1–F9).
// ============================================================================

/// F1 (plan 028 T1): dictionary literals with QUOTED string keys in
/// expression position (computed/fn bodies) + index reads by variable key.
/// `taskPlanStatusLabel`-style color/label tables are the first consumers.
#[test]
fn dict_literal() {
    let sfc = gen_sfc(
        r##"
widget W {
    model { var status str = "running" }
    computed {
        colors => { "running": "#f59e0b", "completed": "#10b981" }
        color => .colors[.status] ?? "#999999"
        fallback => .colors["failed"] ?? "#666666"
    }
    view { col { text .color } }
}
"##,
    );
    assert!(
        sfc.contains("'running': '#f59e0b'") && sfc.contains("'completed': '#10b981'"),
        "quoted-key dict literal survives to JS:\n{sfc}"
    );
    assert!(
        sfc.contains("computed<Record<string, string>>"),
        "uniform str→str dict infers Record type:\n{sfc}"
    );
    assert!(
        sfc.contains("colors[status]") || sfc.contains("colors.value[status.value]"),
        "index read by variable key:\n{sfc}"
    );
    assert!(
        sfc.contains("?? '#999999'"),
        "missing-key null-coalesce default:\n{sfc}"
    );
    assert!(
        sfc.contains("['failed']"),
        "string-literal key index read:\n{sfc}"
    );
    assert!(sfc.contains("{{ color }}"), "computed color in template:\n{sfc}");
}

/// F2 (plan 028 T2): view `if` conditions allow MULTI-ARG fn calls in
/// comparison/logic combinations — `if isLastMessage(.msgs, .m.id) { }`
/// previously forced a pre-computed bypass (chats_view gap G2).
#[test]
fn if_multiarg_call() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var msgs list = [] }
    view {
        col {
            for m in .msgs {
                if isLastMessage(.msgs, m.id) { text "last" }
                if isLastMessage(.msgs, m.id) && m.role == "assistant" { text "last-assistant" }
                if msgTimeLabel(m.ts) != "" { text "timed" }
            }
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("v-if=\"isLastMessage(msgs, m.id)\""),
        "bare multi-arg fn condition:\n{sfc}"
    );
    assert!(
        sfc.contains("v-if=\"isLastMessage(msgs, m.id) && m.role == 'assistant'\""),
        "multi-arg fn in logic combination:\n{sfc}"
    );
    assert!(
        sfc.contains("v-if=\"msgTimeLabel(m.ts) != ''\""),
        "multi-arg fn in comparison:\n{sfc}"
    );
}

// ============================================================================
// Plan 028 F3 — host API bridge: JSON / Date.format / Math / str methods.
// The Auto side names an API; the Vue backend maps it to native JS.
// ============================================================================

/// F3 (plan 028 T3): `JSON.parse` / `JSON.stringify` in expressions.
#[test]
fn host_api_json() {
    let sfc = gen_sfc(
        r#"
widget W {
    model {
        var raw str = "{\"q\": 1}"
        var v list = []
    }
    computed {
        parsed => JSON.parse(.raw)
        out => JSON.stringify(.v)
    }
    view { col { text .out } }
}
"#,
    );
    assert!(
        sfc.contains("JSON.parse(") && sfc.contains("raw.value"),
        "JSON.parse maps to native:\n{sfc}"
    );
    assert!(
        sfc.contains("JSON.stringify("),
        "JSON.stringify maps to native:\n{sfc}"
    );
}

/// F3 (plan 028 T3): `Date.format(ts, "HH:mm")` — a narrow date API instead
/// of exposing the whole Date surface. Maps to toLocaleTimeString.
#[test]
fn host_api_date() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var ts int = 1720000000 }
    computed { time => Date.format(.ts, "HH:mm") }
    view { col { text .time } }
}
"#,
    );
    assert!(
        sfc.contains("toLocaleTimeString"),
        "Date.format wraps toLocaleTimeString:\n{sfc}"
    );
    assert!(
        sfc.contains("hour: '2-digit'") && sfc.contains("minute: '2-digit'"),
        "HH:mm pattern maps to 2-digit hour/minute options:\n{sfc}"
    );
}

/// F3 (plan 028 T3): `Math.max` / `Math.min` pass through to native JS.
#[test]
fn host_api_math() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var a int = 1 }
    computed {
        span => Math.max(.a, 3)
        lo => Math.min(.a, 0)
    }
    view { col { text .span } }
}
"#,
    );
    assert!(sfc.contains("Math.max("), "Math.max native:\n{sfc}");
    assert!(sfc.contains("Math.min("), "Math.min native:\n{sfc}");
}

/// F3 (plan 028 T3): `str.char_code_at(i)` → `charCodeAt` (token estimate,
/// avatar hash). `char_at`/`slice` were already mapped — lock them together.
#[test]
fn host_api_str_char_code() {
    let sfc = gen_sfc(
        r#"
widget W {
    model {
        var text str = "hi"
        var i int = 0
    }
    computed {
        code => .text.char_code_at(.i)
        first => .text.char_at(0)
        rest => .text.slice(1, 2)
    }
    view { col { text .first } }
}
"#,
    );
    assert!(
        sfc.contains(".charCodeAt("),
        "char_code_at maps to charCodeAt:\n{sfc}"
    );
    assert!(sfc.contains(".charAt(0)"), "char_at stays mapped:\n{sfc}");
    assert!(
        sfc.contains(".substring(1, 2)"),
        "slice maps to substring:\n{sfc}"
    );
}

/// F3 (plan 028 T3): `str.split(sep)` → native split (CSV / mention parsing).
#[test]
fn host_api_str_split() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var csv str = "a,b" }
    computed { parts => .csv.split(",") }
    view { col { text .csv } }
}
"#,
    );
    assert!(
        sfc.contains(".split(',')"),
        "split maps to native split:\n{sfc}"
    );
}

// ============================================================================
// Plan 028 F4 — Regex subset (Rust syntax canonical, mechanical a2ts
// conversion): test/match/replace + named-group pattern rewrite.
// ============================================================================

#[test]
fn regex_subset() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var s str = "a1b2" }
    computed {
        hasDigit => Regex.test(.s, "[0-9]")
        digits => Regex.match(.s, "[0-9]+", "g")
        cleaned => Regex.replace(.s, "[0-9]", "", "g")
        sub1 => Regex.replace(.s, "a([0-9])", "$1", "")
        named => Regex.match(.s, "(?P<d>[0-9])")
    }
    view { col { text .s } }
}
"#,
    );
    assert!(
        sfc.contains("new RegExp('[0-9]').test("),
        "Regex.test maps to RegExp.test:\n{sfc}"
    );
    assert!(
        sfc.contains(".match(new RegExp('[0-9]+', 'g'))"),
        "Regex.match maps to String.match:\n{sfc}"
    );
    assert!(
        sfc.contains(".replace(new RegExp('[0-9]', 'g'), '')"),
        "Regex.replace maps to String.replace:\n{sfc}"
    );
    assert!(
        sfc.contains("'$1'"),
        "$1 backreference passes through:\n{sfc}"
    );
    assert!(
        sfc.contains("(?<d>[0-9])") && !sfc.contains("(?P<d>"),
        "Rust named group (?P<n>) rewritten to JS (?<n>):\n{sfc}"
    );
}

// ============================================================================
// Plan 028 F5/F6 — Index-position v-model + default-value props.
// ============================================================================

/// F5 (plan 028 T5): `value: .answers[q.id]` + `oninput:` folds to
/// `v-model="answers[q.id]"` — the questionnaire dynamic-key case, which
/// previously forced a controlled value+oninput pair (gap G8).
#[test]
fn index_vmodel() {
    let sfc = gen_sfc(
        r#"
widget W {
    msg Msg { Changed }
    model { var answers map = {} }
    view {
        col {
            for q in .questions {
                input { value: .answers[q.id], oninput: .Changed(q.id, $event) }
            }
        }
    }
    on {
        .Changed(qid, e) -> { }
    }
}
"#,
    );
    assert!(
        sfc.contains("v-model=\"answers[q.id]\""),
        "Index-position value folds to dynamic-key v-model:\n{sfc}"
    );
    assert!(
        !sfc.contains(":value=\"answers"),
        "redundant :value binding must be replaced by the fold:\n{sfc}"
    );
}

/// F6 (plan 028 T5): default-value props emit `withDefaults(defineProps, …)`
/// with real default values (not just `?:` optionality).
#[test]
fn default_props() {
    let sfc = gen_sfc(
        r#"
widget W(label: str = "hi", count: int = 3, loading: bool = false) {
    view { col { text .label } }
}
"#,
    );
    assert!(
        sfc.contains("withDefaults(defineProps<{"),
        "defaults wrap defineProps:\n{sfc}"
    );
    assert!(
        sfc.contains("}>(), {"),
        "defineProps<> must be CALLED (with parens after the type args); \
         the bare instantiation-expression form is not recognized as a \
         macro by vue-tsc and collapses props to unknown:\n{sfc}"
    );
    assert!(
        sfc.contains("label?: string") && sfc.contains("count?: number") && sfc.contains("loading?: boolean"),
        "defaulted props stay optional:\n{sfc}"
    );
    assert!(
        sfc.contains("label: 'hi'") && sfc.contains("count: 3") && sfc.contains("loading: false"),
        "default values carried into withDefaults object:\n{sfc}"
    );
}

/// Null defaults on non-nullable prop types (e.g. `[]str = null` → `any[]`)
/// are cast — withDefaults' InferDefault<T> rejects a bare `null` there.
#[test]
fn default_props_null_default_cast() {
    let sfc = gen_sfc(
        r#"
widget W(items: []str = null, cb: any = null) {
    view { col { text "x" } }
}
"#,
    );
    assert!(
        sfc.contains("items: (null as any),"),
        "null default on any[] prop is cast:\n{sfc}"
    );
    assert!(
        sfc.contains("cb: null,"),
        "null default on any-typed prop stays bare:\n{sfc}"
    );
}

// ============================================================================
// Plan 028 F7 — restricted v-html bridge (trusted HTML from an .at fn, e.g.
// renderMentions). Declared as the `html:` widget prop; a2ts → v-html,
// a2r degrades to a text node.
// ============================================================================

#[test]
fn html_binding() {
    let sfc = gen_sfc(
        r#"
widget W {
    model { var mentionHtml str = "<b>@dev</b>" }
    view {
        col {
            span { html: .mentionHtml }
            span { html: renderMentions(.mentionHtml) }
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("v-html=\"mentionHtml\""),
        "state-bound html prop emits v-html:\n{sfc}"
    );
    assert!(
        sfc.contains("v-html=\"renderMentions(mentionHtml)\""),
        "fn-call html prop emits v-html:\n{sfc}"
    );
    assert!(
        !sfc.contains(" html=\"") && !sfc.contains("<span html="),
        "no plain html= attribute may leak:\n{sfc}"
    );
}

// ============================================================================
// Plan 028 F8/F9 — platform stream protocol: `on stream sse(url[, "event"])`
// store subscriptions. The platform layer pre-parses SSE data (已决③); the
// handler body branches on `.ev.type` directly.
// ============================================================================

/// F9 (plan 028 T7): default-message SSE subscription — EventSource +
/// onmessage dispatching the pre-parsed event into the handler.
#[test]
fn store_on_stream() {
    let code = gen_store(
        r#"
store ChatStore {
    model { var draft str = "" }
    on {
        stream sse("/api/chats/session/s1/stream") -> {
            if ev.type == "delta" { .draft = .draft + ev.text }
        }
    }
}
"#,
    );
    assert!(
        code.contains("new EventSource('/api/chats/session/s1/stream')"),
        "EventSource at the declared url:\n{code}"
    );
    assert!(
        code.contains("es.onmessage") && code.contains("JSON.parse(ev.data)"),
        "default events dispatch pre-parsed payload:\n{code}"
    );
    assert!(
        code.contains("(ev: any)"),
        "handler fn receives the parsed event as `ev`:\n{code}"
    );
    assert!(
        code.contains("ev.type == 'delta'"),
        "body branches on .ev.type:\n{code}"
    );
    assert!(
        code.contains("let __streamConnected_api_chats_session_s1_stream = false;"),
        "per-url connection guard:\n{code}"
    );
}

/// F8 (plan 028 T7): named-event filter — relay streams listen on a named SSE
/// event (`run_event`) via addEventListener instead of onmessage.
#[test]
fn platform_sse() {
    let code = gen_store(
        r#"
store RelayStore {
    model { var status str = "" }
    on {
        stream sse("/api/relay/runs/r1/events", "run_event") -> {
            if ev.type == "status" { .status = ev.status }
        }
    }
}
"#,
    );
    assert!(
        code.contains("addEventListener('run_event'"),
        "named-event subscription uses addEventListener:\n{code}"
    );
    assert!(
        !code.contains("es.onmessage"),
        "no default onmessage when an event filter is declared:\n{code}"
    );
    assert!(
        code.contains("JSON.parse((ev as MessageEvent).data)"),
        "named events parse MessageEvent data:\n{code}"
    );
}

/// F8 (plan 028 T7): platform HTTP protocol — `Http.get` / `Http.post` map to
/// awaited fetch + .json() in store handler bodies (relay loadRun etc.).
#[test]
fn platform_http() {
    let code = gen_store(
        r#"
store RelayStore {
    model { var run obj = {} }
    msg Msg { Load }
    on {
        .Load -> {
            let r = Http.get("/api/relay/runs/r1");
            .run = r
        }
    }
}
"#,
    );
    assert!(
        code.contains("(await fetch('/api/relay/runs/r1')).json()"),
        "Http.get maps to awaited fetch:\n{code}"
    );
    assert!(
        code.contains("const Load = async"),
        "await in body makes the action async:\n{code}"
    );
}

/// Plan 014: pac.at `default_classes: off` — VueGenerator skips the doc-theme
/// default Tailwind classes for every non-layout-primitive tag, while layout
/// primitives (row/col/...) keep their structural classes. Default stays on.
///
/// The authoritative pac.at plumbing lives in ComponentGenOptions.default_classes
/// (auto-man threads it from pac.at); this locks the generator-level gate.
fn gen_sfc_with_default_classes(src: &str, default_classes: bool) -> String {
    let session = CompilerSession::ui();
    let mut parser = Parser::from(src).with_session(session);
    let ast = parser.parse().expect("widget source must parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = extract_widget_from_decl(decl).expect("extract widget");
    VueGenerator::new()
        .with_default_classes(default_classes)
        .generate(&widget)
        .expect("generate SFC")
}

const DEFAULT_CLASSES_SRC: &str = r#"
widget App {
    view {
        col {
            h1 "Title"
            text "body copy"
            button "save"
        }
    }
}
"#;

#[test]
fn cap_default_classes_on_keeps_doc_theme_defaults() {
    let sfc = gen_sfc_with_default_classes(DEFAULT_CLASSES_SRC, true);
    assert!(
        sfc.contains("flex flex-col"),
        "default on: col keeps layout classes:\n{sfc}"
    );
    assert!(
        sfc.contains("text-4xl font-bold"),
        "default on: h1 keeps doc-theme defaults:\n{sfc}"
    );
    assert!(
        sfc.contains("text-muted-foreground leading-7"),
        "default on: text keeps doc-theme defaults:\n{sfc}"
    );
    assert!(
        sfc.contains("px-4 py-2 rounded"),
        "default on: button keeps doc-theme defaults:\n{sfc}"
    );
}

#[test]
fn cap_default_classes_off_skips_defaults_but_keeps_layout() {
    let sfc = gen_sfc_with_default_classes(DEFAULT_CLASSES_SRC, false);
    assert!(
        sfc.contains("flex flex-col"),
        "default_classes off: col (layout primitive) must keep flex classes:\n{sfc}"
    );
    assert!(
        !sfc.contains("text-4xl"),
        "default_classes off: h1 must not get doc-theme defaults:\n{sfc}"
    );
    assert!(
        !sfc.contains("leading-7"),
        "default_classes off: text must not get doc-theme defaults:\n{sfc}"
    );
    assert!(
        !sfc.contains("px-4 py-2 rounded"),
        "default_classes off: button must not get doc-theme defaults:\n{sfc}"
    );
}

/// Plan 014: division/mod lowering — `/` and `%` lower to `Math.trunc(a/b)`
/// ONLY when both operands are proven int (int literals or int-typed
/// state/props). f64-typed and unknown-type operands keep native JS float
/// semantics. Regression lock: c2f57577 truncated f64 operands
/// (`Math.trunc(scrollTop / scrollHeight)`), silently zeroing ratio math in
/// the demo CustomScrollbar thumb (height/top computed as 0 → drag dead).
#[test]
fn cap_div_mod_trunc_only_for_proven_int() {
    let sfc = gen_sfc(
        r#"
widget P(ratio: f64, count: int) {
    model {
        var scroll_top f64 = 0
        var total f64 = 100
        var n int = 7
        var m int = 2
        var r f64 = 0
    }
    view {
        col {
            button "go" { onclick: .Go }
        }
    }
    on {
        .Go -> {
            .r = .scroll_top / .total
            .n = .n / .m
            .n = 7 / 2
            .n = .n % .m
            .r = .scroll_top / 2
            .r = .n / .ratio
        }
    }
}
"#,
    );
    // int state / int state → trunc
    assert!(
        sfc.contains("Math.trunc(n.value / m.value)"),
        "int / int lowers to Math.trunc:\n{sfc}"
    );
    // int literals → trunc
    assert!(
        sfc.contains("Math.trunc(7 / 2)"),
        "int literal / int literal lowers to Math.trunc:\n{sfc}"
    );
    // int % int → trunc (emitter quirk: js_op for Mod is " %" — no trailing space)
    assert!(
        sfc.contains("Math.trunc(n.value %m.value)"),
        "int % int lowers to Math.trunc:\n{sfc}"
    );
    // f64 state / f64 state → native float division, NO trunc
    assert!(
        sfc.contains("scroll_top.value / total.value"),
        "f64 / f64 keeps float division:\n{sfc}"
    );
    assert!(
        !sfc.contains("Math.trunc(scroll_top.value / total.value)"),
        "f64 / f64 must not be truncated:\n{sfc}"
    );
    // f64 / int literal → float (one operand not proven int)
    assert!(
        sfc.contains("scroll_top.value / 2"),
        "f64 / int literal keeps float division:\n{sfc}"
    );
    assert!(
        !sfc.contains("Math.trunc(scroll_top.value / 2)"),
        "f64 / int literal must not be truncated:\n{sfc}"
    );
    // int / f64 prop → float
    assert!(
        sfc.contains("n.value / props.ratio") || sfc.contains("n.value / ratio"),
        "int / f64 prop keeps float division:\n{sfc}"
    );
    assert!(
        !sfc.contains("Math.trunc(n.value / props.ratio)"),
        "int / f64 prop must not be truncated:\n{sfc}"
    );
}

// ============================================================================
// Plan 421: code_editor vue contract — five props + oncursor/oncontextmenu
// events thread through to the scaffolded CodeEditor shell
// (auto-man src/components/CodeEditor.vue). Fragment assertions per the file
// header (SFC emission order is HashMap-unstable); shadcn mode matches the
// real scaffolded project (`render: "vue"` + shadcn).
// ============================================================================

/// Same as gen_sfc but in shadcn mode (code_editor arms only run there).
fn gen_sfc_shadcn(src: &str) -> String {
    let session = CompilerSession::ui();
    let mut parser = Parser::from(src).with_session(session);
    let ast = parser.parse().expect("widget source must parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = extract_widget_from_decl(decl).expect("extract widget");
    let mut gen = VueGenerator::new_shadcn();
    gen.generate(&widget).expect("generate SFC")
}

/// Plan 421: full props+events combination — the generated binding surface
/// matches what the scaffold component consumes (modelValue/lang/
/// line-numbers/wrap/vi/highlight-current-line/tab-width/font-size/search +
/// @cursor/@contextmenu with the position payload threaded into handlers
/// that declare params).
#[test]
fn cap_code_editor_props_and_position_events() {
    let sfc = gen_sfc_shadcn(
        r#"
widget EditorPage {
    model {
        var src str = "fn main() {}"
        var query str = "main"
    }
    view {
        col {
            code_editor (key: "ed", lang: "auto", tab_width: 2, font_size: 16, search: .query, wrap: true, vi: true) {
                content: .src
                oncursor: .CursorMoved
                oncontextmenu: .CtxMenu
            }
        }
    }
    on {
        .CursorMoved(line, col) -> { }
        .CtxMenu(x, y) -> { }
    }
}
"#,
    );

    // Component + import.
    assert!(sfc.contains("<CodeEditor"), "component tag:\n{sfc}");
    assert!(
        sfc.contains("import CodeEditor from '@/components/CodeEditor.vue'"),
        "default-style import:\n{sfc}"
    );
    // The five Plan 421 props.
    assert!(
        sfc.contains(":line-numbers=\"true\""),
        "line-numbers default true:\n{sfc}"
    );
    assert!(
        sfc.contains(":highlight-current-line=\"true\""),
        "highlight-current-line default true:\n{sfc}"
    );
    assert!(sfc.contains(":tab-width=\"2\""), "tab-width:\n{sfc}");
    assert!(sfc.contains(":font-size=\"16\""), "font-size:\n{sfc}");
    assert!(sfc.contains(":search=\"query\""), "search ref binding:\n{sfc}");
    // Pre-421 props still flow.
    assert!(sfc.contains("lang=\"auto\""), "lang:\n{sfc}");
    assert!(sfc.contains(":wrap=\"true\""), "wrap:\n{sfc}");
    assert!(sfc.contains(":vi=\"true\""), "vi (vue ignores, iced consumes):\n{sfc}");
    assert!(sfc.contains("data-editor-key=\"ed\""), "editor key:\n{sfc}");
    // Position events with payload args + matching script signatures.
    assert!(
        sfc.contains("@cursor=\"CursorMoved($event.line, $event.column)\""),
        "cursor payload args:\n{sfc}"
    );
    assert!(
        sfc.contains("@contextmenu=\"CtxMenu($event.x, $event.y)\""),
        "contextmenu payload args:\n{sfc}"
    );
    assert!(
        sfc.contains("function CursorMoved(line: any, col: any)"),
        "cursor handler signature:\n{sfc}"
    );
    assert!(
        sfc.contains("function CtxMenu(x: any, y: any)"),
        "contextmenu handler signature:\n{sfc}"
    );
}

// Plan 015 P1#4: handlers never referenced from the view (called only by
// extensions) are emitted as functions AND declared in defineEmits — the
// old used_handlers gate silently dropped them (inert-decoy workaround).
#[test]
fn cap_unused_handler_still_emitted() {
    let sfc = gen_sfc(
        r#"
widget Panel {
    state {
        var query str = ""
    }
    msg Msg { QueryInput(str), ExtOnly(str) }
    view {
        input (placeholder: "search") { oninput: .QueryInput($event) }
    }
    on {
        .QueryInput(v) -> { .query = v }
        .ExtOnly(v) -> { .query = v }
    }
}
"#,
    );
    assert!(
        sfc.contains("function ExtOnly"),
        "extension-only handler emitted as a function:\n{sfc}"
    );
    assert!(
        sfc.contains("ExtOnly: [string]"),
        "extension-only handler declared in defineEmits:\n{sfc}"
    );
}

// Plan 015 P1#5 (gap 37a): bool/float literals are valid view event args
// (".HoverChange(true)") — previously a loud parse error (int 1/0 stand-ins).
#[test]
fn cap_event_arg_bool_and_float_literals() {
    let sfc = gen_sfc(
        r#"
widget P {
    state { var hover bool = false
        var ratio float = 0.0 }
    msg Msg { HoverChange(bool), SetRatio(float) }
    view {
        col {
            div { onmouseenter: .HoverChange(true) }
            div { onchange: .SetRatio(0.5) }
        }
    }
    on {
        .HoverChange(v) -> { .hover = v }
        .SetRatio(v) -> { .ratio = v }
    }
}
"#,
    );
    assert!(
        sfc.contains("HoverChange(true)"),
        "bool literal passed through to the handler call:
{sfc}"
    );
    assert!(
        sfc.contains("SetRatio(0.5)"),
        "float literal passed through to the handler call:
{sfc}"
    );
}



// Plan 015 P1#6: multi-payload msg variants declare the FULL tuple in
// defineEmits — "(str, int)" used to declare only [string] while emit()
// forwarded both args.
#[test]
fn cap_emit_tuple_all_payload_types() {
    let sfc = gen_sfc(
        r#"
widget W {
    msg Msg { Resize(str, int) }
    view {
        col {
            div { onclick: .Resize("x", 3) }
        }
    }
    on {
        .Resize(name, n) -> { }
    }
}
"#,
    );
    assert!(
        sfc.contains("Resize: [string, number]"),
        "full payload tuple declared:\n{sfc}"
    );
}

// Plan 015 P1#7: `let x = nil` locals hold runtime objects (facade stores
// etc.); `.contains` on them must pass through unchanged instead of being
// mapped to the Auto collection semantics (.includes / splice).
#[test]
fn cap_null_init_local_contains_not_mapped() {
    let sfc = gen_sfc(
        r#"
widget W {
    use { composable: useTabsStore from "src/front/utils/x_ext.ts" }
    state { var hits Array<str> = [] }
    msg Msg { Go }
    view {
        col {
            button "b" { onclick: .Go }
        }
    }
    on {
        .Go -> {
            let ctx = nil
            ctx = grabContext()
            if ctx.contains("editor") {
                .hits = ctx.listPaths()
            }
        }
    }
}
"#,
    );
    assert!(
        sfc.contains("ctx.contains("),
        "null-init local's .contains passes through unmapped:\n{sfc}"
    );
    assert!(
        !sfc.contains("ctx.includes("),
        "no mis-mapped .includes on null-init local:\n{sfc}"
    );
}

/// Plan 443 file-level helper: compile a multi-widget .at source through the
/// real file driver (`generate_component_from_file`, temp file) and return
/// (widget name → SFC). Needed because bound-channel downgrading is decided
/// per FILE (two-pass), not per widget.
fn gen_sfc_file(src: &str) -> std::collections::HashMap<String, String> {
    let tmp = std::env::temp_dir().join(format!(
        "cap443_{}_{:x}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    let at_path = tmp.join("app.at");
    std::fs::write(&at_path, src).unwrap();
    let result = auto_lang::ui_gen::generate_component_from_file(
        &at_path,
        auto_lang::ui_gen::ComponentGenOptions::default(),
    )
    .expect("file must compile");
    let _ = std::fs::remove_dir_all(&tmp);
    result
        .all_widget_codes
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>()
}

/// Plan 443: a model channel some parent actually binds (`v-model:x` at the
/// call site) downgrades the child's var to defineModel; a sibling unbound
/// model var in the SAME child stays a plain ref. The bound channel's
/// object-literal default is factory-wrapped (props-default semantics —
/// a bare `{}` would be shared across child instances).
#[test]
fn cap_model_channel_bound_downgrades_child() {
    let codes = gen_sfc_file(
        r#"
widget App {
    model { var doc map = {} }
    view { col { Child(doc: .doc) text "hi" } }
}

widget Child {
    model {
        var doc map = {}
        var local int = 0
    }
    view { col { text "c" } }
}
"#,
    );
    let app = codes.get("App").expect("App SFC");
    let child = codes.get("Child").expect("Child SFC");

    assert!(
        app.contains("v-model:doc=\"doc\""),
        "parent folds the channel to v-model:\n{app}"
    );
    assert!(
        child.contains("const doc = defineModel<any>(\"doc\", { default: () => ({}) })"),
        "bound channel downgrades to defineModel with factory default:\n{child}"
    );
    assert!(
        child.contains("const local = ref<number>(0)"),
        "unbound sibling stays a plain ref:\n{child}"
    );
    assert!(
        !child.contains("defineModel<any>(\"local\""),
        "unbound sibling must NOT be a model:\n{child}"
    );
}

/// Plan 443: with NO parent binding, the same shape as above keeps the
/// pre-T4 ref semantics everywhere (deep reactivity — the jade whiteboard
/// `doc.value.shapes.push` regression).
#[test]
fn cap_model_channel_unbound_keeps_ref() {
    let codes = gen_sfc_file(
        r#"
widget App {
    model { var doc map = {} }
    view { col { Child(path: "x") text "hi" } }
}

widget Child {
    model {
        var doc map = {}
        var local int = 0
    }
    view { col { text "c" } }
}
"#,
    );
    let child = codes.get("Child").expect("Child SFC");
    assert!(
        child.contains("const doc = ref<any>({})"),
        "unbound channel stays ref (deep mutation reactivity):\n{child}"
    );
    assert!(
        child.contains("const local = ref<number>(0)"),
        "unbound sibling stays ref:\n{child}"
    );
    assert!(
        !child.contains("defineModel"),
        "no defineModel anywhere without a binding:\n{child}"
    );
}

// ---------------------------------------------------------------------------
// Plan 444 (ash-shell-057): vue codegen five-defect canaries
// ---------------------------------------------------------------------------

/// Plan 444 ①a/①b (convention path): a callback prop whose Pascal form IS a
/// child emit — parent binds `@Select`, child never declares `on_select` as
/// a data prop, and a bare `on_select(x)` invocation rewrites to the emit.
#[test]
fn cap_444_callback_channel_convention() {
    let codes = gen_sfc_file(
        r#"
widget App {
    msg Msg { Selected(int) }
    model { var active int = 0 }
    view { col { Counter(on_select: .Selected) text .active { } } }
    on { .Selected(i) -> { .active = i } }
}

widget Counter(on_select: msg) {
    msg Msg { Select(int), Bump(int) }
    model { var n int = 0 }
    view { col { button "increment" { onclick: .Bump(.n + 1) } text .n { } } }
    on { .Bump(next) -> { .n = next; on_select(next) } }
}
"#,
    );
    let child = codes.get("Counter").expect("Counter SFC");
    assert!(
        !child.contains("on_select:"),
        "msg callback prop must not be a data prop:\n{child}"
    );
    assert!(
        child.contains("emit('Select', "),
        "bare on_select(next) rewrites to the Pascal emit:\n{child}"
    );
    assert!(
        child.contains("Select: [number]"),
        "callback-contract emit declared with payload:\n{child}"
    );
    let app = codes.get("App").expect("App SFC");
    assert!(
        app.contains("@Select="),
        "parent binds the prop-derived event (convention path):\n{app}"
    );
}

/// Plan 444 ①a/①b (pass-through path): `on_delete: msg` with NO `Delete`
/// variant — the child re-emits its own `DeleteBlock` via the empty-handler
/// bridge, so the parent must bind `@DeleteBlock` and the child must not
/// require `on_delete` (ash-gui BlockItem shape).
#[test]
fn cap_444_callback_channel_pass_through() {
    let codes = gen_sfc_file(
        r#"
widget App {
    msg Msg { DeleteBlock(int) }
    model { var gone int = 0 }
    view { col { Item(on_delete: .DeleteBlock) } }
    on { .DeleteBlock(id) -> { .gone = id } }
}

widget Item(on_delete: msg) {
    msg Msg { DeleteBlock(int) }
    view { col { button "del" { onclick: .DeleteBlock(7) } } }
    on { .DeleteBlock(id) -> { } }
}
"#,
    );
    let child = codes.get("Item").expect("Item SFC");
    assert!(
        !child.contains("on_delete:"),
        "unmatched msg prop must not stay as a required data prop:\n{child}"
    );
    let app = codes.get("App").expect("App SFC");
    assert!(
        app.contains("@DeleteBlock="),
        "parent binds the child's actually-emitted msg (pass-through path):\n{app}"
    );
    assert!(
        !app.contains("@Delete="),
        "prop-derived @Delete would never fire:\n{app}"
    );
}

/// Plan 444 ③: multi-payload msg variants referenced from the template with
/// no on-block handler — the stub must become a synthesized-arg emit bridge
/// and defineEmits must carry the payload arity (was `Sort: []` + TS2554).
#[test]
fn cap_444_multi_param_emit_bridge() {
    let codes = gen_sfc_file(
        r#"
widget App {
    msg Msg { Sort(int, int), Filter(str) }
    view { col {
        button "s" { onclick: .Sort(1, 2) }
        input { oninput: .Filter("q") }
    } }
}
"#,
    );
    let app = codes.get("App").expect("App SFC");
    assert!(
        app.contains("Sort: [number, number]"),
        "defineEmits carries both payload types:\n{app}"
    );
    assert!(
        app.contains("Filter: [string]"),
        "defineEmits carries str payload:\n{app}"
    );
    assert!(
        app.contains("function Sort(arg0: any, arg1: any): void {\n  emit('Sort', arg0, arg1)"),
        "undefined handler becomes a synthesized-arg emit bridge:\n{app}"
    );
    assert!(
        app.contains("function Filter(arg0: any): void {\n  emit('Filter', arg0)"),
        "filter bridge:\n{app}"
    );
    assert!(
        !app.contains("// TODO: handler not defined in on-block"),
        "no arg-less TODO stubs for declared emits:\n{app}"
    );
}

/// Plan 444 ①c: `text "#" + j.id` parses as ONE text node — the receiver is
/// not dropped to a bare `{{ id }}` interpolation (App.vue TS2339).
#[test]
fn cap_444_text_concat_chain() {
    let codes = gen_sfc_file(
        r##"
widget App {
    model { var jobs []obj = [] }
    view { col {
        for j in .jobs {
            row { text "#" + j.id { style: "w-8" } }
        }
    } }
    on { .Init -> { } }
}
"##,
    );
    let app = codes.get("App").expect("App SFC");
    assert!(
        app.contains("{{ '#' + j.id }}"),
        "concat chain renders as one interpolation with quotes + operator:\n{app}"
    );
    assert!(
        !app.contains("{{ id }}"),
        "receiver must not be dropped:\n{app}"
    );
    assert!(
        !app.contains(">{{#}}<") && !app.contains("+\n"),
        "no stray '+' text node / split spans:\n{app}"
    );
}

/// Plan 444 ②: member access on a PascalCase variant field asserts non-null
/// (`cell.Tagged!.text`) — TS18049 in strict builds.
#[test]
fn cap_444_variant_field_nonnull_assert() {
    let codes = gen_sfc_file(
        r#"
widget App {
    model { var cell obj = {} }
    view { col { text "x" { } } }
    on {
        .Init -> {
            if .cell.Text != None {
                var a str = .cell.Text
            } else {
                var b str = .cell.Tagged.text
            }
        }
    }
}
"#,
    );
    let app = codes.get("App").expect("App SFC");
    assert!(
        app.contains("Tagged!.text"),
        "variant-field access gets a non-null assertion:\n{app}"
    );
}

/// Plan 444 ⑤: dynamic member read on a str-typed model ref goes through the
/// any channel (`(s.value as any).Failed`) — runtime is a bare string OR a
/// {"Failed": msg} object (TS2339 on `string`).
#[test]
fn cap_444_str_ref_dynamic_member_any_channel() {
    let codes = gen_sfc_file(
        r#"
widget App {
    model {
        var s str = ""
        var msg str = ""
    }
    view { col { text "x" { } } }
    on {
        .Init -> {
            if .s == "Cancelled" {
                .msg = ""
            } else {
                .msg = .s.Failed
            }
        }
    }
}
"#,
    );
    let app = codes.get("App").expect("App SFC");
    assert!(
        app.contains("(s.value as any).Failed"),
        "str-ref member read goes through the any channel:\n{app}"
    );
}

/// Plan 444 ④a: an api call (or complete()) inside an ELSE branch must still
/// mark the handler async / debounced — the walkers previously stopped at
/// branch bodies.
#[test]
fn cap_444_api_call_in_else_branch() {
    let codes = gen_sfc_file(
        r#"
use back.api: complete
widget App {
    msg Msg { Go }
    model { var q str = "" }
    view { col { button "go" { onclick: .Go } } }
    on {
        .Go -> {
            if .q == "" {
                .q = "x"
            } else {
                .q = complete(.q, 1)
            }
        }
    }
}
"#,
    );
    let app = codes.get("App").expect("App SFC");
    assert!(
        app.contains("import { complete } from '@/lib/api'"),
        "api fn used in an else branch still imports:\n{app}"
    );
    assert!(
        app.contains("setTimeout(async () => {"),
        "complete-handler in an else branch gets the debounced async wrapper:\n{app}"
    );
    // The debounced design keeps `function Go(): void` sync — the await lives
    // inside the async setTimeout callback, which is exactly what the output
    // above shows. No TS1304/TS1308 remains: `complete` is imported and every
    // `await` sits in an async context.
}


/// Plan 444 ④b: VM-only natives (fs.*/File.*) degrade to the throwing
/// __vmOnly stub — explicit error instead of silently illegal JS.
#[test]
fn cap_444_vm_only_native_stub() {
    let codes = gen_sfc_file(
        r#"
widget App {
    msg Msg { Go }
    view { col { button "go" { onclick: .Go } } }
    on {
        .Go -> {
            var raw str = fs.read_dir(".")
            if File.is_dir(raw) {
                var dummy int = 1
            }
        }
    }
}
"#,
    );
    let app = codes.get("App").expect("App SFC");
    assert!(
        app.contains("__vmOnly('fs.read_dir', '.')"),
        "fs.read_dir rewrites to the stub:\n{app}"
    );
    assert!(
        app.contains("__vmOnly('File.is_dir', raw)"),
        "File.is_dir rewrites to the stub:\n{app}"
    );
    assert!(
        app.contains("function __vmOnly(name: string, ...args: any[]): never"),
        "the stub helper is declared:\n{app}"
    );
}

#[test]
fn zz_plan444_debug_015_app() {
    let tmp = std::env::temp_dir().join("p444dbg");
    std::fs::create_dir_all(&tmp).unwrap();
    let src = std::fs::read_to_string("../../examples/ui/015-notes/src/front/app.at").unwrap();
    let at = tmp.join("app.at");
    std::fs::write(&at, src).unwrap();
    let result = auto_lang::ui_gen::generate_component_from_file(
        &at,
        auto_lang::ui_gen::ComponentGenOptions::default(),
    )
    .unwrap();
    for w in &result.validation_warnings {
        eprintln!("WARN: {} {} {:?}", w.rule, w.severity, w.message);
    }
    for (name, code) in &result.all_widget_codes {
        std::fs::write(tmp.join(format!("{}.vue", name)), code).unwrap();
    }
}

/// Plan 435 P6-1(D2):`api_functions_used` 是 HashSet,≥2 个 API 函数的 SFC
/// import 行若不排序则字节不确定(跨进程 RandomState)。修复后必须:多轮生成
/// 字节一致,且 import 列表按字母序发射。
#[test]
fn cap_435_p6_api_import_sorted_and_deterministic() {
    let src = r#"
widget App {
    msg Msg { Load }
    model { var out str = "" }
    view { col { button "load" { onclick: .Load } text .out } }
    on {
        .Load -> {
            .out = listusers()
            .out = createUser("a", "b")
        }
    }
}
"#;
    let mut outputs: Vec<String> = Vec::new();
    for _ in 0..8 {
        let codes = gen_sfc_file(src);
        outputs.push(codes.get("App").expect("App SFC").clone());
    }
    let first = &outputs[0];
    assert!(
        first.contains("import { createUser, listusers } from '@/lib/api'"),
        "api import 必须按字母序发射:\n{first}"
    );
    for (i, out) in outputs.iter().enumerate() {
        assert_eq!(out, first, "run #{i} 与首跑字节不一致(D2 确定性回归)");
    }
}
