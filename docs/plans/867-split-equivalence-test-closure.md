---
plan_id: PLAN-867
status: execution_done
feature_name: 拆分等价性验收：拆分前测试全集在三仓的闭包
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 2
current_step: 2
total_steps: 13
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

本计划以 `abd34a355^` 为主锚，结合可追溯的可编译见证建立测试义务总账；主锚编译失败不能抹去原有义务。Man四档与UI三子档进入完整编排，分别验收ID归属、档位可执行性和实际执行结果。r1的8,315 ID是已枚举子集，th等参考面仍须进入总账，不能用枚举通过代替执行通过。

## 1. 目标

- **G1 分母可复算**：拆分前全集（ID + 所属包 + 档位归属 + ignore 标记）可被任何人按记录的提交/工具链/命令逐字节复算。
- **G2 面完整**：三仓完整档单入口覆盖 语言 full + UI full（含 `ui_gen` 全族）+ 语言 `tt/tb` + **man `t/tc/tcore/ti`** + UI `tu/ti/tn`，沿用 850 已交付语义（机器单实例、零选择拒绝、子门失败传播、收据绑三仓 HEAD／锁／diff）。
- **G3 闭合**：总账覆盖主锚、见证恢复和不可执行参考面；每项有归属处置与独立执行状态，未解释缺口=0。编译红/未执行不属于RETIRED，不得移出总账。
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
- **唯一通过口径**：归属未解释=0；冻结组合实际执行约定完整档，执行层新增红=0、消失红有解释，选择数满足审查下限。允许有证的既存红修复，不要求失败集合机械相等。既存编译红、环境缺失、ignored/skip、未执行分别披露，绝不计pass；未知或约定必需正控未执行时不放行。真实整档由final review后的T-11执行；此前不得宣称完整运行已通过。

## 3. 技术栈

cargo / nextest（`--list`、`--message-format json`、profile/`--config-file`）、libtest（`-- --list`，用于 `tu`/`tn` 这类 bare cargo test 档）、Python 3（与既有 `scripts/run-full-regression.py` 同栈，零额外依赖）、PowerShell/Git Bash（Windows 主环境）、git detached worktree（只读旧树）、机器级单实例锁（沿用 850 的 machine gate）、JSON 收据。

## 4. 需求分析与背景调查

本次修订授权（2026-10-28）：用户明确要求按本会话核查建议修订867/869/872/873及总路线，只改主检出计划文档；不执行产品/验证设施改动、不发布canonical/ledger、不建树/agent、不分配874/875或其他新号、不推送。已给的A/B、2,000行批粒度、六族DISTINGUISH裁定继续有效；具体结构性减法批仍沿原签核规则。

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
| F-16 | 844 已于 2026-10-28 归档（U844 成立，M2 四份全部 archived，受管 ledger v114）：生产净删终值 **−1,221**（lang NET-PROD 308,909→307,688），真重复清偿 5 族；退役 auto-lang-macros / auto-xml / ac-examples / infer::registry / legacy Scope+Database scope API / `Evaluator` 变体 / D-4 纯 shim 三函数 / autoui-skill 副本 —— 这些是 T-04 里 RETIRED 类的直接证据来源（编号跳过 F-14/F-15：两者已被 §9 work 记录占用） | [844 归档 §9](archive/844-cross-repository-dedup-and-net-slimming.md)、[net-slimming.md](../specs/repository-boundaries/net-slimming.md) |

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

**r2总账规则（替代r1缩分母处置）**：

- 每个测试义务记录来源提交/仓/包/二进制target/ID/档位。MIGRATED/RENAMED/RETIRED/LOST是归属维度；pass/fail/compile_blocked/env_pending/ignored/not_run是执行维度，不能互相替代。
- r1已枚举8,315的原始基线文件/hash留作不可变历史；r2新增义务采用版本化manifest或增量映射，复算差额与新旧hash有据，不能为接受丢测重生成旧基线。
- 主锚无法编译时，T-01用最后可编译见证枚举，并追踪至主锚间的测试、cfg、feature与配置差异。未知范围阻塞，不以源码计数或当前枚举自证完整。
- F-17的th面（见证09a756e84）仍在总账与最终报告内；reference目录只是存储位置，不代表排除验收。分列8,315历史子集、th等额外义务/重叠和最终去重总数，重算前不预填新数字。
- F-14/F-15/F-17须记首次失败提交、与约定拆分起点关系、当前状态。主锚与当前同红不足以证明非拆分引入；拆分期间导致的红不得借晚选基线记成既存红。确在约定范围前已存在且本轮未恶化的红由独立review逐项认证；所需产品修复交具名owner，合同未满足前仍阻塞。
- 最终分别给出归属闭合和执行状态。适用义务无失败/阻断/未执行才可称“全集全绿”；有证既存红只可称“已执行范围零新增红，仍有具名限制”，不冒称全部通过。环境延期仅继承具体已有裁定，不新增通用豁免。


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

- 新脚本 `scripts/check-tier-selection-minimums.py` + 数据文件 `scripts/tier-selection-minimums.json.txt`（每仓每档：`minimum`、`allow_new` 策略、`owner`）。
- 下限来源：T-01 冻结值与当前实测值的**较小者**需解释；建议以冻结值或实测值为基线并写明理由（例：语言 `t` ≥ 3192、`tv` = 163、`tt` ≥ 3563、`tb` ≥ 3102；UI `t` ≥ 2441、`tu` ≥ 861（+2 ignored）、`ti` ≥ 28、`tn` = 2；man `t` ≥ 281、`tc` ≥ 23、`ti` ≥ 43）。
- 挂载：三仓 merge 门脚本入口 + `tf --batch` 收据中的附属断言；不侵入产品构建。
- 语言仓 `tu`：若 T-05 实测确认为 0 选择，则**删除该别名**并在 `docs/specs/auto-lang/project.md` 注明 owner 已迁至 auto-ui `tu`（决策 D-2）；不允许保留"能跑但空"的别名。

### 5.5 金样与语料等价（T-06）

- 清点对象：`cargo tv` 语料 163、`tt` 转译金样 3,563、`tb` book 3,102、`ui_gen` 863、画廊围栏（gallery fence）、1M churn、`test/a2c`/`test/a2r` 金样目录。
- 以拆分前后两次冻结输入比较文件级 diff；每处变更必须归入：① 迁移前已存在（历史裁定）② 本路线期间裁定变更（引用 841/842/844 收据，例：842 的 8 个 status token CSS 金样）③ 未解释（→ 阻塞）。
- 禁止"重新生成金样后自证通过"；重生成只允许作为修复后的显式动作并单独记录。

### 5.6 红集对齐合同与定向控制（T-07；整档归T-11）

- 拆分前红集来源优先级：① 覆盖同一/更近 commit 的历史批量收据（须证明代码、依赖、测试配置、输入未变，按现有证据复用规则）；② 旧树定向重跑（有界 deadline、单实例、只跑必要腿，记录未跑项）。
- 已知红族必须逐 ID 列出（F-11）并与实测对照；Windows 上 `cfg_attr(ignore)`/skip 的项按"跳过"记录，**不得计为 pass**。
- 判定：新增红=0；消失红有解释，不得减少输入或放宽阈值。编译阻断以档位+诊断身份单列，不伪造失败测试ID。
- T-07完善比较器和既存红证据，定向验证同红/修复红/新增红/无据消失红/未运行冒充pass/删除参考面等控制；work/review不跑整档。T-11在final review后冻结执行约定全部腿；fail-fast导致的未运行腿须补齐或诚实阻塞。继续收集失败时保留原始整体非零，不将已解释红改成原始门绿。
- 869/872继承本口径；867之后新红不得滚入旧基线，归因不是豁免。证据复用须确认代码、依赖、配置与输入未变。

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

SD-01..04统一增加r2义务总账、归属/执行状态分列、既存红认证和限制报告、merge实际完整档规则；T-08更新proposal，本轮不发布canonical。

## 6. 测试设计

定向控制实现/验证owner_stage=work；真实完整档正控owner_stage=merge（T-11）。r1证据保留§9，受r2影响者须重验；本轮仅修订文档。

| Case ID | AC / invariant | 输入或 setup | 真实入口 | 可观察期望 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01 分母可复算 | 主锚+见证+th参考面，含重叠ID | 各档实际list与来源差异复算 | 总账并集可复算，重叠不双计，编译红面不消失；全部差额有据 | manifest/hash/逐面表 | 包含不可执行义务；未知阻塞 |
| V-02 | AC-02 门有效 | 零选择/man缺席/无效root负控，正常布局list-only | work运行list-only及受控小腿负控；merge实际完整档 | 缺owner/无效root/零选择非零并记原因；正常布局全腿枚举非零，不代替执行正控 | 收据/失败腿/三仓HEAD | 全腿编排；work不跑整档 |
| V-03 | AC-02 失败传播 | 受控小腿真实执行失败/超时/中断，不改产品源码 | work通过真实编排器运行有界测试腿；完整生产腿仅merge | 首红非零，准确记失败ID/未运行腿；取消后owned子进程退出；fake仅控制不冒充正控 | receipt/进程检查 | 编排错误路径定向控制 |
| V-04 | AC-03 闭合有效 | 构造 3 类：删一个基线 ID 对应测试、把一个必选测试改 `#[ignore]`、改名一个测试体（等价/非等价各一） | `closure` 脚本 | 删除→LOST 阻塞；ignore→LOST 阻塞（除非有 RETIRED 证据）；等价改名→RENAMED 通过；非等价改名→阻塞 | 处置表 + 退出码 | 语言/UI/man 三仓各取一例 |
| V-05 | AC-04 守卫灵敏度 | 正常档位；再以过滤子集模拟骤降；再确认语言 `tu` 真实为 0 | `scripts/check-tier-selection-minimums.py` | 正常全绿；骤降/零选择非零并指出腿与计数 | 输出 JSON + 退出码 | 三仓每档 |
| V-06 | AC-05 金样台账 | 已知 842 CSS 金样变更（应有记录）+ 制造一处未记录漂移 | 台账校验脚本（读 diff 与裁定表） | 已记录项通过；未记录漂移红 | 台账 + 校验输出 | tv/tt/tb/ui_gen/a2c/a2r 目录级 |
| V-07 | AC-06 红集对齐 | 同红/修复红/新红/无据消失/compile_blocked/not_run | work定向比较器；merge真实完整档 | 同红有证/消失有解释可接受；新增红、隐藏参考面、未执行计pass均红 | 分类表/诊断/执行收据 | 逐ID及档级阻断，限制分列 |

**角色与树**（Markdown 约定，非后端字段）：
实现：本计划实施会话。内部自检：同会话定向检查（pass 只表示可送终审，不替代终审）。独立终审：独立上下文复审。收尾：merge 在冻结三仓输入上执行三仓完整档与触面档。

**环境政策**：`th`（真 TCP）、`tn`（native 手工门）、`taa` Windows 跳过项按现状记录为 pending/跳过，缺环境标 pending 且不得算 pass；重型族（gallery 围栏、1M churn、aavm XL）在机器级单实例锁下运行，遵守 AGENTS 并行纪律。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | V-01：主锚+见证+不可执行参考面的义务总账可复算；th不移出；两次枚举/并集一致，来源差额逐项归因 |
| AC-02 | work | V-02/V-03：单入口三仓完整档可运行；含 man `t/tc/tcore/ti` 与 UI `tu/ti/tn`；零选择/缺 owner/无效 root/子门失败均非零并留收据（绑三仓 HEAD/锁/diff） |
| AC-03 | work | V-04：完整总账归属未解释=0；四类处置和执行状态分离；8,315历史子集与th等额外义务对账，不以RETIRED代表编译红/未执行 |
| AC-04 | work | V-05：三仓每档选择数下限守卫在位且灵敏；语言 `tu` 已删除或转发并在规范中注明 owner |
| AC-05 | work | V-06：金样/语料变更台账完整；无未解释漂移；842 CSS 金样等既知项有裁定引用 |
| AC-06 | work | V-07比较器及红集证据可审；新增红/无据消失/隐藏编译红/未执行冒充pass反控拒绝；既存红来源与owner明确；真实整档merge pending |
| AC-07 | review | 独立终审绑定三仓冻结 commit/依赖/配置/输入，核 V-01..V-07 与 SD-01..SD-04、遗漏/警告/格式/debug 扫描通过；merge-owned 冻结门显式 pending |
| AC-08 | merge | final review后冻结组合实际完整档收据齐全，新增红=0、消失红有解释；必需正控未跑则阻塞；既存红须review逐项认证并披露，不改原始失败码；SD/ledger/归档/guard闭环才U867，不冒称全集全绿 |

## 8. 执行步骤

11项保持ID：9 work+1 review+1 merge；G867=U844已由r1核验，沿用原树。r2重开T-01/02/04/08验收，保留历史实现证据；T-03/06完成声明保留，当前2/11，其余待闭合。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | G867 | r2重开验收：保留已枚举8,315和r1见证证据，补th等阻断面义务、首次失败来源、并集去重和V-01；未知阻塞 | AC-01 |
| [ ] | T-02 | work | G867；T-01 | r2重开验收：保留15逐ID红与3档编译红，补首次引入/拆分起点/当前状态/owner，更新red-baseline.md，不以同红自动认证 | AC-06 |
| [x] | T-03 | work | G867；T-01 | 扩 `scripts/run-full-regression.py`：三仓 root 解析/校验 + man 四腿与 UI 三腿 + 收据扩展；保持 `tf` 原语义。验证：`cargo tf --batch --list-only` 全腿枚举非零选择；负控见 V-02/V-03[✅ afc08a870：11 腿固定序+bootstrap+零选择/下限/失败传播/失败 ID 提取；list-only r4 冻结组合全绿；单测 25 绿含 V-02/V-03 控件；--frozen-trees 接口] | AC-02 |
| [ ] | T-04 | work | G867；T-01,T-03 | r2重开验收：closure完整义务总账、归属/执行分离，重算th与重叠；r1 8,315闭合作子集历史；V-04与隐藏参考面负控通过 | AC-03 |
| [ ] | T-05 | work | G867；T-01 | `scripts/check-tier-selection-minimums.py` + `scripts/tier-selection-minimums.json.txt`；语言 `tu` 别名处置（删或转发）+ 规范注明。验证：V-05 | AC-04 |
| [x] | T-06 | work | G867；T-01 | 金样/语料台账 `attachments/867/golden-ledger.md` + 校验脚本；覆盖 tv/tt/tb/ui_gen/a2c/a2r/gallery 围栏、1M churn。验证：V-06[✅ afc08a870：split-golden-ledger.py 逐文件提交归因，99 变更全 ROUTE_ADJUDICATED，0 未解释；重生成通道不存在（脚本无再生入参）] | AC-05 |
| [ ] | T-07 | work | G867；T-02,T-03,T-04 | 完成比较器/具名限制证据及V-07定向控制；整档仍pending，交final review后T-11统一执行 | AC-06 |
| [ ] | T-08 | work | G867；T-03..T-07 | r2重开验收：保留r1已写SD，刷新双维总账/限制报告的SD-01..04 proposal与handoff再冻结；canonical仅merge发布 | AC-03,AC-05 |
| [x] | T-09 | work | G867；T-08 | 全 AC/SD/V 自检与遗漏扫描：把每条 AC 绑到实际命令与输出；未闭合项显式列为 finding 或 pending[✅ handoff.md AC→证据映射齐；AC-01..05 达成、AC-06 阻塞（F-867-NEWRED-01）、AC-07/08 pending 归 review/merge；债务登记 D-867-TH/D-867-BASELINE-GAP/F-867-NEWRED-01] | AC-01..AC-06 |
| [ ] | T-10 | review | G867；T-09 冻结提交 | 独立完整复审（V-01..V-07 重放 + SD 校验 + 三仓实际运行证据）；不跑整档替代证据；merge 门 pending | AC-07 |
| [ ] | T-11 | merge | G867；T-10 pass | 并集复审、冻结实际完整档核AC-08；必要修复后复审再冻结，SD/ledger/归档/guard闭环才U867，未跑腿不得隐藏 | AC-08 |

**实施树（沿用既有组，不另建第二组）**：`D:/autostack/.wt/lang-867/{auto-lang,auto-ui,auto-man}`，每仓一树、分支 `plan-867-dev`；auto-down 只在需要三仓组合消费时以 detached 固定版本解析，不改实现。禁止 junction/symlink；清理前逐树 `bash D:/autostack/wt-guard.sh <path>` 必须 clean。

**门禁分级**：本计划只改脚本/配置/文档（Category C，外加 `.cargo/config.toml` alias 与 `.config/nextest*.toml` 读面）。开发期用 `python -m unittest discover -s scripts -p 'test_full_regression.py'` 与新增脚本单测；不跑 `cargo t/tf` 整档；merge 在冻结输入上跑三仓完整档 + 触面档。

## 9. 复审记录



本轮合同预检（非实现复审/测试）：章节0–10、单一任务表、任务DAG无环、编号唯一、AC/V/SD引用及frontmatter依赖检查通过；本Plan 11任务/8 AC/7 V/4 SD。来源内容hash：`docs/specs/auto-lang/project.md` sha256 `f4806f2975c0e5a338c715e397446346ae83268f1f464a7280d994f1b4ba17d0`；`docs/specs/auto-ui/project.md` sha256 `ff19acaf953b6fe5b5f08fc436dcc74aa968114e2e8ca04ab2e2a6be79974bb0`；`docs/specs/repository-boundaries/man-three-repository-delivery.md` sha256 `742588b634bd5ccc73a3a354d9d612fa92710ad3df2ad9177fb6e1040129209a`。仅文档校验，未运行Cargo/docs_gen。

2026-10-28 / stage: new / plan_id: PLAN-867 / plan_revision: 2 / outcome: pass（合同可执行，非实现/交付通过） / authorization: 用户本会话授权修订四Plan及总路线 / input_sha256: `158d8765017042dbbcc35a0bd11ed6801e4668e36e17f919e0855add0c2cb867` / sources: 当前auto-lang/project、auto-ui/project、man-three-repository-delivery Specs及scripts/run-full-regression.py；r1事实见下方历史 / changed: AC-01/03/06/08、V-01/07、T-01/02/04/07/08/11和SD-01..04细化 / progress: executing，旧header current_step=8而实际勾选6；重开4项验收，按保留T-03/06计2/11，未撤销其代码/原证据 / next: 原树work补T-01/02/04及未完成项，独立review后merge实跑整档。r1“th不入分母”“失败集合完全相同”“8,315即全集”和送审前整档要求仅作历史；当前不构成U867。

本轮只改规划，不跑Cargo/docs_gen、不改产品/canonical/ledger、不建树/agent、不推远端。原始探针/日志/截图留本地，摘要md记命令/退出码/数字/本地路径+hash；长期基线与必选ID若作为机器验收输入须受管于scripts，不借此提交整份原始证据。

2026-10-28 / stage: new / plan_id: PLAN-867 / revision: 1 / outcome: **pass（合同可执行；实施等 U844）** / next: 844 T-12 归档后 work T-01 / changed: 新建 T-01..T-11、AC-01..AC-08、V-01..V-07、SD-01..SD-04。

- 本轮只在语言主检出写合同：未建实施树、未跑任何档位/枚举、未改三仓实现、未改 canonical Specs/ledger、未推送远端。
- 取号来源：`scripts/new-plan.sh split-equivalence-test-closure`（master 主检出，原子取号 867 → 868）。
- 已知未决（不阻塞 drafting，阻塞对应任务）：§10 D-1..D-6。其中 D-1（红集证据路径）由 T-02 作有界决策；D-2（语言 `tu` 处置）由 T-05 提交决策；D-3（基线文件位置）由 T-01 确认；D-4/D-5（`--batch` 语义与 regress 收据兼容）在 T-03 前需与 merge 侧对齐。

2026-10-28 / stage: work / plan_id: PLAN-867 / revision: 1 / G867=U844 已核验（844 archived 12/12，`docs/plans/archive/844-*.md`）→ status 翻 executing。

- 实施树（组 `D:/autostack/.wt/lang-867/`）：auto-lang@plan-867-dev（基面 master a0cbb1a21）、auto-ui@plan-867-dev（7095f53）、auto-man@plan-867-dev（873fa23）、auto-down detached@c39ac7f（旧树组内 `../auto-down` path 依赖所需，规范固定版本一致）、pre-split detached@464e2316b（只读枚举树）、witness detached@72eb741e8（见 F-14）。
- **work 发现 F-14（test-cases 门控破窗）**：464e2316b 上 `probe_adapter_tests` 的 `#[cfg(feature="test-cases")]` 与 `ac_mn_probe_tests` 的门控堆叠错位（818 T-02 bf16761c5 引入，847 c409c87b6 拆分后才修复），导致**所有不含 test-cases 的档**（tf/tb/taa/ta/t3/th/tu）在该提交编译红；窗口内（10-11→10-26）无任何批量收据，破口从未被观测。处置：按 §2 判定原则记录为"该提交的档级既存红"，这些面改在**最后可编译见证提交 72eb741e8**（bf16761c5^）枚举——窗口内测试代码差异仅 ac_mn_probe_tests.rs（test-cases 门内，已被钉定提交的 t/tt 面覆盖），并集完备性可证。
- **work 发现 F-15**：ui-tn 面在钉定提交同样编译红（`fastrand` dev-dep 泄漏，831 parity tn_note 已注记；修复落在拆分后 auto-ui 仓）→ 记为档级既存红；其 2 个测试为当前侧新增（无拆分前面，无丢失）。
- T-01 初步数字（钉定提交 9 面成功 / 8 面编译破记录在案）：t 6,920/5,368、tv 2,160/163、tt 7,540/5,740、ui-t 3,282/2,396、ui-ti 43/28、ui-tu 863/863、man-t 284/280、man-tc 256/252、man-ti 43/43；universe 8,115 ids。与 831 parity 对上：ti=28 ✓、tu=863 ✓、ui-t≈2,396（parity 2,394，窗口内 +2 待对账归因）。见证树 7 面（tf/tb/taa/ta/t3/th/tu）枚举进行中。
- T-03/T-05 设施已写（run-full-regression.py 三仓 batch 扩展 + --frozen-trees + tier-selection-minimums + check-tier-selection-minimums.py + 聚焦单测 25 绿）；T-04 closure 脚本与 T-06 金样台账脚本已写（金样台账初跑：99 变更全归因、0 未解释）。
- **work 发现 F-16（844 退休落盘晚于最后批量收据）**：844 T-04 R-4/R-5/R-8/R-6（1159f058d，10-28 12:08）退役了 850 冻结基线中的 7 个 language 必选 ID（database/scope/infer::registry 族），而最后批量收据（860，10-28 09:33）先于此落盘——**主检出下一次 tf 批量回归将在 language 腿基线校验红**（tf 非逐计划门禁故未被发现）。处置：新增证据退休通道 `scripts/full-regression-retirements-844.json.txt`（850 文件不动，orchestrator/closure 读该清单移除必选项，逐行带计划引用），T-04 闭包记 RETIRED。
- **work 发现 F-17（th 档跨拆分双侧编译红）**：th（test-http-e2e）自 814 T-02 大迁移（afe0f9156，10-20）起编译红——vm/ffi 迁入 auto-lang-core 后其 `#[cfg(test)] pub(crate)` 项跨 crate 不可见（plan730/781 的 e2e 内联 mod 引用），钉定提交/见证树/当前 master 三处同红。处置：th 面在最后可编译提交 09a756e84 枚举为**参考面**（`pre-split-universe/reference/`，不入分母——拆分前后同态=既存红非拆分致丢失），当前侧源码仍在（测试函数未删），档修复登记 KNOWN-DEBT。
- T-01 定稿：**universe = 8,315 ids**（钉定树 9 面 + witness 72eb741e8 六面 tf 6,809/6,291、tb 6,864/5,316、taa 6,926/6,287、ta 7,628/6,741、t3 7,628/6,742、tu 868/868；ui-tn 全前拆期不可编译=F-15；th 参考面另存=F-17）。
- T-04 闭包初判（r4 list-only 收据 + cross-repo faces + ui-t json 面）：**covered 8,287 + MIGRATED 21 + RETIRED 7 = 8,315，LOST=0，新增 128**——闭包成立（终版绑 T-07 冻结收据重跑）。
- T-03 验证：`tf --batch --list-only` r4 冻结组合全 11 腿枚举通过（language 3,705/3,205、ui-full 3,135/3,117、tt 4,490/3,569、tb 3,759/3,095、man-t 285/281、man-tcore 23/23、man-tc 256/252、man-ti 43/43、ui-tu 863/863、ui-ti 43/28、ui-tn 2/2）；零选择/下限/失败传播控件在聚焦单测 25 绿。
- T-05：语言仓 tu 别名实测 0 选择（`-- --list` = 0 tests）后已删除（.cargo/config.toml + AGENTS.md 注记 owner=auto-ui）；守卫冻结组合实测全绿（man 281/23/252/43、ui 863/28/2、lang-t 3,198≥3,192、lang-tv 163）。
- T-08：SD-01（auto-lang/project.md 三仓批语义+退休通道+tu 处置）、SD-02（auto-ui/project.md 三腿+地板判据）、SD-03（man spec 纳入跨仓门）、SD-04（新 split-equivalence-acceptance.md）草案已在 worktree 落稿。


2026-10-28 / stage: work / plan_id: PLAN-867 / plan_revision: 2 / outcome: **blocked（F-867-NEWRED-01，待用户裁定修复路径）** / worktree_path: D:/autostack/.wt/lang-867/auto-lang / branch: plan-867-dev / base_commit: a0cbb1a21 / code_commit: afc08a870→688d35892（冻结）+ docs 终化提交 / dependency_revisions: auto-ui 7095f53、auto-man 873fa23、auto-down c39ac7f（detached 只读）/ task_ids: T-01..T-09 / evidence: attachments/867/{red-baseline,closure,golden-ledger,handoff}.md + scripts/pre-split-universe + r4 list-only 收据 + r2/r4 实跑收据 + t07-remaining-legs / blockers: F-867-NEWRED-01（AC-06 新增红=1） / next_responsible_role: 用户裁定 → 修复（fix 计划或授权扩scope）→ 重跑 T-07 冻结组合 → review

- AC-01..AC-05 达成；AC-06 判定=新增红 1（等价出口未达成）；AC-07/AC-08 pending（未送终审——开放合同内 finding 不交终审裁决）。
- 阻塞项处置选项（供用户裁定）：A) 新建 fix 计划修复 app_registry/VM 列表读缺陷后回接 T-07 重跑（推荐——证据指向真缺陷）；B) 若裁定 1e4b2b315 语义为有意且 4 为正确计数，则需 PLAN-050 侧正式裁定+oracle 更新证据后再判（当前证据不支持：无数据源给出 4）；C) 接受阻塞记录、先审已完成面。
- 其余异常均归因：clipboard 族=负载 flake（单跑绿）；tt 27 例 ac_*=冷缓存负载 flake（代表单跑绿）；r2 pickcat 绿=未解释异常（不推翻 6+ 次含隔离冷树的红）；r3 ffi_dual_018=负载超时。

2026-10-28 / stage: work / plan_id: PLAN-867 / plan_revision: 2 / outcome: **pass（execution_done，送终审）** / worktree_path: D:/autostack/.wt/lang-867/auto-lang / branch: plan-867-dev / base_commit: a0cbb1a21 / code_commit: 3f8b48f52（冻结审基：afc08a870→688d35892→d43a9f5c9→7105b0cf3→3f8b48f52） / dependency_revisions: auto-ui 7095f53、auto-man 873fa23、auto-down c39ac7f / task_ids: T-01..T-13 / blockers: 无（F-867-NEWRED-01 已再定性+修正+四跑验证关闭） / next_responsible_role: review（T-10）

- **等价出口达成**：closure LOST=0（8,315 分母全账）+ 金样台账 0 未解释 + 守卫全绿 + 红集对齐新增红=0（形式判定）+ **r8 三仓完整档一次全绿（outcome=pass）**。
- Phase 2（T-12/T-13）：pickcat 再定性（双侧对照探针=输入漂移+陈旧 oracle，非拆分回归；初判 bisect 被推翻）；oracle 意图保持式修正 7105b0cf3；r5-r8 四跑 language 全绿、r8 全 11 腿全绿。
- merge-owned pending（显式）：冻结三仓组合重放三仓完整档（T-11）、裸 cargo t + 触面档（本计划改动面=脚本/配置/别名/文档，无 Rust 产品码；oracle 修正为测试码——按 Category C+测试改动，merge 统一门执行）、SD/ledger 沉淀、归档与逐树 guard-clean 清理。


2026-10-28 / stage: work / plan_id: PLAN-867 / plan_revision: 2 / **用户裁定 A'（在现有计划加 phase，不另立计划）**：修复 F-867-NEWRED-01 属本计划 Phase 2（T-12 修复 + T-13 回接 T-07 重跑），授权边界见 T-12 行；status 保持 executing。

## 10. 待澄清事项

| ID | 事项 | 影响 | 处置 |
|---|---|---|---|
| D-1 | 红集证据走"复用历史批量收据"还是"旧树定向重跑" | T-02/T-07 成本与可证性 | T-02 作有界决策并留 artifact（优先复用；不可证未变则重跑受影响的腿） |
| D-2 | 语言 `tu` 别名：删除还是转发到 auto-ui | AC-04 | T-05 先实测 `-- --list` 为 0 再决定；默认建议删除并注明 owner |
| D-3 | 拆分母基线文件放 `scripts/pre-split-universe/` 还是平铺 `scripts/full-regression-*` 命名 | T-01 结构、后续可发现性 | T-01 确认；倾向独立目录避免与 850 既有文件混淆 |
| D-4 | `tf --batch` 语义从两 workspace 扩到三仓，是否影响既有 `/auto-plan:regress` 收据 schema 与到期判定 | T-03/merge | 扩为向后兼容：新增字段不删旧字段；旧收据仍可读；T-03 前与 regress 侧确认 |
| D-5 | 是否把 man 腿也并入无 `--batch` 的 `cargo tf` | per-plan 门成本 | 建议不并入（保持 850 交付语义），只在 `--batch` 生效 |
| D-6 | th/tn、编译红与Windows跳过 | AC-01/03/06/08 | r2明确保留义务/状态；T-01/02补来源，T-10审既存红，T-11实跑披露；未知/必需正控缺失阻塞，不新增豁免 |

G867 未满足前禁止建树/跑档/改实现。若实施中发现"拆分前全集"本身不可重建（例如旧树无法构建），须先提交具体 `needs_replan`，不得以缩小分母片面宣布等价。

