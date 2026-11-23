use std::collections::{BTreeMap, BTreeSet};

use ace_av_core::program_snapshot::SavedGlobalCells;
use ace_core::source::age_package::{
    AgePackageCompilation, AgeWidgetProperty, AgeWidgetRuntimeWatch, WIDGET_OUTBOUND_FIELD,
};

use super::{
    add_cell_call, add_scalar_setter, field_value_to_arg, find_function, fresh_context_token, live_view,
    ComponentFieldValue, ExecLimits, MirGlobal, MirModule, Value, WidgetFocusRequest, WidgetSession, WidgetViewItem,
    AGE_PACKAGE_HIDDEN_PREFIX,
};
use super::watch::{self, Watch, Watched};

pub(super) struct ChildrenProgram {
    root: usize,
    root_model: u32,
    root_watches: Vec<Watch>,
    components: BTreeMap<usize, Child>,
    sites: BTreeMap<(usize, usize), Site>,
    scratch: Vec<u32>,
    entries: Vec<u32>,
    host_messages: BTreeSet<String>,
}

struct Child {
    initialize: Option<String>,
    inputs: Vec<u32>,
    model: Option<u32>,
    view: String,
    handlers: BTreeMap<String, String>,
    context_handler: Option<String>,
    watches: Vec<Watch>,
    forwards: BTreeMap<String, String>,
    outbound: Option<Outbound>,
    focus_field: Option<usize>,
    store: Option<StoreFields>,
}

struct StoreFields {
    from_root: Vec<(usize, usize)>,
    to_root: Vec<(usize, usize)>,
}

struct Site {
    node: usize,
    props: Vec<Property>,
    channels: BTreeMap<String, String>,
}

enum Property {
    Evaluated(String),
    Scoped {
        name: String,
        setter: String,
        ty: &'static str,
    },
}

struct Outbound {
    field: usize,
    messages: BTreeMap<String, Vec<String>>,
}

type MountPath = Vec<(usize, SiteKey)>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SiteKey {
    Index(usize),
    Authored(String),
}

#[derive(Clone)]
struct Mount {
    site: (usize, usize),
    parent: MountPath,
    inputs: SavedGlobalCells,
    model: Option<SavedGlobalCells>,
    observed_model: Option<SavedGlobalCells>,
    context_token: Option<String>,
}

#[derive(Clone, Default)]
pub(super) struct Children {
    mounts: BTreeMap<MountPath, Mount>,
    positions: BTreeMap<Vec<usize>, MountPath>,
    observed_root: Option<SavedGlobalCells>,
    focus: Vec<(MountPath, String)>,
}

impl Children {
    pub(super) fn request_root_focus(&mut self, requests: Vec<String>) {
        self.request_focus(&[], requests);
    }

    fn request_focus(&mut self, path: &[(usize, SiteKey)], requests: Vec<String>) {
        self.focus.extend(requests.into_iter().map(|target| (path.to_vec(), target)));
    }

    pub(super) fn take_focus_requests(&mut self) -> Vec<WidgetFocusRequest> {
        std::mem::take(&mut self.focus).into_iter().filter_map(|(path, target)| {
            let component = if path.is_empty() {
                Vec::new()
            } else {
                self.positions.iter().find_map(|(position, candidate)| (candidate == &path).then(|| position.clone()))?
            };
            Some(WidgetFocusRequest { component, target })
        }).collect()
    }
}

impl ChildrenProgram {
    pub(super) fn prepare(
        compiled: &AgePackageCompilation,
        mir: &mut MirModule,
        root: usize,
        root_model: u32,
    ) -> Result<Self, String> {
        let mut program = Self {
            root,
            root_model,
            root_watches: Vec::new(),
            components: BTreeMap::new(),
            sites: BTreeMap::new(),
            scratch: Vec::new(),
            entries: Vec::new(),
            host_messages: compiled.description.components[root]
                .members
                .iter()
                .filter(|row| {
                    let name = row.name.trim_matches('"');
                    row.role == "bind"
                        && (name.starts_with("file:") || name.starts_with("storage:") || name == "service:todo")
                })
                .filter_map(|row| row.text.clone())
                .collect(),
        };
        let resolve = |module: &str, name: &str, mir: &MirModule| {
            let symbol = compiled
                .symbol_of_in(module, name)
                .ok_or_else(|| format!("missing checked widget entry {module}::{name}"))?;
            find_function(mir, &format!("d_{symbol}"))
        };
        let root_runtime = compiled.widget_runtime.iter().find(|runtime| runtime.node == root)
            .ok_or("missing checked root runtime")?;
        let root_module = &compiled.description.components[root].module;
        let root_component = &compiled.description.components[root];
        program.root_watches = program.watches(
            mir, &root_runtime.watches, &[root_model], Some(root_model),
            |name, mir| resolve(root_module, name, mir),
        )?;
        for runtime in compiled
            .widget_runtime
            .iter()
            .filter(|runtime| runtime.node != root)
        {
            let component = &compiled.description.components[runtime.node];
            if !runtime.timers.is_empty() {
                return Err(format!(
                    "ace.desktop.widget-timer: child timers are unavailable for {}",
                    component.name
                ));
            }
            let view = resolve(&component.module, &runtime.view, mir)?;
            let params = mir
                .sig_of(mir.funcs[view as usize].sig)
                .ok_or("missing checked child view signature")?
                .params
                .clone();
            let value_count = params
                .len()
                .checked_sub(usize::from(runtime.initialize.is_some()))
                .ok_or("checked child view is missing its model parameter")?;
            let inputs = params[..value_count]
                .iter()
                .map(|&ty| program.add_scratch(mir, ty))
                .collect::<Vec<_>>();
            let mut call_inputs = inputs.clone();
            let (initialize, model) = if let Some(initializer) = &runtime.initialize {
                let target = resolve(&component.module, initializer, mir)?;
                let signature = mir
                    .sig_of(mir.funcs[target as usize].sig)
                    .ok_or("missing checked child initializer signature")?;
                let model = program.add_scratch(mir, signature.result);
                call_inputs.push(model);
                let initialize = program.adapter(mir, target, &inputs, Some(model))?;
                (Some(initialize), Some(model))
            } else {
                (None, None)
            };
            let view = program.adapter(mir, view, &call_inputs, None)?;
            let mut handlers = BTreeMap::new();
            for handler in &runtime.handlers {
                let target = resolve(&component.module, &handler.function, mir)?;
                let entry = program.adapter(mir, target, &call_inputs, model)?;
                handlers.insert(handler.message.clone(), entry);
            }
            let outbound = runtime.outbound.then(|| Outbound {
                field: component.fields.len(),
                messages: component
                    .messages
                    .iter()
                    .filter(|message| !component.handler_msgs.contains(&message.name))
                    .map(|message| (message.name.clone(), message.payload.clone()))
                    .collect(),
            });
            let watches = program.watches(mir, &runtime.watches, &call_inputs, model,
                |name, mir| resolve(&component.module, name, mir))?;
            let store = component.imported_store.as_ref().map(|imported| {
                let root = root_component.imported_store.as_ref().ok_or(
                    "ace.desktop.shared-store: the root must own the imported Store",
                )?;
                if root.module != imported.module || root.name != imported.name || root.fields != imported.fields
                    || model.is_none()
                {
                    return Err("ace.desktop.shared-store: incompatible root and child Store bindings");
                }
                let from_root = root.fields.iter().map(|name| {
                    let source = root_component.fields.iter().position(|(field, _)| field == name);
                    let destination = component.fields.iter().position(|(field, _)| field == name);
                    source.zip(destination).ok_or("ace.desktop.shared-store: missing model field")
                }).collect::<Result<Vec<_>, _>>()?;
                let to_root = from_root.iter().map(|&(source, destination)| (destination, source)).collect();
                Ok(StoreFields { from_root, to_root })
            }).transpose()?;
            program.components.insert(
                runtime.node,
                Child {
                    initialize,
                    inputs,
                    model,
                    view,
                    handlers,
                    context_handler: runtime.context_handler.clone(),
                    watches,
                    outbound,
                    focus_field: runtime.focus.then_some(component.fields.len() + usize::from(runtime.outbound)),
                    forwards: component.handler_forwards.clone(),
                    store,
                },
            );
        }

        for runtime in &compiled.widget_runtime {
            let owner = runtime.node;
            let component = &compiled.description.components[owner];
            let key = (component.module.clone(), component.name.clone());
            for site in compiled.widget_refs.get(&key).into_iter().flatten() {
                let node = compiled
                    .description
                    .components
                    .iter()
                    .position(|node| node.module == site.module && node.name == site.node)
                    .ok_or("unknown checked child component")?;
                let child_inputs = program
                    .components
                    .get(&node)
                    .ok_or("ace.desktop.widget-view: recursive widget composition")?
                    .inputs
                    .clone();
                let owner_inputs = if owner == root {
                    vec![root_model]
                } else {
                    let current = &program.components[&owner];
                    current
                        .inputs
                        .iter()
                        .copied()
                        .chain(current.model)
                        .collect()
                };
                let mut props = Vec::new();
                let mut channels = site
                    .events
                    .iter()
                    .map(|event| (event.name.clone(), event.target.clone()))
                    .collect::<BTreeMap<_, _>>();
                for property in &site.properties {
                    match property {
                        AgeWidgetProperty::Message { name, target, .. } => {
                            channels.insert(name.clone(), target.clone());
                        }
                        AgeWidgetProperty::Value {
                            module,
                            evaluators,
                            authored_default,
                            runtime_evaluator,
                            ..
                        } => {
                            let cell = *child_inputs
                                .get(props.len())
                                .ok_or("extra checked child property")?;
                            let (name, inputs) = if let Some(evaluator) = runtime_evaluator {
                                (evaluator, owner_inputs.as_slice())
                            } else {
                                let name = evaluators
                                    .first()
                                    .ok_or("missing checked child property evaluator")?;
                                // An authored value on a stateless parent still
                                // reads that parent's checked input parameters.
                                let inputs = if *authored_default { &[][..] } else { owner_inputs.as_slice() };
                                (name, inputs)
                            };
                            let entry = resolve(module, name, mir)?;
                            if *authored_default && !inputs.is_empty() {
                                return Err(
                                    "checked default property unexpectedly uses parent state"
                                        .into(),
                                );
                            }
                            let signature = mir
                                .sig_of(mir.funcs[entry as usize].sig)
                                .ok_or("missing checked child property signature")?;
                            if signature.params.len() != inputs.len() {
                                return Err(format!(
                                    "ace.desktop.widget-runtime-inputs: missing inputs for {name}"
                                ));
                            }
                            props.push(Property::Evaluated(program.adapter(
                                mir,
                                entry,
                                inputs,
                                Some(cell),
                            )?));
                        }
                        AgeWidgetProperty::ScopedValue { name, .. } => {
                            let cell = *child_inputs
                                .get(props.len())
                                .ok_or("extra checked child property")?;
                            program.entries.push(mir.funcs.len() as u32);
                            let (setter, ty) = add_scalar_setter(mir, cell)?;
                            props.push(Property::Scoped {
                                name: name.clone(),
                                setter,
                                ty,
                            });
                        }
                    }
                }
                if props.len() != child_inputs.len() {
                    return Err("missing checked child properties".into());
                }
                program.sites.insert(
                    (owner, site.span.start),
                    Site {
                        node,
                        props,
                        channels,
                    },
                );
            }
        }
        Ok(program)
    }

    fn add_scratch(&mut self, mir: &mut MirModule, ty: u32) -> u32 {
        let cell = mir.globals.len() as u32;
        mir.globals.push(MirGlobal {
            name: format!("{AGE_PACKAGE_HIDDEN_PREFIX}widget_cell_{cell}"),
            ty,
            mutable: true,
        });
        self.scratch.push(cell);
        cell
    }

    fn adapter(
        &mut self,
        mir: &mut MirModule,
        target: u32,
        inputs: &[u32],
        output: Option<u32>,
    ) -> Result<String, String> {
        self.entries.push(mir.funcs.len() as u32);
        add_cell_call(mir, target, inputs, output)
    }

    fn watches(
        &mut self, mir: &mut MirModule, watches: &[AgeWidgetRuntimeWatch],
        inputs: &[u32], model: Option<u32>, resolve: impl Fn(&str, &MirModule) -> Result<u32, String>,
    ) -> Result<Vec<Watch>, String> {
        watches.iter().map(|watch| {
            let target = resolve(&watch.function, mir)?;
            let model = model.ok_or("checked watch requires a model")?;
            let entry = self.adapter(mir, target, inputs, Some(model))?;
            Ok(Watch::new(entry, watch))
        }).collect()
    }

    pub(super) fn entries(&self) -> &[u32] {
        &self.entries
    }

    pub(super) fn compose(
        &self,
        session: &WidgetSession,
        previous: &Children,
        limits: &ExecLimits,
    ) -> Result<(Vec<WidgetViewItem>, Children), String> {
        let scratch = session.snapshot.save_global_cells(&self.scratch)?;
        let mut next = Children { focus: previous.focus.clone(), ..Children::default() };
        let observed_root = if self.root_watches.is_empty() {
            None
        } else {
            let model = session.snapshot.save_global_cells(&[self.root_model])?;
            let previous = previous.observed_root.as_ref().map(|model| Watched { inputs: None, model });
            watch::run(&self.root_watches, previous, Watched { inputs: None, model: &model }, |entry| {
                session.execute(entry, &[], limits)?;
                next.request_root_focus(session.focus_targets(self.root_model, session.program.focus_field)?);
                Ok(())
            })?;
            Some(session.snapshot.save_global_cells(&[self.root_model])?)
        };
        next.observed_root = observed_root;
        let mut items = session.view(limits)?;
        let mut draw = Draw {
            program: self,
            session,
            previous,
            next,
            limits,
            active: BTreeSet::from([self.root]),
            path: Vec::new(),
            position: Vec::new(),
            remaining: limits.max_slot_bytes,
        };
        draw.items(self.root, &Vec::new(), &mut items)?;
        session.snapshot.install_saved_global_cells(&scratch)?;
        Ok((items, draw.next))
    }

    pub(super) fn dispatch(
        &self,
        session: &WidgetSession,
        position: &[usize],
        message: &str,
        payload: &[ComponentFieldValue],
        children: &mut Children,
        limits: &ExecLimits,
    ) -> Result<(), String> {
        let path = children
            .positions
            .get(position)
            .cloned()
            .ok_or("ace.desktop.widget-event-stale: child instance is no longer mounted")?;
        let scratch = session.snapshot.save_global_cells(&self.scratch)?;
        let mut remaining = 4096;
        self.deliver(
            session,
            &path,
            message,
            payload,
            children,
            limits,
            &mut remaining,
        )?;
        session.snapshot.install_saved_global_cells(&scratch)?;
        Ok(())
    }

    fn deliver(
        &self,
        session: &WidgetSession,
        path: &MountPath,
        message: &str,
        payload: &[ComponentFieldValue],
        children: &mut Children,
        limits: &ExecLimits,
        remaining: &mut usize,
    ) -> Result<(), String> {
        *remaining = remaining
            .checked_sub(1)
            .ok_or("ace.desktop.widget-outbound: transition limit exceeded")?;
        if path.is_empty() {
            if self.host_messages.contains(message) {
                return Err(
                    "ace.desktop.widget-outbound: a host binding owns the target message".into(),
                );
            }
            children.request_root_focus(session.transition_root(message, payload, limits)?);
            return Ok(());
        }
        let mount = children
            .mounts
            .get_mut(path)
            .ok_or("missing checked child instance")?;
        let site = &self.sites[&mount.site];
        let child = &self.components[&site.node];
        if child.context_handler.as_deref() == Some(message) {
            return Err("ace.desktop.widget-context: only the host can deliver widget context".into());
        }
        if let Some(channel) = child.forwards.get(message) {
            let target = site
                .channels
                .get(channel)
                .ok_or("unbound checked child callback")?;
            let parent = mount.parent.clone();
            return self.deliver(
                session, &parent, target, payload, children, limits, remaining,
            );
        }
        let entry = child.handlers.get(message).ok_or_else(|| {
            format!("source.age.component-event: unknown child message {message:?}")
        })?;
        session.snapshot.install_saved_global_cells(&mount.inputs)?;
        if let Some(model) = &mount.model {
            session.snapshot.install_saved_global_cells(model)?;
        }
        if let Some(fields) = &child.store {
            let model = child.model.ok_or("missing checked child Store model")?;
            session.snapshot.copy_model_fields(&session.program.mir, self.root_model, model, &fields.from_root)?;
        }
        if let Some((handler, token)) = child.context_handler.as_ref().zip(mount.context_token.as_deref()) {
            session.apply_context(&child.handlers[handler], token, limits)?;
        }
        let payload = payload.iter().map(field_value_to_arg).collect::<Vec<_>>();
        session.execute(entry, &payload, limits)?;
        if let Some(fields) = &child.store {
            let model = child.model.ok_or("missing checked child Store model")?;
            session.snapshot.copy_model_fields(&session.program.mir, model, self.root_model, &fields.to_root)?;
        }
        let events = outgoing(session, child)?;
        let mut focus = Vec::new();
        if let Some(model) = child.model {
            mount.model = Some(session.snapshot.save_global_cells(&[model])?);
            focus = session.focus_targets(model, child.focus_field)?;
        }
        let parent = mount.parent.clone();
        children.request_focus(path, focus);
        for (name, payload) in events {
            if let Some(message) = site.channels.get(&name) {
                self.deliver(
                    session, &parent, message, &payload, children, limits, remaining,
                )?;
            }
        }
        Ok(())
    }
}

struct Draw<'a> {
    program: &'a ChildrenProgram,
    session: &'a WidgetSession,
    previous: &'a Children,
    next: Children,
    limits: &'a ExecLimits,
    active: BTreeSet<usize>,
    path: MountPath,
    position: Vec<usize>,
    remaining: usize,
}

impl Draw<'_> {
    fn items(
        &mut self,
        owner: usize,
        parent: &MountPath,
        items: &mut [WidgetViewItem],
    ) -> Result<(), String> {
        let mut occurrences = BTreeMap::new();
        for (index, item) in items.iter_mut().enumerate() {
            let WidgetViewItem::Element(element) = item else {
                continue;
            };
            let occurrence = occurrences.entry(element.site).or_insert(0);
            let key = element
                .properties
                .iter()
                .rev()
                .find(|(name, _)| name == "key")
                .map_or(SiteKey::Index(*occurrence), |(_, key)| {
                    SiteKey::Authored(key.clone())
                });
            *occurrence += 1;
            self.path.push((element.site, key));
            self.position.push(index);
            let site_key = (owner, element.site);
            if let Some(site) = self.program.sites.get(&site_key) {
                if !element.children.is_empty() {
                    return Err("ace.desktop.widget-view: unexpected widget slot content".into());
                }
                if !self.active.insert(site.node) {
                    return Err("ace.desktop.widget-view: recursive widget composition".into());
                }
                let path = self.path.clone();
                element.children =
                    self.child(site_key, site, &path, parent, &element.properties)?;
                self.items(site.node, &path, &mut element.children)?;
                self.active.remove(&site.node);
            } else {
                self.items(owner, parent, &mut element.children)?;
            }
            self.position.pop();
            self.path.pop();
        }
        Ok(())
    }

    fn child(
        &mut self,
        key: (usize, usize),
        site: &Site,
        path: &MountPath,
        parent: &MountPath,
        properties: &[(String, String)],
    ) -> Result<Vec<WidgetViewItem>, String> {
        let child = &self.program.components[&site.node];
        if self.next.mounts.contains_key(path) {
            return Err("ace.desktop.widget-view: duplicate child instance key".into());
        }
        for property in &site.props {
            match property {
                Property::Evaluated(entry) => {
                    self.session.execute(entry, &[], self.limits)?;
                }
                Property::Scoped { name, setter, ty } => {
                    let (_, text) = properties.iter().rev().find(|(property, _)| property == name)
                        .ok_or_else(|| format!("ace.desktop.widget-runtime-inputs: missing scoped property {name:?}"))?;
                    let value = live_view::scalar(ty, text)?;
                    self.session
                        .execute(setter, &[field_value_to_arg(&value)], self.limits)?;
                }
            }
        }
        let previous = self.previous.mounts.get(path);
        let context_token = child.context_handler.as_ref().map(|_| {
            previous.and_then(|mount| mount.context_token.clone()).unwrap_or_else(fresh_context_token)
        });
        if let Some(model) = previous.and_then(|mount| mount.model.as_ref()) {
            self.session.snapshot.install_saved_global_cells(model)?;
        } else if let Some(initializer) = &child.initialize {
            self.session.execute(initializer, &[], self.limits)?;
            if let Some((handler, token)) = child.context_handler.as_ref().zip(context_token.as_deref()) {
                self.session.apply_context(&child.handlers[handler], token, self.limits)?;
            }
            if let Some(init) = child.handlers.get("Init") {
                self.session.execute(init, &[], self.limits)?;
                if let Some(model) = child.model {
                    self.next.request_focus(path, self.session.focus_targets(model, child.focus_field)?);
                }
                if outgoing(self.session, child)?
                    .iter()
                    .any(|(name, _)| site.channels.contains_key(name))
                {
                    return Err(
                        "ace.desktop.widget-init: outbound events require a mounted parent".into(),
                    );
                }
            }
        }
        if let Some(fields) = &child.store {
            let model = child.model.ok_or("missing checked child Store model")?;
            self.session.snapshot.copy_model_fields(
                &self.session.program.mir, self.program.root_model, model, &fields.from_root,
            )?;
        }
        let inputs = self.session.snapshot.save_global_cells(&child.inputs)?;
        if !child.watches.is_empty() {
            let cell = child.model.ok_or("checked watch requires a model")?;
            let model = self.session.snapshot.save_global_cells(&[cell])?;
            let previous = previous.and_then(|mount| mount.observed_model.as_ref().map(|model| Watched {
                inputs: Some(&mount.inputs), model,
            }));
            watch::run(&child.watches, previous, Watched { inputs: Some(&inputs), model: &model }, |entry| {
                self.session.execute(entry, &[], self.limits)?;
                self.next.request_focus(path, self.session.focus_targets(cell, child.focus_field)?);
                Ok(())
            })?;
        }
        let model = child
            .model
            .map(|model| self.session.snapshot.save_global_cells(&[model]))
            .transpose()?;
        if !inputs.charge(&mut self.remaining)
            || model
                .as_ref()
                .is_some_and(|model| !model.charge(&mut self.remaining))
        {
            return Err(
                "ace.desktop.widget-view: child instance state exceeds the slot budget".into(),
            );
        }
        let view = self.session.execute(&child.view, &[], self.limits)?;
        self.next
            .positions
            .insert(self.position.clone(), path.clone());
        self.next.mounts.insert(
            path.clone(),
            Mount {
                site: key,
                parent: parent.clone(),
                inputs,
                observed_model: (!child.watches.is_empty()).then(|| model.clone()).flatten(),
                model,
                context_token,
            },
        );
        live_view::decode_view(view)
    }
}

type OutgoingEvent = (String, Vec<ComponentFieldValue>);

fn outgoing(session: &WidgetSession, child: &Child) -> Result<Vec<OutgoingEvent>, String> {
    let Some((model, outbound)) = child.model.zip(child.outbound.as_ref()) else {
        return Ok(Vec::new());
    };
    let cells = session.snapshot.global_cells();
    let Some(Value::Record(record)) = cells.get(model as usize) else {
        return Err("ace.desktop.widget-outbound: missing checked child model".into());
    };
    let Some(Value::List(queue)) = record.fields.get(outbound.field) else {
        return Err(format!(
            "ace.desktop.widget-outbound: invalid {WIDGET_OUTBOUND_FIELD}"
        ));
    };
    let queue = queue.rc.borrow();
    let mut cells = queue.elems.iter();
    let mut events = Vec::new();
    while let Some(value) = cells.next() {
        let name = live_view::text(value)?;
        let types = outbound
            .messages
            .get(name)
            .ok_or_else(|| format!("ace.desktop.widget-outbound: unknown message {name:?}"))?;
        let args = live_view::payload(&mut cells, types.iter().map(String::as_str))?;
        events.push((name.to_owned(), args));
    }
    Ok(events)
}
