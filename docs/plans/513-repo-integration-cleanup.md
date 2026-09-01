---
plan_id: PLAN-513
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: repo-integration-cleanup
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-513] 仓库整合清理——遗产计划审计处置 + 卫生回收 + 债务簿核销

## 变更摘要

502/508/511/512 四个机制级计划密集合入后，计划队列与仓库卫生出现可 enumerated
的腐烂面。本计划为**零代码改动（Category A）**的整合清理批，四线合一：

1. **遗产计划处置**（19 份非归档计划 + 3 份时点审计文档，全部经四代理取证
   审计，处置结论见「需求分析」总表）：已交付漏归档的归档、残留转账
   KNOWN-DEBT 后归档、superseded 标注后归档、活草案刷新状态段保留。
2. **仓库卫生回收**：scratch/ 132M 中 ~127M 已归档计划残留清理（p502 探针
   构建目录 122M 为最大头）；三个已跟踪遗产 spike + worktree-cleanup 打捞
   目录 `git rm` 瘦身；`.worktrees/host.pid` 死 pid 删除；spec-lint 唯一
   警告（502 断链）一行修。
3. **债务簿核销/转账**：P499-7 等已清偿未核销条目划线；407/412/420 残留
   转账为 KNOWN-DEBT 新条目（P513-x）。
4. **撞号修复**：453-w5-ext-impl-lib-methods 与已归档桌面 453 撞号，取
   新号（514）重命名。

## 目标

- **G1 队列一致性**：`docs/plans/` 下每份文件要么是证据干净的在途活件
  （frontmatter 状态与现实一致），要么进 archive；消除"头部声称待做、实际
  已交付"与"台账说未开工、实际已归档"两族漂移。
- **G2 残留转账零静默**：407/412/420 三家的未做残留全部以 KNOWN-DEBT 条目
  显式登记，不留"计划归档了但残留没人跟"的洞。
- **G3 磁盘与索引瘦身**：scratch/ 回收 ≥120M；已跟踪遗产 spike/打捞目录
  出库；spec-lint 回到 0 errors 0 warnings。
- **G4 撞号消除**：453-w5 改号 514，git 取证不再双义。
- **非目标**：任何 crates/ 源码改动（债务清偿实体如 P506-1 UAF、测试红族
  归"债务批二期"另立）；015-notes 等在途会话的未提交工作（平行会话资产，
  不动）；442 观察期未到期前的提前归档（09-03 后另行）；401/448 残余
  工作量本身的实施。

## 架构方案

纯文档/文件系统操作批，无架构内容。门禁类别 = Category A（纯文档/资产），
按 AGENTS.md 变更分级：**严禁 cargo t / docs_gen**；验证手段为 grep/git/
脚本级断言。

## 需求分析与背景调查

审计方法：四个只读子代理分组取证（语言线 6 份 / UI 线 10 份 / 队列与新式
+时点文档 / 仓库卫生清点），每份计划要求至少一条硬证据（commit 哈希 /
archive 文件名 / 代码现状）。处置总表：

### A. 直接归档（已交付，纯漏网或时点文档）

| 文件 | 证据 |
|---|---|
| 462-virtual-window-wm | 头标 ✅ 完成 2026-08-28（T1-T7+实机），交付 `08d1c1b70`；下游 463/464/465 均已归档 |
| 416-lsp-vscode-phase5-6 | 5-A..6-C 全交付（merge `7727c6571`/`6cf5d15e6`），残留 F5 已在 KNOWN-DEBT L98 |
| 458-auto-ui-theme-system | T1-T5+复审俱在（merge `9a5c93d91`），头部 "Status: Planned" 过时 |
| 441-028-launcher | 头部已有被 464 吸收声明；464 已交付归档（`45316828e`） |
| plans-status-audit-2026-08.md | 头部自称已被 08-20 版取代；零活链接 |
| plans-360-369-status-summary.md | 360-369 全部已归档，文档完全历史化 |

### B. 回填/转账后归档

| 文件 | 动作 |
|---|---|
| 405-023-realworld | 头部"token 认证空桩"过时——B1 已交付（`7f2faaa26`/merge `621c8f2e1`）；回填后归档 |
| 407-minesweeper-rust-backend | 残留 R7 动态窗口 resize + Phase 4 三后端对比 → 转 KNOWN-DEBT P513-1；归档 |
| 412-layout-gallery | 残留 §10.4 视觉通道（结构通道已绿 `ccf9ac0e9` 等）→ 转 KNOWN-DEBT P513-2；归档 |
| 420-auto-edit-tabs-workspace | 残留 P4 拖拽排序 + 挂账 #3（ActNew InvalidOpCode）/#4（dirty bool 读回乱码）→ 转 KNOWN-DEBT P513-3；归档 |
| 400-api-gen-a2r-body-transpilation | Phase 1+2 已交付（`d26429433`）；Phase 3/4 标注 superseded by 442（VM serve 路线）后归档 |
| 455-auto-ui-parity | 矩阵跟踪职能被 506/512 批量桌面化 + autoui-verifier 接管；标注 superseded 后归档 |
| plans-status-audit-2026-08-20.md | 本轮审计即接任者；归档并修三处活链接（docs/specs/overview.md:103、docs/design/00-intro.md:170、docs/handoff-2026-08-22.md:10） |

### C. 保留在途但刷新状态段（不动位置）

- 242-a2r-feature-gap-tracker（活 tracker，头明言不归档；被 415 接管的行补指针）
- 415-a2r-remaining-big-items（A/D 勾销 ✅，B/C/E 真待办保留）
- 448-autoui-syntax-improvements（滚动收集；头部"待合并"刷新为已合并 `7f4ed335c`）
- 401-autoui-examples-upgrade（残余 019/020/021/025 四项刷新为待裁定）
- 394-await-future-external-architecture（parked 设计储备，frontmatter 旧式
  `status: draft` 加注记即可）
- 439/440/509（合法活草案，零执行提交，保留）
- autos-desktop-program.md（台账回写：464/465/472/478/479 五行标已归档）
- 442-cross-platform-closure（观察期 2026-09-03 到期；**若本计划执行时
  ≥09-03 且期间无回滚**，同批归档，否则保持现状留尾注）

### D. 撞号修复

453-w5-ext-impl-lib-methods（草案，仅立项 `d0e75953b`）与已归档
`archive/453-multi-app-session-runtime.md`（桌面计划）撞号：
`bash scripts/new-plan.sh w5-ext-impl-lib-methods` 取 514 号生成骨架后，
把 453-w5 内容迁入新文件并删除旧文件（git mv 语义保留历史）。

### E. 仓库卫生回收

| 目标 | 动作 | 依据 |
|---|---|---|
| scratch/p502/m1probe + m2probe（122M） | 删除（可重建构建目录；23 张 PNG 截图保留） | 502 merge `2f1e9984f` 注"探针移主检出存档"，但构建产物无存档价值 |
| scratch/p499/（1.5M）、scratch/p498/（60K） | 删除 | 499/498 均已归档 |
| scratch/stella-shot1.mjs/stella-shot2.mjs/stella-shots/（2.8M） | 删除 | 503 已归档，视觉基线由 golden 承载 |
| scratch/p507-merge/deposit.py、scratch/p502_merge_upsert.py | 删除 | 一次性沉淀脚本，使命已完成 |
| scratch/worktree-cleanup-2026-08-23/（3.7M，已跟踪） | 先核对三份打捞补丁（428-fold-b/plan-061/auto-musk）是否已落 master：已落 → `git rm -r`；未落 → 保留并登记 P513-4 | 428 已归档 |
| scratch/spike-465/、scratch/code-editor-spike/、scratch/ime-spike/（已跟踪，均 <110K） | `git rm -r` | 465/413/452 均已归档 |
| scratch/drift_*.txt + schema_drift_audit.py（已跟踪） | 保留不动（schema_drift 门禁脚本唯一副本，可选 promote 留待澄清） | 门禁 `78a9f138c` |
| .worktrees/host.pid | 删除（死 pid 39019，无进程、无写入方在册） | — |
| .worktrees/auto-down | **不动**（活体符号链接 → 独立仓，015-notes npm_deps 引用） | — |
| auto/probe_lib_use.at + auto/scratch/probe_lib_use.at（未跟踪） | 删除 | 511 事后 ad-hoc 探针 |
| docs/specs/auto-lang/ui/design/diagram-components.md:4 | `../../../` → `../../../../`（spec-lint 唯一警告） | 502 写错层级 |

### F. 债务簿核销

- P499-7（cookbook tv 红）：510 worktree `7a8ac1d2e` 已偿还，条目内已注——
  划线核销。
- 转账新增：P513-1（407 R7+Phase 4）、P513-2（412 视觉通道）、P513-3
  （420 P4+挂账两项）、P513-4（worktree-cleanup 补丁核对结论，若未落地）。
- 其余 ~25 条"值得近期做"的清偿候选（P506-1 UAF、P499-1 timer 空转、
  测试红族 444×3/P487-2/P495-1/P496-2/P504-1/P507-2/P502-1 等）**不在
  本批**——归"债务批二期"另立，本计划在 KNOWN-DEBT 文末留一行指引。

## 详细设计

无架构内容。操作纪律：

1. 全程在 `.worktrees/plan-513-dev` 执行（虽然零代码改动，保持范式一致；
   且并行会话活跃，master 工作区有其未提交资产，隔离更安全）。
2. 归档统一 `git mv docs/plans/<f>.md docs/plans/archive/`；旧格式文件
   归档前在头部补一行终态注记（不套新 frontmatter，避免伪造范式历史）。
3. 每步验证 = grep/git 级断言（Category A 禁 cargo）。
4. 442 条件步骤：执行日 ≥2026-09-03 且无回滚证据才归档。

## 测试设计

- T1 归档完整性：`ls docs/plans/*.md | grep -E '^(405|407|412|416|420|441|455|458|462|400)-'`
  零命中；archive 对应文件头部终态注记存在。
- T2 残留转账：`grep -c 'P513-[1234]' docs/plans/KNOWN-DEBT-AND-RISKS.md` ≥ 3。
- T3 撞号消除：`git log --oneline --all --grep='plan-453'` 不再双义；
  `ls docs/plans/ | grep '^453-'` 零命中、`514-w5-*.md` 存在。
- T4 卫生回收：`du -sh scratch/` 前后对比 ≥120M 回收；`.worktrees/host.pid`
  不存在；`python scripts/spec-lint.py` 0 errors 0 warnings。
- T5 活件刷新：242/415/448/401/394/autos-desktop-program 各自刷新段落
  grep 命中（如 autos 台账 464 行含"已归档"）。
- T6 三处活链接（修 08-20 审计归档后）：overview.md:103/00-intro.md:170/
  handoff-2026-08-22.md:10 指向新位置或新审计段。

## 验收标准

1. T1-T6 全绿。
2. `git status` 中本计划引入的变更全部已提交；master 工作区其余脏文件
   （并行会话资产）原样未动。
3. KNOWN-DEBT 文末新增"债务批二期候选清单指引"一行。
4. 零 crates/ 变更（`git diff --stat -- crates/` 为空）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **A 组直接归档**：462/416/458/441/405（先回填 B1 状态行）/
   plans-status-audit-2026-08.md/plans-360-369-status-summary.md 七件
   `git mv` + 终态注记。验证：T1 对应段。
2. **B 组转账归档**：KNOWN-DEBT 先登记 P513-1/2/3，再 407/412/420/400/455
   五件归档（400/455 头部补 superseded 注记）。验证：T1+T2。
3. **08-20 审计归档 + 修链**：归档 plans-status-audit-2026-08-20.md；修
   docs/specs/overview.md:103、docs/design/00-intro.md:170、
   docs/handoff-2026-08-22.md:10 三处指向（改指本计划的审计结论段或
   archive 路径）。验证：T6。
4. **C 组活件刷新**：242/415/448/401/394 状态段刷新；autos-desktop-program.md
   台账五行回写；442 条件步骤（到期则归档）。验证：T5。
5. **撞号修复**：`bash scripts/new-plan.sh w5-ext-impl-lib-methods` 取 514，
   内容迁移 + 删旧 453-w5 文件。验证：T3。
6. **scratch/ 卫生回收**：按 E 表逐行执行（worktree-cleanup 补丁先核对再
   定 git rm 或转 P513-4）；`.worktrees/host.pid` 删除；auto/ 两探针删除。
   验证：T4 体积段。
7. **spec-lint 断链修复 + 债务簿收尾**：diagram-components.md 层级修正；
   P499-7 划线核销；KNOWN-DEBT 文末加债务批二期指引行。验证：T4 lint 段。
8. **收尾**：门禁复核（Category A：零 crates diff 断言 + 全部 T 复跑）；
   spec 沉淀归 merge；状态翻 execution_done → review。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① 442 观察期**：执行日若未到 2026-09-03，442 保持现状（留尾注）；
  到期后谁触发归档——本计划执行中到期则顺手，否则留 KNOWN-DEBT 指引行。
- **② schema_drift_audit.py promote**：scratch 副本是门禁脚本唯一副本，
  是否 `git mv` 到 scripts/ 正式化——默认不动（门禁在 cargo t 内已闭合），
  登记候裁定。
- **③ 015-notes 在途会话**：八文件未提交改动无计划簿记，建议（不强制）
  该会话补 515 簿记并顺手核销 P482-2/3——本计划仅提示不代劳。
- **④ worktree-cleanup-2026-08-23 补丁核对**：若三份打捞补丁有未落地内容，
  是否值得救（逐个评估工作量和时效，超本批范围则转 P513-4 留账）。
