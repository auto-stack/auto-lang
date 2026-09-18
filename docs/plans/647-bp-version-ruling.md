---
plan_id: PLAN-647
status: execution_done         # drafting → executing → execution_done → reviewed → archived
feature_name: bp-version-ruling（blueprints 多版本/锁面裁定——P639-D3 收口）
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/blueprint/contract.md#Q5-版本面-MVP-半句
new_spec_components:
  - docs/specs/blueprint/contract.md#Q5-版本面规则（终版）
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [blueprint]          # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 5
total_steps: 5
---

# [PLAN-647] bp-version-ruling —— blueprints 多版本/锁面裁定（P639-D3 收口）

## 0. 变更摘要

P639-D3 悬置债收口：**blueprints 包库的版本语义正式裁定**。现状是"三无 + 两隐患"：
无锁面（`dep bps { path }` 纯路径、无版本约束）、无注册表语义（解析走
`resolve_module_path`，`lib.rs:2768`）、无版本字段消费；但 ① `blueprints/pac.at`
已带 `version: "0.1.0"`（库级元数据存在、无人消费、语义未成文）；② bp 的
`spec.md` frontmatter 若写 `version` 键会被 serde **静默忽略**（`BlueprintSpec`
无此字段且不拒未知键，`bp/spec.rs:14-39`）——半吊子版本化的两个口子都开着。

本计划是**裁定型计划**（默认 Option A，见 §5.1）：维持单版本 MVP 并成文终版规则
+ 落两个防半吊子护栏 + 登记"何时升级锁面"的触发条件，为 auto-down PLAN-070 的
消费侧铺开扫清 P639-D3 前置。**默认零机制新增**（锁面/lock 工件为 Option B，
仅在用户明裁时启用，工作量另计）。

## 1. 目标

- **G1 版本面规则终版成文**（contract.md Q5）：单版本语义、升版重构建语义裁定、
  pac.at `version` 字段定性（仅元数据，非约束面）、跨仓 worktree 对齐规则、
  锁面引入触发条件——五项一次裁齐。
- **G2 防半吊子护栏**：`spec.md` frontmatter 显式拒绝 `version` 键（报错并指向
  裁定）；`dep` 声明的版本键行为按裁定落定（默认：明确报错，防止"看起来有版本
  约束"的假象）。
- **G3 消费方影响评估在案**：046 fixture / PLAN-070（auto-down）/ bps-gallery /
  CI 对"升版"的实际反应（重构建范围、对齐方式）盘点成文，作为裁定的证据底座。
- **G4 P639-D3 核销 + 070 移交记录**：债核销；裁定摘要与 070 侧承接事项（跨仓
  对齐落点）登记，**不越仓改 auto-down 文件**。

### 非目标

- 不实现锁面/lock 工件/版本注册表（Option B 机制面，默认不启用）。
- 不改 `resolve_module_path` 解析序（连字符 key 不可达问题是另一条债
  KNOWN-DEBT:90，归后续 L1 组装样板计划收）。
- 不做 blueprints 独立仓化/发版（仅成文触发条件）。
- 不动 bind 工件 GENERATED 登记行格式（现状已含来源+日期，评估后如需 content
  hash 仅记录为 Option B 附件，不默认做）。

## 2. 架构方案

裁定型计划：**文档裁定为主体 + 两个解析层护栏 + 债务核销**。

- **Option A（默认，推荐）**：blueprints/ = 单调滚动的权威源。消费方（046、070
  的应用、gallery）经既有解析序拿当前工作区内容；**升版即全体消费方下次构建
  重构建**——接受，理由：消费方全部在 autostack 仓族内，解析序（env → 组内
  `../auto-lang` → 主检出）+ master CI 全绿门禁已是事实上的对齐机制；跨仓场景
  的可复现性由 **git 层面对齐**（消费仓 CI pin auto-lang commit / 同 commit
  检出）承载，不进 bp 层。pac.at `version` 定性为库展示元数据。
- **Option B（仅用户明裁时启用）**：最小锁面——`dep` 支持 pin（commit/content
  hash）+ lock 工件 + 漂移告警，动 `resolve_module_path` + pac schema + CLI，
  中等工作量，另立执行段。
- 护栏与裁定互为表里：**在锁面语义被正式裁定引入之前，任何版本面字段都必须
  显式失败而不是静默忽略**——这是防止"假版本约束"漂移进生态的最低成本手段。

## 3. 技术栈

- 裁定面：Markdown spec（contract.md Q5 行内规则 + PLAN-070 移交记录）。
- 护栏面：Rust 解析层小改（`crates/auto-lang/src/ui_gen/bp/spec.rs` 未知键
  处置 + `lib.rs` pac 解析的 dep 键面，T-01 实读后定精确落点）+ 单测。
- 验证：`cargo check -p auto-lang`、scoped `cargo t`（bp/spec 测试）、
  `auto bp check` 冒烟（全 13+ 包不回归）。Category B。

## 4. 需求分析与背景调查

### 授权记录

- 2026-09-18 会话：用户确认起草"锁面裁定"计划（此前会话排序表 ③，原拟 645，
  因 645/646 被占用顺延为 **PLAN-647**）。仓库=auto-lang 单仓；worktree
  `D:/autostack/.wt/lang-647/auto-lang`。
- 时序前置：**T-02（contract.md Q5 编辑）须在 PLAN-643 合并后进行**（643 SD-02
  同文件不同节——验证面节 vs 本计划 Q5 打包行；串行沉淀避免同文件并发）。643
  review 已过、正在 merge，正常时序下自动满足；若执行时 643 尚未落地，T-02
  等待、T-01/T-03 可先行。

### 证据（路径实勘，2026-09-18）

- **债源**：KNOWN-DEBT:2262 P639-D3——"PLAN-070 消费侧铺开前须裁定多版本/锁面
  机制（升版即全体消费方重构建的语义是否可接受、跨仓 worktree 场景的版本
  对齐）"。
- **现状三无**：`blueprints/pac.at` = `name: "bps"` 纯 path 库声明；046 消费面
  `examples/capability-tests/046-bp-import/pac.at` `dep bps { path:
  "../../../blueprints" }` 无版本键；解析链 `crates/auto-lang/src/lib.rs:2768`
  `resolve_module_path`（base_dir→父目录→逐级 deps/path dep 探测）。
- **隐患①**：`blueprints/pac.at` 已带 `version: "0.1.0"`——存在但无消费方、
  无门控，语义未成文。
- **隐患②**：`BlueprintSpec`（`crates/auto-lang/src/ui_gen/bp/spec.rs:13-39`）
  无 `version` 字段且 serde 默认不拒未知键——spec.md 写 `version = "2"` 会被
  静默忽略（负空间实验可在 T-01 用一个 fixture 确证）。
- **bind 登记面现状**：`046 .../bps/login_bind.at` 头注 = "Source:
  blueprints/form/login @ minimal (2026-09-17)"——有来源与日期、无 content
  hash/commit pin。
- **消费方清单（T-01 完成全名单）**：046 fixture；auto-down PLAN-070（jade-garden/
  musk/widgets-gallery 收敛为 L1，其 pac.at 已起草）；`examples/bps-gallery`
  （glob 直读磁盘，构建即取最新）；CI 面 `build-bps-gallery.yml`（paths 含
  `blueprints/**`——升版触发 gallery 重构建的现成实例）。

### T-01 实勘结论（2026-09-18，worktree plan-647-dev@master=1f4e3e32c）

1. **解析序与 dep 键面现状**：`resolve_module_path`（lib.rs:2768-2921）=
   external back root → base_dir probe → 父目录 probe → 向上 4 级（deps/
   声明门控 + workspace members + pac.at 文本式 path dep）。dep 声明解析是
   **文本式**（`dep "name"`/`dep name` → 只找 `path:` 取行尾）——版本键现状
   =**静默忽略**；仅写 `version:` 无 `path:` 时报的是下游 unresolved 类错误，
   不指向版本键（风险实锤：假约束无诊断）。fixture 实测由
   `plan647_bp_version_tests::resolve_module_path_*` 永久钉住。
2. **隐患②运行期确证**：`BlueprintSpec::parse` 走 `toml::from_str` 无
   deny_unknown_fields，frontmatter `version` 键被 serde 静默丢弃——红测试
   `spec_frontmatter_version_key_is_rejected`（实现前 FAIL、实现后 PASS）
   即确证记录。
3. **护栏 A 落点定夺**：**不用 `deny_unknown_fields`**（frontmatter 未来可能
   进合法键如 `promotions:`——契约变体提升评审流程；且 `[dataSource]` 表内
   `version` 槽名是合法 fetcher 签名），改用 parse 前置 `toml::Table` 顶层
   键定点扫描。现网 14 包 frontmatter 键面实勘（awk 全量 tally）：全部落在
   已知字段/`[dataSource]` 槽内，无版本键。
4. **护栏 B 落点定夺**：`pac_dep_version_violation` 纯文本扫描 helper
   （词首边界防 `prev:`/`versions:` 误伤；按 dep 名只扫本声明 `{}` 块）+
   `resolve_module_path` 内 dep_declared 后接线：违规=显式 eprintln（指向
   Q5）+ return None **硬失败**（即使 path 本可解析——防假约束；错误通道
   与 ghost-dep eprintln 先例同型，lib.rs 返回 Option 签名不动以控制波及面）。
   dep_scanner.rs 的 `dep serde(version:)` 是 Rust crate 级另一形态，不属
   pac.at 包声明面、不在护栏范围。
5. **消费方"升版反应"盘点**：046——blueprints/** 变更下次构建重解析重构建；
   bps-gallery——无 pac.at，Vite `import.meta.glob('../../../blueprints/*/*/')`
   构建即取最新，CI build-bps-gallery.yml paths `blueprints/**` 触发重构建；
   auto-down PLAN-070——起草中（auto-down master d8f11bf 无成品 pac.at），
   跨仓对齐=git 层 pin。**"升版即全体消费方下次构建重构建"现状即成立**，
   CI paths 是现成对齐机制实例。
6. **pac.at `version` 消费面**：`AutoConfig::version()`（config.rs:116）全仓
   **零调用点**；`auto bp list` 不读 pac.at 不显示 version（cmd_bp.rs list
   只打 kind/name）——"展示元数据可示未示"，本计划零机制不接展示。
7. **worktree/依赖记录**：base=1f4e3e32c；worktree `D:/autostack/.wt/lang-647/
   auto-lang`（plan-647-dev）；依赖兄弟 worktree `D:/autostack/.wt/lang-647/
   auto-down`（detached@auto-down master d8f11bf，只读；cargo workspace
   path dep `../../../auto-down/...` 组内兄弟解析所需）。

### 风险

- 护栏②（dep 版本键报错）若现状是"解析错误已存在"则改为成文现状；若现状是
  "静默忽略"则升为显式报错——两种都小，但需 T-01 实测定方向。
- 070 在 auto-down 侧活跃推进，裁定晚落一天=070 多一天按 MVP 假设裸奔——本计划
  宜快（文档主体可先行合并节奏不受 643 卡）。

## 5. 详细设计

### 5.1 裁定正文（T-02 落 contract.md Q5 行，五项）

1. **单版本语义**：`blueprints/` = 单调滚动权威源；同 一时刻每包仅一个有效
   版本；历史由 git 承载（无目录内多版本共存）。
2. **升版语义**：包内容变更=新版本；全体消费方**下次构建即重构建**（无缓存
   失效协议、无兼容窗口承诺——breaking change 以 spec 契约字段演进 +
   gotchas 登记）；接受理由与对齐兜底=解析序 + master CI 门禁。
3. **pac.at `version` 定性**：库级展示元数据（`auto bp list` 可示），**非约束
   面**——消费方不得据此做版本判断；维护规则：随目录级演进手工递增，不承诺
   语义化。
4. **跨仓对齐规则**：消费仓（如 auto-down）以 git 对齐——CI pin auto-lang
   commit 或同 commit 家族检出；**禁止**在 bp 层自造 pin/lock 直到 Option B
   正式立项。
5. **锁面触发条件**（何时升级 Option B）：出现仓族外消费者 / blueprints 独立
   仓化或独立发版节奏 / 消费方需要长周期不跟随 master 的冻结构建——三者任一
   满足即立项。

### 5.2 护栏设计（T-03）

- **护栏 A**：`BlueprintSpec::parse` 拒绝 frontmatter `version` 键——实现形态
  T-01 定（候选：parse 后显式扫描原文本键集，或 serde `deny_unknown_fields`
  若确认无合法未知键用途），错误消息指向 Q5 终版规则。
- **护栏 B**：`dep` 声明出现版本类键（`version`/`pin`/`rev` 等）→ 显式报错
  （"版本面未启用，见 contract Q5"），防假约束。现状实测定改或成文。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/blueprint/contract.md（Q5 打包与解析行） | before："版本面 MVP=主检出单版本"一句带过；after：§5.1 五项终版规则整段落位 | P639-D3 收口；070 铺开前置；消除 pac.at version 与 spec 静默忽略两个语义空洞 | AC-01 |
| SD-02 | modify | docs/specs/blueprint/contract.md（验证面节，护栏句） | before：无版本键面验证规则；after：spec frontmatter `version` 键与 dep 版本类键=显式错误（指向 Q5） | 护栏与裁定互为表里：未裁定前禁止半吊子版本化渗入 | AC-02 |

（无 retire；P639-D3 核销与 070 移交为簿记。）

## 6. 测试设计

- **护栏负测试**（`bp/spec.rs` 测试模块 + pac 解析测试）：spec.md 带
  `version = "1"` → parse 错误且消息含裁定指引；dep 声明带版本键 → 构建期
  显式错误（或按 T-01 现状成文的既定行为断言）。
- **正测试不回归**：现存 13+ 包 `BlueprintRegistry::with_defaults()` 扫描 +
  `palette_drift` 全绿；`auto bp check` 全包通过。
- **文档一致性**：Q5 规则与护栏错误消息互指（复审 checklist 项）。
- 门禁：Category B（`cargo check -p auto-lang` + scoped `cargo t`；无 VM/
  编译器/文档生成器改动，不触发 tv/docs_gen/taa）。

## 7. 验收标准

- **AC-01 裁定成文**：contract.md Q5 含 §5.1 五项（单版本/升版语义/pac.at
  version 定性/跨仓对齐/锁面触发条件）。验证：spec diff 审读 + 复审 checklist。
- **AC-02 护栏生效**：spec frontmatter `version` 键 → 显式错误；dep 版本类键 →
  显式错误（或 T-01 实勘后成文的等价既定行为）。验证：负测试绿 +
  `cargo t plan647`。
- **AC-03 影响评估在案**：消费方清单与升版反应（重构建范围、对齐方式）记录于
  本计划 §4/§8 证据区。验证：T-01 产出审读。
- **AC-04 P639-D3 核销 + 070 移交**：KNOWN-DEBT P639-D3 行标核销（指向本计划
  与 Q5 终版）；070 承接事项（git pin 落点）登记。验证：DEBT 文件 diff。
- **AC-05 spec 沉淀**：SD-01/02 落地；merge 时 specs.json upsert +
  `python scripts/spec-index.py`。
- **AC-06（条件项，仅 Option B 被裁时生效）**：lock/pin 机制交付与测试。
  默认 A 形态下本 AC 记"未启用"于复审记录——**不静默删项**。

## 8. 执行步骤

> worktree `D:/autostack/.wt/lang-647/auto-lang`（分支 `plan-647-dev`）；
> plan 簿记留主检出。全计划轻量，T-01 半日内可毕。

- **T-01 [有界调查] 现状盘点与隐患确证**（0.5d）[x]
  无依赖。操作：①实读 `resolve_module_path`（lib.rs:2768 起）解析序与 dep 键
  面现状（版本键现在是报错还是忽略——fixture 实测）；②spec frontmatter
  `version` 键静默忽略负空间实验确证；③消费方全名单盘点（046/070 pac.at
  草案/gallery/CI paths），每家"升版反应"记录；④护栏 A 实现落点定夺
  （deny_unknown_fields 可行性）。产出：§5.1 证据补全 + §5.2 两护栏的精确
  落点。验证：结论与 fixture 输出记入本节。→ AC-02/03 前置
  [✅ 已完成] 七项结论记于 §4"T-01 实勘结论"（2026-09-18）；红测试先 FAIL
  后 PASS 即隐患①②运行期确证；deny_unknown_fields 否决（promotions/
  dataSource 槽名保护），定点扫描落地。
- **T-02 裁定落文**（前置：PLAN-643 已合并）[x]
  影响：`docs/specs/blueprint/contract.md` Q5 行（SD-01）+ 验证面护栏句
  （SD-02）。验证：diff 审读，五项齐备。→ AC-01
  [✅ 已完成] 643 已归档（1f4e3e32c）时序解锁；Q5"版本面 MVP"半句→五项
  终版整段 + 验证面节"版本键面护栏"句（与护栏错误消息互指 Q5）；
  commit bf3573851。
- **T-03 护栏落地**：`bp/spec.rs` 拒 version 键 + dep 版本类键显式错误（落点
  按 T-01）+ 负/正测试。[x]
  验证：`cargo check -p auto-lang` 零警告；`cargo t plan647` 绿；
  `auto bp check` 全包冒烟不回归。→ AC-02
  [✅ 已完成] 护栏 A=spec.rs `VERSION_KEYS` 定点拒（[dataSource] 槽名不误伤）
  + 护栏 B=`pac_dep_version_violation` + resolve 硬失败；`plan647_bp_version_
  tests` 9 测试（负/正对照/词首边界/集成面）+ spec.rs 1 负测试全绿；改面
  零新增警告；commit bf3573851。
- **T-04 债核销与 070 移交记录**：KNOWN-DEBT P639-D3 核销行；070 承接事项
  （跨仓 git pin 落点、MVP 假设转正）登记于本计划复审区与债文件。[x]
  验证：DEBT diff。→ AC-04
  [✅ 已完成] KNOWN-DEBT 新增"2026-09-18 增补三（PLAN-647 核销与移交）"节：
  P639-D3 ✅ 核销 + 070 三项承接事项（不越仓改文件）；commit 92bd47a7c。
- **T-05 门禁与复审兜底**：门禁档位复核（Category B）、全量 `auto bp list/
  check` 冒烟输出留档。[x]
  验证：全 AC 兜底。→ AC-03/05
  [✅ 已完成] Category B 复核：改动=解析层小改+文档，无 VM/编译器/docs_gen/
  aavm 面，不触发 tv/docs_gen/taa。门禁实测：`cargo check -p auto-lang` 过
  （改面零新增警告，存量为仓基线）；`cargo t plan647 rejects_frontmatter` 9/9
  绿；`cargo t ui_gen::bp` 9/9 绿（含 14 真包扫描+palette 无漂移）；`auto bp
  list` 14 包全列；全包 `bp check` 循环 22 变体=16 过 6 败，与 master 基线
  二进制逐项一致（预存：reference 变体缺 loading/error 槽），零回归。

依赖链：T-01 → T-03；T-02 仅依赖 643 合并（与 T-01/T-03 可并行）；T-04/T-05
收尾。

## 9. 复审记录

- 2026-09-18 draft handoff（/auto-plan:new）：plan_revision 1，stage: new，
  outcome: pass（授权范围内可交付 work），next: work。
  待用户确认项见 §10（两项，均有默认裁定；Option A/B 为方向性默认，B 启用即
  触发 §7 AC-06 条件项与范围修订流程）。
- 2026-09-18 work completion（/auto-plan:work）：stage: work | plan_id:
  PLAN-647 | plan_revision: 1 | outcome: **pass** | code_commit:
  bf3573851（护栏+contract）+ 92bd47a7c（债核销）| base: 1f4e3e32c |
  worktree: `D:/autostack/.wt/lang-647/auto-lang`（plan-647-dev）+
  依赖兄弟 auto-down detached@d8f11bf | task_ids: T-01/T-02/T-03/T-04/T-05
  全毕（5/5）| evidence: §4 实勘结论 + §8 各条验证注记；`cargo check` 过、
  `cargo t plan647 rejects_frontmatter` 9/9、`cargo t ui_gen::bp` 9/9、
  `auto bp list` 14 包、`bp check` 全包循环与 master 基线逐项一致（预存
  6 败非本计划引入）| blockers: 无 | next: review（/auto-plan:review）。
  - **§10 裁定记录**：用户指示"实施"=授权按默认执行——①Option A（零机制
    新增）落地；②dep 版本类键=显式报错（硬失败形态）。
  - **AC-06（条件项）**：Option B 未被裁启用——lock/pin 机制**未交付**，
    按计划记"未启用"，非静默删项；触发条件成文于 contract Q5 ⑤。
  - **AC-05 部分（specs.json upsert / spec-index.py）**：属 merge 阶段动作
    （/auto-plan:merge），work 阶段仅完成 contract.md 规范增量本体（SD-01/
    02 已落 worktree 分支）。

## 10. 待澄清事项

1. **Option A vs B**（§2）：默认 A（维持 MVP 成文 + 护栏 + 触发条件）；若你
   判断 070 的消费规模/外部消费者已迫近，明裁 B（lock/pin 机制，工作量 +
   ~2-3 天，另含 AC-06 交付项）。
2. **dep 版本类键的处置**（§5.2 护栏 B）：默认显式报错；若倾向"告警不阻断"
   （容忍手写残留），T-03 一处参数化调整。
3. （记录性）pac.at `version` 递增纪律默认"手工、不承诺语义化"；若要
   `auto bp` 校验它与目录演进一致，属 Option B 附件，不默认做。
