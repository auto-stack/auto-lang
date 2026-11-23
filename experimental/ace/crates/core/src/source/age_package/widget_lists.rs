use std::collections::{BTreeMap, BTreeSet};

use crate::atom_text::Span;

use super::super::age_component::{
    model_type, ComponentEntryValue, ComponentParam, ComponentScenario, ComponentViewItem,
};
use super::super::ast::{Arg, Expr, ExprKind, FieldDecl, FnItem, Ident, Item, Receiver, TypeAnn, TypeRef};
use super::super::cst::{project_packages_front_with_widget_lists, PackagesFace};
use super::super::packages::{unit_key, PackageDiagnostic, PackageSyntax, PackagedAgeComponent};
use super::{
    is_bare_list_annotation, list_element, walk_widget_sites, WidgetArgumentSite, WidgetSiteTargets,
    AGE_PACKAGE_HIDDEN_PREFIX,
};

#[derive(Default)]
pub(crate) struct WidgetListProjection {
    pub(crate) inputs: BTreeMap<usize, TypeAnn>,
    pub(crate) model_fields: BTreeSet<usize>,
    pub(crate) selectors: BTreeMap<usize, SelectorInput>,
    pub(crate) record_fields: BTreeMap<String, Vec<FieldDecl>>,
    pub(crate) specialized_name: Option<String>,
}
type WidgetParamKey = (String, usize);
type RecordOrigins = BTreeMap<String, BTreeMap<String, (String, Ident)>>;

pub(crate) const SPARSE_RECORD_PREFIX: &str = "__age_p_hidden_sparse_widget_record_";

pub(crate) fn is_sparse_record(name: &str) -> bool {
    name.find(AGE_PACKAGE_HIDDEN_PREFIX)
        .is_some_and(|start| name[start..].starts_with(SPARSE_RECORD_PREFIX))
}

struct ListSite {
    caller: String,
    span: usize,
    inputs: Vec<(TypeAnn, Span)>,
    selectors: BTreeMap<usize, SelectorInput>,
}

impl WidgetListProjection {
    fn bind_type(
        &mut self,
        caller: &PackageSyntax,
        source: &str,
        mut ty: TypeAnn,
        span: Span,
    ) -> Option<(String, Ident)> {
        if is_bare_list_annotation(&ty) {
            let kind = TypeRef::Record(Ident { name: "List".into(), span });
            self.inputs.insert(span.start, TypeAnn { kind, span });
            return None;
        }
        let TypeRef::Generic { name: list, args } = &mut ty.kind else { unreachable!() };
        // Retain the caller's nominal identity, but locate generated type
        // positions in the widget source where the parameter was declared.
        ty.span = span;
        list.span = span;
        args[0].span = span;
        let origin = if let TypeRef::Record(name) = &mut args[0].kind {
            let alias = format!("@age-widget-type/{source}/{}", name.name);
            if let Some(fields) = caller.program.items.iter().find_map(|item| match item {
                Item::Type(record) if record.name.name == name.name => Some(&record.fields),
                _ => None,
            }) {
                self.record_fields.insert(alias.clone(), fields.clone());
            }
            let origin = (alias.clone(), name.clone());
            name.name = alias;
            name.span = span;
            Some(origin)
        } else {
            None
        };
        self.inputs.insert(span.start, ty);
        origin
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SelectorInput {
    Text(BTreeSet<String>),
    TextList(BTreeSet<String>),
}

impl SelectorInput {
    fn from_expr(value: &Expr) -> Option<Self> {
        let text = |value: &Expr| match &value.kind {
            ExprKind::StrLit(value) => Some(value.clone()),
            _ => None,
        };
        match &value.kind {
            ExprKind::StrLit(value) => Some(Self::Text([value.clone()].into())),
            ExprKind::ArrayLit(items) => items
                .iter()
                .map(text)
                .collect::<Option<_>>()
                .map(Self::TextList),
            _ => None,
        }
    }

    fn merge(&mut self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(known), Self::Text(supplied))
            | (Self::TextList(known), Self::TextList(supplied)) => {
                known.extend(supplied.iter().cloned());
                true
            }
            _ => false,
        }
    }
}

fn needs_type(param: &ComponentParam) -> bool {
    param.ty == "List"
        && param.collection_type.is_none()
        && param.default.as_ref().is_none_or(|value| {
            matches!(&value.kind, ExprKind::ArrayLit(items) if items.is_empty())
        })
}

fn supplied_value<'a>(site: &WidgetArgumentSite<'a>, property: &str) -> Option<&'a Expr> {
    site.element.items.iter().find_map(|item| match item {
        ComponentViewItem::Entry(entry) if entry.key == property => match &entry.value {
            ComponentEntryValue::Expr(value) => Some(value),
            _ => None,
        },
        _ => None,
    })
}

fn supplied_model_field<'a>(site: &WidgetArgumentSite<'a>, property: &str) -> Option<&'a Ident> {
    let value = supplied_value(site, property)?;
    let ExprKind::Field { base, name } = &value.kind else {
        return None;
    };
    matches!(&base.kind, ExprKind::Var(root) if root.name == "self").then_some(name)
}

fn caller_list_type(
    component: &PackagedAgeComponent,
    field: &Ident,
    units: &BTreeMap<String, PackageSyntax>,
) -> Option<TypeAnn> {
    let ty = if let Some(field) = component.node.fields.iter().find(|candidate| candidate.name == field.name) {
        model_type(field, true).ok()?
    } else {
        computed_bare_list_type(component, field, units)?
    };
    if is_bare_list_annotation(&ty) {
        return Some(ty);
    }
    match &list_element(&ty)?.kind {
        TypeRef::Builtin(_) | TypeRef::Record(_) => Some(ty),
        _ => None,
    }
}

fn computed_bare_list_type(
    component: &PackagedAgeComponent,
    field: &Ident,
    units: &BTreeMap<String, PackageSyntax>,
) -> Option<TypeAnn> {
    let computed = component.node.members.iter()
        .find(|row| row.role == "computed" && row.name == field.name)?;
    let ExprKind::Call { callee, .. } = &computed.value.as_ref()?.kind else { return None };
    let ty = &declared_helper(component, callee, units)?.ret;
    is_bare_list_annotation(ty).then(|| ty.clone())
}

fn declared_helper<'a>(
    component: &PackagedAgeComponent,
    callee: &Ident,
    units: &'a BTreeMap<String, PackageSyntax>,
) -> Option<&'a FnItem> {
    fn free_function<'a>(unit: &'a PackageSyntax, name: &str) -> Option<&'a FnItem> {
        unit.program.items.iter().find_map(|item| match item {
            Item::Fn(function) if function.name.name == name
                && matches!(function.receiver, Receiver::Free) => Some(function),
            _ => None,
        })
    }

    let caller = &units[&component.module];
    if let Some(function) = free_function(caller, &callee.name) {
        return Some(function);
    }
    let import = caller.imports.iter().find(|import| import.bound_name() == callee.name)?;
    let (package, _) = component.module.split_once("::")?;
    let (key, name) = match import.path.as_slice() {
        [module, name] => (unit_key(package, module), name),
        [package, module, name] => (unit_key(package, module), name),
        _ => return None,
    };
    let unit = units.get(&key)?;
    // This is a type hint. The package binder still resolves and checks the
    // actual import and the arguments to the computed call.
    unit.exports.contains(name).then(|| free_function(unit, name)).flatten()
}

fn computed_argument_list_type(
    component: &PackagedAgeComponent,
    param: &ComponentParam,
    units: &BTreeMap<String, PackageSyntax>,
) -> Option<TypeAnn> {
    if param.default.is_none() {
        return None;
    }
    component.node.members.iter().filter(|row| row.role == "computed").find_map(|row| {
        let ExprKind::Call { callee, args } = &row.value.as_ref()?.kind else { return None };
        let function = declared_helper(component, callee, units)?;
        let index = args.iter().position(|arg| matches!(arg,
            Arg::Pos(Expr { kind: ExprKind::Field { base, name }, .. })
                if name.name == param.name
                    && matches!(&base.kind, ExprKind::Var(root) if root.name == "self")))?;
        let ty = &function.params.get(index)?.ty;
        is_bare_list_annotation(ty).then(|| ty.clone())
    })
}

/// A bare List can take its type from an authored model field or computed call
/// at a resolved widget site. The other arguments are still checked at each
/// site against the inferred signature by the ordinary package checker.
pub(crate) fn bind(
    components: &mut Vec<PackagedAgeComponent>,
    units: &mut BTreeMap<String, PackageSyntax>,
    entry: &str,
    scenario: &ComponentScenario,
    available_packages: &BTreeSet<&str>,
) -> Result<WidgetSiteTargets, PackageDiagnostic> {
    let mut sites = WidgetSiteTargets::new();
    if !components
        .iter()
        .any(|component| component.node.params.iter().any(needs_type))
    {
        return Ok(sites);
    }
    let (entry_package, _) = entry.split_once("::").expect("bound entry module");

    let mut projections = BTreeMap::<String, WidgetListProjection>::new();
    walk_widget_sites(components, available_packages, scenario.instances.max(1), &sites, |site, target| {
        for param in target.node.params.iter().filter(|param| needs_type(param)) {
            if let Some(field) = supplied_model_field(&site, &param.name) {
                if let Some(data) = site
                    .component
                    .node
                    .view_data
                    .iter()
                    .find(|data| data.name == field.name)
                {
                    projections
                        .entry(site.component.module.clone())
                        .or_default()
                        .model_fields
                        .insert(data.span.start);
                }
            }
        }
        Ok(())
    })?;
    for (module, projection) in &projections {
        reproject(
            module,
            projection,
            components,
            units,
            entry_package,
            scenario,
        )?;
    }
    let mut variants = BTreeMap::<String, Vec<Option<ListSite>>>::new();
    walk_widget_sites(components, available_packages, scenario.instances.max(1), &sites, |site, target| {
        if target.module == entry || units[&target.module].age_components.len() != 1
            || !target.node.params.iter().any(needs_type)
        {
            return Ok(());
        }
        let exact = site.loops.is_empty().then(|| {
            let inputs = target.node.params.iter().filter(|param| needs_type(param)).map(|param| {
                let field = supplied_model_field(&site, &param.name)?;
                let ty = caller_list_type(site.component, field, units)?;
                Some((ty, param.ty_span))
            }).collect::<Option<Vec<_>>>()?;
            let selectors = target.node.params.iter().filter_map(|param| {
                let value = supplied_value(&site, &param.name).or_else(|| param.default.as_ref())?;
                Some((param.ty_span.start, SelectorInput::from_expr(value)?))
            }).collect();
            Some(ListSite { caller: site.component.module.clone(), span: site.element.span.start, inputs, selectors })
        }).flatten();
        variants.entry(target.module.clone()).or_default().push(exact);
        Ok(())
    })?;
    variants.retain(|_, group| group.len() > 1 && group.iter().all(Option::is_some));

    let mut inferred = BTreeMap::<WidgetParamKey, (String, TypeAnn, Span)>::new();
    let mut computed_defaults = BTreeMap::new();
    let mut explicitly_supplied = BTreeSet::new();
    let mut callers = BTreeMap::<WidgetParamKey, BTreeSet<String>>::new();
    let mut selectors = BTreeMap::<WidgetParamKey, Option<SelectorInput>>::new();
    walk_widget_sites(components, available_packages, scenario.instances.max(1), &sites, |site, target| {
        if variants.contains_key(&target.module) {
            return Ok(());
        }
        // The host can supply an entry widget's values outside authored sites.
        if target.module != entry && target.node.params.iter().any(needs_type) {
            for param in &target.node.params {
                let key = (target.module.clone(), param.ty_span.start);
                let value = supplied_value(&site, &param.name)
                    .or_else(|| param.default.as_ref())
                    .and_then(SelectorInput::from_expr);
                selectors
                    .entry(key)
                    .and_modify(|known| {
                        if !known
                            .as_mut()
                            .zip(value.as_ref())
                            .is_some_and(|(known, supplied)| known.merge(supplied))
                        {
                            *known = None;
                        }
                    })
                    .or_insert(value);
            }
        }
        for param in target.node.params.iter().filter(|param| needs_type(param)) {
            let key = (target.module.clone(), param.ty_span.start);
            let caller = &site.component.module;
            callers
                .entry(key.clone())
                .or_default()
                .insert(caller.clone());
            if inferred.contains_key(&key) {
                continue;
            }
            if supplied_value(&site, &param.name).is_some() {
                explicitly_supplied.insert(key.clone());
            } else if let Some(ty) = computed_argument_list_type(target, param, units) {
                computed_defaults.insert(key.clone(), (target.module.clone(), ty, param.ty_span));
            }
            let field = supplied_model_field(&site, &param.name);
            if let Some(ty) = field.and_then(|field| caller_list_type(site.component, field, units)) {
                inferred.insert(key, (caller.clone(), ty, param.ty_span));
            }
        }
        Ok(())
    })?;
    // With no authored value at any site, a computed call can give the
    // empty default its list type. Supplied values still take precedence.
    inferred.extend(computed_defaults.into_iter().filter(|(key, _)| !explicitly_supplied.contains(key)));

    let mut typed_widgets = BTreeSet::new();
    let mut records = RecordOrigins::new();
    for ((module, offset), (source, ty, param_span)) in inferred {
        let projection = projections.entry(module.clone()).or_default();
        if let Some((alias, name)) = projection.bind_type(&units[&source], &source, ty, param_span) {
            let binding = (source, name);
            for unit in callers[&(module.clone(), offset)].iter().chain([&module]) {
                records
                    .entry(unit.clone())
                    .or_default()
                    .insert(alias.clone(), binding.clone());
            }
        }
        typed_widgets.insert(module);
    }

    for ((module, offset), value) in selectors {
        if let Some(value) = value.filter(|_| typed_widgets.contains(&module)) {
            projections
                .entry(module)
                .or_default()
                .selectors
                .insert(offset, value);
        }
    }
    for module in typed_widgets {
        reproject(
            &module,
            &projections[&module],
            components,
            units,
            entry_package,
            scenario,
        )?;
    }
    for (module, group) in variants {
        let model_fields = projections.remove(&module).unwrap_or_default().model_fields;
        for (ordinal, site) in group.into_iter().flatten().enumerate() {
            let variant = components.len();
            let mut projection = WidgetListProjection {
                model_fields: model_fields.clone(),
                selectors: site.selectors,
                specialized_name: (ordinal > 0).then(|| format!("{AGE_PACKAGE_HIDDEN_PREFIX}widget_site_{variant}")),
                ..Default::default()
            };
            for (ty, span) in site.inputs {
                if let Some((alias, name)) = projection.bind_type(&units[&site.caller], &site.caller, ty, span) {
                    for unit in [&site.caller, &module] {
                        records.entry(unit.clone()).or_default()
                            .insert(alias.clone(), (site.caller.clone(), name.clone()));
                    }
                }
            }
            if ordinal == 0 {
                reproject(&module, &projection, components, units, entry_package, scenario)?;
            } else {
                let fresh = project_variant(&module, &units[&module], &projection, variant, entry_package, scenario)?;
                append_variant(&module, fresh, components, units);
                sites.insert((site.caller, site.span), variant);
            }
        }
    }
    for (module, bindings) in records {
        units
            .get_mut(&module)
            .expect("widget input module was parsed")
            .age_widget_record_types = bindings;
    }
    Ok(sites)
}

fn reproject(
    module: &str,
    projection: &WidgetListProjection,
    components: &mut [PackagedAgeComponent],
    units: &mut BTreeMap<String, PackageSyntax>,
    entry_package: &str,
    scenario: &ComponentScenario,
) -> Result<(), PackageDiagnostic> {
    let unit = units.get_mut(module).expect("widget module was parsed");
    let node_base = components
        .iter()
        .position(|component| component.module == module)
        .expect("typed module has a widget declaration");
    *unit = project_variant(module, unit, projection, node_base, entry_package, scenario)?;
    for (component, (node, text_fns, scene_fns)) in components[node_base..]
        .iter_mut()
        .take_while(|component| component.module == module)
        .zip(&unit.age_components)
    {
        component.node = node.clone();
        component.text_fns = text_fns.clone();
        component.scene_fns = scene_fns.clone();
    }
    Ok(())
}

fn project_variant(
    module: &str,
    unit: &PackageSyntax,
    projection: &WidgetListProjection,
    node_base: usize,
    entry_package: &str,
    scenario: &ComponentScenario,
) -> Result<PackageSyntax, PackageDiagnostic> {
    // Specializations use the retained source and never create a new module.
    let piece = unit.pieces.first().expect("widget has an original syntax piece");
    let tree = piece.tree.clone().expect("widget retains its CST");
    let (package, _) = module.split_once("::").expect("bound module key");
    project_packages_front_with_widget_lists(
        &piece.source,
        vec![tree],
        piece.tokens.clone(),
        PackagesFace::Age {
            defer_styles: package != entry_package,
        },
        scenario,
        node_base,
        Some(projection),
        None,
    )
    .map_err(|mut diagnostics| PackageDiagnostic {
        module: module.into(),
        diagnostic: diagnostics.remove(0),
    })
}

fn append_variant(
    module: &str,
    mut fresh: PackageSyntax,
    components: &mut Vec<PackagedAgeComponent>,
    units: &mut BTreeMap<String, PackageSyntax>,
) {
    fn declaration(item: &Item) -> Option<&str> {
        match item {
            Item::Fn(function) => Some(&function.name.name),
            Item::Type(record) => Some(&record.name.name),
            _ => None,
        }
    }
    let unit = units.get_mut(module).expect("original widget module");
    let (node, text_fns, scene_fns) = fresh.age_components.pop().expect("specialized widget");
    let existing: BTreeSet<_> = unit.program.items.iter().filter_map(declaration).map(str::to_owned).collect();
    unit.program.items.extend(fresh.program.items.into_iter().filter(|item| {
        declaration(item).is_some_and(|name| !existing.contains(name))
    }));
    unit.age_components.push((node.clone(), text_fns.clone(), scene_fns.clone()));
    components.push(PackagedAgeComponent {
        module: module.into(), node, module_imports: unit.imports.clone(), text_fns, scene_fns,
    });
}
