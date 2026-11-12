---
plan_id: PLAN-1004
status: drafting
feature_name: auto-plan 的计划模板与现行契约 Spec
author: [Codex]
created_at: 2026-11-11
updated_at: 2026-11-11
plan_revision: 1
current_step: 0
total_steps: 5
supersedes_spec_components: [docs/specs/README.md]
new_spec_components: []
touched_goals: []
---

# PLAN-1004 · 让 auto-plan 按实际交付更新 Specs

## 0. 变更摘要

将已商定的文档分工写入计划生成入口、四阶段交接和 auto-lang 的 Specs 规约。验收以读者实际会读到的文字为准。§6 给出这套规则对 961、1001 和 1003 的判定及成文示例。

## 1. 目标

让使用者在模块 Spec 查到当前能依赖的行为、适用条件和失败边界。路线会话维护版本范围、次序与下一步；长期 Design 按功能主题更新，保存已批准设计、约束和取舍。单计划的执行、测试结果、完成报告进入该 Plan。跨计划里程碑可以单独报告。

## 2. 架构方案

沿用现有 Plan 状态、§0–10、SD/AC/T 编号和影响字段。新建时同时定位设计依据与已交付基线，执行时修正拟议增量，复审核实际契约，合并时落经复审的增量。路线可复用已有 Design；普通局部修复可直接从现行契约开始。

## 3. 技术栈

Markdown、YAML frontmatter、Bash 计划骨架。六类知识视图仍由应用派生。此变更限于撰写与交接规则。

## 4. 需求分析与背景调查

- 用户已确认文档分工，要求更新 auto-plan 的计划文件并查看未来 Specs 的效果。两个示例按本次会话核对。现有归档计划、执行中的 1003、全库文档重写及应用后端均在本次修改范围之外。
- auto-lang 基线 `df190c1f5`。从 [Specs 总览](../specs/overview.md) 核到 [Design 27 §3](../design/27-knowledge-base-lifecycle.md)、[auto-plan 设计](../design/autoplan-spec-ledger.md)、[Specs 规约 §4、§6](../specs/README.md) 与 [取号脚本及其内嵌模板](../../scripts/new-plan.sh)。现行规约还要求 merge 回写模块概述、目标进展，并将 designs/tests 视角指向 Specs 子目录。脚本只提示从模块 Spec 取材，尚未给出规范增量的填写示例。
- auto-musk 基线 `ccc7629`。[四技能源](../../../auto-musk/.agents/skills/)可经用户安装入口读取。new 的影响表和 review 的前后规则可沿用；work/review 仍有宽泛的“项目知识／持久决定”措辞，merge 对 Spec 目标另有路径保护。应明确“无契约影响”和“把已改变的契约放到其他目录”分别处理。
- 实施树为 `D:/autostack/.wt/lang-1004/auto-lang`（`plan-1004-dev`）和同组 `auto-musk`（`auto-lang-dev`）。当前阶段为计划审阅。

## 5. 详细设计

### 以后填写计划的方式

| 位置 | 作者要回答的问题 | 成文位置 |
|---|---|---|
| §4 背景 | 这项工作承接哪条路线或已接受设计？调用方今天能依赖什么？ | 引用路线／Design 的相关章节和已有模块 Spec；缺 Spec 时记明并核源码、测试与调用者 |
| §5 详细设计 | 哪项既有设计判断要改？哪项调用者可观察的行为会变？ | 优先修订对应 Design 章节。SD 行只描述拟交付契约的前后变化、适用范围与关联 AC |
| §6、§7 验证与验收 | 哪个入口和断言能区分旧行为与新行为？ | 测试代码承载实例，Plan 绑定验收。长期测试策略归专题设计，必要时从 Spec 引用代表性测试 |
| §9 复审与收尾 | 实际交付哪些规则？哪些只是裁定、研究或测量？路线还有哪项工作？ | 已交付规则按主题更新 Spec；本次结果、数字、限制与来源归入 Plan，路线跟踪页承接下一步 |

复核到已有行为但缺文档时，可以引用当前代码、测试和调用方补写 Spec。混合计划按每项交付分别判断。纯研究、内部重构或验证经核实确无契约变化时，在 §5 说明原因，影响路径允许为空。研究同时交付新工具或接口时仍需核其使用契约。多个计划共用同一模块主题；新增独立 Spec 需说明既有主题为何容纳不下。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/README.md | 从按知识视角指导回写，改为按实际契约影响修订已有主题或记录无影响；明确目标、决策、测试、报告的来源 | 支持按读者问题维护 Spec | AC-01、AC-02、AC-03 |

本 Plan 改变的是文档工作流程。优先更新已有规约，无需为 1004 另建业务 Spec。旧设计卡、ADR、研究页和索引可继续被引用，大规模迁移另议。

## 6. 测试设计

下面是按新口径试写的预期结果。它们供审阅和后续实测对照，尚未发布到任何模块 Spec。

| 案例 | 应可读到的文字或收尾结论 | 核对来源 |
|---|---|---|
| 961 · 已交付的数值语义 | **Spec 示例**　Numeric 源码支持 `.shl(n)` 和 `.shr(n)`。值和移位量须为同一整数类型。有符号移位量为负，或移位量达到或超过类型位宽时，运行触发 `ShiftOutOfRange`。四端验收过程留在 961 §9。 | [数值 Spec](../specs/ace-migration/m2-numbers.md)、[源码用例](../../experimental/ace/crates/core/tests/cases/numeric_source.rs)、[961 归档](archive/961-ace-m2-numbers.md) |
| 1001 · 缓存治理工具 | **Spec 示例**　`--preview --policy` 生成清理快照。`--apply-manifest --idle-confirmed` 使用显式快照，并在执行前重核候选集；遇到活动构建锁时拒绝清理。| [已有工具契约](../specs/auto-test-case/build-storage-lifecycle.md) |
| 1001 · 调试策略研究 | **§5 无生产默认值增量**　对照实验完成，采用 C 的决定及理由归 Design 53；冻结输入、样例数字、局限归 1001 §9 或其引用的实测节。保留上述缓存治理的实际增量。| [既有研究页](../specs/repository-boundaries/generated-debug-policy-study.md)、[Design 53 §9](../design/53-independent-build-and-generated-cargo-governance.md)、[1001 归档](archive/1001-incremental-cache-lifecycle-and-generated-debug-policy-validation.md) |
| 1003 · 生产默认值切换 | **交付后方可采用的示例**　受管生成工程的 dev 默认保留 workspace 成员的行号，并关闭非成员依赖的调试信息。届时补上验收证实的覆盖优先级、完整调试恢复方式和适用入口。一次构建的字节数与耗时保存在 1003。执行中的分支和尚待验收的承诺不发布为主线契约。 | [1003 §5、§7、§9](1003-generated-debug-policy-c-production.md)、[主线生成器](../../crates/auto-generation-core/src/rust_workspace.rs) |

验证方法：在隔离样本目录使用实际取号脚本生成一份新骨架，检查 §0–10、影响字段与 §5；按四阶段的新说明对上述四行做纸面复演。反例包括把研究数据写成产品保证、提前宣告 1003 已切默认，以及用“纯研究”漏掉 1001 的工具契约。由复审者从源文件独立核结论。执行阶段再运行，当前仅已核对源文与实例。

## 7. 验收标准

- AC-01（work）实际生成的骨架与四技能对目标、设计、Spec 和 Plan 收尾的归属一致，现有编号章节和解析字段可用。
- AC-02（work）§6 四行逐项得到预期，已有主题优先、理由充分的无影响和旧能力补文档都可表达。
- AC-03（review）独立复审核收窄后的规约及反例，真实契约变化仍受目标路径保护，研究报告和待交付行为不会进入现行契约。
- AC-04（merge）两个主检出及用户实际使用的四个技能入口读回已审版本，生成的 Plan 能引用新规约，归档记录交付身份。

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖 | 改动与核验 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | — | 更新 Design 27、auto-plan 设计及 Specs README 中受影响的职责、回写和六视角表；核对三者无冲突 | AC-01、02 |
| [ ] | T-02 | work | T-01 | 更新 auto-musk 四份 SKILL.md 对应的起草、执行、复审和收尾段，以及 auto-lang 的 new-plan.sh 骨架；保持取号和原状态机 | AC-01 |
| [ ] | T-03 | work | T-02 | 在隔离目录生成样本，按 §6 检查正例、反例、空影响与模板兼容，记录结果 | AC-01、02 |
| [ ] | T-04 | review | T-03 | 从修改和样本独立核验 AC-01 至 AC-03，冻结实际增量 | AC-03 |
| [ ] | T-05 | merge | T-04 | 按仓库规约合入、检查实际技能入口与规约，写本计划收尾摘要并归档 | AC-04 |

## 9. 复审记录

2026-11-11 草案审阅入口。stage=new，PLAN-1004 r1。已核两仓来源及三个案例的四项归属。next=审阅本次跨仓模板与四技能修改范围后进入 work。实施、独立复审和正式落地待执行。

## 10. 待澄清事项

文档分工已确认。样例按已读基线解释；实施时如 1003 已交付，以主线实际内容及归档证据更新复演结果。
