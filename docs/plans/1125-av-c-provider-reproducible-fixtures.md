---
plan_id: PLAN-1125
status: drafting
feature_name: "AV/C 语言验收提供者工件可重建"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-22
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-967, PLAN-978, PLAN-1093, PLAN-1096, PLAN-1114]
supersedes_spec_components: [docs/specs/auto-av/language-parity.md, docs/specs/ace-migration/m2-host.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-abi, auto-test-case, auto-av, ace-m2-build]
---

# [PLAN-1125] AV/C 语言验收提供者工件可重建

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](archive/1096-av-parity-full-language-certification.md)。

## 1. 目标

让干净 Windows 检出使用可追溯的 C provider 源和准备流程，重放 1093 的显式 C 标准库正例及 C 头部拼写反例，关闭 1114 复验仍报告的两条工件缺失。

## 2. 架构方案

入库的 C provider fixture 源和配方 → 现有显式 build/prepare → AV worker 装配已钉 provider → 普通源码调用与受检诊断。构建使用测试既有准备段，运行阶段消费已准备工件。

## 3. 技术栈

`crates/auto-lang/test/cases/language/ffi/imports-callbacks.case.ad`、ACE provider 准备及 worker、现有 Windows C 工具链和闭包元数据。

拟实施树为 `D:/autostack/.wt/lang-1125/auto-lang`，分支 `plan-1125-dev`。

## 4. 需求分析与背景调查

[1093](archive/1093-av-parity-foreign-language-surface.md)保留真实 C 调用及原反例。[1114 冻结收据](archive/1114-ace-av-rs-stdlib-providers.md)点名 `av.parity.1093.explicit-c-library` 与 `av.parity.1093.c-header-spelling-rejected`，失败为 `foreign.prepared`，C 闭包源未入库。活动 [1108](1108-batch-regression-gate-repair-after-1098.md)负责更广的固定门红项和 `source_foreign` authority 对账，T-01 对照共享准备故障，复用已经交付的修复。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

## 5. 详细设计

把受控 C 输入、签名、构建参数及实际输出身份绑定到已有测试准备流程。独立验证正例确实进入 C 并调用库，反例实际到达约定的源码拒绝阶段。按同一闭包重放缺工件、撤权和错误身份。生成文件落在执行计划区，验收后依现行规则清理。旧完整报表里的其他基面红保留原治理队列；同根因改动只交一处。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/auto-av/language-parity.md`](../specs/auto-av/language-parity.md) | 补齐 1093 Windows 两项的准备入口、独立 oracle 和范围 | AC-01/02 |
| SD-02 | modify | [`docs/specs/ace-migration/m2-host.md`](../specs/ace-migration/m2-host.md) | 补记 C 测试 provider 重建来源、身份和显式准备边界 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01 · T-02 | 干净 Windows 检出，无旧 provider 根或缓存，显式准备并复放正例 | `explicit-c-library` 调用真实 C `strlen`，结果为 5，记录源及工件身份。 |
| V-02 | AC-01 · T-02 | 同一准备闭包复放 `c-header-spelling-rejected` | 到达 Case 当前约定的阶段/诊断。缺工件另触发 `foreign.prepared`，不能冒充该反例成功。 |
| V-03 | AC-02 · T-03 | 换敏感 provider 输入或制造错误身份/撤权，恢复原输入 | 有意义的已授权变更可观察，错身份/撤权具名拒绝且无调用。恢复后原正反例通过。 |

冻结验证命令为 `cargo t auto-test-case ace-abi ace-av ace-m2-build case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [ ] AC-01 · owner_stage=work。V-01/02 在干净检出可复建，旧两条必要用例都得到自己的 oracle。
- [ ] AC-02 · owner_stage=work。V-03 证明身份、来源与授权敏感。固定组保留其他已知失败的实际计数和 owner。
- [ ] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [ ] T-01 核定闭包失效原因与 1108 交界
  - owner_stage=work。以现有 recipe 核准备输入、resolver 必读字段和 ABI 红项归属，定位缺失最小源与编译入口。
- [ ] T-02 纳入 C fixture 并接正式准备入口
  - owner_stage=work。依赖 T-01。纳入可追溯测试源和配方，使用现有 prepare/replay 在干净环境运行 V-01/02。
- [ ] T-03 验证准备和授权反控
  - owner_stage=work。依赖 T-02。完成 V-03、来源身份和恢复记录，运行受影响的固定组。
- [ ] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [ ] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1125 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

## 10. 待澄清事项

本计划可直接核干净检出失败。1108 如已修共享 producer，以其当前交付复用，保留本计划两条原 Case 的独立关闭条件。
