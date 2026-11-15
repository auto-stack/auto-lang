//! PLAN-979 T-03：组件场景驱动的 **AV 消费端**（共享 lower→MIR→ABC→AV 取真值）。
//!
//! 驱动是普通 `i32` 函数（`__age_drive`），由共享链执行；观察值即场景编码
//! （instance 0 在个位、instance i 在 10^i 位）。文本叶仍按 H0 口径取共享 Str 值。

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::age_component::{
    compile_age_component, compile_component_scenario, compile_component_scenario_observing,
    ComponentCompilation, ComponentScenario,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_lowering::lower_checked_owned;

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

fn format_diags(ds: &[ace_core::atom_text::Diagnostic]) -> String {
    ds.iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn execute_i32(checked: &ace_core::verify::CheckedModule, def_id: &str) -> Result<i32, String> {
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
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => Ok(v),
        other => Err(format!(
            "component driver did not produce an i32: {other:?}"
        )),
    }
}

fn execute_text(checked: &ace_core::verify::CheckedModule, def_id: &str) -> Result<String, String> {
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
        ExecOutcome::Success {
            value: Value::Str(h),
            ..
        } => String::from_utf8(h.bytes().to_vec()).map_err(|e| format!("text not UTF-8: {e}")),
        other => Err(format!("text evaluation did not produce a Str: {other:?}")),
    }
}
