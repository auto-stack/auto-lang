//! PLAN-1124: Schema-driven `JsonEncodable` method synthesis (`fn json_encode() Str`).
//!
//! Derives JSON object encoding directly from the compiler's checked `TypeItem.fields`
//! in declaration order, rejecting unsupported field types with `source.json-encode-unsupported`.

use std::collections::{BTreeMap, BTreeSet};

use crate::atom_text::{Diagnostic, Span, Stage};

use super::ast::{
    AggArmBody, AggMatchArm, Arg, BinOp, Block, Expr, ExprKind, FnItem, Ident, Item,
    MatchPattern, Program, Receiver, Stmt, TypeAnn, TypeItem, TypeRef, TypeWord,
};

const JSON_ENCODABLE_SPEC: &str = "JsonEncodable";

struct JsonHelpers {
    json_str_fn: &'static str,
    f64_str_fn: &'static str,
    f32_str_fn: &'static str,
}

impl JsonHelpers {
    fn detect(program: &Program) -> Self {
        let package_mode = program
            .items
            .iter()
            .any(|item| matches!(item, Item::Fn(f) if f.name.name == "p3_stdm4_json_json_str"));
        if package_mode {
            Self {
                json_str_fn: "p3_stdm4_json_json_str",
                f64_str_fn: "p3_stdm4_json___std_json_f64_str",
                f32_str_fn: "p3_stdm4_json___std_json_f32_str",
            }
        } else {
            Self {
                json_str_fn: "json_str",
                f64_str_fn: "__std_json_f64_str",
                f32_str_fn: "__std_json_f32_str",
            }
        }
    }
}

/// Synthesizes `fn json_encode() Str` on every concrete `TypeItem` in `program`
/// that declares conformance to `JsonEncodable` without a user-defined `json_encode`.
///
/// Returns `(record_name, "json_encode")` pairs for newly synthesized methods so package
/// metadata can register origin locations.
pub fn synthesize_json_encodable_methods(
    program: &mut Program,
) -> Result<Vec<(String, String)>, Vec<Diagnostic>> {
    let helpers = JsonHelpers::detect(program);
    let record_specs: BTreeMap<String, Vec<String>> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Type(t) if t.type_params.is_empty() => {
                Some((t.name.name.clone(), t.spec_memberships.clone()))
            }
            _ => None,
        })
        .collect();
    let record_fields: BTreeMap<String, Vec<(String, TypeAnn, Span)>> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Type(t) if t.type_params.is_empty() => Some((
                t.name.name.clone(),
                t.fields
                    .iter()
                    .map(|f| (f.name.name.clone(), f.ty.clone(), f.span))
                    .collect(),
            )),
            _ => None,
        })
        .collect();

    let mut diags = Vec::new();
    let mut added = Vec::new();

    for item in &mut program.items {
        let Item::Type(record) = item else {
            continue;
        };
        if !record.type_params.is_empty()
            || !record
                .spec_memberships
                .iter()
                .any(|spec| spec == JSON_ENCODABLE_SPEC)
        {
            continue;
        }
        if record
            .methods
            .iter()
            .any(|m| m.name.name == "json_encode" && m.receiver == Receiver::View)
        {
            continue;
        }

        let ctx = SynthCtx {
            helpers: &helpers,
            record_specs: &record_specs,
            record_fields: &record_fields,
        };
        let mut stack = BTreeSet::new();
        stack.insert(record.name.name.clone());
        match build_json_encode_method(record, &ctx, &mut stack) {
            Ok(method) => {
                added.push((record.name.name.clone(), "json_encode".to_string()));
                record.methods.push(method);
            }
            Err(d) => diags.push(d),
        }
    }

    if diags.is_empty() {
        Ok(added)
    } else {
        Err(diags)
    }
}

struct SynthCtx<'a> {
    helpers: &'a JsonHelpers,
    record_specs: &'a BTreeMap<String, Vec<String>>,
    record_fields: &'a BTreeMap<String, Vec<(String, TypeAnn, Span)>>,
}

fn build_json_encode_method(
    record: &TypeItem,
    ctx: &SynthCtx<'_>,
    stack: &mut BTreeSet<String>,
) -> Result<FnItem, Diagnostic> {
    let sp = record.name.span;
    let out_id = ident("__json_out", sp);
    let mut counter: usize = 0;

    let mut stmts = Vec::new();
    if record.fields.is_empty() {
        stmts.push(Stmt::Return {
            value: str_lit("{}", sp),
            span: sp,
        });
    } else {
        stmts.push(Stmt::Let {
            kw: sp,
            name: out_id.clone(),
            ann: Some(TypeAnn {
                kind: TypeRef::Builtin(TypeWord::StrOwned),
                span: sp,
            }),
            mutable: true,
            value: str_lit("{", sp),
            span: sp,
        });

        for (idx, field) in record.fields.iter().enumerate() {
            let prefix = if idx == 0 {
                format!("\"{}\":", escape_json_key(&field.name.name))
            } else {
                format!(",\"{}\":", escape_json_key(&field.name.name))
            };
            append_out(&mut stmts, &out_id, str_lit(&prefix, field.span), field.span);
            let field_read = Expr {
                kind: ExprKind::Field {
                    base: Box::new(var_expr("self", field.span)),
                    name: ident(&field.name.name, field.span),
                },
                span: field.span,
            };
            emit_value(
                &mut stmts,
                &out_id,
                field_read,
                &field.ty,
                false,
                false,
                field.span,
                ctx,
                stack,
                &mut counter,
            )?;
        }

        append_out(&mut stmts, &out_id, str_lit("}", sp), sp);
        stmts.push(Stmt::Return {
            value: var_expr(&out_id.name, sp),
            span: sp,
        });
    }

    Ok(FnItem {
        name: ident("json_encode", sp),
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::StrOwned),
            span: sp,
        },
        body: Block { stmts, span: sp },
        span: sp,
        receiver: Receiver::View,
        owner: Some(record.name.clone()),
        script: false,
        proto: false,
        test_attribute: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn emit_value(
    stmts: &mut Vec<Stmt>,
    out_id: &Ident,
    expr: Expr,
    ty: &TypeAnn,
    is_list_elem: bool,
    in_elem_view: bool,
    span: Span,
    ctx: &SynthCtx<'_>,
    stack: &mut BTreeSet<String>,
    counter: &mut usize,
) -> Result<(), Diagnostic> {
    match &ty.kind {
        TypeRef::Builtin(word) => {
            match word {
                TypeWord::StrOwned => {
                    let arg_expr = if is_list_elem || in_elem_view {
                        method_call_0(expr, "clone", span)
                    } else {
                        expr
                    };
                    let encoded = call_1(ctx.helpers.json_str_fn, arg_expr, span);
                    append_out(stmts, out_id, encoded, span);
                }
                TypeWord::Str => {
                    let owned = to_str_expr(expr, span);
                    let encoded = call_1(ctx.helpers.json_str_fn, owned, span);
                    append_out(stmts, out_id, encoded, span);
                }
                TypeWord::Bool => {
                    stmts.push(Stmt::If {
                        cond: expr,
                        then: Block {
                            stmts: vec![assign_append_lit(out_id, "true", span)],
                            span,
                        },
                        els: Some(Block {
                            stmts: vec![assign_append_lit(out_id, "false", span)],
                            span,
                        }),
                        span,
                    });
                }
                TypeWord::Int
                | TypeWord::I32
                | TypeWord::I8
                | TypeWord::U8
                | TypeWord::Byte
                | TypeWord::Uint
                | TypeWord::U32
                | TypeWord::I64
                | TypeWord::U64 => {
                    append_out(stmts, out_id, to_str_expr(expr, span), span);
                }
                TypeWord::F64 | TypeWord::Double => {
                    let encoded = call_1(ctx.helpers.f64_str_fn, expr, span);
                    append_out(stmts, out_id, encoded, span);
                }
                TypeWord::Float => {
                    let encoded = call_1(ctx.helpers.f32_str_fn, expr, span);
                    append_out(stmts, out_id, encoded, span);
                }
                TypeWord::Char | TypeWord::Void => {
                    return Err(unsupported_diag(
                        format!("field type `{}` does not support automatic `JsonEncodable` synthesis", type_desc(&ty.kind)),
                        span,
                    ));
                }
            }
            Ok(())
        }
        TypeRef::Record(rec_ident) => {
            let Some(specs) = ctx.record_specs.get(&rec_ident.name) else {
                return Err(unsupported_diag(
                    format!("unknown nested record `{}` in `JsonEncodable` field", rec_ident.name),
                    span,
                ));
            };
            if !specs.iter().any(|s| s == JSON_ENCODABLE_SPEC) {
                return Err(unsupported_diag(
                    format!("nested record `{}` must implement `JsonEncodable`", rec_ident.name),
                    span,
                ));
            }
            if !stack.insert(rec_ident.name.clone()) {
                return Err(unsupported_diag(
                    format!("recursive record cycle `{}` in `JsonEncodable` fields", rec_ident.name),
                    span,
                ));
            }
            if is_list_elem {
                if let Some(fields) = ctx.record_fields.get(&rec_ident.name) {
                    if fields.is_empty() {
                        append_out(stmts, out_id, str_lit("{}", span), span);
                    } else {
                        append_out(stmts, out_id, str_lit("{", span), span);
                        for (idx, (fname, fty, fspan)) in fields.iter().enumerate() {
                            let prefix = if idx == 0 {
                                format!("\"{}\":", escape_json_key(fname))
                            } else {
                                format!(",\"{}\":", escape_json_key(fname))
                            };
                            append_out(stmts, out_id, str_lit(&prefix, *fspan), *fspan);
                            let field_read = Expr {
                                kind: ExprKind::Field {
                                    base: Box::new(expr.clone()),
                                    name: ident(fname, *fspan),
                                },
                                span: *fspan,
                            };
                            emit_value(
                                stmts,
                                out_id,
                                field_read,
                                fty,
                                false,
                                true,
                                *fspan,
                                ctx,
                                stack,
                                counter,
                            )?;
                        }
                        append_out(stmts, out_id, str_lit("}", span), span);
                    }
                    stack.remove(&rec_ident.name);
                    return Ok(());
                }
            }
            if let Some(fields) = ctx.record_fields.get(&rec_ident.name) {
                for (_, fty, fspan) in fields {
                    validate_nested_field(fty, *fspan, ctx, stack)?;
                }
            }
            stack.remove(&rec_ident.name);
            let encoded = method_call_0(expr, "json_encode", span);
            append_out(stmts, out_id, encoded, span);
            Ok(())
        }
        TypeRef::Optional(inner) => {
            if is_list_elem {
                return Err(unsupported_diag(
                    "list elements of optional type do not support automatic `JsonEncodable` synthesis",
                    span,
                ));
            }
            let bind_name = next_tmp("json_v", counter);
            let bind_id = ident(&bind_name, span);
            let mut some_stmts = Vec::new();
            emit_value(
                &mut some_stmts,
                out_id,
                var_expr(&bind_name, span),
                inner,
                false,
                false,
                span,
                ctx,
                stack,
                counter,
            )?;
            let mut none_stmts = Vec::new();
            append_out(&mut none_stmts, out_id, str_lit("null", span), span);

            stmts.push(Stmt::AggIsMatch {
                scrutinee: expr,
                arms: vec![
                    AggMatchArm {
                        pat: MatchPattern::Variant {
                            qual: None,
                            name: ident("Some", span),
                            bindings: vec![bind_id],
                        },
                        body: AggArmBody::Block(Block {
                            stmts: some_stmts,
                            span,
                        }),
                        guard: None,
                        span,
                    },
                    AggMatchArm {
                        pat: MatchPattern::Variant {
                            qual: None,
                            name: ident("None", span),
                            bindings: Vec::new(),
                        },
                        body: AggArmBody::Block(Block {
                            stmts: none_stmts,
                            span,
                        }),
                        guard: None,
                        span,
                    },
                ],
                span,
            });
            Ok(())
        }
        TypeRef::Generic { name, args } if name.name == "List" && args.len() == 1 => {
            if is_list_elem {
                return Err(unsupported_diag(
                    "nested `List<List<...>>` does not support automatic `JsonEncodable` synthesis",
                    span,
                ));
            }
            let idx_name = next_tmp("json_i", counter);
            let len_name = next_tmp("json_n", counter);
            let idx_id = ident(&idx_name, span);
            let len_id = ident(&len_name, span);

            stmts.push(Stmt::Let {
                kw: span,
                name: len_id.clone(),
                ann: Some(TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::I32),
                    span,
                }),
                mutable: false,
                value: method_call_0(expr.clone(), "len", span),
                span,
            });
            stmts.push(Stmt::Let {
                kw: span,
                name: idx_id.clone(),
                ann: Some(TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::I32),
                    span,
                }),
                mutable: true,
                value: Expr {
                    kind: ExprKind::IntLit(0),
                    span,
                },
                span,
            });
            append_out(stmts, out_id, str_lit("[", span), span);

            let mut loop_stmts = Vec::new();
            loop_stmts.push(Stmt::If {
                cond: Expr {
                    kind: ExprKind::Binary {
                        op: BinOp::Gt,
                        lhs: Box::new(var_expr(&idx_name, span)),
                        rhs: Box::new(Expr {
                            kind: ExprKind::IntLit(0),
                            span,
                        }),
                    },
                    span,
                },
                then: Block {
                    stmts: vec![assign_append_lit(out_id, ",", span)],
                    span,
                },
                els: None,
                span,
            });
            let elem_read = Expr {
                kind: ExprKind::MethodCall {
                    recv: Box::new(expr),
                    name: ident("get", span),
                    args: vec![Arg::Pos(var_expr(&idx_name, span))],
                    instance: None,
                },
                span,
            };
            emit_value(
                &mut loop_stmts,
                out_id,
                elem_read,
                &args[0],
                true,
                true,
                span,
                ctx,
                stack,
                counter,
            )?;
            loop_stmts.push(Stmt::Assign {
                target: super::ast::AssignTarget {
                    base: idx_id,
                    path: Vec::new(),
                },
                value: Expr {
                    kind: ExprKind::Binary {
                        op: BinOp::Add,
                        lhs: Box::new(var_expr(&idx_name, span)),
                        rhs: Box::new(Expr {
                            kind: ExprKind::IntLit(1),
                            span,
                        }),
                    },
                    span,
                },
                span,
            });

            stmts.push(Stmt::While {
                cond: Expr {
                    kind: ExprKind::Binary {
                        op: BinOp::Lt,
                        lhs: Box::new(var_expr(&idx_name, span)),
                        rhs: Box::new(var_expr(&len_name, span)),
                    },
                    span,
                },
                body: Block {
                    stmts: loop_stmts,
                    span,
                },
                span,
            });
            append_out(stmts, out_id, str_lit("]", span), span);
            Ok(())
        }
        other => Err(unsupported_diag(
            format!(
                "field type `{}` does not support automatic `JsonEncodable` synthesis",
                type_desc(other)
            ),
            span,
        )),
    }
}

fn validate_nested_field(
    ty: &TypeAnn,
    span: Span,
    ctx: &SynthCtx<'_>,
    stack: &mut BTreeSet<String>,
) -> Result<(), Diagnostic> {
    match &ty.kind {
        TypeRef::Record(rec_ident) => {
            if !stack.insert(rec_ident.name.clone()) {
                return Err(unsupported_diag(
                    format!("recursive record cycle `{}` in `JsonEncodable` fields", rec_ident.name),
                    span,
                ));
            }
            if let Some(fields) = ctx.record_fields.get(&rec_ident.name) {
                for (_, fty, fspan) in fields {
                    validate_nested_field(fty, *fspan, ctx, stack)?;
                }
            }
            stack.remove(&rec_ident.name);
            Ok(())
        }
        TypeRef::Optional(inner) => validate_nested_field(inner, span, ctx, stack),
        TypeRef::Generic { name, args } if name.name == "List" && args.len() == 1 => {
            validate_nested_field(&args[0], span, ctx, stack)
        }
        _ => Ok(()),
    }
}

fn unsupported_diag(message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Bind, "source.json-encode-unsupported", message.into(), span)
}

fn type_desc(kind: &TypeRef) -> String {
    match kind {
        TypeRef::Builtin(w) => format!("{w:?}"),
        TypeRef::Record(id) => id.name.clone(),
        TypeRef::Optional(inner) => format!("?{}", type_desc(&inner.kind)),
        TypeRef::Generic { name, args } => {
            let parts: Vec<String> = args.iter().map(|a| type_desc(&a.kind)).collect();
            format!("{}<{}>", name.name, parts.join(", "))
        }
        TypeRef::Tuple(_) => "Tuple".into(),
        TypeRef::Function { .. } => "Fn".into(),
        TypeRef::ResultBang(_) => "!T".into(),
        TypeRef::Stream(_) => "~Iter".into(),
        TypeRef::TyVar(id) => id.name.clone(),
        TypeRef::Inferred => "_".into(),
    }
}

fn escape_json_key(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn next_tmp(prefix: &str, counter: &mut usize) -> String {
    let id = *counter;
    *counter += 1;
    format!("__{prefix}_{id}")
}

fn ident(name: &str, span: Span) -> Ident {
    Ident {
        name: name.to_string(),
        span,
    }
}

fn var_expr(name: &str, span: Span) -> Expr {
    Expr {
        kind: ExprKind::Var(ident(name, span)),
        span,
    }
}

fn str_lit(value: &str, span: Span) -> Expr {
    Expr {
        kind: ExprKind::StrLit(value.to_string()),
        span,
    }
}

fn call_1(callee: &str, arg: Expr, span: Span) -> Expr {
    Expr {
        kind: ExprKind::Call {
            callee: ident(callee, span),
            args: vec![Arg::Pos(arg)],
        },
        span,
    }
}

fn method_call_0(recv: Expr, name: &str, span: Span) -> Expr {
    Expr {
        kind: ExprKind::MethodCall {
            recv: Box::new(recv),
            name: ident(name, span),
            args: Vec::new(),
            instance: None,
        },
        span,
    }
}

fn to_str_expr(recv: Expr, span: Span) -> Expr {
    Expr {
        kind: ExprKind::Method {
            recv: Box::new(recv),
            name: ident("to", span),
            args: vec![Arg::Pos(Expr {
                kind: ExprKind::TypeWordArg(TypeWord::StrOwned),
                span,
            })],
        },
        span,
    }
}

fn assign_append_lit(out_id: &Ident, text: &str, span: Span) -> Stmt {
    Stmt::Assign {
        target: super::ast::AssignTarget {
            base: out_id.clone(),
            path: Vec::new(),
        },
        value: Expr {
            kind: ExprKind::Binary {
                op: BinOp::Add,
                lhs: Box::new(var_expr(&out_id.name, span)),
                rhs: Box::new(str_lit(text, span)),
            },
            span,
        },
        span,
    }
}

fn append_out(stmts: &mut Vec<Stmt>, out_id: &Ident, rhs: Expr, span: Span) {
    stmts.push(Stmt::Assign {
        target: super::ast::AssignTarget {
            base: out_id.clone(),
            path: Vec::new(),
        },
        value: Expr {
            kind: ExprKind::Binary {
                op: BinOp::Add,
                lhs: Box::new(var_expr(&out_id.name, span)),
                rhs: Box::new(rhs),
            },
            span,
        },
        span,
    });
}
