//! Collect template demands from the ordinary binder's actual argument types.
//!
//! Discovery stops collecting in a body as soon as it has an unfinished
//! call. The materializer rebinds that call and the binder replays the body
//! before any dependent argument can contribute a type.

use super::{
    resolve_detailed_in_mode, vt_of_ann_coll, CollTables, FnResolver, RExprKind, ResolvedArg,
    ResolvedExpr, ResolvedRecord, TypePos, VT,
};
use crate::atom_text::{Diagnostic, Span};
use crate::source::ast::{
    ConstArgUse, FnItem, Ident, InstanceArguments, Program, Receiver, TypeAnn, TypeParamKind,
    TypeRef, TypeWord,
};
use crate::source::SourceProfile;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

pub struct AutoInstanceDemand {
    pub owner: String,
    pub call: Span,
    pub callee: String,
    pub types: BTreeMap<String, TypeAnn>,
    pub consts: Vec<u32>,
}

pub(super) struct InstanceProbe<'a> {
    templates: &'a BTreeMap<String, FnItem>,
    demands: RefCell<Vec<AutoInstanceDemand>>,
    body_diagnostics: Cell<usize>,
    owner: RefCell<String>,
}

impl InstanceProbe<'_> {
    pub(super) fn begin_body(&self, name: &str, owner: Option<&str>, diagnostics: usize) {
        self.body_diagnostics.set(diagnostics);
        *self.owner.borrow_mut() = match owner {
            Some(owner) => format!("{owner}.{name}"),
            None => name.to_owned(),
        };
    }
}

/// The discovery pass cannot return a resolved program. An empty successful
/// result means all call sites have been rebound; the caller then performs
/// normal binding and verification on the concrete AST.
pub fn discover_auto_instance_calls(
    program: &Program,
    templates: &BTreeMap<String, FnItem>,
    capabilities: &[String],
) -> Result<Vec<AutoInstanceDemand>, Vec<Diagnostic>> {
    let probe = InstanceProbe {
        templates,
        demands: RefCell::new(Vec::new()),
        body_diagnostics: Cell::new(0),
        owner: RefCell::new(String::new()),
    };
    let diagnostics = resolve_detailed_in_mode(
        program,
        capabilities,
        SourceProfile::Auto,
        None,
        None,
        None,
        false,
        Some(&probe),
    )
    .err()
    .expect("instance discovery never publishes a resolved program");
    let demands = probe.demands.take();
    if demands.is_empty() && !diagnostics.is_empty() {
        Err(diagnostics)
    } else {
        Ok(demands)
    }
}

impl FnResolver<'_, '_> {
    pub(super) fn is_template_call(&self, callee: &Ident) -> bool {
        self.instance_probe.is_some_and(|probe| {
            probe.templates.contains_key(&callee.name)
                && self.lookup(&callee.name).is_none()
                && !self.nested.scope.contains_key(&callee.name)
        })
    }

    pub(super) fn probe_template_call(
        &mut self,
        callee: &Ident,
        args: Vec<ResolvedArg>,
        explicit: Option<(&[TypeAnn], &[ConstArgUse])>,
        call: Span,
    ) -> Option<ResolvedExpr> {
        let probe = self.instance_probe?;
        let Some(template) = probe
            .templates
            .get(&callee.name)
            .filter(|_| self.is_template_call(callee))
        else {
            if self.diags.len() == probe.body_diagnostics.get() {
                self.bind_err(
                    "source.instances-unsupported",
                    format!("`{}` is not a visible function template", callee.name),
                    call,
                );
            }
            return Some(pending_expression(call));
        };
        Some(self.probe_known_template(probe, &callee.name, template, args, explicit, call))
    }

    pub(super) fn probe_template_method(
        &mut self,
        static_rec: Option<usize>,
        receiver: Option<VT>,
        method: &Ident,
        args: &mut Vec<ResolvedArg>,
        explicit: Option<&InstanceArguments>,
        call: Span,
    ) -> Option<ResolvedExpr> {
        let probe = self.instance_probe?;
        let (owner, is_static) = match (static_rec, receiver) {
            (Some(record), None) => (&self.records[record].name, true),
            (None, Some(VT::Rec(record))) => (&self.records[record].name, false),
            (None, Some(VT::Variant(variant))) => (&self.coll.variants[variant].name, false),
            _ => return None,
        };
        let key = format!("{owner}.{}", method.name);
        let template = probe.templates.get(&key)?;
        if (template.receiver == Receiver::Static) != is_static {
            self.bind_err(
                "source.not-callable",
                format!(
                    "`{key}` requires {} receiver",
                    if is_static { "a value" } else { "a type" }
                ),
                method.span,
            );
            return Some(pending_expression(call));
        }
        Some(self.probe_known_template(
            probe,
            &key,
            template,
            std::mem::take(args),
            explicit.map(|a| (a.types.as_slice(), a.consts.as_slice())),
            call,
        ))
    }

    fn probe_known_template(
        &mut self,
        probe: &InstanceProbe<'_>,
        key: &str,
        template: &FnItem,
        mut args: Vec<ResolvedArg>,
        explicit: Option<(&[TypeAnn], &[ConstArgUse])>,
        call: Span,
    ) -> ResolvedExpr {
        let unresolved = pending_expression(call);
        if self.diags.len() != probe.body_diagnostics.get() {
            // A preceding error, including an unfinished call, can poison
            // a later binding in this scope. Revisit the body after replay.
            return unresolved;
        }
        let (explicit_types, consts) = explicit.unwrap_or_default();
        let Some(mut types) = self.explicit_instance_types(template, explicit_types, consts, call)
        else {
            return unresolved;
        };
        let names: Vec<_> = template
            .params
            .iter()
            .map(|p| p.name.name.clone())
            .collect();
        let defaults: Vec<_> = template.params.iter().map(|p| p.default.clone()).collect();
        self.fill_default_args(&names, &[], &defaults, &mut args, call);
        self.check_argument_shape(key, &names, names.len(), &args, call);
        if self.diags.len() != probe.body_diagnostics.get() {
            return unresolved;
        }
        if explicit_types.is_empty() {
            for (index, arg) in args.iter().enumerate() {
                let slot = arg
                    .name
                    .as_ref()
                    .and_then(|name| names.iter().position(|n| n == &name.name))
                    .unwrap_or(index);
                let formal = &template.params[slot].ty;
                if let Some(actual) = source_annotation(arg.value.vt, self.records, self.coll) {
                    if !collect_type_arguments(formal, &actual, &mut types) {
                        self.bind_err(
                            "type-mismatch",
                            format!(
                                "conflicting types for `{}` at template `{}`",
                                names[slot], key
                            ),
                            arg.span,
                        );
                        return unresolved;
                    }
                }
            }
        }
        if let Some(param) = template
            .type_params
            .iter()
            .find(|p| p.kind == TypeParamKind::Type && !types.contains_key(&p.name.name))
        {
            self.bind_err(
                "unsolved-variable",
                format!(
                    "cannot determine type argument `{}` of `{}`",
                    param.name.name, key
                ),
                call,
            );
            return unresolved;
        }
        let consts = consts
            .iter()
            .map(|c| u32::try_from(c.value))
            .collect::<Result<Vec<_>, _>>();
        let Ok(consts) = consts else {
            self.bind_err("source.literal-range", "value argument outside u32", call);
            return unresolved;
        };
        probe.demands.borrow_mut().push(AutoInstanceDemand {
            owner: probe.owner.borrow().clone(),
            call,
            callee: key.to_owned(),
            types,
            consts,
        });
        self.bind_err(
            "source.instance-pending",
            "waiting for a concrete template instance",
            call,
        );
        unresolved
    }

    fn explicit_instance_types(
        &mut self,
        template: &FnItem,
        explicit_types: &[TypeAnn],
        consts: &[ConstArgUse],
        call: Span,
    ) -> Option<BTreeMap<String, TypeAnn>> {
        let value_count = template
            .type_params
            .iter()
            .filter(|p| p.kind == TypeParamKind::Const)
            .count();
        if value_count != consts.len() {
            self.bind_err(
                if consts.is_empty() {
                    "value-argument-required"
                } else {
                    "value-arity-mismatch"
                },
                format!(
                    "`{}` needs {value_count} value argument(s), found {}",
                    template.name.name,
                    consts.len()
                ),
                call,
            );
            return None;
        }
        let mut types = BTreeMap::new();
        if explicit_types.is_empty() {
            // The existing value-only form can infer its type arguments.
            return Some(types);
        }
        let count = template.type_params.len() - value_count;
        if count != explicit_types.len() {
            self.bind_err(
                "type-arity-mismatch",
                format!(
                    "`{}` needs {count} type argument(s), found {}",
                    template.name.name,
                    explicit_types.len()
                ),
                call,
            );
            return None;
        }
        let mut positioned: Vec<_> = explicit_types
            .iter()
            .map(|ann| (ann.span, Some(ann)))
            .chain(consts.iter().map(|c| (c.span, None)))
            .collect();
        positioned.sort_by_key(|(span, _)| span.start);
        for (param, (span, argument)) in template.type_params.iter().zip(positioned) {
            match (param.kind, argument) {
                (TypeParamKind::Type, Some(ann)) => {
                    let before = self.diags.len();
                    let ty =
                        vt_of_ann_coll(ann, self.rec_index, self.coll, self.diags, TypePos::Local);
                    if self.diags.len() != before {
                        return None;
                    }
                    let Some(canonical) = source_annotation(ty, self.records, self.coll) else {
                        self.bind_err(
                            "source.instances-unsupported",
                            "unsupported explicit instance type",
                            span,
                        );
                        return None;
                    };
                    types.insert(param.name.name.clone(), canonical);
                }
                (TypeParamKind::Const, None) => {}
                _ => {
                    self.bind_err(
                        "source.instances-argument-kind",
                        format!(
                            "argument kind does not match template parameter `{}`",
                            param.name.name
                        ),
                        span,
                    );
                    return None;
                }
            }
        }
        Some(types)
    }
}

pub(super) fn pending_expression(span: Span) -> ResolvedExpr {
    ResolvedExpr {
        span,
        vt: VT::I32,
        kind: RExprKind::IntLit(0),
    }
}

fn collect_type_arguments(
    formal: &TypeAnn,
    actual: &TypeAnn,
    types: &mut BTreeMap<String, TypeAnn>,
) -> bool {
    use TypeRef::*;
    match (&formal.kind, &actual.kind) {
        (TyVar(id), _) => {
            let previous = types
                .entry(id.name.clone())
                .or_insert_with(|| actual.clone());
            previous == actual
        }
        (Optional(formal), Optional(actual))
        | (ResultBang(formal), ResultBang(actual))
        | (Stream(formal), Stream(actual)) => collect_type_arguments(formal, actual, types),
        (
            Generic { name: formal, args },
            Generic {
                name: actual,
                args: actual_args,
            },
        ) if formal.name == actual.name => {
            args.len() == actual_args.len()
                && args
                    .iter()
                    .zip(actual_args)
                    .all(|(a, b)| collect_type_arguments(a, b, types))
        }
        (Tuple(formal), Tuple(actual)) => {
            formal.len() == actual.len()
                && formal
                    .iter()
                    .zip(actual)
                    .all(|(a, b)| collect_type_arguments(a, b, types))
        }
        (
            Function { params, result },
            Function {
                params: actual_params,
                result: actual_result,
            },
        ) => {
            params.len() == actual_params.len()
                && params
                    .iter()
                    .zip(actual_params)
                    .all(|(a, b)| collect_type_arguments(a, b, types))
                && collect_type_arguments(result, actual_result, types)
        }
        // Concrete positions are checked by the normal call binder after replay.
        _ => true,
    }
}

fn source_annotation(vt: VT, records: &[ResolvedRecord], tables: &CollTables) -> Option<TypeAnn> {
    // Spans are kept on the demand. Canonical type equality has no coordinates.
    let span = Span { start: 0, end: 0 };
    let ident = |name: &str| Ident {
        name: name.to_string(),
        span,
    };
    let ann = |ty| source_annotation(ty, records, tables);
    let kind = match vt {
        VT::I32 => TypeRef::Builtin(TypeWord::Int),
        VT::Bool => TypeRef::Builtin(TypeWord::Bool),
        VT::Str => TypeRef::Builtin(TypeWord::Str),
        VT::StrOwned => TypeRef::Builtin(TypeWord::StrOwned),
        VT::Char => TypeRef::Builtin(TypeWord::Char),
        VT::Unit => TypeRef::Builtin(TypeWord::Void),
        VT::I8 => TypeRef::Builtin(TypeWord::I8),
        VT::U8 => TypeRef::Builtin(TypeWord::U8),
        VT::U32 => TypeRef::Builtin(TypeWord::U32),
        VT::I64 => TypeRef::Builtin(TypeWord::I64),
        VT::U64 => TypeRef::Builtin(TypeWord::U64),
        VT::F32 => TypeRef::Builtin(TypeWord::Float),
        VT::F64 => TypeRef::Builtin(TypeWord::F64),
        VT::Rec(index) => TypeRef::Record(ident(&records.get(index)?.name)),
        VT::List(index) => TypeRef::Generic {
            name: ident("List"),
            args: vec![ann(*tables.lists.get(index)?)?],
        },
        VT::Variant(index) => {
            let variant = tables.variants.get(index)?;
            match variant.name.as_str() {
                "Option" => TypeRef::Optional(Box::new(ann(variant.arms.first()?.1?)?)),
                "Result" => TypeRef::Generic {
                    name: ident("Result"),
                    args: variant
                        .arms
                        .iter()
                        .map(|(_, payload)| ann((*payload)?))
                        .collect::<Option<_>>()?,
                },
                _ => TypeRef::Record(ident(&variant.name)),
            }
        }
        VT::Function(index) => {
            let sig = tables.function_types.get(index)?;
            TypeRef::Function {
                params: sig.params.iter().copied().map(ann).collect::<Option<_>>()?,
                result: Box::new(ann(sig.result)?),
            }
        }
        VT::Tuple(index) => TypeRef::Tuple(
            tables
                .tuples
                .get(index)?
                .iter()
                .copied()
                .map(ann)
                .collect::<Option<_>>()?,
        ),
        _ => return None,
    };
    Some(TypeAnn { kind, span })
}
