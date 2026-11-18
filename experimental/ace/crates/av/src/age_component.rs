//! PLAN-979 T-03：组件场景驱动的 **AV 消费端**（共享 lower→MIR→ABC→AV 取真值）。
//!
//! 驱动是普通 `i32` 函数（`__age_drive`），由共享链执行；观察值即场景编码
//! （instance 0 在个位、instance i 在 256^i 位）。文本叶仍按 H0 口径取共享 Str 值。

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::age_component::{
    compile_age_component, compile_component_scenario, compile_component_scenario_observing,
};
use ace_lowering::lower_checked_owned;

// PLAN-980 STATE：消费者（UI 桥）经本模块取组件面的配置/结果类型，不必直接
// 依赖 ace-core。再导出为纯加法，既有路径不受影响。
pub use ace_core::source::age_component::{
    compile_component_scenario_states, compile_component_scenario_states_at,
    compile_component_scenario_with_initials, project_component_candidates, ComponentChannelCall,
    ComponentCompilation, ComponentInitials, ComponentScenario, InitialValue, ScenarioOp,
};
pub use ace_core::source::age_h0::LegacyWidgetAlias;

/// 一次组件场景的真实观察结果。
#[derive(Debug)]
pub struct ObservedComponent {
    /// 驱动返回值（场景编码）。
    pub encoded: i32,
    /// 每个实例的文本叶共享 Str 值（实例序，每项按叶声明序）。
    pub instance_texts: Vec<Vec<String>>,
}

/// 场景观察：编译（共享前门 + 唯一 verify）→ 共享链执行驱动与文本叶。
pub fn observe_component(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    scenario: &ComponentScenario,
) -> Result<ObservedComponent, String> {
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled = compile_component_scenario(src, module_path, alias, &caps, scenario)
        .map_err(|ds| format_diags(&ds))?;
    observe_compiled(&compiled)
}

/// 场景观察（可指定观测字段）：同 `observe_component`，但驱动编码指定字段。
pub fn observe_component_observing(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    scenario: &ComponentScenario,
    observed: Option<&str>,
) -> Result<ObservedComponent, String> {
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled =
        compile_component_scenario_observing(src, module_path, alias, &caps, scenario, observed)
            .map_err(|ds| format_diags(&ds))?;
    observe_compiled(&compiled)
}

/// 静态面观察：编译（共享前门 + 唯一 verify）后逐个文本叶取共享 Str 真值。
///
/// 返回按节点分组的叶值（节点序、叶声明序）；这是静态描述之外的**值**证据。
pub fn observe_component_texts(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
) -> Result<Vec<Vec<String>>, String> {
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled =
        compile_age_component(src, module_path, alias, &caps).map_err(|ds| format_diags(&ds))?;
    let mut per_node = Vec::new();
    for node in &compiled.description.nodes {
        let mut texts = Vec::new();
        for t in &node.view.texts {
            texts.push(execute_text(&compiled.checked, &t.def_id)?);
        }
        per_node.push(texts);
    }
    Ok(per_node)
}

/// PLAN-981（D-981-AGE-F32-SURFACE）：执行任意已编译 def 并返回共享 `Value`。
///
/// 这是状态观察的**宽值出口**：窄枚举 [`ComponentFieldValue`] 只承载
/// int/bool/str（auto-ui 消费端按该枚举穷尽匹配；扩 F32 须与其 `AceStateValue`、
/// TS 能力类型与文本呈现同窗改），f32 状态经本函数在 AV 面取值而不破坏消费端。
pub fn execute_component_def(
    compiled: &ComponentCompilation,
    def_id: &str,
) -> Result<Value, String> {
    execute_value(&compiled.checked, def_id)
}

/// 对已编译结果执行驱动与文本叶（可复用于暖路径）。
pub fn observe_compiled(compiled: &ComponentCompilation) -> Result<ObservedComponent, String> {
    // 驱动 def id 由共享 adapter 唯一决定（`d___age_drive`）。
    let encoded = execute_i32(&compiled.checked, "d___age_drive")?;
    let mut instance_texts: Vec<Vec<String>> = Vec::new();
    for node in &compiled.description.nodes {
        for per in &node.instance_texts {
            let mut texts = Vec::new();
            for t in per {
                texts.push(execute_text(&compiled.checked, &t.def_id)?);
            }
            instance_texts.push(texts);
        }
    }
    Ok(ObservedComponent {
        encoded,
        instance_texts,
    })
}

/// 单个 (实例, 字段) 的真值（按源类型映射；PLAN-980 STATE）。
///
/// PLAN-981（`D-981-F32-CONSUMER-WINDOW`，2026-11-18，用户裁定同窗）：f32 状态
/// 进入窄观察枚举。`F32` 的相等按**位**判定（`to_bits`）——`NaN` payload 可
/// 判别、`-0.0 ≠ 0.0`，`Eq` 因之仍成立（消费端不必为 `f32` 去掉 `Eq`）。
/// 呈现不在本枚举里做（消费端按 Rust `f32` 的 `Display`：可往返、与地区无关）。
#[derive(Clone, Debug)]
pub enum ComponentFieldValue {
    Int(i32),
    Bool(bool),
    Str(String),
    F32(f32),
}

impl PartialEq for ComponentFieldValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Str(a), Self::Str(b)) => a == b,
            (Self::F32(a), Self::F32(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}

impl Eq for ComponentFieldValue {}

/// 每实例、每字段的真值观察结果（实例序、字段声明序）。
#[derive(Clone, Debug)]
pub struct ObservedStates {
    pub per_instance: Vec<Vec<(String, ComponentFieldValue)>>,
}

/// 逐字段状态观察：执行每个 `__age_state_<i>_<field>` def，按源类型映射真值。
///
/// 与 `observe_compiled` 的整型打包编码互补：驱动编码适合场景级 oracle；本
/// API 是 UI 桥的 `read_state` 通道（int/bool/str 全域）。
pub fn observe_component_states(compiled: &ComponentCompilation) -> Result<ObservedStates, String> {
    let mut per_instance = Vec::new();
    for node in &compiled.description.nodes {
        for per in &node.instance_states {
            let mut row = Vec::new();
            for s in per {
                let value = match s.ty.as_str() {
                    "int" | "i32" => match execute_value(&compiled.checked, &s.def_id)? {
                        Value::I32(v) => ComponentFieldValue::Int(v),
                        other => {
                            return Err(format!(
                                "state `{}` did not produce an i32: {other:?}",
                                s.fn_name
                            ))
                        }
                    },
                    "bool" => match execute_value(&compiled.checked, &s.def_id)? {
                        Value::Bool(b) => ComponentFieldValue::Bool(b),
                        other => {
                            return Err(format!(
                                "state `{}` did not produce a bool: {other:?}",
                                s.fn_name
                            ))
                        }
                    },
                    "str" | "Str" => match execute_value(&compiled.checked, &s.def_id)? {
                        Value::Str(h) => String::from_utf8(h.bytes().to_vec())
                            .map(ComponentFieldValue::Str)
                            .map_err(|e| format!("state text not UTF-8: {e}"))?,
                        other => {
                            return Err(format!(
                                "state `{}` did not produce a Str: {other:?}",
                                s.fn_name
                            ))
                        }
                    },
                    // PLAN-981（D-981-F32-CONSUMER-WINDOW）：f32 状态经窄枚举返回，
                    // 与 `execute_component_def` 的宽值出口同一执行链（不是第二条）。
                    "f32" | "float" => match execute_value(&compiled.checked, &s.def_id)? {
                        Value::F32(v) => ComponentFieldValue::F32(v),
                        other => {
                            return Err(format!(
                                "state `{}` did not produce an f32: {other:?}",
                                s.fn_name
                            ))
                        }
                    },
                    other => {
                        return Err(format!(
                            "state `{}` carries type `{other}`, which the narrow \
                             ComponentFieldValue view does not carry (走 \
                             `execute_component_def` 的共享 Value 出口)",
                            s.fn_name
                        ))
                    }
                };
                row.push((s.field.clone(), value));
            }
            per_instance.push(row);
        }
    }
    Ok(ObservedStates { per_instance })
}

/// 逐字段状态 + 文本叶双观察（PLAN-980 NATIVE：免驱动源的完整观察面——
/// states 按 int/bool/str 全域、texts 按实例取共享 Str 真值）。
pub fn observe_component_states_and_texts(
    compiled: &ComponentCompilation,
) -> Result<(ObservedStates, Vec<Vec<String>>), String> {
    let states = observe_component_states(compiled)?;
    let mut texts: Vec<Vec<String>> = Vec::new();
    for node in &compiled.description.nodes {
        for per in &node.instance_texts {
            let mut row = Vec::new();
            for t in per {
                row.push(execute_text(&compiled.checked, &t.def_id)?);
            }
            texts.push(row);
        }
    }
    Ok((states, texts))
}

/// 场景级便捷入口：**states-only** 编译（带初值）→ 逐字段状态观察
/// （PLAN-980 STATE 的 UI 桥热路径）。
///
/// states-only 不发射文本叶求值函数——f-string 叶（`TextDisplay`）在
/// av.owned/1 车道不可 lowering，状态通道不依赖文本叶，故此类源仍可观察
/// 状态。完整文本观察走 `observe_compiled`（f-string 源在那条车道保持拒绝）。
pub fn observe_component_scenario_states(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    scenario: &ComponentScenario,
    initials: &ComponentInitials,
) -> Result<ObservedStates, String> {
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled =
        compile_component_scenario_states(src, module_path, alias, &caps, scenario, initials)
            .map_err(|ds| format_diags(&ds))?;
    observe_component_states(&compiled)
}

fn format_diags(ds: &[ace_core::atom_text::Diagnostic]) -> String {
    ds.iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn execute_i32(checked: &ace_core::verify::CheckedModule, def_id: &str) -> Result<i32, String> {
    match execute_value(checked, def_id)? {
        Value::I32(v) => Ok(v),
        other => Err(format!(
            "component driver did not produce an i32: {other:?}"
        )),
    }
}

fn execute_text(checked: &ace_core::verify::CheckedModule, def_id: &str) -> Result<String, String> {
    match execute_value(checked, def_id)? {
        Value::Str(h) => {
            String::from_utf8(h.bytes().to_vec()).map_err(|e| format!("text not UTF-8: {e}"))
        }
        other => Err(format!("text evaluation did not produce a Str: {other:?}")),
    }
}

/// 共享链执行单个 def：lower → MIR verify → ABC admit → AV（PLAN-980 STATE
/// 起为状态函数/驱动/文本共用的通用执行器）。
fn execute_value(checked: &ace_core::verify::CheckedModule, def_id: &str) -> Result<Value, String> {
    let mir = lower_checked_owned(checked, def_id, 0x979)
        .map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
    mir_verify::verify_mir(&mir).map_err(|e| format!("verify MIR `{def_id}`: {e:?}"))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode `{def_id}`: {e:?}"))?;
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).map_err(|e| format!("decode `{def_id}`: {e:?}"))?;
    let loaded = abc_verify::verify_abc(decoded, &mir.owner)
        .map_err(|e| format!("admit `{def_id}`: {e:?}"))?;
    let limits = ExecLimits::default();
    let cancel = CancelFlag::new(false);
    let (outcome, _trace) = execute(&loaded, &limits, &cancel);
    match outcome {
        ExecOutcome::Success { value, .. } => Ok(value),
        other => Err(format!(
            "execution of `{def_id}` did not succeed: {other:?}"
        )),
    }
}
