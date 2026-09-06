# AutoLang Agent Guidelines & Workflows

## Standard (Design + Plan + Worktree) Workflow

All AI coding assistants working in this repository must strictly adhere to the following workflow principles based on task complexity.

> **开发范式（2026-08-28 起，Plan 467）**：标准流程采用 **auto-plan 四技能范式**
> （`/auto-plan:new` → `/auto-plan:work` → `/auto-plan:review` → `/auto-plan:merge`），
> 范式设计与规约见 [docs/design/autoplan-spec-ledger.md](docs/design/autoplan-spec-ledger.md)
> 与 [docs/specs/README.md](docs/specs/README.md) §4。**路径映射**（四技能为 auto-os 仓书写，
> 在本仓执行时以此为准）：技能文档中的 `docs/plans/archived/` 在本仓为 **`docs/plans/archive/`**；
> 技能文档中的 `.worktrees/plan-<NNN>-dev` 在本仓为 **`D:/autostack/.wt/lang-<NNN>/auto-lang`**
> （Plan 529 分组平铺布局；在途的 525/526 仍用旧 `.worktrees/`，fold 后退役）；
> `.autoos/specs.json` 两仓一致。
> Plan 状态机：`drafting → executing → execution_done → reviewed → archived`（终态）。
>
> **Worktree 红线（2026-09-03 三仓 .git 删除事故，Plan 529）**：worktree 内**禁止创建任何
> junction/symlink**——`git worktree remove` 的递归删除会穿透链接删除目标仓内容（实测复现）。
> 跨仓依赖用解析序解决：`$AUTO_LANG_ROOT 等 env 覆盖 → 组内 ../auto-lang →
> D:/autostack/auto-lang 主检出`。移除任何 worktree 前必须先跑
> `bash D:/autostack/wt-guard.sh <worktree 路径>`（reparse point 扫描，非空即拒）。

---

### 1. Task Sizing & Triage

- **L0: Trivial Fixes (轻微修改)**
  - *Criteria*: Minor typos, comment updates, or 1-2 line simple bugfixes without side-effects.
  - *Action*: Directly modify on the current branch, run verification tests, and commit with a clean commit message.
- **L1: Feature / Module Tasks (模块/特性任务)**
  - *Criteria*: Any new feature, multi-file change, complex bugfix, or cross-backend parity implementation.
  - *Action*:
    1. Run `scripts/new-plan.sh <slug>` on the default checkout (master) to atomically take the next `<NNN>` plan ID from `docs/plans/.next-id` and create the plan skeleton (v2 frontmatter).
    2. Fill `docs/plans/<NNN>-<plan-name>.md` (needs-analysis seeded from [docs/specs/overview.md](docs/specs/overview.md)) detailing goals, design, task checklist, and verification plan; present for confirmation before executing.
    3. Create a dedicated worktree in the sibling-group layout (Plan 529): `git worktree add D:/autostack/.wt/lang-<NNN>/auto-lang -b plan-<NNN>-dev`. Cross-repo plans add sibling worktrees into the same group dir (e.g. `.wt/down-047/{auto-down, auto-lang}`) so `../auto-lang` resolves uniformly.
    4. Perform all code implementation and testing inside that worktree; plan-file bookkeeping (`[✅]` markers, frontmatter flips) stays on the default checkout.
- **L2: Architectural Overhaul (重大架构级任务)**
  - *Criteria*: Changes impacting overall architecture, compiler/VM pipelines, core protocol definitions, or cross-system runtime contracts.
  - *Action*:
    1. First create/update a formal architecture design document in `docs/design/<NN>-<topic>.md` (register it in `docs/design/00-intro.md`).
    2. Decompose the design into 1 to N moderate-sized, independently executable Plan documents in `docs/plans/<NNN>-*.md`.
    3. Execute each Plan sequentially in its own dedicated worktree (one worktree per plan for its whole lifetime).

---

### 2. Execution Discipline in Worktree

- Always perform code modifications, builds, and test runs within the plan's worktree (`D:/autostack/.wt/lang-<NNN>/auto-lang`; legacy in-flight plans keep their `.worktrees/` path until folded) to keep `master` clean (one worktree per plan per repo; multi-phase plans fold per phase and re-sync).
- **Never place junctions/symlinks inside a worktree** (see red line above); resolve cross-repo deps via the documented order (env → group sibling → main checkout), never via links.
- **Fast Iteration during Development**:
  - During development, **DO NOT run full test suites repeatedly**. Use fast syntax/type checks:
    - Fast type check: `cargo check -p auto-lang`
    - Scoped single-module test (optional): `cargo t <module_name>` (e.g. `cargo t iced`)
  - Reuse standardized automation scripts from `.agents/skills/autoui-verifier/scripts/` (`test_vm_mcp.py`, `test_vue_playwright.mjs`) instead of writing ad-hoc scripts.
- **Change-Scoped Verification Gate (按改动范围分级测试门禁)**:
  - **Category A: Pure Verification / Asset / Doc-Only Tasks (纯验证/示例资产/计划跟踪)**:
    - 若未修改 `crates/` 下的 Rust 源码（仅截图、更新 `docs/plans/` 矩阵或修改测试脚本），**严禁运行 `cargo t` 和 `docs_gen`**。仅完成目标验证即可直接合入。
  - **Category B: Scoped Rust Code Changes (局部 Rust 模块改动)**:
    - 快速语法/类型检查：`cargo check -p auto-lang`
    - 局部模块验证：`cargo t <module_name>`（如 `cargo t iced` 或 `cargo t ui`）
    - 涉及编译器/VM/核心协议重构时，才在最终合入前运行一次 `cargo tf`（full 档，含 1M churn；Plan 466）。
    - **AAVM 专项（Plan 568）**：改 VM/编译器 → `cargo tv`（纯 .at 语料 golden，**不含 aavm**——aavm 无实用面，不需要关心是否被改坏，守护=CI+`ta`）；只有 diff 触及 aavm 代码（`auto/lib/*.at`、`test/vm/aavm2/**`、`parity/**`、aavm2 测试基建）才跑 `cargo taa`，且按 §AAVM/AA2R Test Tier 作用域映射缩小范围，不全量跑。
  - **Category C: Docs / Schema Changes (文档与元数据改动)**:
    - **仅当**修改了文档生成器、Schema 定义文件或语法参考时，才运行 `cargo test -p auto-lang --test docs_gen`。
  - **AutoUI 跨端验证（双端模式）**:
    - Vue 模式：`auto run`
    - VM 模式：`auto run -r vm`
    - 自动化双端一致性：调用 `autoui-verifier` 技能 (`.agents/skills/autoui-verifier`)。

#### AAVM/AA2R Test Tier (Plan 568)

**概念**："改 VM/编译器后的回归"（`cargo tv`，纯 .at 语料 golden）与"AAVM 自举展示"（`cargo taa`）是两个独立概念。AAVM（auto/lib/*.at 自举 + a2r.at 发射对齐）目前无实用面，平时改 VM/编译器**不需要**测 aavm（守护=CI `vm-files-ci.yml` push/PR + `cargo ta` 全量档 + fold 前裸 `taa`）。

**触发条件（只有这些路径的改动才跑 `taa`）**：`auto/lib/*.at`、`test/vm/aavm2/**`、`parity/**`、`crates/auto-lang/src/tests/aavm2_*.rs` / `aavm_runner_tests.rs`、`lib.rs` 的 `aavm2_lib_source`/`AUTO_LIB_FILES*`。其余改动零触发。

**作用域映射（改什么 → 跑哪个闸门；耗时为 2026-09-05 实测）**：

| 改动位置 | 跑什么 | 耗时 |
|---|---|---|
| `corpus_m1/**` | `cargo taa aavm2_m1` | 31s |
| `corpus_m2/**` | `cargo taa aavm2_m2` | 172s |
| `corpus_m3/**` | `cargo taa aavm2_m3` | 47s |
| `corpus_m4/**` | `cargo taa aavm2_m4` | ~315s |
| `corpus_use/**` | `cargo taa aavm2_m4 aavm2_m5` | ~127s |
| `corpus_a2r/**` | `cargo taa aavm2_a2r` | ~185s |
| `auto/lib/engine.at`（终段执行器） | `cargo taa aavm2_m5` | ~350s |
| `auto/lib/a2r.at`（终段发射器） | `cargo taa aavm2_a2r aavm_at_mode` | ~260s |
| `auto/lib/{token,lexer,parser,typeinfo,codegen}.at`（上游共享） | 全管线级联，直接裸 `cargo taa` | ~10min 量级 |

review/fold 前无论改了什么 aavm 文件，一律裸 `cargo taa` 全量兜底。闸门粒度=测试目录级（单语料文件由所属闸门整体覆盖）。compile 腿（`compile_corpus`/`compile_use_corpus`）现场 cargo build 产物按内容 hash 缓存，二次运行秒级。

**全档资源表**（测试数/耗时/内存；"待实测"由 Plan 568 T7 回填）：

| 档位 | 适用场景 | 测试数 | 实测耗时 | 内存 |
|---|---|---|---|---|
| `cargo t` | 日常快速回归（1M churn 排除） | 4304 | ~46s（Plan 507） | 轻池 <50MB/测 |
| `cargo tf` | review/折叠前全量门禁（含 1M churn） | 3441 | 77.2s（Plan 564） | ≤2GB 预算 |
| `cargo tv` | 改 VM/编译器后——纯 .at 语料 golden（**不含 aavm**，Plan 568） | ~3579 | 待实测 | 同日常档 |
| `cargo tt` | 改 transpiler 后 | ~373（非 ignore） | 待实测 | 未测 |
| `cargo tb` | 改 book/文档后（每测 5-7s） | 69 | 待实测 | 未测 |
| `cargo taa` | **仅** aavm 改动后（触发条件/作用域见上） | 21 + 日常面 | 待实测 | 重测试已知 0.8-1.2GB/个 |
| `cargo ta` | 终极全量（VM+aavm+trans+book+1M churn） | ~4400 | 待实测 | 同 tf 预算 |
| `cargo th` | 改 HTTP 服务后（真 TCP，串行） | 20 | 待实测 | 未测 |

#### Cargo Test Aliases Reference (from `.cargo/config.toml`)
- `cargo t`  - Fast daily tests (~3200 unit tests via nextest in parallel; 1M churn tier excluded, Plan 466)
- `cargo tf` - Full-scale daily tests (all tests incl. 1M churn tier) — the review / pre-fold full-suite gate (Plan 466)
- `cargo tv` - VM file tests (`--features test-vm-files`)——纯 .at 语料 golden，**不含 aavm**（Plan 568；aavm 在 `taa` 档）
- `cargo tt` - Transpiler tests (`--features test-trans`)
- `cargo tb` - Book listing tests (`--features test-book`)
- `cargo taa` - AAVM/AA2R self-hosting tier (`--features test-aavm`, implies vm-files)——**仅 aavm 改动后使用**，裸跑=全集兜底、追加滤串缩小作用域（如 `cargo taa aavm2_m5`）；触发条件/作用域映射/资源表见 §AAVM/AA2R Test Tier
- `cargo ta` - All test suites combined (`--features test-aavm,test-trans,test-book`; full scale)


---

### 3. Mandatory Independent Review Gate (独立复审)

Before merging or archiving, the agent **must explicitly execute an independent review step** (`/auto-plan:review` paradigm — verify, don't trust):
1. **Checklist Audit**: Re-verify every acceptance criterion in `docs/plans/<NNN>-*.md` against the actual code/diff — a checked box is a claim, not evidence.
2. **遗漏/延后/Workaround Scan**: Hunt explicitly for dropped sub-items, unapproved deferrals, and workaround patches; record findings as debt candidates in `docs/plans/KNOWN-DEBT-AND-RISKS.md`.
3. **Health Check**: Ensure zero unhandled compiler warnings, clean formatting, and no stray debug print statements.
4. Fill the spec-impact metadata (`supersedes_spec_components` / `new_spec_components` / `touched_goals`) so merge knows what to deposit.

---

### 4. Plan Archiving & Status Tracking

1. Update `docs/plans/<NNN>-<plan-name>.md` with the review record (复审记录) and completion summary; flip `status: reviewed`.
2. Deposit knowledge per `/auto-plan:merge` + [docs/specs/README.md](docs/specs/README.md) §4 extension (`.autoos/specs.json` upsert + module overview/ADR/plans.md 回写 + `python scripts/spec-index.py`).
3. Archive the plan (terminal state — archived plans do not go back):
   ```bash
   git mv docs/plans/<NNN>-<plan-name>.md docs/plans/archive/
   ```
   then set `status: archived` in its frontmatter. **Note:** the user-level auto-plan skills write `docs/plans/archived/` — in THIS repo the archive directory is `docs/plans/archive/`.
4. `docs/plans/.next-id` is bumped atomically by `scripts/new-plan.sh` at plan creation time.

---

### 5. Worktree Merge & Cleanup

1. Switch to `master`, merge the `plan-<NNN>-dev` branch (or cherry-pick/fast-forward) with Conventional Commit format:
   ```bash
   feat(<scope>): <description> (Plan <NNN>)
   ```
2. Remove the temporary worktree and branch (guard first — mandatory):
   ```bash
   bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-<NNN>/auto-lang   # 必须输出 clean 才继续
   git worktree remove D:/autostack/.wt/lang-<NNN>/auto-lang
   git branch -d plan-<NNN>-dev
   # 组内已无兄弟 worktree 时删除组目录：rmdir D:/autostack/.wt/lang-<NNN>
   ```
3. Summarize the completed work.
