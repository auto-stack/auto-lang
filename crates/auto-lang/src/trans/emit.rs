//! Plan 560 T02: AST → Auto 源发射器（s2s 改写器的产物面）。
//!
//! P555-D2 裁定落地：W1 的 token 粒度帧无法表达 E2 语句展开与表达式
//! 重排，W2 起 s2s 走 **AST 帧形**——规则改写 `Code`（AST），本模块把
//! 改写后的 AST 打印回正常模式 Auto 源。
//!
//! 覆盖面 = 脚本子集（.as 语料/py 套件面）：fn/use/if/for/try/var/
//! return/break/continue/block 与值表达式族。**未覆盖变体一律响亮报错**
//! （`emit: unsupported ...`）——扩展点显式，不静默丢语句。
//!
//! 布局约定：4 空格缩进；块 ` {` 同行、`}` 独占一行缩进对齐；语句尾
//! 无分号。发射器**不写尾随换行**（由语句层统一补）——保证 if/else 链
//! 与块表达式拼接无杂散大括号。

use crate::ast::{Arg, Args, Body, Call, Code, Expr, Fn, For, If, Iter, Stmt, Store, StoreKind, Try, Use, UseKind};
use crate::error::{AutoError, AutoResult};

/// 发射整棵 Code（顶层语句逐条）。
pub fn emit_code(code: &Code) -> AutoResult<String> {
    let mut out = String::new();
    for stmt in &code.stmts {
        emit_stmt(stmt, 0, &mut out)?;
        out.push('\n');
    }
    Ok(out)
}

fn indent(n: usize, out: &mut String) {
    for _ in 0..n {
        out.push_str("    ");
    }
}

fn unsupported(what: &str) -> AutoError {
    AutoError::Msg(format!("emit: unsupported {}", what))
}

/// auto_val::Op 的 Display 是 S-expr 调试形态（"(op <)"）——源发射用
/// 符号表（Plan 560 T02：语料靶场实证）。
fn op_symbol(op: &auto_val::Op) -> AutoResult<&'static str> {
    use auto_val::Op;
    Ok(match op {
        Op::Add => "+",
        Op::Sub => "-",
        Op::Mul => "*",
        Op::Div => "/",
        Op::Mod => "%",
        Op::AddEq => "+=",
        Op::SubEq => "-=",
        Op::MulEq => "*=",
        Op::DivEq => "/=",
        Op::ModEq => "%=",
        Op::Not => "!",
        Op::And => "&&",
        Op::Or => "||",
        Op::Asn => "=",
        Op::Eq => "==",
        Op::Neq => "!=",
        Op::Lt => "<",
        Op::Gt => ">",
        Op::Le => "<=",
        Op::Ge => ">=",
        Op::Range => "..",
        Op::RangeEq => "..=",
        Op::Dot => ".",
        Op::Colon => ":",
        Op::In => "in",
        _ => return Err(unsupported(&format!("op {:?}", op))),
    })
}

/// Type 的源形（Display 是 S-expr 调试形态）。脚本子集覆盖面；
/// 未覆盖变体响亮报错。
fn type_source(ty: &crate::ast::Type) -> AutoResult<String> {
    use crate::ast::Type;
    Ok(match ty {
        Type::Byte => "byte".into(),
        Type::Int => "int".into(),
        Type::Uint => "uint".into(),
        Type::USize => "usize".into(),
        Type::I64 => "i64".into(),
        Type::U64 => "u64".into(),
        Type::Float => "float".into(),
        Type::Double => "double".into(),
        Type::Bool => "bool".into(),
        Type::Char => "char".into(),
        Type::StrFixed(n) => format!("str({})", n),
        Type::StrSlice => "str".into(),
        Type::StrOwned => "Str".into(),
        Type::CStrLit => "cstr".into(),
        Type::List(elem) => format!("List<{}>", type_source(elem)?),
        Type::Map(k, v) => format!("Map<{}, {}>", type_source(k)?, type_source(v)?),
        Type::Option(inner) => format!("?{}", type_source(inner)?),
        Type::Result(inner) => format!("!{}", type_source(inner)?),
        Type::Slice(st) => format!("[]{}", type_source(&st.elem)?),
        Type::Array(at) => format!("[{}]{}", at.len, type_source(&at.elem)?),
        Type::User(decl) => decl.name.as_str().to_string(),
        Type::Void | Type::Unknown | Type::Variadic => "void".into(),
        _ => return Err(unsupported("type")),
    })
}

/// 类型是否可安全省略（Unit/Unknown=无标注；Store 的推断型省略后
/// 再解析由推断回填，fn 签名不省略）。
fn type_omittable(ty: &crate::ast::Type) -> bool {
    matches!(
        ty,
        crate::ast::Type::Void | crate::ast::Type::Unknown
    )
}

/// 块体：` {\n ...stmts... \n indent}`——不写尾随换行。
fn emit_body(body: &Body, depth: usize, out: &mut String) -> AutoResult<()> {
    if body.stmts.is_empty() {
        out.push_str(" { }");
        return Ok(());
    }
    out.push_str(" {\n");
    for stmt in &body.stmts {
        emit_stmt(stmt, depth + 1, out)?;
        out.push('\n');
    }
    indent(depth, out);
    out.push('}');
    Ok(())
}

fn emit_stmt(stmt: &Stmt, depth: usize, out: &mut String) -> AutoResult<()> {
    match stmt {
        Stmt::Fn(f) => {
            indent(depth, out);
            emit_fn(f, out)?;
        }
        Stmt::Use(u) => {
            indent(depth, out);
            emit_use(u, out)?;
        }
        Stmt::Store(s) => {
            indent(depth, out);
            emit_store(s, out)?;
        }
        Stmt::Return(e) => {
            indent(depth, out);
            out.push_str("return ");
            emit_expr(e, depth, out)?;
        }
        Stmt::Expr(e) => {
            indent(depth, out);
            emit_expr(e, depth, out)?;
        }
        Stmt::If(i) => {
            indent(depth, out);
            emit_if(i, depth, out)?;
        }
        Stmt::For(f) => {
            indent(depth, out);
            emit_for(f, depth, out)?;
        }
        Stmt::Try(t) => {
            indent(depth, out);
            emit_try(t, depth, out)?;
        }
        Stmt::Block(b) => {
            indent(depth, out);
            emit_body(b, depth, out)?;
        }
        Stmt::Break => {
            indent(depth, out);
            out.push_str("break");
        }
        Stmt::Continue => {
            indent(depth, out);
            out.push_str("continue");
        }
        Stmt::Comment(c) => {
            indent(depth, out);
            out.push_str("//");
            out.push_str(c.as_str());
        }
        Stmt::EmptyLine(_) => {
            indent(depth, out);
            // 空行：语句层补换行即成
        }
        other => return Err(unsupported(&format!("stmt {}", stmt_kind(other)))),
    }
    Ok(())
}

fn stmt_kind(s: &Stmt) -> &'static str {
    match s {
        Stmt::Is(_) => "Is",
        Stmt::EnumDecl(_) => "EnumDecl",
        Stmt::TypeDecl(_) => "TypeDecl",
        Stmt::Union(_) => "Union",
        Stmt::Tag(_) => "Tag",
        Stmt::SpecDecl(_) => "SpecDecl",
        Stmt::Node(_) => "Node",
        Stmt::Dep(_) => "Dep",
        Stmt::OnEvents(_) => "OnEvents",
        Stmt::Alias(_) => "Alias",
        Stmt::TypeAlias(_) => "TypeAlias",
        Stmt::Reply(_) => "Reply",
        Stmt::Ext(_) => "Ext",
        Stmt::WidgetDecl(_) => "WidgetDecl",
        Stmt::StoreDecl(_) => "StoreDecl",
        Stmt::ActionsDecl(_) => "ActionsDecl",
        Stmt::ViewFragmentDecl(_) => "ViewFragmentDecl",
        Stmt::MsgDecl(_) => "MsgDecl",
        Stmt::ModelBlock(_) => "ModelBlock",
        Stmt::ViewBlock(_) => "ViewBlock",
        Stmt::SceneDecl(_) => "SceneDecl",
        Stmt::TaskDef(_) => "TaskDef",
        Stmt::HashIf(_) => "HashIf",
        Stmt::UseWeb(_) => "UseWeb",
        _ => "Stmt",
    }
}

fn emit_fn(f: &Fn, out: &mut String) -> AutoResult<()> {
    if f.parent.is_some() {
        return Err(unsupported("method fn (parent)"));
    }
    if !f.type_params.is_empty() || !f.const_params.is_empty() {
        return Err(unsupported("generic fn"));
    }
    out.push_str("fn ");
    out.push_str(f.name.as_str());
    out.push('(');
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| -> AutoResult<String> {
            if type_omittable(&p.ty) {
                Ok(p.name.as_str().to_string())
            } else {
                Ok(format!("{} {}", p.name.as_str(), type_source(&p.ty)?))
            }
        })
        .collect::<Result<_, _>>()?;
    out.push_str(&params.join(", "));
    out.push(')');
    if !type_omittable(&f.ret) {
        out.push_str(" -> ");
        out.push_str(&type_source(&f.ret)?);
    }
    emit_body(&f.body, 0, out)?;
    Ok(())
}

fn emit_use(u: &Use, out: &mut String) -> AutoResult<()> {
    let kind = match u.kind {
        UseKind::Auto => "use",
        UseKind::C => "use.c",
        UseKind::Rust => "use.rs",
        UseKind::Py => "use.py",
    };
    out.push_str(kind);
    out.push(' ');
    let path = if let Some(mp) = &u.module_path {
        mp.to_string()
    } else {
        u.paths
            .iter()
            .map(|p| p.as_str().to_string())
            .collect::<Vec<_>>()
            .join(".")
    };
    out.push_str(&path);
    if !u.items.is_empty() {
        out.push_str(": ");
        let items: Vec<&str> = u.items.iter().map(|i| i.as_str()).collect();
        out.push_str(&items.join(", "));
    }
    Ok(())
}

fn emit_store(s: &Store, out: &mut String) -> AutoResult<()> {
    let kw = match s.kind {
        StoreKind::Let => "let",
        StoreKind::Var => "var",
        StoreKind::Const => "const",
        StoreKind::Shared => "shared",
        StoreKind::CVar => return Err(unsupported("CVar store")),
        StoreKind::Field => return Err(unsupported("Field store")),
    };
    out.push_str(kw);
    out.push(' ');
    out.push_str(s.name.as_str());
    // Store 类型标注省略：源标注与推断型在 AST 不可区分，省略后
    // 再解析由推断回填（脚本语料形态 `var x = ...` 占绝对多数；
    // 显式 `var x: T = ...` 的 T 若与推断不同属罕见面，T12 迁移
    // 语料实证后按需展开）。
    out.push_str(" = ");
    emit_expr(&s.expr, 0, out)
}

fn emit_if(i: &If, depth: usize, out: &mut String) -> AutoResult<()> {
    for (n, branch) in i.branches.iter().enumerate() {
        if n == 0 {
            out.push_str("if ");
        } else {
            out.push_str(" else if ");
        }
        emit_expr(&branch.cond, depth, out)?;
        emit_body(&branch.body, depth, out)?;
    }
    if let Some(else_body) = &i.else_ {
        out.push_str(" else");
        emit_body(else_body, depth, out)?;
    }
    Ok(())
}

fn emit_for(f: &For, depth: usize, out: &mut String) -> AutoResult<()> {
    if f.init.is_some() {
        return Err(unsupported("for-init"));
    }
    match &f.iter {
        Iter::Named(name) => {
            out.push_str("for ");
            out.push_str(name.as_str());
            out.push_str(" in ");
        }
        Iter::Indexed(ix, it) => {
            out.push_str("for ");
            out.push_str(ix.as_str());
            out.push_str(", ");
            out.push_str(it.as_str());
            out.push_str(" in ");
        }
        crate::ast::Iter::Destructured(k, v) => {
            out.push_str("for ");
            out.push_str(k.as_str());
            out.push_str(", ");
            out.push_str(v.as_str());
            out.push_str(" in ");
        }
        Iter::Cond => {
            // `while cond { }` 解析为 Iter::Cond（parser.rs:1248 同语义）
            out.push_str("while ");
            emit_expr(&f.range, depth, out)?;
            return emit_body(&f.body, depth, out);
        }
        other => {
            return Err(unsupported(&format!(
                "Iter::{:?}",
                std::mem::discriminant(other)
            )))
        }
    }
    emit_expr(&f.range, depth, out)?;
    emit_body(&f.body, depth, out)
}

fn emit_try(t: &Try, depth: usize, out: &mut String) -> AutoResult<()> {
    // Plan 560 T09：finally 发射（with-as 的出口保证形态）。
    // 空 catch 体省略 catch 子句（with-as 构造形态）。
    out.push_str("try");
    emit_body(&t.body, depth, out)?;
    if !t.catch_body.stmts.is_empty() {
        out.push_str(" catch");
        if let Some(param) = &t.catch_param {
            out.push(' ');
            out.push_str(param);
        }
        emit_body(&t.catch_body, depth, out)?;
    }
    if let Some(fb) = &t.finally_body {
        out.push_str(" finally");
        emit_body(fb, depth, out)?;
    }
    Ok(())
}

fn emit_expr(e: &Expr, depth: usize, out: &mut String) -> AutoResult<()> {
    match e {
        Expr::Int(v) => out.push_str(&v.to_string()),
        Expr::Uint(v) => out.push_str(&v.to_string()),
        Expr::I64(v) => out.push_str(&v.to_string()),
        Expr::U64(v) => out.push_str(&v.to_string()),
        Expr::Byte(v) => out.push_str(&v.to_string()),
        Expr::I8(v) => out.push_str(&v.to_string()),
        Expr::Float(_, orig) => out.push_str(orig.as_str()),
        Expr::Double(_, orig) => out.push_str(orig.as_str()),
        Expr::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Expr::Char(c) => {
            out.push('\'');
            out.push(*c);
            out.push('\'');
        }
        Expr::Str(s) => {
            // Plan 560 T12：再转义（源含 \" 的字符串在发射产物里必须
            // 保持合法字面量——py_json 套件实证）。
            out.push('"');
            for ch in s.as_str().chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    _ => out.push(ch),
                }
            }
            out.push('"');
        }
        Expr::Ident(n) | Expr::GenName(n) => out.push_str(n.as_str()),
        Expr::Ref(n) => {
            out.push('&');
            out.push_str(n.as_str());
        }
        Expr::Unary(op, inner) => {
            out.push_str(op_symbol(op)?);
            emit_expr(inner, depth, out)?;
        }
        Expr::Bina(l, op, r) => {
            emit_expr(l, depth, out)?;
            out.push(' ');
            out.push_str(op_symbol(op)?);
            out.push(' ');
            emit_expr(r, depth, out)?;
        }
        Expr::Dot(obj, field) => {
            emit_expr(obj, depth, out)?;
            out.push('.');
            out.push_str(field.as_str());
        }
        Expr::Range(r) => {
            emit_expr(&r.start, depth, out)?;
            out.push_str(if r.eq { "..=" } else { ".." });
            emit_expr(&r.end, depth, out)?;
        }
        Expr::Array(elems) => {
            out.push('[');
            for (i, el) in elems.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                emit_expr(el, depth, out)?;
            }
            out.push(']');
        }
        Expr::Object(pairs) => {
            out.push('{');
            for (i, p) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                // Key 三形态（NamedKey 裸名/IntKey/BoolKey/StrKey 带引号）
                match &p.key {
                    crate::ast::Key::NamedKey(n) => out.push_str(n.as_str()),
                    crate::ast::Key::IntKey(v) => out.push_str(&v.to_string()),
                    crate::ast::Key::BoolKey(b) => {
                        out.push_str(if *b { "true" } else { "false" })
                    }
                    crate::ast::Key::StrKey(s) => {
                        out.push('"');
                        out.push_str(s.as_str());
                        out.push('"');
                    }
                }
                out.push_str(": ");
                emit_expr(&p.value, depth, out)?;
            }
            out.push('}');
        }
        Expr::Call(c) => emit_call(c, depth, out)?,
        Expr::Index(obj, idx) => {
            emit_expr(obj, depth, out)?;
            out.push('[');
            emit_expr(idx, depth, out)?;
            out.push(']');
        }
        Expr::Block(b) => {
            // 块表达式：单表达式体直接内联（闭包体常见形态）
            if b.stmts.len() == 1 {
                if let Stmt::Expr(inner) = &b.stmts[0] {
                    emit_expr(inner, depth, out)?;
                    return Ok(());
                }
            }
            emit_body(b, depth, out)?;
        }
        Expr::If(i) => {
            out.push_str("if ");
            emit_if(i, depth, out)?;
        }
        Expr::Nil | Expr::Null => out.push_str("null"),
        Expr::None => out.push_str("None"),
        Expr::Some(inner) => {
            out.push_str("Some(");
            emit_expr(inner, depth, out)?;
            out.push(')');
        }
        Expr::Ok(inner) => {
            out.push_str("Ok(");
            emit_expr(inner, depth, out)?;
            out.push(')');
        }
        Expr::Err(inner) => {
            out.push_str("Err(");
            emit_expr(inner, depth, out)?;
            out.push(')');
        }
        Expr::NullCoalesce(l, r) => {
            emit_expr(l, depth, out)?;
            out.push_str(" ?? ");
            emit_expr(r, depth, out)?;
        }
        Expr::ErrorPropagate(inner) => {
            emit_expr(inner, depth, out)?;
            out.push_str(".?");
        }
        Expr::To { expr, target_type } => {
            emit_expr(expr, depth, out)?;
            out.push_str(".to(");
            out.push_str(&type_source(target_type)?);
            out.push(')');
        }
        Expr::Cast { expr, target_type } => {
            emit_expr(expr, depth, out)?;
            out.push_str(".as(");
            out.push_str(&type_source(target_type)?);
            out.push(')');
        }
        Expr::Move(inner) => {
            out.push_str("move ");
            emit_expr(inner, depth, out)?;
        }
        Expr::Closure(c) => {
            let params: Vec<String> =
                c.params.iter().map(|p| p.name.as_str().to_string()).collect();
            if params.len() == 1 {
                out.push_str(&params[0]);
            } else {
                out.push('(');
                out.push_str(&params.join(", "));
                out.push(')');
            }
            // 无尾空格：Block 臂自带前导空格（" => {"）
            out.push_str(" =>");
            emit_expr(&c.body, depth, out)?;
        }
        _ => return Err(unsupported("expr")),
    }
    Ok(())
}

fn emit_call(c: &Call, depth: usize, out: &mut String) -> AutoResult<()> {
    if !c.type_args.is_empty() {
        return Err(unsupported("turbofish call"));
    }
    emit_expr(&c.name, depth, out)?;
    out.push('(');
    let parts: Vec<String> = c
        .args
        .args
        .iter()
        .map(|a| -> AutoResult<String> {
            match a {
                Arg::Pos(e) => expr_to_string(e, depth),
                Arg::Name(n) => Ok(n.as_str().to_string()),
                Arg::Pair(n, e) => {
                    Ok(format!("{}: {}", n.as_str(), expr_to_string(e, depth)?))
                }
            }
        })
        .collect::<Result<_, _>>()?;
    out.push_str(&parts.join(", "));
    out.push(')');
    Ok(())
}

pub(crate) fn expr_to_string(e: &Expr, depth: usize) -> AutoResult<String> {
    let mut s = String::new();
    emit_expr(e, depth, &mut s)?;
    Ok(s)
}
