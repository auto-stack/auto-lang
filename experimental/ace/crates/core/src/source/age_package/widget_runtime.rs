//! Checked model initialization and transitions for an interactive widget closure.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    computed_fn, is_model_record_list, live_view, text_diag, widget_input_params,
    widget_prop_evaluator, widget_prop_type, widget_value_params, AgeWidgetProperty, AgeWidgetRef, AgeWidgetRefs,
    PackagedAgeComponent, ViewScope, AGE_PACKAGE_HIDDEN_PREFIX, WIDGET_CONTEXT_BINDING,
};
use crate::atom_text::{Diagnostic, Span};
use crate::source::age_component::{
    handler_check_fn, instance_stmts, local_name, model_type, outbound, rename_expr, watch_check_fn, widget_model_field,
    ComponentEntryValue, ComponentHandler, ComponentMemberRow, ComponentNode, ComponentScenario, ComponentStmt,
    ComponentViewElement, ComponentViewItem, FieldBinding,
};
use crate::source::ast::{
    Block, Expr, ExprKind, FieldDecl, FnItem, Ident, Item, NamedCtorField, Param, ParamMode,
    Receiver, Stmt, TypeAnn, TypeItem, TypeRef, TypeWord,
};
use crate::source::packages::{PackageDiagnostic, PackageSyntax};

#[derive(Clone, Debug)]
pub struct AgeWidgetRuntimeComponent {
    /// Index in the checked package's component description.
    pub node: usize,
    /// Source name of the initializer. It accepts widget value inputs in declaration order.
    pub initialize: Option<String>,
    /// Source name of the view that accepts widget value inputs and the current model, if any.
    pub view: String,
    pub handlers: Vec<AgeWidgetRuntimeHandler>,
    pub timers: Vec<AgeWidgetRuntimeTimer>,
    pub watches: Vec<AgeWidgetRuntimeWatch>,
    /// The model returned by a transition includes an ordered outbound queue.
    pub outbound: bool,
    pub focus: bool,
    /// Host-only transition for a mount token and logical viewport width.
    pub context_handler: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AgeWidgetRuntimeHandler {
    pub message: String,
    /// Source name of the function accepting widget inputs, previous state, then payload.
    pub function: String,
}

#[derive(Clone, Debug)]
pub struct AgeWidgetRuntimeWatch {
    pub sources: Vec<AgeWidgetWatchSource>,
    pub immediate: bool,
    pub function: String,
}

#[derive(Clone, Copy, Debug)]
pub enum AgeWidgetWatchSource {
    Property(usize),
    Model(usize),
}

#[derive(Clone, Debug)]
pub struct AgeWidgetRuntimeTimer {
    pub interval_ms: u32,
    pub message: String,
    /// Checked predicate accepting the same inputs and model as the view.
    pub condition: Option<String>,
}

pub(super) fn timer_guard(
    node: &ComponentNode,
    name: &str,
    value: &Expr,
    binding: FieldBinding<'_>,
    params: &[Param],
) -> Result<Item, Vec<Diagnostic>> {
    let scope = ViewScope::with_binding(name, node, binding, &[])?;
    let Item::Fn(mut guard) = computed_fn(name.to_owned(), scope.rename(value)?) else {
        unreachable!("a timer guard is a function");
    };
    guard.params = params.to_vec();
    guard.ret = TypeAnn {
        kind: TypeRef::Builtin(TypeWord::Bool),
        span: value.span,
    };
    Ok(Item::Fn(guard))
}

pub(super) fn timer_declarations(
    node: &ComponentNode,
) -> Result<Vec<(&ComponentMemberRow, u32)>, Vec<Diagnostic>> {
    node.members.iter().filter(|entry| entry.role == "timer").map(|entry| {
        let interval_ms = entry.num
            .and_then(widget_timer_interval_ms)
            .ok_or_else(|| vec![text_diag(
                "source.age.timer-interval", "every_ms must be between 1 and 2147483647", entry.span,
            )])?;
        if !node.messages.iter().any(|msg| msg.name == entry.name && msg.payload.is_empty())
            || !node.handlers.iter().any(|handler| handler.msg == entry.name)
        {
            return Err(vec![text_diag(
                "source.age.timer-message",
                format!("timer requires a declared message without payload and a handler: `{}`", entry.name),
                entry.span,
            )]);
        }
        Ok((entry, interval_ms))
    }).collect()
}

/// Match the host timer contract's minimum polling interval and the web host's upper bound.
pub fn widget_timer_interval_ms(value: i64) -> Option<u32> {
    let interval = u32::try_from(value).ok()?;
    (1..=i32::MAX as u32).contains(&interval).then_some(interval.max(16))
}

fn runtime_timers(
    node: &ComponentNode,
    index: usize,
    binding: FieldBinding<'_>,
    params: &[Param],
    items: &mut Vec<Item>,
) -> Result<Vec<AgeWidgetRuntimeTimer>, Vec<Diagnostic>> {
    let mut timers = Vec::new();
    for (entry, interval_ms) in timer_declarations(node)? {
        let condition = if let Some(value) = &entry.value {
            let name = format!("{AGE_PACKAGE_HIDDEN_PREFIX}runtime_timer_{index}_{}", timers.len());
            items.push(timer_guard(node, &name, value, binding, params)?);
            Some(name)
        } else {
            None
        };
        timers.push(AgeWidgetRuntimeTimer { interval_ms, message: entry.name.clone(), condition });
    }
    Ok(timers)
}

pub(crate) fn add_widget_runtime_state(
    components: &[PackagedAgeComponent],
    units: &mut BTreeMap<String, PackageSyntax>,
    refs: &mut AgeWidgetRefs,
    entry: &str,
) -> Result<Vec<AgeWidgetRuntimeComponent>, PackageDiagnostic> {
    let Some(root) = components
        .iter()
        .position(|component| component.module == entry)
    else {
        return Ok(Vec::new());
    };
    let mut pending = vec![root];
    let mut active = BTreeSet::new();
    while let Some(index) = pending.pop() {
        if !active.insert(index) {
            continue;
        }
        let component = &components[index];
        let key = (component.module.clone(), component.node.name.name.clone());
        for site in refs.get(&key).into_iter().flatten() {
            if let Some(child) = components.iter().position(|candidate| {
                candidate.module == site.module && candidate.node.name.name == site.node
            }) {
                pending.push(child);
            }
        }
    }
    active
        .into_iter()
        .map(|index| {
            let component = &components[index];
            let key = (component.module.clone(), component.node.name.name.clone());
            let program = &mut units
                .get_mut(&component.module)
                .expect("bound widget module")
                .program;
            let locate = |mut diagnostics: Vec<Diagnostic>| PackageDiagnostic {
                module: component.module.clone(),
                diagnostic: diagnostics.remove(0),
            };
            let sites = refs.get(&key).map_or(&[][..], Vec::as_slice);
            let result =
                add_model(index, &component.node, sites, &mut program.items).map_err(locate)?;
            if let Some(sites) = refs.get_mut(&key) {
                add_runtime_properties(
                    index,
                    &component.node,
                    sites,
                    components,
                    &mut program.items,
                )
                .map_err(locate)?;
            }
            Ok(result)
        })
        .collect()
}

fn add_runtime_properties(
    index: usize,
    node: &ComponentNode,
    sites: &mut [AgeWidgetRef],
    components: &[PackagedAgeComponent],
    items: &mut Vec<Item>,
) -> Result<(), Vec<Diagnostic>> {
    let Some(model) = runtime_model_param(node, index) else {
        return Ok(());
    };
    let mut elements = BTreeMap::new();
    record_elements(&node.view.tree, &mut elements);
    for (_, tree) in &node.view.named_trees {
        record_elements(tree, &mut elements);
    }
    for site in sites {
        let element = elements[&site.span.start];
        let target = components
            .iter()
            .find(|component| {
                component.module == site.module && component.node.name.name == site.node
            })
            .expect("widget site has a bound declaration");
        for property in &mut site.properties {
            let AgeWidgetProperty::Value {
                name,
                evaluators,
                authored_default: false,
                runtime_evaluator,
                ..
            } = property
            else {
                continue;
            };
            let param = target
                .node
                .params
                .iter()
                .find(|param| param.name == *name)
                .expect("property has a declaration");
            let ty = widget_prop_type(param).expect("property type already checked");
            let expression = element
                .items
                .iter()
                .find_map(|item| match item {
                    ComponentViewItem::Entry(entry) if entry.key == *name => match &entry.value {
                        ComponentEntryValue::Expr(value) => Some(value),
                        _ => None,
                    },
                    _ => None,
                })
                .expect("provided property has an authored expression");
            let symbol = format!("{}_runtime", evaluators[0]);
            let binding = FieldBinding::runtime_at(index, node, &model.name);
            let scope = ViewScope::with_binding(&symbol, node, binding, &[])?;
            let value = if is_model_record_list(&ty, expression) {
                expression.clone()
            } else {
                scope.rename(expression)?
            };
            let mut function = widget_prop_evaluator(symbol.clone(), value, &ty, false);
            function.params = widget_input_params(node);
            function.params.push(model.clone());
            items.push(Item::Fn(function));
            *runtime_evaluator = Some(symbol);
        }
    }
    Ok(())
}

fn record_elements<'a>(
    items: &'a [ComponentViewItem],
    found: &mut BTreeMap<usize, &'a ComponentViewElement>,
) {
    for item in items {
        match item {
            ComponentViewItem::Element(element) => {
                found.insert(element.span.start, element);
                record_elements(&element.items, found);
            }
            ComponentViewItem::Cond(branch) => {
                record_elements(&branch.then_items, found);
                record_elements(&branch.else_items, found);
            }
            ComponentViewItem::For(iteration) => record_elements(&iteration.items, found),
            _ => {}
        }
    }
}

pub(super) fn unit_callback<'a>(node: &ComponentNode, handler: &'a ComponentHandler) -> Option<&'a str> {
    let [call] = handler.channel_calls.as_slice() else {
        return None;
    };
    (!handler.body_foreign
        && handler.stmts.is_empty()
        && handler.params.is_empty()
        && handler.msg != "Init"
        && call.args.is_empty()
        && node.messages.iter().any(|msg| msg.name == handler.msg && msg.payload.is_empty())
        && node.params.iter().any(|param| param.name == call.channel && param.ty == "msg"))
        .then_some(call.channel.as_str())
}

fn add_model(
    index: usize,
    node: &ComponentNode,
    sites: &[AgeWidgetRef],
    items: &mut Vec<Item>,
) -> Result<AgeWidgetRuntimeComponent, Vec<Diagnostic>> {
    // PLAN-118（T-03）：惰性 view-data 字段（VM 轨 `var px = []` 一类）不再
    // 判死 live 视图——`live_view::build_view` 的 ViewScope 已按只读字面量
    // 内联 bare view-data 名（`px42=${.px[42]}` 真语料形）；仍要求普通 view 出口。
    if node.view.exit != "view" {
        return Err(vec![text_diag(
            "source.age.widget-runtime-view",
            "a checked widget runtime requires an ordinary widget view",
            node.view.span,
        )]);
    }
    let view = format!("{AGE_PACKAGE_HIDDEN_PREFIX}runtime_view_{index}");
    let mut output = AgeWidgetRuntimeComponent {
        node: index,
        initialize: None,
        view,
        handlers: Vec::new(),
        timers: Vec::new(),
        watches: Vec::new(),
        outbound: outbound::has_messages(node),
        focus: outbound::has_focus(node),
        context_handler: context_handler(node)?,
    };
    let model_param = runtime_model_param(node, index);
    let binding = model_param.as_ref().map_or_else(
        || FieldBinding::state_at(index, 0, node),
        |model| FieldBinding::runtime_at(index, node, &model.name),
    );
    let mut view_params = widget_input_params(node);
    view_params.extend(model_param.iter().cloned());
    items.extend(live_view::build_view(
        node,
        &output.view,
        sites,
        binding,
        view_params.clone(),
    )?);
    output.timers = runtime_timers(node, index, binding, &view_params, items)?;
    if model_param.is_some() {
        for member in &node.members {
            if member.role == "computed" {
                if let Some(value) = &member.value {
                    let value = rename_expr(value, binding, &node.fields)?;
                    let Item::Fn(mut function) =
                        computed_fn(runtime_computed_name(index, &member.name), value)
                    else {
                        unreachable!("computed projection emits a function");
                    };
                    function.params = view_params.clone();
                    items.push(Item::Fn(function));
                }
            }
        }
    }
    if node.fields.is_empty() && node.handlers.iter().all(|handler| unit_callback(node, handler).is_some()) {
        return Ok(output);
    }
    let prefix = format!("{AGE_PACKAGE_HIDDEN_PREFIX}runtime_");
    let span = node.head_span;
    let Some(model_param) = model_param else {
        return Err(vec![text_diag(
            "source.age.widget-runtime-handler",
            "a widget with handlers must declare a model",
            span,
        )]);
    };
    let model_type_ann = model_param.ty.clone();
    let TypeRef::Record(model) = &model_type_ann.kind else {
        unreachable!("widget model is a record")
    };
    let queues = [(output.outbound, outbound::FIELD), (output.focus, outbound::FOCUS_FIELD)]
        .into_iter().filter_map(|(used, field)| used.then_some(field)).collect::<Vec<_>>();
    items.push(Item::Type(model_record(node, model, &queues)?));

    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    let mut initial = instance_stmts(0, index, node, &scenario, &Default::default(), true)?;
    for &field in &queues {
        initial.push(outbound::empty_queue(field, span));
    }
    initial.push(return_model(node, model, 0, &queues));
    let initialize = format!("{prefix}init_{index}");
    items.push(Item::Fn(FnItem {
        name: ident(&initialize, span),
        type_params: Vec::new(),
        params: widget_input_params(node),
        ret: model_type_ann.clone(),
        body: Block {
            stmts: initial,
            span,
        },
        span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
        proto: false,
        test_attribute: None,
    }));
    output.initialize = Some(initialize);

    let handler_prefix = format!("{prefix}handler_");
    for (ordinal, handler) in node.handlers.iter().enumerate() {
        if unit_callback(node, handler).is_some() {
            continue;
        }
        let unsupported = || {
            vec![text_diag(
                "source.age.widget-runtime-handler",
                format!(
                    "cannot emit a checked widget transition for handler `{}`",
                    handler.msg
                ),
                handler.span,
            )]
        };
        if !handler.channel_calls.is_empty() {
            return Err(unsupported());
        }
        let Some(Item::Fn(function)) =
            handler_check_fn(node, index, ordinal, &handler_prefix, true).transpose()?
        else {
            return Err(unsupported());
        };
        let function = model_transition(node, &model_param, model, &queues, function)
            .ok_or_else(unsupported)?;
        output.handlers.push(AgeWidgetRuntimeHandler {
            message: handler.msg.clone(),
            function: function.name.name.clone(),
        });
        items.push(Item::Fn(function));
    }
    for (ordinal, row) in node.members.iter().enumerate().filter(|(_, row)| row.watch_body.is_some()) {
        let (sources, immediate) = watch_sources(node, row)?;
        let Item::Fn(function) = watch_check_fn(node, index, ordinal, "__age_p_hidden_runtime_watch_", row)? else {
            unreachable!("watch projection emits a function");
        };
        let function = model_transition(node, &model_param, model, &queues, function)
            .ok_or_else(|| vec![text_diag("source.age.watch-model", "invalid checked watch transition", row.span)])?;
        output.watches.push(AgeWidgetRuntimeWatch { sources, immediate, function: function.name.name.clone() });
        items.push(Item::Fn(function));
    }
    Ok(output)
}

fn context_handler(node: &ComponentNode) -> Result<Option<String>, Vec<Diagnostic>> {
    let mut rows = node.members.iter()
        .filter(|row| row.role == "bind" && row.name == WIDGET_CONTEXT_BINDING);
    let Some(row) = rows.next() else { return Ok(None) };
    let fail = || vec![text_diag("source.age.widget-context",
        "widget:context must bind one handler that stores the str token and int width in model fields",
        row.span)];
    if rows.next().is_some() {
        return Err(fail());
    }
    let name = row.text.as_ref().ok_or_else(fail)?;
    let handler = node.handlers.iter().find(|handler| &handler.msg == name).ok_or_else(fail)?;
    let Some(message) = node.messages.iter().find(|message| &message.name == name) else {
        return Err(fail());
    };
    if handler.body_foreign || !handler.channel_calls.is_empty() || message.payload != ["str", "int"]
        || handler.stmts.len() != 2 || handler.params.len() != 2
    {
        return Err(fail());
    }
    for ((param, _), ty) in handler.params.iter().zip(["str", "int"]) {
        let stores = handler.stmts.iter().any(|stmt| {
            let ComponentStmt::Assign(assign) = stmt else { return false };
            matches!(&assign.rhs.kind, ExprKind::Var(value) if &value.name == param)
                && node.fields.iter().any(|field| field.name == assign.field && field.mutable && field.ty == ty)
        });
        if !stores {
            return Err(fail());
        }
    }
    Ok(Some(name.clone()))
}

fn watch_sources(
    node: &ComponentNode,
    row: &ComponentMemberRow,
) -> Result<(Vec<AgeWidgetWatchSource>, bool), Vec<Diagnostic>> {
    let mut immediate = false;
    let mut sources = Vec::new();
    for (source, &span) in row.sources.iter().zip(&row.source_spans) {
        let name = match source.split_once('.') {
            Some((name, "immediate")) => {
                immediate = true;
                name
            }
            // Typed widget values compare by content across frame publications.
            Some((name, "deep")) => name,
            Some((_, modifier)) => return Err(vec![text_diag(
                "source.age.watch-modifier", format!("unsupported watch modifier `{modifier}`"), span,
            )]),
            None => source,
        };
        if let Some(index) = node.fields.iter().position(|field| field.name == name) {
            sources.push(AgeWidgetWatchSource::Model(index));
        } else {
            let index = widget_value_params(node).iter()
                .filter(|param| widget_prop_type(param).is_some())
                .position(|param| param.name == name)
                .ok_or_else(|| vec![text_diag(
                    "source.age.member-field", format!("unknown watch source `{name}`"), span,
                )])?;
            sources.push(AgeWidgetWatchSource::Property(index));
        }
    }
    Ok((sources, immediate))
}

fn model_transition(
    node: &ComponentNode,
    model_param: &Param,
    model: &Ident,
    queues: &[&str],
    mut function: FnItem,
) -> Option<FnItem> {
    let mut args = widget_input_params(node);
    args.push(model_param.clone());
    function.params.splice(..0, args);
    // Keep the checked body between a snapshot of the prior model and a
    // whole-model return.
    for (stmt, field) in function.body.stmts.iter_mut().zip(&node.fields) {
        let Stmt::Let { name, ann: Some(ty), value, .. } = stmt else { return None };
        if name.name != local_name(0, &field.name) {
            return None;
        }
        *value = widget_model_field(
            &model_param.name, &field.name, field.span, field.span,
            ty.kind == TypeRef::Builtin(TypeWord::StrOwned),
        );
    }
    let last @ Stmt::Return { .. } = function.body.stmts.last_mut()? else { return None };
    *last = return_model(node, model, 0, queues);
    function.ret = model_param.ty.clone();
    Some(function)
}

pub(super) fn runtime_model_param(node: &ComponentNode, index: usize) -> Option<Param> {
    let span = node.head_span;
    (!node.fields.is_empty()).then(|| Param {
        name: ident(format!("{AGE_PACKAGE_HIDDEN_PREFIX}runtime_model"), span),
        ty: TypeAnn {
            kind: TypeRef::Record(ident(
                format!("{AGE_PACKAGE_HIDDEN_PREFIX}runtime_model_{index}"),
                span,
            )),
            span,
        },
        mode: ParamMode::Default,
        default: None,
        span,
    })
}

pub(crate) fn runtime_computed_name(index: usize, field: &str) -> String {
    format!("{AGE_PACKAGE_HIDDEN_PREFIX}runtime_computed_{index}_{field}")
}

pub(super) fn model_record(node: &ComponentNode, model: &Ident, queues: &[&str]) -> Result<TypeItem, Vec<Diagnostic>> {
    let mut fields = node.fields.iter().map(|field| {
        Ok(FieldDecl {
            name: ident(&field.name, field.span),
            ty: model_type(field, true)?,
            span: field.span,
        })
    }).collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
    for &field in queues {
        fields.push(FieldDecl { name: ident(field, model.span), ty: outbound::queue_type(model.span), span: model.span });
    }
    Ok(TypeItem {
        name: model.clone(),
        type_params: Vec::new(),
        instance: None,
        conforms: Vec::new(),
        spec_memberships: Vec::new(),
        fields,
        methods: Vec::new(),
        span: model.span,
    })
}

pub(super) fn return_model(node: &ComponentNode, model: &Ident, instance: usize, queues: &[&str]) -> Stmt {
    let mut fields: Vec<_> = node
        .fields
        .iter()
        .map(|field| NamedCtorField {
            name: ident(&field.name, field.span),
            value: variable(&ident(local_name(instance, &field.name), field.span)),
            span: field.span,
        })
        .collect();
    for &field in queues {
        let name = ident(field, model.span);
        fields.push(NamedCtorField {
            value: variable(&name),
            name,
            span: model.span,
        });
    }
    Stmt::Return {
        span: model.span,
        value: Expr {
            span: model.span,
            kind: ExprKind::CtorNamed {
                ty: model.clone(),
                args: Vec::new(),
                fields,
            },
        },
    }
}

fn ident(name: impl Into<String>, span: Span) -> Ident {
    Ident {
        name: name.into(),
        span,
    }
}

fn variable(name: &Ident) -> Expr {
    Expr {
        span: name.span,
        kind: ExprKind::Var(name.clone()),
    }
}
