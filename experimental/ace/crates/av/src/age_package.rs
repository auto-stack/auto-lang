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
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::age_package::AgePackageCompilation;
use ace_lowering::lower_checked_owned;

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
    /// 逐实例的 `on` 消息名（作者拼写，含 `Life.Tick` 这类限定名）。
    pub handler_msgs: Vec<String>,
}

/// 观察 AGE 包：编译结果里的每个实例函数走共享链取真值。
///
/// `instances` 必须与编译时给出的场景实例数一致（状态函数按实例生成；不一致
/// 是具名错误，不静默返回空表）。
pub fn observe_age_package(
    compiled: &AgePackageCompilation,
    instances: usize,
) -> Result<ObservedAgePackage, String> {
    let mut components = Vec::new();
    for (component_index, c) in compiled.description.components.iter().enumerate() {
        if c.texts.is_empty() {
            components.push(ObservedAgeComponent {
                name: c.name.clone(),
                module: c.module.clone(),
                instance_texts: Vec::new(),
                instance_states: Vec::new(),
                handler_msgs: c.handler_msgs.clone(),
            });
            continue;
        }
        // 叶按实例分组：编译期为每个 (实例, 叶) 各生成一份求值函数。
        let per_instance = if instances == 0 {
            1
        } else {
            instances
        };
        if instances > 0 && c.texts.len() % per_instance != 0 {
            return Err(format!(
                "component `{}` has {} value functions which is not a multiple of {} instances",
                c.name,
                c.texts.len(),
                per_instance
            ));
        }
        let mut instance_texts: Vec<Vec<String>> = vec![Vec::new(); per_instance];
        for (i, t) in c.texts.iter().enumerate() {
            let inst = i % per_instance;
            instance_texts[inst].push(execute_text(&compiled.checked, &t.def_id)?);
        }
        // 实例状态：按实例取每个 `int` 字段的状态函数值。
        let mut instance_states: Vec<Vec<i32>> = vec![Vec::new(); per_instance];
        let int_fields: Vec<&str> = c
            .fields
            .iter()
            .filter(|(_, ty)| ty == "int" || ty == "i32")
            .map(|(n, _)| n.as_str())
            .collect();
        for inst in 0..per_instance {
            for f in &int_fields {
                // R64-4：与包前门同一套多 node 消歧命名。
                let name = ace_core::source::age_component::state_fn_name_at(
                    component_index, inst, f,
                );
                let symbol = resolve_symbol(compiled, &c.name, &name)?;
                instance_states[inst].push(execute_i32(&compiled.checked, &symbol)?);
            }
        }
        components.push(ObservedAgeComponent {
            name: c.name.clone(),
            module: c.module.clone(),
            instance_texts,
            instance_states,
            handler_msgs: c.handler_msgs.clone(),
        });
    }
    Ok(ObservedAgePackage { components })
}

/// 源名 → 绑定器给出的 HIR 符号（`sf_<mangled>`）。
fn resolve_symbol(
    compiled: &AgePackageCompilation,
    _component: &str,
    source_name: &str,
) -> Result<String, String> {
    compiled
        .symbol_of(source_name)
        .ok_or_else(|| format!("no HIR symbol for source name `{source_name}`"))
}

fn execute_i32(
    checked: &ace_core::verify::CheckedModule,
    symbol: &str,
) -> Result<i32, String> {
    let def_id = format!("d_{symbol}");
    let mir = lower_checked_owned(checked, &def_id, 0x981).map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
    mir_verify::verify_mir(&mir).map_err(|e| format!("verify MIR `{def_id}`: {e:?}"))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode `{def_id}`: {e:?}"))?;
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).map_err(|e| format!("decode `{def_id}`: {e:?}"))?;
    let loaded = abc_verify::verify_abc(decoded, &mir.owner)
        .map_err(|e| format!("admit `{def_id}`: {e:?}"))?;
    let (outcome, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match outcome {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => Ok(v),
        other => Err(format!("state function did not produce i32: {other:?}")),
    }
}

fn execute_text(
    checked: &ace_core::verify::CheckedModule,
    def_id: &str,
) -> Result<String, String> {
    let mir = lower_checked_owned(checked, def_id, 0x981).map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
    mir_verify::verify_mir(&mir).map_err(|e| format!("verify MIR `{def_id}`: {e:?}"))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode `{def_id}`: {e:?}"))?;
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).map_err(|e| format!("decode `{def_id}`: {e:?}"))?;
    let loaded = abc_verify::verify_abc(decoded, &mir.owner)
        .map_err(|e| format!("admit `{def_id}`: {e:?}"))?;
    let (outcome, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match outcome {
        ExecOutcome::Success {
            value: Value::Str(h),
            ..
        } => String::from_utf8(h.bytes().to_vec()).map_err(|e| format!("text not UTF-8: {e}")),
        other => Err(format!("value function did not produce a Str: {other:?}")),
    }
}
