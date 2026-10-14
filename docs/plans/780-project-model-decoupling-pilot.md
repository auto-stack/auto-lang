---
plan_id: PLAN-780
status: drafting
feature_name: 轻项目模型与 pac.at 兼容性探针
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-13
plan_revision: 1
current_step: 0
total_steps: 7
supersedes_spec_components: [docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/project-model/project.md]
touched_goals: []
affects: [docs/specs/auto-man/project.md]
---

# [PLAN-780] 轻项目模型与 pac.at 兼容性探针

## 0. 变更摘要

验证轻project-model脱离compiler/VM/UI表达并读取静态工程声明，与当前执行型配置对拍后形成全面迁移决策。新增受控入口，不切换所有生产pac读取。

## 1. 目标

小模型crate独立构建；包/dep/member/render/展示等选定字段有DTO、错误/路径来源；静态旧新归一化结果相同，执行型/unknown明确分类，不截字段假成功。语言loader/UI注册表/man各字段owner与后续adapter明确。

非目标：马上摘整个man→lang、迁UI生成器/建仓、改dep声明门/锁格式、砍动态pac、重写Atom-v2、命令路由。

## 2. 架构方案

拟新 `crates/auto-project-model` 仅依赖auto-atom/auto-val与必要纯解析库（T01核定），不依赖lang/man/gen/UI。ProjectDescriptor/DependencyDecl/RenderSpec/source信息不含Builder/IndexStore/AutoVM。compiler adapter在上层，不把整个Pac直接搬成新包。

## 3. 技术栈

Rust、现有auto_atom::AtomParser；serde仅在实际协议需求下。对拍oracle：AutoConfig::from_code→Pac::new及真实pac输入。

## 4. 需求分析与背景调查

用户2026-10-13只授权本轮调查与报告/规划；实施需确认本Plan revision 1，不创建新仓、不发布/合入，不改其他在途Plan/WIP。

前置777。Specs auto-man/project的PLAN635声明门、四名称/opens规则、frontend/runtime、Design37；hash见reports/v06-modularization-baseline.jsonl。实锚man/pac.rs:12/196、automan/asset；lang/config.rs::AutoConfig::from_code、atom.rs::AtomReader::parse（执行VM）、auto-atom/parser.rs::AtomParser::parse（不支持变量/函数/循环/插值）；ui/app_registry.rs::parse_pac_fields/parse_pac_back_project；lib装载器dep/member文本读取。

## 5. 详细设计

T01盘所有受管pac.at，分类static/dynamic/unknown；涵盖comments、裸/引号dep、members、render single/split/multi、legacy backend、四名称/locale、back.project、未知fields与路径基点。新parser无法表达的项写unsupported-static，不catch失败后默默丢字段；需要执行的旧输入由显式adapter保持。

首轮冻结最小已证实static子集，所有其他形态仍有分类/限制记录；未支持的输入不得算对拍相等。新增 `test/fixtures/modularity/project-model/` 与man `src/project_model_adapter.rs`（拟新，显式probe不改默认）。新包表示source与未知字段策略，不含网络/物化/构建效果。

执行型pac去留、manifest严格化与static默认切换后续决定。当前所有生产入口保持。模板eval仍compiler adapter，不能因为model小包存在就声称auto-man不依赖语言。

### 规范增量

| delta_id | 操作 | docs/specs target | before/after | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/project-model/project.md（新） | 实际DTO/static支持面/错误/限制 | 独立消费 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-man/project.md | 旧读取不变；增加explicit pilot/分类/后续adapter归属 | 不假记已切换 | AC-02/03 |

## 6. 测试设计

实现者未指派；work自检后由独立上下文review在同一唯一worktree验收，不修代码；merge仅处理已验收沉淀/归档/清理。本草案没有派遣agent/chat。

T02后 `cargo check -p auto-project-model`+`cargo test -p auto-project-model`；T03拟建consumer gate `cargo test --manifest-path test/fixtures/modularity/project-model/consumer/Cargo.toml`、man新module filter `cargo test -p auto-man project_model`。man Rust变更需 `cargo check -p auto-lang -p auto-man`+裸`cargo t`；不碰VM/codegen则不tv/tu/tt/taa，不生成外仓book金样。

| V | AC/不变量 | 输入/入口（T03建） | oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/独立 | consumer只依赖model | parse/build通过，normal deps无lang/man/UI | auto-atom仍用val |
| V-02 | AC-02/static | dep裸/引号、comments、members、render多形态/backend/extras | 已支持子集旧新normalizedDTO相等；不支持明确分类 | 不是全pac语法 |
| V-03 | AC-02/dynamic | 真实或专门变量/调用/插值输入 | static拒/分类，explicit旧adapter保输出/错误，无silentfallback | 不砍旧能力 |
| V-04 | AC-03/策略 | 四名称/locale、back.project、declared/ghostdep | 当前规则/路径来源不变，effects不混模型 | 对照Spec |
| V-05 | AC-03/坏输入 | 缺括号、类型错误、重复冲突、源路径错误 | 明确stage/path诊断，无网络/写项目/build副作用 | extras≠坏类型 |

## 7. 验收标准

| ID | owner_stage | 可观察結果 |
|---|---|---|
| AC-01 | work | model normal deps无compiler/UI，external真通过；V01 |
| AC-02 | work | 全受管pac分类与冻结子集对拍，dynamic/unknown不假绿；V02/03 |
| AC-03 | work | path/展示/声明策略不变，读无effects；V04/05，后续owner明确 |
| AC-04 | review | 独立分类/兼容/consumer审计，bindcommit/revision |
| AC-05 | merge | SD01/02只沉淀实际pilot、归档/guard-clean清理 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 777+确认r1 | 受管pac/读点分类、static feasibility；冻结字段/normalizedoracle | 02/03 |
| [ ] | T-02 | work | T01 | 新model包/DTO/static入口，无compiler执行类型 | 01/02 |
| [ ] | T-03 | work | T02 | 新fixtures/consumer/manadapter，V01–05，不切默认 | 01/02/03 |
| [ ] | T-04 | work | T03 | 明确man/loader/registry字段owner、后续消除文本副本接线决策 | 03 |
| [ ] | T-05 | work | T04 | 包/consumer/man scoped+t门，证明旧production读取不变 | 01/02/03 |
| [ ] | T-06 | review | T05 | 独立review，不删动态输入假等价 | 04 |
| [ ] | T-07 | merge | reviewpass | 真实Spec/归档/合流/guard-clean清理 | 05 |

拟tree `D:/autostack/.wt/lang-780/auto-lang`，branch `plan-780-dev`；构建/测试在该树，主检出只簿记。外仓代码不在本Plan改，cross-repo接线后续立项；禁止链接。

## 9. 复审记录

2026-10-13 stage=new，r1，outcome=pass（草案preflight）；next=确认后work。0/7任务；新包/fixture未创建运行，不是auto-man拆仓成功或独立实现复审pass。

## 10. 待澄清事项

T01必须明确字段/未知输入支持oracle；长期动态manifest去留另议。纯parser不足则显式保adapter+最小扩展提案，不另造完整compiler。
