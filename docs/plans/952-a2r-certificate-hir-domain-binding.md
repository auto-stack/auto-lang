---
plan_id: PLAN-952
status: executing
feature_name: Rust证书绑定实际HIR域
plan_revision: 1
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
supersedes_spec_components: [docs/specs/auto-test-case/a2r-hir-certification.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-test-case]
depends_on: [PLAN-892, PLAN-902, PLAN-904]
current_step: 0
total_steps: 6
---

# [PLAN-952] Rust证书绑定实际HIR域

## 0. 变更摘要

独立关闭941真实认证发现F-941-02：Rust证书通过字符串裁剪重建HIR域，`auto.source.scalar-i32.draft`等落入default core；同一actual的manifest标量域与证书core域矛盾。直接绑定既有EmitOutcome.hir_domain，删除第二套错误映射；不在941消费者宽容坏身份。

## 1. 目标

所有已支持core/scalar/record/text/text-record/collections profile的证书HIR域与实际编译bundle/manifest一致；保持原schema、oracle、完整actual/reference、工具、配置、预算与daily/cert分轴。

## 2. 架构方案

现source→实际compile/emit的EmitOutcome已有hir_domain；Rust certify与C一样消费该字段。移除私有domain_of副本。新回归放原ace-a2x-worker conformance target，真实worker认证+解析cert与manifest，并与事前指定的profile域逐值比较；不新增runner、编译器或store。

## 3. 技术栈

Rust；`experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs`、新`tests/cases/cert_domain_952.rs`及原conformance注册；既有serde_json、原Job/deadline编译运行。原固定完整组`cargo t ace-a2x`。

## 4. 需求分析与背景调查

用户2026-11-06明确答复“授权独立修复并继续（推荐）”：最小producer域绑定修复、各profile回归、复审合入后继续941，全链授权继承，不再次确认。新旧计划不互替；941仍0/15。main基线c4282e663（含951）；941检查点55c8b37ea的真实scalar cert为core-i32-draft，而同场manifest为scalar-i32-draft；代码domain_of误识别带-i32 profile。文档取材`docs/specs/auto-test-case/a2r-hir-certification.md`（严格身份/隔离通道）与实际bundle field；C直接消费out.hir_domain为合法正控。当前main只有其他会话文档WIP，零代码WIP，不纳本包。

唯一实施树`D:/autostack/.wt/lang-952/auto-lang`、branch `plan-952-dev`，不使用941树实现本包。唯一材料目录`D:/autostack/plan-artifacts/plan-952/`，登记owner，在途保护，merge全删；跨仓只读auto-down组兄弟。无子agent/chat授权，复审由同会话重新读取冻结合同/diff/证据独立执行，明确上下文限制。

## 5. 详细设计

不再把view profile名称当HIR域解析规则；`CertDraft.hir_domain = out.hir_domain.clone()`。日常emitter/manifest/能力/profile dispatch不改。各profile的事前域依次core-i32-draft/scalar-i32-draft/record-i32-draft/text-i32-draft/text-record-i32-draft/collections-draft，由现profile源编译事实与typed emitter核查，若实际明确枚举不同以正式源事实记录，不凭测试输出自创expected。

### 规范增量

| delta_id | action | target | before/after | AC |
|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/a2r-hir-certification.md | 明确证书profile/HIR域须来自同一实际compiled bundle并与manifest相等，禁止从profile字符串另造域 | AC-01/02 |

## 6. 测试设计

V-01=真实worker各profile实际actual+手写reference认证，标准JSON解析cert/manifest，事前域与双件一致；修前scalar/record/text/text-record应红，core/collections正控。V-02=原完整ace-a2x冻结组（merge owner），不减样本/ignore/预算；C与951序列化控制保留。V-03=941严格Q4实际source及Rust/C独立reference重放（941自己树重建后），不以source commit改变代相同身份。

修复仅证书元数据来源，不改语义/增量/运行算法，不新采无关性能全矩阵。原完整组含原条件排除，排除不充PASS。测试与输出只在专属树/统一scope，不在main跑构建。

## 7. 验收标准

| ID | owner_stage | 条件 |
|---|---|---|
| AC-01 | work | 各profile真实cert与manifest及正式域相等，修前红/修后绿，不只手构typed draft |
| AC-02 | work | 原schema/完整文件/工具/reference/oracle/预算不改，C与951控制不退；无新warning/format/debug问题 |
| AC-03 | review | 冻结提交逐AC独立复审、遗漏/延期/workaround scan，SD核准 |
| AC-04 | merge | reviewed身份合入、canonical沉淀、归档、guard清理、材料路径对账完成；返回941 |

## 8. 执行步骤

| ID | owner_stage | deps | 内容 / gate | 状态 |
|---|---|---|---|---|
| T-01 | work | - | 登记scope/树，补真实profile回归并保修前红 | [ ] |
| T-02 | work | T-01 | 原producer绑定out.hir_domain、移除domain_of，定向回归/格式/编译 | [ ] |
| T-03 | work | T-02 | 提交代码/SD草案，execution_done，完整组门留merge | [ ] |
| T-04 | review | T-03 | 冻结逐AC/遗漏/健康/SD独立复审 | [ ] |
| T-05 | merge | T-04 | 冻结cargo t ace-a2x完整组、合main及Spec、归档 | [ ] |
| T-06 | merge | T-05 | guard/tree/branch/组/scope清理、941交付回执 | [ ] |

## 9. 复审记录

未复审。首次执行待T-01。941上一goal轮为progress（55c8b37ea、3认证观察控制、35治理控制及真实双端oracle），本包为用户授权的新修复，不重置941合同。

## 10. 待澄清事项

无。原授权范围明确，不扩大后端能力或改变941验收阈值。
