//! PLAN-981（SOURCE 批，IF-981-MODULE）：AGE 包面的 **AV 消费端**。
//!
//! 输入是包前门的受检结论（`ace_core::source::age_package`）：每个文本叶/每个
//! 实例字段都有一个由共享 verify 检查过的**普通**函数（隐藏求值函数 / 状态
//! 函数）。本消费者按 def id 走共享
//! `lower → MIR verify → ABC 编解码 → AV 执行`得到真实值，再按实例组成可观察
//! 结构（实例序状态 + 逐实例文本）。
//!
//! 不是第二个解释器：不把表达式当字符串再解析，不用 Rust 常量补值；换被导入
//! helper 的返回值必须改变观察结果。

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag};
pub use ace_av_core::av::ExecLimits;
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};
use ace_av_core::mir::{
    CalleeRef, FnSig, MirBlock, MirFieldDef, MirFunction, MirInstr, MirModule, MirSlot,
    MirConst, MirType, RecordTypeRef, SlotKind, Terminator,
};
use ace_core::source::age_package::{
    computed_fn_name_at, widget_timer_fn_name_at, widget_view_fn_name_at, AgePackageCompilation,
    AgeWidgetProperty, AgeWidgetRef,
};

mod live_view;
pub mod live_session;
pub use live_view::{WidgetElement, WidgetEvent, WidgetTimer, WidgetViewItem};
use ace_lowering::lower_checked_age_package;
use crate::age_component::ComponentFieldValue;

/// 一次 AGE 包运行的真实观察。
#[derive(Debug, Clone)]
pub struct ObservedAgePackage {
    /// 逐组件的观察行（组件声明序）。
    pub components: Vec<ObservedAgeComponent>,
}

/// 一个组件的观察结果。
#[derive(Debug, Clone)]
pub struct ObservedAgeComponent {
    pub name: String,
    pub module: String,
    /// 逐实例的文本叶值（实例序，每项按叶声明序）。
    pub instance_texts: Vec<Vec<String>>,
    /// 逐实例的 `model` int 字段值：`[实例][字段声明序]`。
    pub instance_states: Vec<Vec<i32>>,
    /// The scalar model snapshot, with field names in declaration order.
    pub instance_scalar_states: Vec<Vec<(String, ComponentFieldValue)>>,
    /// 逐实例的 `on` 消息名（作者拼写，含 `Life.Tick` 这类限定名）。
    pub handler_msgs: Vec<String>,
    /// PLAN-981 B-W1（IF-981-PROOF-AV）：场景属性值函数的 AV 真值（按
    /// `description.components[].scene_values` 登记序一一对应——含被具名
    /// 跳过的参数化行，不静默丢行）。场景值函数按**组件**发射（本切片场景
    /// 不按实例复制），故不按实例分组。
    pub scene_values: Vec<ObservedSceneValue>,
}

/// 一条场景属性值的观察结果（与 `AgeSceneValueFn` 登记序对位）。
#[derive(Debug, Clone, PartialEq)]
pub struct ObservedSceneValue {
    /// 场景行实例名（`red_disc` / `bar`）。
    pub row: String,
    /// 属性名（`position` / `x`）。
    pub attr: String,
    /// 元组属性的第几分量（单值属性为 `None`）。
    pub component: Option<usize>,
    /// 执行结果；参数化行（引用 node 参数）不执行、具名跳过（不猜实参）。
    pub value: SceneObserved,
}

/// 场景属性值的 AV 执行结果。
#[derive(Debug, Clone, PartialEq)]
pub enum SceneObserved {
    F32(f32),
    I32(i32),
    Bool(bool),
    /// 参数化值函数：无实例实参不可执行（登记保留，值面看 `cpu`/`attrs`）。
    Parametric,
}

/// 观察 AGE 包：编译结果里的每个实例函数走共享链取真值。
///
/// `instances` 必须与编译时给出的场景实例数一致（状态函数按实例生成；不一致
/// 是具名错误，不静默返回空表）。
pub fn observe_age_package(
    compiled: &AgePackageCompilation,
    instances: usize,
) -> Result<ObservedAgePackage, String> {
    if let Some(widget) = compiled.first_widget_ref() {
        return Err(format!(
            "source.age.av-widget-unsupported: cannot observe checked widget `{}` from `{}`",
            widget.node, widget.module
        ));
    }
    // PLAN-981（view-if 第二片）：包观察按平铺文本行逐叶取值——含条件分支
    // 时两支会被当作同时可见。包编译/绑定检查保持通过（平铺进 texts 是主线
    // 设计）；观察入口见描述面 `cond_spans` 非空即具名拒（锚作者最前 `if`）。
    if let Some(span) = compiled
        .description
        .components
        .iter()
        .find_map(|c| c.cond_spans.first().copied())
    {
        let d = ace_core::source::age_component::view_conditional_observation_diag(span);
        return Err(format!(
            "{} @{}..{}: {}",
            d.code, d.span.start, d.span.end, d.message
        ));
    }
    observe_age_package_layout_inputs(compiled, instances)
}

/// Evaluate checked state and text slots for a renderer that applies branch visibility.
///
/// The returned text rows contain every declared slot in source order, including
/// inactive branches. They are layout inputs and must be filtered using the view
/// tree and corresponding instance state before presentation.
pub fn observe_age_package_layout_inputs(
    compiled: &AgePackageCompilation,
    instances: usize,
) -> Result<ObservedAgePackage, String> {
    if let Some(widget) = compiled.first_widget_ref() {
        return Err(format!(
            "source.age.av-widget-unsupported: cannot observe checked widget `{}` from `{}`",
            widget.node, widget.module
        ));
    }
    let mut components = Vec::new();
    for (component_index, c) in compiled.description.components.iter().enumerate() {
        if c.texts.is_empty() {
            // R70-1：有 model 字段却无文本叶的组件，AV 在本切片里**看不到**它的状态
            // （场景节点），静默给空行会让消费者把"没观察"当成"状态为 0"。具名拒。
            if !c.fields.is_empty() {
                return Err(format!(
                    "component `{}` declares model fields but has no text leaves; the AV \
                     observation cannot surface its state in this slice (named rejection, \
                     not a silent empty row)",
                    c.name
                ));
            }
            // PLAN-981 B-W1（IF-981-PROOF-AV）：纯场景组件（无文本叶、无 model）
            // 现在给出场景值函数的 AV 真值——不再只回空行；"混合 UI+2D/3D 在
            // AV 具名拒 F32"的旧边界由 owned 车道的 f32 映射关闭。
            let scene_values = observe_scene_values(compiled, c)?;
            components.push(ObservedAgeComponent {
                name: c.name.clone(),
                module: c.module.clone(),
                instance_texts: Vec::new(),
                instance_states: Vec::new(),
                instance_scalar_states: Vec::new(),
                handler_msgs: c.handler_msgs.clone(),
                scene_values,
            });
            continue;
        }
        // 叶按实例分组：编译期为每个 (实例, 叶) 各生成一份求值函数。
        let per_instance = if instances == 0 { 1 } else { instances };
        if instances > 0 && c.texts.len() % per_instance != 0 {
            return Err(format!(
                "component `{}` has {} value functions which is not a multiple of {} instances",
                c.name,
                c.texts.len(),
                per_instance
            ));
        }
        let instance_texts = c
            .texts
            .chunks_exact(c.texts.len() / per_instance)
            .map(|row| {
                row.iter()
                    .map(|text| execute_text(&compiled.checked, &text.def_id))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut instance_scalar_states = Vec::with_capacity(per_instance);
        for inst in 0..per_instance {
            instance_scalar_states.push(observe_package_scalar_states(compiled, component_index, inst)?);
        }
        let instance_states = instance_scalar_states.iter().map(|row| {
            row.iter().filter_map(|(_, value)| match value {
                ComponentFieldValue::Int(value) => Some(*value),
                _ => None,
            }).collect()
        }).collect();
        // PLAN-981 B-W1（IF-981-PROOF-AV）：混合包的 2D/3D 场景值与 UI 文本/
        // 状态同一次观察（同一 AV 车道，不再整模块拒 F32）。
        let scene_values = observe_scene_values(compiled, c)?;
        components.push(ObservedAgeComponent {
            name: c.name.clone(),
            module: c.module.clone(),
            instance_texts,
            instance_states,
            instance_scalar_states,
            handler_msgs: c.handler_msgs.clone(),
            scene_values,
        });
    }
    Ok(ObservedAgePackage { components })
}

/// Read a component's scalar model without running its view or observing child
/// views. An input-bound component still needs its checked property arguments.
pub fn observe_package_scalar_states(
    compiled: &AgePackageCompilation,
    index: usize,
    instance: usize,
) -> Result<Vec<(String, ComponentFieldValue)>, String> {
    observe_package_scalar_states_with_limits(compiled, index, instance, &ExecLimits::default())
}

/// Read scalar states under the host's checked execution limits.
pub fn observe_package_scalar_states_with_limits(
    compiled: &AgePackageCompilation,
    index: usize,
    instance: usize,
    limits: &ExecLimits,
) -> Result<Vec<(String, ComponentFieldValue)>, String> {
    observe_package_scalar_fields(compiled, index, instance, None, limits)
}

/// Read only the scalar fields needed by the host, in declaration order.
pub fn observe_package_scalar_fields(
    compiled: &AgePackageCompilation,
    index: usize,
    instance: usize,
    fields: Option<&[&str]>,
    limits: &ExecLimits,
) -> Result<Vec<(String, ComponentFieldValue)>, String> {
    let component = compiled.description.components.get(index)
        .ok_or("unknown package component")?;
    let scalars = component.fields.iter().filter(|(name, ty)| {
        fields.is_none_or(|wanted| wanted.contains(&name.as_str()))
            && matches!(ty.as_str(), "int" | "i32" | "str" | "Str" | "bool" | "f32" | "float")
    });
    let Some((first, _)) = scalars.clone().next() else {
        return Ok(Vec::new());
    };
    let state_definition = |field: &str| {
        let name = ace_core::source::age_component::state_fn_name_at(index, instance, field);
        resolve_symbol(compiled, &component.module, &name).map(|symbol| format!("d_{symbol}"))
    };
    let first = state_definition(first)?;
    // Lower the package once. Each getter verifies and runs only its entry
    // closure, just as the view and timer projections do.
    let mut mir = lower_checked_age_package(&compiled.checked, &first, 0x981)
        .map_err(|error| format!("lower `{first}`: {error:?}"))?;
    let mut row = Vec::new();
    for (field, ty) in scalars {
        let definition = state_definition(field)?;
        let entry = mir.funcs.iter().position(|function| function.name == definition)
            .ok_or_else(|| format!("missing lowered state getter `{definition}`"))?;
        mir.entry = entry as u32;
        let value = execute_widget_value(mir.clone(), &definition, limits)?;
        let value = match (ty.as_str(), value) {
            ("int" | "i32", Value::I32(value)) => ComponentFieldValue::Int(value),
            ("bool", Value::Bool(value)) => ComponentFieldValue::Bool(value),
            ("f32" | "float", Value::F32(value)) => ComponentFieldValue::F32(value),
            ("str" | "Str", Value::Str(value)) => String::from_utf8(value.bytes().to_vec())
                .map(ComponentFieldValue::Str)
                .map_err(|error| format!("state `{field}` text is not UTF-8: {error}"))?,
            (_, other) => return Err(format!("state `{field}` declared `{ty}` but produced {other:?}")),
        };
        row.push((field.clone(), value));
    }
    Ok(row)
}

/// PLAN-981 B-W1（IF-981-PROOF-AV）：逐条执行场景属性值函数（登记序对位）。
/// 参数化行（形参数 > 0）具名跳过——不猜实参、不填默认值。
fn observe_scene_values(
    compiled: &AgePackageCompilation,
    c: &ace_core::source::age_package::AgePackageComponent,
) -> Result<Vec<ObservedSceneValue>, String> {
    let mut out = Vec::new();
    for f in &c.scene_values {
        let value = if f.params > 0 {
            SceneObserved::Parametric
        } else {
            let symbol = resolve_symbol(compiled, &c.module, &f.fn_name)?;
            match execute_value(&compiled.checked, &format!("d_{symbol}"))? {
                Value::F32(v) => SceneObserved::F32(v),
                Value::I32(v) => SceneObserved::I32(v),
                Value::Bool(v) => SceneObserved::Bool(v),
                other => {
                    return Err(format!(
                        "scene value `{}.{}` did not produce a scene scalar: {other:?}",
                        f.row, f.attr
                    ))
                }
            }
        };
        out.push(ObservedSceneValue {
            row: f.row.clone(),
            attr: f.attr.clone(),
            component: f.component,
            value,
        });
    }
    Ok(out)
}

/// Execute a checked scene, text or state value function.
fn execute_value(
    checked: &ace_core::verify::CheckedModule,
    def_id: &str,
) -> Result<Value, String> {
    let mir = lower_checked_age_package(checked, def_id, 0x981)
        .map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
    execute_value_module(&mir, def_id, &ExecLimits::default())
}

/// Read a checked non-loop widget argument or its authored default.
pub fn evaluate_widget_property(
    compiled: &AgePackageCompilation,
    property: &AgeWidgetProperty,
    instance: usize,
) -> Result<Value, String> {
    let def_id = widget_property_def(compiled, property, instance)?;
    let mir = lower_checked_age_package(&compiled.checked, &def_id, 0x981)
        .map_err(|err| format!("lower `{def_id}`: {err:?}"))?;
    execute_widget_value(mir, &def_id, &ExecLimits::default())
}

fn widget_property_def(
    compiled: &AgePackageCompilation,
    property: &AgeWidgetProperty,
    instance: usize,
) -> Result<String, String> {
    let AgeWidgetProperty::Value { module, evaluators, .. } = property else {
        return Err("source.age.av-widget-property: this binding needs the view or event scope".into());
    };
    let evaluator = evaluators.get(instance).ok_or_else(|| {
        format!("source.age.av-widget-property: unknown property instance {instance}")
    })?;
    Ok(format!("d_{}", resolve_symbol(compiled, module, evaluator)?))
}

/// Evaluate a computed value against one parent instance and its bound child site.
pub fn evaluate_widget_computed(
    compiled: &AgePackageCompilation,
    widget: &AgeWidgetRef,
    instance: usize,
    field: &str,
    limits: &ExecLimits,
) -> Result<Value, String> {
    let (node, component) = widget_component(compiled, widget)?;
    if !component.members.iter().any(|member| member.role == "computed" && member.name == field) {
        return Err(format!("source.age.av-widget-property: unknown computed field `{field}`"));
    }
    evaluate_widget_function(compiled, widget, instance, &computed_fn_name_at(node, instance, field), &[], limits)
}

/// Materialize the authored widget view using verified package execution.
pub fn evaluate_widget_view(
    compiled: &AgePackageCompilation,
    widget: &AgeWidgetRef,
    instance: usize,
    limits: &ExecLimits,
) -> Result<Vec<WidgetViewItem>, String> {
    let (node, _) = widget_component(compiled, widget)?;
    let name = widget_view_fn_name_at(node, instance);
    let result = evaluate_widget_function(compiled, widget, instance, &name, &[], limits)?;
    live_view::decode_view(result)
}

/// Execute a child site whose scalar values were emitted by the parent's
/// checked view at this occurrence of its enclosing loop.
pub fn evaluate_scoped_widget_view(
    compiled: &AgePackageCompilation,
    widget: &AgeWidgetRef,
    instance: usize,
    properties: &[(String, String)],
    limits: &ExecLimits,
) -> Result<Vec<WidgetViewItem>, String> {
    let (node, _) = widget_component(compiled, widget)?;
    let name = widget_view_fn_name_at(node, instance);
    let result = evaluate_widget_function(compiled, widget, instance, &name, properties, limits)?;
    live_view::decode_view(result)
}

/// Materialize the entry view. Its child sites retain their checked package
/// bindings so the host can assemble them with their own instance state.
pub fn evaluate_package_entry_view(
    compiled: &AgePackageCompilation,
    instance: usize,
    limits: &ExecLimits,
) -> Result<Vec<WidgetViewItem>, String> {
    let (mir, def_id) = lower_package_entry_view(compiled, instance)?;
    live_view::decode_view(execute_widget_value(mir, &def_id, limits)?)
}

/// Evaluate the entry widget's checked timer condition against this scenario.
pub fn evaluate_package_timer(
    compiled: &AgePackageCompilation,
    node: usize,
    instance: usize,
    timer: usize,
    limits: &ExecLimits,
) -> Result<bool, String> {
    let owner = compiled.description.components.get(node)
        .filter(|owner| owner.module == compiled.description.entry)
        .ok_or("source.age.av-timer: the timer owner is not the entry widget")?;
    let name = widget_timer_fn_name_at(node, instance, timer);
    let symbol = compiled.symbol_of_in(&owner.module, &name)
        .ok_or("source.age.av-timer: missing checked condition")?;
    let def_id = format!("d_{symbol}");
    let mir = lower_checked_age_package(&compiled.checked, &def_id, 0x981)
        .map_err(|err| format!("lower `{def_id}`: {err:?}"))?;
    match execute_widget_value(mir, &def_id, limits)? {
        Value::Bool(enabled) => Ok(enabled),
        _ => Err("source.age.av-timer: condition did not return bool".into()),
    }
}

fn lower_package_entry_view(
    compiled: &AgePackageCompilation,
    instance: usize,
) -> Result<(MirModule, String), String> {
    let (index, root) = compiled.description.components.iter().enumerate()
        .find(|(_, component)| component.module == compiled.description.entry)
        .ok_or("source.age.widget-view-output: the entry module has no component")?;
    let name = widget_view_fn_name_at(index, instance);
    let symbol = compiled.symbol_of_in(&root.module, &name)
        .ok_or("source.age.widget-view-output: the entry component has no checked view")?;
    let def_id = format!("d_{symbol}");
    let mir = lower_checked_age_package(&compiled.checked, &def_id, 0x981)
        .map_err(|err| format!("lower `{def_id}`: {err:?}"))?;
    Ok((mir, def_id))
}

/// Assemble the entry and its checked child views for one package instance.
pub fn evaluate_package_view(
    compiled: &AgePackageCompilation,
    instance: usize,
    limits: &ExecLimits,
) -> Result<Vec<WidgetViewItem>, String> {
    let (mir, def_id) = lower_package_entry_view(compiled, instance)?;
    let items = live_view::decode_view(execute_widget_value(mir.clone(), &def_id, limits)?)?;
    live_view::compose(compiled, &mir, instance, limits, items)
}

fn widget_component<'a>(
    compiled: &'a AgePackageCompilation,
    widget: &AgeWidgetRef,
) -> Result<(usize, &'a ace_core::source::age_package::AgePackageComponent), String> {
    compiled.description.components.iter().enumerate()
        .find(|(_, component)| component.module == widget.module && component.name == widget.node)
        .ok_or_else(|| format!("source.age.av-widget-property: unknown widget `{}`", widget.node))
}

fn evaluate_widget_function(
    compiled: &AgePackageCompilation,
    widget: &AgeWidgetRef,
    instance: usize,
    name: &str,
    scoped: &[(String, String)],
    limits: &ExecLimits,
) -> Result<Value, String> {
    let symbol = compiled.symbol_of_in(&widget.module, name)
        .ok_or_else(|| format!("source.age.av-widget-property: missing checked widget function `{name}`"))?;
    let def_id = format!("d_{symbol}");
    let mir = lower_checked_age_package(&compiled.checked, &def_id, 0x981)
        .map_err(|err| format!("lower `{def_id}`: {err:?}"))?;
    execute_bound_widget(compiled, widget, instance, scoped, limits, mir, &def_id)
}

fn execute_bound_widget(
    compiled: &AgePackageCompilation,
    widget: &AgeWidgetRef,
    instance: usize,
    scoped: &[(String, String)],
    limits: &ExecLimits,
    mut mir: MirModule,
    def_id: &str,
) -> Result<Value, String> {
    let mut inputs = Vec::new();
    for binding in &widget.properties {
        match binding {
            AgeWidgetProperty::Message { .. } => {}
            AgeWidgetProperty::Value { .. } => {
                let getter = widget_property_def(compiled, binding, instance)?;
                let slot = mir.funcs.iter().position(|f| f.name == getter)
                    .ok_or_else(|| format!("missing checked widget property `{getter}`"))?;
                inputs.push(WidgetInput::Getter(slot as u32));
            }
            AgeWidgetProperty::ScopedValue { name, .. } => {
                let (_, value) = scoped.iter().find(|(key, _)| key == name)
                    .ok_or_else(|| format!("source.age.av-widget-property: missing scoped property `{name}`"))?;
                inputs.push(WidgetInput::Scoped(value));
            }
        }
    }
    bind_widget_inputs(&mut mir, &inputs)?;
    execute_widget_value(mir, def_id, limits)
}

enum WidgetInput<'a> {
    Getter(u32),
    Scoped(&'a str),
}

fn bind_widget_inputs(mir: &mut MirModule, inputs: &[WidgetInput<'_>]) -> Result<(), String> {
    let entry = mir.entry;
    let target = &mir.funcs[entry as usize];
    let sig = mir.sig_of(target.sig).ok_or("missing checked widget signature")?.clone();
    if sig.params.len() != inputs.len() {
        return Err("widget input bindings do not match the checked signature".into());
    }
    let result = sig.result;
    let owner = target.owner.clone();
    let source_snapshot = target.source_snapshot;
    let mut slots = Vec::with_capacity(inputs.len() + 1);
    let mut instrs = Vec::with_capacity(inputs.len() + 1);
    for (input, &param) in inputs.iter().zip(&sig.params) {
        let dst = slots.len() as u32;
        let ty = match *input {
            WidgetInput::Getter(getter) => {
                let sig = mir.sig_of(mir.funcs[getter as usize].sig)
                    .ok_or("missing checked property signature")?;
                if !sig.params.is_empty() {
                    return Err("source.age.av-widget-property: this binding needs ancestor widget inputs".into());
                }
                instrs.push(MirInstr::Call { dst, callee: CalleeRef::Func(getter), args: Vec::new() });
                sig.result
            }
            WidgetInput::Scoped(text) => {
                let invalid = || format!("source.age.av-widget-property: invalid scoped property for param {dst}");
                let value = match &mir.types[param as usize] {
                    MirType::Str => MirConst::Str(text.as_bytes().into()),
                    MirType::I32 => MirConst::I32(text.parse().map_err(|_| invalid())?),
                    MirType::Bool => MirConst::Bool(text.parse().map_err(|_| invalid())?),
                    _ => return Err(invalid()),
                };
                instrs.push(MirInstr::Const { dst, value: mir.intern_const(value) });
                param
            }
        };
        slots.push(MirSlot { kind: SlotKind::Temp, ty, name: None });
    }
    let output = slots.len() as u32;
    instrs.push(MirInstr::Call { dst: output, callee: CalleeRef::Func(entry), args: (0..output).collect() });
    slots.push(MirSlot { kind: SlotKind::Temp, ty: result, name: None });
    let sig = mir.intern_type(MirType::Fn(FnSig { params: Vec::new(), result }));
    mir.entry = mir.funcs.len() as u32;
    mir.funcs.push(MirFunction {
        owner,
        name: format!("d___age_p_hidden_widget_inputs_{entry}"),
        sig,
        slots,
        blocks: vec![MirBlock { instrs, term: Terminator::Return { value: output } }],
        entry: 0,
        source_snapshot,
        spans: Vec::new(),
        env: None,
    });
    Ok(())
}

fn execute_widget_value(mut mir: MirModule, def_id: &str, limits: &ExecLimits) -> Result<Value, String> {
    let transport = wrap_list_result(&mut mir)?;
    live_view::retain_entry_closure(&mut mir)?;
    let value = execute_value_module(&mir, def_id, limits)?;
    if transport {
        match value {
            Value::Record(record) => match record.fields.as_slice() {
                [Value::List(value)] => Ok(Value::List(value.clone())),
                _ => Err("widget property did not return its checked list".into()),
            },
            _ => Err("widget property did not return a record result".into()),
        }
    } else {
        Ok(value)
    }
}

fn wrap_list_result(mir: &mut MirModule) -> Result<bool, String> {
    let entry = mir.entry;
    let function = &mir.funcs[entry as usize];
    let MirType::Fn(signature) = &mir.types[function.sig as usize] else {
        return Err("widget property has no function signature".into());
    };
    if !signature.params.is_empty() {
        return Err("widget property evaluation requires a closed value function".into());
    }
    let list = signature.result;
    if !matches!(mir.types[list as usize], MirType::List(_)) {
        return Ok(false);
    }
    // The dynamic module ABI carries aggregate results in a nominal record.
    // Keep the authored getter's exact list type and verify the adapter too.
    let owner = function.owner.clone();
    let source_snapshot = function.source_snapshot;
    let name = format!("__age_property_result_{entry}");
    let record = mir.intern_type(MirType::Record(RecordTypeRef {
        owner_module: owner.module.clone(),
        name: name.clone(),
        fields: vec![MirFieldDef { name: "value".into(), ty: list }],
    }));
    let sig = mir.intern_type(MirType::Fn(FnSig { params: Vec::new(), result: record }));
    mir.entry = mir.funcs.len() as u32;
    mir.funcs.push(MirFunction {
        owner,
        name: format!("d_{name}"),
        sig,
        slots: vec![
            MirSlot { kind: SlotKind::Temp, ty: list, name: None },
            MirSlot { kind: SlotKind::Temp, ty: record, name: None },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Call { dst: 0, callee: CalleeRef::Func(entry), args: Vec::new() },
                MirInstr::Aggregate { dst: 1, record_ty: record, operands: vec![0] },
            ],
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot,
        spans: Vec::new(),
        env: None,
    });
    Ok(true)
}

fn execute_value_module(mir: &MirModule, def_id: &str, limits: &ExecLimits) -> Result<Value, String> {
    mir_verify::verify_mir(mir).map_err(|e| format!("verify MIR `{def_id}`: {e:?}"))?;
    let wire = encode_abc(mir).map_err(|e| format!("encode `{def_id}`: {e:?}"))?;
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).map_err(|e| format!("decode `{def_id}`: {e:?}"))?;
    let loaded = abc_verify::verify_abc(decoded, &mir.owner)
        .map_err(|e| format!("admit `{def_id}`: {e:?}"))?;
    let (outcome, _trace) = execute(&loaded, limits, &CancelFlag::new(false));
    match outcome {
        ExecOutcome::Success { value, .. } => Ok(value),
        other => Err(format!("value function `{def_id}` did not succeed: {other:?}")),
    }
}

/// 源名 → 绑定器给出的 HIR 符号（`sf_<mangled>`）。
///
/// PLAN-981（W1 跨模块实例身份，2026-11-18 第六批）：**按组件所属模块**解析
/// ——`symbols`（源名键）在多模块下是有损的（两个模块的同名脱糖函数只留得下
/// 一条），此前实例状态函数因此只能"两 node 同模块"。改为先用
/// `symbol_of_in(module, name)`，仅在同模块窗口取不到时回退到源名表（单模块
/// 路径与旧行为逐字一致）。
fn resolve_symbol(
    compiled: &AgePackageCompilation,
    module: &str,
    source_name: &str,
) -> Result<String, String> {
    compiled
        .symbol_of_in(module, source_name)
        .or_else(|| compiled.symbol_of(source_name))
        .ok_or_else(|| {
            format!("no HIR symbol for source name `{source_name}` in module `{module}`")
        })
}

fn execute_text(checked: &ace_core::verify::CheckedModule, def_id: &str) -> Result<String, String> {
    match execute_value(checked, def_id)? {
        Value::Str(h) => String::from_utf8(h.bytes().to_vec()).map_err(|e| format!("text not UTF-8: {e}")),
        other => Err(format!("value function did not produce a Str: {other:?}")),
    }
}
