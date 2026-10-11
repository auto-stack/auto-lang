---
plan_id: PLAN-761
status: archived
feature_name: root-directory-cleanup
author: [Codex]
created_at: 2026-10-11
updated_at: 2026-10-11
plan_revision: 2
current_step: 3
total_steps: 3
supersedes_spec_components: [docs/specs/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/plans.md, docs/specs/auto-lang/trans/overview.md, docs/specs/auto-lang/trans/design/test-convention.md, docs/specs/stdlib/design/http-service-deployment.md]
new_spec_components: [docs/specs/design/repository-layout.md, docs/specs/plans.md]
touched_goals: [GOAL-016, GOAL-018]
completion_kind: delivered
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
| SD-04 | modify | docs/specs/auto-lang/{ui/overview.md,ui/plans.md,trans/overview.md,trans/design/test-convention.md}, docs/specs/stdlib/design/http-service-deployment.md | 实际工具/runner/部署路径 → 新位置；保持语义 | 当前入口可执行 | AC-03 |

## 6. 测试设计

资产哈希/清单对照、根项数和引用扫描；Python/Node/Shell/PowerShell 语法及工具入口无副作用检查；Plan446 实际加载（禁止 SKIP）；Auto 轻量 runner。Rust cargo check -p auto-lang + 裸 cargo t（夹具定位变化）+ cargo tv。未改 ui_gen/aavm/Schema/生成器，不跑 tu/taa 或独立 docs_gen。已知红逐项与基面/依赖比较，不把新增红记债算完成。

## 7. 验收标准

- AC-01：根 Git 树 30 项（18目录+12文件），19个散落入口及4被归并目录消失。
- AC-02：搬迁资产字节保持（必要路径/安全规则改动有解释），截图哈希一致；主检出本机配置副本和其他会话 WIP 保留。
- AC-03：夹具/runner/工具/运维新入口可用，无有效旧引用，无链接；ignore 不吞源码。
- AC-04：规范增量、账本、归档、复审/merge 收据完整；master 合入及 guard 清理完成。

## 8. 执行步骤

- [x] T-01（AC-01/02）：lang-761 worktree 清理/搬迁/ignore/共享技能；记录映射和哈希。
- [x] T-02（依赖T-01；AC-03）：修 Rust Plan446 定位、MJS/tools 目录计算、runner/部署说明/安全文档；语法与无副作用验证。
- [x] T-03（依赖T-02；AC-01/02/03）：check/t/tv、实际夹具加载、资产/根树复核；提交实现，同会话独立复审角色从工件重建证据并记录限制。
流程收尾 M-01（复审之后；AC-04）：canonical Specs/派生账本、线性合入、归档、guard 清理及批量回归到期判定；属于 merge 生命周期，最终在合入收据核对，不计实现任务。

## 9. 复审记录

- stage: new；PLAN-761:r1；outcome: pass；next: work。报告范围已获用户执行/合并授权，无待审批决策。

## 10. 待澄清事项

无。Node 锁统一为条件性选项，本次保持三个根入口。账本接口可达性实施中核查；不可达时准确保留待办，不伪造成功。


### work 进展（2026-10-11）

- worktree D:/autostack/.wt/lang-761/auto-lang，branch plan-761-dev，base b25729f5a；只读兄弟 auto-down@895f8d0。实现提交 28ddc73b1。
- T-01/T-02：64 资产搬迁；44 原字节、20 路径/安全说明调整；Git tree 30 项；截图/风险台账保持字节一致。映射 docs/reports/root-layout-761.json。Python/Node/Bash/PowerShell 语法、5 MCP --check、Vue bps 和代理 fixture 路径解析、Tailwind 再生8861类金样相等、fstr dry-run/截图差分工具 smoke、native-fixture cargo check 均通过。
- auto debug CLI 使用新 test/auto/field_access.at，7/7通过（0.051s）。cargo check -p auto-lang通过，406存量告警；native-fixture 1存量 unnecessary unsafe 告警。未新增 Rust声明/逻辑，仅 fixture 路径及注释变化。
- 首轮 cargo t：CARGO_TARGET_DIR 环境设置被嵌套oracle构建继承，5测产物落共享目录导致预期原路径缺失；已移除环境覆盖，改外层 cargo --config build.target-dir 后 --no-fail-fast 重跑，属于验证环境纠正。
- 账本：现有825条六段旧版兼容JSON（无project/version/section_type），隔离 musk store GET /api/specs 返回500。未发送写入请求，未更改现有账本；已停自建隔离服务。依用户提供AGENTS §4及本仓docs/specs/README §5的手工回退/兼容upsert要求执行最小投影，保留旧schema和全部无关条目，禁止为本计划迁整个账本。此repo约定覆盖通用技能的store-only格式假设，合入收据记录逐项完整性验证。

- r2 合同澄清：将原T-04从实现任务中拆为后置M-01合入生命周期，避免复审/合入循环；目标、验收、全部收尾要求保持，沿用户原授权执行；补齐SD-04实际canonical路径触面。
- stage: work | PLAN-761:r2 | outcome: pass | code_commit: a48cd5b5336d7bcc4f33c7e4f08328c558acf88c | T-01/02/03 complete | next: review。
- 最终日常档：5243 run，5242pass/1known flake/1541skipped，107.061s；Plan484 isolated两次1/1pass。tv 162/162pass，2.135s。Plan446 explicit ignored跑出Init歧义（非SKIP）；原b25729导出的22文件重放同错误，证实历史语料兼容问题；7用例Auto字段测试通过。
- 额外CLI runner尝试暴露filter只过滤报告，不能作为通过证据；VM整族30s超时停止，未改变其runner/语料语义。P761-D1/D2在worktree的KNOWN-DEBT追加；没有将这些非目标问题改成计划验收门槛。

### review（r2）

- stage: review | outcome: pass | reviewed_commit: 62eb5435ecf3e2c5675cb9dd47557a419b621232 | base: ef778ae31 | auto-down:895f8d0f9355c9f5ec3ce8fca268bdb768395846 | spec_delta_sha256:55B3531451AC18FF7655A53B8FB4639840A7707812C9F50F43F18F28F2D1FC4A | next: merge。
- 同会话独立复核角色，直接重算Git对象与路径引用，限制明示。AC01/03和SD01..04 pass；AC02主检出本机副本恢复、AC04发布/归档后置项由merge复核。无阻断发现；P761-D1/D2在KNOWN-DEBT登记。持久证据：docs/plans/reports/761-root-directory-review.md。
- canonical delta与代码一致，44/20对象比对及两组guard clean。56fa096c2为仅添加复审报告的descendant，未改变代码/规范内容。

### merge prepared（r2）

- prepared: reviewed=62eb5435ecf3e2c5675cb9dd47557a419b621232（等价02fb7e4c1）；delivery=a1d7531616f045d6fad200ce4a4bf67e78d682eb；canonical SD-01..04保持复审SHA256不变。后继仅复审报告/派生账本。账本六段825旧项及原字节全部保留，仅追加4项/68行；829项验证，投影收据docs/reports/761-ledger-projection.json。
- 目标项：designs/repo-layout、tests/repo-layout-verification、reviews/PLAN-761-review、reports/PLAN-761-deposit；来源版本/哈希见投影收据。隔离store旧schema加载500未写入，依本仓README §5兼容回退，无无关schema迁移。
- 主检出已有759复审报告/台账WIP及其他会话本地临时输出，均保留；仅重叠台账暂存后逐字节补丁回放。本机六配置合入后恢复并核对SHA256。

### merge 收据（PLAN-761:r2）

- stage: merge | outcome: pass（批量回归到期移交后置）| delivery_commit:24affbb4a9a16857ae9303543498999558042216。
- landed: master由75cc18531 fast-forward到delivery，实际HEAD相等；无merge commit。最终rebase range-diff四项均“=”：a0aad3da0→b6247cc0c，02fb7e4c1→9b515b581，35f77e41c→1b693117e，a1d753161→24affbb4a。前序28ddc73b1→27791bb71→a0aad3da0、a48cd5b53→62eb5435e→02fb7e4c1，均已验证patch等价。canonical delta SHA256保持55B3531451AC18FF7655A53B8FB4639840A7707812C9F50F43F18F28F2D1FC4A。
- ledger_refreshed: 主检出.autoos/specs.json，829条；825旧项及原字节保留，仅追加4项；目标ID/源commit/文件SHA256和账本before/after见docs/reports/761-ledger-projection.json。read-back与全部来源SHA256一致；依本仓README §5执行兼容投影，不转换旧schema。
- AC-01/02：master Git树30项；六份本机设置由scratch/root-cleanup-761恢复，SHA256均等于合入前；759报告文件原字节不变；KNOWN-DEBT旧WIP补丁正文相等，临时stash已安全恢复并删除。其他会话本地输出未删除，临时忽略仅影响Git展示。
- AC-03：64资产映射、blueprints/代理/当前引用检查均通过；未改生产逻辑，仅测试夹具定位与代码注释。
- archived: docs/plans/archive/761-root-directory-cleanup.md，completion_kind=delivered，状态终态archived；canonical Spec及review/projection指向archive一致。
- cleaned: 2026-10-11，归档commit=d42992167；验证branch全祖先已落地，两worktree各自fresh wt-guard clean后移除；plan-761-dev已删除，lang-761空组已删除；git worktree list / show-ref / 文件系统三重回读确认无残留。
- 生产产物观察：release auto.exe修改时间为2026-10-10T18:44:06.418489+08:00；本计划未改变运行时逻辑，现有release不需为资产搬迁重启。debug门禁在共享target复编；未触auto-down代码、前端生成器/发布bundle。本次未执行部署或远程push。
- 批量回归：上一收据覆盖755；760整五计划此后落地，故DUE。按auto-plan-regress主检出单实例tf/tt/tb，结果与最终收据另记；当前无cargo/nextest重型进程。
