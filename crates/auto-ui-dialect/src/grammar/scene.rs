//! Godot scene 声明文法（Plan 306；PLAN-810 T-03 迁移。门在任何
//! scenario 生效，Core 经 Parser 薄委托调用）。

use super::helpers::parse_string_lit;
use crate::UiParseContext;
use auto_parser_api::ast::*;
use auto_parser_api::error::AutoResult;
use auto_parser_api::token::TokenKind;

/// Check if the current position is `scene <Ident>` — a scene declaration.
/// Uses one-token lookahead so `scene.foo()` or `scene()` still parse as
/// ordinary expressions/statements.
pub fn looks_like_scene_decl(cx: &mut dyn UiParseContext) -> bool {
    if !(cx.cur_kind() == TokenKind::Ident && cx.cur_text() == "scene") {
        return false;
    }
    // PLAN-810: advance_peek_kind_resync（旧臂双 push+next 重同步形态，
    // prev 终值=被观察 token——F810-R1-04 与 stream 手动恢复形态区分）。
    cx.advance_peek_kind_resync() == TokenKind::Ident
}

/// Plan 306: Parse a Godot scene declaration.
///
/// ```text
/// scene Name : Type {
///     script = "path.gd"
///     prop = value
///     node Type ["Name"] { ... }
///     instance Name "res://path.tscn"
///     connect signal from "NodePath" to "NodePath" method "method_name"
/// }
/// ```
pub fn parse_scene_decl(cx: &mut dyn UiParseContext) -> AutoResult<Stmt> {
    cx.expect_ident("scene")?;
    let name = cx.cur_text();
    cx.advance();
    cx.expect(TokenKind::Colon)?;
    let node_type = cx.cur_text();
    cx.advance();
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut props = Vec::new();
    let mut script = None;
    let mut children = Vec::new();
    let mut connections = Vec::new();
    let mut signals = Vec::new();

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        let kw = cx.cur_text();
        match kw.as_str() {
            "script" => {
                cx.advance();
                cx.expect(TokenKind::Asn)?;
                script = Some(parse_string_lit(cx)?);
            }
            "node" => {
                children.push(parse_scene_node(cx)?);
            }
            "instance" => {
                children.push(parse_scene_instance(cx)?);
            }
            "connect" => {
                connections.push(parse_scene_connection(cx)?);
            }
            // Plan 306: signal declaration — `signal name(p T, ...)` or `signal name`.
            // Treated as a special scene body element (not a child node; no .tscn output).
            "signal" => {
                signals.push(parse_scene_signal(cx)?);
            }
            _ => {
                // property assignment: name = value
                props.push(parse_scene_prop(cx)?);
            }
        }
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::RBrace)?;
    Ok(Stmt::SceneDecl(SceneDecl {
        name: name.into(),
        node_type: node_type.into(),
        props,
        script,
        children,
        connections,
        signals,
    }))
}

/// Parse a child scene node: `node Type ["Name"] { props; children }`
pub(crate) fn parse_scene_node(cx: &mut dyn UiParseContext) -> AutoResult<SceneNode> {
    cx.expect_ident("node")?;
    let node_type = cx.cur_text();
    cx.advance();

    // Optional explicit instance name: node Type "Name"
    let name = if cx.is_kind(TokenKind::Str) {
        let n = cx.cur_text();
        cx.advance();
        Some(n)
    } else {
        None
    };

    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut props = Vec::new();
    let mut children = Vec::new();

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        let kw = cx.cur_text();
        match kw.as_str() {
            "node" => children.push(parse_scene_node(cx)?),
            "instance" => children.push(parse_scene_instance(cx)?),
            _ => {
                props.push(parse_scene_prop(cx)?);
            }
        }
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(SceneNode::Node {
        node_type: node_type.into(),
        name: name.map(Into::into),
        props,
        children,
    })
}

/// Parse a scene instance: `instance Name "res://path.tscn"`
pub(crate) fn parse_scene_instance(cx: &mut dyn UiParseContext) -> AutoResult<SceneNode> {
    cx.expect_ident("instance")?;
    let name = cx.cur_text();
    cx.advance();
    let path = parse_string_lit(cx)?;
    Ok(SceneNode::Instance {
        name: name.into(),
        path,
    })
}

/// Parse a signal connection:
/// `connect <signal> from "NodePath" to "NodePath" method "method_name"`
pub(crate) fn parse_scene_connection(cx: &mut dyn UiParseContext) -> AutoResult<SceneConnection> {
    cx.expect_ident("connect")?;
    let signal = cx.cur_text();
    cx.advance();
    cx.expect_ident("from")?;
    let from = parse_string_lit(cx)?;
    cx.expect_ident("to")?;
    let to = parse_string_lit(cx)?;
    cx.expect_ident("method")?;
    let method = parse_string_lit(cx)?;
    Ok(SceneConnection {
        signal: signal.into(),
        from,
        to,
        method,
    })
}

/// Parse a signal declaration inside a scene:
/// `signal name` or `signal name(p1 T1, p2 T2)`.
/// Params use Auto space syntax (`p1 T1`) and are emitted as GDScript
/// typed signal params (`p1: T1`).
pub(crate) fn parse_scene_signal(cx: &mut dyn UiParseContext) -> AutoResult<SceneSignal> {
    cx.expect_ident("signal")?;
    let name = cx.cur_text();
    cx.advance();

    let mut params = Vec::new();
    if cx.is_kind(TokenKind::LParen) {
        cx.advance(); // skip (
        cx.skip_empty_lines();
        while !cx.is_kind(TokenKind::RParen) && !cx.is_kind(TokenKind::EOF) {
            let pname = cx.cur_text();
            cx.advance();
            let pty = cx.parse_type()?;
            params.push(SceneSignalParam {
                name: pname.into(),
                ty: pty,
            });
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
                cx.skip_empty_lines();
            }
        }
        cx.expect(TokenKind::RParen)?;
    }

    Ok(SceneSignal {
        name: name.into(),
        params,
    })
}

/// Parse a single `name = value` scene property.
///
/// Shared by the scene root, child nodes, and inline sub-resources. The
/// value may be an ordinary expression or a typed inline sub-resource.
pub(crate) fn parse_scene_prop(cx: &mut dyn UiParseContext) -> AutoResult<SceneProp> {
    let prop_name = cx.cur_text();
    cx.advance();
    cx.expect(TokenKind::Asn)?;
    let value = parse_scene_value(cx)?;
    Ok(SceneProp {
        name: prop_name.into(),
        value,
    })
}

/// Parse a scene property value — either a typed inline sub-resource
/// (`TypeName { ... }`) or a regular expression.
pub(crate) fn parse_scene_value(cx: &mut dyn UiParseContext) -> AutoResult<SceneValue> {
    if looks_like_subresource(cx) {
        Ok(SceneValue::SubResource(parse_scene_subresource(cx)?))
    } else {
        Ok(SceneValue::Expr(cx.parse_expr()?))
    }
}

/// Two-token lookahead: is this `Ident {`? (a typed inline sub-resource).
/// Fully restores the cursor regardless of the answer.
pub(crate) fn looks_like_subresource(cx: &mut dyn UiParseContext) -> bool {
    if !cx.is_kind(TokenKind::Ident) {
        return false;
    }
    // PLAN-810: advance_peek_kind_resync（旧臂双 push+next 重同步形态，
    // prev 终值=被观察 token——F810-R1-04）。
    cx.advance_peek_kind_resync() == TokenKind::LBrace
}

/// Parse an inline typed sub-resource: `TypeName { name = value; ... }`.
pub(crate) fn parse_scene_subresource(cx: &mut dyn UiParseContext) -> AutoResult<SceneSubResource> {
    let res_type = cx.cur_text();
    cx.advance(); // consume type identifier
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut props = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        props.push(parse_scene_prop(cx)?);
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(SceneSubResource {
        res_type: res_type.into(),
        props,
    })
}
