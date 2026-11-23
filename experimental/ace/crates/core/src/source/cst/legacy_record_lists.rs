use std::collections::{BTreeMap, BTreeSet};

use crate::atom_text::Span;
use crate::hir::ObjectKey;
use crate::source::age_component::{
    ComponentField, ComponentHandler, ComponentParam, ComponentStmt,
};
use crate::source::ast::{
    AggArmBody, Arg, BinOp, Expr, ExprKind, FieldDecl, Ident, TypeAnn, TypeItem, TypeRef, TypeWord,
    UnaryOp,
};

#[derive(Clone)]
enum LocalKind {
    Scalar(TypeWord),
    Record(String),
    List(Box<LocalKind>),
    Builder(usize),
}

#[derive(Default)]
struct Builder {
    element: Option<BuilderElement>,
    mixed: bool,
}

impl Builder {
    fn observe(&mut self, element: Option<BuilderElement>) {
        match (&self.element, element) {
            (None, Some(element)) => self.element = Some(element),
            (Some(existing), Some(element)) if existing.same_as(&element) => {}
            _ => self.mixed = true,
        }
    }

    fn element(&self) -> Option<&BuilderElement> {
        (!self.mixed).then_some(self.element.as_ref()).flatten()
    }
}

enum BuilderElement {
    Scalar(TypeWord),
    Record(BTreeMap<String, FieldDecl>),
}

impl BuilderElement {
    fn same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scalar(left), Self::Scalar(right)) => left == right,
            (Self::Record(left), Self::Record(right)) => same_row(left, right),
            _ => false,
        }
    }
}

type Scope = BTreeMap<String, LocalKind>;

#[derive(Default)]
struct HandlerLists {
    builders: BTreeMap<usize, Builder>,
    targets: BTreeMap<String, BTreeSet<usize>>,
    record_fields: BTreeMap<String, BTreeMap<String, TypeWord>>,
}

/// An empty legacy List can get its type from the values appended in a
/// handler. The shared checker verifies all appends and destination values.
pub(super) fn infer_built_lists(
    node: &Ident,
    fields: &mut [ComponentField],
    params: &[ComponentParam],
    handlers: &mut [ComponentHandler],
    records: &mut Vec<TypeItem>,
    argument_records: Option<&BTreeMap<String, Vec<FieldDecl>>>,
) {
    let mut state = params
        .iter()
        .map(|param| (&param.name, &param.collection_type))
        .filter_map(|(name, ty)| Some((name.clone(), annotation_kind(ty.as_ref()?)?)))
        .collect::<Scope>();
    for field in fields.iter() {
        state.remove(&field.name);
        let known = field
            .collection_type
            .as_ref()
            .and_then(annotation_kind)
            .or_else(|| crate::source::age_component::field_type(field).map(LocalKind::Scalar));
        if let Some(kind) = known {
            state.insert(field.name.clone(), kind);
        }
    }
    let record_fields = records
        .iter()
        .map(|record| (&record.name.name, &record.fields))
        .chain(argument_records.into_iter().flatten())
        .map(|(name, fields)| {
            let scalar_fields = fields
                .iter()
                .filter_map(|field| {
                    let TypeRef::Builtin(word) = field.ty.kind else {
                        return None;
                    };
                    Some((field.name.name.clone(), word))
                })
                .collect();
            (name.clone(), scalar_fields)
        })
        .collect();
    let mut facts = HandlerLists {
        record_fields,
        ..HandlerLists::default()
    };
    for handler in handlers.iter() {
        facts.collect(&handler.stmts, &state, &Scope::new());
    }

    let mut annotations: BTreeMap<_, _> = facts.builders.iter().filter_map(|(offset, builder)| {
        let BuilderElement::Scalar(word) = builder.element()? else { return None };
        let span = Span { start: *offset, end: *offset };
        Some((*offset, list_type(TypeRef::Builtin(*word), span)))
    }).collect();
    for field in fields {
        if field.ty != "List" || field.collection_type.is_some() || !is_empty_list(&field.init) {
            continue;
        }
        let Some(ids) = facts.targets.get(&field.name) else {
            continue;
        };
        let mut elements = ids.iter().map(|id| facts.builders[id].element());
        let Some(Some(element)) = elements.next() else {
            continue;
        };
        if !elements.all(|other| other.is_some_and(|other| element.same_as(other))) {
            continue;
        }
        let ann = match element {
            BuilderElement::Scalar(word) => list_type(TypeRef::Builtin(*word), field.ty_span),
            BuilderElement::Record(row) => {
                let record = intern_record(node, field.span, row.values().cloned().collect(), records, false);
                field.ty = format!("List<{}>", record.name);
                list_of(record, field.ty_span)
            }
        };
        field.collection_type = Some(ann.clone());
        annotations.extend(ids.iter().map(|id| (*id, ann.clone())));
    }
    for handler in handlers {
        annotate_builders(&mut handler.stmts, &annotations);
    }
}

impl HandlerLists {
    fn collect(&mut self, stmts: &[ComponentStmt], state: &Scope, inherited: &Scope) {
        let mut scope = inherited.clone();
        for statement in stmts {
            match statement {
                ComponentStmt::Let {
                    name,
                    mutable,
                    ann,
                    value,
                    ..
                } => {
                    let bare_list = ann.as_ref().is_none_or(
                        |ann| matches!(&ann.kind, TypeRef::Record(name) if name.name == "List"),
                    );
                    let kind = if *mutable && bare_list && is_empty_list(value) {
                        self.builders.insert(name.span.start, Builder::default());
                        Some(LocalKind::Builder(name.span.start))
                    } else if let Some(ann) = ann {
                        annotation_kind(ann)
                    } else {
                        self.value_kind(value, state, &scope)
                    };
                    scope.remove(&name.name);
                    if let Some(kind) = kind {
                        scope.insert(name.name.clone(), kind);
                    }
                }
                ComponentStmt::Effect { value, .. } | ComponentStmt::Call { call: value, .. } => {
                    let (ExprKind::Method { recv, name, args } | ExprKind::MethodCall { recv, name, args, .. }) = &value.kind else {
                        continue;
                    };
                    if name.name != "push" {
                        continue;
                    }
                    let ExprKind::Var(name) = &recv.kind else {
                        continue;
                    };
                    let Some(LocalKind::Builder(id)) = scope.get(&name.name) else {
                        continue;
                    };
                    let element = match args.as_slice() {
                        [Arg::Pos(value)] => self.builder_element(value, state, &scope),
                        _ => None,
                    };
                    self.builders
                        .get_mut(id)
                        .expect("local builder")
                        .observe(element);
                }
                ComponentStmt::Assign(assign) => {
                    if let ExprKind::Var(name) = &assign.rhs.kind {
                        if let Some(LocalKind::Builder(id)) = scope.get(&name.name) {
                            self.targets
                                .entry(assign.field.clone())
                                .or_default()
                                .insert(*id);
                        }
                    }
                }
                ComponentStmt::If { then, els, .. } => {
                    self.collect(then, state, &scope);
                    if let Some(otherwise) = els {
                        self.collect(otherwise, state, &scope);
                    }
                }
                ComponentStmt::For {
                    name, iter, body, ..
                } => {
                    let mut inner = scope.clone();
                    inner.remove(&name.name);
                    if let Some(kind) = self
                        .value_kind(iter, state, &scope)
                        .and_then(|kind| self.list_element(kind))
                    {
                        inner.insert(name.name.clone(), kind);
                    }
                    self.collect(body, state, &inner);
                }
                ComponentStmt::While { body, .. } => self.collect(body, state, &scope),
                ComponentStmt::AssignLocal { .. } | ComponentStmt::Send { .. } | ComponentStmt::Focus { .. } => {}
                ComponentStmt::Try { body, catch, .. } => {
                    self.collect(body, state, &scope);
                    self.collect(catch, state, &scope);
                }
            }
        }
    }

    fn builder_element(&self, value: &Expr, state: &Scope, scope: &Scope) -> Option<BuilderElement> {
        if matches!(value.kind, ExprKind::ObjectLit { .. }) {
            return record_fields(value, state, scope, self).map(BuilderElement::Record);
        }
        let LocalKind::Scalar(word) = self.value_kind(value, state, scope)? else { return None };
        Some(BuilderElement::Scalar(match word {
            TypeWord::I32 => TypeWord::Int,
            // Lowercase str still has owned list storage. It selects the
            // legacy append operation that snapshots the source text.
            TypeWord::StrOwned => TypeWord::Str,
            word => word,
        }))
    }

    fn list_element(&self, kind: LocalKind) -> Option<LocalKind> {
        match kind {
            LocalKind::List(element) => Some(*element),
            LocalKind::Builder(id) => match self.builders.get(&id)?.element()? {
                BuilderElement::Scalar(word) => Some(LocalKind::Scalar(*word)),
                BuilderElement::Record(_) => None,
            },
            LocalKind::Scalar(_) | LocalKind::Record(_) => None,
        }
    }

    fn record_field(&self, kind: LocalKind, field: &str) -> Option<TypeWord> {
        let LocalKind::Record(record) = kind else {
            return None;
        };
        let column = self.record_fields.get(&record)?.get(field).copied();
        column.or_else(|| {
            crate::source::age_package::widget_lists::is_sparse_record(&record)
                .then_some(TypeWord::Int)
        })
    }

    fn value_kind(&self, value: &Expr, state: &Scope, scope: &Scope) -> Option<LocalKind> {
        let word = match &value.kind {
            ExprKind::IntLit(_) => TypeWord::Int,
            ExprKind::BoolLit(_) => TypeWord::Bool,
            ExprKind::StrLit(_) => TypeWord::Str,
            ExprKind::TypedLit { ty, .. } => *ty,
            ExprKind::Call { callee, .. } if callee.name == "__fstr" => TypeWord::Str,
            ExprKind::Var(name) => return scope.get(&name.name).cloned(),
            ExprKind::Field { base, name } if matches!(&base.kind, ExprKind::Var(id) if id.name == "self") => {
                return state.get(&name.name).cloned()
            }
            ExprKind::Field { base, name } => {
                self.record_field(self.value_kind(base, state, scope)?, &name.name)?
            }
            ExprKind::Index { base, index } => {
                let base = self.value_kind(base, state, scope)?;
                match &index.kind {
                    ExprKind::StrLit(key) => self.record_field(base, key)?,
                    _ => return self.list_element(base),
                }
            }
            ExprKind::IsExpr { arms, .. } => {
                let mut words = arms.iter().map(|arm| match &arm.body {
                    AggArmBody::Expr(value) => match self.value_kind(value, state, scope)? {
                        LocalKind::Scalar(word) => Some(word),
                        _ => None,
                    },
                    _ => None,
                });
                let first = words.next()??;
                if !words.all(|word| word == Some(first)) {
                    return None;
                }
                first
            }
            ExprKind::Unary { op: UnaryOp::Neg, operand } => return self.value_kind(operand, state, scope),
            ExprKind::Binary { op: BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div, lhs, rhs } => {
                let (LocalKind::Scalar(left), LocalKind::Scalar(right)) =
                    (self.value_kind(lhs, state, scope)?, self.value_kind(rhs, state, scope)?)
                else { return None };
                if left != right { return None; }
                left
            }
            _ => return None,
        };
        Some(LocalKind::Scalar(word))
    }
}

fn is_empty_list(value: &Expr) -> bool {
    matches!(&value.kind, ExprKind::ArrayLit(values) if values.is_empty())
}

fn annotation_kind(ann: &TypeAnn) -> Option<LocalKind> {
    match &ann.kind {
        TypeRef::Builtin(word) => Some(LocalKind::Scalar(*word)),
        TypeRef::Record(name) if name.name != "List" => Some(LocalKind::Record(name.name.clone())),
        TypeRef::Generic { name, args } if name.name == "List" => match args.as_slice() {
            [element] if matches!(element.kind, TypeRef::Builtin(_) | TypeRef::Record(_)) => {
                Some(LocalKind::List(Box::new(annotation_kind(element)?)))
            }
            _ => None,
        },
        _ => None,
    }
}

fn record_fields(
    value: &Expr,
    state: &Scope,
    scope: &Scope,
    facts: &HandlerLists,
) -> Option<BTreeMap<String, FieldDecl>> {
    let ExprKind::ObjectLit { fields } = &value.kind else {
        return None;
    };
    let mut row = BTreeMap::new();
    for field in fields {
        let ObjectKey::Text(name) = &field.key else {
            return None;
        };
        let mut chars = name.chars();
        if !chars
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
            || !chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        {
            return None;
        }
        let LocalKind::Scalar(word) = facts.value_kind(&field.value, state, scope)? else {
            return None;
        };
        let word = match word {
            TypeWord::I32 => TypeWord::Int,
            TypeWord::StrOwned => TypeWord::Str,
            word => word,
        };
        if row
            .insert(
                name.clone(),
                FieldDecl {
                    name: Ident {
                        name: name.clone(),
                        span: field.span,
                    },
                    ty: TypeAnn {
                        kind: TypeRef::Builtin(word),
                        span: field.value.span,
                    },
                    span: field.span,
                },
            )
            .is_some()
        {
            return None;
        }
    }
    (!row.is_empty()).then_some(row)
}

fn same_row(left: &BTreeMap<String, FieldDecl>, right: &BTreeMap<String, FieldDecl>) -> bool {
    left.len() == right.len()
        && left.iter().all(|(name, field)| {
            right
                .get(name)
                .is_some_and(|other| other.ty.kind == field.ty.kind)
        })
}

pub(super) fn intern_record(
    node: &Ident,
    span: Span,
    schema: Vec<FieldDecl>,
    records: &mut Vec<TypeItem>,
    sparse: bool,
) -> Ident {
    use crate::source::age_package::widget_lists::{is_sparse_record, SPARSE_RECORD_PREFIX};
    if let Some(record) = records.iter().find(|record| {
        is_sparse_record(&record.name.name) == sparse
            && record.fields.len() == schema.len()
            && record.fields.iter().zip(&schema).all(|(field, other)| {
                field.name.name == other.name.name && field.ty.kind == other.ty.kind
            })
    }) {
        return record.name.clone();
    }
    let prefix = if sparse { SPARSE_RECORD_PREFIX } else { "__age_p_hidden_record_" };
    let name = Ident { name: format!("{prefix}{}_{}", node.name, records.len()), span };
    records.push(TypeItem {
        name: name.clone(),
        fields: schema,
        type_params: Vec::new(),
        instance: None,
        conforms: Vec::new(),
        spec_memberships: Vec::new(),
        methods: Vec::new(),
        span,
    });
    name
}

pub(super) fn list_of(record: Ident, span: Span) -> TypeAnn {
    list_type(TypeRef::Record(record), span)
}

fn list_type(element: TypeRef, span: Span) -> TypeAnn {
    TypeAnn {
        kind: TypeRef::Generic {
            name: Ident {
                name: "List".into(),
                span,
            },
            args: vec![TypeAnn {
                kind: element,
                span,
            }],
        },
        span,
    }
}

fn annotate_builders(stmts: &mut [ComponentStmt], annotations: &BTreeMap<usize, TypeAnn>) {
    for statement in stmts {
        match statement {
            ComponentStmt::Let { name, ann, .. } => {
                if let Some(ty) = annotations.get(&name.span.start) {
                    *ann = Some(ty.clone());
                }
            }
            ComponentStmt::If { then, els, .. } => {
                annotate_builders(then, annotations);
                if let Some(otherwise) = els {
                    annotate_builders(otherwise, annotations);
                }
            }
            ComponentStmt::For { body, .. } | ComponentStmt::While { body, .. } => {
                annotate_builders(body, annotations)
            }
            _ => {}
        }
    }
}
