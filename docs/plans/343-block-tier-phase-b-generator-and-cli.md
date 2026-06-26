# Plan 343: Block 层 Phase B — AI 生成器 + `auto block add` CLI + spec/reference CI

> **类型**:完整计划(实施)
> **状态**:设计待确认,实施未开始
> **日期**:2026-06-26
> **战略文档**:[docs/design/17-blocks-first-class.md](../design/17-blocks-first-class.md)(Skill 模型,§2.3/§2.4/§9)
> **前置**:[Plan 342](342-block-tier-phase-a-package-foundation.md)(block 包格式 + `BlockRegistry` + 首批包)
> **软依赖**:[Design 16](../design/16-app-generation-and-ai-authoring.md) Rung 5 的 `auto dev` 热重载回路(若未就绪,回退到 `auto build` 校验)
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

## 1. 目标

让 block 真正可用:消费者能从 block 包**得到一份定制 `.at`**——要么拷参考实现,要么让 **AI 照 spec 生成**。并建立 **spec ↔ reference 一致性 CI**(reference 既是默认产物,也是 spec 的回归 fixture)。具体:

1. **`auto block` 子命令**:`list`、`add`(两模式:`--reference <variant>` 拷贝;`--from "<自然语言需求>"` AI 生成)。
2. **AI 生成器**:读 spec + 消费者意图 → 生成 `.at` → `auto build` 校验 → 失败则修复回路(N 轮)→ 写入消费者项目。复用 `/auto-lang-creator` skill 基建。
3. **spec CI 回归**:对每个 block 包,"仅凭 spec 让 AI 复现 reference"的回归 harness(确认 spec 足够让 AI 生成等价物;不能 → spec 缺东西)。
4. **`dataSource` 接入约定**:文档 + 生成器在输出里标注接入点。

完成后:block 层从"可读的包目录"升级为"可消费、可生成、有质量门"的一等公民。

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **AI 调用形态** | 走 `/auto-lang-creator` skill 约定(读 spec 当指令 + intent),不在 codegen 里硬编码 LLM | 复用既有 skill 基建;spec 本就是"经库策展的 skill 文件"(Design 17 §2.3) |
| **校验回路** | 生成 → `auto build`(+ vue-tsc)→ 失败把错误回灌 AI 修复,最多 N 轮 | outcome 评判(Design 17 §2.2);N 是 Design 16 的评测度量 |
| **`auto dev` 依赖** | 软依赖;未就绪则用 `auto build` 一次性校验 | Rung 5 未落地不该卡 Phase B |
| **生成产物归属** | 写到消费者 `src/front/blocks/<name>.at`,owned(可改可 eject) | Design 17 §2.2:拥有的是输出 |
| **spec CI 形态** | 先"手动/nightly eval"(AI-in-CI 重);脚本对每包跑"spec→AI→对照 acceptance" | 避免每 PR 跑 LLM;够用作回归信号 |
| **reference 模式** | 直接拷 `reference/<variant>.at` + 报告 palette 依赖 + 标注 dataSource 接入 | 开箱即用(Design 17 §2.3 adopt 路径) |

## 3. 非目标(留给后续)

- 命名 slot / 运行时 block 复用 → Phase D(AURA 扩展)。
- 全量 block 目录 → Phase C(基准驱动)。
- `dataSource` 类型系统 → Design 16 Rung 2。
- AI 生成的确定性种子 → 先不解决(Design 17 §12 开放问题)。

---

## 实施计划

> **Repo rules (CLAUDE.md):** 在专用 worktree 开发;改 CLI 后 `cargo build -p auto`。
>
> **Goal:** `auto block add`(reference + AI 两模式)+ AI 生成器(校验修复回路)+ spec/reference CI 回归 + dataSource 接入约定。
>
> **Architecture:** `BlockRegistry`(342)→ `auto block` CLI →(reference 模式:拷;AI 模式:调 `/auto-lang-creator` + spec/intent → 生成 → `auto build` 校验 N 轮)→ owned `.at` 于消费者。

## Pre-flight: Worktree

```bash
git worktree add -b plan-343/block-phase-b ../auto-lang-343
cd ../auto-lang-343
```

---

## Phase 1 — `auto block list` + reference 模式 add

### Task 1.1: clap 枚举加 `Block` 命令

**Files:** `crates/auto/src/main.rs`(`Commands::Block { action: BlockAction }`、`BlockAction::{List, Add{…}}`)、`crates/auto/src/cmd_block.rs`(新)。

**Step 1:** `BlockAction::List`(打印按 kind 分组的 block,来自 `BlockRegistry`)。
**Step 2:** `auto block list` 手测 → 打印 form/login、data-display/note-list。
**Commit:** `feat(cli): 'auto block list'`。

### Task 1.2: `auto block add --reference <variant>`

**Files:** `cmd_block.rs`

**Step 1:** `add <kind>/<name> [--reference <variant>] [--out <dir>]`:从包拷 `reference/<variant>.at` 到 `<out>/<name>.at`(默认 `src/front/blocks/`)。
**Step 2:** 拷后报告:palette 依赖(需 `@auto-ui/widgets` 里哪些 widget)+ `dataSource` 接入点(spec 里的签名)+ gotchas 摘要(前几条提醒)。
**Step 3:** 手测:`auto block add form/login --reference minimal` → 项目里出现 `src/front/blocks/login.at` + 终端打印依赖/接入/gotchas。
**Commit:** `feat(cli): 'auto block add --reference' copies a variant + reports deps/gotchas`。

---

## Phase 2 — AI 生成模式

### Task 2.1: 生成器核心(spec + intent → .at,带校验修复回路)

**Files:** `crates/auto/src/cmd_block.rs`(或 `crates/auto/src/block_gen.rs` 新模块)

**Step 1:** `add <kind>/<name> --from "<NL intent>"`:组装 prompt = spec(frontmatter + NL 正文 + palette + extension_points + acceptance + gotchas)+ 消费者 intent + 项目上下文(可用 widgets、已有的 back.api 签名)。
**Step 2:** 调生成(经 `/auto-lang-creator` skill 约定;具体 LLM 接入走既有 agent 通道)→ 得 `.at`。
**Step 3:** 校验回路:把 `.at` 写临时文件 → `auto build`(单包)→ 失败则把编译错误回灌、再生成,最多 N 轮(默认 3);成功才写入消费者目录。
**Step 4:** 输出仍是 owned `.at`,带 EDIT 区标注 + dataSource 接入提示。
**Commit:** `feat(cli): 'auto block add --from' AI-generates a block from spec + intent (build-repair loop)`。

### Task 2.2: acceptance 校验

**Files:** `cmd_block.rs` / `block_gen.rs`

**Step 1:** 生成产物除"能编译"外,按 spec 的 `acceptance` 清单做静态检查(暴露的扩展点 EDIT 区存在、用了 palette 内 widget、有 loading/error 槽等可机检项)。
**Step 2:** 未过 → 进修复回路(把未满足的 acceptance 回灌)。
**Commit:** `feat(block): acceptance-check generated blocks before writing`。

---

## Phase 3 — spec ↔ reference CI 回归

### Task 3.1: "spec→AI→reference"回归 harness

**Files:** `tools/block-spec-eval/`(脚本)+ 文档

**Step 1:** 脚本:遍历 `blocks/`,对每个包:仅给 spec(不给 reference)让 AI 生成各 variant → 对照 acceptance + `auto build` → 与 reference 做"结构等价"比对(扩展点齐全、palette 一致、关键状态机在)。
**Step 2:** 输出报告:每包每 variant "AI 能否复现等价物"。失败 = spec 需补(沉淀为 gotcha 或改 spec)。
**Step 3:** 定位为**手动/nightly**(不在每 PR 跑,避免 LLM 成本);README 记录怎么跑、怎么读结果。
**Commit:** `tools(blocks): spec→reference regeneration eval harness (nightly)`。

### Task 3.2: 守卫测试补充

**Files:** `crates/auto-lang/src/ui_gen/block/`(测试)

**Step 1:** 单测(每 PR 跑,无 LLM):每个 reference 仍可编译;frontmatter 合法;palette 项在 WidgetRegistry;variants ↔ reference 文件一一对应(342 已有,此处补齐边界用例)。
**Commit:** `test(blocks): strengthen non-LLM package invariants`。

---

## Phase 4 — dataSource 接入约定 + 文档

### Task 4.1: dataSource 约定文档

**Files:** `docs/design/blocks/datasource-convention.md`

**内容:** block 如何在 spec 声明 dataSource 签名(参数/返回类型,引用 `#[api]` 类型);消费者 `auto block add` 后如何把一个真实 `#[api]` fn 绑到该槽;类型化契约如何检查(指向 Design 16 Rung 2)。
**Commit:** `docs(blocks): dataSource wiring convention`。

### Task 4.2: README / 端到端流程

**Files:** `examples/blocks-gallery/README.md`(或 blocks/ README)

**内容:** 两种消费路径(reference 拷贝 / AI 生成)的端到端步骤;何时用哪个;eject 后怎么办;如何新增 block 包(指向 Plan 342 的格式规范)。
**Commit:** `docs(blocks): end-to-end consumption + authoring workflow`。

---

## Definition of Done

- [ ] `auto block list` 按 kind 分组打印。
- [ ] `auto block add --reference <variant>`:拷贝 + 报告 palette 依赖 / dataSource 接入 / gotchas。
- [ ] `auto block add --from "<NL>"`:AI 照 spec+intent 生成,`auto build` 校验 + 失败修复回路(N 轮),acceptance 静态检查,产物 owned + 带标注。
- [ ] spec→reference 回归 harness(nightly)就绪,跑通首批两个包并产出报告。
- [ ] 非-LLM 守卫测试(reference 编译 / frontmatter / palette / variant 一致)在每 PR 跑。
- [ ] dataSource 接入约定文档 + 端到端 README 就绪。
- [ ] worktree 分支在 build + 测试绿后合并回 `master`。

---

## 后续(不在本 Plan)

- **Phase C(目录覆盖)**:由 Design 16 M1-M3 基准驱动,补齐每类 block(随基准需要而加;不预先穷举)。可散入各基准 Plan,或单列"block 目录扩充"Plan。
- **Phase D(命名 slot / 运行时复用)**:依赖 AURA 扩展,远期。让 block 可作"活组件"(非生成/非拷贝)运行时组合。
