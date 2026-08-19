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
