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
> `.autoos/specs.json` 仅作只读旧数据来源（auto-musk PLAN-107）；当前规范在 docs/specs，分类/关系/历史由应用派生，agent 不再写该 JSON。
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
- **计划证据不入 git（2026-10-31 用户裁定，升级 2026-10-28）**：
  `docs/plans/attachments/` 整目录**不入库**（含 `*.md`，`docs/plans/` 下只有计划文件本身入库）；
  根 .gitignore 整目录忽略 + pre-commit 拒绝该路径的任何暂存（`git add -f` 与已跟踪文件再暂存
  同样拦截；豁免 `AUTO_LANG_ALLOW_EVIDENCE=1`，须在 plan 记录理由）。存量 8548 个已跟踪
  attachments 文件已随本裁定去跟踪（`git rm -r --cached`，磁盘文件保留、git 历史不重写）；
  已 push 历史中的证据不做历史重写，未 push 存量的历史过滤任务不变。auto-musk / auto-down
  同规则整目录忽略。auto-plan-* 技能已同步（auto-musk 源 merge 技能 Evidence handling 章节）。
### 计划暂存与证据：plan-artifacts 唯一暂存区，merge 即全删（2026-10-31 用户裁定，取代 2026-10-30 最近-5 裁定）

- **唯一暂存区**：除计划文件本身外，计划相关的一切文件（探针源码与输出、日志、截图、
  gate/ledger 转储、receipts/handoffs/spec-delta、构建产物副本）统一放
  `D:/autostack/plan-artifacts/plan-<NNN>/`（无 plan 号的轻量任务用任务 slug，如 `fix-<slug>/`）。
  计划开工即建目录，在途期间受保护；**绝不放进任何仓的项目目录**（attachments 三仓已整目录
  忽略；当日早前 `.wt/<group>/evidence/` 约定同日废除）。无 worktree 的纯文档工作用系统临时目录。
- **禁止散落输出**：外部消费者验证源码副本、SDK安装副本、发行包及需留存的构建产物均属
  计划材料，禁止另建 `D:/autostack-build`、其他盘符同名目录、散落的 `planNNN-*` 或第二套
  证据目录。外部验证必须显式指定输出和 target 路径，不依赖当前目录或工具默认值；正常
  构建默认在专属 worktree 内，确需外置的计划专用 target 只能放上述统一计划目录，并登记
  为可删除缓存。跨仓按仓/计划身份区分，避免同号覆盖；创建材料目录时即登记 owner、用途、
  路径与在途状态。
- **merge 即全删**：merge 收尾把仍需要的证据以 markdown 摘要（命令、退出码、关键数字、
  代码/日志节选、保留件的本地路径+内容哈希）**折叠进计划文件本身**（§7 验证记录 / §9 复审记录），
  然后**删除该计划的整个 plan-artifacts 目录**；构建中间产物（cargo target 树、.pdb、exe 副本）
  **不再回收暂存**，随 worktree 直接删除。仅**具名未解决问题**持有可保留（owner+理由+关闭
  条件记入 plan 与登记表）。理由：历史计划的原始证据几乎不会被回看——后续核验一律用最新
  代码重新测验；双暂存区+保留窗口已累积 67GB 构建中间产物，检出内附件目录还泄漏了 8916 个
  文件进 git 历史。
- **取代关系**：本节取代 2026-10-30"最近5个已完成计划"保留裁定及其 500 MiB/10 GiB 容量提醒、
  每日03:00维护指令；也取代 2026-10-27"保留每个 exe"旧令与当日早前 `.wt` 组内证据目录约定。
  清理由 merge 收尾自清 + 机会性孤儿清扫承担；未知归属/状态的目录 fail-closed，先核查再处置。
- **登记表**：`.retention/policy.json` 保留为"磁盘现存物+持有"索引（在途 scope、具名持有），
  merge 收尾刷新（记录删除与持有）。归档状态单独不足以证明问题已闭；持有不因时间流逝自动
  释放——须核对当前债务/关闭记录。`artifact_retention.py` 保留为保守的手动清扫/预览工具，
  其最近-5 逻辑不再是政策；链接仅删除链接本身，绝不穿透目标，不盲删活动进程相关目录。
- **收尾路径对账**：必须逐项核对本计划命令/日志中实际使用的所有输出目录（包括其他盘符
  与外部安装/消费根），不能只检查 worktree 和已登记 scope。发现散落目录先核查归属、链接、
  活动进程及唯一依赖；按本节保留条件需要保留的材料移入统一 scope，迁移前后核对文件数、
  字节数及SHA256并登记清单，其余安全删除。计划内记载原路径、去向、删除量和核查结果，
  刷新登记与完成文档hash；确认没有遗漏目录才能报告清理完成。未知归属不得盲删。

### 固定Cargo包与Rust模块测试（PLAN-896）

- 正式入口：`cargo t <组名> [组名 ...]`；例如`cargo t parser`、`cargo t parser vm`。
  分组写死于`scripts/test-groups.toml`，直接对应Cargo package/target及Rust模块namespace，
  不先发现全仓测试，不建叶目录/分片新鲜度系统，不自动展开消费者图。`cargo t --list`只读固定组名。
- `cargo t ace`显式运行独立`experimental/ace` workspace的常规测试及语言侧私有协议桥接；正式计量/压力以`cargo t ace-performance`运行，保原样本、规模、ignore和平台条件。ACE内部同树target复用，四个no_std静态运行时保panic-abort专用profile；普通语言测试不构建ACE。
- 无参数`cargo t`保固定日常范围；`cargo t all`显式运行语言仓全部固定组。两者不能混为一谈。
  全部组仍保ignored/平台/里程碑条件，不含外仓UI/Man所有权腿。模块过滤不保证同crate独立编译。
- 计划按真实修改明确选择完整职责组及必要集成/消费者组，review核选择理由；共享AST/语义/IR变更
  须明确补下游验证，不能仅跑parser自证所有消费者。包构建依赖不等于所有依赖方测试都应执行。
  未知触面具名审查/扩大，必要时选固定日常范围，不静默编全部experimental。
- 同配置多个组一次nextest并行调度；HTTP、重内存、正式性能门的原串行/资源组/Job/输出预算/截止
  保留。原样本、压力、金样和oracle不减；真实终态及包装内部check核对，父exit0不代替子项。
- 开发迭代跑相关反例与受影响完整组；最终审查后在冻结提交跑明确的固定组/必要集成并集。
  复用身份/输入未变的有效证据，不为修复、提交或复审反复跑全目录。纯文档/资产只做目标验证。
- `tm`与旧档别名（t除外的tv/tt/tb/th/tf/taa/ta/t3）已整体退役删除（PLAN-898，2026-10-31
  用户裁定豁免2026-11-30兼容期）；888/895目录及tm引擎一并删除，不再有"迁移审计资料"。
  义务由固定原生组承接；UI/Man保原owner入口。
  周期到期仍为计划号%5、或>48h且有合并、缺收据即到期（`cargo t all`，主检出单实例），
  不成为每计划全局门。
- 非ACE组不得补建ACE worker/实验工程；ACE相关及真实共享修改显式选对应ACE组。ACE宿主接线
  用默认关闭test-ace隔离，普通legacy TestCase保持可用。不得靠计划标题豁免真正必要的ACE验证。
- AAVM/AA2R仅保代码/测试/夹具/期望与显式手动入口，实验测试默认ignore（2026-10-31用户裁定）；
  正常VM/Rust实现A2R和真实产品消费者不误关。其他research/special不得据名/耗时批量停用。
- 列用例数/实际执行数/原条件排除数，准备/冷编译、发现/运行/清理和整命令时间分列；完整组尽量
  ≤60秒但不是强杀门，不按时间机械细拆VM；特别长原子具名交owner。无统一三warm/P90或A/B/C矩阵。
- 未执行、首次红、环境归因和重放绿分别保留，条件排除不充PASS；全量目录对账仅明确治理任务。
  下方AG专项/重内存规则和888 B/C试点原白名单/预算继续有效，不因换入口推广批量worker。

#### AutoGraphics (experimental/) Test Tiering (2026-10-28 user ruling)

- **每个计划只测自己的修改涉及到的用例（per-plan affected cases only）**：`experimental/autographics-*`
  线的计划在 work/review/merge 阶段只跑该计划改动触面的测试文件与命名 case（如
  `--test <新/受影响 test file>`、命名单测、新 checker 与其负控），**不为合并而跑全量 AG 套件**。
- **全量 AG suite 属里程碑档**：等到里程碑时单独抽空执行（单独时段、单实例），不占用日常合并窗口；
  2026-10-28 用户裁定“合并时跑全部 AG 测试套件约 40 分钟 computer use，期间用户无法用电脑”，
  因此 merge 不再以全量套件作为合入门槛，改为记录“里程碑档 pending”并保留一次已跑过的全量结果作为参考。
- 该线的定向面：新增/受影响的 `tests/*.rs`（单进程 `--test-threads=1`）、`verify_planNNN.py --controls`、
  `check_*.py`（含 `--controls`）、owner/窗口探针的命名 case；根 crate 零改动时**不**跑根
  `cargo t/tu/docs_gen`（沿用既有裁定；旧 tf/tv 别名已随 PLAN-898 删除）。
- 已知环境边界：原生窗口类测试（`authoring_interaction`/`authoring_probe_exit` 等）在同机并发或反复
  spawn 窗口时可能瞬时红/挂起，隔离重放绿时应按环境归因记录，不据此判代码失败。

#### Heavy-Mem Test Tiering (Plan 564)

- Per-test peak weights live in `.config/test-mem-weights.md` (single source of truth; measured by `python scripts/measure_test_mem.py <filter>`, nextest `--jobs=1` serial + per-process peak polling).
- Tiers: XL ≥800MB, LG 300-800MB, MD 100-300MB. nextest `[test-groups]` in `.config/nextest.toml` + `nextest-full.toml` + `nextest-t3.toml` enforce xl/lg/md = 1/1/2 threads. Since Plan 568's feature split, aavm heavy tests run via the `aavm` fixed group (manual `--ignored` replay) and the T3 milestone swap (`T3_MILESTONE=1` with the full-group union) — the daily tier never includes them (structural exclusion via `test-aavm` feature); groups in the daily config remain as a guard for ad-hoc `cargo nextest run --features test-aavm` without `--config-file`.
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
2. Deposit reviewed current knowledge in canonical module Specs/ADR per `/auto-plan:merge` and [docs/specs/README.md](docs/specs/README.md) §4. Verify landing/source identity; the application derives indexes. Never write legacy specs.json or require index/cache availability for archive (PLAN-107).
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
