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
    ConstArgUse, EnumItem, Expr, ExprKind, FnItem, Ident, InstanceArguments, Program, Receiver,
    TypeAnn, TypeParamKind, TypeRef, TypeWord,
};
use crate::source::SourceProfile;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

pub struct AutoInstanceDemand {
    pub owner: String,
    pub call: Span,
    pub callee: String,
    pub types: BTreeMap<String, TypeAnn>,
    pub consts: Vec<u32>,
}

pub struct AutoEnumDemand {
    /// An annotation demands a type without an expression to rebind.
    pub owner: Option<String>,
    pub call: Span,
    pub template: String,
    pub types: BTreeMap<String, TypeAnn>,
}

pub struct AutoInstanceDemands {
    pub calls: Vec<AutoInstanceDemand>,
    pub constructors: Vec<AutoEnumDemand>,
}

pub(super) struct InstanceProbe<'a> {
    templates: &'a BTreeMap<String, FnItem>,
    enums: &'a BTreeMap<String, EnumItem>,
    constructors: RefCell<Vec<AutoEnumDemand>>,
    demands: RefCell<Vec<AutoInstanceDemand>>,
    body_diagnostics: Cell<usize>,
    owner: RefCell<String>,
}

impl InstanceProbe<'_> {
    pub(super) fn register_enum_types(&self, tables: &mut CollTables) {
        tables.enum_type_templates = self
            .enums
            .iter()
            .map(|(name, item)| {
                let parameters = item
                    .type_params
                    .iter()
                    .map(|param| param.name.name.clone())
                    .collect();
                (name.clone(), parameters)
            })
            .collect();
    }

    pub(super) fn collect_enum_annotations(&self, tables: &mut CollTables) {
        self.constructors
            .borrow_mut()
            .append(&mut tables.enum_type_requests);
    }

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
    enums: &BTreeMap<String, EnumItem>,
    capabilities: &[String],
) -> Result<AutoInstanceDemands, Vec<Diagnostic>> {
    let probe = InstanceProbe {
        templates,
        enums,
        constructors: RefCell::new(Vec::new()),
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
        false,
    )
    .err()
    .expect("instance discovery never publishes a resolved program");
    let calls = probe.demands.into_inner();
    let constructors = probe.constructors.into_inner();
    if calls.is_empty() && constructors.is_empty() && !diagnostics.is_empty() {
        Err(diagnostics)
    } else {
        Ok(AutoInstanceDemands {
            calls,
            constructors,
        })
    }
}

impl FnResolver<'_, '_> {
    pub(super) fn generic_enum_receiver<'e>(
        &self,
        receiver: &'e Expr,
    ) -> Option<(&'e Ident, &'e [TypeAnn])> {
        let probe = self.instance_probe?;
        let (name, types) = match &receiver.kind {
            ExprKind::Var(name) if self.lookup(&name.name).is_none() => (name, &[][..]),
            ExprKind::GenericType { name, args } => (name, args.as_slice()),
            _ => return None,
        };
        probe
            .enums
            .contains_key(&name.name)
            .then_some((name, types))
    }

    pub(super) fn probe_enum_constructor(
        &mut self,
        name: &Ident,
        annotations: &[TypeAnn],
        member: &Ident,
        args: &[ResolvedArg],
        span: Span,
    ) -> ResolvedExpr {
        let pending = pending_expression(span);
        let probe = self.instance_probe.expect("enum discovery has a probe");
        if self.diags.len() != probe.body_diagnostics.get() {
            return pending;
        }
        let template = &probe.enums[&name.name];
        let Some(arm) = template
            .arms
            .iter()
            .find(|arm| arm.name.name == member.name)
        else {
            self.bind_err(
                "source.pattern-unknown-arm",
                format!("enum `{}` has no arm named `{}`", name.name, member.name),
                member.span,
            );
            return pending;
        };
        let mut types = BTreeMap::new();
        let mut required = BTreeSet::new();
        if !annotations.is_empty() {
            if annotations.len() != template.type_params.len() {
                self.bind_err(
                    "type-arity-mismatch",
                    format!(
                        "`{}` needs {} type argument(s), found {}",
                        name.name,
                        template.type_params.len(),
                        annotations.len()
                    ),
                    span,
                );
                return pending;
            }
            for (param, ann) in template.type_params.iter().zip(annotations) {
                let Some(canonical) = self.canonical_explicit_type(ann) else {
                    return pending;
                };
                types.insert(param.name.name.clone(), canonical);
            }
        } else {
            let params = match &arm.payload {
                Some(payload) => match &payload.kind {
                    TypeRef::Tuple(elements) => elements.iter().collect(),
                    _ => vec![payload],
                },
                None => arm.fields.iter().map(|(_, ty)| ty).collect::<Vec<_>>(),
            };
            let labels = arm
                .fields
                .iter()
                .map(|(id, _)| id.name.clone())
                .collect::<Vec<_>>();
            self.check_argument_shape(&member.name, &labels, params.len(), args, span);
            if self.diags.len() != probe.body_diagnostics.get() {
                return pending;
            }
            for (index, arg) in args.iter().enumerate() {
                let slot = arg
                    .name
                    .as_ref()
                    .and_then(|name| labels.iter().position(|label| label == &name.name))
                    .unwrap_or(index);
                let formal = params[slot];
                collect_type_variables(formal, &mut required);
                if let Some(actual) = source_annotation(arg.value.vt, self.records, self.coll) {
                    if !collect_type_arguments(formal, &actual, &mut types) {
                        self.bind_err(
                            "type-mismatch",
                            format!(
                                "conflicting payload types for `{}.{}`",
                                name.name, member.name
                            ),
                            arg.span,
                        );
                        return pending;
                    }
                }
            }
            if let Some(var) = required.iter().find(|name| !types.contains_key(*name)) {
                self.bind_err(
                    "unsolved-variable",
                    format!("cannot determine type argument `{var}` for this variant payload"),
                    span,
                );
                return pending;
            }
        }
        probe.constructors.borrow_mut().push(AutoEnumDemand {
            owner: Some(probe.owner.borrow().clone()),
            call: span,
            template: name.name.clone(),
            types,
        });
        self.bind_err(
            "source.instance-pending",
            "waiting for a concrete enum instance",
            span,
        );
        pending
    }

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
        expected: Option<VT>,
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
        Some(self.probe_known_template(
            probe,
            &callee.name,
            template,
            args,
            explicit,
            call,
            expected,
        ))
    }

    pub(super) fn probe_template_method(
        &mut self,
        static_rec: Option<usize>,
        receiver: Option<VT>,
        method: &Ident,
        args: &mut Vec<ResolvedArg>,
        explicit: Option<&InstanceArguments>,
        call: Span,
        expected: Option<VT>,
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
            expected,
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
        expected: Option<VT>,
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
        if explicit_types.is_empty() {
            if let Some(expected) =
                expected.and_then(|ty| source_annotation(ty, self.records, self.coll))
            {
                let mut result_types = BTreeMap::new();
                if collect_type_arguments(&template.ret, &expected, &mut result_types) {
                    for (param, ty) in result_types {
                        types.entry(param).or_insert(ty);
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

    fn canonical_explicit_type(&mut self, ann: &TypeAnn) -> Option<TypeAnn> {
        let before = self.diags.len();
        let ty = vt_of_ann_coll(ann, self.rec_index, self.coll, self.diags, TypePos::Local);
        if self.diags.len() != before {
            return None;
        }
        let canonical = source_annotation(ty, self.records, self.coll);
        if canonical.is_none() {
            self.bind_err(
                "source.instances-unsupported",
                "unsupported explicit instance type",
                ann.span,
            );
        }
        canonical
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
                    let canonical = self.canonical_explicit_type(ann)?;
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

fn collect_type_variables(ty: &TypeAnn, names: &mut BTreeSet<String>) {
    match &ty.kind {
        TypeRef::TyVar(name) => {
            names.insert(name.name.clone());
        }
        TypeRef::Generic { args, .. } | TypeRef::Tuple(args) => {
            for arg in args {
                collect_type_variables(arg, names);
            }
        }
        TypeRef::Function { params, result } => {
            for arg in params {
                collect_type_variables(arg, names);
            }
            collect_type_variables(result, names);
        }
        TypeRef::Optional(inner) | TypeRef::ResultBang(inner) | TypeRef::Stream(inner) => {
            collect_type_variables(inner, names);
        }
        TypeRef::Builtin(_) | TypeRef::Record(_) | TypeRef::Inferred => {}
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
    source_annotation_by_name(
        vt,
        &|record| records.get(record).map(|r| r.name.clone()),
        tables,
    )
}

fn source_annotation_by_name(
    vt: VT,
    record_name: &impl Fn(usize) -> Option<String>,
    tables: &CollTables,
) -> Option<TypeAnn> {
    // Spans are kept on the demand. Canonical type equality has no coordinates.
    let span = Span { start: 0, end: 0 };
    let ident = |name: &str| Ident {
        name: name.to_string(),
        span,
    };
    let ann = |ty| source_annotation_by_name(ty, record_name, tables);
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
        VT::Rec(index) => record_instance_annotation(&record_name(index)?, tables, span),
        VT::List(index) => TypeRef::Generic {
            name: ident("List"),
            args: vec![ann(*tables.lists.get(index)?)?],
        },
        VT::Variant(index) => {
            if let Some(instance) = tables.enum_instances.get(&index) {
                if let Some(args) = instance.arguments.iter().cloned().collect() {
                    return Some(TypeAnn {
                        span,
                        kind: TypeRef::Generic {
                            name: ident(&instance.template),
                            args,
                        },
                    });
                }
            }
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

fn record_instance_annotation(name: &str, tables: &CollTables, span: Span) -> TypeRef {
    let ident = |name: &str| Ident {
        name: name.to_owned(),
        span,
    };
    match tables.record_instances.get(name) {
        Some(instance) => TypeRef::Generic {
            name: ident(&instance.template),
            args: instance
                .arguments
                .iter()
                .map(|argument| {
                    let kind = match &argument.kind {
                        TypeRef::Record(record) => {
                            record_instance_annotation(&record.name, tables, span)
                        }
                        TypeRef::Builtin(TypeWord::I32) => TypeRef::Builtin(TypeWord::Int),
                        kind => kind.clone(),
                    };
                    TypeAnn { kind, span }
                })
                .collect(),
        },
        None => TypeRef::Record(ident(name)),
    }
}

pub(super) fn enum_type_annotation(
    name: &Ident,
    args: &[TypeAnn],
    rec_index: &BTreeMap<String, usize>,
    tables: &mut CollTables,
    diags: &mut Vec<Diagnostic>,
    span: Span,
) -> Option<VT> {
    let arity = tables
        .enum_type_templates
        .get(&name.name)
        .map(Vec::len)
        .or_else(|| {
            tables
                .enum_instances
                .values()
                .find_map(|item| (item.template == name.name).then_some(item.arguments.len()))
        })?;
    if args.len() != arity {
        diags.push(Diagnostic::new(
            crate::atom_text::Stage::Bind,
            "type-arity-mismatch",
            format!(
                "`{}` needs {arity} type argument(s), found {}",
                name.name,
                args.len()
            ),
            span,
        ));
        return Some(VT::I32);
    }
    let diag_count = diags.len();
    let mut canonical = Vec::with_capacity(args.len());
    for arg in args {
        let vt = vt_of_ann_coll(arg, rec_index, tables, diags, TypePos::Local);
        if diags.len() != diag_count {
            return Some(VT::I32);
        }
        let resolved = source_annotation_by_name(
            vt,
            &|record| {
                rec_index
                    .iter()
                    .find_map(|(symbol, &index)| (index == record).then(|| symbol.clone()))
            },
            tables,
        );
        let Some(resolved) = resolved else {
            diags.push(Diagnostic::new(
                crate::atom_text::Stage::Bind,
                "source.instances-unsupported",
                "unsupported enum type argument",
                arg.span,
            ));
            return Some(VT::I32);
        };
        canonical.push(Some(resolved));
    }
    if let Some((index, _)) = tables
        .enum_instances
        .iter()
        .find(|(_, item)| item.template == name.name && item.arguments == canonical)
    {
        return Some(VT::Variant(*index));
    }
    if let Some(params) = tables.enum_type_templates.get(&name.name) {
        tables.enum_type_requests.push(AutoEnumDemand {
            owner: None,
            call: span,
            template: name.name.clone(),
            types: params
                .iter()
                .cloned()
                .zip(canonical.into_iter().flatten())
                .collect(),
        });
    }
    diags.push(Diagnostic::new(
        crate::atom_text::Stage::Bind,
        "source.instance-pending",
        format!("waiting for a concrete type for `{}`", name.name),
        span,
    ));
    Some(VT::I32)
}
