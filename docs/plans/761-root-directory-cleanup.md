---
plan_id: PLAN-761
status: drafting
feature_name: root-directory-cleanup
author: [Codex]
created_at: 2026-10-11
updated_at: 2026-10-11
plan_revision: 1
current_step: 0
total_steps: 4
supersedes_spec_components: [docs/specs/overview.md]
new_spec_components: [docs/specs/design/repository-layout.md, docs/specs/plans.md]
touched_goals: [GOAL-016, GOAL-018]
affects: [repository-layout]
---

# [PLAN-761] 根目录精简与资产归位

## 0. 变更摘要

按已批准的根目录检查报告移除误跟踪临时件/本机设置，归位文档和脚本，将 tests/tmp-corpus/tools/deploy 合入现有 test/scripts，根 Git 树由 53 项减为 30 项。

## 1. 目标

缩短 GitHub 根文件列表，让 README 更快出现；保留全部真实测试入口、工具、部署模板与验收证据。非目标：核心模块拆仓、aavm 退役、Node 包管理政策变更、清除其他会话的本地输出、真实部署/强杀进程。

## 2. 架构方案

保留源码/发布根入口；测试统一 test，辅助工具统一 scripts。本机配置从 Git 退出但主检出本地副本保留。复现探针进入 scripts/repro，历史证据进入 docs/plans/evidence。禁止 worktree junction/symlink。

## 3. 技术栈

Git worktree、Python/PowerShell 路径与资产核查、Rust check/nextest、Auto CLI 夹具加载验证。

## 4. 需求分析与背景调查

- 授权：用户 2026-10-11 明确“OK，按照你的方式合并”，批准报告内目录精简、worktree 实施、验证及本地 master 合入；不包含远程部署或清理其他会话证据。
- 基面：auto-lang dde29cf2cdb68d5851aa61c3eb536a04d5aa1fae；依赖 auto-down 895f8d0f9355c9f5ec3ce8fca268bdb768395846。
- 来源：docs/reports/root-directory-audit-20261011.md；docs/specs/overview.md；docs/specs/README.md §4；docs/specs/auto-cli/project.md；docs/specs/auto-lang/ui/overview.md。
- 已有主检出 WIP 仅 docs/plans/KNOWN-DEBT-AND-RISKS.md、docs/plans/reports/759-post-landing-review-20261011.md；不得覆盖或并入本计划提交。
- Plan446 fixture 缺失会跳过，必须确认加载新路径；Plan600 PNG 有历史验收引用，必须原字节保存。

## 5. 详细设计

1. 删除六个 .tmp-p730/p734 修改器、check_error.txt、donut_sfc.txt、uncompleted_plans.md、空 .autoos/specs.json.tmp2；flow.at 移 docs/design/examples/flow.at。
2. 取消跟踪 .agent/.claude/.vscode/.clangd，保留主检出副本；共享 commit 技能按本仓纪律重写到 .agents/skills/commit/SKILL.md。
3. DEBTS.md → docs/plans/DEBTS.md（不混写在途 KNOWN-DEBT）；SAFETY.md → docs/guides/filesystem-safety.md 并修过期规则；PNG → docs/plans/evidence/600；两个运维脚本 → scripts。
4. tests 四 runner → test/runners，tests/auto → test/auto，vue-gen-gap → test/ui/vue-gen-gap；tmp-corpus 夹具 → test/fixtures/plan446，MJS → scripts/repro/p446；tools → scripts/tools；deploy → scripts/deploy。
5. 更新执行入口、相对目录计算、规范/当前文档引用和 Plan600 证据指向；历史计划不全量改写。账本兼容 upsert，不删除无关历史。
6. 根 Node 三件保留。新增根锚定临时件忽略，构建 debug 忽略收紧到根，避免吞源码。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/overview.md | test/tests、tools/deploy 分散 → test/scripts 单入口 | 地图与代码一致 | AC-01,AC-03 |
| SD-02 | add | docs/specs/design/repository-layout.md | 已交付路径、本地配置、证据保存规则 | 持久维护约束 | AC-02,AC-03,AC-04 |
| SD-03 | add | docs/specs/plans.md | 根级布局索引 | 可追溯 | AC-04 |

## 6. 测试设计

资产哈希/清单对照、根项数和引用扫描；Python/Node/Shell/PowerShell 语法及工具入口无副作用检查；Plan446 实际加载（禁止 SKIP）；Auto 轻量 runner。Rust cargo check -p auto-lang + 裸 cargo t（夹具定位变化）+ cargo tv。未改 ui_gen/aavm/Schema/生成器，不跑 tu/taa 或独立 docs_gen。已知红逐项与基面/依赖比较，不把新增红记债算完成。

## 7. 验收标准

- AC-01：根 Git 树 30 项（18目录+12文件），19个散落入口及4被归并目录消失。
- AC-02：搬迁资产字节保持（必要路径/安全规则改动有解释），截图哈希一致；主检出本机配置副本和其他会话 WIP 保留。
- AC-03：夹具/runner/工具/运维新入口可用，无有效旧引用，无链接；ignore 不吞源码。
- AC-04：规范增量、账本、归档、复审/merge 收据完整；master 合入及 guard 清理完成。

## 8. 执行步骤

- [ ] T-01（AC-01/02）：lang-761 worktree 清理/搬迁/ignore/共享技能；记录映射和哈希。
- [ ] T-02（依赖T-01；AC-03）：修 Rust Plan446 定位、MJS/tools 目录计算、runner/部署说明/安全文档；语法与无副作用验证。
- [ ] T-03（依赖T-02；AC-01/02/03）：check/t/tv、实际夹具加载、资产/根树复核；提交实现，同会话独立复审角色从工件重建证据并记录限制。
- [ ] T-04（依赖T-03；AC-04）：canonical Specs/索引/派生账本、线性合入、归档、guard 清理及批量回归到期判定。

## 9. 复审记录

- stage: new；PLAN-761:r1；outcome: pass；next: work。报告范围已获用户执行/合并授权，无待审批决策。

## 10. 待澄清事项

无。Node 锁统一为条件性选项，本次保持三个根入口。账本接口可达性实施中核查；不可达时准确保留待办，不伪造成功。
