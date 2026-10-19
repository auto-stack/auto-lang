//! 纯 helper（无游标）：自 auto-lang Parser 迁出的 UI 文法静态工具
//! （PLAN-810 T-03/T-04 逐字迁移；端口映射见 contract-baseline §2）。

use crate::UiParseContext;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

/// Parse a string literal token, returning its (unquoted) text content.
pub(crate) fn parse_string_lit(cx: &mut dyn UiParseContext) -> AutoResult<AutoStr> {
    if cx.is_kind(TokenKind::Str) {
        let s = cx.cur_text();
        cx.advance();
        Ok(s.into())
    } else {
        Err(SyntaxError::UnexpectedToken {
            expected: "string literal".to_string(),
            found: cx.cur_text().to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into())
    }
}

/// Get the primary (major) property name for a view element tag.
/// This is used for the shorthand syntax: `tag "value"` → `tag (primary_prop: "value")`
///
/// Priority: id > name > text
/// - Elements with `id` prop → "id" is major
/// - Elements with `name` prop (no id) → "name" is major
/// - All other elements → "text" is major
///
/// Future: This should come from schema with `#[major]` attribute.
pub(crate) fn get_primary_prop(tag: &str) -> Option<&'static str> {
    // Elements with "id" as primary prop
    // These are typically components/containers that need identification
    match tag {
        "preview-card" | "PreviewCard" | "codeblock" | "Codeblock" | "tabs" | "Tabs" | "dialog"
        | "Dialog" | "sheet" | "Sheet" | "popover" | "Popover" | "dropdown-menu"
        | "DropdownMenu" | "context-menu" | "ContextMenu" | "alert-dialog" | "AlertDialog"
        | "drawer" | "Drawer" | "modal" | "Modal" | "tabs-trigger" | "TabsTrigger"
        | "tabs-content" | "TabsContent" => Some("id"),

        // Elements with "name" as primary prop (form inputs)
        "input" | "Input" | "select" | "Select" | "textarea" | "Textarea" | "checkbox"
        | "Checkbox" | "switch" | "Switch" | "radio-group" | "RadioGroup" | "slider" | "Slider"
        | "range" | "Range" | "combobox" | "Combobox" | "autocomplete" | "Autocomplete" => {
            Some("name")
        }

        // Elements with "src" as primary prop (images/media)
        "image" | "Image" | "img" | "video" | "Video" | "audio" | "Audio" | "icon" | "Icon" => {
            Some("src")
        }

        // Elements with "text" as primary prop
        // All text content elements and buttons
        "text" | "Text" | "h1" | "H1" | "h2" | "H2" | "h3" | "H3" | "h4" | "H4" | "h5" | "H5"
        | "h6" | "H6" | "p" | "P" | "span" | "Span" | "label" | "Label" | "button" | "Button"
        | "a" | "link" | "Link" | "th" | "Th" | "td" | "Td" | "li" | "Li" | "option" | "Option"
        | "summary" | "Summary" | "badge" | "Badge" | "tag" | "Tag" | "chip" | "Chip" | "toast"
        | "Toast" | "alert" | "Alert" | "menu-item" | "MenuItem" | "context-menu-item"
        | "ContextMenuItem" | "dropdown-item" | "DropdownItem" => Some("text"),

        // Default: text prop for all other elements
        _ => Some("text"),
    }
}

/// Extract template and bindings from f-string expression
/// Returns (template_string, vec_of_binding_names)
pub(crate) fn extract_fstr_template_and_bindings(expr: &Expr) -> (String, Vec<String>) {
    let mut template = String::new();
    let mut bindings = Vec::new();

    // fstr() returns Expr::FStr with parts
    if let Expr::FStr(fstr) = expr {
        for part in &fstr.parts {
            match part {
                Expr::Str(s) => {
                    // Literal string part
                    template.push_str(s.as_str());
                }
                Expr::Ident(name) => {
                    // $ident interpolation
                    let name_str = name.as_str();
                    if name_str.starts_with('.') {
                        // State reference: .count -> ${.count}
                        let binding = name_str[1..].to_string();
                        bindings.push(binding.clone());
                        template.push_str(&format!("${{{}.{}}}", ".", binding));
                    } else {
                        // Variable reference: name -> ${name}
                        bindings.push(name_str.to_string());
                        template.push_str(&format!("${{{}}}", name_str));
                    }
                }
                Expr::Dot(obj, field) => {
                    // $obj.field interpolation (including .field which is parsed as self.field)
                    let field_str = field.as_str();
                    if let Expr::Ident(obj_name) = obj.as_ref() {
                        if obj_name.as_str() == "." {
                            // .count -> ${.count}
                            bindings.push(field_str.to_string());
                            template.push_str(&format!("${{{}.{}}}", ".", field_str));
                        } else if obj_name.as_str() == "self" {
                            // self.count -> ${.count} (state reference)
                            bindings.push(field_str.to_string());
                            template.push_str(&format!("${{{}.{}}}", ".", field_str));
                        } else {
                            // obj.field -> ${obj.field}
                            bindings.push(format!("{}.{}", obj_name, field_str));
                            template.push_str(&format!("${{{}.{}}}", obj_name, field_str));
                        }
                    }
                }
                _ => {
                    // Complex expression - just use placeholder
                    template.push_str("${...}");
                }
            }
        }
    }

    (template, bindings)
}

/// Plan 425: fragment param type hint (raw token string) → `Type`.
/// Mirrors the Plan 408 `fragment_param_type` mapping (extract.rs) so the
/// sugared `component fn` produces byte-identical props: primitive hints
/// map to primitives, everything else (custom types, composites, empty)
/// erases to `Type::Unknown` → `any`.
pub(crate) fn fragment_param_hint_to_type(type_hint: &str) -> Type {
    match type_hint.trim() {
        "str" => Type::StrSlice,
        "int" | "i64" | "uint" | "usize" => Type::Int,
        "float" | "double" => Type::Float,
        "bool" => Type::Bool,
        _ => Type::Unknown,
    }
}

/// PLAN-533 T5: 模态对话框家族 tag 角色（归一化：剥 `-`/`_` + 小写，
/// 与 ui_gen/rust.rs 的 modal_dialog_tag_role 同规则）。
pub(crate) fn modal_dialog_tag_role(tag: &str) -> Option<&'static str> {
    let norm: String = tag
        .chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase();
    match norm.as_str() {
            "alertdialog" | "dialog" | "dropdownmenu"
            // PLAN-534: sheet/drawer 并入自管开合铸造机器（可关闭族,同
            // dialog 语义——gallery 页无显式 open,靠铸造 toggle 开合）。
            | "sheet" | "drawer" => Some("root"),
            "alertdialogtrigger" | "dialogtrigger" | "dropdownmenutrigger"
            | "sheettrigger" | "drawertrigger" => Some("trigger"),
            "alertdialogcontent" | "dialogcontent" | "dropdownmenucontent"
            | "sheetcontent" | "drawercontent" => Some("content"),
            "alertdialogcancel" => Some("cancel"),
            "alertdialogaction" => Some("action"),
            "alertdialogclose" | "dialogclose"
            | "sheetclose" | "drawerclose" => Some("close"),
            "dropdownmenuitem" => Some("item"),
            "dropdownmenulabel" => Some("label"),
            "dropdownmenuseparator" => Some("separator"),
            _ => None,
        }
}

/// PLAN-534 D4: hovercard 根/触发器判定（归一化同 modal_dialog_tag_role:
/// 去 -/_ 小写）。不并入该表——hover 走 enter/leave 而非 toggle/close,
/// 复用同一条铸造机器但接线面不同。
pub(crate) fn hover_card_role(tag: &str) -> Option<&'static str> {
    let norm: String = tag
        .chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase();
    match norm.as_str() {
        "hovercard" => Some("root"),
        "hovercardtrigger" => Some("trigger"),
        _ => None,
    }
}

/// The state field a `value:` prop targets, when it is a direct state
/// reference — `.field` (parsed as `Dot(Ident("self"), field)`) or the
/// legacy bare `field` form. Multi-level dots and non-expr values return
/// None (mirrors the `is_direct_self` rule of ui_gen/rust.rs's
/// scan_input_fields and dynamic.rs's scan_node_for_inputs).
pub(crate) fn direct_state_value_field(props: &[ViewProp]) -> Option<String> {
    let prop = props.iter().find(|p| p.name.as_str() == "value")?;
    match &prop.value {
        ViewPropValue::Expr(Expr::Ident(name)) => Some(name.to_string()),
        ViewPropValue::Expr(Expr::Dot(obj, field)) => match obj.as_ref() {
            Expr::Ident(base) if base.as_str() == "self" || base.as_str() == "." => {
                Some(field.to_string())
            }
            _ => None,
        },
        _ => None,
    }
}

/// PLAN-533 T5: 在对话框根子树内递归接线 close 家族
/// （cancel/action/close 无 onclick → 补 `.__dlg_close_<n>`）。
pub(crate) fn wire_modal_close_recursive(node: &mut ViewNode, close_variant: &str) {
    match node {
        ViewNode::Element {
            tag,
            events,
            children,
            ..
        } => {
            if matches!(
                modal_dialog_tag_role(tag),
                Some("cancel") | Some("action") | Some("close")
            ) && !events
                .iter()
                .any(|e| matches!(e.name.as_str(), "onclick" | "onClick" | "on_click"))
            {
                events.push(ViewEvent {
                    name: "onclick".to_string(),
                    handler: format!(".{close_variant}"),
                    params: Vec::new(),
                    inline: None,
                });
            }
            for c in children.iter_mut() {
                wire_modal_close_recursive(c, close_variant);
            }
        }
        ViewNode::ForLoop { body, .. } => {
            for c in body.iter_mut() {
                wire_modal_close_recursive(c, close_variant);
            }
        }
        ViewNode::Conditional {
            then_body,
            else_body,
            ..
        } => {
            for c in then_body.iter_mut() {
                wire_modal_close_recursive(c, close_variant);
            }
            if let Some(else_nodes) = else_body {
                for c in else_nodes.iter_mut() {
                    wire_modal_close_recursive(c, close_variant);
                }
            }
        }
        ViewNode::Link { children, .. } => {
            for c in children.iter_mut() {
                wire_modal_close_recursive(c, close_variant);
            }
        }
        _ => {}
    }
}

/// Plan 451: 从 on-handler 模式串提取事件名——".ActNew" / ".AddItem(text)"
/// → "ActNew" / "AddItem"（actions 块 handler 的编译期校验用）。
pub(crate) fn on_handler_event_name(pattern: &str) -> String {
    let p = pattern.trim_start_matches('.');
    p.split('(').next().unwrap_or(p).to_string()
}

/// Damerau-Levenshtein distance == 1 test (insert/delete/substitute/adjacent
/// transpose) — used for near-miss block-keyword warnings without allocating
/// a full distance matrix.
pub(crate) fn damerau_levenshtein_1(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (la, lb) = (a.len(), b.len());
    match la.cmp(&lb) {
        std::cmp::Ordering::Equal => {
            let mut diff = Vec::with_capacity(2);
            for i in 0..la {
                if a[i] != b[i] {
                    diff.push(i);
                    if diff.len() > 2 {
                        return false;
                    }
                }
            }
            match diff.as_slice() {
                [_] => true,
                [i, j] if *j == *i + 1 => a[*i] == b[*j] && a[*j] == b[*i],
                _ => false,
            }
        }
        std::cmp::Ordering::Less => {
            if lb - la != 1 {
                return false;
            }
            let mut ai = 0;
            let mut skipped = false;
            for &c in b {
                if ai < la && a[ai] == c {
                    ai += 1;
                } else if !skipped {
                    skipped = true;
                } else {
                    return false;
                }
            }
            true
        }
        std::cmp::Ordering::Greater => {
            // mirror: one deletion from a
            if la - lb != 1 {
                return false;
            }
            let mut bi = 0;
            let mut skipped = false;
            for &c in a {
                if bi < lb && b[bi] == c {
                    bi += 1;
                } else if !skipped {
                    skipped = true;
                } else {
                    return false;
                }
            }
            true
        }
    }
}

/// Plan 426: does an expression contain an `.await`? Setup-block MVP rejects
/// awaits (async setup needs a Suspense boundary — future work).
pub(crate) fn stmt_expr_contains_await(expr: &Expr) -> bool {
    match expr {
        Expr::Await { .. } => true,
        Expr::Call(call) => {
            stmt_expr_contains_await(&call.name)
                || call
                    .args
                    .args
                    .iter()
                    .any(|a| stmt_expr_contains_await(&a.get_expr()))
        }
        Expr::Bina(l, _, r) => stmt_expr_contains_await(l) || stmt_expr_contains_await(r),
        Expr::Unary(_, e) => stmt_expr_contains_await(e),
        Expr::Dot(obj, _) => stmt_expr_contains_await(obj),
        Expr::Array(items) => items.iter().any(stmt_expr_contains_await),
        _ => false,
    }
}

/// 前导点两可判定（Dot token 或 text 为 "." 的标识符——条件 DSL 捕获用；
/// 自 Parser parse_actions_cond_attr/parse_condition_expr 的同名闭包提升，
/// 语义逐字保持）。
pub(crate) fn at_dot(cx: &mut dyn UiParseContext) -> bool {
    cx.is_kind(TokenKind::Dot) || cx.cur_text() == "."
}
