use std::collections::BTreeMap;

use super::{
    execute_bound_widget, widget_component, widget_view_fn_name_at, AgePackageCompilation,
    CalleeRef, ComponentFieldValue, ExecLimits, MirInstr, MirModule, Value,
};

#[derive(Clone, Debug, PartialEq)]
pub enum WidgetViewItem {
    Text(String),
    Element(WidgetElement),
    Timer(WidgetTimer),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetTimer {
    pub message: String,
    pub every_ms: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetElement {
    pub tag: String,
    pub site: usize,
    pub label: Option<String>,
    pub properties: Vec<(String, String)>,
    pub events: Vec<WidgetEvent>,
    pub children: Vec<WidgetViewItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetEvent {
    pub key: String,
    pub message: String,
    pub args: Vec<ComponentFieldValue>,
}

// A widget value is invoked through one entry. Sibling projections can carry
// independent copies of large authored models and need not be packaged again
// for this invocation. Retain initializer roots and closure constructor targets
// alongside normal callees. The resulting module still goes through admission.
pub(super) fn retain_entry_closure(mir: &mut MirModule) -> Result<(), String> {
    let mut retained = vec![false; mir.funcs.len()];
    let mut pending = mir.init.clone();
    pending.push(mir.entry);
    while let Some(index) = pending.pop() {
        let selected = retained
            .get_mut(index as usize)
            .ok_or_else(|| invalid("widget function index out of bounds"))?;
        if *selected {
            continue;
        }
        *selected = true;
        pending.extend(
            mir.funcs[index as usize]
                .blocks
                .iter()
                .flat_map(|block| &block.instrs)
                .filter_map(|instruction| match instruction {
                    MirInstr::Call {
                        callee: CalleeRef::Func(target),
                        ..
                    }
                    | MirInstr::CreateClosure {
                        callee: CalleeRef::Func(target),
                        ..
                    } => Some(*target),
                    _ => None,
                }),
        );
    }
    let mut next = 0;
    let mapping: Vec<_> = retained
        .iter()
        .map(|&keep| {
            keep.then(|| {
                let index = next;
                next += 1;
                index
            })
        })
        .collect();
    let mut retained = retained.into_iter();
    mir.funcs.retain(|_| {
        retained
            .next()
            .expect("mapping matches original function table")
    });
    let remap = |index: &mut u32| {
        *index = mapping
            .get(*index as usize)
            .copied()
            .flatten()
            .ok_or_else(|| invalid("widget function target is outside its entry closure"))?;
        Ok::<(), String>(())
    };
    remap(&mut mir.entry)?;
    for index in &mut mir.init {
        remap(index)?;
    }
    for instruction in mir
        .funcs
        .iter_mut()
        .flat_map(|function| &mut function.blocks)
        .flat_map(|block| &mut block.instrs)
    {
        match instruction {
            MirInstr::Call {
                callee: CalleeRef::Func(target),
                ..
            }
            | MirInstr::CreateClosure {
                callee: CalleeRef::Func(target),
                ..
            } => remap(target)?,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn entry_closure_retains_initializers_and_remaps_closure_bodies() {
    use ace_av_core::mir::{MirBlock, MirFunction, OwnerScope, Terminator};

    let owner = OwnerScope {
        schema: "mir".into(),
        module: "widget".into(),
    };
    let function = |name: &str, instrs| MirFunction {
        owner: owner.clone(),
        name: name.into(),
        sig: 0,
        slots: Vec::new(),
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };
    let call = |target| MirInstr::Call {
        callee: CalleeRef::Func(target),
        dst: 0,
        args: Vec::new(),
    };
    let mut mir = MirModule::new(owner.clone());
    mir.funcs = vec![
        function("discard", vec![]),
        function("first", vec![]),
        function("discard", vec![]),
        function("second", vec![call(1)]),
        function("closure", vec![]),
        function("discard", vec![]),
        function(
            "helper",
            vec![MirInstr::CreateClosure {
                dst: 0,
                callee: CalleeRef::Func(4),
                captures: Vec::new(),
            }],
        ),
        function("entry", vec![call(6)]),
    ];
    mir.entry = 7;
    mir.init = vec![3, 1];

    retain_entry_closure(&mut mir).unwrap();

    let names: Vec<_> = mir.funcs.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["first", "second", "closure", "helper", "entry"]);
    assert_eq!(mir.entry, 4);
    assert_eq!(mir.init, [1, 0]);
    assert!(matches!(
        mir.funcs[1].blocks[0].instrs[0],
        MirInstr::Call {
            callee: CalleeRef::Func(0),
            ..
        }
    ));
    assert!(matches!(
        mir.funcs[3].blocks[0].instrs[0],
        MirInstr::CreateClosure {
            callee: CalleeRef::Func(2),
            ..
        }
    ));
    assert!(matches!(
        mir.funcs[4].blocks[0].instrs[0],
        MirInstr::Call {
            callee: CalleeRef::Func(3),
            ..
        }
    ));
}

/// The wrapper elements retain the author's child sites. Consumers can use
/// those boundaries to route events through the same checked bindings.
pub(super) fn compose(
    compiled: &AgePackageCompilation,
    mir: &MirModule,
    instance: usize,
    limits: &ExecLimits,
    mut items: Vec<WidgetViewItem>,
) -> Result<Vec<WidgetViewItem>, String> {
    let root = compiled
        .description
        .components
        .iter()
        .find(|component| component.module == compiled.description.entry)
        .ok_or_else(|| invalid("entry component is missing"))?;
    let owner = (root.module.clone(), root.name.clone());
    let mut view = Composition {
        compiled,
        mir,
        instance,
        limits,
        active: vec![owner.clone()],
        scoped: BTreeMap::new(),
    };
    view.expand(&owner, &mut items)?;
    Ok(items)
}

struct Composition<'a> {
    compiled: &'a AgePackageCompilation,
    mir: &'a MirModule,
    instance: usize,
    limits: &'a ExecLimits,
    active: Vec<(String, String)>,
    scoped: BTreeMap<ScopedSite, Vec<WidgetViewItem>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ScopedSite {
    owner: (String, String),
    site: usize,
    properties: Vec<(String, String)>,
}

impl Composition<'_> {
    fn expand(
        &mut self,
        owner: &(String, String),
        items: &mut [WidgetViewItem],
    ) -> Result<(), String> {
        for item in items {
            let WidgetViewItem::Element(element) = item else {
                continue;
            };
            let site = self
                .compiled
                .widget_refs
                .get(owner)
                .into_iter()
                .flatten()
                .find(|site| site.span.start == element.site && site.authored_node == element.tag);
            let Some(site) = site else {
                self.expand(owner, &mut element.children)?;
                continue;
            };
            if !element.children.is_empty() {
                return Err(invalid(
                    "projecting child content through a widget slot is unsupported",
                ));
            }
            let target = (site.module.clone(), site.node.clone());
            if self.active.contains(&target) {
                return Err(invalid("recursive widget composition"));
            }
            let key = ScopedSite {
                owner: owner.clone(),
                site: element.site,
                properties: element.properties.clone(),
            };
            if let Some(items) = self.scoped.get(&key) {
                element.children = items.clone();
                continue;
            }
            let (node, _) = widget_component(self.compiled, site)?;
            let name = widget_view_fn_name_at(node, self.instance);
            let symbol = self
                .compiled
                .symbol_of_in(&site.module, &name)
                .ok_or_else(|| invalid("missing checked child view"))?;
            let def_id = format!("d_{symbol}");
            let entry = self
                .mir
                .funcs
                .iter()
                .position(|function| function.name == def_id)
                .ok_or_else(|| invalid("missing lowered child view"))?;
            let mut mir = self.mir.clone();
            mir.entry = entry as u32;
            let result = execute_bound_widget(
                self.compiled,
                site,
                self.instance,
                &element.properties,
                self.limits,
                mir,
                &def_id,
            )?;
            let mut items = decode_view(result)?;
            self.active.push(target.clone());
            self.expand(&target, &mut items)?;
            self.active.pop();
            element.children = items.clone();
            self.scoped.insert(key, items);
        }
        Ok(())
    }
}

pub(super) fn decode_view(result: Value) -> Result<Vec<WidgetViewItem>, String> {
    let Value::List(list) = result else {
        return Err(invalid("expected a list"));
    };
    let list = list.rc.borrow();
    let mut cells = list.elems.iter();
    let mut frames: Vec<WidgetElement> = Vec::new();
    let mut root = Vec::new();
    while let Some(cell) = cells.next() {
        let operation = text(cell)?;
        match operation {
            "element" => {
                let tag = next(&mut cells)?.to_owned();
                let site = number(next(&mut cells)?)?;
                frames.push(WidgetElement {
                    tag,
                    site,
                    label: None,
                    properties: Vec::new(),
                    events: Vec::new(),
                    children: Vec::new(),
                });
            }
            "end" => {
                let element = frames.pop().ok_or_else(|| invalid("unexpected end"))?;
                if element.tag != next(&mut cells)? {
                    return Err(invalid("unbalanced elements"));
                }
                children(&mut root, &mut frames).push(WidgetViewItem::Element(element));
            }
            "text" => {
                children(&mut root, &mut frames)
                    .push(WidgetViewItem::Text(next(&mut cells)?.to_owned()));
            }
            "timer" if frames.is_empty() => {
                let message = next(&mut cells)?.to_owned();
                let every_ms = next(&mut cells)?.parse::<u32>().ok()
                    .filter(|ms| (1..=i32::MAX as u32).contains(ms))
                    .ok_or_else(|| invalid("invalid timer interval"))?;
                root.push(WidgetViewItem::Timer(WidgetTimer { message, every_ms }));
            }
            "label" | "property" | "event" => {
                let current = frames
                    .last_mut()
                    .ok_or_else(|| invalid("attribute without an element"))?;
                match operation {
                    "label" => current.label = Some(next(&mut cells)?.to_owned()),
                    "property" => current
                        .properties
                        .push((next(&mut cells)?.to_owned(), next(&mut cells)?.to_owned())),
                    "event" => current.events.push(event(&mut cells)?),
                    _ => unreachable!(),
                }
            }
            _ => return Err(invalid("unknown operation")),
        }
    }
    if !frames.is_empty() {
        return Err(invalid("missing element end"));
    }
    Ok(root)
}

fn children<'a>(
    root: &'a mut Vec<WidgetViewItem>,
    frames: &'a mut [WidgetElement],
) -> &'a mut Vec<WidgetViewItem> {
    frames
        .last_mut()
        .map_or(root, |element| &mut element.children)
}

fn event<'a>(cells: &mut impl Iterator<Item = &'a Value>) -> Result<WidgetEvent, String> {
    let key = next(cells)?.to_owned();
    let message = next(cells)?.to_owned();
    let count = number(next(cells)?)?;
    let mut args = Vec::new();
    for _ in 0..count {
        let ty = next(cells)?;
        let value = next(cells)?;
        args.push(match ty {
            "str" | "Str" => ComponentFieldValue::Str(value.to_owned()),
            "bool" => ComponentFieldValue::Bool(
                value
                    .parse()
                    .map_err(|_| invalid("invalid boolean payload"))?,
            ),
            "int" | "i32" => ComponentFieldValue::Int(
                value
                    .parse()
                    .map_err(|_| invalid("invalid integer payload"))?,
            ),
            _ => return Err(invalid("unknown payload type")),
        });
    }
    Ok(WidgetEvent { key, message, args })
}

fn text(cell: &Value) -> Result<&str, String> {
    let Value::Str(value) = cell else {
        return Err(invalid("expected text"));
    };
    std::str::from_utf8(value.bytes()).map_err(|_| invalid("invalid UTF-8"))
}

fn next<'a>(cells: &mut impl Iterator<Item = &'a Value>) -> Result<&'a str, String> {
    text(cells.next().ok_or_else(|| invalid("truncated output"))?)
}

fn number(value: &str) -> Result<usize, String> {
    value.parse().map_err(|_| invalid("invalid integer"))
}

fn invalid(why: &str) -> String {
    format!("source.age.widget-view-output: {why}")
}
