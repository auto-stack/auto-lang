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
  - *Action* (**2026-09-15 用户裁定：小型改动同样必须走 worktree，master 零 WIP 代码**):
    轻量路径（无需正式 plan 文档）：`git worktree add D:/autostack/.wt/fix-<slug>/auto-lang -b fix-<slug>`
    （或一键建组含 auto-down 兄弟仓：`bash scripts/new-wt-group.sh fix-<slug> --branch fix-<slug>`，
    PLAN-726 T-04——组内 cargo 解析即开即用，缺兄弟仓时 check 直接失败）
    → 修改 + 按改动范围跑验证门禁 → 提交 → 合回 master → `bash D:/autostack/wt-guard.sh
    D:/autostack/.wt/fix-<slug>/auto-lang`（必须 clean）→ 移除 worktree/分支/组目录。
    实机走查/核查会话中的**顺手修复**同样适用本条（2026-09-15 三线 WIP 直接落 master 的事后矫正）。
    master 上仅允许计划簿记类改动（`docs/plans/**`、`.next-id`、specs/docs 沉淀）。
- **L1: Feature / Module Tasks (模块/特性任务)**
  - *Criteria*: Any new feature, multi-file change, complex bugfix, or cross-backend parity implementation.
  - *Action*:
    1. Run `scripts/new-plan.sh <slug>` on the default checkout (master) to atomically take the next `<NNN>` plan ID from `docs/plans/.next-id` and create the plan skeleton (v2 frontmatter).
    2. Fill `docs/plans/<NNN>-<plan-name>.md` (needs-analysis seeded from [docs/specs/overview.md](docs/specs/overview.md)) detailing goals, design, task checklist, and verification plan; present for confirmation before executing.
    3. Create a dedicated worktree in the sibling-group layout (Plan 529): `git worktree add D:/autostack/.wt/lang-<NNN>/auto-lang -b plan-<NNN>-dev`，或一键 `bash scripts/new-wt-group.sh lang-<NNN> --branch plan-<NNN>-dev`（自动补 auto-down 兄弟 detached；基面恒钉 master，PLAN-726 T-04）. Cross-repo plans add sibling worktrees into the same group dir (e.g. `.wt/down-047/{auto-down, auto-lang}`) so `../auto-lang` resolves uniformly.
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
- **大文件入库门禁（2026-10-28 用户裁定）**：`pre-commit` 钩子（`.githooks/pre-commit`，
  经 `core.hooksPath=.githooks` 生效，主检出与各 plan worktree 共用该配置）拒绝单个
  >5MB 的暂存文件（防二进制/大日志/大量截图误入跟踪；已跟踪存量不受影响），单次提交
  暂存总量 >100MB 仅警告（证据批量沉淀合法）。豁免：`AUTO_LANG_ALLOW_LARGE=1 git commit`
  并在对应 plan 记录理由；证据一律提交截断/摘要（2026-10-21 裁定）。
- **复审证据不入库（2026-10-28 用户裁定）**：`docs/plans/attachments/**` 下的复审/执行
  证据（探针源码与输出、日志、截图、gate/ledger 转储、delivery 拷贝）一律**本地保留、
  禁止 git add/commit/push**——attachments 的 .gitignore 只放行 `*.md`（摘要/契约/治理
  文档），pre-commit 钩子拒绝暂存非 md 证据路径（`git add -f` 与已跟踪文件再暂存同样
  拦截；豁免 `AUTO_LANG_ALLOW_EVIDENCE=1`，须在 plan 记录理由）。证据只在复审与修复
  期间有意义；摘要（结论/命令/退出码/关键数字/本地路径+内容哈希）写入 plan 文件即可。
  已 push 的存量证据不动；未 push 的存量证据在全部在途 worktree 合并后做历史过滤再
  push。auto-plan-* 技能已同步此政策（auto-musk 源 b98aacd）。
### 模块选测与验证门（PLAN-888 r2）

- 本节替代旧的“裸 cargo t + tv/tt/tb/taa 触面档”逐档叠跑规则。测试义务按
  `scripts/test-governance/modules.json` 及其内容哈希分片划分为不相交叶模块；每条原子
  运行身份唯一归属，源码、期望、profile、平台/ignored/里程碑条件不因拆分削弱。
- 公开入口：`cargo tm --list`；`cargo tm <module-id> [module-id ...]`；
  `cargo tm --changes <逐行变更路径文件> --select-only --out <本地目录>`。
  去掉 `--select-only` 执行所选完整模块及直接/传递消费者与集成闭包的去重并集。
  无参数不运行；未知触面保守扩大，过期目录重新发现，非空零选择/漏项/重复/漏消费者失败。
- **开发迭代**：在计划 worktree 完成全部相关修复，运行有意义的反例及受影响模块，必要时
  `cargo check -p auto-lang`。不为单个 finding、提交或再次送审重复运行周期性全集。
- **合入门**：最终代码/验收/规范审查通过后，在冻结提交执行受影响完整模块并集，包含
  消费者、integration 和必要慢模块。明确列出选择文件、源/依赖/配置身份与执行收据。
  不再隐式叠跑旧 t/tv/tt/tb/taa。目录/runner 等横切迁移自身可能触及全部本仓模块，
  必须据实际影响验证，不能只测 runner 自身就撤旧门。
- **时间口径**：常规叶模块至少三次完整 warm，nearest-rank P90 目标≤60秒；模块墙钟
  包括运行时发现、执行、原比较器和清理，预构建/冷态/整条命令总时长另列。可拆的超标
  聚合继续拆，单条不可分割的正式性能采样/规模/生命周期 oracle 登记具名慢模块，仍由
  触面计划执行。不得减样本/改金样/删除断言/60秒强杀或 SKIP 冒绿。
- **条件与结果**：原 ignored/平台/手动/里程碑条件明确报告 NOT_APPLICABLE；发现、选择、
  编译、实际执行和通过分别统计。原聚合包装 exit0 不代替内部原子比较。既存红保留原始
  结果并具名归因，不用重放绿覆盖首次失败。未执行条件不认证为全环境绿。
- **纯文档/资产/核查**：只做目标验证；不无故运行 Rust 测试或 docs_gen。
  文档生成器/Schema/语法参考变动才触发 docs_gen 对应模块。测试工具变动执行其真实消费者。
- **外仓 owner**：语言仓没有 cargo tu；UI 生成/运行/原生窗口由 auto-ui 对应入口维护，
  Man 由 auto-man 维护；按真实跨仓触面选择，不因本仓测试重划运行无关 UI/Man/AG 全套。
- **旧入口兼容**：t/tv/tt/tb/th 截至2026-11-30只作显式过渡导航，映射冻结旧身份到完整
  叶模块；附加滤串选择完整模块，不在模块内抽样。tf/taa/ta/t3 经 module-aware 周期驱动，
  旧列表仅守恒审计、不重复执行语言档。新计划和新脚本使用 tm 的明确模块/变更清单。
  不要求新增名为“全部测试”的入口；任何未来 all 只是显式模块并集。
- **周期批次**：保留计划号%5落地、或>48h且期间有合并（含L0）、缺收据即到期的条件；
  `docs/plans/.last-batch-regression.json` 为收据。到期由 auto-plan-regress 在主检出单实例
  执行 `cargo tf --batch`，语言侧消费 `periodic-module-schedule.json` 的显式 recipe 清单，
  输出实际叶模块选择并只执行一次去重并集；UI/Man 原 owner 腿保持。它不是每计划第二道全局门。
- **并行与重型纪律**：普通定向验证保留原 nextest 隔离、资源组和截止；HTTP 族串行。
  画廊围栏/1M churn 保留 `heavy_gate.rs` 机器单实例锁，第二实例确定性拒绝。
  正式性能测量与周期批次串行；M888 专用全叶成本属于授权测量，不是开发提交反复跑全集。
  不停止他方进程；观察到外部构建/测试污染时保留原结果并按合同整轮重测。
- **AAVM**：仅真实改动 `packages/aavm/lib/*.at`、`test/vm/aavm2/**`、`test/parity/**`、
  aavm2 harness 或 lib.rs 中 aavm 源绑定时，选对应 AAVM 完整模块及消费者；一般 VM/编译器
  改动不凭名称反射性选 AAVM。上游共享变化会扩大闭包，必要时包含整个管线。
  双重解释器路径 Windows 原12条ignore及其栈边界裁定保持；重型对拍用转译+编译+运行形态，
  不新增进程内双解释重负载。T3_MILESTONE 条件原样保留，不能把未触发的秒退当里程碑通过。
- **B/C试点**：test-case-batch 默认关闭，只允许固定15例白名单；测试专用 pilot 默认
  B k=10，单case重放默认 A。其他 recipe、负诊断、资源/会话/FFI/性能硬门继续原独立路径。
  C 只观察原D3的owned-record组合结果，不替代四路径或原单文件义务。原 guest/采样预算不变。

#### AutoGraphics (experimental/) Test Tiering (2026-10-28 user ruling)

- **每个计划只测自己的修改涉及到的用例（per-plan affected cases only）**：`experimental/autographics-*`
  线的计划在 work/review/merge 阶段只跑该计划改动触面的测试文件与命名 case（如
  `--test <新/受影响 test file>`、命名单测、新 checker 与其负控），**不为合并而跑全量 AG 套件**。
- **全量 AG suite 属里程碑档**：等到里程碑时单独抽空执行（单独时段、单实例），不占用日常合并窗口；
  2026-10-28 用户裁定“合并时跑全部 AG 测试套件约 40 分钟 computer use，期间用户无法用电脑”，
  因此 merge 不再以全量套件作为合入门槛，改为记录“里程碑档 pending”并保留一次已跑过的全量结果作为参考。
- 该线的定向面：新增/受影响的 `tests/*.rs`（单进程 `--test-threads=1`）、`verify_planNNN.py --controls`、
  `check_*.py`（含 `--controls`）、owner/窗口探针的命名 case；根 crate 零改动时**不**跑根
  `cargo t/tf/tv/tu/docs_gen`（沿用既有裁定）。
- 已知环境边界：原生窗口类测试（`authoring_interaction`/`authoring_probe_exit` 等）在同机并发或反复
  spawn 窗口时可能瞬时红/挂起，隔离重放绿时应按环境归因记录，不据此判代码失败。

#### Heavy-Mem Test Tiering (Plan 564)

- Per-test peak weights live in `.config/test-mem-weights.md` (single source of truth; measured by `python scripts/measure_test_mem.py <filter>`, nextest `--jobs=1` serial + per-process peak polling).
- Tiers: XL ≥800MB, LG 300-800MB, MD 100-300MB. nextest `[test-groups]` in `.config/nextest.toml` + `nextest-full.toml` + `nextest-t3.toml` enforce xl/lg/md = 1/1/2 threads. Since Plan 568's feature split, aavm heavy tests run under `taa`/`ta` (full config) and `t3` — the daily tier never includes them (structural exclusion via `test-aavm` feature); groups in the daily config remain as a guard for ad-hoc `cargo nextest run --features test-aavm` without `--config-file`.
- XL/LG tests carry an in-body `heavy_gate` guard (`crates/auto-lang/src/tests/heavy_gate.rs`): under bare `cargo test` (no `NEXTEST` env) they SKIP instantly — the 2026-09-05 incident (bare run, 12 threads, 9.78GB peak) cannot recur. SKIP guidance is visible with `--nocapture`; force-run with `AUTO_LANG_HEAVY_MEM=1`.
- Registering a new heavy test (3 steps): measure → classify in the weights file → add to nextest overrides (all three configs) + wire `heavy_gate` at the test's first line.

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
