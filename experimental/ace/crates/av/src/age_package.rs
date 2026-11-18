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
                let name =
                    ace_core::source::age_component::state_fn_name_at(component_index, inst, f);
                let symbol = resolve_symbol(compiled, &c.module, &name)?;
                instance_states[inst].push(execute_i32(&compiled.checked, &symbol)?);
            }
        }
        // PLAN-981 B-W1（IF-981-PROOF-AV）：混合包的 2D/3D 场景值与 UI 文本/
        // 状态同一次观察（同一 AV 车道，不再整模块拒 F32）。
        let scene_values = observe_scene_values(compiled, c)?;
        components.push(ObservedAgeComponent {
            name: c.name.clone(),
            module: c.module.clone(),
            instance_texts,
            instance_states,
            handler_msgs: c.handler_msgs.clone(),
            scene_values,
        });
    }
    Ok(ObservedAgePackage { components })
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
            match execute_scene_scalar(&compiled.checked, &symbol)? {
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

/// 场景标量值函数：lower→verify→ABC→AV（与状态/文本函数同一条链）。
fn execute_scene_scalar(
    checked: &ace_core::verify::CheckedModule,
    symbol: &str,
) -> Result<Value, String> {
    let def_id = format!("d_{symbol}");
    let mir = lower_checked_owned(checked, &def_id, 0x981)
        .map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
    mir_verify::verify_mir(&mir).map_err(|e| format!("verify MIR `{def_id}`: {e:?}"))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode `{def_id}`: {e:?}"))?;
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).map_err(|e| format!("decode `{def_id}`: {e:?}"))?;
    let loaded = abc_verify::verify_abc(decoded, &mir.owner)
        .map_err(|e| format!("admit `{def_id}`: {e:?}"))?;
    let (outcome, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match outcome {
        ExecOutcome::Success { value, .. } => Ok(value),
        other => Err(format!("scene value function did not succeed: {other:?}")),
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

fn execute_i32(checked: &ace_core::verify::CheckedModule, symbol: &str) -> Result<i32, String> {
    let def_id = format!("d_{symbol}");
    let mir = lower_checked_owned(checked, &def_id, 0x981)
        .map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
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

fn execute_text(checked: &ace_core::verify::CheckedModule, def_id: &str) -> Result<String, String> {
    let mir = lower_checked_owned(checked, def_id, 0x981)
        .map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
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
