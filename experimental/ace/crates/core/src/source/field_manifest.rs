//! PLAN-980（B-DIALECT-I，IF-980-FIELDS）：**字段级投影完整性清单**的通用机制。
//!
//! 既有 [`crate::source::domain::mapping_complete`] 与
//! [`crate::source::age_component::component_coverage`] 以 **role 粒度**判必需项
//! （`node`/`view`/`model`…）；980/DIALECT 需要按**字段/构造 ID** 索引的覆盖：
//! 完整 `WidgetDecl` 分母逐字段建表（版本化必需清单），投影侧按同一 ID 集对账。
//! 本模块补这一层通用数据/检查：
//!
//! - [`FieldManifest`] + [`RequiredFieldRow`]：领域 owner（auto-ui 的 WidgetDecl
//!   分母；981 的 AGE 构造分母）**只交数据行**；行内语义字段由 PLAN-980 r10 §5
//!   冻结（`id/origin/source_span/parent_role/type_rule/applicability/
//!   projection_path/check_id/consumer_owner`）。
//! - [`field_coverage`]：必需行缺失（missing）与投影侧**未知构造**（unknown，
//!   出现了清单没有的 ID——不允许静默丢弃）都在一个结果里具名可判。
//! - [`validate_field_manifest`]：清单自检（ID 唯一/非空、条件适用必带可核理由）。
//! - [`component_field_ids`]：把既有组件面候选（`project_component_candidates`
//!   的 `ComponentNode`，带原 span）按调用方给定的 ID 词汇表 walk 成字段级
//!   出现集，接到同一覆盖检查——不引入产品语义（词汇表由清单 owner 提供）。
//!
//! 边界：Core 只保通用数据/检查，不反依 UI crate，不建第二 parser/类型检查器。

use crate::atom_text::Span;
use crate::source::age_component::ComponentNode;
use crate::source::domain::DomainVersion;

/// 一行的适用性（清单 owner 冻结；条件适用必须给可核理由）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldApplicability {
    /// 必需：闭合分母内出现即必须投影；覆盖时缺席 = missing（红）。
    Required { mode: String },
    /// 可选：出现时投影；不出现不算缺。
    Optional { mode: String },
    /// 条件适用：须给可核理由（如“仅 router widget 声明 routes”）；条件不满足
    /// 时不投影不算缺，但行本身不得从分母删除。
    Conditional { mode: String, reason: String },
}

impl FieldApplicability {
    /// 该行是否计入覆盖分母（missing 只看必需行）。
    pub fn is_required(&self) -> bool {
        matches!(self, FieldApplicability::Required { .. })
    }
}

/// 必需字段/构造清单的一行（语义字段按 PLAN-980 r10 §5 冻结）。
#[derive(Debug, Clone)]
pub struct RequiredFieldRow {
    /// 稳定字段/构造 ID（清单 owner 的命名空间；覆盖两侧按完整字符串相等连接）。
    pub id: String,
    /// 来源：Spec/原case锚点 + source hash（如 `auto-ui ast.rs WidgetDecl.model
    /// <sha16>` / `examples/ui/002-counter app.at <sha16>`）。
    pub origin: String,
    /// 该行锚定的原源 span（锚定 fixture 内的字段原始字节范围；无源锚点时
    /// start==end 且 origin 必须写明 AST/Spec 锚点）。
    pub source_span: Span,
    /// 父角色（如 `widget`、`widget.view`、`widget.model`）。
    pub parent_role: String,
    /// 允许的类型/形状规则（文字描述；投影侧按此判同类型非法槽）。
    pub type_rule: String,
    /// 适用性与模式（是否必需、适用模式、条件理由）。
    pub applicability: FieldApplicability,
    /// 投影位置：该字段落到领域候选/投影的哪一层（如
    /// `ComponentDescription.nodes[].fields[]`、`DomainRow tree`）。
    pub projection_path: String,
    /// 共享验证准入：该行由哪个永久检查核对（check/test 名）。
    pub check_id: String,
    /// 实际 consumer owner（如 `980/DIALECT`、`980/STATE`、`980/PARITY`）。
    pub consumer_owner: String,
}

/// 版本化必需清单（交接件身份：domain id + 版本 + 行集）。
#[derive(Debug, Clone)]
pub struct FieldManifest {
    /// 清单领域身份（如 `ui.widget-decl`、`age.component`；`<domain>/v<major>` 进交接件）。
    pub domain: String,
    pub version: DomainVersion,
    pub rows: Vec<RequiredFieldRow>,
}

/// 字段级覆盖结果：必需/出现/缺失/未知各自具名，调用方按需判红。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldCoverage {
    /// 计入覆盖分母的必需行 ID（清单序）。
    pub required: Vec<String>,
    /// 投影侧实际出现且清单承认的 ID（去重、首现序）。
    pub present: Vec<String>,
    /// 必需行缺席（红：映射被删/投影断链）。
    pub missing: Vec<String>,
    /// 投影侧出现但清单没有的 ID（红：未知构造不得静默丢弃）。
    pub unknown: Vec<String>,
}

impl FieldCoverage {
    /// 覆盖闭合：无缺失且无未知构造。
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.unknown.is_empty()
    }
}

/// 字段级覆盖核对（`mapping_complete` 的字段粒度版）。
///
/// `presented` 是投影侧的字段出现集（ID 可重复——出现次数不影响覆盖判定）；
/// 必需行缺席进 `missing`；出现但清单不认识的 ID 进 `unknown`（两侧都判红，
/// 合法近邻/可选行缺席不受影响）。
pub fn field_coverage(presented: &[FieldPresented], manifest: &FieldManifest) -> FieldCoverage {
    let required: Vec<String> = manifest
        .rows
        .iter()
        .filter(|r| r.applicability.is_required())
        .map(|r| r.id.clone())
        .collect();
    let mut present: Vec<String> = Vec::new();
    for p in presented {
        if manifest.rows.iter().any(|r| r.id == p.id) && !present.contains(&p.id) {
            present.push(p.id.clone());
        }
    }
    let missing: Vec<String> = required
        .iter()
        .filter(|want| !present.contains(want))
        .cloned()
        .collect();
    let mut unknown: Vec<String> = Vec::new();
    for p in presented {
        if !manifest.rows.iter().any(|r| r.id == p.id) && !unknown.contains(&p.id) {
            unknown.push(p.id.clone());
        }
    }
    FieldCoverage {
        required,
        present,
        missing,
        unknown,
    }
}

/// 清单自检：ID 唯一且非空、语义字段非空、条件适用必带可核理由、span 有序。
/// 返回问题清单（空 = 清单可执行；非空调用方判红——不能交一份不可执行的表）。
pub fn validate_field_manifest(manifest: &FieldManifest) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    if manifest.domain.is_empty() {
        problems.push("manifest domain is empty".to_string());
    }
    let mut seen: Vec<&str> = Vec::new();
    for (idx, row) in manifest.rows.iter().enumerate() {
        let at = format!("row {idx} `{}`", row.id);
        if row.id.is_empty() {
            problems.push(format!("{at}: id is empty"));
        } else if seen.contains(&row.id.as_str()) {
            problems.push(format!("{at}: duplicate id"));
        } else {
            seen.push(&row.id);
        }
        for (label, value) in [
            ("origin", &row.origin),
            ("parent_role", &row.parent_role),
            ("type_rule", &row.type_rule),
            ("projection_path", &row.projection_path),
            ("check_id", &row.check_id),
            ("consumer_owner", &row.consumer_owner),
        ] {
            if value.is_empty() {
                problems.push(format!("{at}: {label} is empty"));
            }
        }
        if row.source_span.start > row.source_span.end {
            problems.push(format!("{at}: source_span is not ordered"));
        }
        let (kind, mode, reason) = match &row.applicability {
            FieldApplicability::Required { mode } => ("required", mode, None),
            FieldApplicability::Optional { mode } => ("optional", mode, None),
            FieldApplicability::Conditional { mode, reason } => ("conditional", mode, Some(reason)),
        };
        if mode.is_empty() {
            problems.push(format!("{at}: applicability {kind} has an empty mode"));
        }
        if let Some(reason) = reason {
            if reason.is_empty() {
                problems.push(format!(
                    "{at}: conditional applicability needs a verifiable reason"
                ));
            }
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// 投影侧的一个字段出现（ID + 原 span；`field_coverage` 的输入单元）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldPresented {
    pub id: String,
    pub span: Span,
}

/// 组件面候选 → 字段级出现集的 ID 词汇表（由清单 owner 提供；Core 不含产品语义）。
///
/// 词汇表把 [`ComponentNode`] 候选里投影侧实际产生的每种结构映射为清单 ID；
/// 词汇表没覆盖到的结构**不会**被静默略过——`component_field_ids` 对每个
/// node 头/名字/model 字段/on 处理器/view/text 叶都产出出现，调用方用清单对账
/// 时未知面（清单缺行或词汇表缺项）都会在覆盖结果里红出来。
#[derive(Debug, Clone)]
pub struct ComponentFieldVocab {
    pub node_head: String,
    pub node_name: String,
    pub model_block: String,
    pub model_field: String,
    /// `var` 可变前缀出现（传统 widget 工程形）。
    pub model_field_var: String,
    pub on_block: String,
    pub on_handler: String,
    pub view_block: String,
    pub text_leaf: String,
    /// PLAN-980（完整批）：widget 成员槽位/view 树的字段 ID 词汇。
    pub msg_block: String,
    pub msg_variant: String,
    pub msg_variant_payload: String,
    pub design_block: String,
    pub design_entry: String,
    pub view_element: String,
    /// 非事件属性行（`key: value` 且 key 不以 `on` 开头）。
    pub view_prop_entry: String,
    /// 事件行（key 以 `on` 开头——与 auto-ui 方言同规则）。
    pub view_event_entry: String,
    /// 内联 lambda 事件（`() => { … }`）。
    pub view_event_inline: String,
    pub bind_key: String,
    pub route_def: String,
    pub expose_name: String,
    pub lifecycle_method: String,
    pub computed_property: String,
    pub watch_decl: String,
    pub timer_entry: String,
    pub named_view: String,
}

/// 把组件面候选（`project_component_candidates` 的产物，带原 span）walk 成
/// 字段级出现集（每个实际出现一行，含原 span）。
///
/// 这是 [`crate::source::age_component::component_coverage`]（role 粒度）在
/// 字段粒度上的对应物：ID 由词汇表给定，结构/次数/span 来自真实候选；
/// 候选通过共享 verify（`compile_age_component`）与覆盖核对分开执行。
pub fn component_field_ids(
    candidates: &[ComponentNode],
    vocab: &ComponentFieldVocab,
) -> Vec<FieldPresented> {
    let mut out = Vec::new();
    for n in candidates {
        out.push(FieldPresented {
            id: vocab.node_head.clone(),
            span: n.head_span,
        });
        out.push(FieldPresented {
            id: vocab.node_name.clone(),
            span: n.name.span,
        });
        if !n.fields.is_empty() {
            out.push(FieldPresented {
                id: vocab.model_block.clone(),
                span: n.span,
            });
            for f in &n.fields {
                out.push(FieldPresented {
                    id: vocab.model_field.clone(),
                    span: f.span,
                });
                if f.mutable {
                    out.push(FieldPresented {
                        id: vocab.model_field_var.clone(),
                        span: f.span,
                    });
                }
            }
        }
        if !n.handlers.is_empty() {
            out.push(FieldPresented {
                id: vocab.on_block.clone(),
                span: n.span,
            });
            for h in &n.handlers {
                out.push(FieldPresented {
                    id: vocab.on_handler.clone(),
                    span: h.span,
                });
            }
        }
        if !n.view.role.is_empty() {
            out.push(FieldPresented {
                id: vocab.view_block.clone(),
                span: n.view.span,
            });
            // 文本叶出现经下方 walk_view_tree（tree 与 texts 同源；此处不重复计）。
        }
        // PLAN-980（完整批）：msg/design/成员槽位/view 树的字段级出现。
        if !n.messages.is_empty() {
            out.push(FieldPresented {
                id: vocab.msg_block.clone(),
                span: n.span,
            });
            for m in &n.messages {
                out.push(FieldPresented {
                    id: vocab.msg_variant.clone(),
                    span: m.span,
                });
                if !m.payload.is_empty() {
                    out.push(FieldPresented {
                        id: vocab.msg_variant_payload.clone(),
                        span: m.span,
                    });
                }
            }
        }
        if !n.design.is_empty() {
            out.push(FieldPresented {
                id: vocab.design_block.clone(),
                span: n.span,
            });
            for d in &n.design {
                out.push(FieldPresented {
                    id: vocab.design_entry.clone(),
                    span: d.span,
                });
            }
        }
        for r in &n.members {
            let id = match r.role {
                "bind" => &vocab.bind_key,
                "route" => &vocab.route_def,
                "expose" => &vocab.expose_name,
                "lifecycle" => &vocab.lifecycle_method,
                "computed" => &vocab.computed_property,
                "watch" => &vocab.watch_decl,
                "timer" => &vocab.timer_entry,
                "named-view" => &vocab.named_view,
                _ => continue,
            };
            out.push(FieldPresented {
                id: id.clone(),
                span: r.span,
            });
        }
        walk_view_tree(&n.view.tree, vocab, &mut out);
    }
    out
}

/// view 树（元素/属性行/文本叶）→ 字段级出现（事件归类按 `on` 前缀）。
fn walk_view_tree(
    items: &[crate::source::age_component::ComponentViewItem],
    vocab: &ComponentFieldVocab,
    out: &mut Vec<FieldPresented>,
) {
    use crate::source::age_component::{ComponentEntryValue, ComponentViewItem};
    for item in items {
        match item {
            ComponentViewItem::Text(t) => out.push(FieldPresented {
                id: vocab.text_leaf.clone(),
                span: t.span,
            }),
            ComponentViewItem::Element(e) => {
                out.push(FieldPresented {
                    id: vocab.view_element.clone(),
                    span: e.span,
                });
                walk_view_tree(&e.items, vocab, out);
            }
            ComponentViewItem::Entry(v) => {
                if v.is_event {
                    out.push(FieldPresented {
                        id: vocab.view_event_entry.clone(),
                        span: v.span,
                    });
                    if matches!(v.value, ComponentEntryValue::Lambda(_)) {
                        out.push(FieldPresented {
                            id: vocab.view_event_inline.clone(),
                            span: v.span,
                        });
                    }
                } else {
                    out.push(FieldPresented {
                        id: vocab.view_prop_entry.clone(),
                        span: v.span,
                    });
                }
            }
        }
    }
}
