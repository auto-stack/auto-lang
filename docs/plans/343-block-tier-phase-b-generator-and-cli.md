# Plan 343: Block 层 Phase B — `auto block` CLI(agent-driven 生成)+ 静态 acceptance check

> **类型**:完整计划(实施)
> **状态**:设计待确认,实施未开始
> **日期**:2026-06-26(2026-06-26 按"agent-driven"架构重定范围)
> **战略文档**:[docs/design/17-blocks-first-class.md](../design/17-blocks-first-class.md)(Skill 模型,§2.3/§2.4/§9)
> **前置**:[Plan 342](342-block-tier-phase-a-package-foundation.md)(block 包格式 + `BlockRegistry` + 首批包)
> **架构定调(重定范围)**:`auto` 二进制**无** LLM/HTTP 基建(无 reqwest / API key / agent crate)。故 AI 生成**不在 `auto` 内部**;按 Design 17 的 Skill 模型,**生成器 = agent**(本会话 / auto-musk),`auto` 只负责"供给 spec + 校验产物"。
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

## 1. 目标(agent-driven 重定范围)

让 block 可被 **agent 消费**:agent 读 spec、写 `.at`、用 `auto build` 校验、修复。`auto` 二进制提供这四个**无 LLM 依赖**的使能命令:

1. **`auto block list`** —— 按 kind 列出 block 目录(来自 `BlockRegistry`)。
2. **`auto block show <kind>/<name>`** —— 打印 spec(frontmatter + NL 正文)+ variants + gotchas 摘要。**这是 agent 读的"skill 接口"**。
3. **`auto block add <kind>/<name> [--reference <variant>] [--out <dir>]`** —— 拷一份 reference `.at` 到消费者;报告 palette 依赖 + dataSource 接入点 + gotchas 摘要(adopt-and-edit 路径)。
4. **`auto block check <file>.at`** —— 静态 acceptance 检查:EDIT 区标记齐全、palette 内 widget 被用、loading/error 契约槽位在。**这是 agent 生成回路的校验器**(配合 `auto build`)。

**AI 生成本身**:agent 驱动 —— `auto block show` 取 spec → agent 写 `.at` → `auto build` + `auto block check` 校验 → 失败回灌修复。**不在 `auto` 内调 LLM**。文档化为工作流(§Phase 3)。

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **AI 生成层** | agent 驱动,**不**在 `auto` 二进制内置 LLM | auto 无 LLM/HTTP 基建;且属 agent 层职责(Design 17 Skill 模型) |
| **`auto` 的角色** | 供给 spec(`show`)+ 校验产物(`check` + `auto build`) | 编译器本分;不越界做生成 |
| **add 模式** | 只 `--reference <variant>` 拷贝(原 `--from` 改为 agent 工作流,非 CLI flag) | 避免 CLI 内 LLM;adopt-and-edit 是开箱路径 |
| **check 形态** | 静态、无 LLM:扫 `.at` 文本,核对 EDIT 区 / palette / loading-error 槽 | 给 agent 修复回路一个快速、确定性的门禁 |
| **block 目录来源** | `BlockRegistry::with_defaults()`(读仓库 `blocks/`) | 342 已落地;分发(随 npm/包)是后续 |

## 3. 非目标(留给后续)

- 命名 slot / 运行时 block 复用 → Phase D(AURA 扩展)。
- 全量 block 目录 → Phase C(基准驱动)。
- `dataSource` 类型系统 → Design 16 Rung 2。
- `auto` 内置 LLM 通道 → 如未来确需,另立基础设施 Plan(不属编译器本分)。
- block 目录的分发(随包发布给外部消费者)→ 后续。

---

## 实施计划

> **Repo rules (CLAUDE.md):** 在专用 worktree 开发;改 CLI 后 `cargo build -p auto`。
>
> **Goal:** `auto block` 的四个使能命令(list / show / add --reference / check)+ agent-driven 生成工作流文档 + dataSource 约定。**无 LLM 进 `auto` 二进制**。
>
> **Architecture:** `BlockRegistry`(342)→ `auto block` CLI(供给 spec + 校验产物)。AI 生成 = agent:`auto block show` 取 spec → agent 写 `.at` → `auto build` + `auto block check` 校验 → 修复。

## Pre-flight: Worktree

```bash
git worktree add -b plan-343/block-phase-b ../auto-lang-343
cd ../auto-lang-343
```

---

## Phase 1 — `auto block list` + `show`

### Task 1.1: clap 枚举加 `Block` 命令

**Files:** `crates/auto/src/main.rs`(`Commands::Block { action: BlockAction }`、`BlockAction::{List, Show, Add, Check}`)、`crates/auto/src/cmd_block.rs`(新)。

**Step 1:** `BlockAction::List`(按 kind 分组打印 `kind/name`,来自 `BlockRegistry`)。
**Step 2:** `auto block list` 手测 → 打印 `data-display/note-list`、`form/login`。
**Commit:** `feat(cli): 'auto block list'`。

### Task 1.2: `auto block show <kind>/<name>`

**Files:** `cmd_block.rs`

**Step 1:** 打印 spec 完整内容(frontmatter + NL 正文)+ variants 列表 + gotchas 全文。这是 **agent 读的 skill 接口**。
**Step 2:** 手测:`auto block show form/login` → 完整 spec + 2 variant + gotchas。
**Commit:** `feat(cli): 'auto block show' prints spec + variants + gotchas (agent skill interface)`。

---

## Phase 2 — `auto block add --reference` + `check`

### Task 2.1: `auto block add [--reference <variant>]`

**Files:** `cmd_block.rs`

**Step 1:** `add <kind>/<name> [--reference <variant>] [--out <dir>]`:从包拷 `reference/<variant>.at`(缺省取首个 variant)到 `<out>/<name>.at`(默认 `src/front/blocks/`)。
**Step 2:** 拷后报告:palette 依赖、`dataSource` 接入点(spec 签名)、gotchas 摘要(前几条标题)。
**Step 3:** 手测:`auto block add form/login --reference minimal --out tmp/blk` → 出现 `tmp/blk/login.at` + 终端报告。
**Commit:** `feat(cli): 'auto block add --reference' copies a variant + reports deps/gotchas`。

### Task 2.2: `auto block check <file>.at`

**Files:** `cmd_block.rs`(静态 acceptance 检查)

**Step 1:** `check <path>` 扫 `.at` 文本,核对(可机检项):
  - 每个 `extension_point`(若 `--spec` 给定)都有对应 `// EDIT: <point>` 标记(未指定 spec 时跳过此项);
  - 出现 loading / error 状态变量或分支(契约槽位);
  - 用到的 widget 标签在 palette 内(若 `--spec` 给定)。
**Step 2:** 输出清单:每项 ✓/✗ + 退出码(有 ✗ → 非 0,供 agent 回路判定)。
**Step 3:** 单测:对 `blocks/form/login/reference/minimal.at` 跑 check 应全过(它是 known-good)。
**Commit:** `feat(cli): 'auto block check' static acceptance gate for the agent repair loop`。

---

## Phase 3 — agent-driven 生成工作流文档

### Task 3.1: 工作流文档

**Files:** `docs/design/blocks/agent-generation-workflow.md`

**内容:** agent 如何用 Phase 1-2 的命令生成一个定制 block:
1. `auto block show <kind>/<name>` 取 spec(skill 输入);
2. agent 按 spec + 消费者 intent + 项目上下文(可用 widgets、已有 `#[api]` 签名)写 `.at`;
3. `auto block check <file> --spec <kind>/<name>` + `auto build` 校验;
4. 失败 → 把错误/未满足项回灌 agent → 重写(N 轮);
5. 过 → 落到消费者 `src/front/blocks/<name>.at`(owned)。
明确:**生成在 agent,不在 `auto`**;`auto` 只供给 + 校验。
**Commit:** `docs(blocks): agent-driven generation workflow`。

---

## Phase 4 — dataSource 接入约定 + README

### Task 4.1: dataSource 约定文档

**Files:** `docs/design/blocks/datasource-convention.md`

**内容:** spec 如何声明 `dataSource` 签名;消费者如何把真实 `#[api]` fn 绑到该槽;类型化契约检查指向 Design 16 Rung 2。
**Commit:** `docs(blocks): dataSource wiring convention`。

### Task 4.2: README / 端到端

**Files:** `examples/blocks-gallery/README.md`(或 `blocks/README.md`)

**内容:** 两种消费路径(reference 拷贝 / agent 生成)的端到端步骤;eject 后怎么办;如何新增 block 包(指向 Plan 342 格式规范)。
**Commit:** `docs(blocks): end-to-end consumption + authoring workflow`。

---

## Definition of Done

- [ ] `auto block list` 按 kind 分组打印。
- [ ] `auto block show <kind>/<name>` 打印 spec + variants + gotchas(agent skill 接口)。
- [ ] `auto block add [--reference <variant>] [--out]`:拷贝 + 报告 palette/dataSource/gotchas。
- [ ] `auto block check <file>.at [--spec …]`:静态 acceptance 门(EDIT 区 / loading-error 槽 / palette),known-good reference 全过,退出码反映结果。
- [ ] agent-driven 生成工作流文档 + dataSource 约定 + 端到端 README 就绪。
- [ ] worktree 分支在 `cargo build -p auto` + 相关测试绿后合并回 `master`。

> **显式不做**(原 Plan 的 `--from` CLI 内 AI 生成、spec→reference nightly eval harness):AI 生成由 agent 驱动(Phase 3 文档),不在 `auto` 内调 LLM。如未来要 CLI 内生成,另立基础设施 Plan。

---

## 后续(不在本 Plan)

- **Phase C(目录覆盖)**:由 Design 16 M1-M3 基准驱动,补齐每类 block。可散入各基准 Plan,或单列"block 目录扩充"Plan。
- **Phase D(命名 slot / 运行时复用)**:依赖 AURA 扩展,远期。
- **CLI 内 LLM 通道**(若未来确需):另立基础设施 Plan。
