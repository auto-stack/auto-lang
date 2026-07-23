# Plan + Spec 混合开发模型设计

> **Status**: 草案 v1（待评审）
> 日期：2026-07-23
> 范围：auto-lang monorepo 全仓库的知识体系与开发流程设计
> 前置阅读：本设计基于对 docs/plans/（59 活跃 + 296 old）、docs/specs/（旧 .ad 金字塔尝试）、docs/design/（105 篇）、docs/plan-indices/、docs/plan-reports/、superpowers 技能链与 forge-write-* 技能链的全面调研。

---

## 1. 背景与问题诊断

### 1.1 三种开发模式的教训

| 模式 | 实践 | 优点 | 暴露的问题 |
|---|---|---|---|
| 基于 chat | 日常小改动 | 灵活、快 | 无结构、无知识沉淀（docs/ai_history/ 归档试验 2026-01 后已废弃） |
| 基于 plan | docs/plans/ 370+ 编号计划，superpowers 四技能（brainstorm → write-plan → execute-plan → review） | 粒度合适、过程完整、可验证、有历史 | plan 之间无关联；对项目无结构化知识表达；状态追踪碎片化（Status 头/文末追加/索引表/归档位置四处可能矛盾） |
| 基于 spec | docs/specs/ 旧 .ad 金字塔（G/A/D/I/P/S/V/X 八类文档 + manifest.at） | 类型化 ID + depends_on 追溯理念好 | 文档类型过多 → 信息分散、流程链过长（10 环节、多 role agent 接力断链）；按类型单文件不扩展（designs.ad 877 行）；手工 manifest 与内容漂移（实证：reviews 标 empty 但有内容）；随 auto-forge 迁出整体成孤儿 |

### 1.2 旧 spec 尝试失败的三个根因

1. **按文档类型切分，而非按知识结构切分**。goals/architecture/designs/plans/tests/reviews/reports 七个文件是"流程产物"的分类，不是"项目知识"的分类。结果就是所有主题混排在大文件里，且只装下 forge-relay 一个特性。
2. **维护成本放在开发主链路里**。spec 写入被拆成 8 个 forge-write-* skill 由不同 role 接力，任何一环中断，spec 就烂尾。
3. **手工维护二级真相源**。manifest.at 的状态与时间戳靠手改，必然漂移。

### 1.3 现存可用资产（新体系应吸收而非重造）

- `docs/design/00–20` 编号章节：全仓库最系统、最新（2026-06/07 仍在更新）的主题知识库，主题域划分可直接作为 module 骨架。
- `docs/plan-reports/` 16 篇：361 个 plan 蒸馏出的主题叙事，是"过程记录 → 持久知识"的现成中间层。
- `docs/language/specification.md` + `spec-updates/`：语言规范与增量演进记录格式。
- `docs/conformance/`：唯一带"规范 → 对偶测试 → 实现"流程的目录，其条目格式可复用。
- `docs/specs/http-server-spec.md`：按特性单文件、含显式非目标的 spec 范式，是模块级 design 文档的模板基准。
- superpowers 四技能骨架：已被 370+ 个 plan 验证，整体沿用。
- 旧 .ad 体系的 ADR 模板（备选方案 pros/cons + Consequences）与 reviews/reports 表格模板。

---

## 2. 设计原则

1. **按知识结构组织，不按流程产物组织**。spec 树的第一级划分是 project（子项目），第二级是 module（模块），与代码结构同构——代码在哪，知识的归属就在哪。
2. **plan 管过程，spec 管沉淀，各司其职**。plan 是时间线上的完整叙事（动机、决策、步骤、验证），永不删改只归档；spec 是当前状态的切片（是什么、为什么、现状如何），持续重写。两者通过引用互相锚定，不互相复制内容。
3. **spec 写入集中在流程末端的一个环节**。开发主链路保持 superpowers 的短链条（brainstorm → plan → execute → review），spec 蒸馏是收尾的第 5 环，由**一个** skill 一次完成——不按 role 拆 agent，从根上消除接力断链。
4. **文档类型最少化**。每个 module 最多 4 种文档（overview / architecture / design / plans），另加 project 级 1 种（project.md）。没有明确读者的文档类型一律不设。
5. **索引可生成，不手维护**。一切派生数据（plan 索引、全局导航、状态汇总）由脚本从文件内容生成；人只维护一手内容。
6. **状态就近声明**。每个文档自己声明 Status（如 design 章节已有的 Implemented/Partial/Planned 惯例），状态查询读文件本身，不靠中央 manifest。

---

## 3. 概念模型

```
repo (auto-lang monorepo)
 └── project          子项目：一个可独立演进的交付单元
 │                      判定规则：一个 Cargo crate / 一个 npm package /
 │                      一个顶层资源目录（stdlib、website、blocks、parity…）
 │    └── module       模块：project 内的一个内聚功能单元
 │                      判定规则：对应 src/ 下的一个目录或一组强相关文件
 │           └── spec 文档（overview / architecture / design / plans）

时间轴（与上正交）：
 plan (docs/plans/NNN-slug.md)  —— 一次开发任务的完整过程记录
 plan 收尾时经 spec-sync 蒸馏进受影响 module 的 spec 文档
```

**命名说明**：第一级采用 `project`（而非 domain/area/component），理由：与用户既有词汇一致、与 crate/package 边界一一对应、无需额外学习。project 之上的分组（语言核心/工具链/UI 生态/外围）只作为 INDEX.md 里的视图分组，不设实体目录——避免再引入一层要维护的概念。

微型 project（auto-val、auto-atom、auto-bindgen、a2r-std 等基础库）允许只维护 `project.md` 一个文件，不强制建 module 层。

---

## 4. 目录架构

```
docs/specs/
├── README.md                  # 体系规约：概念、文档类型、流程、skill 使用说明
├── INDEX.md                   # 全局索引（脚本生成，人读）：project 分组视图 + 状态汇总
├── _archive/                  # 旧 .ad 金字塔等历史 spec 封存（只读，不再维护）
│
├── auto-lang/                 # 语言核心（crates/auto-lang，最大 project）
│   ├── project.md             # 项目卡：定位/目标/模块架构图（mermaid，节点即链接）
│   ├── lexer/
│   │   ├── overview.md        # 概述：职责、现状、关键代码入口、Owner 知识
│   │   ├── architecture.md    # 架构说明 + ADR 追加日志（ADR-01, ADR-02…）
│   │   ├── design/            # 主题设计文档（蒸馏产物，按 slug 命名，可多篇）
│   │   │   └── error-recovery.md
│   │   └── plans.md           # 相关 plan 索引表（spec-sync 维护，lint 校验）
│   ├── parser/ …
│   ├── types/ …               # infer/typeck/ownership
│   ├── vm/ …                  # AutoVM
│   ├── trans/ …               # a2c/a2r/a2ts/a2py 转译器（也可再拆子 module）
│   ├── ui/ …                  # AURA/ui_gen/a2ui
│   └── …
│
├── auto-cli/                  # crates/auto（主 CLI）
├── auto-man/                  # 构建器/包管理
├── auto-gen/                  # 代码生成器
├── auto-lsp/                  # LSP
├── auto-playground/           # crates/auto-playground + frontend
├── auto-val/                  # 微型 project：仅 project.md
├── auto-atom/                 # 微型 project：仅 project.md
├── stdlib/                    # Auto 标准库（module：auto/c/collections/aura/may/result）
├── widgets/                   # packages/widgets
├── forge-ui/ · lab-ui/ · playground-vue/
├── website/
├── blocks/                    # AutoUI blocks 生态
├── parity/                    # 移植对齐验证（独立 workspace）
└── aavm/                      # auto/ 自举编译器实验
```

**与 docs/design/ 的关系**：design/ 保留为"跨 project 的全局设计思考"与草稿池（raw/）；凡内容明确归属某 project/module 的，逐步迁移进 specs 树，原位置留重定向链接。迁移完成后，design/00–20 章中已被吸收的内容在 00-intro.md 标注去向。

### 4.1 文档类型定义（共 5 种）

| 文档 | 级别 | 内容 | 更新时机 | 篇幅纪律 |
|---|---|---|---|---|
| `project.md` | project | 定位一句话、状态、目标（链接 roadmap）、**模块架构图（mermaid，节点链接到 module 目录）**、模块清单表、对外接口 | 模块增删、目标变化时 | ≤150 行 |
| `overview.md` | module | 职责、当前实现状态、关键代码入口（文件:行级路径）、使用示例、已知坑 | 相关 plan 收尾时（spec-sync） | ≤100 行 |
| `architecture.md` | module | 架构说明（一张图）+ **ADR 追加日志**：ADR-NN、日期、决策、备选 pros/cons、后果、来源 plan | 有架构决策的 plan 收尾时 | ADR 只追加不改写（Superseded 标记） |
| `design/<slug>.md` | module | 某一主题的深度设计知识（机制、数据结构、算法、协议），模板基准为 specs/http-server-spec.md：范围/原则/细节/显式非目标 | spec-sync 蒸馏时新增或重写 | 单主题单文件，不限篇幅 |
| `plans.md` | module | 相关 plan 索引表：编号/标题/状态/归档位置/一句话沉淀 | spec-sync 追加；spec-lint 校验 | 纯表格，脚本可解析 |

**显式不设的类型**（及理由）：
- Goals/roadmap → 并入 project.md 一节；全局路线留在 docs/roadmap.md。
- Tests spec → 测试复杂模块（vm、trans、conformance）可在 design/ 内设 testing 主题文档，不单设类型；conformance/ 目录维持自治。
- Reviews/Reports → 属过程记录，留在 plan 文件内（实施结果追加节）与 plan-reports 迁移后的 design/ 文档，不进 spec 树。
- APIs spec → API 文档属于 design/ 或 overview 的一节；对外 API 参考文档属 website/tour，不属 specs。
- 全局 manifest → 由 spec-lint 脚本扫描生成 INDEX.md，不手工维护。

### 4.2 ID 与引用规则

- **plan 编号**：沿用 NNN 顺序制。新增**中央取号**：`docs/plans/.next-id` 文件 + `scripts/new-plan.sh`（或在 master 上由 write-plan skill 执行取号 commit），消除并发 worktree 撞号（336/337/338/342/351/355/359 重复事故的根因）。
- **ADR 编号**：模块内局部编号（ADR-01…），全局引用写作 `auto-lang/vm#adr-03`。废弃旧体系的全局 G/A/D/P/S/V/X 前缀——全局 ID 要求中央注册表，正是旧体系漂移之源。
- **design 文档**：语义 slug 命名，不用编号。
- **引用格式**：plan → spec 用 `see auto-lang/vm/design/async-scheduling.md`；spec → plan 用 `(plan-336)`。spec 文档中引用 plan 一律指向归档后的稳定路径 `docs/plans/`（活跃）或 `docs/plans/archive/`。

---

## 5. 混合开发流程（五环）

```
┌──────────────── 开发期：plan 模式（信息完整、粒度合适）────────────────┐
│                                                                      │
│  1 brainstorm ──→ 2 write-plan ──→ 3 execute-plan ──→ 4 review       │
│  (superpowers     (改造：编号制      (沿用：worktree    (沿用：code-    │
│   沿用，结论进     + frontmatter     隔离 + 分批执行)    reviewer agent)│
│   plan"已确认                          │                            │
│   决策"节)                             │                            │
└────────────────────────────────────────┼────────────────────────────┘
                                          ▼ plan 完成、合并
┌──────────────── 沉淀期：spec 模式（结构化、长期知识）──────────────────┐
│                                                                      │
│  5 spec-sync（新增，单一 skill 一次完成，不再拆 role）:                 │
│    a. 读 plan + diff + review 结论，识别受影响 project/module         │
│    b. 逐 module 蒸馏：                                                │
│       - 更新 overview.md（现状、入口变化）                             │
│       - 有架构决策 → architecture.md 追加 ADR-NN                      │
│       - 有机制性知识 → design/<slug>.md 新增/重写                     │
│       - plans.md 追加索引行                                           │
│    c. 归档：plan 移入 archive/，更新 plan-indices（过渡期）            │
│    d. 重生成 INDEX.md；在 plan 文末追加"spec-sync 回写记录"节          │
└──────────────────────────────────────────────────────────────────────┘
```

**关键约束**：

- 第 1–4 环**不写 spec**。开发期信息只进 plan，保证过程完整性和单一事实源。这保留了 plan 模式的核心优点。
- 第 5 环是**合并门禁的一部分**：plan 不算完成，直到 spec-sync 回写记录生成。小改动（typo 级）可由 skill 判定为"无沉淀价值"，仅更新 plans.md 一行。
- brainstorm 阶段允许（且鼓励）先读相关 module 的 spec 文档作为上下文——spec 树同时是新会话/agent 的**冷启动知识入口**，这是它相对 plan 考古的核心价值。

### 5.1 文档与环节的责任矩阵

| 环节 | 读 | 写 |
|---|---|---|
| brainstorm | specs 树（相关 module）、docs/design | plan 草案的"已确认决策"节 |
| write-plan | specs 树、plan 草案 | docs/plans/NNN-slug.md（声明影响面 projects/modules） |
| execute-plan | plan | 代码、测试、commit |
| review | plan + diff | review 结论（进 plan 或 commit） |
| spec-sync | plan + diff + review | specs 树、archive、plan-indices、INDEX |

---

## 6. Skill / Agent 设计

### 6.1 新增（3 个）

**`spec-sync`**（核心新增，合并现有 plan-archiver 职责）
- 触发：plan 完成合并后，显式调用（`/spec-sync 370`）或作为 execute-plan 的收尾步骤。
- 输入：plan 编号（从 plan 文件 frontmatter 读影响面声明）。
- 行为：§5 第 5 环的 a–d 全部步骤，一次会话内完成。
- 产出物：spec 文档修改 + plan 归档 + plan 文末回写记录节。
- 反断链设计：单 skill 单职责单会话；产出物清单固定（checklist 式），中途失败可从 checklist 续跑。

**`spec-init`**（迁移期 + 新模块用）
- 为指定 project/module 生成骨架文件（project.md / overview.md / architecture.md / plans.md 模板）。
- 带 `--from-design` 模式：从 docs/design 指定章节/文件蒸馏初稿（迁移期批量用）。
- 带 `--scan <crate-path>` 模式：扫描代码结构预填 overview 的模块清单和入口路径。

**`spec-lint`**（健康检查，先脚本后 CI）
- 检查项：plan 编号唯一性；plans.md 行与实际 plan 文件一致；spec 中引用的 plan/文件路径存在；module 缺 overview 报警；Status 字段超期（如 90 天未动的 overview 标 stale）。
- 产出：报告（不自动改），可挂 CI 或定期跑。

### 6.2 改造（1 个）

**`write-plan`**（基于 superpowers writing-plans 适配本仓库）
- 编号制：从 `.next-id` 取号（取代 superpowers 的 YYYY-MM-DD 前缀约定，该约定在本仓库从未实行）。
- plan frontmatter 增加影响面声明：
  ```yaml
  ---
  plan: 371
  title: vm-async-scheduling-fix
  affects: [auto-lang/vm]
  status: draft | in-progress | complete
  ---
  ```
- 保留原技能全部优点：bite-sized 步骤、精确路径与命令、TDD、频繁 commit。

### 6.3 沿用（3 个）

- `brainstorming`（superpowers）：唯一改动是产出设计结论时提示"先读 specs 树相关 module"。
- `executing-plans`（superpowers）：原样。
- `code-reviewer` agent（superpowers）：原样。

### 6.4 显式不做

- 不做 forge-write-* 式的 8 个分 role skill（旧体系失败根因）。
- 不做 spec 专用存储后端/API（jade 工具教训：先纯文件约定，跑顺了再考虑工具化）。
- skills 放置：仓库级 `.claude/skills/`（版本管理、随仓库演进），需要用户级共享时再同步到 `D:/autostack/skills/`。

---

## 7. 现有文档生态的处置

| 现有 | 处置 |
|---|---|
| docs/plans/（活跃 59 + archive + old 296） | 不动。新增 `.next-id` 取号文件。spec-sync 接管归档动作后，archive/ 为唯一归档目标（old/ 封存为历史，不再迁入新文件，消除 archive/old 双归档不一致） |
| docs/plan-indices/ 16 篇 | 过渡期由 spec-sync 继续更新；spec-lint 具备聚合能力后改为脚本从各 module plans.md 生成，最终退役手工维护 |
| docs/plan-reports/ 16 篇 | 停更。其中 Design 叙述章节作为 spec-init --from-design 的蒸馏素材迁入 module design/ |
| docs/design/00–20 章 | 主体迁移：章节内容按 module 拆入 specs/auto-lang/*/design/；00-intro.md 更新为指向 specs 树。跨模块全局内容保留在 design/ |
| docs/design/raw/ 71 篇 | 保持草稿池定位；被吸收的篇目在文首标注去向链接 |
| docs/specs/*.ad（旧金字塔） | 移入 docs/specs/_archive/forge-relay/，README 注明历史背景，不删 |
| docs/specs/http-server-spec.md | 迁入 specs/stdlib/design/http-server.md（或 specs/auto-lang/networking/，按归属判定），作为 design 模板范例 |
| docs/language/specification.md + spec-updates/ | 迁入 specs/auto-lang/（语言规范属核心 project 的顶层文档）；spec-updates 增量格式保留 |
| docs/conformance/ | 维持自治目录，INDEX 中链接 |
| docs/ 根级 26 个散文件 | 随迁移逐个归类：*-implementation.md / *-summary.md → 对应 module design/；transpiler 指南 → specs/auto-lang/trans/ 或 tour；重名 migration-guide 合并 |
| docs/ai_history/、analysis/、requirements/、implementation/、cli/、scratch/ | 封存/归并（多为已死目录），不迁入 specs |
| docs/roadmap.md | 保留全局定位；各 project.md 的目标节链接它 |

---

## 8. 落地路线图

- **Phase 0（本设计批准后）**：建 specs 骨架——README.md（本设计的操作版）、INDEX.md 生成脚本、全部 project 的 project.md（内容从本调研结论填充）、`.next-id` + new-plan 取号、旧 .ad 封存。验收：每个 project 有一张项目卡，mermaid 架构图可点击导航。
- **Phase 1**：spec-init/spec-sync/spec-lint 三个 skill 落地（先 markdown 规约 + 少量脚本）；auto-lang 核心 module（lexer/parser/types/vm/trans/ui）从 design/00–20 蒸馏建 module specs。验收：任意挑一个 module，新人/agent 只读 spec 树能回答"它是什么、怎么工作、最近发生了什么"。
- **Phase 2**：write-plan 改造上线（编号 + 影响面 frontmatter）；此后的 plan 全部走五环流程；plan-reports 停更、plan-indices 转生成。验收：连续 3 个 plan 完整走通五环，spec 树随之生长。
- **Phase 3**：存量清理（根级散文件归类、design/ 章节迁移收尾、死目录封存）；spec-lint 进 CI。验收：docs/ 无无主文档，lint 全绿。

每 Phase 独立可交付、可回滚；所有存量文档只移动+留链接，不删除。

---

## 9. 反模式清单（从本仓库历史提炼，skill 中应内置检查）

1. **手工维护中央 manifest**（旧 specs/manifest.at 漂移实证）→ 一切索引脚本生成。
2. **按流程产物分文档类型**（8 类 .ad）→ 按 project/module 知识结构组织。
3. **spec 写入拆多 role 接力**（forge-write-* 断链）→ spec-sync 单 skill 一次完成。
4. **并发取号无注册**（7 组重复 plan 编号）→ `.next-id` 中央取号。
5. **同构双索引**（plan-indices vs plan-reports 重叠且双双落后）→ plans.md 唯一事实源，其余皆生成物。
6. **spec 描述不存在的代码**（.ad 金字塔指向已删除的 auto-forge）→ spec-lint 校验引用路径存在；project 迁出时其 specs 目录一并归档。
7. **spec 与 plan 内容互相复制**（信息双写必漂移）→ 只引用、不复制；plan 存过程，spec 存现状。
8. **归档双轨**（archive/ 与 old/ 并存且规则不一）→ 唯一归档目标 + 历史封存。

---

## 附：与旧体系的取舍对照

| 旧体系元素 | 新体系取舍 |
|---|---|
| G/A/D/P/S/V/X 全局 ID + depends_on | 弃全局 ID，留模块内 ADR 编号 + 路径引用 |
| manifest.at 机器可读索引 | 改为脚本生成的 INDEX.md |
| 8 个 forge-write-* skill | 收敛为 spec-sync / spec-init / spec-lint 3 个 |
| ADR 模板（备选 pros/cons + Consequences） | 保留，作为 architecture.md 的 ADR 条目格式 |
| reviews/reports 表格模板 | 保留在 plan 生命周期内使用，不进 spec 树 |
| http-server-spec.md 单文件范式 | 提升为 design/ 文档的模板基准 |
| superpowers 四技能 | 整体沿用，仅 write-plan 适配编号制 |
