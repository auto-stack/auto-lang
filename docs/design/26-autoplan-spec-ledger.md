# 26 — AutoPlan Spec Ledger：开发范式与知识账本 v2

> **状态**：现行（2026-08-28，Plan 467 确立）
> **取代**：[plan-spec-hybrid-model.md](plan-spec-hybrid-model.md)（v1，2026-07-23；其 §1 诊断与 §9 反模式清单被本文继承）
> **适用**：auto-lang monorepo 全部开发任务
> **操作规约**：[docs/specs/README.md](../specs/README.md)（v2）——本文讲"为什么"，规约讲"怎么做"

---

## 1. 背景与诊断

### 1.1 v1 为什么停摆

v1（plan-spec-hybrid-model）设计了五环流程（brainstorm → write-plan → execute-plan →
review → **spec-sync**），specs 树按 project/module 组织。诊断其实践结果：

- **spec-sync 是纯手工环节**，依赖执行者收尾时"记得回写"——Plan 437 之后再无一次完整回写，
  specs 树停更于 2026-08 初（不覆盖 437–466 共 30 个计划带来的 AutoUI 桌面线大变迁）。
- **没有门禁形态的状态机**：plan 状态是散文式标注（draft/in-progress/complete 混用或缺失），
  归档与否、复审与否无从机器判定。
- 同期 auto-os 生态仓演化出 **auto-plan 四技能范式**（new/work/review/merge +
  `.autoos/specs.json` 账本 + musk 后端 `/api/plans/{seq}/merge` 自动沉淀），并已在本仓
  实际使用（Plan 446/466 走完全程并沉淀账本条目 P446-\*/P466-\*）。
- 结果：**两套流程并存**——`.worktree/plan-NNN`（旧）与 `.worktrees/plan-NNN-dev`（新）
  同期存在，specs v1 树与 specs.json 账本互不知晓。

### 1.2 决策（2026-08-28，用户裁定）

**全面收敛到 auto-plan 四技能范式**；specs v1 的 project/module 知识树**保留**（它是对的——
按知识结构组织优于按流程产物组织，这正是 forge .ad 金字塔失败的教训），作为 6 段账本的
人类可读投影；v1 的 spec-sync 手工环节由 merge 技能的账本 upsert + 本仓规约的回写扩展取代。

## 2. 概念模型

### 2.1 三层资产

```
docs/design/   设计层：意图与方案（"为什么/打算怎么做"）——含 raw/ 历史素材
docs/specs/    账本层：现状知识（"现在什么样"）——project/module 树 + 6 段 ledger 投影
docs/plans/    过程层：一次任务的完整叙事（"怎么做的"）——active + archive/（终态）
.autoos/specs.json  机器账本：merge 技能的 upsert 目标（musk 后端可用时经 API 写入）
```

### 2.2 Plan 状态机（唯一合法迁移）

```
drafting → executing → execution_done → reviewed → archived
   new        work          work           review      merge（终态，不可逆）
```

- 每个状态恰由一个技能负责推进（单一职责）；跳步 = 流程 bug。
- `archived` 的 plan 位于 `docs/plans/archive/`，不再改动（复审补充除外）。
- 历史计划（≤466）大量无 frontmatter，**不回溯改造**；新计划一律带 v2 frontmatter
  （`plan_id/status/feature_name/current_step/total_steps` + review 预留字段）。

### 2.3 六段账本（6-section ledger）

账本的知识维度沿袭 forge 的 Spec 分类思想（见 [forge/spec-categories.md](forge/spec-categories.md)）：

| 段 | 机器存储 | 人类投影（docs/specs/） | 沉淀时机 |
|---|---|---|---|
| goals | specs.json `goals` | `goals.md`（全局 GOAL-NNN）+ 各 `project.md` 目标节 | merge |
| architecture | specs.json `architecture` | `<module>/architecture.md`（ADR 追加日志） | merge |
| designs | specs.json `designs` | `<module>/design/<slug>.md` | merge |
| tests | specs.json `tests` | `<module>/overview.md` 测试节 / `design/testing-*.md` | merge |
| reviews | specs.json `reviews` | plan 文件 `## 复审记录`（过程记录，不复制） | review→merge |
| reports | specs.json `reports` | plan 文件变更摘要 + `plans.md` 行 | merge |

条目溯源契约：每条账目带 `file`（来源 plan 路径）+ `related: [PLAN-NNN]`；
markdown 侧以 `(plan-NNN)` 行内引用锚定。**只引用，不复制**（v1 反模式 #7 继承）。

## 3. 目录架构（specs v2）

```
docs/specs/
├── README.md       # v2 规约：概念、6 段映射、四技能本仓操作手册、路径映射表
├── INDEX.md        # 脚本生成（scripts/spec-index.py，勿手改）
├── overview.md     # ★ 全局总览：仓库地图/模块状态/活跃线/孤儿清单——/auto-plan:new Step 2 的取材源
├── goals.md        # ★ 全局目标账本：GOAL-NNN——/auto-plan:review 填 touched_goals 的引用对象
├── _archive/       # 历史 spec 封存（只读）
└── <project>/      # v1 树全保留：project.md + <module>/{overview,architecture,design/,plans}.md
```

## 4. 四技能 ↔ 本仓路径映射表

技能文档（用户级 `~/.zcode/skills/auto-plan-*/SKILL.md`，为 auto-os 仓书写）中的硬编码
路径在本仓的对应关系——**执行技能时以本表为准**：

| 技能文档写的 | 本仓实际 | 说明 |
|---|---|---|
| `docs/plans/NNN-slug.md` | 相同 | ✅ 一致 |
| `docs/plans/archived/` | **`docs/plans/archive/`** | 不改名：仓内 68 处引用 + archive-plan 技能 + auto-musk 跨仓引用均用 archive/ |
| `.worktrees/plan-<NNN>-dev` | 相同 | ✅ 一致（AGENTS.md 旧 `.worktree/plan-<NNN>` 已废弃） |
| `.autoos/specs.json` | 相同 | ✅ 一致（project=auto-lang） |
| `docs/designs/008-auto-plan.md` | 本文档（Design 26） | 范式设计源的本仓对应 |
| musk 后端 `127.0.0.1:8080` | 通常不可用 | 走技能的手工回退路径 + §5 扩展程序 |

## 5. 沉淀程序（/auto-plan:merge 的本仓扩展）

技能本体流程（gate → fold worktree → specs.json upsert → archive → verify）之上，
本仓规约追加两步（详见 [specs/README.md](../specs/README.md) §4）：

1. **module 回写**：按 plan frontmatter 的 `affects`（或 review 填写的
   `new_spec_components`/`supersedes_spec_components`），把一句话级现状变化写进对应
   `<module>/overview.md`（或追加 ADR / design 文档），并在 `plans.md` 追加索引行。
   typo 级改动可只写 plans.md 行。
2. **索引再生**：`python scripts/spec-index.py` 重生成 INDEX.md。

门禁不变：merge 只收 `reviewed` 的 plan；archived 终态不可逆。

## 6. 取舍记录

| 议题 | 决策 | 理由 / 代价 |
|---|---|---|
| archive/ 是否改名 archived/（对齐技能文档） | **不改** | 改名波及 68 文件 + 跨仓引用 + archive-plan 技能；用 §4 映射表让智能体适配零成本 |
| specs.json 与 markdown 树双轨 | **保留双轨，分工明确** | specs.json=机器账本（技能 upsert、幂等 P\<seq\>-n）；markdown=人类视图（module 树）。双写漂移风险以"markdown 只存蒸馏结论、明细留在 plan"约束 |
| 438 个归档 plan 是否回溯入账 | **不回溯** | 历史知识已蒸馏在 design 00–24 章、plan-reports/、KNOWN-DEBT；账本向前生长。overview.md 提供现状总览兜底 |
| v1 spec-sync / spec-init 技能 | **废弃**（spec-init 的建卡职能保留为手工惯例） | 接力断链根因；merge 扩展程序取而代之 |
| AGENTS.md 的 L0/L1/L2 分诊与测试门禁分级 | **保留不动** | 与范式正交且已被 466 收敛为 cargo t/tf 体系 |

## 7. 反模式清单（v1 §9 全部继承，新增 3 条）

1–8. 见 [plan-spec-hybrid-model.md](plan-spec-hybrid-model.md) §9（手工中央 manifest、
按流程产物分文档、多 role 接力、并发取号、同构双索引、描述不存在的代码、内容互相复制、归档双轨）。

9. **状态机跳步**：drafting 未经确认就 executing、execution_done 未经复审就 archived——
   每个门禁存在的意义就是拦截上一步的惰性收敛。
10. **账本双写不一致**：merge 只写 specs.json 不回写 module 树（或反之）——按 §5 程序两处都写。
11. **plan 里复制 spec 全文**：plan 只写增量与决策，现状描述链接 specs（继承 #7 的具体化）。

## 8. 落地清单（Plan 467 执行）

- [x] design 树整理（编号冲突消解、strategy/ 归位、索引重写）→ 00-intro.md
- [x] specs v2 三全局件（README v2 / overview.md / goals.md）
- [x] AGENTS.md + scripts/new-plan.sh 对齐（worktree 路径、frontmatter、archive 映射）
- [x] INDEX 覆盖补全（4 张新项目卡 + 分组更新）
- [ ] （后续计划）按新范式走完首个完整特性循环并按 §5 回写——本计划自身即首个样板
