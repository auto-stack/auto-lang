---
plan_id: PLAN-779
status: drafting
feature_name: UI Frame Native 宿主注入探针
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-13
plan_revision: 1
current_step: 0
total_steps: 7
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md]
new_spec_components: []
touched_goals: [GOAL-007, GOAL-003]
affects: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md]
---

# [PLAN-779] UI Frame Native 宿主注入探针

## 0. 变更摘要

以frame.begin_ms/present_ms两函数验证native定义与实现装配分离。真实external provider供值，保持现有UI生产装配与未捕获行为；之后才能推广clipboard/media/terminal。

## 1. 目标

剪断选定frame shim的core→ui::frame_bench直接实现依赖，保持canonical名、ID9918/9919、i32 clamp、未捕获0。生产UI从host注入；两VM实例可用不同provider；register/merge的contract/proof规则保持。

非目标：全natives迁移、新VM线程模型/NanoValue、async终值/776合同、媒体寿命、全UI独立仓或通用动态插件平台。

## 2. 架构方案

复用NativeInterface的register/typed contract/binding revision/proof；最小FrameProvider或等价能力接口由host factory装配。Core提供合法未捕获0，UI adapter读现有frame_bench；不另造native系统，不用全局单例provider造成实例串线。

## 3. 技术栈

Rust、现有Arc/NativeInterface/SDK宿主装配；provider Send+Sync不代表!Send VM可跨线程。

## 4. 需求分析与背景调查

用户2026-10-13只授权本轮调查与报告/规划；实施需确认本Plan revision 1，不创建新仓、不发布/合入，不改其他在途Plan/WIP。

前置777；建议778后顺序实施。Specs vm/ui、Design37与 reports/v06-modularization-baseline.jsonl hash。实锚vm/native_catalog.rs:63–64；native.rs::production/register_std_shims/shim_frame_begin_ms/shim_frame_present_ms；ui/frame_bench.rs的frame_*与note_*；AutoVM::new/merge_native_interface、UI vm_bridge/handler_codegen/lib装载路径；a2r_std.rs的frame同源语义。

## 5. 详细设计

T01枚举全部选定族注册/调用，冻结无provider行为、ID/contract/值编码。拟新 `src/vm/host_capabilities.rs` 或native等价API，`src/ui/frame_provider.rs`宿主adapter，`test/fixtures/modularity/native-provider-consumer/`独立无UI类型fixture。

UI所有受支持VM创建路径必须注入；core选定实现不再直读frame_bench，不能只多加接口而production继续旁路旧全局。普通Core路径按当前未捕获规则；a2r已有frame调用保持或等价adapter，剩余依赖明确，不冒充已拆runtime。

fake返回非零且两值不同，真实VM stdout证明它被调用；大于i32::MAX仍clamp，binding identity不得以动态分配ID误覆盖9918/9919。T01若发现需要改变已有行为须先修订，不silentfallback为0过门。

### 规范增量

| delta_id | 操作 | docs/specs target | before/after | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | frame由core读UI→host供给，ID/编码/降级不变 | 依赖倒置 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | 明确frame provider生产注入者与时点 | 不漏真实路径 | AC-03 |

## 6. 测试设计

实现者未指派；work自检后由独立上下文review在同一唯一worktree验收，不修代码；merge仅处理已验收沉淀/归档/清理。本草案没有派遣agent/chat。

T03拟建external gate `cargo test --manifest-path test/fixtures/modularity/native-provider-consumer/Cargo.toml`；worktree `cargo check -p auto-lang`+裸`cargo t`+`cargo tv`。仅改UI codegen才tu，不碰aavm/trans不taa/tt，tf按batch规则。

| V | AC/不变量 | setup/实际入口（T03建） | oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/真注入 | A=(17,31)、B=(43,59)分别装VM执行两native | stdout分别真实17/31、43/59，不是0/0，实例不串线 | 无heap能力 |
| V-02 | AC-02/表示 | 未捕获/无provider、大值i32::MAX+1 | 0、i32::MAX，int lane不变 | 标量族 |
| V-03 | AC-02/identity | 名/ID查找、merge/重注册/proof既有路径 | 9918/9919不撞别族；冲突诊断/proof按revision失效 | 保留现有contract规则 |
| V-04 | AC-03/host | frame_bench note→实际UI host创建VM读取 | 真实观测到值，注入在使用前，core直引被剪断后仍通 | 实际受支持创建路径 |
| V-05 | AC-03/兼容 | 既有corpus+a2r frame约定 | 无新增红/漂移，剩余族与依赖明示 | 不等完整拆仓 |

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 选定frame core无UI类型/直接实现调用，external实值到VM；V01 |
| AC-02 | work | ID/编码/降级/实例/contract/proof正确；V01–03 |
| AC-03 | work | 真UI路径注入，其他装配不损；V04/05；未迁族清单 |
| AC-04 | review | 独立逐AC与旁路扫描，bindcommit/revision |
| AC-05 | merge | SD01/02按pilot实情沉淀、归档、guard-clean清理 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 777+确认r1 | 点检catalog/production/UI创建路径；冻结注入与缺席矩阵 | 01/02/03 |
| [ ] | T-02 | work | T01 | provider+frame shims接线，保留ID，剪core frame直引 | 01/02 |
| [ ] | T-03 | work | T02 | 独立consumer，V01–03真VM assertions | 01/02 |
| [ ] | T-04 | work | T03 | UI adapter与所有生产装配；V04/05；不改media/776 | 03 |
| [ ] | T-05 | work | T04 | check+t+tv+externalgate与宿主实测，后续native族清单 | 01/02/03 |
| [ ] | T-06 | review | T05 | 独立review，不能外推heap/async已解决 | 04 |
| [ ] | T-07 | merge | reviewpass | Spec/归档/合流/guard-clean清理 | 05 |

拟tree `D:/autostack/.wt/lang-779/auto-lang`，branch `plan-779-dev`；构建/测试在该树，Plan簿记在master，禁止链接。

## 9. 复审记录

2026-10-13 stage=new，r1，outcome=pass（草案preflight）；next=确认后work。0/7任务，external/host probe均未创建运行，不是独立实现复审pass。

## 10. 待澄清事项

T01遇到公共Core既有行为改变先提出差异/最小修订；不能缺注入却返回0假过。更大host接口、media handles/async生命周期另立Plan。
