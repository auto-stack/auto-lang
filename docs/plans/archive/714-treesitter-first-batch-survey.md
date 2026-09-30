---
plan_id: PLAN-714
status: archived
completion_kind: delivered
feature_name: tree-sitter 首批勘定件（auto-edit M4 供料包供④ 前半——语言集定界/管线选型+烟测/syntect 共存策略/增量高亮管线要点+实施件契约草案）
author: [agent]
created_at: 2026-09-30T16:15:39+08:00
updated_at: 2026-09-30T17:05:00+08:00
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md（SD-01：勘定契约——语言集表/选型决策记录/共存策略/增量管线要点/实施件边界）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（供料驱动面，703/710 先例注记式）
affects: [docs/plans/reports/714-treesitter-survey.md（新）, docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md（新）]
---

# [PLAN-714] tree-sitter 首批勘定件（auto-edit 供④ 承接前半）

## 0. 变更摘要

auto-edit **M4 解阻供料包供④**（`auto-edit/docs/upstream/
2026-09-m4-perf-unblock-supply.md` §4——tree-sitter 首批，2026-09-29
落档）的**承接前半件**：供料原文定**两段式**（「先勘定件定界[语言集/
管线选型/与 syntect 共存策略]，实施件另立」）——本件=勘定决策件。
它是 M4 长尾关键路径（同时清 M3 尾巴「语法高亮联动」与 M4 产出 4
「语法高亮首批」的唯一堵点；auto-lang 侧 705-713 均他域，供④ 至今
无人认领）。四勘定面：**①语言集定界**（常见 20 语言清单——下游
消费域现实[.at/Rust/web 族/配置族]×grammar 来源/许可/维护度表）→
**②管线选型+烟测实证**（tree-sitter crate 形态[统一 crate+feature
vs 子 crate 族]/grammar 分发形[build-time 编译 vs 预编译嵌入 vs
运行时加载]——判据=构建复杂度/体积[installer ≤15MB 联动]/增量 API
面；**镜像拉取+最小 parse 烟测**[703 imara 先例，spike 隔离不入
主线——spike-428-bench 先例]）→**③syntect/two-face 共存策略**
（双轨迁移[feature 并存按 lang 路由]vs 一步切换；two-face 退役
=installer 联动收益量化[供料档 §5 want 的生命周期止点注记]）→
**④增量高亮管线设计要点**（rope 快照消费/失效域[编辑路径增量重
高亮非全量重算——diff 后台任务先例同款形态]/big 态 plain 旁路保持/
与 plan047 memo 依赖录制协同注记）。产出=**勘定报告+实施件契约
草案**（715+ 立项基——任务骨架+AC 草案+验收设计）。**本件零
crates 改动**（烟测 spike 隔离）。

## 1. 目标

- **G-1 语言集定界**：首批语言清单（战略 §2.2「常见 20 语言起步」
  原文）——定界依据=下游消费域现实（auto-edit tab 面语言分布+
  战略口径）候选族：.at/Auto、Rust、Python、TypeScript/JavaScript、
  JSON、TOML、YAML、Markdown、HTML、CSS、C、C++、C#、Go、Java、
  SQL、Shell/Bash、Batch/PowerShell、INI/Properties（T-01 按下游
  现实定稿 20±）；每语言=grammar 来源（tree-sitter org 官方 vs
  社区）/许可（MIT 大宗+例外标注）/维护度（最近发布/CRATE 状态）
  三列表。
- **G-2 管线选型+烟测实证**：两轴定案——**crate 形态**（统一
  `tree-sitter` runtime+`tree-sitter-<lang>` 子 crate 族[生态主流]
  vs 单 crate feature 门——判据=依赖面/版本管理/构建复杂度）；**
  grammar 分发形**（build-time 编译[cc 链，生态默认]/预编译嵌入
  [体积入 exe]/运行时加载[dll 侧车——与「单 exe 无运行时依赖」
  战略语义冲突面注记]）——判据=构建复杂度/体积[installer 联动]/
  增量 API 面（edit/tree 重算面）；**烟测实证**：镜像拉取 runtime
  +2-3 代表 grammar（.at 邻接形[Rust/Python]）+最小 parse 用例
  （parse→node 树→高亮查询烟测）——可行性+体积初值（spike 目录
  隔离，不入主线 crates）。
- **G-3 syntect/two-face 共存策略**：现状=code-editor feature 全
  syntect 5/two-face 0.4（`two_face::syntax::extra_no_newlines()`
  全量内嵌，highlight.rs:135 实锚）+cosmic-text 自带 syntect 面；
  两案：**(a) 双轨迁移**（新 feature `highlight-treesitter` 并存，
  按 lang 路由——迁移期稳、体积双付）vs **(b) 一步切换**（首批
  语言集内切 tree-sitter，其余落 plain——体积单付、覆盖面收缩）；
  定案+**two-face 退役联动量化**（.rdata 12.4MB 主项域[019 构成
  表]——退役=installer 预算行收益数字；供料档 §5 want 生命周期
  止于本线注记）。
- **G-4 增量高亮管线设计要点**：rope 快照消费（编辑路径零锁——
  diff 后台任务同款形态）；**失效域**（编辑区间→受影响节点重算
  而非全量——tree-sitter 增量 edit API 消费形）；big 态 plain
  旁路语义保持（013——懒语法/lang plain 臂不受扰的边界成文）；
  plan047（依赖录制基建）协同注记——增量面是否消费其通道属实施
  件选型空间（供料原文注记承接）。
- **G-5 基准与验收设计**：上游基准（首批语言集高亮正确性 fixture
  [每语言金样本→token 化对照]+增量重高亮延迟[编辑→重高亮完成
  墙钟]）；下游回归面（auto-edit 矩阵语法面+bench 大文件装载墙钟
  不回退[plain 旁路域不变]——供料验收建议原文承接）。
- **G-6 实施件契约草案+落账**：勘定报告（docs/plans/reports/
  714-treesitter-survey.md）+**实施件契约草案**（715+ 立项基：
  任务骨架/AC 草案/SD 面预定——按四勘定结论成形）+SD-01 勘定
  契约册+specs.json P714-1 投影。

### 非目标

- **实施件**（tree-sitter 管线落地/语言集接入/增量面——715+ 另立，
  本件零 crates 改动[烟测 spike 隔离]）。
- 供②③ 消费与 two-face 子集独立清偿（§5 want 若在供④ 前被承接
  则独立演进——本件只注记生命周期协同，不并案）。
- LSP/符号索引/大纲面（L2 线——战略 §3 裁剪清单域外）；tree-sitter
  之外的语法引擎复评（选型域已由供料定为 tree-sitter——§4.2 内核
  路线）。
- 下游消费件（auto-edit 语法面矩阵/联动——实施件 delivered 后）。
- `.at` 语法 grammar 的自建（若语言集含 .at——tree-sitter grammar
  需自写，属实施件可选任务；本件只定界「含/不含+成本注记」）。

## 2. 架构方案

分层落点（2026-09-30 实勘，auto-lang master@05974f71d）：

| 面 | 现状 | 本期形态 | 依据 |
|---|---|---|---|
| 语法高亮 | code-editor feature=syntect 5+two-face 0.4+cosmic-text（Cargo.toml:69/255-257 实锚）；highlight.rs 两态构建（:135 extra_no_newlines 全量+LazyThemeSet） | 零改动（勘定域）——现状全貌+调用面清单=T-00 勘定首项 | 供料 §4「现势=全 syntect/two-face、零 tree-sitter 依赖」 |
| 体积联动 | two-face 全量内嵌=.rdata 12.4MB 主项（019 构成表） | 退役收益量化+§5 want 生命周期注记 | 供料 §4「体积影响评估（two-face 退役=installer 联动）」 |
| 增量面 | rope 快照消费先例在册（diff 后台任务/673 find_next）；无语法增量面 | 设计要点成文（失效域/edit API 消费形/big 旁路边界） | 供料 §4「增量高亮（编辑路径增量重高亮——rope 快照同款）」 |
| 并行域 | plan046/047（memo/keyed+依赖录制）在途 | 协同注记（正交性+消费空间——供料原文） | 供料 §4 协同注记 |
| spike | spike-428-bench 先例在册（docs/plans/spike-428-bench） | 烟测隔离同形（不入主线 crates/依赖） | 仓内先例 |
| 落账 | ui/design/ 册族（703/710 SD 先例） | SD-01 勘定契约册+报告+账本 | 703/710 SD 家族 |

**关键设计约束（frozen）**：
① **勘定不实施**（零 crates 改动——烟测 spike 隔离，证据入报告
不入主线依赖）。② 语言集定界以**下游消费域现实**为准（auto-edit
tab 面分布实勘——非上游闭门造车）。③ 选型判据可复核（每判据=
实证锚[烟测数字/构成表/许可实勘]——非偏好陈述）。④ 实施件契约
草案=建议性骨架（715 立项时以本件结论重起草——草案不预支授权）。
⑤ big 态 plain 旁路保持为硬边界（013 语义零扰动——实施件继承）。

## 3. 技术栈

勘定=文档+spike 烟测（cargo scratch 项目[隔离目录]+镜像拉取
tree-sitter crate+2-3 代表 grammar+最小 parse/查询用例——703
imara 镜像先例）；下游现实实勘=auto-edit 侧只读（矩阵/会话 fixture
语言分布——零改动）；报告+契约=markdown。无主线 crates/依赖变更。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-09-30 会话指令「OK，auto-plan-new 新建
计划 021；然后再新建 auto-lang 的相关计划」——授权=**起草本件**
（供④ 承接前半——两段式的勘定段）；执行/work 待用户另行启动
（auto-lang 会话——713 稿并行在途，启动时核 714 编号与建组）。
范围=auto-lang docs/spike（零 crates 主线改动）；auto-edit 零改动
（语言分布实勘只读）。无预算/自动续跑授权。

**来源与版本**：

- 供料包：`auto-edit/docs/upstream/2026-09-m4-perf-unblock-supply.md`
  §4 供④（2026-09-29 落档，PLAN-018 T-01；同日 019 SD-05 登记
  two-face 子集 want=§5——本件 G-3 生命周期注记的对接位）。
- 战略口径：auto-edit 战略 §4.2 内核路线（「学 Zed 的三课」——
  tree-sitter 化；Zed=tree-sitter 作者的编辑器）；§2.2 M2 清单
  「语法高亮：常见 20 语言起步（内核 tree-sitter 化后解锁）」；
  §6 M4 行「语法高亮首批（tree-sitter）」；M3 收口注记尾行（
  overview M3 第五件：「剩余语法高亮联动=M4 首批 tree-sitter——
  供料驱动」）。
- 现状实勘（auto-lang master@05974f71d）：Cargo.toml code-editor
  feature 面（:69/:255-257——syntect 5/two-face 0.4/cosmic-text
  0.15[vi+syntect]）；highlight.rs（:135 extra_no_newlines 全量
  内嵌+:144 LazyThemeSet）；零 tree-sitter 依赖（全树 grep）；
  plan046/047 在途注记（memo/keyed 渲染域——正交性勘定首项）。
- 下游实勘（auto-edit main@70c5c60）：013 大文件模式 big 态 plain
  旁路语义（懒语法/lang plain）；019 构成表（two-face .rdata
  12.4MB 主项域）；installer 预算行（29.8MB 分阶段+two-face want）。
- 历史关联：PLAN-673（rope 快照消费先例）、703（imara 选型烟测
  先例+SD 家族）、710（供① 承接件——两段式供料承接的完施先例）。

## 5. 详细设计

### T-00 现势复核（决策件首项）

highlight.rs 全貌（两态构建/调用面/token 流消费形[cosmic-text 侧
配合面]）；code-editor feature 依赖闭包（cargo tree——体积贡献
分解对齐 019 构成表）；big 态 plain 臂实锚（013 面——旁路边界
成文基）；plan046/047 正交性注记。

### T-01 语言集定界（G-1）

下游现实实勘（auto-edit 会话/矩阵 fixture 语言分布+战略 §2.2 口
径）→首批清单定稿（20±）；三列表（来源/许可/维护度）——许可
实勘为准（MIT 大宗预期+例外标注[如 tree-sitter-c-sharp 等]）。

### T-02 管线选型+烟测（G-2）

两轴判据表+**spike 烟测**（隔离目录 cargo scratch：镜像拉取
runtime+2-3 代表 grammar→parse 烟测[最小 .rs/.py 样本→node 树
断言]→高亮查询烟测[queries 目录→capture 流]→体积初值[依赖闭包
编译产物尺寸]）；分发形三案对比（构建复杂度/体积/增量 API/「单
exe」语义冲突面[运行时加载案注记]）——**定案+证据**。

### T-03 共存策略+体积联动（G-3）

(a) 双轨 vs (b) 一步切换定案（判据=迁移风险[矩阵语法面回归域]/
体积双付期/覆盖面[首批外语言落 plain 的下游影响]）；two-face
退役收益量化（12.4MB 域→installer 预算行数字联动）；§5 want
生命周期注记（供④ 实施件收口则 §5 独立清偿案并档）。

### T-04 增量管线要点（G-4）

rope 快照消费形（diff 后台任务同款——Send+Sync 只读）；失效域
设计要点（编辑区间→tree.edit 增量重算——非全量）；big 态旁路
边界成文（013 语义继承条款）；047 协同注记（消费空间——实施件
选型域）。

### T-05 基准/验收设计+实施件契约草案（G-5）

上游基准设计（正确性 fixture 族[每语言金样本→token 化对照]+
增量延迟档）；下游回归面（矩阵语法面+bench 装载墙钟不回退——
plain 域不变断言）；**实施件契约草案**（715+：任务骨架[feature
落位/runtime 接入/首批语言接入/增量面/退役面]/AC 草案/SD 面预
定/工期量级估计——按四勘定结论成形）。

### T-06 落账（G-6）

SD-01 勘定契约册（docs/specs/auto-lang/ui/design/
treesitter-highlight-survey.md）+勘定报告（docs/plans/reports/
714-treesitter-survey.md——四勘定结论+证据+spike 实录）+specs.json
reviews 段 P714-1 外科插入（703/710 先例）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md | before：语法高亮面=syntect/two-face 现状无专册（feature 面散于 Cargo 注记）；tree-sitter 化无契约 / after：勘定契约册——语言集表（20±×来源/许可/维护度）/管线选型决策记录（crate 形态+分发形+烟测证据）/共存策略（双轨 vs 切换定案+two-face 退役联动量化）/增量管线要点（快照消费/失效域/big 旁路边界）/实施件边界（715+ 契约指向） | 供④ 前半落账；实施件的内核侧真源 | AC-01..05 |

## 6. 测试设计

- **spike 烟测**（隔离，不入主线）：parse 烟测（2-3 语言最小样本
  →node 树断言）+高亮查询烟测（capture 流非空+类别抽样）+体积
  初值（依赖闭包产物尺寸记录）——证据入勘定报告。
- **勘定可复核性**：每判据=实证锚（许可=crates.io/仓库实勘；体积
  =构成表/spike 数字；正交性=调用面 grep）——复审可逐项重放。
- **主线零改动断言**：crates/ 与 Cargo.toml 零 diff（spike 目录
  隔离+gitignore 或不入 git——spike-428-bench 入档形态沿用的则
  入档）。
- **无运行时面**（勘定件零测试面新增——报告+册为交付物，审校=
  逐判据证据核对）。

## 7. 验收标准

- **AC-01 语言集表**：首批清单定稿+三列表（来源/许可/维护度——
  许可逐项实勘非假设）。验证：勘定报告 §语言集+表在档。
- **AC-02 选型决策记录**：两轴定案+判据表+spike 烟测实证（parse/
  查询绿+体积初值）。验证：报告 §选型+spike 实录（命令可复现）。
- **AC-03 共存策略定案**：双轨/切换定案+判据+two-face 退役量化
  数字+§5 生命周期注记。验证：报告 §共存+数字对照 019 构成表。
- **AC-04 增量管线要点**：快照消费/失效域/big 旁路边界/047 协同
  注记成文。验证：SD-01 册对应节在档。
- **AC-05 实施件契约草案**：715+ 骨架（任务/AC/SD 面/工期量级）
  在档。验证：报告 §实施件契约。
- **AC-06 落账+主线零改动**：SD-01 册+P714-1 投影回读 True+
  crates/Cargo.toml 零 diff。验证：账本断言+git diff 路径断言。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 现势复核 | — | highlight.rs+Cargo feature 面+046/047 注记 | 全貌+正交性基 | AC-03/04 | [x] 勘定记录在档——报告 §1（调用面清单 4 文件/闭包 syntect 5.3.0+two-face 0.4.5+onig 6.5.3/big 旁路 editor_store.at:595-602 实锚/046-047=auto-os memo 档 C） |
| 1 | T-01 语言集定界 | T-00 | 报告 §语言集（下游实勘只读） | 首批清单+三列表 | AC-01 | [x] 表在档（许可实勘）——报告 §2（20 语言/21 crate 三源定界+crates.io 2026-09-30 实勘+围栏分布；ini=Apache-2.0 唯一例外；.at 不含自建注记） |
| 2 | T-02 选型+烟测 | T-00 | spike 隔离目录+报告 §选型 | 两轴定案+实证 | AC-02 | [x] 烟测绿+体积初值在档——报告 §3（SMOKE-OK：ABI v15 兼容 v13+/parse 双绿/增量 changed_ranges=[44..59)/查询捆绑 21/21 实证/exe 3,618,304B[归档终版源码]/冷构建 3.0s） |
| 3 | T-03 共存策略 | T-01/02 | 报告 §共存 | 定案+退役量化 | AC-03 | [x] 数字对照在档——报告 §4（双轨定案+退役≈12.4MB .rdata→16.46MB≈4.1MB 门富余对照 019+§5 want 生命周期止于本线注记） |
| 4 | T-04 增量要点 | T-00 | SD-01 册 §增量 | 管线设计要点 | AC-04 | [x] 册节在档——SD-01 §4（快照消费/失效域含 spike API 纪要/big 旁路硬边界/047 协同注记） |
| 5 | T-05 契约草案 | T-01..04 | 报告 §实施件契约 | 715+ 立项基 | AC-05 | [x] 骨架在档——报告 §6（T-1..T-7 任务骨架+AC 草案+715/716 拆分+工期量级+基准/验收设计） |
| 6 | T-06 落账 | T-01..05 | SD-01+报告+specs.json | 勘定收口 | AC-06 | [x] P714-1 True+零 diff 断言——specs.json reviews P714-1（外科插入：roundtrip 字节等价 1,183,344B 先证+五段零扰动回读+reinsert stable；git diff master crates/+Cargo.toml=0 行） |

## 9. 复审记录

- 2026-09-30 起草 handoff：`stage: new`，PLAN-714，plan_revision 1。
  `outcome: pass`（起草完备：供④ 两段式原文承接[勘定段]——四勘
  定面+烟测实证设计+主线零改动 frozen；语言集以下游现实定界的
  纪律防闭门造车；实施件契约为建议性骨架不预支授权；路径/符号
  经 auto-lang@05974f71d 与 auto-edit 供料档/构成表三源锚定；
  授权=起草[用户指令原文在录]，执行待用户启动——auto-lang 会话，
  注意 713 稿并行协调）。`next: work`。

- 2026-09-30 work handoff：`stage: work`，PLAN-714，plan_revision 1。
  `outcome: pass`。`code_commit`: plan-714-dev 9e935f423（报告
  docs/plans/reports/714-treesitter-survey.md+SD-01 册
  docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md
  +spike 源档 docs/plans/spike-714-treesitter/；基线 master@05974f71d，
  worktree D:/autostack/.wt/lang-714/auto-lang）。`task_ids`: T-00..T-06
  全账（7/7，§8 表逐项证据在列）。`evidence`: 四定案（语言集 20/21
  crate 三源定界+许可实勘[ini=Apache-2.0 唯一例外]；管线=0.27 runtime
  +子 crate 族+build-time cc+自带查询 21/21 实证；双轨迁移+two-face
  退役量化≈12.4MB→门富余；增量要点含 spike API 纪要[tree.edit 平移
  range/has_changes=标记面]）；spike SMOKE-OK（parse/增量/查询三段
  绿+体积初值 3.41MB+冷构建 3.0s）；AC-06 零 diff 断言
  （git diff master crates/+Cargo.toml=0 行）+specs.json P714-1 外科
  插入（roundtrip 字节等价先证+五段零扰动+reinsert stable）。
  Category A 门禁：零 crates 改动、无测试面新增，cargo t/docs_gen 免。
  `blockers`: 无（Q-1 按默认下游现实定界执行；Q-2 烟测定案
  build-time cc；Q-3 实施件排期待用户）。`next: review`。

- 2026-09-30 review handoff：`stage: review`，PLAN-714，plan_revision 1。
  `outcome: pass`（含修复环 1 轮）。`reviewed_commit`: plan-714-dev
  2530e6d6b（终版=work 9e935f423+修复 F-1/F-2）；`base_commit`:
  master@05974f71d（merge-base；执行窗内 master 并行前进至 1f25465e7
  [711 簿记/712 收纳]——与本案文件零交叠实证：git diff master...dev
  仅 5 件本案交付物）。`dependency_revisions`: auto-edit main@70c5c60
  （只读实勘面）/auto-os p047 evidence（046-047 身份锚）。
  `spec_inputs`: SD-01 册（worktree 终版）+供料档 §4/§5+019 构成表。
  `acceptance_results`: AC-01 pass（许可三项抽查复跑 crates.io API：
  ini=Apache-2.0/rust=MIT/md=tree-sitter-grammars 全中）；AC-02 pass
  （spike 复跑 SMOKE-OK）；AC-03 pass（019 数字 grep 4 处锚点吻合）；
  AC-04 pass（SD-01 §4 在档 :82）；AC-05 pass（报告 §6.3 在档 :322）；
  AC-06 pass（crates/+Cargo.toml/Cargo.lock 对 merge-base 零 diff=0 行
  +specs.json P714-1 回读 True+INDEX 再生 no-op）。`findings`:
  F-1（低，已修）exe 体积收据陈旧——旧引 3,578,368B 为高亮段重写前
  构建值，归档终版源码复跑实测 3,618,304B（+40KB，量级结论 3.4-3.5MB
  不变）——报告 §0/§3.1/§3.2+README+计划 §8+账本 P714-1 六处对齐
  （账本外科再编辑：roundtrip 守卫+单行 ±）；F-2（低，已修）SD-01 册
  报告链接相对路径少一级（../../../ → ../../../../，ls 解析断言过）。
  `evidence`: 复审独立声明（实施会话内复审——判定从工件重建：烟测
  复跑/许可 API 抽查/数字 grep/diff 断言/readback 全部本会话重放）；
  修复提交 2530e6d6b（3 文件 ±5 行）；Category A 门禁维持（零 crates
  改动，无测试面）。`next: merge`。

- 2026-09-30 merge 收据：`stage: merge`，PLAN-714:r1，
  `completion_kind: delivered`。**prepared**：reviewed 基线 2530e6d6b
  （pass，AC-01..06 复放在案）；canonical delta=SD-01 add（
  docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md，分支
  内已备）；projection target=specs.json designs 段新项 P714-2
  （file+docsha+commit 标签）；归档目标 docs/plans/archive/。
  **landed**：rebase master 后 range-diff 双等（9e935f423→0697929b6、
  2530e6d6b→41ace6107，补丁全等证明）；ff-only 合并零合并提交，
  master tip=delivery 41ace6107 实证（5 文件 867 行）；docs-only
  集成烟=INDEX 再生 no-op。**ledger_refreshed**：designs 段 P714-2
  外科插入（roundtrip 字节等价先证+五段零扰动 designs 116→117+回读
  True，docsha 5e7c2fa187fee8ef）@ ad95f78e7；INDEX 再生 no-op 复核；
  plans.md/overview 行回写按 710 先例不适用（勘定件无行为面——
  710/711/712 均无行实证）。**archived**：git mv 至 archive/+
  status: archived+completion_kind: delivered（本节收据）。批量回归
  到期判定=**不到期**（.last-batch-regression.json last_covered=712
  @7fcf913eb，收据 2026-09-30T08:04:49Z 仅 9h；714%5=4 非五倍数；
  无 48h+合并触发）。**部署观察**：docs-only 落地（crates/ 零 diff
  断言在案）——backend release/daemon/gen front 三面零重建项维持。
  **cleaned**：待下笔（清理后补）。

## 10. 待澄清事项

- **Q-1 语言集预裁定（可选）**：默认 T-01 按下游现实+战略「常见
  20 语言起步」定界（候选族见 §5 T-01——含 .at[需自写 grammar，
  成本注记后定含/不含]）；若用户对首批语言有明确偏好（加/删），
  执行前示知——否则按勘定定稿。
- **Q-2 分发形倾向确认（无需裁定，烟测定案）**：默认倾向
  build-time 编译（生态主流+「单 exe」语义无冲突）；若烟测显示
  构建复杂度/体积反转（如 cc 链在 Windows 工具链的面），预编译
  嵌入案升位——按 T-02 证据定，两案对比表在档。
- **Q-3 实施件排期（勘定交付后，用户件）**：715+ 立项时点=勘定
  delivered 后（M4 长尾关键路径——建议勘定后尽早排期；同时清
  M3 尾巴「语法高亮联动」与 M4 产出 4，下游消费件随后）。
- **执行注记（2026-09-30）**：Q-1 按默认定界执行（用户未另行裁定，
  §2.2 表定稿可复审调整）；Q-2 烟测定案=build-time cc（预编译案
  未升位——§3.1 判据表在档）；Q-3 仍待用户（715 立项基已备）。
