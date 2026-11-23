//! Executable widget view output. The authored control flow runs in the checked
//! package; renderers consume the resulting element, property and message rows.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    resolve_tick_spelling, text_diag, widget_input_params, widget_runtime, AgeWidgetProperty,
    AgeWidgetRef, AgeWidgetRefs, Arg, BinOp,
    Block, ComponentNode, ComponentViewEntry, ComponentViewFor, ComponentViewItem, Diagnostic,
    Expr, ExprKind, FnItem, Ident, Item, PackagedAgeComponent, Receiver, Span, Stmt, TypeAnn,
    TypeRef, TypeWord, ViewScope, AGE_PACKAGE_HIDDEN_PREFIX, FILE_LIST_BINDING, FILE_MUTATE_BINDING,
    FILE_OPEN_BINDING, TODO_SERVICE_BINDING, WIDGET_CONTEXT_BINDING,
};
use crate::source::{
    age_component::{
        instance_stmts, payload_type, ComponentClassPart, ComponentEntryValue, ComponentScenario,
        FieldBinding,
    },
    ast::{AssignTarget, Param, ParamMode},
    packages::{PackageDiagnostic, PackageSyntax},
};

pub fn widget_view_fn_name_at(node: usize, instance: usize) -> String {
    format!("{AGE_PACKAGE_HIDDEN_PREFIX}widget_view_{node}_{instance}")
}

pub fn widget_timer_fn_name_at(node: usize, instance: usize, timer: usize) -> String {
    format!("{AGE_PACKAGE_HIDDEN_PREFIX}widget_timer_{node}_{instance}_{timer}")
}

/// Input supplied by the native image surface when the event fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSurfaceEventPayload {
    Wheel,
    Pan,
}

impl ImageSurfaceEventPayload {
    pub fn from_binding(tag: &str, key: &str) -> Option<Self> {
        if !matches!(tag, "image_surface" | "image-surface" | "imagesurface") {
            return None;
        }
        match key {
            "onwheel" => Some(Self::Wheel),
            "onpan" => Some(Self::Pan),
            _ => None,
        }
    }

    pub(super) fn matches_arguments(self, args: &[Arg]) -> bool {
        match (self, args) {
            (Self::Wheel, [Arg::Pos(value)]) => {
                matches!(&value.kind, ExprKind::Field { base, name }
                    if name.name == "deltaY"
                        && matches!(&base.kind, ExprKind::Var(name) if name.name == "nevent"))
            }
            (Self::Pan, [Arg::Pos(dx), Arg::Pos(dy), Arg::Pos(phase)]) => {
                [dx, dy, phase].into_iter().zip(["n0", "n1", "n2"]).all(|(value, slot)|
                    matches!(&value.kind, ExprKind::Var(name) if name.name == slot))
            }
            _ => false,
        }
    }

    pub(super) fn matches_types(self, payload: &[String]) -> bool {
        let expected: &[TypeWord] = match self {
            Self::Wheel => &[TypeWord::Float],
            Self::Pan => &[TypeWord::Float, TypeWord::Float, TypeWord::StrOwned],
        };
        payload.iter().map(|ty| payload_type(ty)).eq(expected.iter().copied().map(Some))
    }
}

pub(super) fn add_widget_views(
    components: &[PackagedAgeComponent],
    units: &mut BTreeMap<String, PackageSyntax>,
    refs: &AgeWidgetRefs,
    entry: &str,
    scenario: &ComponentScenario,
) -> Result<(), PackageDiagnostic> {
    let targets: BTreeSet<_> = refs
        .values()
        .flatten()
        .map(|widget| (widget.module.as_str(), widget.node.as_str()))
        .collect();
    let instances = scenario.instances;
    let entry_node = components.iter().position(|component| component.module == entry);
    for (index, component) in components.iter().enumerate() {
        let node = &component.node;
        let sites = refs.get(&(component.module.clone(), node.name.name.clone()));
        let is_live_root = instances != 0 && Some(index) == entry_node;
        let handles_root_timers = is_live_root && node.members.iter().any(|row| row.role == "timer");
        let has_host_bindings = is_live_root && node.members.iter().any(|row| {
            row.role == "bind" && (matches!(row.name.as_str(), FILE_LIST_BINDING | FILE_MUTATE_BINDING | FILE_OPEN_BINDING | TODO_SERVICE_BINDING | WIDGET_CONTEXT_BINDING)
                || row.name.trim_matches('"').starts_with("storage:"))
        });
        if node.view.exit != "view"
            || !node.view_data.is_empty()
            || (!handles_root_timers && !has_host_bindings && sites.is_none()
                && !targets.contains(&(component.module.as_str(), node.name.name.as_str())))
        {
            continue;
        }
        let sites = sites.map_or(&[][..], Vec::as_slice);
        let program = &mut units
            .get_mut(&component.module)
            .expect("widget module")
            .program;
        add_component_views(node, index, sites, scenario, handles_root_timers, &mut program.items)
            .map_err(|mut diagnostics| PackageDiagnostic {
                module: component.module.clone(),
                diagnostic: diagnostics.remove(0),
            })?;
    }
    Ok(())
}

fn add_component_views(
    node: &ComponentNode,
    index: usize,
    sites: &[AgeWidgetRef],
    scenario: &ComponentScenario,
    handles_root_timers: bool,
    items: &mut Vec<Item>,
) -> Result<(), Vec<Diagnostic>> {
    let span = node.head_span;
    let model = ident(&format!("{AGE_PACKAGE_HIDDEN_PREFIX}view_model"), span);
    let record = ident(&format!("{AGE_PACKAGE_HIDDEN_PREFIX}view_model_{index}"), span);
    let snapshot = node.legacy_widget
        && !node.fields.is_empty()
        && !node.members.iter().any(|member| member.role == "computed");
    let scenario = resolve_tick_spelling(scenario, node);
    if snapshot {
        items.push(Item::Type(widget_runtime::model_record(node, &record, &[])?));
    }
    let timers = if handles_root_timers {
        widget_runtime::timer_declarations(node)?
    } else {
        Vec::new()
    };
    for instance in 0..scenario.instances.max(1) {
        let symbol = widget_view_fn_name_at(index, instance);
        let params = widget_input_params(node);
        let state = FieldBinding::state_at(index, instance, node);
        let binding = if snapshot { FieldBinding::runtime_at(index, node, &model) } else { state };
        let mut view_params = params.clone();
        if snapshot {
            view_params.push(Param {
                name: model.clone(),
                ty: TypeAnn { kind: TypeRef::Record(record.clone()), span },
                mode: ParamMode::Default,
                default: None,
                span,
            });
        }
        let mut views = build_view(node, &symbol, sites, binding, view_params)?;
        let Some(Item::Fn(mut view)) = views.pop() else {
            unreachable!("checked view builder ends with the entry function");
        };
        items.extend(views);
        for (timer, (row, _)) in timers.iter().enumerate() {
            if let Some(value) = &row.value {
                let name = widget_timer_fn_name_at(index, instance, timer);
                items.push(widget_runtime::timer_guard(node, &name, value, state, &params)?);
            }
        }
        if snapshot {
            // Replay the checked model once per view call. The view reads the
            // resulting fields, so adding a read does not duplicate the handlers.
            let replay = ident(&format!("{symbol}_replay"), span);
            let ty = TypeAnn { kind: TypeRef::Record(record.clone()), span };
            let mut stmts = instance_stmts(instance, index, node, &scenario, &Default::default(), true)?;
            stmts.push(widget_runtime::return_model(node, &record, instance, &[]));
            items.push(Item::Fn(FnItem {
                name: replay.clone(),
                type_params: Vec::new(),
                params,
                ret: ty.clone(),
                body: Block { stmts, span },
                span,
                receiver: Receiver::Free,
                owner: None,
                script: false,
                proto: false,
                test_attribute: None,
            }));
            view.params.pop();
            view.body.stmts.insert(0, Stmt::Let {
                kw: span,
                span,
                name: model.clone(),
                ann: Some(ty),
                mutable: false,
                value: Expr { span, kind: ExprKind::Call { callee: replay, args: state.widget_args(span) } },
            });
        }
        items.push(Item::Fn(view));
    }
    Ok(())
}

pub(super) fn build_view(
    node: &ComponentNode,
    symbol: &str,
    sites: &[AgeWidgetRef],
    binding: FieldBinding<'_>,
    params: Vec<Param>,
) -> Result<Vec<Item>, Vec<Diagnostic>> {
    let span = node.head_span;
    let scope = ViewScope::with_binding(symbol, node, binding, &[])?;
    let out = ident(&format!("{symbol}_output"), span);
    let ret = TypeAnn {
        span,
        kind: TypeRef::Generic {
            name: ident("List", span),
            args: vec![TypeAnn {
                kind: TypeRef::Builtin(TypeWord::StrOwned),
                span,
            }],
        },
    };
    let mut stmts = vec![Stmt::Let {
        kw: span,
        span,
        name: out.clone(),
        ann: Some(ret.clone()),
        mutable: false,
        value: Expr {
            kind: ExprKind::ArrayLit(Vec::new()),
            span,
        },
    }];
    let mut build = ViewBody {
        symbol,
        out: &out,
        loops: 0,
        cells: 0,
        sites,
        params: &params,
        output_ty: &ret,
        emitted_cells: 0,
        fragments: Vec::new(),
    };
    build.items(&node.view.tree, &scope, &mut stmts, "", false)?;
    for timer in node.members.iter().filter(|member| member.role == "timer") {
        let period = timer.num.filter(|ms| (1..=i64::from(i32::MAX)).contains(ms))
            .ok_or_else(|| view_error("a timer outside the host interval range", timer.span))?;
        let mut enabled = Vec::new();
        build.emit(&mut enabled, "timer", [
            literal(&timer.name, timer.span),
            literal(&period.to_string(), timer.span),
        ]);
        if let Some(when) = &timer.value {
            stmts.push(if_stmt(scope.rename(when)?, enabled, Vec::new(), timer.span));
        } else {
            stmts.append(&mut enabled);
        }
    }
    stmts.push(Stmt::Return {
        value: read(&out),
        span,
    });
    let mut functions = build.fragments;
    functions.push(Item::Fn(FnItem {
        name: ident(symbol, span),
        type_params: Vec::new(),
        params,
        ret,
        body: Block {
            stmts,
            span: node.view.span,
        },
        span: node.span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
        proto: false,
        test_attribute: None,
    }));
    Ok(functions)
}

// Split ordered tape appends before large views accumulate a wide MIR frame.
// A fragment uses the same checked inputs and the same output list as its caller.
const VIEW_FRAGMENT_CELLS: usize = 96;

struct ViewBody<'a> {
    symbol: &'a str,
    out: &'a Ident,
    loops: usize,
    cells: usize,
    sites: &'a [AgeWidgetRef],
    params: &'a [Param],
    output_ty: &'a TypeAnn,
    emitted_cells: usize,
    fragments: Vec<Item>,
}

impl ViewBody<'_> {
    fn items(
        &mut self,
        items: &[ComponentViewItem],
        scope: &ViewScope<'_>,
        stmts: &mut Vec<Stmt>,
        element_tag: &str,
        skip_entries: bool,
    ) -> Result<(), Vec<Diagnostic>> {
        let mut start = stmts.len();
        let mut cells_before = self.emitted_cells;
        for item in items {
            match item {
                ComponentViewItem::Text(text) => {
                    self.emit(stmts, "text", [display(scope.rename(&text.expr)?)])
                }
                ComponentViewItem::Element(element) => {
                    let span = element.span;
                    let site = self.sites.iter().find(|site| site.span == span);
                    if scope.node.legacy_widget && site.is_none() && element.tag == "memo" {
                        if element.label.is_some() {
                            return Err(view_error("a label on a memo block", span));
                        }
                        let mut deps = false;
                        for item in &element.items {
                            let ComponentViewItem::Entry(entry) = item else { continue };
                            let literal_list = matches!(
                                &entry.value,
                                ComponentEntryValue::Expr(Expr { kind: ExprKind::ArrayLit(items), .. })
                                    if items.iter().all(|item| matches!(item.kind, ExprKind::StrLit(_)))
                            );
                            if entry.key != "deps" || !literal_list || deps {
                                return Err(view_error("this memo block binding", entry.span));
                            }
                            deps = true;
                        }
                        // The checked view already reevaluates after model changes.
                        // Keep the children and ignore the legacy VM cache hint.
                        self.items(&element.items, scope, stmts, element_tag, true)?;
                        continue;
                    }
                    let tag = site.map_or(element.tag.as_str(), |site| site.authored_node.as_str());
                    self.emit(
                        stmts,
                        "element",
                        [
                            literal(tag, span),
                            literal(&span.start.to_string(), span),
                        ],
                    );
                    if let Some(label) = &element.label {
                        self.emit(stmts, "label", [display(scope.rename(label)?)]);
                    }
                    // Closed child bindings use the package's typed property
                    // getters. Scoped scalar bindings are evaluated in place
                    // while the current row and index are live.
                    let closed = site.is_some_and(|site| {
                        !site.properties.iter().any(|property| {
                            matches!(property, AgeWidgetProperty::ScopedValue { .. })
                        })
                    });
                    self.items(&element.items, scope, stmts, &element.tag, closed)?;
                    self.emit(stmts, "end", [literal(tag, span)]);
                }
                ComponentViewItem::Entry(entry) if !skip_entries => {
                    self.entry(entry, element_tag, scope, stmts)?
                }
                ComponentViewItem::Entry(_) => {}
                ComponentViewItem::Cond(branch) => {
                    let mut then = Vec::new();
                    let mut els = Vec::new();
                    self.items(&branch.then_items, scope, &mut then, element_tag, false)?;
                    self.items(&branch.else_items, scope, &mut els, element_tag, false)?;
                    stmts.push(if_stmt(scope.rename(&branch.cond)?, then, els, branch.span));
                }
                ComponentViewItem::For(iteration) => {
                    self.iteration(iteration, scope, stmts, element_tag)?;
                }
            }
            // Loop bodies reference generated row/index locals. Keep those
            // scopes intact and split around the complete loop instead.
            if scope.visible.is_empty() && self.emitted_cells - cells_before >= VIEW_FRAGMENT_CELLS {
                let body = stmts.split_off(start);
                stmts.push(self.fragment(body));
                self.emitted_cells = cells_before + 1;
                cells_before = self.emitted_cells;
                start = stmts.len();
            }
        }
        Ok(())
    }

    fn fragment(&mut self, mut stmts: Vec<Stmt>) -> Stmt {
        let span = self.out.span;
        let name = ident(&format!("{}_fragment_{}", self.symbol, self.fragments.len()), span);
        let mut params = self.params.to_vec();
        params.push(Param {
            name: self.out.clone(), ty: self.output_ty.clone(), mode: ParamMode::Default,
            default: None, span,
        });
        let args = params.iter().map(|param| {
            let value = read(&param.name);
            if param.mode == ParamMode::Move {
                Arg::MovePos(Expr {
                    span,
                    kind: ExprKind::Method { recv: Box::new(value), name: ident("clone", span), args: Vec::new() },
                })
            } else {
                Arg::Pos(value)
            }
        }).collect();
        stmts.push(Stmt::Return { value: Expr { kind: ExprKind::IntLit(0), span }, span });
        self.fragments.push(Item::Fn(FnItem {
            name: name.clone(), type_params: Vec::new(), params,
            ret: TypeAnn { kind: TypeRef::Builtin(TypeWord::I32), span },
            body: Block { stmts, span }, span,
            receiver: Receiver::Free, owner: None, script: false, proto: false, test_attribute: None,
        }));
        Stmt::ExprStmt { value: Expr { kind: ExprKind::Call { callee: name, args }, span }, span }
    }

    fn iteration(
        &mut self,
        iteration: &ComponentViewFor,
        scope: &ViewScope<'_>,
        stmts: &mut Vec<Stmt>,
        element_tag: &str,
    ) -> Result<(), Vec<Diagnostic>> {
        let span = iteration.span;
        let source = ident(&format!("{}_source_{}", self.symbol, self.loops), span);
        stmts.push(Stmt::Let {
            kw: span,
            name: source.clone(),
            ann: None,
            mutable: false,
            value: scope.rename(&iteration.iter)?,
            span,
        });
        let mut inside = scope.clone();
        let (value, index) = inside.bind_loop_names(self.symbol, iteration, self.loops);
        self.loops += 1;
        let mut body = Vec::new();
        self.items(&iteration.items, &inside, &mut body, element_tag, false)?;
        if let Some(index) = index {
            stmts.push(Stmt::Let {
                kw: span,
                name: index.clone(),
                ann: None,
                mutable: true,
                value: Expr {
                    kind: ExprKind::IntLit(0),
                    span,
                },
                span,
            });
            body.push(Stmt::Assign {
                target: AssignTarget {
                    base: index.clone(),
                    path: Vec::new(),
                },
                value: Expr {
                    span,
                    kind: ExprKind::Binary {
                        op: BinOp::Add,
                        lhs: Box::new(read(&index)),
                        rhs: Box::new(Expr {
                            kind: ExprKind::IntLit(1),
                            span,
                        }),
                    },
                },
                span,
            });
        }
        stmts.push(Stmt::ForIn {
            var: value,
            iter: read(&source),
            body: Block { stmts: body, span },
            span,
        });
        Ok(())
    }

    fn entry(
        &mut self,
        entry: &ComponentViewEntry,
        element_tag: &str,
        scope: &ViewScope<'_>,
        stmts: &mut Vec<Stmt>,
    ) -> Result<(), Vec<Diagnostic>> {
        let span = entry.span;
        let prop = |value| [literal(&entry.key, span), value];
        if entry.is_event && !matches!(entry.value, ComponentEntryValue::ChildEvent(_)) {
            return self.event(entry, element_tag, scope, stmts);
        }
        let boolean = matches!((element_tag, entry.key.as_str()),
            ("drawer" | "alert-dialog", "open") | ("button" | "input", "disabled")
            | ("sidebar_menu_button", "active") | (_, "checked"));
        match &entry.value {
            ComponentEntryValue::Expr(expr) => {
                self.value_property(entry, expr, scope, stmts, boolean)?;
            }
            ComponentEntryValue::If {
                cond,
                then_value,
                else_chain,
                else_value,
            } => {
                let mut then = Vec::new();
                let mut els = Vec::new();
                self.value_property(entry, then_value, scope, &mut then, boolean)?;
                self.value_property(entry, else_value, scope, &mut els, boolean)?;
                // PLAN-106：else-if 链臂——从后往前构建嵌套 if（末臂 else=
                // 主 else_value；链臂 else=下一臂整条 if），主 if 的 else 侧
                // 挂到首链臂，保持源序。
                let mut chain_tail = els;
                for (chain_cond, chain_value) in else_chain.iter().rev() {
                    let mut chain_then = Vec::new();
                    self.value_property(entry, chain_value, scope, &mut chain_then, boolean)?;
                    chain_tail = vec![if_stmt(
                        scope.rename(chain_cond)?,
                        chain_then,
                        chain_tail,
                        span,
                    )];
                }
                stmts.push(if_stmt(scope.rename(cond)?, then, chain_tail, span));
            }
            ComponentEntryValue::ClassList(parts) => {
                let classes = ident(&format!("{}_classes_{}", self.symbol, self.cells), span);
                self.cells += 1;
                stmts.push(Stmt::Let {
                    kw: span, name: classes.clone(), ann: None, mutable: true,
                    value: literal("", span), span,
                });
                let append = |value| Stmt::Assign {
                    target: AssignTarget { base: classes.clone(), path: Vec::new() },
                    value: Expr {
                        span,
                        kind: ExprKind::Binary {
                            op: BinOp::Add,
                            lhs: Box::new(read(&classes)),
                            rhs: Box::new(Expr {
                                span,
                                kind: ExprKind::Binary {
                                    op: BinOp::Add,
                                    lhs: Box::new(literal(" ", span)),
                                    rhs: Box::new(value),
                                },
                            }),
                        },
                    },
                    span,
                };
                for part in parts {
                    match part {
                        ComponentClassPart::Value(value) => stmts.push(append(scope.rename(value)?)),
                        ComponentClassPart::If { cond, then_value, else_value } => {
                            stmts.push(if_stmt(
                                scope.rename(cond)?,
                                vec![append(scope.rename(then_value)?)],
                                vec![append(scope.rename(else_value)?)],
                                span,
                            ));
                        }
                    }
                }
                // The view output owns its text beyond the scope of this local.
                let owned = Expr {
                    kind: ExprKind::Method {
                        recv: Box::new(read(&classes)),
                        name: ident("clone", span),
                        args: Vec::new(),
                    },
                    span,
                };
                self.emit(stmts, "property", prop(owned));
            }
            ComponentEntryValue::Lambda { .. } => {
                return Err(view_error("an inline event lambda", span))
            }
            ComponentEntryValue::ChildEvent(_) => {}
        }
        Ok(())
    }

    fn value_property(
        &mut self,
        entry: &ComponentViewEntry,
        value: &Expr,
        scope: &ViewScope<'_>,
        stmts: &mut Vec<Stmt>,
        boolean: bool,
    ) -> Result<(), Vec<Diagnostic>> {
        let span = entry.span;
        let property = |value| [literal(&entry.key, span), value];
        if boolean {
            let mut then = Vec::new();
            let mut els = Vec::new();
            self.emit(&mut then, "property", property(literal("true", span)));
            self.emit(&mut els, "property", property(literal("false", span)));
            stmts.push(if_stmt(scope.rename(value)?, then, els, value.span));
        } else {
            let value = match value.kind {
                ExprKind::BoolLit(value) => literal(if value { "true" } else { "false" }, span),
                ExprKind::StrLit(_) => value.clone(),
                _ => display(scope.rename(value)?),
            };
            self.emit(stmts, "property", property(value));
        }
        Ok(())
    }

    fn event(
        &mut self,
        entry: &ComponentViewEntry,
        element_tag: &str,
        scope: &ViewScope<'_>,
        stmts: &mut Vec<Stmt>,
    ) -> Result<(), Vec<Diagnostic>> {
        let span = entry.span;
        let (name, args) = match (entry.message_call(), &entry.value) {
            (Some(call), _) => call,
            (
                _,
                ComponentEntryValue::Expr(Expr {
                    kind: ExprKind::Field { base, name },
                    ..
                }),
            ) if matches!(&base.kind, ExprKind::Var(root) if root.name == "self") => {
                (name, [].as_slice())
            }
            _ => return Err(view_error("this event binding", span)),
        };
        if ImageSurfaceEventPayload::from_binding(element_tag, &entry.key)
            .is_some_and(|payload| payload.matches_arguments(args))
        {
            self.emit(stmts, "image-surface-event", [
                literal(&entry.key, span),
                literal(&name.name, span),
            ]);
            return Ok(());
        }
        self.emit(
            stmts,
            "event",
            [
                literal(&entry.key, span),
                literal(&name.name, span),
                literal(&args.len().to_string(), span),
            ],
        );
        if !args.is_empty() {
            let message = scope
                .node
                .messages
                .iter()
                .find(|message| message.name == name.name)
                .ok_or_else(|| view_error("an undeclared event", span))?;
            for (arg, ty) in args.iter().zip(&message.payload) {
                let Arg::Pos(value) = arg else {
                    return Err(view_error("a named event argument", span));
                };
                let value = scope.rename(value)?;
                match payload_type(ty) {
                    Some(TypeWord::Bool) => {
                        let mut then = Vec::new();
                        let mut els = Vec::new();
                        self.emit(&mut then, "bool", [literal("true", span)]);
                        self.emit(&mut els, "bool", [literal("false", span)]);
                        stmts.push(if_stmt(value, then, els, span));
                    }
                    Some(TypeWord::StrOwned) => {
                        let arg_span = value.span;
                        let name = ident(&format!("{}_event_text_{}", self.symbol, self.cells), arg_span);
                        self.cells += 1;
                        stmts.push(Stmt::Let {
                            kw: arg_span,
                            name: name.clone(),
                            ann: Some(TypeAnn { kind: TypeRef::Builtin(TypeWord::StrOwned), span: arg_span }),
                            mutable: false,
                            value,
                            span: arg_span,
                        });
                        self.emit(stmts, ty, [display(read(&name))]);
                    }
                    Some(TypeWord::I32) => {
                        self.emit(stmts, ty, [display(value)]);
                    }
                    // PLAN-117（U05 031 真语料）：`OnWheel(float)` / `OnPan(float,…)`
                    // 事件载荷——float/f64 与 int 同列显示发射（AV cell 域有 F32）。
                    Some(TypeWord::Float | TypeWord::Double | TypeWord::F64) => {
                        self.emit(stmts, ty, [display(value)]);
                    }
                    _ => return Err(view_error("this event payload type", span)),
                }
            }
        }
        Ok(())
    }

    fn emit(&mut self, stmts: &mut Vec<Stmt>, op: &str, values: impl IntoIterator<Item = Expr>) {
        let span = self.out.span;
        for mut value in std::iter::once(literal(op, span)).chain(values) {
            self.emitted_cells += 1;
            if !matches!(value.kind, ExprKind::StrLit(_)) {
                let value_span = value.span;
                let cell = ident(&format!("{}_cell_{}", self.symbol, self.cells), value_span);
                self.cells += 1;
                stmts.push(Stmt::Let {
                    kw: value_span,
                    name: cell.clone(),
                    ann: Some(TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::StrOwned),
                        span: value_span,
                    }),
                    mutable: false,
                    value,
                    span: value_span,
                });
                value = Expr {
                    span: value_span,
                    kind: ExprKind::Method {
                        recv: Box::new(read(&cell)),
                        name: ident("move", value_span),
                        args: Vec::new(),
                    },
                };
            }
            stmts.push(Stmt::ExprStmt {
                value: Expr {
                    span,
                    kind: ExprKind::Method {
                        recv: Box::new(read(self.out)),
                        name: ident("push", span),
                        args: vec![Arg::Pos(value)],
                    },
                },
                span,
            });
        }
    }
}

fn ident(name: &str, span: Span) -> Ident {
    Ident {
        name: name.to_owned(),
        span,
    }
}

fn read(name: &Ident) -> Expr {
    Expr {
        kind: ExprKind::Var(name.clone()),
        span: name.span,
    }
}

fn literal(text: &str, span: Span) -> Expr {
    Expr {
        kind: ExprKind::StrLit(text.to_owned()),
        span,
    }
}

fn display(value: Expr) -> Expr {
    let span = value.span;
    Expr {
        kind: ExprKind::Call {
            callee: ident("__fstr", span),
            args: vec![Arg::Pos(value)],
        },
        span,
    }
}

fn if_stmt(cond: Expr, then: Vec<Stmt>, els: Vec<Stmt>, span: Span) -> Stmt {
    Stmt::If {
        cond,
        then: Block { stmts: then, span },
        els: (!els.is_empty()).then_some(Block { stmts: els, span }),
        span,
    }
}

fn view_error(what: &str, span: Span) -> Vec<Diagnostic> {
    vec![text_diag(
        "source.age.widget-view-unsupported",
        format!("cannot execute {what} in a widget view"),
        span,
    )]
}
