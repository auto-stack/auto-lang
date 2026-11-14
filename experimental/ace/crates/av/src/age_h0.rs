//! PLAN-979（T-H0-I）：AGE H0 的 CPU consumer（领域消费者，非产品宿主）。
//!
//! 输入是领域前门的受检结论（`ace_core::source::age_h0`）：每个文本叶有一个
//! 由共享 verify 检查过的隐藏求值函数。本消费者**按该 def id 走共享
//! lowering → MIR 校验 → ABC 编码/解码/准入 → AV 执行**得到真实值，再用它
//! 组成可观察的最小结构（根 node / view 角色 / text 叶值 / 原 span）。
//!
//! 它不是第二个解释器：没有把表达式当字符串再解析，也没有用 Rust 常量补值；
//! 换文本/换函数返回值必须改变观察结果。

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::age_h0::{
    compile_age_h0, AgeH0Description, AgeH0Node, LegacyWidgetAlias, TextKind,
};
use ace_lowering::lower_checked_owned;

/// 观察到的一个文本叶。
#[derive(Clone, Debug)]
pub struct AgeObservedText {
    pub role: String,
    /// 共享执行链产出的真实 UTF-8 文本。
    pub value: String,
    /// 共享 resolver/verify 判定的文本种类。
    pub kind: TextKind,
    pub component_span: (usize, usize),
    pub span: (usize, usize),
}

/// 观察到的一个 view。
#[derive(Clone, Debug)]
pub struct AgeObservedView {
    pub role: String,
    pub texts: Vec<AgeObservedText>,
}

/// 观察到的一个 node（最小 CPU 结构）。
#[derive(Clone, Debug)]
pub struct AgeObservedNode {
    pub name: String,
    pub head_role: String,
    pub legacy_widget: bool,
    pub view: AgeObservedView,
    pub span: (usize, usize),
}

/// 一次 AGE H0 观察：领域包身份 + 观察到的结构。
#[derive(Clone, Debug)]
pub struct AgeH0Observation {
    pub domain: &'static str,
    pub grammar_id: String,
    pub alias: LegacyWidgetAlias,
    pub nodes: Vec<AgeObservedNode>,
}

/// 领域入口：编译（Schema 驱动 + 唯一 verify）并执行文本叶观察值。
pub fn observe_age_h0(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
) -> Result<AgeH0Observation, String> {
    let compiled = compile_age_h0(src, module_path, alias, &[]).map_err(|ds| render_diags(&ds))?;
    observe_compiled(src, alias, &compiled.description, &compiled.checked)
}

fn render_diags(diags: &[ace_core::atom_text::Diagnostic]) -> String {
    diags
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn observe_compiled(
    src: &str,
    alias: LegacyWidgetAlias,
    description: &AgeH0Description,
    checked: &ace_core::verify::CheckedModule,
) -> Result<AgeH0Observation, String> {
    let mut nodes = Vec::new();
    for n in &description.nodes {
        nodes.push(observe_node(src, n, checked)?);
    }
    Ok(AgeH0Observation {
        domain: description.domain,
        grammar_id: description.grammar_id.clone(),
        alias,
        nodes,
    })
}

fn observe_node(
    src: &str,
    n: &AgeH0Node,
    checked: &ace_core::verify::CheckedModule,
) -> Result<AgeObservedNode, String> {
    let mut texts = Vec::new();
    for t in &n.view.texts {
        let value = execute_text(checked, &t.def_id)?;
        // 观察值与源文本同源：值必须真实来自执行（这里只做诊断增强，
        // 不用 src 参与求值）。
        let _ = src;
        texts.push(AgeObservedText {
            role: t.role.clone(),
            value,
            kind: t.kind,
            component_span: (t.component_span.start, t.component_span.end),
            span: (t.span.start, t.span.end),
        });
    }
    Ok(AgeObservedNode {
        name: n.name.clone(),
        head_role: n.head_role.clone(),
        legacy_widget: n.legacy_widget,
        view: AgeObservedView {
            role: n.view.role.clone(),
            texts,
        },
        span: (n.span.start, n.span.end),
    })
}

/// 单个文本叶的共享执行链（lower → MIR verify → ABC → AV）。
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
        other => Err(format!(
            "text evaluation did not produce a Str value: {other:?}"
        )),
    }
}
