//! Checked model initialization and transitions for an interactive widget closure.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    computed_fn, is_model_record_list, live_view, text_diag, widget_input_params,
    widget_prop_evaluator, widget_prop_type, AgeWidgetProperty, AgeWidgetRef, AgeWidgetRefs,
    PackagedAgeComponent, ViewScope, AGE_PACKAGE_HIDDEN_PREFIX,
};
use crate::atom_text::{Diagnostic, Span};
use crate::source::age_component::{
    handler_check_fn, instance_stmts, local_name, model_type, rename_expr, widget_model_field,
    ComponentEntryValue, ComponentHandler, ComponentNode, ComponentScenario, ComponentViewElement,
    ComponentViewItem, FieldBinding,
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
}

#[derive(Clone, Debug)]
pub struct AgeWidgetRuntimeHandler {
    pub message: String,
    /// Source name of the function accepting widget inputs, previous state, then payload.
    pub function: String,
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
    if node.view.exit != "view" || !node.view_data.is_empty() {
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
    };
    let model_param = runtime_model_param(node, index);
    let binding = model_param.as_ref().map_or_else(
        || FieldBinding::state_at(index, 0, node),
        |model| FieldBinding::runtime_at(index, node, &model.name),
    );
    let mut view_params = widget_input_params(node);
    view_params.extend(model_param.iter().cloned());
    items.push(Item::Fn(live_view::build_view(
        node,
        &output.view,
        sites,
        binding,
        view_params.clone(),
    )?));
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
    items.push(Item::Type(model_record(node, model)?));

    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    let mut initial = instance_stmts(0, index, node, &scenario, &Default::default(), true)?;
    initial.push(return_model(node, model, 0));
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
        let Some(Item::Fn(mut function)) =
            handler_check_fn(node, index, ordinal, &handler_prefix, true).transpose()?
        else {
            return Err(unsupported());
        };
        let mut args = widget_input_params(node);
        args.push(model_param.clone());
        function.params.splice(..0, args);
        // The check function starts with one declaration per model field and
        // ends with a projection. The transition uses the same handler body
        // between a snapshot of the prior model and a whole-model return.
        for (stmt, field) in function.body.stmts.iter_mut().zip(&node.fields) {
            let Stmt::Let {
                name,
                ann: Some(ty),
                value,
                ..
            } = stmt
            else {
                return Err(unsupported());
            };
            if name.name != local_name(0, &field.name) {
                return Err(unsupported());
            }
            *value = widget_model_field(
                &model_param.name,
                &field.name,
                field.span,
                field.span,
                ty.kind == TypeRef::Builtin(TypeWord::StrOwned),
            );
        }
        let Some(last @ Stmt::Return { .. }) = function.body.stmts.last_mut() else {
            return Err(unsupported());
        };
        *last = return_model(node, model, 0);
        function.ret = model_type_ann.clone();
        output.handlers.push(AgeWidgetRuntimeHandler {
            message: handler.msg.clone(),
            function: function.name.name.clone(),
        });
        items.push(Item::Fn(function));
    }
    Ok(output)
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

pub(super) fn model_record(node: &ComponentNode, model: &Ident) -> Result<TypeItem, Vec<Diagnostic>> {
    let fields = node.fields.iter().map(|field| {
        Ok(FieldDecl {
            name: ident(&field.name, field.span),
            ty: model_type(field, true)?,
            span: field.span,
        })
    }).collect::<Result<_, Vec<Diagnostic>>>()?;
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

pub(super) fn return_model(node: &ComponentNode, model: &Ident, instance: usize) -> Stmt {
    let fields = node
        .fields
        .iter()
        .map(|field| NamedCtorField {
            name: ident(&field.name, field.span),
            value: variable(&ident(local_name(instance, &field.name), field.span)),
            span: field.span,
        })
        .collect();
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
