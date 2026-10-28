---
plan_id: PLAN-867
status: drafting
feature_name: 拆分等价性验收：拆分前测试全集在三仓的闭包
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1
current_step: 0
total_steps: 11
supersedes_spec_components: [docs/specs/auto-lang/project.md, docs/specs/auto-ui/project.md, docs/specs/repository-boundaries/man-three-repository-delivery.md]
new_spec_components: [docs/specs/repository-boundaries/split-equivalence-acceptance.md]
touched_goals: []              # 有意留空：本计划不改任何 GOAL 的定义/达成状态，只冻结其回归基线（见 §5 规范增量末注）
affects: [test-infrastructure, language-sdk, auto-ui, auto-man, repository-boundaries]
depends_on: [PLAN-844]
---

# [PLAN-867] 拆分等价性验收：拆分前测试全集在三仓的闭包

> **整 Plan 启动门 G867 = U844**（844 全部 AC／final review／冻结门／三仓实际合入／Spec 与受管 ledger／归档收尾闭环）。844 未归档前只允许只读核查与规划簿记；不建实施树、不跑整档、不改任何仓实现。
> 本计划是 v0.6 拆分路线（M1+M2 之后）的**测试等价出口**，不是新的重构计划。

## 0. 变更摘要

844 是 M2 的最后一环，它的 T-09 只核 841/842/843 的交付收据，**不包含**"拆分前测试全集在新三仓仍可跑并全过"这一命题。因此 844 归档只证明计划链闭环，不证明测试等价：当前跨仓完整档只调度语言与 UI 两个 workspace（`auto-man` 面缺席）、"拆分前全集"从未以拆分前提交实测冻结、也没有"未解释缺口=0"的判据，且存在**空选择假绿的死别名**（语言仓 `tu` 在 `ui_gen` 迁出后选择 0 个测试，跑出来是绿的）。

本计划建立该判定：以拆分前提交 `abd34a355^`（UI 五包与 auto-man 仍在语言 workspace）实际枚举出的测试 ID 全集为唯一分母，把 auto-man 三档与 UI 的 `tu/ti/tn` 纳入三仓完整档，逐 ID 处置差额（迁移／改名／有意退役／丢失），为每仓每档加"选择数下限"守卫，并在冻结三仓组合上产出"拆分等价"收据。

## 1. 目标

- **G1 分母可复算**：拆分前全集（ID + 所属包 + 档位归属 + ignore 标记）可被任何人按记录的提交/工具链/命令逐字节复算。
- **G2 面完整**：三仓完整档单入口覆盖 语言 full + UI full（含 `ui_gen` 全族）+ 语言 `tt/tb` + **man `t/tc/tcore/ti`** + UI `tu/ti/tn`，沿用 850 已交付语义（机器单实例、零选择拒绝、子门失败传播、收据绑三仓 HEAD／锁／diff）。
- **G3 闭合**：`拆分前全集 − (三仓现全集 ∪ 迁移账本)` 的每一项都有四类处置之一与证据，**未解释缺口 = 0**。
- **G4 防假绿**：每仓每个档位有"最小选择数"断言；零选择或骤降即红；语言仓失效的 `tu` 别名有明确处置。
- **G5 行为等价**：迁移期间的金样/语料/期望值变更逐项有裁定记录；相对拆分前红集**新增红 = 0**。

**非目标**：不修拆分前既存红；不新增产品行为或测试；不改产品语义/公共 API；不重新生成金样以接受删除；不把 UI/Man 并回语言 workspace；不处理 80 万行净瘦身、THR-D1 crate member 拆分、UI 大文件结构残留（各自另立项）；不替 844 补其合同内项。

## 2. 架构方案

三段式，全部只读或"验证设施"改动，不触碰产品代码：

1. **冻结分母**：在 `abd34a355^` 建 detached 只读树，按档执行真实枚举（nextest `list --message-format json` / libtest `-- --list`），产出可入库的基线文件（与 850 的 `scripts/full-regression-*-{ids,selected}.txt` 同构），并采集同一提交的红集证据。
2. **建门**：扩 `scripts/run-full-regression.py` 增加 man 与 UI 子档腿，`cargo tf --batch` 语义从"两个 workspace"扩为"三仓完整档"；新增档位选择守卫脚本；两处都不改变既有档位选择语义，只补缺席与防退化。
3. **判定**：集合差 → 四类处置 → 金样台账 → 红集对齐 → 收据。任何一类无法解释即阻塞，不允许以"总体绿"替代。

判定原则（写死，review 按此复核）：

- 分母只能来自**实际选择面**（编译产物 + nextest/libtest 枚举），不能用源码 `#[test]` 计数或"相似函数"替代。
- 基线文件是**验收输入**：不得为接受删除/ignore/改名而重新生成；重新生成只能在"基线被证明错误"时进行，并需在 §9 记录原因与新旧哈希。
- **"通过"的形式定义** = 新三仓在冻结组合上跑完三仓完整档后，失败 ID 集合与拆分前红集**完全相同**，且三仓全部档位选择数 ≥ 冻结下限。

## 3. 技术栈

cargo / nextest（`--list`、`--message-format json`、profile/`--config-file`）、libtest（`-- --list`，用于 `tu`/`tn` 这类 bare cargo test 档）、Python 3（与既有 `scripts/run-full-regression.py` 同栈，零额外依赖）、PowerShell/Git Bash（Windows 主环境）、git detached worktree（只读旧树）、机器级单实例锁（沿用 850 的 machine gate）、JSON 收据。

## 4. 需求分析与背景调查

### 4.1 授权与轮次

- 用户 2026-10-28 明确要求：把"844 之后如何确保拆分路线正确结束 = 拆分的新项目跑拆分前所有测试用例均能通过"的工作，总结成计划文件（本轮 `/auto-plan:new`）。
- 本轮授权范围：只在语言主检出写计划/设计与只读核查；**不**建实施树、不跑整档、不改三仓实现、不改 canonical Specs/ledger、不推送远端。
- 未来实施在 G867 满足后按 §8 建组树执行；默认授权覆盖验证设施（脚本/基线/守卫）与文档，不覆盖产品语义变更；若需改变既有支持承诺则 `needs_replan`。

### 4.2 已观察事实（全部可复核，2026-10-28）

| # | 事实 | 证据 |
|---|---|---|
| F-1 | 跨仓完整档只含语言 + UI 两腿；`legs()` 返回 `language`、`ui-full`，batch 追加 `tt`/`tb` | `scripts/run-full-regression.py:239-261` |
| F-2 | 冻结基线：语言 3,718 listed / 3,218 selected；UI 3,091 unique / 3,073 selected；UI 含 868 个 `ui_gen::` | `scripts/full-regression-language-{ids,selected}.txt`、`scripts/full-regression-ui-{ids,selected}.txt` |
| F-3 | 迁移账本 21 项（10 language→ui-full、9 ui-full 内改名、2 language 内改名） | `scripts/full-regression-migrations-841.json.txt` |
| F-4 | 拆分前状态 = `abd34a355^`（"PLAN-831 T-06 切源"的父提交）：`crates/auto-man` 与 UI 五包均在库 | `git show abd34a355^:Cargo.toml`；`git cat-file -e abd34a355^:crates/auto-man/Cargo.toml` → 0 |
| F-5 | 831 引用的语言源提交 `e199caf8a` 是 831 树内提交，主检出已不可寻址 | `git cat-file -e e199caf8a` → fatal（非有效对象） |
| F-6 | auto-man 家族迁出前后 `#[test]` 计数：`crates/auto` 37→35、`auto-man` 33→34，其余七包不变（合计 341→340） | `git grep -c` 于 `857efab79^`（"switch Man and CLI ownership (Plan 832)" 的父）与 `D:/autostack/auto-man` |
| F-7 | 语言仓 `tu` 别名 = `test -p auto-lang --lib --features auto-lang/ui-iced ui_gen::`，而 `ui_gen` 模块已迁出（现仅 `auto-ui-compiler` 有 `pub mod ui_gen`）→ 该别名选择 0 个测试 | `.cargo/config.toml`（语言仓 alias 段）；`D:/autostack/auto-ui/crates/auto-ui-compiler/src/lib.rs` |
| F-8 | auto-ui 日常 `t` 用 `default-filter = "not test(ui_gen::)"` 排除生成器族；其 owner 是 `tu`（861 passed + 2 ignored） | `D:/autostack/auto-ui/.config/nextest.toml`；842 merge 收据的 ui tu 数 |
| F-9 | man 三档：`t`（workspace lib + auto bin）、`tc`（管理 core 七包）、`tcore`（`core/` 子 workspace）、`ti`（集成档） | `D:/autostack/auto-man/.cargo/config.toml`；`docs/specs/repository-boundaries/man-three-repository-delivery.md` L28 |
| F-10 | 漂移已在发生但未逐 ID 归因：批量 860 收据为 language 3,712/3,212、ui-full 3,135/3,117（"含 4 个新增 ID"），与冻结基线不一致 | `docs/plans/.last-batch-regression.json`；`KNOWN-DEBT` D-824 解决记录 |
| F-11 | 拆分前存在预存红族：plan606、schema_fence、plan707 flake、gallery charts 预存红、`th` 3 项环境红、Windows 上 aavm 12 个 cfg 跳过 | `AGENTS.md` 全档资源表；`docs/specs/aavm/project.md` 验证矩阵注记 |
| F-12 | 850 已交付"零选择假绿拒绝、子门失败传播、单实例锁、收据绑 HEAD/锁/diff"，但其作用域限于 tf 批量子门，不含各仓日常别名 | `scripts/full-regression.md`、`run-full-regression.py` |
| F-13 | 850 文档引用的 UI 基线来源 `PLAN-846 merge-r2/auto_ui_*-pre831-root-binary-list.txt` **在库不存在**（`docs/plans/attachments/846/` 只有 merge-r1/r3/r4、review/work 等目录，无 merge-r2，全仓无 `*pre831*` 文件） | `Get-ChildItem -Recurse -Filter '*pre831*'` → 空；`git ls-files docs/plans/attachments/846/**` 无匹配 |

### 4.3 规范依据（现状，非目标态）

- [auto-lang/project.md](../specs/auto-lang/project.md) L18-20：完整测试编排 = `cargo tf/taa/ta/t3` 经 `scripts/full-regression-driver` 调 Python 编排器，**在语言与 UI 各自 workspace 串行执行**；零选择/子门失败/超时非零并保留收据。
- [auto-ui/project.md](../specs/auto-ui/project.md) §测试与档位 L16-20：UI 自持 `t/tu/ti/tn`；完整面由语言仓 `cargo tf` 串行调度**两个** workspace；迁仓前 3,091 唯一 ID 与当前选择逐项核对。
- [man-three-repository-delivery.md](../specs/repository-boundaries/man-three-repository-delivery.md) L28,L39：Man 根 9 包、`t/tc/tcore/ti` 档位与实测数；**未**纳入跨仓完整门。
- [PLAN-850](archive/850-restore-cross-workspace-full-regression.md)：跨 workspace 全量回归编排交付（本计划的上游基础）。
- [PLAN-844](844-cross-repository-dedup-and-net-slimming.md)：M2 收尾与净瘦身；本计划的启动门与"不补票"边界来源。

### 4.4 假设、未决与不声称

- **假设 A1**：拆分前红集可用"同 commit 附近的历史批量收据 + KNOWN-DEBT 已登记红族"证明；若收据缺失或输入/配置不可证未变，则回退为旧树定向重跑（T-02 作出有界决定并留 artifact）。
- **假设 A2**：man 与 UI 的档位在迁移中未被"重新定义"（只换了 owner）；若发现档位语义被改，记 finding 并由 T-04 逐项裁定，不静默接受。
- **不声称**：本轮未执行任何枚举/档位/基线文件生成；F-1..F-12 全部是只读观察。`e199caf8a` 不可寻址、`abd34a355^` 尚未做 real-list（T-01 才做），因此"分母数字"在 T-01 前保持未知，不预填。

## 5. 详细设计

### 5.1 分母冻结（T-01）

- 只读旧树：`git worktree add --detach D:/autostack/.wt/lang-867/pre-split abd34a355^`；**禁止 junction/symlink**；用完 `bash D:/autostack/wt-guard.sh <path>` 必须 clean 后 `git worktree remove`。
- 同一棵树上按当时在库的档位枚举（档位清单来自该提交的 `.cargo/config.toml` 与其 `.config/nextest*.toml`，不套用今天的选择器）：
  - 语言：`t`/`tf`/`tv`/`tt`/`tb`/`taa`/`ta`/`t3`/`th`/`tu`/`tn`（`tu`/`tn` 用 libtest `-- --list`，其余用 `nextest list --message-format json`）；
  - UI 五包：当时在库的 t / tu / ti / tn 选择面；
  - man 九包：当时在库的 `t`/`tc`/`tcore`/`ti` 选择面。
- 记录每档：命令原文、工具链（`rustc -V`、`cargo -V`、`nextest -V`）、提交、`--list` 原始 JSON 哈希、ID 行数、unique 数、ignore 数、profile 归属。
- 产物（**入库**，与 850 同构）：`scripts/pre-split-universe/<tier>-{ids,selected}.txt` + `scripts/pre-split-universe/manifest.json`（commit/工具链/命令/sha256）。命名与目录由 T-01 在实现时确认（见 §10 D-3）。
- 对账：与 `scripts/full-regression-*-{ids,selected}.txt` 及 [831 包级 parity](attachments/831/tier-parity-f831-09-04.json)（基线 `e199caf8a` 五包 `only_baseline`/`only_current`）三方比对，差异逐项归入 §5.3 的四类处置（不允许出现"因为换选择器所以不同"这类未归因项）。850 引用的 `846 merge-r2` pre831 清单在库缺失（F-13），T-01 以"旧树重采 + 831 parity"替代该 source；source 缺失不得成为降低对账强度的理由。

### 5.2 三仓完整档（T-03）

- 扩 `scripts/run-full-regression.py`：
  - `root()` 增加 `auto-man`（解析序：`AUTO_MAN_ROOT` → 组内兄弟 `../auto-man` → `D:/autostack/auto-man` 主检出），`validate_layout()` 校验三仓 metadata 与消费关系；
  - 新增腿：`man-t`、`man-tcore`、`man-tc`、`man-ti`（CWD=`auto-man`）、`ui-tu`、`ui-ti`、`ui-tn`（CWD=`auto-ui`）；
  - 顺序固定：language → ui-full → tt → tb → man-t → man-tcore → man-tc → man-ti → ui-tu → ui-ti → ui-tn；首红停止、`--no-fail-fast` 兼容、每腿 deadline 不改变 nextest 单测 deadline；
  - 收据增三仓 HEAD、锁文件哈希、代码/配置 diff、每腿选择数与唯一 ID 数、失败 ID 列表。
- 兼容：`cargo tf`（无 `--batch`）保持"language + ui-full"原语义，不动 per-plan 门；**三仓完整档只在 `--batch`**（见 §10 D-4/D-5）。
- 零选择拒绝扩展：任一腿 selected = 0 → 整门非零（沿用 850 语义，不新增"看起来绿"的路径）。

### 5.3 ID 闭合账本（T-04）

- 键：`package⇥test-id`（850 已用该形态）；libtest 档单独命名空间。
- 四类处置（每项必须落一类，且有证据引用）：

| 类 | 判定 | 必备证据 |
|---|---|---|
| MIGRATED | 在迁移账本中（841/842 的显式 old→new 映射） | 账本条目 + 新 owner 实际枚举命中 |
| RENAMED | 同测试体换模块路径/包名 | 测试体逐字相同，或逐行等价论证 + 独立复核签署 |
| RETIRED | 其被测代码已被有意退役 | 引用 841/842/844 的退役项（含 844 §3 R-1..R-7 与 §2 D 族）及消费者证据 |
| LOST | 以上皆非 | 必须恢复（回到 MIGRATED/RENAMED 或补回测试），否则阻塞 |

- 输出：`docs/plans/attachments/867/closure.md`（md 摘要：各类计数、逐项表、哈希、复算命令）；全量差集清单作为本地工件，不入库。
- 已知必须先结案的差额：语言 3,718↔3,712、UI 3,091↔3,135（含"4 新增"）、`ui_gen` 866 selected ↔ 当前 863、man `crates/auto` −2 与 `auto-man` +1。

### 5.4 档位选择守卫（T-05）

- 新脚本 `scripts/check-tier-selection-minimums.py` + 数据文件 `scripts/tier-selection-minimums.json`（每仓每档：`minimum`、`allow_new` 策略、`owner`）。
- 下限来源：T-01 冻结值与当前实测值的**较小者**需解释；建议以冻结值或实测值为基线并写明理由（例：语言 `t` ≥ 3192、`tv` = 163、`tt` ≥ 3563、`tb` ≥ 3102；UI `t` ≥ 2441、`tu` ≥ 861（+2 ignored）、`ti` ≥ 28、`tn` = 2；man `t` ≥ 281、`tc` ≥ 23、`ti` ≥ 43）。
- 挂载：三仓 merge 门脚本入口 + `tf --batch` 收据中的附属断言；不侵入产品构建。
- 语言仓 `tu`：若 T-05 实测确认为 0 选择，则**删除该别名**并在 `docs/specs/auto-lang/project.md` 注明 owner 已迁至 auto-ui `tu`（决策 D-2）；不允许保留"能跑但空"的别名。

### 5.5 金样与语料等价（T-06）

- 清点对象：`cargo tv` 语料 163、`tt` 转译金样 3,563、`tb` book 3,102、`ui_gen` 863、画廊围栏（gallery fence）、1M churn、`test/a2c`/`test/a2r` 金样目录。
- 以拆分前后两次冻结输入比较文件级 diff；每处变更必须归入：① 迁移前已存在（历史裁定）② 本路线期间裁定变更（引用 841/842/844 收据，例：842 的 8 个 status token CSS 金样）③ 未解释（→ 阻塞）。
- 禁止"重新生成金样后自证通过"；重生成只允许作为修复后的显式动作并单独记录。

### 5.6 红集对齐（T-07）

- 拆分前红集来源优先级：① 覆盖同一/更近 commit 的历史批量收据（须证明代码、依赖、测试配置、输入未变，按现有证据复用规则）；② 旧树定向重跑（有界 deadline、单实例、只跑必要腿，记录未跑项）。
- 已知红族必须逐 ID 列出（F-11）并与实测对照；Windows 上 `cfg_attr(ignore)`/skip 的项按"跳过"记录，**不得计为 pass**。
- 判定：新增红 = 0；消失红需说明原因（不得以减少输入/放宽阈值实现）。

### 5.7 执行合同（work/review/merge 共用一版）

| 字段 | 内容 |
|---|---|
| 规则与边界 | G1–G5；仅验证设施与基线；不修产品缺陷；不重生成金样；三仓各一树 |
| 实际路径 | 旧树枚举 → 基线文件 → 三仓门 → 集合差 → 四类处置 → 金样台账 → 红集对齐 → 收据；新机制（man/UI 腿、守卫脚本）标注为"新增，T-03/T-05 负责可运行" |
| 所有权与状态 | 基线文件为验收输入（只增不改，改需原因+哈希）；各腿 owner=其仓；收据 owner=语言仓编排器；失败 ID 列表为唯一红集口径 |
| 区分性用例 | §6 V-01..V-07：合法控制（正常全绿）+ 反例（零选择、必选 ID 被删/ignore、金样未记录漂移、红集新增） |
| Oracle 与关闭 | 每项 AC 绑定可执行命令与期望值；关闭需实际输出 + 哈希；review/merge 各自 pending 项显式在档 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/project.md | 完整编排"语言 + UI 两个 workspace" → "三仓完整档（含 man `t/tc/tcore/ti`、UI `tu/ti/tn`），仅在 `tf --batch` 生效"；零选择/档位下限拒绝扩展到全部别名 | 拆分后 man 面与 UI 子档不在门内 | AC-02, AC-04 |
| SD-02 | modify | docs/specs/auto-ui/project.md | §测试与档位：保留"日常 t 排除 ui_gen、tu 为其 owner"，新增"档位选择数下限守卫"与"ui-full 必含 ui_gen 全族"的机器判据 | 防空选择假绿 | AC-04 |
| SD-03 | modify | docs/specs/repository-boundaries/man-three-repository-delivery.md | Man 三档仅在仓内自证 → 纳入跨仓完整门与收据绑定（三仓 HEAD/锁/diff） | 832 交付时未纳入跨仓门 | AC-02, AC-05 |
| SD-04 | add | docs/specs/repository-boundaries/split-equivalence-acceptance.md | 无统一"拆分等价"口径 → 冻结分母/四类处置/金样台账/红集对齐/未解释缺口=0 的形式定义与出口收据 | M2 之后缺出口判据 | AC-01..AC-06 |

> `touched_goals` 有意留空：本计划不改任何 GOAL 的定义或达成状态，只冻结其回归基线（GOAL-003/004/007 的语料与 UI 生成族正在被本计划的基线覆盖，但不构成目标变更）。

## 6. 测试设计

所有 Case 的 `owner_stage` 均为 `work`；本轮**未执行**，期望值在 T-01..T-06 建立对应能力后才可观测。

| Case ID | AC / invariant | 输入或 setup | 真实入口 | 可观察期望 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01 分母可复算 | `abd34a355^` 只读树；同工具链 | 各档 `nextest list --message-format json` / libtest `-- --list` | 两次独立枚举逐字节相同；与 850 冻结文件、831 包级 parity 的三方差集全部归因（无"选择器不同"类未归因项） | 基线文件 sha256 + manifest + 差异表 | 拆分前在库的档位；不含拆分后才有的新档 |
| V-02 | AC-02 门有效 | 三仓布局 + 故意注入：某腿过滤不命中（零选择）／man 腿缺失／无效 root | `python scripts/run-full-regression.py tf --batch --list-only` 与真实 `--batch` | 零选择、缺 owner、无效 root 均非零且收据写明原因；正常布局全腿枚举通过 | 收据 `outcome`、失败腿名、三仓 HEAD 字段 | 三仓完整档；不含其它路线工具 |
| V-03 | AC-02 失败传播 | 一条腿人为失败（只读副本或临时用例，不改生产源码） | 同上真实执行 | 首红停止、退出码非零、收据保留首红腿与 ID；子进程超时/中断不留悬挂进程 | receipt + 进程树检查 | 单机串行语义 |
| V-04 | AC-03 闭合有效 | 构造 3 类：删一个基线 ID 对应测试、把一个必选测试改 `#[ignore]`、改名一个测试体（等价/非等价各一） | `closure` 脚本 | 删除→LOST 阻塞；ignore→LOST 阻塞（除非有 RETIRED 证据）；等价改名→RENAMED 通过；非等价改名→阻塞 | 处置表 + 退出码 | 语言/UI/man 三仓各取一例 |
| V-05 | AC-04 守卫灵敏度 | 正常档位；再以过滤子集模拟骤降；再确认语言 `tu` 真实为 0 | `scripts/check-tier-selection-minimums.py` | 正常全绿；骤降/零选择非零并指出腿与计数 | 输出 JSON + 退出码 | 三仓每档 |
| V-06 | AC-05 金样台账 | 已知 842 CSS 金样变更（应有记录）+ 制造一处未记录漂移 | 台账校验脚本（读 diff 与裁定表） | 已记录项通过；未记录漂移红 | 台账 + 校验输出 | tv/tt/tb/ui_gen/a2c/a2r 目录级 |
| V-07 | AC-06 红集对齐 | 用既有收据红集回放；再制造一条新红 | 对齐脚本 | 红集相同→pass；新增红→fail 并列出 ID；消失红未解释→fail | 红集差异报告 | 拆分前红族 + 环境腿 pending 政策 |

**角色与树**（Markdown 约定，非后端字段）：
实现：本计划实施会话。内部自检：同会话定向检查（pass 只表示可送终审，不替代终审）。独立终审：独立上下文复审。收尾：merge 在冻结三仓输入上执行三仓完整档与触面档。

**环境政策**：`th`（真 TCP）、`tn`（native 手工门）、`taa` Windows 跳过项按现状记录为 pending/跳过，缺环境标 pending 且不得算 pass；重型族（gallery 围栏、1M churn、aavm XL）在机器级单实例锁下运行，遵守 AGENTS 并行纪律。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | V-01：拆分前全集以 `abd34a355^` 实测冻结并入 `scripts/`；两次枚举一致；与 850 冻结文件/831 parity 的三方差集逐项归因 |
| AC-02 | work | V-02/V-03：单入口三仓完整档可运行；含 man `t/tc/tcore/ti` 与 UI `tu/ti/tn`；零选择/缺 owner/无效 root/子门失败均非零并留收据（绑三仓 HEAD/锁/diff） |
| AC-03 | work | V-04：`closure` 产出未解释缺口 = 0；四类处置齐全且证据可点开；已知差额（语言 3,718↔3,712、UI 3,091↔3,135、ui_gen 866↔863、man −2/+1）全部结案 |
| AC-04 | work | V-05：三仓每档选择数下限守卫在位且灵敏；语言 `tu` 已删除或转发并在规范中注明 owner |
| AC-05 | work | V-06：金样/语料变更台账完整；无未解释漂移；842 CSS 金样等既知项有裁定引用 |
| AC-06 | work | V-07：相对拆分前红集新增红 = 0；消失红有解释；环境 pending 显式在档 |
| AC-07 | review | 独立终审绑定三仓冻结 commit/依赖/配置/输入，核 V-01..V-07 与 SD-01..SD-04、遗漏/警告/格式/debug 扫描通过；merge-owned 冻结门显式 pending |
| AC-08 | merge | 并集影响复审通过；在冻结三仓组合上实际跑三仓完整档并留收据；SD 落地 + 受管 ledger 读回 + 归档与逐树 guard clean 清理；本计划构成拆分路线的测试等价出口 |

## 8. 执行步骤

11 项 = 9 work + 1 review + 1 merge；G867=U844 对全部行生效，门前保持 drafting 0/11。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | G867 | 建 `abd34a355^` detached 只读树，按当时在库档位枚举三仓全部测试面；产出 `scripts/pre-split-universe/*` 与 manifest（commit/工具链/命令/sha256）；与 850 冻结文件/831 parity 对账。验证：两次枚举逐字节一致（复算脚本随 T-01 交付） | AC-01 |
| [ ] | T-02 | work | G867；T-01 | 红集证据路径有界决策（复用历史收据 vs 旧树重跑）+ 采集：逐 ID 红集、跳过项、环境 pending 清单；artifact `docs/plans/attachments/867/red-baseline.md` | AC-06 |
| [ ] | T-03 | work | G867；T-01 | 扩 `scripts/run-full-regression.py`：三仓 root 解析/校验 + man 四腿与 UI 三腿 + 收据扩展；保持 `tf` 原语义。验证：`cargo tf --batch --list-only` 全腿枚举非零选择；负控见 V-02/V-03 | AC-02 |
| [ ] | T-04 | work | G867；T-01,T-03 | `closure` 脚本 + `attachments/867/closure.md`：四类处置、已知差额结案、未解释=0。验证：V-04 三类构造 + 已知差额表 | AC-03 |
| [ ] | T-05 | work | G867；T-01 | `scripts/check-tier-selection-minimums.py` + `scripts/tier-selection-minimums.json`；语言 `tu` 别名处置（删或转发）+ 规范注明。验证：V-05 | AC-04 |
| [ ] | T-06 | work | G867；T-01 | 金样/语料台账 `attachments/867/golden-ledger.md` + 校验脚本；覆盖 tv/tt/tb/ui_gen/a2c/a2r/gallery 围栏、1M churn。验证：V-06 | AC-05 |
| [ ] | T-07 | work | G867；T-02,T-03 | 红集对齐：在冻结组合上跑三仓完整档，产出差异报告；新增红=0 或逐条归因（归因不豁免） | AC-06 |
| [ ] | T-08 | work | G867；T-03..T-07 | SD-01..SD-04 草案落地到 `docs/specs/`（只改本计划声明面）+ `attachments/867/handoff.md` 冻结送审；不改产品代码 | AC-03,AC-05 |
| [ ] | T-09 | work | G867；T-08 | 全 AC/SD/V 自检与遗漏扫描：把每条 AC 绑到实际命令与输出；未闭合项显式列为 finding 或 pending | AC-01..AC-06 |
| [ ] | T-10 | review | G867；T-09 冻结提交 | 独立完整复审（V-01..V-07 重放 + SD 校验 + 三仓实际运行证据）；不跑整档替代证据；merge 门 pending | AC-07 |
| [ ] | T-11 | merge | G867；T-10 pass | 并集影响复审、冻结三仓输入上实跑三仓完整档、SD/ledger 沉淀、归档与清理，构成拆分等价出口 | AC-08 |

**实施树（未来，G867 后）**：`D:/autostack/.wt/lang-867/{auto-lang,auto-ui,auto-man}`，每仓一树、分支 `plan-867-dev`；auto-down 只在需要三仓组合消费时以 detached 固定版本解析，不改实现。禁止 junction/symlink；清理前逐树 `bash D:/autostack/wt-guard.sh <path>` 必须 clean。

**门禁分级**：本计划只改脚本/配置/文档（Category C，外加 `.cargo/config.toml` alias 与 `.config/nextest*.toml` 读面）。开发期用 `python -m unittest discover -s scripts -p 'test_full_regression.py'` 与新增脚本单测；不跑 `cargo t/tf` 整档；merge 在冻结输入上跑三仓完整档 + 触面档。

## 9. 复审记录

2026-10-28 / stage: new / plan_id: PLAN-867 / revision: 1 / outcome: **pass（合同可执行；实施等 U844）** / next: 844 T-12 归档后 work T-01 / changed: 新建 T-01..T-11、AC-01..AC-08、V-01..V-07、SD-01..SD-04。

- 本轮只在语言主检出写合同：未建实施树、未跑任何档位/枚举、未改三仓实现、未改 canonical Specs/ledger、未推送远端。
- 取号来源：`scripts/new-plan.sh split-equivalence-test-closure`（master 主检出，原子取号 867 → 868）。
- 已知未决（不阻塞 drafting，阻塞对应任务）：§10 D-1..D-6。其中 D-1（红集证据路径）由 T-02 作有界决策；D-2（语言 `tu` 处置）由 T-05 提交决策；D-3（基线文件位置）由 T-01 确认；D-4/D-5（`--batch` 语义与 regress 收据兼容）在 T-03 前需与 merge 侧对齐。

## 10. 待澄清事项

| ID | 事项 | 影响 | 处置 |
|---|---|---|---|
| D-1 | 红集证据走"复用历史批量收据"还是"旧树定向重跑" | T-02/T-07 成本与可证性 | T-02 作有界决策并留 artifact（优先复用；不可证未变则重跑受影响的腿） |
| D-2 | 语言 `tu` 别名：删除还是转发到 auto-ui | AC-04 | T-05 先实测 `-- --list` 为 0 再决定；默认建议删除并注明 owner |
| D-3 | 拆分母基线文件放 `scripts/pre-split-universe/` 还是平铺 `scripts/full-regression-*` 命名 | T-01 结构、后续可发现性 | T-01 确认；倾向独立目录避免与 850 既有文件混淆 |
| D-4 | `tf --batch` 语义从两 workspace 扩到三仓，是否影响既有 `/auto-plan:regress` 收据 schema 与到期判定 | T-03/merge | 扩为向后兼容：新增字段不删旧字段；旧收据仍可读；T-03 前与 regress 侧确认 |
| D-5 | 是否把 man 腿也并入无 `--batch` 的 `cargo tf` | per-plan 门成本 | 建议不并入（保持 850 交付语义），只在 `--batch` 生效 |
| D-6 | `th`/`tn` 与 Windows 跳过项的 pending 政策 | AC-06 判定口径 | 沿用既有：缺环境标 pending、跳过不计 pass；不新增豁免 |

G867 未满足前禁止建树/跑档/改实现。若实施中发现"拆分前全集"本身不可重建（例如旧树无法构建），须先提交具体 `needs_replan`，不得以缩小分母片面宣布等价。

