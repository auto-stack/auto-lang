# PLAN-755 外部审计打回——应对证据（2026-10-11）

## 0. 结论（TL;DR）

PLAN-755 已落地交付物在 master（HEAD `d8fba471b`）**复核全部等价通过**：守卫 4/4、
两原反例 2/2、全档 th 102 run/0 timeout。若外部审计的打回依据是"主检出日常档
红数远超 755 复审记录的 14 红"，其实测红源不在 755 触面，而是主检出工作树上的
**另一任务未提交 WIP：`blueprints/` 整目录 68 文件被删（HEAD 在树、工作树缺失）**
——PLAN-758 立项时已登记"主检出已有其他任务 blueprints 删除与临时文件，禁止纳入
或重置"。该 WIP 归属处置需用户裁定（恢复 = `git checkout -- blueprints/` 一条命令；
若为蓄意改名/迁移中间态则须由其属主会话收尾），本会话依 auto-plan-work 技能预检规则
与 758 既定裁定不触碰。

## 1. 审计发现来源说明

用户 2026-10-11 转述"计划755被外部审计打回"。仓内（docs/plans/**、reports/**、
KNOWN-DEBT、specs.json 账本、git 分支/stash、auto-os/auto-musk/auto-ai/auto-down
兄弟仓、会话归档）均未检索到该外部审计的落盘工件。本报告以"审计者会在主检出跑
门禁"为可复现假设，独立重建审计视图并逐项归因；若审计原文另有所指，请提供原文
对账。

## 2. master 落地态复核（2026-10-11 实测，主检出 HEAD `d8fba471b`）

| 项 | 755 收据/复审记录 | 本次实测 | 判定 |
|---|---|---|---|
| plan755 守卫族 | 4/4 | 4/4 PASS（0.242s）：`plan755_http_public_face_contract_coverage`、`plan755_http_contracts_survive_cffi_merge`、`panic_payload_is_forwarded_through_real_spawn_path`、`panic_payload_text_covers_string_static_and_foreign` | 等价 |
| 两原反例（真实 `crate::run` + wire） | 2/2 | 2/2 PASS（0.641s/0.658s）：`http_e2e_plan730_vm_plain_endpoint_untouched`、`e2e_a_redirect_302_with_location` | 等价 |
| 全档 th（http e2e） | 102/101 pass/1 fail/0 timeout（串行 200.4s） | **102 run/100 pass/2 fail/0 timeout**（40.3s 并行档）：红 1=`back_proxy_tests::http_e2e_back_proxy_real_routes_corpora_data_face`（在册基线红，同形未恶化）；红 2=`plan730_vm_quoted_boundary_wire` 并行 flake——**隔离复跑 PASS（0.627s）**，与 T-05 收据"并行态曾 flake、隔离与串行均 PASS"同签名 | 等价（14 timeout 族维持清零） |
| 日常档 `cargo t --no-fail-fast` | 5229 run/14 fail | **5231 run/32 fail** | 表面恶化，逐项归因见 §3，全部非 755 触面 |

## 3. 日常档 32 红逐项分类

### 3.1 在册基线红 13 项（755 T-06 记录 14 项中今日复现 13）

`musk_p053_4`、`plan606`（10.2s 长尾）、`schema_drift_fence`、`queue_coverage_drift_fence`、
`kitchen_sink_page_in_sync`、`ash_stream_leak_probe`、`projector_counter_layout_and_hits`、
`p748_t00_handler_{static,dynamic}_arm`、`iced renderer ×2`、`plan502_m3`、`plan484_024`。
755 记录中的 `concurrent_atomic_writes_never_tear` 今日未复现（flake 类）。

### 3.2 主检出未提交 blueprints/ 删除致红 18 项（今日新增红主体）

直接读盘 Os NotFound（实证 `plan640_bp_tests.rs:103` panic `Os { code: 3, NotFound }`）：
plan639×1、plan640×5、plan643×3、plan649×4、plan657×2、plan670 corpus_041。

间接符号缺失（`examples/ui/041-auto-edit/pac.at:16`："共享 filetree bp 依赖
（tree_util 支撑件跨包直用；L1 零副本）"→ 物理依赖
`blueprints/navigation/filetree/tree_util.at`，文件在工作树被删 → 链接
`Undefined symbol: tree_util.toggle_id in module App`）：`launch_three_real_apps_via_registry_resolver`、
`native_gate_runtime_views_of_six`。

### 3.3 并行负载 flake 1 项

`plan716_supply::diff_window_census`——隔离复跑 PASS（1.587s 组内）。

### 3.4 汇总

13（在册）+ 18（blueprints WIP 删除）+ 1（flake）= 32，全部闭合，无 755 触面新红。

## 4. blueprints/ WIP 事实记录

- `git status`：` D blueprints/**` 共 68 文件（5391 行），未暂存、未提交；HEAD
  `d8fba471b` 树内完整保留 `blueprints/`（`git ls-tree HEAD` 可证）。
- 磁盘实测：`blueprints/` 目录不存在（含 `navigation/filetree/tree_util.at`、`pac.at`）。
- 归属登记：[758-autojade-rename-and-promo.md](../758-autojade-rename-and-promo.md)
  §4（2026-10-10，commit `d3fc793f6`）："主检出已有其他任务 blueprints 删除与临时
  文件，禁止纳入或重置。"——即该删除早于 755 合并即存在，属另一任务。
- 755 门禁当时跑在 lang-755 worktree（干净树、blueprints 在位），故 14 红基线不含
  本族；主检出工作树态不在 755 复审视野内——这是"收据 14 红 vs 审计 32 红"差异的
  全部解释。

## 5. 处置路由（2026-10-11 用户裁定并执行）

用户裁定：审计实质即门禁红、无另附 findings；blueprints/ 删除判废弃物，**恢复**。

- 执行：`git checkout -- blueprints/`（2026-10-11），恢复后 `git status` blueprints 删除
  计数 0，`blueprints/navigation/filetree/tree_util.at` 回盘。
- 定向复验：受影响 7 族 + 2 间接项 nextest 单滤串 **31/31 PASS**（1.804s）——§3.2
  的 18 项断红全部转绿。
- 全档终态：`cargo t --no-fail-fast` **5231 run / 5218 pass / 13 fail**（111.6s）——
  13 红全部为 §3.1 在册基线红（较 755 收据 14 红少 1：concurrent_atomic_writes
  flake 本轮未现）。主检出工作树对 HEAD 的唯一残留差异已消除，master 恢复
  零 WIP 纪律态（除本报告等簿记类新增文件）。

**结案**：PLAN-755 落地态经重建审计全维复核无罪；打回红源（blueprints 未提交
删除）已按用户裁定清除并复验。归档计划不回退、不另立修复计划。

## 6. 本报告证据命令（主检出，2026-10-11）

- `cargo nextest run -p auto-lang --lib plan755` → 4/4 PASS
- `cargo nextest run -p auto-lang --lib --features test-http-e2e http_e2e_plan730_vm_plain_endpoint_untouched e2e_a_redirect_302_with_location` → 2/2 PASS
- `cargo th --no-fail-fast` → 102/100/2/0（40.3s）
- `cargo nextest run -p auto-lang --lib --features test-http-e2e http_e2e_plan730_vm_quoted_boundary_wire` → 1/1 PASS
- `cargo t --no-fail-fast` → 5231/5199/32（143.7s）
- `cargo nextest run -p auto-lang --lib plan640_bp_tests::t02_vm_track_signup_reference` → Os NotFound @plan640_bp_tests.rs:103
- `cargo nextest run -p auto-lang --lib plan716_supply::diff_window_census launch_three_real_apps_via_registry_resolver native_gate_runtime_views_of_six` → 1 pass + 2 fail（041-auto-edit `Undefined symbol: tree_util.toggle_id`）
- 恢复后：`git checkout -- blueprints/`；7 族+2 间接项单滤串 31/31 PASS（1.804s）；`cargo t --no-fail-fast` → 5231/5218/13（111.6s，全为在册基线红）
