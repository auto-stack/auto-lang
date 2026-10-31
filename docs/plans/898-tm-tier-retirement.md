---
plan_id: PLAN-898
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: tm-tier-retirement
author: [zcode-session 2026-10-31]
created_at: 2026-10-31
updated_at: 2026-10-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: [auto-test-case/module-selection(888-era tm 段), auto-test-case/periodic-batch(tf --batch 段)]
new_spec_components: [auto-test-case/periodic-batch(cargo t all 形态)]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-test-case, test-infrastructure]
current_step: 0
total_steps: 6
---

# [PLAN-898] tm-tier 概念清退（单一编译 profile 第一步）

## 变更摘要

PLAN-896 已让 `cargo tm` 退出（exit 2）并交付固定组范式（`cargo t <组>`），但
tm/tier 的尸体仍在：别名、引擎脚本、冻结目录、周期 tf/taa/ta/t3 腿、治理测试与
三副本技能文档全部残留，且 auto-plan-* 技能仍在教学已失效指令（merge/work/review
验证门因此带病运行）。本计划按 [Design 52](../design/52-single-compile-profile-consolidation.md)
完成清退：删尽 tm/tier 概念，周期批量回归改 `cargo t all`，技能与文档对齐现行范式。

## 目标

1. `cargo tm`、`--legacy-tier`、`tf/taa/ta/t3` 概念在**活跃代码/脚本/配置/技能**中零残留
   （archive 计划与 KNOWN-DEBT 历史记录除外）。
2. 周期批量回归门 = `cargo t all`（79→75 组并集，见 T-04），auto-plan-regress 技能
   与收据格式同步；due 规则（plan%5 / >48h 有合并）不变。
3. 保留 PLAN-896 现行机制不动：固定组发现/执行/缓存所有权（fixed-tests.py、
   test-groups.toml 主体、nextest 三配置、case/python runner、重档纪律）。
4. `cargo t all` 语义可验证：删 legacy-tv/tt/tb/th 兼容组后选择面不缩（select-only 对比）。

## 架构方案

无新架构；纯退役 + 文档/技能对齐。依赖方向：本计划不动 Cargo 依赖与 feature
（那是 Plan A/B）。唯一行为变化：周期批量从"recipe→leaf 调度"变为"固定组显式并集"，
与 PLAN-896 §2.3 的单调用合并语义一致。

## 需求分析与背景调查

- PLAN-896（docs/plans/archive/896-*.md，r4/r7）：tm 退役、固定组范式、`cargo t` 入口；
  tv/tt/tb/th 兼容期至 2026-11-30（本计划经用户 2026-10-31 表态直接豁免）。
- 消费者盘点（2026-10-31 实测）：`cargo tf --batch` 仅 auto-plan-regress 技能引用；
  cross-repo-tiers.py/check-tier-selection-minimums.py 仅被已退役脚本与
  run-full-regression.py 自身引用；冻结目录仅被将删的引擎脚本与治理测试引用。
- 治理测试分组：infra-modules/infra-purpose/infra-regression 三组绑定将删设施（随删）；
  infra-meter/infra-pilot-cli/infra-fixed 绑定存活设施（保留）。

## 详细设计

### 删除清单（SET A 确定删 / SET B 存疑先核）

SET A：
- `.cargo/config.toml`：别名 `tm/tv/tt/tb/th/tf/taa/ta/t3`（保留 `t`）
- `scripts/full-regression-driver/src/main.rs`：tm 退役桩、`--legacy-tier` 分支、
  tf/taa/ta/t3 转发（仅剩 `t` 直通 fixed-tests.py）
- `scripts/run-full-regression.py`、`scripts/test_full_regression.py`
- `scripts/fixed-tests.py`：`--legacy-tier` 选项与 t→[daily] 等映射
- SET A 引擎（scripts/test-governance/）：`module_command.py module_catalog.py
  select_tests.py run_selected.py discover_modules.py discover_legacy.py
  build_modules.py legacy_catalog.py catalog_contract.py impact_graph.py
  measure_modules.py audit_test_purposes.py extract_group_evidence.py
  module_observations.py` 与冻结目录 `modules.json recipes.json legacy-contract.json
  legacy-wrapper-contract.json periodic-module-schedule.json top-level-modules.json
  impact_rules.json catalog/`
- `scripts/full-regression-*-ids.txt`、`*-selected.txt`、
  `tier-selection-minimums.json.txt`、`check-tier-selection-minimums.py`、
  `cross-repo-tiers.py`
- 治理测试与组：`scripts/tests/test_module_selection.py`、
  `scripts/tests/test_test_purpose_policy.py` + test-groups.toml 的
  `infra-modules`/`infra-purpose`/`infra-regression` 组；`legacy-tv/tt/tb/th` 四组
- 测试目的策略 json 若仅服务 audit_test_purposes（`test-purpose-policy.json`、
  `partition-policy.json` 归 SET B，见下）

SET B（grep 存活消费者——fixed-tests.py/prepare_tools.py/python_tests.py/
case_artifacts.py/recipe_runtime.py/native_meter.py/process_capture.py——为零才删，
否则保留并在复审记录具名）：`build_storage.py discover_cases.py inventory.py
input_inventory.py recipe_runtime.py partition_policy.py external_workloads.py
output_admission.py coverage-matrix.json case-control-oracles.json
atomic-contract.json input-contract.json measurement-contract.json
compiler-auxiliaries.json toolchains.json daily-admission-proposal.json
python_tests.py prepare_tools.py native_meter.py`

### 文档对齐

- `AGENTS.md`：删 PLAN-888 r3"模块选测与验证门"节残留与 tv/tt/tb/th 兼容句、
  tf/taa/ta/t3 周期节；批量回归改述 `cargo t all`（PLAN-896 节为权威，仅去残留）
- `scripts/test-governance/README.md`、`scripts/full-regression.md`：删"历史实现
  与审计说明"以下旧范式段落，主述固定组
- `docs/specs/auto-test-case/project.md`：888 历史节改过去时（"以 cargo tm 执行"→
  "曾以 cargo tm 执行，898 已退役"）
- `docs/plans/893-ace-test-framework-and-slow-case-attribution.md`（drafting）：
  验证面引用 cargo tm → 改 `cargo t` 组；若整体已被 896 取代则在文头标注 superseded
- `docs/plans/KNOWN-DEBT-AND-RISKS.md`：新增一条"tm/tier 概念清退（898）"，不改历史记录

### 技能更新（3 副本，内容当前逐行一致）

`.agents/skills/`（仓内，随本计划提交）+ `C:/Users/zhaop/.zcode/skills/` +
`C:/Users/zhaop/.agents/skills/`（用户级，先出 diff 供过目再同步）：
- auto-plan-merge：merge 验证门 `cargo tm --changes/--select-only/--out` →
  `cargo t <受影响组> --select-only` 预览 + 组并集执行；`cargo tf --batch` due 检查改述
- auto-plan-work：受影响模块 → 受影响固定组（`cargo t <组>`）
- auto-plan-new：验证设计段同步
- auto-plan-review：finding 重放与落地前门同步
- auto-plan-regress：语言腿 `cargo tf --batch`（periodic-module-schedule 调度）→
  `cargo t all`（固定组显式并集）；收据 `command` 字段与 legs 说明同步；due 规则不变
- autoui-verifier / commit：个别引用行同步
- 用户级副本属用户资产：执行 T-06 前把仓内 diff 摘要呈报用户过目

### 收据

`docs/plans/.last-batch-regression.json` 新运行起 `command: "cargo t all"`、
`legs` 记实际组并集来源（legacy 域组清单）；旧字段语义保持。

## 测试设计

- SET B 删除判定：每文件 `grep -l <file-stem> <survivors>` 零命中才删
- `cargo t --list` 正常；`cargo t`（daily）绿
- 选择面守恒：删 legacy-tv/tt/tb/th 组前后 `cargo t --select-only all`（或
  fixed-tests.py 等价 select-only 输出）测试名集合对比——四组的测试名必须 ⊆ 其余组并集
- python 治理测试（infra-fixed/meter/pilot-cli）绿；将删三组先确证随删无残引
- 全仓 grep 审计：`git grep -nE "cargo tm|--legacy-tier|tf --batch|taa|cargo t3"`
  活跃路径零命中（archive/KNOWN-DEBT/本计划与设计文档自身除外）
- 技能三副本 grep 同口径零残留

## 验收标准

1. [ ] 别名仅剩 `t`；`cargo tm` 不再是已注册别名（原样输入报 unknown alias）
2. [ ] SET A 全删；SET B 零消费者才删，保留者复审记录具名
3. [ ] `cargo t all` 在冻结提交全绿（或仅已知红，逐条归属）；选择面守恒证明归档
4. [ ] AGENTS/README/specs/893 无 tm/tier 现行时态残留
5. [ ] 技能 3 副本更新且用户级副本经 diff 过目
6. [ ] 复审走 /auto-plan:review，spec delta 如 frontmatter 所列回写

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- T-01 基线快照：`cargo t --select-only all > /tmp/all-before.txt`；`cargo t`（daily）
  记录绿；记录 `--list` 组数。[验证：两输出非空]
- T-02 SET A 删除 + 驱动收敛（config.toml 别名、main.rs、fixed-tests.py、
  run-full-regression.py、引擎/目录/txt/两脚本、治理测试三组+四 legacy 组）。
  [验证：`cargo t --list`；`cargo tm` 报 unknown alias；python -m pytest 存活治理测试]
- T-03 SET B 逐文件消费者核查，零消费者者删，否则留档。[验证：grep 审计清单入复审记录]
- T-04 选择面守恒：`cargo t --select-only all > /tmp/all-after.txt`，差集仅允许
  为空（四组测试名 ⊆ 并集）。[验证：diff 输出空或具名解释]
- T-05 文档对齐（AGENTS/README×2/project.md/893/KNOWN-DEBT）。[验证：grep 现行时态零残留]
- T-06 技能更新：仓内副本改毕提交；用户级两副本 diff 摘要呈报用户后同步。
  [验证：三副本 grep 零残留]
- T-07 冻结合入门：worktree 内 `cargo t all` 单实例全跑 + 收尾 grep 审计。
  [验证：全绿或已知红归属清单]

## 复审记录

## 待澄清事项

- SET B 中 partition-policy/test-purpose-policy 族是否已被 896 固定组机制完全取代：
  执行时以消费者核查结果为准，存疑保留并记录（不影响本计划目标）。
