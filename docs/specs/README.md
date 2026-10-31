# Auto-Lang Specs 体系规约

> 版本：v3 文档单源（2026-10-31，auto-musk PLAN-107；保留 PLAN-543 知识分层）
> 设计文档：[docs/design/autoplan-spec-ledger.md](../design/autoplan-spec-ledger.md)（现行范式）
> 知识生命周期：[Design 27](../design/27-knowledge-base-lifecycle.md)（canonical-source 与同步模型）
> 历史版本：v1 见 [docs/design/plan-spec-hybrid-model.md](../design/plan-spec-hybrid-model.md)（已被取代，
> 其文档类型定义与反模式清单被本版继承）
> 本文档是**操作规约**：目录怎么组织、文档怎么写、流程怎么走。改动本规约需先改设计文档。

---

## 1. 概念

```
project   子项目 = 一个 Cargo crate / 一个 npm package / 一个顶层资源目录
module    模块   = project 内一个内聚功能单元（对应 src/ 下一个目录或一组强相关文件）
plan      一次开发任务的过程记录（docs/plans/NNN-slug.md），存"过程"，状态机管理
spec      本目录下的文档，存"现状与知识"，持续重写
adr       已接受的重要决策、理由与后果；接受后只追加 superseding 决策
catalog   从 Git 跟踪文档与代码 metadata 生成的结构/引用索引，不手工维护
ledger    goals/architecture/designs/tests/reviews/reports 六个知识视角；
          canonical 形态 = 本目录 Git 跟踪的 Markdown current-state，
          机器分类/关系/历史 = 应用从文档与归档 Plan 派生；旧 JSON 仅作迁移输入
```

原则：**Design/RFC 管 target-state，ADR 管原因，Spec 管 current-state，Plan 管过程，
Generated Catalog 管代码事实；只引用，不复制；状态机推进，门禁不跳步。**

`.autoos/specs.json` 当前被 Git ignore 且覆盖依赖本地执行，因此不是共享 canonical source。
PLAN-107 起停止 agent/merge 写入；原记录已逐条保全，当前知识在 Markdown 中维护，应用从文档与归档 Plan 单向生成索引。

## 2. 目录结构

```
docs/specs/
├── README.md            # 本规约
├── INDEX.md             # 全局索引（scripts/spec-index.py 生成，勿手改）
├── overview.md          # ★ 全局总览：仓库地图/模块状态/活跃线/孤儿清单
│                        #   （/auto-plan:new Step 2 "读 spec overview" 的取材源）
├── goals.md             # ★ 全局目标账本 GOAL-NNN（review 填 touched_goals 的引用对象）
├── _archive/            # 历史 spec 封存（只读）
└── <project>/
    ├── project.md       # 项目卡（必有）
    └── <module>/        # 微型 project 可无 module 层
        ├── overview.md      # 模块概述（必有）
        ├── architecture.md  # 架构图 + ADR 追加日志（有架构内容才建）
        ├── design/          # 主题设计文档（按需，slug 命名）
        └── plans.md         # 可选的旧 plan 索引表（保留既有内容，不要求 merge 回写）
```

## 3. 文档类型与模板

v1 的五种文档类型（project.md / overview.md / architecture.md / design\<slug\>.md / plans.md）
**全部保留**，模板见 v1 设计文档 §4.1（或任一现存卡为范例）。要点重申：

- `project.md`：≤150 行，mermaid 模块图（节点可点击下钻）+ 模块清单表。
- `overview.md`：≤100 行，职责/现状/关键入口（`文件:符号`）/已知坑。
- `architecture.md`：一张图 + **ADR 追加日志**（只追加不改写，格式：日期/来源 plan/决策/备选
  pros-cons/后果/状态）。
- `design/<slug>.md`：单主题单文件，范围/原则/细节/**显式非目标**（模板基准
  `stdlib/design/http-server.md` 风格）。
- `plans.md`：纯表格 `| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`，脚本可解析。

新增两个全局件：

- `overview.md`（全局，≤150 行）：仓库地图 + auto-lang 九模块状态行 + 活跃开发线 +
  孤儿/废弃清单 + 入口链接。**每次 merge 后如有结构性变化须更新**。
- `goals.md`（全局）：`GOAL-NNN：一句话 | 状态 | 关联` 条目，源自 roadmap.md 与战略
  design/plan。plan review 的 `touched_goals` 填 `GOAL-NNN: <一行>`。

## 4. 开发流程：auto-plan 四技能范式（v2 核心）

```
/auto-plan:new ─→ /auto-plan:work ─→ /auto-plan:review ─→ /auto-plan:merge
   drafting         executing          execution_done        reviewed → archived
   读 overview.md    worktree 隔离       逐项验收审查           canonical 文档落地核验 + 归档
   取材背景         plan 作执行合同      遗漏/延后/workaround   current spec/ADR → landing → archive
```

- **new**：`scripts/new-plan.sh <slug>` 中央取号（`.next-id`）；drafting 等用户确认。
- **work**：一切实现修改在 `D:/autostack/.wt/lang-<NNN>/auto-lang` sibling-group worktree
  （一 plan 一 worktree 全生命周期，禁止 junction/symlink）；
  `[✅]` 簿记写主检出的 plan 文件；scoped 测试；多阶段计划按阶段 fold + re-sync；
  测试遵守当前 AGENTS.md 的固定职责组与必要集成组；merge 在复审后的冻结基线执行受影响并集，周期批次沿当前单实例与到期规则，不因知识迁移加全局测试门。
- **review**：逐项核验验收（不信勾选框），只跑必要反例/受影响模块，不触发周期全集；填
  `supersedes_spec_components` / `new_spec_components` / `touched_goals`。
- **merge**：gate（必须 reviewed）→ fold worktree → current-state Spec/ADR 蒸馏 →
  核验真实 landing 与 canonical 源 → 归档。索引由应用派生，旧 JSON 不写；Plan 正文不复制进 current-state Spec。

**merge 本仓扩展程序**（技能文档之外，本规约追加）：

1. **module 回写**：按 plan 的 `affects`/review 元数据，把一句话级现状变化写进对应
   `<module>/overview.md`（或 architecture.md 追加 ADR / design 新增），并在该 module
   文档中保留相关 Plan 出处。无需为了历史清单或配额强制追加 plans.md。
2. **全局件维护**：结构性变化（新模块/新 crate/状态翻转）时更新 `overview.md`；
   目标推进时更新 `goals.md` 对应条目状态。
3. **派生索引**：由应用读取文档生成；已有 scripts/spec-index.py 可作可选维护工具，运行与否不参与交付门。
4. 在归档 Plan 中记录已落地 canonical 路径/哈希与提交、清理和迁移对账结果。

## 5. 路径映射表（四技能硬编码路径 → 本仓）

执行用户级 auto-plan-\* 技能时，**以本表为准**（技能为 auto-os 仓书写）：

| 技能文档写的 | 本仓实际 |
|---|---|
| `docs/plans/NNN-slug.md` | 相同 |
| `docs/plans/archived/` | **`docs/plans/archive/`** |
| `.worktrees/plan-<NNN>-dev` | `D:/autostack/.wt/lang-<NNN>/auto-lang`（Plan 529 sibling-group） |
| `.autoos/specs.json` | 相同 |
| `docs/designs/008-auto-plan.md` | `docs/design/autoplan-spec-ledger.md` |
| musk 后端 `127.0.0.1:8080` | 应用不可用不阻断文档交付；按 §4 验证 Git 源，不回填 JSON |

## 6. 六段账本映射

| 知识视角 | 应用派生分类（不由 agent 写 JSON） | markdown canonical source |
|---|---|---|
| goals | `goals` | `goals.md` + project.md 目标节 |
| architecture | `architecture` | `<module>/architecture.md`（ADR） |
| designs | `designs` | `<module>/design/<slug>.md` |
| tests | `tests` | overview.md 测试节 / `design/testing-*.md` |
| reviews | `reviews` | plan 文件 `## 复审记录`（只引用不复制） |
| reports | `reports` | 归档 Plan 变更摘要；`plans.md` 为可选历史导航 |

条目溯源：文档保留规范路径及 Plan 出处；旧 file/related/状态全字段见历史保全，不重新生成中央账本。
发生分歧时以 Git 跟踪的 current-state Spec/ADR 和可执行代码/测试为准。

## 7. 编号与引用

- plan 编号：`docs/plans/.next-id` 中央取号（`scripts/new-plan.sh`），禁止自行估算。
- GOAL 编号：`goals.md` 内 `GOAL-NNN` 顺序编号。
- ADR：模块内局部编号，全局引用 `auto-lang/vm#adr-03`。
- 引用格式：spec → plan 写 `(plan-318)`，指向归档稳定路径；plan → spec 写相对路径。
- 禁止全局 G/A/D/P/S/V/X 式前缀（旧 .ad 体系教训）。

## 8. 工具

| 工具 | 作用 |
|---|---|
| `scripts/new-plan.sh <slug>` | 原子取号 + v2 frontmatter plan 骨架（master 上运行） |
| `scripts/spec-index.py` | 扫描本树生成 INDEX.md |
| `scripts/spec-lint.py` | 健康检查：断链、编号冲突、缺卡、stale |

## 9. 反模式

v1 §9 八条（手工中央 manifest / 按流程产物分文档 / 多 role 接力 / 并发自行取号 /
同构双索引 / 描述不存在的代码 / 内容互相复制 / 归档双轨）+ Design 26 新增三条
（状态机跳步 / 双事实源 / plan 复制 spec 全文）——全文见
[autoplan-spec-ledger.md §7](../design/autoplan-spec-ledger.md)。Design 27 进一步禁止把本地
`.autoos/specs.json` 当作共享真相，以及在 overview 中长期手工维护可生成的数量。

## 10. 旧账本迁移与回滚

[PLAN-107 对账](history/legacy-ledger/README.md)保全每条旧记录及来源；原 JSON 不改写不删除。当前知识仍以本目录文档为准，历史验证数字不自动批准为现状，别名副本不重复导入。临时恢复材料按证据政策保留，长期撤销使用已审阅的 Git 反向提交并保护后续用户修改。
