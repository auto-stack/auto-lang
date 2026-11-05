---
plan_id: PLAN-951
status: drafting
feature_name: HIR Rust证书严格JSON序列化修复
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

# [PLAN-951] HIR Rust证书严格JSON序列化修复

## 0. 变更摘要

独立关闭941首例发现的F-941-01/D-941-1：原`CertDraft::to_json`在capabilities后缺逗号、末尾execution_config多逗号，产生不可严格解析的证书。仅修producer序列化、补真实JSON解析回归与原worker重验；不宽容坏JSON，不扩语言/后端能力，不替换legacy A2R的38格认证身份。

## 1. 目标

scalar、record-text、collections三种Rust证书均产生合法UTF-8 JSON，字段/枚举/完整actual/reference/toolchain身份、原预算及daily/cert分轴保持。空/非空capabilities、execution_config=null/有效canonical配置都以实际JSON decoder验证，不以typed draft合法或两次坏字节相等冒证据。

## 2. 架构方案

保留手写serializer及字段顺序，只修对象字段分隔。回归放既有`ace-a2x-worker` conformance test target，复用其已有serde_json依赖，显式注册新测试模块；不引入依赖或另建runner。显式原worker继续使用既有受管编译/执行。

## 3. 技术栈

`experimental/ace/crates/a2x-rust/src/cert.rs`；`experimental/ace/crates/a2x-worker/tests/{conformance.rs,cases/cert_json_951.rs}`（新模块）；现`cargo t ace-a2x`固定组及`--test-name cert_json_951`定向入口。

## 4. 需求分析与背景调查

授权：2026-11-06用户答复“授权独立修复并继续（推荐）”，明确允许本独立包work/review/merge及返回941；不再重问同范围授权。取材main `6e2abe9fe`，941检查点`91968fd40`。原scope的真实Rust证书hash `63a49eab9a4d8f9a0cf9b1bc3ee7947dcf481a1beaf8f28ac91b51484f52c8ad`，标准parser报line36 column3；C正控合法。原测试只验typed合法和重复序列化稳定。当前Specs要求严格证书，不授权parser修补。

编号簿起点949已被占用，取号前对账全部在途/归档文件，将落后游标提升到实际最大编号+1，再运行`scripts/new-plan.sh a2r-certificate-json-serialization`；脚本分配951（并发另包950被保留），next=952。其余main仅文档WIP，不纳本包提交。唯一实施树拟`D:/autostack/.wt/lang-951/auto-lang`、branch `plan-951-dev`；材料仅`D:/autostack/plan-artifacts/plan-951/`，开工登记owner，用完merge全删。941树继续保留，非本包实现/复审副本。

应用goal当前仍显示blocked，但用户已明确授权“并继续”；本包按该直接指令执行，不另建goal、不声称已由工具恢复active。

## 5. 详细设计

producer `CertDraft` → `to_json` → 原worker证书文件 → 严格JSON decoder。修capabilities字段后的逗号，最后execution_config字段不再带尾逗号；非空配置字符串继续沿原kv转义、原字段顺序不变。空数组/多项数组、三schema、null/带引号及换行的合法字符串参数在实际serde_json解析中保持原值。

### 规范增量

| delta_id | action | target | before/after | AC |
|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/a2r-hir-certification.md | 严格证书规则不变；明确typed合法/稳定序列化不足，输出须标准JSON解析并保持配置/能力值 | AC-01/02 |

## 6. 测试设计

| V | 输入/入口 | 独立可观察oracle/关闭条件 |
|---|---|---|
| V-01 | 新`cert_json_951`模块，三Rust schema与空/非空能力、null/collections有效配置 | serde_json实际解析原serializer输出；修前失败、修后通过；关键字段与原typed输入逐值相等 |
| V-02 | 同测试模块中带转义的配置/字符串、重复序列化 | 合法JSON且解码值不变，输出字节稳定；不只是字面标点匹配 |
| V-03 | 原a2x-worker对941同源Q4-001及独立reference真实certify | actual/reference各value:42，证书严格解析/完整文件hash可核；C证书正控仍合法 |
| V-04 | 最终冻结`cargo t ace-a2x` | 原完整职责组全部原条件执行，记录执行/排除/known red，不重跑无关全仓/全性能 |

实现与final review同树顺序进行。无授权spawn/chat，复审由当前会话重新读取冻结diff/AC与真实证据，并明确同一上下文的限制，不冒称独立agent。merge冻结完整组只跑一次；此小serializer修复不改变编译/运行性能算法，不另重采正式P90。

## 7. 验收标准

| ID | owner_stage | 条件 |
|---|---|---|
| AC-01 | work | 三schema、空/非空能力、null/有效配置的真实JSON解析均通过且字段保真，修前红证据保留 |
| AC-02 | work | 既有worker完整实际认证+独立reference仍正确，证书完整可读，C正控不退，schema/预算/legacy38守卫不改 |
| AC-03 | review | 冻结SHA逐AC/SD/遗漏/健康复审通过，明确同会话复审限制 |
| AC-04 | merge | 冻结ace-a2x职责组与必要消费者门通过，SD主线读回、线性合入归档、全部材料/guard清理可核 |

## 8. 执行步骤

| ID | owner_stage | 内容 | 状态 |
|---|---|---|---|
| T-01 | work | 建专属树/scope，实际serializer解析回归修前红 | [ ] |
| T-02 | work | 修serializer并补三schema/配置/能力/转义字段保真，定向回归通过 | [ ] |
| T-03 | work | 原worker实际认证重放/C正控，准备SD，提交冻结实现 | [ ] |
| T-04 | review | 同树重新核AC/SD/反例/健康，提交绑定final verdict | [ ] |
| T-05 | merge | 冻结完整ace-a2x组、必要消费者，线性合入/Spec读回/归档 | [ ] |
| T-06 | merge | 所有输出对账、scope全删、双仓guard后清树分支组，回941 | [ ] |

## 9. 复审记录

2026-11-06 stage:new | PLAN-951 r1 | outcome:pass | 用户明确授权独立修复并继续941 | next:work T-01；不改941原验收/目标。

## 10. 待澄清事项

无新用户决定；发现超出序列化修复的语言/资源缺陷另具名，不塞本包。
