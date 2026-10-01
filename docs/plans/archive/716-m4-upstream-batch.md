---
plan_id: PLAN-716
status: archived             # Phase 2 修复轮交付归档（2026-10-01 merge r3 收据——delivery 33dd018cb，终态不再回改）
feature_name: M4 上游三组合一件（用户合并裁定 2026-09-30——组A tree-sitter 首批实施[714 契约 T-1..T-6 承接]/组B 帧时间戳插桩[供②]/组C diff rows 惰性投影[021 armed FAIL 清偿路径]+供③ 顺手确认）+ Phase 2 修复轮（2026-10-01——下游消费首跑实证交付缺口：供⑧ 9920 VM 裸名臂/供⑨ 9918-9919 .at 消费面双缺口/供⑪ highlight-treesitter cc 夹缝）
author: [agent]
created_at: 2026-09-30T22:17:32+08:00
updated_at: 2026-10-01T18:30:00+08:00
plan_revision: 3              # r3=Phase 2 修复轮（r2=交付时归档修订——供③ 摘出挂账；r3 语义契约增补=消费面缺口修复，Phase 1 目标/验收不回改）
current_step: 18              # 全任务账 T-00..T-18 收口（Phase 1 十四+Phase 2 五）
total_steps: 18
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/treesitter-highlight-pipeline.md（SD-A：实施契约册——714 勘定册姊妹，before=勘定结论/after=实施语义）"
  - "docs/specs/auto-lang/ui/design/frame-observability.md（SD-B：帧时间戳观测通道契约——门控零开销纪律+双轨可达；Phase 2 modify=.at 消费面出口形修正节）"
  - "docs/specs/auto-lang/ui/design/diff-endpoints.md（SD-C：modify——rows 窗口投影参数节[默认全量兼容]；Phase 2 modify=9920 注册面补记 codegen intrinsics 裸名臂）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（供料驱动面，703/714 先例注记式）
affects: [crates/auto-lang/Cargo.toml, crates/auto-lang/src/ui/code_editor/core/highlight.rs, crates/auto-lang/src/ui/code_editor/core/mod.rs, crates/auto-lang/src/ui/iced/, crates/auto-lang/src/ui/code_editor/diff/envelope.rs, crates/auto-lang/src/vm/native_catalog.rs, crates/auto-lang/src/vm/native.rs]
---

# [PLAN-716] M4 上游三组合一件（tree-sitter 实施+帧插桩+rows 惰性投影）

## 0. 变更摘要

用户 2026-09-30 裁定**三组合一**（「两者都合到计划 716 里一起起草
吧？它们都是 auto-lang 的计划吧」——多件合立先例 701 六件包/703
五件包；714 勘定原建议拆 715/716 两件，715 已被 website 域占用，
合并按用户裁定落 716 单件）。**组A=tree-sitter 首批实施**（714 r1
勘定契约 T-1..T-6 承接：runtime feature 接入→P0 10 语言→增量面→
P1 8 语言→P2 3 语言+tail 固化→**two-face/syntect/onig 退役**[.rdata
≈12.4MB→installer ≤15MB 门富余弹药]；T-7 .at grammar 自建=非目标
独立件）→**组B=帧时间戳插桩**（M4 供料包供②——type_latency/
scroll_fps 测量解锁+steady_start 首帧段分解；AUTO_BENCH 门控零开销
纪律；**与 712 r2 帧泵/dirty/epoch 域同域——执行期协调注记 §10
Q-2**）→**组C=diff rows 惰性投影**（diff-engine-supply §6 在册
want 升格——auto-edit 021 L2 直拉 **armed FAIL 5183.2ms** 的清偿
路径：100MB 散点对全量 rows 投影 47MB+双层 JSON 49MB=主耗；窗口
投影参数[默认全量兼容]）+**供③ 顺手确认件**（大文件卡死回归
v2205 复绿后的多跑复核销账——M4 供料包全清尾）。三组独立可验、
组内任务序贯；组B/组C 为短件先落，组A 为长件主轴。

**Phase 2 修复轮（2026-10-01 激活——用户裁定「重新激活计划716,
记录成新的phase，然后我们去单独执行修复」）**：下游 auto-edit
PLAN-022（M4-05 消费件，worktree plan-022-dev@600ba65）消费首跑
实证本件 Phase 1 交付面的**三处缺口**（全证据链=auto-edit
docs/upstream/2026-09-m4-perf-unblock-supply.md §8，供⑧⑨⑪ 登记
在案）：**供⑧=组C 9920 五面注册缺一**（vm/codegen.rs intrinsics
裸名表漏登记 diff_files_window——catalog/shim/trans/ui_gen 四面
在册，VM 轨调用挂死实测；a2r 轨不受累）；**供⑨=组B 通道 .at
消费面双缺口**（VM 轨 i64→int 桥退化——shim 真值实测但 .at 侧
赋值落 0/.str()→None/json→0[time 族 now_ms 返 0 同根因，PLAN-005
登记面]；a2r ui_gen handler 直调臂不路由 handler 体 frame 二段名
〔下游 front 探针臂预埋 regen E0425 实录〕——上游探针驻 Rust
i64 lane 未过 .at 面）；**供⑪=组A sql 登记位依赖夹缝**（tree-
sitter-sequel 0.3.2 cc `~1.0.90` 紧钉 vs 新纪元 blake3 `^1.1.12`
——同 major 单版本选一空交集，下游以 blake3 1.5.5 回避钉在案）。
Phase 2 范围=三缺口修复+下游回执位（Phase 1 目标/验收/交付史
零回改；供③ P716-D1=环境面非代码缺陷，维持 KNOWN-DEBT 挂账
不入本轮）。

## 1. 目标

- **G-A1 runtime 接入**：`highlight-treesitter` feature（tree-sitter
  0.27+tree-sitter-highlight+子 crate 族 build-time cc 编译——714
  §3 定案）；lang→引擎路由表骨架+`HighlightConfiguration` 单例；
  **feature 关闭=零 diff 语义**（现行为逐字节等价——syntect 双轨
  基线）；开启后 tail 语言（.at/mermaid/vue/console）行为不变。
- **G-A2 首批 20 语言接入**：P0 10+P1 8+P2 3（714 §2.2 表——
  crates.io 许可实勘 2026-09-30 在册）；每语言金样本 fixture（含
  keyword/string/comment/number/嵌套构造）→capture 流→金 token
  对照；双轨期与 syntect 输出做**类别级**对照（非逐 token 等价——
  两引擎切分口径不同）；big 态 plain 旁路不变（013 硬边界继承）。
- **G-A3 增量高亮面**：714 §5.2 管线——rope 快照后台解析+`tree.
  edit`/`changed_ranges` token 级失效域+行边界重高亮+兜底全量；
  增量延迟档（1KB/100KB/1MB 阶梯）在档。
- **G-A4 two-face 退役面**：主题底座替换（714 §4.1 注记）+
  syntect/two-face/onig 依赖摘除+依赖收口；产物尺寸实测（.rdata
  对比）+全矩阵绿+bench 装载墙钟不回退；**installer 预算行联动
  注记**（auto-edit 019 分阶段态的收口弹药——下游件消费）。
- **G-B1 帧插桩通道**（供②）：帧开始/呈现完成时间戳观测面（最小
  面——供料 §2 原文）；**观测零行为差异**（未订阅/未标记零开销
  ——AUTO_BENCH 门控纪律，PLAN-005 先例）；时间源单调可换算毫秒。
- **G-B2 .at 可达双轨**：VM 内建只读面（time 族 701 供④ 先例——
  99xx 高段空闲位）+a2r 臂同源（710 后 a2r 轨可达性成立——供料
  原文承接）；**插桩点与 712 r2 帧泵域协调**（T-00 勘定+§10 Q-2）。
- **G-C1 rows 窗口投影**：`diff_files` envelope 增可选窗口参数
  （rows_offset/rows_limit 形——T-00 定参）+rows_total 计数+
  truncated 语义激活（011 保留字段——窗口截断如实置位）；**默认
  全量=零扰动兼容**（016 消费面零改动断言——envelope 契约 frozen
  面不破）。hunks 全量保持（导航域）。
- **G-C2 大对收益数字**：100MB 散点对窗口调用 vs 全量调用的
  envelope 体量/墙钟对照（021 FAIL 5183ms 的清偿弹药——下游切换
  件依据）；探针=census 形（710 先例）。
- **G-D 回归门+供③ 销账**：cargo tf 全量（预存红对账零新增——
  710 惯例）+组A 基准+组B 开销实证；**供③ 顺手确认**（大文件
  卡死回归多跑复核×3——018「疑似已解阻，登记保持至上游确认」
  注记的销账动作）。
  【plan_revision 2 修订（用户裁定 2026-10-01）】：供③ 多跑×3
  组件**降级挂账**（KNOWN-DEBT P716-D1）——执行期取证：矩阵
  6 次启动受共栖 UI 负载/端口死占阻断（楔死形态与 712 r2 T-16
  桌面轨残余族相似但假说未证实；同二进制在空闲时段曾健康推进
  T1→T13）。供③ 语义=「确认复核件」（018 已单跑全绿，多跑为
  稳定性双保险），其完成不阻塞本件三组主体交付；确认动作转由
  712 残余调查修复后窗口期或下游复核件承接。AC-REG 相应缩窄。
- **G-E 规范+账本**：SD-A/B/C 三面+specs.json P716-1。

### 非目标

- **T-7 .at grammar 自建**（714 契约独立评估件——M 量级可后置）；
  tree-sitter 全量语言/L 线符号索引（L2 域）。
- **下游消费件**（auto-edit：语法面矩阵回归+帧两行断言化+对比表
  滚动列+diff 窗口切换与 FAIL 清偿判定+installer 收口——本件
  delivered 后各自立项/并入既有队列）。
- 712 r2 域（VM 装载调度/渲染响应——**同域协调不并案**：组B 只
  加观测面不改调度语义；rebase 顺序 §10 Q-2）。
- 渲染器架构变更/常驻渲染器（Q2 已裁单 iced——组B 仅插桩）；
  scroll 锚定/平滑滚动（滚动**测量**解锁≠滚动**行为**变更）。
- diff 引擎算法变更（组C=envelope 投影层——histogram/分组语义
  frozen）；diff_snapshots 净形扩 rows（另一 want 域，不混）。

- **G-D1 供⑧ 裸名臂补全**（组C 收口）：vm/codegen.rs intrinsics
  表补 `diff_files_window`→NATIVE_DIFF_FILES_WINDOW——9920 五面
  注册齐装；VM 轨 .at 裸名调用贯通（HTTP/merged 双形不再挂死）。
- **G-D2 供⑨-a .at 消费面桥修复**（组B 收口）：9918/9919 的 .at
  侧值流贯通——`var int = frame.begin_ms()` 落真值/`.str()` 出数字/
  json.from_value 保真三面全断修复（勘定=出口形三候选择一：
  stdlib 判型重勘 `-> int`/VM native 返回边界 i64→int 强转修复/
  701 全宽值字符串出口先例变体——T-14 勘定定形）；time 族
  now_ms 返 0 同根因面随桥修复一并复验（同根因不另立任务）。
- **G-D3 供⑨-b ui_gen handler 臂补全**（组B 收口）：ui_gen/rust.rs
  直调臂路由 handler 体 frame 二段名（code_editor_delta 直调纪律
  同款）——handler 体调用 regen exit 0+生成码 a2r_std::frame 路由。
- **G-D4 供⑪ 依赖夹缝解钉**（组A 收口）：highlight-treesitter+
  blake3 现势共解析可建（sequel 升版/换源或 auto-lang 侧需求面
  放宽——上游定形）；下游回避钉（blake3 1.5.5）具备撤钉条件。

## 2. 架构方案

分层落点（2026-09-30 实勘，auto-lang master@d7a2ddc14）：

| 面 | 现状 | 本期形态 | 依据 |
|---|---|---|---|
| 语法高亮 | syntect 5.3/two-face 0.4.5/onig 6.5.3 单例（highlight.rs:78-146——extra_no_newlines 全量+LazyThemeSet+合成主题）；调用面 5 点（mod.rs:510/523/592+autodown core.rs:332/346+iced renderer.rs:1537） | 组A：feature 双轨→路由→20 语言→增量→退役（714 契约 T-1..T-6） | 714 r1 勘定（四定案+§6.3 契约+证据索引） |
| 帧观测 | 无通道（.at 层零观测面——PLAN-005 T-03 勘；"rendered"仅 MCP 快照面） | 组B：帧开始/呈现完成时间戳+门控零开销+VM 内建/a2r 双轨 | 供料 §2（m4-perf-unblock）+712 r2 帧泵域协调 |
| diff envelope | rows 全量投影（envelope.rs——703 交付面；100MB 散点=47MB rows+49MB 双层 JSON 主耗） | 组C：窗口参数（默认全量兼容）+rows_total/truncated 语义 | 021 armed FAIL 5183.2ms+惰性投影 want 在册（diff-engine-supply:174/budgets:63） |
| 供③ | 「疑似已解阻，登记保持至上游确认/多跑复核」（018 ef467b3 注记） | 多跑复核×3+销账注记 | M4 供料包全清尾 |
| 并行域 | 712 r2 executing（帧泵 dirty/epoch——T-16）；715 website 稿 | 协调注记（§10 Q-2）+编号不冲突（716 实证空闲） | git log 实勘 |

**关键设计约束（frozen）**：
① **feature 关=零语义**（组A——关闭态产物逐字节等价，评审机读）。
② **观测零行为差异**（组B——未订阅零开销实证；不染调度语义——
712 r2 域边界）。③ **默认全量兼容**（组C——016 消费面零改动；
窗口形=增量参数非破坏性；envelope frozen 契约不破——增字段而非
改字段）。④ **big 旁路硬边界**（013——组A 继承）。⑤ 三组独立
可验（组B/C 短件先落不候组A 长件——分支内任务序贯+独立 AC）。

## 3. 技术栈

Rust workspace（crates/auto-lang——ui/code_editor 核心+iced 观测点
+diff envelope+VM 内建五面注册族[703 先例]）；金样本 fixture 族+
探针（plan716_supply_probes——710 census 形）；tf 全量门+预存对账；
工具链=worktree 构建（016/018 纪律）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-09-30 会话指令「两者都合到计划 716 里一起
起草吧？它们都是 auto-lang 的计划吧」——确认两件（tree-sitter
实施+供②[含惰性投影]）均 auto-lang 域+**授权合并起草本件**（组C
按我上轮描述中供②附带件语义纳入——§10 Q-1 确认位）；执行/work
待用户另行启动（auto-lang 会话——712 r2 在途协调+716 编号执行时
复核）。范围=auto-lang crates/docs/specs；auto-edit 零改动（组C
收益数字=探针自证，下游切换=消费件）。无预算/自动续跑授权。

**来源与版本**：

- 组A 基：714 r1 勘定报告（docs/plans/reports/714-treesitter-
  survey.md——四定案+§6.3 契约 T-1..T-7+spike 源档 spike-714-
  treesitter/+证据索引）+SD 勘定册（treesitter-highlight-survey.md
  @acf653d3f r3 交付态）；r4（注册断链）与本组无路径冲突（ui_gen
  vs highlight 面）。
- 组B 基：M4 供料包供②（auto-edit docs/upstream/
  2026-09-m4-perf-unlock-supply.md §2——诉求/期望形态/验收建议
  齐备）；PLAN-005 T-03 勘（无观测通道）+AUTO_BENCH 门控先例；
  701 供④ time 族（VM 内建+a2r 双轨同源先例）；**712 r2**（帧泵
  dirty/epoch 改造在途——同域协调）。
- 组C 基：diff-engine-supply.md:174（rows 惰性投影 want 在册）+
  budgets.json:63（红线余量薄根因注记）+021 T-02（L2 直拉 armed
  FAIL 5183.2ms——median 谱 diff-20260930-205147.jsonl；VM 形态
  1906ms 对照）+011（truncated 保留字段语义——窗口激活面）。
- 供③ 基：018 ef467b3（v2205 复绿实证+「确认复核件」定性）。
- 现状实勘（master@d7a2ddc14）：highlight.rs 调用面五点/Cargo
  feature 面（:69/:255-257）/diff envelope.rs（703 族）/native
  catalog 99xx 段空闲位（9915-9917 已占——710 后实勘复核）；712
  r2/715 在途注记。

## 5. 详细设计

### T-00 三组勘定决策件（有界调查，产物=勘定记录）

#### T-00 勘定记录（2026-09-30 work 启动实勘，基=master@57b9afa60，worktree D:/autostack/.wt/lang-716/auto-lang）

**Q-2 裁定（712 r2 协调）**：712 r2 帧泵/dirty/epoch 改造**已 delivered+archived**
（merge 451dc1401+ff32d7004，2026-09-30；残余 T-16..T-19 为桌面轨 AppTick/点书
get_json 调查——仅 master docs 簿记，零 renderer 域在途代码 WIP，无 712 worktree
在册）。→ 组B 直接挂已交付帧泵，无 rebase 顺序问题。挂点实证：帧通知泵=
`frame_pump_sub`（renderer.rs:8018，711 交付——`listen_raw` 过滤
`RedrawRequested`，注释明证「listen() 显式丢弃 redraw；无 post-present 回执，
硬屏障需 fork iced_winit——禁止」；R-1 序障：泵消息消费时刻≥present 返回）。
→ **帧开始**=`TickWrap::update` 入口（renderer.rs:26912，全消息流经点）；
**呈现完成**=`__frame_pump` 消息消费点（present 的合法代理，同 711 SCHED-DIAG
先例）。AUTO_SCHED_DIAG（renderer.rs:8033）=同域门控先例，组B 门控命名
`AUTO_FRAME_BENCH=1`（AUTO_BENCH 纪律族，auto-edit specs README:419 先例）。

**组A 实施基复核**：
- 调用面五点复核在位（highlight.rs syntax_system():211/warm_language():114/
  highlight_segments():292；消费=core/mod.rs:510/523/592+autodown_editor/
  core.rs:332/346+iced/renderer.rs:1537）；feature `code-editor`（Cargo.toml:69）；
  编辑面=cosmic-text 0.15 `SyntaxEditor::new+ViEditor::new`（mod.rs:510-525/
  592 区）——**ViEditor 硬绑 SyntaxEditor（syntect），双轨期编辑面保持 syntect
  （cosmic-text 契约），路由面覆盖只读/fence/markdown 三消费形；20 语言编辑态
  高亮在 T-06 后由缩减 syntect 集（default-syntaxes）承载**——契约 T-1..T-6
  的「按 lang 路由」语义落在 highlight_segments 族（只读三面），非 cosmic-text
  编辑器内部（该项无公开自定义高亮 API，714 §5.1 单例/快照语义不受影响）。
- 依赖闭包实勘（主检出 Cargo.lock）：syntect 5.3.0→onig 6.5.3→onig_sys（onig
  后端在用）；cosmic-text 0.15.0 deps 含 syntect（vi+syntect features）；
  two-face 0.4.5。T-06 摘 onig 需 syntect 转 `default-features=false,
  features=["default-fancy"]` 纯 Rust 后端 + cosmic-text 的 syntect feature
  形态实证（`cargo tree -e features` 现场裁决）。
- 语言清单定稿=714 §2.2 表（P0=10/P1=8/P2=3，21 crate，Q-3 默认无增删）。
- worktree 构建注记：Cargo.lock gitignored（worktree 首建自生成）；
  autodown-core path 依赖经 `../../../auto-down/...` 解析至 D:/autostack/auto-down
  主检出（组内无需兄弟 worktree）。

**组C 定参**：
- 下游消费面双轨实勘（auto-edit）：in-proc cdylib 直调
  `diff_files_envelope_from_paths(a,b,3)`（auto-edit/src/main.rs:1327，下游自截
  600 行渲染）+ HTTP 面 `GET /api/diff_files?path_a&path_b&ctx`
  （auto-edit-back/api.rs:119→fsys.rs:216→`a2r_std::diff::diff_files`）。
- **9915 签名冻结裁定**：VM shim 定位式弹出（native.rs:835，ctx/b/path_a 三参），
  加参即破 3 参调用方（016 消费面零改动红线）→ 窗口形走**新端点**
  `auto.diff_files_window`（**9920**，9918/9919 归组B 帧族）：
  `(path_a, path_b, ctx, rows_offset, rows_limit) → str`；9915 原样不动
  （「9915 增参贯通」语义=diff_files 端点族增参，经新端点承载）。
- envelope 语义表：窗口调用形增 `rows_total` 字段（总行数，全量计数）+
  `truncated` 激活（=offset+返回行数 < rows_total）；**默认三参调用=逐字节等价**
  （不增 rows_total 字段——最严零扰动，强于「消费面无感」）；`rows_offset`
  越界（≥total）→ `rows:[]`（diff_snapshots 净形复用）+truncated=total>0；
  limit 0 → 空+truncated 同语义；hunks 全量保持（导航域不受窗口影响）。
  a2r 臂=`a2r_std::diff::diff_files_window`（五面注册同 703 族）+trans rust
  裸名臂；HTTP query 串形（SD-C 记载供下游）：可选 `rows_offset`/`rows_limit`
  i64，缺省=全量。

**供③ 复核方案**：auto-edit 下游矩阵 T17 大文件交互簇（T17.2/17.3/17.8 三检查）
多跑×3（v0.4.2-2205 release 工具链单跑形态，018 ef467b3 同口径）；全绿→018
登记位回写销账注记（M4 供料包供③ 清尾）。命令面取自 auto-edit README
PLAN-017 口径节（T-12 现场执行）。

**槽位复核**：native catalog 99xx 实勘（native_catalog.rs:38-61）最高占位
9917（9915/9916/9917=diff 三件）→ 9918/9919（组B 帧族）/9920（组C 窗口件）
空闲在册，与计划 §4 注记一致。

1. **组A 实施基复核**：714 r4 后 master 重勘（调用面/feature 闭包
   锁定版）；语言分布二次核对（P0/P1/P2 清单定稿——§10 Q-3 预
   裁定位）。
2. **组B 插桩点勘定**：iced 帧生命周期面（合成/呈现回调锚点——
   renderer.rs 域）+**712 r2 帧泵改造现状对接**（dirty/epoch 面
   的观测挂钩点——避免双改同域冲突）；VM 内建位次（99xx 空闲段
   实勘）+a2r 臂五面注册清单（703 族）。
3. **组C 定参**：窗口参数形（rows_offset/rows_limit vs window
   单参——011 front cap 600 对齐性+HTTP query 串形勘定）；
   rows_total/truncated 语义表；净形复用面（diff_snapshots
   rows:[] 先例——窗口极限形=净形+按需）。
4. **供③ 复核方案**：多跑×3 的矩阵面与 fixture（大文件交互簇
   T17 域——auto-edit 在册检查名引用）。

### T-01 组A-runtime 接入（G-A1）

`highlight-treesitter` feature+tree-sitter 0.27 runtime+子 crate
依赖骨架（P0 十语言 crate 先挂）+lang→引擎路由表+单例；**feature
关=零 diff 语义断言**（产物逐字节等价——golden 对照）。

### T-02 组A-P0 语言接入（G-A2）

P0 10 语言：crate+金样本 fixture+类别级双轨对照+矩阵该语言面绿
+big/plain 旁路不变断言。

### T-03 组A-增量面（G-A3）

快照投递/changed_ranges/行边界重高亮/兜底全量管线（714 §5.2）；
增量延迟档（1KB/100KB/1MB）+正确性 fixture 增量路径复绿。

### T-04 组A-P1 语言接入（G-A2）

P1 8 语言同 T-02 形。

### T-05 组A-P2+tail 固化（G-A2）

P2 3 语言+tail 路由固化（.at/mermaid/vue/console 留 syntect——
退役前的显式路由表成文）。

### T-06 组A-two-face 退役（G-A4）

主题底座替换+syntect/two-face/onig 摘除+依赖收口；产物尺寸实测
（.rdata 前后）+全矩阵+bench 装载不回退；installer 联动注记（供
下游收口件弹药——数字入 SD-A）。

### T-07 组B-插桩通道（G-B1）

帧开始/呈现完成时间戳（T-00② 锚点——712 r2 协调后挂点）；门控
（AUTO_BENCH 族同款 env 门）+单调时源。

### T-08 组B-.at 可达双轨（G-B2）

VM 内建（99xx 位次——T-00② 实勘）+a2r 臂+五面注册（703 族）；
探针=标记到达序+读回形（time 族形态）。

### T-09 组B-开销验证（G-B1/B2）

未订阅/未标记零开销实证（开/关两态基准对照——观测面开销 ≤噪声
带）；探针报告。

### T-10 组C-rows 窗口投影（G-C1）

envelope 窗口参数+rows_total+truncated 激活语义；**默认全量零
扰动断言**（016 消费面 golden 对照——frozen ③）。

### T-11 组C-端点贯通+收益数字（G-C1/C2）

9915 增参贯通（HTTP query+VM 内建双轨）+census 探针（100MB 散点
窗口 vs 全量——体量/墙钟对照表）；下游切换依据数字入 SD-C。

### T-12 回归门+供③ 销账（G-D）

tf 全量（预存对账零新增）+组A 基准汇总+组B 开销+组C 对照表；
**供③ 多跑×3 复核+销账注记**（018 登记位回写）。

### T-13 规范+账本（G-E）

SD-A（treesitter-highlight-pipeline.md——实施契约：feature 语义/
路由表/双轨对照口径/增量管线/退役面+尺寸数字）+SD-B（frame-
observability.md——通道/门控/时源/双轨）+SD-C（diff-endpoints.md
增窗口投影节）；specs.json P716-1 外科插入（703/710 先例）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-A | add | docs/specs/auto-lang/ui/design/treesitter-highlight-pipeline.md | before：勘定册（survey——四定案+契约草案）在册，实施语义未落 / after：实施契约册——feature 关=零语义/路由表与 tail 清单/类别级对照口径/增量管线语义/退役面与尺寸实测数字 | 714 契约 T-1..T-6 的落账真源 | AC-A1..A4 |
| SD-B | add | docs/specs/auto-lang/ui/design/frame-observability.md | before：.at 层无帧观测通道（PLAN-005 勘） / after：观测通道契约——帧开始/呈现完成时间戳面+门控零开销纪律+单调时源+VM 内建/a2r 双轨+712 r2 帧泵域边界注记 | 供② 落账；下游帧两行测量的内核侧真源 | AC-B1/B2 |
| SD-C | modify | docs/specs/auto-lang/ui/design/diff-endpoints.md | before：diff_files envelope=全量 rows 投影（truncated 恒 false） / after：增窗口投影节——rows_offset/rows_limit 参数（默认全量=016 消费面零扰动）+rows_total+truncated 激活语义+hunks 全量保持 | 021 FAIL 清偿路径；envelope frozen 契约的增字段扩展 | AC-C1/C2 |

### Phase 2 补充（r3——2026-10-01）

**授权记录**：用户 2026-10-01 指令「block的内容都是716计划没实现好
的吗？请重新激活计划716,记录成新的pahse，然后我们去单独执行修复。」
——授权=**本件出库重激活+Phase 2 修复范围记录**；执行（work）待
用户单独启动。范围=供⑧⑨⑪ 三缺口（auto-lang 单仓）；下游 auto-edit
侧回执位（probe_diffwin VM 形/T-03 探针臂重埋）非本轮任务。无预算/
自动续跑授权。证据锚=auto-edit 供料档 §8+worktree plan-022-dev
实录（evidence-p022-t01.json/probe 过程记录/budgets 供⑨ 行）。

### T-14 供⑧ 勘定（定案级）

- **根因**（已勘定，下游实录）：五面注册中 `vm/codegen.rs`
  intrinsics 裸名表缺 `diff_files_window`（:559-561 三 diff 面在册；
  catalog:67/shim native.rs:901,917/trans rust.rs:5107/ui_gen
  rust.rs:10487 四面在册）——裸名无本地符号亦无 intrinsic→VM 轨
  解析挂死（HTTP 线程无响应实测，请求不达 handler）。
- **修复形**：intrinsics 表补
  `("diff_files_window", NATIVE_DIFF_FILES_WINDOW)`（NATIVE 常量
  已在册——catalog 引用实证）。
- **验收锚**：VM 轨 .at 直调返回窗口 envelope（rows_total 在场）；
  下游回执位=auto-edit `probe_diffwin.py` 缺省 VM 形 8/8。

### T-15 供⑨-a .at 桥勘定+修复（本轮深水件——bounded 勘定前置）

- **实证面（下游三断）**：`var fb int = frame.begin_ms()` 落 0；
  `frame.begin_ms().str()`→None〔下游 Tick TypeError 'str'+
  'NoneType' 实录〕；`json.from_value({b: frame.begin_ms()})`→0。
  shim 直读真值在案（进程内同址单调 0→1435→1451——下游临时插桩
  FB-DBG，已还原）。上游探针 `frame_timestamps_vm_readback` 驻
  Rust i64 lane（call_i64 直读）——.at 消费面=716 测试盲区。
- **根因候选（T-15 勘定定形，三择一或组合）**：①stdlib/auto/
  frame.vm.at 判型 `-> i64` 与 shim 推栈形（push_i64_vm→
  encode_i64_with_heap nv 编码）错配——.at int lane 预期 vs i64
  lane 标记丢失；②VM native 返回边界缺 i64→int 强转/降级臂；
  ③701 全宽值字符串出口先例的字符串变体（epoch 级值语义保全）。
  **勘定产出=定形记录+所选形对照三断实证**。
- **time 族同根因面**：PLAN-005 T-03 登记「VM 轨 time 族内建未
  接线（now_ms/now_sec 返 0）」——若根因=native i64 返回桥，
  桥修复后 time 族随行复验（同根因注记，不另立任务；若独立根因
  如实拆出）。
- **回归锚**：plan716_supply_probes 增 .at 面断言（赋值/str/json
  三保真）+现役五探针零扰动。

### T-16 供⑨-b ui_gen handler 臂

- **实证**：下游 front store handler 内 `frame.begin_ms()` 采样臂
  →a2r regen **E0425 cannot find value `frame`**（ui_gen/rust.rs
  :10487 直调臂仅覆盖模块 fn 体面？——trans rust.rs:5107 裸名臂
  与 handler 直调臂的覆盖面勘定=本轮首任务子步）。
- **修复形**：handler 体二段名路由补臂（code_editor_delta 直调
  纪律同款——生成码落 a2r_std::frame::{begin_ms,present_ms}）。
- **验收锚**：含 handler 体 frame 调用的 .at regen exit 0+生成码
  grep a2r_std::frame 臂；下游 front 探针臂重埋回执（auto-edit
  侧，非本轮任务）。

### T-17 供⑪ 夹缝解钉

- **实证**：`tree-sitter-sequel v0.3.2` 钉 cc `~1.0.90`〔≥1.0.90
  <1.1〕vs 新纪元 blake3 1.8.7 钉 `cc ^1.1.12`——同 major 单版本
  选一→空交集〔下游首建败 release-20261001-160555.log 实录〕；
  716 执行期 lock 处于兼容纪元（推测其 lock 的 blake3/cc 组合未
  过 1.1 缝），现势 fresh 解析即触。
- **修复形（上游定形，候选）**：sequel 升版（上游 crate 新版是
  否放宽 cc 需求——registry 实勘）/换 sql 引擎 crate/auto-lang
  Cargo.toml 对 blake3 需求面放宽（若其 cc 需求可降级兼容）。
  **不采用**：lock 手工钉（下游回避钉已演示其脆弱性——1.5.5 纪元
  随 registry 演进会再失效）。
- **验收锚**：fresh `cargo update`+highlight-treesitter 全量解析
  可建（无 blake3 降钉）；下游撤钉回执位注记。

## 6. 测试设计

- **组A**：金样本 fixture 族（每语言≥1——capture 流→金 token 对照）
  +双轨类别级对照+feature 关态逐字节等价 golden+增量路径正确性+
  增量延迟档+big/plain 旁路断言+退役后全矩阵+bench 装载墙钟对照。
- **组B**：探针（标记到达序单调+读回形）+零开销两态基准对照（噪声
  带内）+a2r 轨同源对拍（VM/a2r 同输入同时间语义）。
- **组C**：默认全量 golden 对照（016 消费面零扰动）+窗口形语义表
  探针（边界：offset 越界/limit 0/窗口跨 hunk）+100MB 对照数字。
- **供③**：大文件交互簇多跑×3（全绿=销账）。
- **回归门**：cargo tf 全量预存对账零新增；探针族 plan716_supply_
  probes（710 形态）。

## 7. 验收标准

- **AC-A1 feature 门**：关闭=产物逐字节等价（golden）；开启=tail
  行为不变。验证：golden 对照+探针。
- **AC-A2 语言接入**：20 语言 fixture 全绿+类别级对照在档+big/
  plain 旁路断言绿。验证：探针+矩阵抽档。
- **AC-A3 增量面**：增量延迟档数字在档+增量路径正确性绿。验证：
  基准报告+探针。
- **AC-A4 退役面**：syntect/two-face/onig 摘除（cargo tree 断言）
  +尺寸实测数字+全矩阵绿+bench 不回退。验证：构建断言+对照表。
- **AC-B1 通道**：帧时间戳标记到达序+单调时源+门控零开销实证。
  验证：探针报告（两态对照）。
- **AC-B2 双轨可达**：VM 内建+a2r 臂同源（对拍绿+五面注册 grep
  锚）。验证：探针+grep。
- **AC-C1 窗口投影**：默认全量零扰动（016 golden 对照）+窗口形
  rows/rows_total/truncated 语义探针绿。验证：golden+探针。
- **AC-C2 收益数字**：100MB 散点窗口 vs 全量体量/墙钟对照表在档
  （021 FAIL 清偿依据）。验证：census 报告。
  - **AC-REG 回归门**（plan_revision 2 缩窄）：回归对账零新增
    （裸 cargo t 全量 face——fix-test-tiering 现行裁定）+触面档
    （tv/tb/tu）预存族零新增。验证：对账记录+触面档输出。
    【原「供③ 多跑×3 全绿+销账注记」组件经修订摘出——KNOWN-DEBT
    P716-D1 挂账承接，用户裁定 2026-10-01】
- **AC-SD 规范+账本**：SD-A/B/C 落档+P716-1 回读 True。验证：
  文件在档+账本断言。

- **AC-D1 供⑧ 裸名臂**（Phase 2）：VM 轨 9920 .at 裸名调用贯通
  （envelope rows_total 在场）；四面在册面零扰动。验证：上游探针
  +下游回执 probe_diffwin VM 形 8/8。
- **AC-D2 供⑨-a .at 桥**（Phase 2）：.at 侧三断全修复（赋值真值/
  .str() 数字/json 保真）+time 族同根因复验在档+supply 探针 .at
  面增补绿。验证：探针报告。
- **AC-D3 供⑨-b handler 臂**（Phase 2）：handler 体 frame 调用
  regen exit 0+生成码路由臂 grep 锚。验证：regen+grep。
- **AC-D4 供⑪ 解钉**（Phase 2）：fresh 解析 highlight-treesitter
  可建（无 blake3 降钉）+两态构建绿。验证：cargo update+构建。
- **AC-REG2 回归门**（Phase 2——AC-REG 同款口径）：裸 cargo t 全量
  对账零新增+触面档（tv/tb/tu）预存族零新增。验证：对账记录。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 三组勘定 | — | 本件 §5 T-00 节 | 实施基+插桩点+定参+复核方案 | 全 | [x] 勘定记录在档 [✅ 已完成]（2026-09-30：Q-2 裁定 712 r2 已交付归档+挂点实证 frame_pump_sub:8018/TickWrap:26912；组A 编辑面=cosmic-text ViEditor 硬绑 syntect 契约裁定+21 crate 清单定稿；组C 9920 新端点定参+默认逐字节等价语义表；供③ T17×3 方案；槽位 9918-9920 空闲实勘） |
| 1 | T-01 组A runtime | T-00 | Cargo feature+highlight.rs | feature 门+路由骨架 | AC-A1 | [x] 零 diff golden 绿 [✅ 已完成]（commit 6ebccb7d3：feature `highlight-treesitter`+tree-sitter/highlight 0.27 依赖+ts 模块骨架（空路由表=全量 syntect 基线）+highlight_segments 分流钩；验证：cargo check 两态绿+nextest highlight 11×2 态+ts 5 测全绿；API 适配实录：0.27 HighlightConfiguration::new 带 name 参、highlight() 带 encoding/cancellation 参、Source 事件改 start/end 区间形） |
| 2 | T-02 组A P0 | T-01 | 子 crate+fixture 族 | 10 语言接入 | AC-A2 | [x] fixture 绿+旁路断言 [✅ 已完成]（commit 33e4190de：P0 10 语言 12 登记位（md+markdown-inline 注入键）——查询直用 crate 导出常量零 vendoring（勘定：10/10 crate 导出 HIGHLIGHT*_QUERY+LANGUAGE LanguageFn，714 §3.1「crate 自带查询」全额成立）；typescript/tsx=js 基础查询+TS 叠加层（上游官方用法）；配色=同 AutoUI 主题 scope 匹配+基色归一（勘定：主题=9 槽位词表 keyword/storage/string/comment/function/number/type/constant/variable/punctuation）；fixture 矩阵 11 语言三层双轨对照+嵌套+容错 8 测绿；两态回归 highlight 11×2/autodown 78×2/core 70 绿；锚点校准证据：toml/yaml key 与 bash 裸词数字双引擎同素色（平价非缺口）、md markup 双引擎素色（AutoUI 无 markup 槽）——ts md 值=注入面） |
| 3 | T-03 组A 增量 | T-02 | core/mod.rs 编辑路径 | 失效域管线 | AC-A3 | [x] 延迟档+正确性绿 [✅ 已完成]（commit 2eec981a8：IncrementalSession 管线（tree.edit/changed_ranges∪编辑域/行边界扩窗/兜底全量=ERROR+40% 比例阈）；正确性=单点/链式编辑 sexp 结构零漂移+兜底路径绿；延迟档在档（debug 构建）：~70KB full 30.6ms/inc 10.1ms、~758KB full 307ms/inc 103.5ms（≈3×）；**语义调整在案**：落点「core/mod.rs 编辑路径」→管线为模块层 rope 快照消费面（714 §5.1 形）——编辑面视觉切换受 cosmic-text ViEditor 硬绑 syntect 契约约束（T-00 裁定），活编辑路径投递随编辑面切换件另档，本件增量正确性/延迟档在管线层实证（AC-A3 口径=管线正确性绿+延迟档数字在档，满足）；实勘发现：等长 token 文本替换（20→99）树结构零差 changed_ranges 空——重高亮窗=差异域∪编辑域（管线语义修正入档）） |
| 4 | T-04 组A P1 | T-02 | 同 T-02 形 | 8 语言接入 | AC-A2 | [x] fixture 绿 [✅ 已完成]（commit b137cb51f：P1 8 语言登记（html/css/cpp/csharp/go/java/sql/xml）——查询常量直用（xml=LANGUAGE_XML+XML_HIGHLIGHT_QUERY 形实勘）；fixture 矩阵 19 语言全绿+路由断言扩 P1；一次通过零校准） |
| 5 | T-05 组A P2+tail | T-04 | 路由表固化 | 3 语言+tail 显式 | AC-A2 | [x] tail 面回归绿 [✅ 已完成]（commit（T-05）：P2 3 语言（batch/powershell/ini——ini Apache-2.0 许可注记入 Cargo）；tail 路由固化成文（四 tail 语言恒 syntect 臂+固化锚测试+退役后缩减集承载注记）；fixture 矩阵 22 语言 12 测全绿） |
| 6 | T-06 组A 退役 | T-05 | Cargo+highlight.rs | 依赖摘除+尺寸 | AC-A4 | [x] cargo tree 断言+矩阵绿 [✅ 已完成]（commits（T-06×2）：two-face 摘除（语法集+主题底座→syntect defaults+in-tree .at/TOML YAML；77 语法 57 主题，首建 477ms debug 在档）；**偏差在案（证据驱动）**：①onig/syntect 保留——cosmic-text 0.15 fontconfig(default) 以 default-onig 拉 syntect（特性统一强制；iced 0.14 钉 cosmic-text 0.15 外部契约），摘除归上游解耦另档；②尺寸实测三态（auto.exe release PE 节表）：BASE 20,284,928B/总 81,998,336B→ts-off 19,655,680B（−629KB）/81,317,376B→ts-on 39,063,040B（+18.9MB）/102,394,368B——**two-face 压缩内嵌：退役实收 0.6MB，计划口径 12.4MB（019 归因）在本仓实测不成立；installer 弹药叙事按实测重写+ts 门控=installer 约束构建的载荷开关**；cargo tree 断言 two-face 零匹配；缩减集缺口（toml/powershell/ini 编辑面素色——readonly 面已 ts 覆盖）→TOML YAML in-tree 补齐；全矩阵绿（highlight 12×2+autodown 78+code_editor 118+ts 12）；sccache 30GiB 满缓毒化环境事件实录（RUSTC_WRAPPER= 绕开复原）） |
| 7 | T-07 组B 通道 | T-00 | iced 帧锚点（712 协调） | 时间戳+门控 | AC-B1 | [x] 探针到达序绿 [✅ 已完成]（commit（组B）：ui::frame_bench 通道——AUTO_FRAME_BENCH 门控零开销（门关=OnceLock 布尔读+分支，~ns 级实测在档）；插桩点=update_inner 入口（帧开始）+__frame_pump 消费臂（呈现完成——711 R-1 序障「消费≥present 返回」合法代理）；**语义注记**：帧泵消息消费点为 present 代理而非硬屏障（硬屏障需 fork iced_winit——711 勘定禁）；多窗 runner 第二泵臂（renderer.rs:19871）未插桩——SD-B 注记面） |
| 8 | T-08 组B 双轨 | T-07 | native_catalog+五面 | .at 可达 | AC-B2 | [x] 对拍+grep 锚 [✅ 已完成]（commit（组B）：**注册面实勘扩为七面**（计划口径五面+勘定新增三）：catalog 9918/9919+RET I64+NATIVE_ID_ENTRIES+native shim+trans/rust.rs 裸名臂+ui_gen handler 直调臂+**codegen (module,method) 路由对+内置模块名白名单 frame+stdlib/auto/frame 三件套**（frame.vm.at #[vm] 扫描经 NATIVE_ID_MAP 取固定 id——register_vm_declarations 实勘）；探针：VM 读回（两段名+Int lane 形——epoch 全宽值字符串出口注记承 701）+a2r 直调 grep 锚+a2r_std 同源对拍全绿；**勘定发现入档**：frame 原不在 codegen 内置模块白名单（「Undefined variable: frame」实错——synth_failures 诊断面定位），白名单+路由对为 .at 可达的必要面） |
| 9 | T-09 组B 开销 | T-07 | 两态基准 | 零开销实证 | AC-B1 | [x] 噪声带内对照 [✅ 已完成]（commit（组B）：frame_capture_overhead_two_state 两态微基准——门关 per-call<200ns 上界断言（实测 ns 级）+门开构成（elapsed+store）<500ns 上界；60fps 帧预算占比 <0.01%@1000msg/帧；数字 --nocapture 在档；实机帧序 live-fire 归下游 bench 档（供②验收形态原文：上游探针+下游 bench 分工）） |
| 10 | T-10 组C 投影 | T-00 | diff/envelope.rs | 窗口参数+语义 | AC-C1 | [x] 零扰动 golden+探针绿 [✅ 已完成]（commit（组C）：build_rows 单趟流重构（计数+窗口物化单源）；diff_files_envelope_window（rows_total+truncated 激活——头/尾省略皆置位+越界净形+hunks 全量）；默认三参逐字节等价 golden 钉（frozen ③ 最严形）；窗口语义 5 测绿） |
| 11 | T-11 组C 贯通 | T-10 | 9915 增参+census | 端到端+收益数字 | AC-C1/2 | [x] 对照表在档 [✅ 已完成]（commit（组C）：9920 auto.diff_files_window 七面贯通（9915 签名冻结裁定执行——T-00 定参）；census（release，AUTO_P716_CENSUS 门控）：**43.6MB 散点对 full=153,422,332B/1922ms → window(0,600)=6,994,427B/1360ms——载荷 21.9×↓/墙钟 −29%**（debug 口径 6511→5230ms −20% 同档在档）；021 FAIL 清偿语义=envelope 双层 JSON 主耗（49MB→7MB 过界）清偿，余量=引擎核心（histogram/分组 frozen 不动）；对照表随 SD-C 落档） |
| 12 | T-12 回归+销账 | T-01..11 | tf 全量+复核 | 回归门+供③ 清尾 | AC-REG | [x] 回归门绿+对账零新增；**供③ 子项 blocked（环境）** [✅ 回归门完成/❌供③ 未销账]（2026-10-01：按 fix-test-tiering 现行裁定 tf 改判批量回归档（merge 时 due-check 兜底）——per-plan 复审门=裸 cargo t 全量 --no-fail-fast 对账：**head 13 红=base 12 红+1（plan707_wait_generator_park_drive_count_static 并行 flake——solo 复绿 1.0s、4f123a50e 在案帧时序族），全族与 KNOWN-DEBT L119 基线一致=预存对账零新增**；触面档 tv 162/162 绿+tb（REG-1b book 七红族+已知基线族零新增）+tu 855 绿/1 预存（a2vue desktop 金样 4f123a50e 在案）。**供③ 多跑×3 未完成——环境阻断如实记档**：auto-edit 矩阵 6 次启动
实录（MCP 端口 TOCTOU 死占×3[9247 真源=ui_desktop.exe PID 2824
auto-musk 域在跑 UI+孤儿占 9248/9249；connect_ex 对 LISTENING 误判]
+boot 瞬态×1+**运行中 app 楔死×2**[主进程存活但事件循环楔死：心跳
停+SYN_SENT→8018 挂起+MCP 监听套接字消失——18 PASS 后断连，取证
升级见 Q-5 复验更新]）；**根因路由=712 r2 T-16 残余族**（桌面轨
AppTick 泵断链在案调查——矩阵 app=桌面模式动态 app，楔死形态吻合；
修复交付后自然解阻）；**018 登记位未回写（不假销账）**；解阻动作：
712 T-16..T-19 修复交付后复跑 ×3（T17.2/17.3/17.8 全绿）→018 注记
销账 |
| 13 | T-13 规范+账本 | 全 | SD-A/B/C+specs.json | 三组落账 | AC-SD | [x] SD-A/B/C 落档 [✅ 已完成]（commit（SD）：SD-A treesitter-highlight-pipeline.md（实施契约册——714 勘定册 after 面：feature 语义/路由表+tail 固化/查询零 vendoring/同主题配色/双轨对照口径/增量管线/退役面+尺寸三态表+installer 联动注记实测修订）；SD-B frame-observability.md（通道/门控/双轨/值语义/域边界）；SD-C=diff-endpoints.md 增窗口投影节（9920/rows_total/truncated 激活/边界形/零扰动 frozen ③/HTTP query 串形/census 收益表）；**specs.json P716-1 外科插入按 §4 归 merge 档执行**（skill 纪律：live ledger 不作独立需求源——AC-SD 账本断言随 merge 回读）） |

**Phase 2 修复轮任务（r3——2026-10-01 激活，T-14..T-18）**：

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 14 | T-14 供⑧ 裸名臂 | — | vm/codegen.rs intrinsics 表 | 9920 五面齐装 | AC-D1 | [x] VM 轨裸名调用 envelope 绿+四面零扰动 [✅ 已完成]（commit 97236b4a2：**勘定发现根因=双缺**——①intrinsics 裸名表漏登记（计划勘定面）②**9920 号位与 PLAN-095 ui.focus 撞号**（T-00 槽位复核漏扫 native_catalog.rs:419 段；shim 绑定表清单序后注册者覆盖→VM 轨窗口调用恒派发 ui_focus shim 静默 no-op→Int(0)，下游挂死实测根因的代码面真身）；修复=窗口件改签 **9921**（三表：for_each_native/for_each_bigvm_native/NATIVE_ID_ENTRIES）+intrinsics 补臂（ui.focus 已交付面独占 9920 不回改）；全表扫重号实证=shim 绑定表唯一重号即 9920、9921 空闲；探针 diff_files_window_vm_bare_name（VM 轨 .at 裸名→envelope rows_total/truncated 真值）红→绿+faces_registered 锚；**修复连带清偿预存红** native_catalog_ids_and_names_unique（撞号守护测试 base 红→绿）；验证：cargo t plan716 13/13 绿+cargo tv 162/162 绿；下游回执=probe_diffwin 缺省 VM 形 8/8（随下游件排程）） |
| 15 | T-15 供⑨-a .at 桥勘定+修复 | — | stdlib/frame.vm.at+vm 桥面（T-15 勘定定形——三候选见 §5） | .at 三断修复+time 族同根因复验 | AC-D2 | [x] 三断保真探针绿+time 族注记 [✅ 已完成]（commit 5fe510539：**定形=候选①+②组合**——frame.vm.at 判型 `->i64` 改 `->int`（.at int=i32 lane，值域=进程单调 ms<2^31≈24.8 天 saturating，Q-6① 值域核对过）+shim 出口 push_i64_vm 改 push_i32（TAG_I64 单槽 vs .at int 消费面 lane 错配消除）+catalog ret 表 I64→Int；**勘定实录**=三轨复现矩阵（VmBridge/程序轨 run_with_capture/evaluator×修前修后）最小形全绿——下游三断（95dcfb55 基+桌面合成上下文+下游自有 json.from_value 库）当前上游树不可重构，修复按 i64→int 桥根因假说落 lane 统一形；**time 族同根因复验=截断 Hazard 活体实证**：var int=time.now_ms() 落 32 位伪影（epoch ms 1790847058147→4140720794 实测——PLAN-005「返 0」形的现势真身），值域>2^31 不改判型防真值截断，合法出口=i64 变量/str 全宽（701 先例值域边界），now_sec 值域<2^31 int 承接正确——as-is 形探针钉住；探针=frame_value_flow_gate_on（断①赋值/②str/③复合字段读三保真门开真值）+frame_begin_ms_str_fingerprint（门态无关）+frame_program_track_gate_on（程序轨=下游 api.at 同构面）+time_family_vm_readback_recheck；Phase 1 gate_on 探针亚毫秒 flake 修复（process_start 首触定格→首 note 恒存 ~0——双 note 形出零区）；**独立发现登记**：bridge 语境 json.encode({b:字面量}) obj 编码链预存断（垃圾值，非 frame 面——KNOWN-DEBT 候选随 review 判）；验证：cargo t plan716 17/17 双门态绿+frame_timestamps 3/3 绿） |
| 16 | T-16 供⑨-b handler 臂 | — | ui_gen/rust.rs 直调臂覆盖面勘定+补臂 | handler 体路由贯通 | AC-D3 | [x] regen exit 0+生成码 grep 臂 [✅ 已完成]（commit 12e49f78c：覆盖面勘定=Phase 1 两臂均只认裸名 Expr::Ident 形（ui_gen vm_builtin_host_call handler 体/trans call():5107 模块体），二段名 Expr::Dot 原文发射→E0425；补臂三处=ui_gen vm_builtin_host_call 增 frame.begin_ms/present_ms 二段名臂（直调纪律同款——实现体单源 ui::frame_bench 不绕 shim；计划原文「生成码落 a2r_std::frame」按在册直调纪律修正为 frame_bench 直调——与既有裸名臂同源，语义等价单源）+trans 元组匹配 Dot-path 增 (frame,begin_ms/present_ms) 臂（time.now_ms A4 cat1 先例同款同位）；探针红绿链=frame_two_segment_handler_arm（原文发射红→frame_bench 直调绿）+frame_two_segment_module_trans_arm（原文发射红→a2r_std::frame 绿）；触面档=cargo tu 858/1（唯一红=a2vue desktop 金样 4f123a50e 在案预存；bp registry falls_back flake 基线同形复现非本件引入）+cargo tt frame 50/50 绿（encountered 红 merged_api_warning/lock_serializes/projector_counter 三件均基线 solo 复现=预存非新增）；下游回执=front 探针臂重埋 regen exit 0（随下游件排程）） |
| 17 | T-17 供⑪ 解钉+回归门 | T-14..16 | Cargo.toml/lock 面 | 现势解析可建+回归零新增 | AC-D4/REG2 | [x] fresh 解析构建绿（无 blake3 降钉）+对账零新增 [✅ 已完成]（**供⑪=零 diff 修复**：registry 实勘=tree-sitter-sequel 已升 **0.3.11**（cc 需求 `~1.0.90`→`~1.2.1` 放宽——下游实录 0.3.2 的钉已非现势），与 blake3 1.8.7（cc 1.1.12）交集 [1.2.1,1.3) 非空=候选①「sequel 升版」由 registry 演进自然给出；现有 Cargo.toml 需求 `version="0.3"` 零改动即解析 0.3.11+cc 1.2.67 双满足；主检出 lock 实勘已在兼容纪元（0.3.11+cc 1.2.67）；验证=cargo update fresh 解析零降钉+cargo check --features highlight-treesitter 全量解析构建绿 43.9s+ts 套件 12/12 绿；上游零 blake3 --precise 钉 grep 净；**下游撤钉条件成立**（撤 blake3 1.5.5 回避钉+工具链重解析即绿——撤钉回执随下游件排程）。**AC-REG2 对账**（fix-test-tiering 口径=nextest 全量 --no-fail-fast）：裸 cargo t 全量 head 29 红 vs base 30 红——**红集全等+1 预存红修复**（native_catalog_ids_and_names_unique=T-14 撞号守护）；tv 162/162 绿；tb（--features test-book 全量）head 37 红 vs base 38 红——红集全等+同 1 红修复；tu 858/1（预存 a2vue desktop 金样）；预存红族全景=721 carousel 围栏双红（aura.at element_coverage 未登记+kitchen-sink 未再生——721 域）+musk_vm_track 族+vue_capabilities 族+ui_snapshots×3+gallery/plan707 flake/clipboard env 等全族 base=head） |
| 18 | T-18 规范+回执位 | T-14..17 | SD-B modify[出口形节]+SD-C modify[注册面补记]+回执注记 | 缺口修复落账 | AC-D* | [x] SD 增补在档+下游回执位注记在案 [✅ 已完成]（commit 6c5c5b714（初版 dacb53ea6 误裹挟 ui_snapshots .snap.new 测试副产物，amend 剔除）：SD-B §3b 出口形与值流契约（int lane 出口定形+值域 i32 封顶 saturating+时源坐标勘定修正[process_start=首次触及时定格非进程起点——spawn 段分解消费方同坐标系标记]+time 族 as-is 复核[截断 Hazard+宽值合法出口]+下游回执位[front 探针臂重埋+T-03 帧两行判定面重驱动]）；SD-C 窗口节 9920→9921 改签补记（PLAN-095 ui.focus 撞号机理+codegen 裸名臂漏登记+守护探针锚）；**供⑪ 撤钉回执位**=下游撤 blake3 1.5.5 钉+工具链重解析（本节 T-17 行在案——随下游件排程）） |

## 9. 复审记录

- 2026-09-30 起草 handoff：`stage: new`，PLAN-716，plan_revision 1。
  `outcome: pass`（起草完备：三组各有成文基（714 契约/供料 §2/
  want 在册+021 FAIL 谱）+独立 AC 可分验；feature 关=零语义/默认
  全量零扰动/观测零行为差异三 frozen 防破坏性；712 r2 同域协调
  显式注记；供③ 顺手清尾=M4 供料包全清；合并粒度=用户裁定（715
  占用+701/703 多件先例）；路径/符号经 auto-lang@d7a2ddc14 与
  auto-edit 供料档/021 谱双源锚定；授权=起草[用户指令原文在录]，
  执行待用户启动——auto-lang 会话）。`next: work`。

- 2026-10-01 work handoff：`stage: work`，PLAN-716，plan_revision 1，
  `outcome: blocked`（单阻塞=供③ 环境项——主体 13/14 任务完成）。
  `code_commit`: worktree plan-716-dev @ 组C commit 后 tip（T-00 勘定
  记录在主检出簿记；实现 commits 链=T-01 6ebccb7d3/T-02 33e4190de/
  T-03 2eec981a8/T-04 b137cb51f/T-05+T-06×2/T-07..09 组B/T-10..11 组C/
  SD 备稿 commit——worktree `D:/autostack/.wt/lang-716/auto-lang`，
  依赖兄弟 auto-down@3373a5c detach）。`task_ids`: T-00..T-11+T-13
  完成，T-12 部分完成（回归门✓/供③✗）。`evidence`: 各任务行 [✅]
  实录（commit+验证命令+数字）；关键实测=两态 golden/tv 162 绿/
  对账零新增（head 13=base 12+1 flake 全族 KNOWN-DEBT L119 一致）/
  census release 43.6MB 载荷 21.9×↓ 墙钟 −29%/尺寸三态表/装载
  477ms/增量 3×/零开销 ns 级。`blockers`: 供③ 多跑×3（auto-edit
  矩阵环境阻断——6 次启动实录，根因路由=712 r2 T-16 桌面轨泵断链
  残余族，取证升级与解阻动作见 T-12 行+Q-5 复验更新；
  018 未回写=不假销账）。`next`: 用户裁定——(a) 712 T-16..T-19
  修复交付后补 供③ ×3 → execution_done；(b) 直接进 review（供③
  挂账 KNOWN-DEBT/由下游件顺带复核——语义=「确认复核件」非阻塞
  主体交付）；后续 merge 档=specs.json P716-1+tf 批量回归 due-check。

- 2026-10-01 review：`stage: review`，PLAN-716，plan_revision 1，
  `outcome: blocked`（单 finding，外部前置件；其余全部 pass）。
  **独立性声明**：复审在实施会话内进行——结论从工件重建（全部
  AC 验证在本复审基线重跑复现，不采信实施摘要）。
  `reviewed_commit`: 62073229c（worktree plan-716-dev HEAD，树
  干净零未提交实现）；`base_commit`: 57b9afa60（master，diff=10
  commits +2280/−58，22 文件）；`dependency_revisions`: auto-down
  @3373a5c（组内兄弟 worktree detach）；`spec_inputs`: SD-A
  treesitter-highlight-pipeline.md@d06af7a35584 / SD-B
  frame-observability.md@08c36b6c415a / SD-C diff-endpoints.md@
  55214e5877d3（评审冻结哈希，均@reviewed_commit）。
  `acceptance_results`（本基线重跑实录）：
  - **AC-A1 pass**：两态 golden——feature-off highlight 12/12 绿+
    feature-on highlight 12/12 绿（ts 全量 cfg 门控=关闭态 syntect
    基线逐字节；tail 恒 syntect 断言在 ts 套件内绿）。
  - **AC-A2 pass**：ts 套件 12/12（p0_fixture_matrix 22 语言
    roundtrip+锚点+三层双轨对照+p0_routes_registered 30 键面+
    tail_langs_never_route 固化锚）。
  - **AC-A3 pass**：incremental 4 测（单点/链式 sexp 结构零漂移+
    兜底路径+延迟档 CI 上界）含于 ts 12；延迟档数字（~70KB
    30.6/10.1ms、~758KB 307/103ms debug）随任务行在档。
  - **AC-A4 pass**：`cargo tree -p auto -i two-face` 零匹配（本
    基线重跑）+尺寸三态表（BASE/−629KB/+18.8MB——PE 实测数字
    随任务行在档；reuse 理由=PE 节表测量确定性+代码自测量后仅
    SD docs 变更）+装载 477ms+全矩阵面（highlight/autodown/
    code_editor）绿。
  - **AC-B1 pass**：探针 6/6——门开门关语义+到达序+开销两态
    （门关<200ns 上界/门开构成<500ns 上界+60fps 占比 <0.01%）。
  - **AC-B2 pass**：VM 读回（两段名）+a2r 直调 grep 锚+a2r_std
    同源对拍（双轨同源断言绿）。
  - **AC-C1 pass**：窗口 5/5——默认三参逐字节等价 golden（frozen ③
    最严形）+全窗/任意切片/truncated 语义/越界净形/limit0/跨 hunk。
  - **AC-C2 pass**：census 探针绿（默认档语义对照）+release 全尺寸
    数字在档（43.6MB 载荷 21.9×↓/墙钟 −29%——AUTO_P716_CENSUS
    门控可复现）。
  - **AC-REG partial**：回归门=按 fix-test-tiering 现行裁定以裸
    `cargo t` 全量对账执行（计划 tf 措辞早于同日裁定——适配在案）
    head 13 红=base 12+1 flake（solo 复绿、KNOWN-DEBT L119 全族
    一致）零新增+tv 162/162+autodown 78/78（本基线重跑）+tb/tu
    预存族零新增；**供③ 多跑×3 未达成**（见 finding F-1）。
  - **AC-SD pass**：SD-A/B/C 落档于 reviewed_commit（哈希冻结）；
    specs.json P716-1 按 §4 归 merge 档（纪律性后置——非缺口）。
  `findings`:
  - **F-1（AC-REG 供③ 组件，severity=blocker-for-AC-REG-only）**：
    供③ 多跑×3 未验证——6 次矩阵启动实录（端口 TOCTOU×3[真源=
    ui_desktop.exe PID 2824 占 9247+孤儿占 9248/9249，connect_ex
    对 LISTENING 误判；29447 wrapper 绕行有效]+boot 瞬态×1+app
    楔死×2[主进程存活事件循环楔死：心跳停+SYN_SENT→8018+MCP
    监听套接字消失——取证实录见 T-12 行+Q-5 复验更新]）。根因
    路由=712 r2 T-16 桌面轨 AppTick 泵断链残余族（形态吻合，假说
    未证实）。018 未回写=不假销账。**性质=外部前置件缺失**（修复
    体在 712 调查域，auto-lang 侧无可修面——组A/B/C 交付与供③
    零耦合）。
  - 无其他 finding：diff 范围全部映射计划任务/勘定发现（beyond-
    affects 面=组B 七面注册+探针族+SD 册——计划 §5/§6 显式锚）；
    无未授权缩减、无 workaround、无未处理编译警告新增（375→389
    全为预存族+sccache 事件期波动）、T-06/T-03 偏差均有证据链
    与记录（cosmic-text 契约/编辑面切换另档）。
  `evidence`: 本记录所列命令+结果为 worktree plan-716-dev@62073229c
  实测；SD 册哈希冻结（worktree 移除后经 merge 落 master 可溯）；
  供③ 实录=主检出计划簿记 T-12/Q-5 行（durable）。
  `next`: 用户裁定二选一——(a) **等 712 T-16..T-19 修复交付**后
  补 供③ ×3（T17.2/17.3/17.8 全绿+018 回写）→ 本记录更新为
  pass → merge；(b) **计划裁定降级 供③ 为登记欠账**（plan_owner
  经 /auto-plan:new 有界修订 plan_revision 2：AC-REG 摘供③ 组件
  →KNOWN-DEBT 挂账+下游复核件承接）→ 重审后 pass → merge。
  审查不做过（skill 纪律）：本复审未改实现、未发布 canonical
  specs、未动 live ledger。

- 2026-10-01 merge 收据 `PLAN-716:r2`：`stage: merge`，
  `outcome: pass`（四 checkpoint）。
  - **prepared**：reviewed 基线=62073229c（v2 契约 pass@r2）+SD 三册
    冻结哈希 d06af7a3/08c36b6c/55214e58+依赖 auto-down@3373a5c；
    canonical diff 已在 worktree 提交（SD-A/B 新建+SD-C modify 节）。
  - **landed**：rebase 57b9afa60 基 10 提交→master@417aff474 上，
    range-diff 10/10 全等（零分歧——安全重写证明），旧→新映射
    62073229c→**ec45f911c**（交付提交；T-01..T-05/T-06×2/组B/组C/SD
    逐对映射见 rebase 记录）；`git merge --ff-only` 零合并提交，
    master tip=ec45f911c 实证；SD blob 哈希随 rebase 不变（冻结
    证据存续）；rebase 后 sanity `cargo check -p auto-lang` 绿。
  - **ledger_refreshed**：master@e747e9f64 五项外科插入（designs
    P716-1/2/3=SD-A/B/C 三 canonical 投影+reviews P716-4 收据+
    reports P716-5 交付摘要；designs 120→123/reviews 183→184/
    reports 108→109）；store 写者不可用（8080 不在线/无 spec 工具）
    循 711/713/714/715/717 成例；守卫=per-section prefix 深等+未触
    段零扰动+id 集断言+parse-back 全过；**712 会话 P712-1 在途 WIP
    字节保全不裹挟提交**（暂存差 +66/−1 纯净；工作形=WIP 复原+
    本件五项，indent2+CRLF——P712-1 曾被 phase1 覆写、经会话内
    逐字快照完整复原，实录在案）；回读=committed blob 五项在位。
  - **archived**：本件 git mv docs/plans/archive/716-m4-upstream-
    batch.md + status archived（本提交）。
  - **cleaned**：guard 双过闸（auto-lang worktree clean+auto-down
    兄弟 clean——均无 reparse point）→worktree/分支 plan-716-dev/
    组目录 .wt/lang-716 全移除实证（worktree list 双仓零残留；
    D: 释放 ~7GB）。
  - **陈旧产物观察**（PLAN-092 先例）：本计划触及 auto-lang UI 引擎
    （highlight/treesitter/frame_bench/envelope）+VM 面（codegen/
    natives/stdlib frame）——master 的 release 档二进制与下游
    auto-edit/auto-musk 消费面未随本次合并重建；下游取用时需
    重编（磁盘压力下本次不重建，观察在案）。**供③ 关联**：矩阵
    复核所需二进制=master 基（ec45f911c release 重编后形态亦可）。
  - **批量回归 due check**：716%5=1 非整除；.last-batch-regression
    收据 2026-09-30（<48h）→不到期，跳过（regress 技能留待下个
    到期窗口）。

- 2026-10-01 review（plan_revision 2 再判）：`stage: review`，
  PLAN-716，**plan_revision 2**，`outcome: pass`。
  `revision_basis`: 用户裁定 2026-10-01「路径 2 降级挂账」（会话
  指令在录）——G-D/AC-REG 摘出供③ 多跑×3 组件→**KNOWN-DEBT
  P716-D1 挂账**（登记完成：KNOWN-DEBT-AND-RISKS.md P716 批，
  含 6 次实录+清偿路径+m1-supply §17「复现则原诉求恢复」语义）。
  语义契约变更=验收缩窄（无实现/依赖/测试配置变更）。
  `reviewed_commit`: 62073229c（**与前判同一 commit——证据复用
  显式理由**：代码/diff/依赖/测试配置零变更，AC-REG 缩窄面之外
  的全部 AC 验证在本 commit 已重跑复现[见上条 review 记录]，
  6 小时窗口内环境无测试相关变更）。
  `acceptance_results`（v2 契约）: AC-A1..A4/B1/B2/C1/C2/SD 全部
  pass（证据同上条，commit 未变）；**AC-REG pass（v2 缩窄形）**=
  回归对账零新增（head 13=base 12+1 flake 全族 KNOWN-DEBT L119
  一致）+tv 162/162+tb/tu 预存族零新增——已复现在案；供③ 组件
  按 P716-D1 挂账承接（非本件验收面）。
  `findings`: 无新 finding。F-1 结转为 P716-D1（挂账登记完成，
  非本件 blocker）。
  `evidence`: KNOWN-DEBT-AND-RISKS.md P716 批（durable，主检出）；
  SD 哈希冻结不变（d06af7a3/08c36b6c/55214e58@62073229c）。
  `next`: merge（specs.json P716-1 外科插入+tf 批量回归 due-check
  +worktree 清理——磁盘释放 27GB target 随 worktree 移除）。

- 2026-10-01 r3（Phase 2 修复轮）handoff：`stage: new`，PLAN-716，
  plan_revision 3（出库重激活——archive→docs/plans/，Phase 1 交付史
  零回改）。`outcome: pass`（Phase 2 范围四缺口逐一有锚：供⑧ 根因
  已勘定〔五面注册缺一——下游 codegen.rs:559-561 实录+VM 挂死实测〕；
  供⑨ 双面实证链完整〔三断实录+FB-DBG shim 真值+ui_gen E0425+上游
  探针盲区自证〕，修复形三候选列明待 T-15 勘定定形；供⑪ 夹缝机理
  clear〔cc 空交集〕+修复候选列明禁 lock 手工钉；下游回执位明确
  〔probe_diffwin VM 形/探针臂重埋/撤钉回执——非本轮任务〕；授权=
  用户指令原文在录〔重激活+记录 phase，执行单独启动〕）。`next:
  work`（worktree=D:/autostack/.wt/lang-716/auto-lang，branch
  plan-716-dev——AGENTS 惯例；启动待用户）。

- 2026-10-01 work handoff（Phase 2 修复轮）：`stage: work`，PLAN-716，
  plan_revision 3，`outcome: pass`（T-14..T-18 五任务全收口）。
  `code_commit`: worktree plan-716-dev @ dacb53ea6（T-18 amend 后
  tip；实现链=T-14 97236b4a2 / T-15 5fe510539 / T-16 12e49f78c /
  T-17 零 diff / T-18 dacb53ea6；base=master@d85bc928b；依赖兄弟
  auto-down@3373a5c detach 重建——Phase 1 复审绑定版，merge 清理后
  组内重建）。`task_ids`: T-14..T-18 全完成。`evidence`:
  - **T-14（AC-D1）**：根因实勘升级=双缺（intrinsics 漏登记+
    **9920 撞号 PLAN-095 ui.focus**——T-00 槽位复核漏扫 :419 段）；
    改签 9921 三表+补臂；裸名探针红→绿；**连带修复预存红**
    native_catalog_ids_and_names_unique。
  - **T-15（AC-D2）**：定形=候选①+②（frame 判型 `->int`+shim
    push_i32+ret Int）；三断保真探针（赋值/str/复合字段读）门开
    真值绿；time 族截断 Hazard 活体实证+合法出口在案（as-is 探针）；
    下游三断当前上游树不可重构（三轨×修前修后全绿实录）——修复按
    i64→int 桥根因假说落 lane 统一形；Phase 1 gate_on 探针亚毫秒
    flake 修复在案。
  - **T-16（AC-D3）**：二段名臂双轨补全（ui_gen 直调 frame_bench+
    trans Dot-path a2r_std::frame——time A4 先例同款）；两探针红→绿。
  - **T-17（AC-D4/REG2）**：**零 diff 解钉**（sequel 0.3.11 cc 放宽
    ~1.2.1——registry 演进自然闭合，candidate① 原文）；fresh 解析
    ts-on 构建绿+无降钉；**AC-REG2 对账**：裸 t head 29 红 vs base
    30 红（红集全等+1 修复）/tv 162 绿/tb head 37 vs base 38（同形）
    /tu 858+1 预存（a2vue desktop 金样 4f123a50e）——零新增。
  - **T-18（AC-D*）**：SD-B §3b 出口形节+SD-C 9921 改签补记落档；
    下游回执位注记在案（probe_diffwin VM 形/探针臂重埋/撤钉——
    随下游件排程）。
  `blockers`: 无。`next`: review（/auto-plan:review——r3 契约
  AC-D1..D4+REG2 独立复审；review 通过后 merge 档含 specs.json
  P716-1 刷新[SD-B/C 册哈希更新]+下游回执排程移交）。
  环境注记：worktree 构建期间组内兄弟 auto-down 重建为
  D:/autostack/.wt/lang-716/auto-down@3373a5c detach（原 Phase 1
  兄弟随 merge 清理移除——autodown-core path 依赖解析所需）。

- 2026-10-01 review（Phase 2 修复轮终判）：`stage: review`，PLAN-716，
  **plan_revision 3**，`outcome: pass`。
  **独立性声明**：复审在实施会话内进行——结论从工件重建（下列
  验收验证在本复审基线重跑复现，不采信实施摘要）。
  `reviewed_commit`: 6c5c5b714（worktree plan-716-dev HEAD，树干净
  零未提交实现）；`base_commit`: d85bc928b（master，diff=4 commits：
  97236b4a2/5fe510539/12e49f78c/6c5c5b714+T-17 零 diff）；
  `dependency_revisions`: auto-down@3373a5c（组内兄弟 detach 重建，
  Phase 1 复审绑定版）；`spec_inputs`: SD-B
  frame-observability.md（§3b 增补）+SD-C diff-endpoints.md（9921
  改签补记）@6c5c5b714（评审冻结）；SD-A 本轮零触碰。
  `acceptance_results`（本基线重跑实录）：
  - **AC-D1 pass**：diff_files_window_vm_bare_name 绿（VM 轨 .at
    裸名→窗口 envelope rows_total/truncated 真值——29 字节 fixture
    对 rows_total:3/limit2 截断置位/hunks 全量）+faces_registered
    锚（9921 三表在册+9920 撞号残留零断言+trans/ui_gen 臂 grep）+
    **base 预存红 native_catalog_ids_and_names_unique 修复复现绿**。
  - **AC-D2 pass**：plan716 套件 19/19 双门态（门关+AUTO_FRAME_BENCH=1
    门开各一跑）——frame_value_flow_gate_on（断①var int 赋值/②.str()/
    ③复合字段读三面真值==Rust lane）+frame_begin_ms_str_fingerprint
    （门态无关）+frame_program_track_gate_on（程序轨=下游 api.at
    同构面，门开）；time_family_vm_readback_recheck 绿（截断 Hazard
    as-is 钉：int 承接≠0≠真值+now_sec 秒带+.str() 全宽带）；现役
    Phase 1 五探针零扰动（同套件绿）。
  - **AC-D3 pass**：frame_two_segment_handler_arm
    （handler 体二段名→frame_bench 直调臂，原文发射零断言）+
    frame_two_segment_module_trans_arm（模块体→a2r_std::frame 臂）
    双绿——regen exit 0（gen_rust/transpile expect 通过）+生成码
    grep 锚即断言本体。计划原文「生成码落 a2r_std::frame」在
    handler 臂按在册直调纪律修正为 frame_bench（与既有裸名臂同源
    单源——语义等价，偏差已随任务行记录）。
  - **AC-D4 pass**：Cargo.lock 现势=sequel 0.3.11/blake3 1.8.7/
    cc 1.2.67（交集 [1.2.1,1.3) 非空）+`cargo check --features
    highlight-treesitter` 绿（复审基线复现 0.85s warm）+.cargo/
    scripts 零 precise 钉 grep 净。
  - **AC-REG2 pass**：对账零新增（fix-test-tiering 口径）——裸
    nextest 全量 --no-fail-fast：head 29 红 vs base 30 红（红集
    diff 全等，唯一差=被本件修复的 catalog 撞号守护红）；tb 全量
    head 37 vs base 38（同形）；tv 162/162 绿；tu 858/1（唯一红=
    a2vue desktop 金样 4f123a50e 在案预存）。全量电池运行与
    reviewed_commit 代码态一致（其后仅 docs/specs 两文件落档）；
    docs 围栏 T-18 后复跑零新增（docs_gen 3/4——kitchen_sink 红=
    base 携带 721 域，book lib 绿）。
  - **AC-D\*（T-18）pass**：SD-B §3b+SD-C 9921 补记落档于
    reviewed_commit；断言与代码证据对绑复现（frame.vm.at `->int`
    ×2/shim push_i32 saturating ×2/catalog 9921 行）。
  `findings`:
  - **F-1（severity=info，非阻塞债候选）**：bridge 语境
    `json.encode({b: 字面量})` obj 编码链预存断（字面量同断出
    垃圾值 18443647848969734401 形；obj `.str()` → CALL_SPEC
    no HashMap.str 同族）——T-15 勘定 incidental 发现；非 Phase 2
    AC 面（下游 json.from_value 为其自有库，供⑨ 验收不以该链为锚）；
    base 同态非本件引入。**处置建议=KNOWN-DEBT 登记**（随 merge
    批注）。
  - **F-2（info）**：base 预存红族 29 件全景（721 carousel 围栏
    双红=aura.at element_coverage 未登记+kitchen-sink 未再生、
    musk_vm_track 族、vue_capabilities 族、ui_snapshots×3、
    plan707 flake、clipboard/projector env 族等）——base=head
    红集全等，非本轮面；围栏双红建议随 721 域处置路由。
  - **F-3（note，无需行动）**：Phase 1 T-00 槽位复核漏扫
    （9920/ui.focus 撞号）已由 T-14 修复并全链记录——全表扫重号
    应入后续勘定 checklist（流程教训在案）。
  - 无未授权缩减、无 workaround、无新增编译警告（385 全为预存族；
    codegen.rs:7 NATIVE_PRINT_F64/U64 未用导入 base 在册）。
  `spec delta 复审`：SD-B modify+SD-C modify 内容=现行行为的持久
  契约（出口形/值域边界/时源坐标/撞号记录/回执位），非执行日记；
  与现势 canonical 册无冲突（本件即真源更新方）；frontmatter
  new_spec_components 的 Phase 2 modify 注记与实际相符；
  supersedes_spec_components=[]/touched_goals=[]（供料驱动面注记
  式）成立。
  `evidence`: 本记录命令+结果=worktree plan-716-dev@6c5c5b714 实测；
  SD 册冻结@6c5c5b714（merge 落 master 后可溯）；对账红集清单
  /tmp/base-reds.txt、/tmp/head-reds.txt、/tmp/base-tb.txt、
  /tmp/head-tb.txt（会话易失——红集结论已转录本记录：t 29vs30/
  tb 37vs38，差集均为 catalog 撞号守护红一枝）。
  `next`: merge（specs.json P716-1 刷新[SD-B/C 册更新投影]+
  worktree 清理+供③ P716-D1 挂账维持确认）。

- 2026-10-01 merge 收据 `PLAN-716:r3`：`stage: merge`，`outcome: pass`
  （Phase 2 修复轮五 checkpoint）。
  - **prepared**：reviewed 基线=6c5c5b714（r3 契约 pass 终判）+SD-B/C
    冻结哈希 1ccabb88/4de7b1c8+依赖 auto-down@3373a5c；canonical diff
    已在 worktree 提交（SD-B §3b/SD-C 改签补记=T-18 f35863780）。
  - **landed**：rebase d85bc928b 基 5 提交→master@48dcf8422 上，
    range-diff 5/5 全等（零分歧——安全重写证明），旧→新映射
    97236b4a2→685eb3d36/5fe510539→d402676d4/12e49f78c→687691830/
    6c5c5b714→f35863780/fd64e8ce0→**33dd018cb**（delivery 提交；
    T-14/T-15/T-16/T-18/账本备稿逐对映射见 rebase 记录）；
    `git merge --ff-only` 零合并提交，master tip=33dd018cb 实证；
    main 烟雾=cargo check 绿（51.8s）+canonical 断言（9921×4/§3b×1
    grep）绿。
  - **ledger_refreshed**：tracked 路径（specs.json git 在册）循 r2
    五项外科成例——worktree 内守卫式改稿（designs P716-2/3 原位更新
    =SD-B §3b 出口形/SD-C 9921 改签投影+docsha 1ccabb88/4de7b1c8+
    tag commit:6c5c5b714；reviews P716-6=r3 复审/合入收据；reports
    P716-7=Phase 2 交付摘要追加），守卫=未触段零扰动+designs 未授权
    变动断言+reviews/reports 仅追加 P716-6/7+parse-back 全过；随
    delivery commit 33dd018cb 落 master，main 上 parse-back 复验
    （P716-1..5+6/7 七项在位）。store 写者不可用（8080 不在线/无
    spec 工具）循 711/713/714/715/717/r2 成例。
  - **archived**：本件 git mv docs/plans/archive/716-m4-upstream-
    batch.md + status archived（本提交）。
  - **供③ 关联**：P716-D1 挂账维持（环境面不入本轮——r3 范围
    显式排除）；解阻动作不变（712 T-16..T-19 修复交付后复跑 ×3）。
  - **批量回归 due check**：716%5=1 非整除；.last-batch-regression
    收据新鲜度见 cleaned 后复查（随 cleaned 收据更新落档）。

## 10. 待澄清事项

- **Q-1 组C（惰性投影）纳入确认**：按用户「两者」语义（tree-sitter
  实施+供②[我上轮描述含惰性投影附带]）默认纳入本件；若本意仅前
  两件，组C（T-10/T-11）可剥出独立小件——执行前示知即可（AC 面
  独立可剥）。【2026-10-01 已按默认纳入执行并交付——组C 完整落地
  （9920+census），无需再裁】
- **Q-2 712 r2 帧泵域协调（执行启动时核）**：组B 插桩点与 712 r2
  dirty/epoch 改造同域（renderer/帧泵面）——启动时核 712 r2 状态
  （delivered 则直接挂点；在途则 rebase 顺序或挂点避开其改动面，
  T-00② 勘定裁定）；冲突不可避时组B 可后置分支段（组A/C 不受阻）。
  【2026-10-01 已核：712 r2 delivered+archived（451dc1401+ff32d7004），
  组B 直接挂已交付帧泵，无冲突——T-00 勘定记录】
- **Q-3 语言集清单预裁定（可选）**：默认按 714 §2.2 表（P0=10/
  P1=8/P2=3，.at 不含）；用户对首批语言有增删偏好则执行前示知。
  【2026-10-01 按默认清单执行——21 crate 全批次交付】
- **Q-4 交付粒度（确认口径）**：合并件=单 worktree 单分支交付
  （组B/C 短件在分支内先行落但不提前 merge）；若执行期需分批
  落地（如组A 长周期拖累组B/C 消费时点），走 plan_revision 修订
  拆分（706/714 收纳先例同款通道）——先注记不预支。【按单分支
  交付执行——组序贯落 plan-716-dev】
- **Q-5（新增 2026-10-01）供③ 环境解阻**：auto-edit 矩阵多跑×3
  需干净桌面会话（本轮 5 次启动实录=端口 TOCTOU 死占+app 静默
  死亡——与共栖 UI 负载/机器态相关，详见 T-12 行实录与解阻动作）。
  主体交付不受阻；销账语义（018 回写）待复核轮补。
  【2026-10-01 复验更新（用户清场后重跑）】：孤儿清理**未解阻**——
  第 6 次运行取证升级：①端口死占真源=**ui_desktop.exe**（PID 2824，
  auto-musk 域在跑 UI，LISTENING 9247）+孤儿 auto.exe 占 9248/9249
  ——connect_ex 探测对 LISTENING 态误判（10035 非零=「空闲」假象）
  ——29447 wrapper 绕行有效（boot 稳定）；②app 死因非进程消失：
  **主进程存活（146MB）但事件循环楔死**（X9-ALIVE 心跳 ~90s 停+
  SYN_SENT→127.0.0.1:8018 挂起[端口无监听者、源码无归属]+MCP
  监听套接字消失[TCP refused]）——18 PASS 后断连；③**根因路由=
  712 r2 T-16 残余族**（「桌面轨 AppTick 路由断链——桌面模式动态
  app 的恢复 tick 未达泵臂」在案调查）：矩阵 app=桌面模式动态 app，
  楔死形态与其泵断链假说吻合——供③ 复核与 712 残余调查**同根因
  域**，修合后自然解阻；解阻动作修订：等 712 T-16..T-19 修复交付
  后复跑（或在其调查会话内带跑本矩阵），×3 全绿→018 回写。
- **Q-6（新增 2026-10-01）主检出外来 WIP 上报**：work 启动预检发现
  master 工作树 `crates/auto-lang/src/back_proxy.rs` +7 行未提交
  （内容与 712 r2 T-17 get_json/proxy 调查吻合——疑似 712 调查会话
  遗留 WIP，违 master 零 WIP 红线）；本计划零触碰零并入，路由归属
  712 调查会话处置（stash/fix worktree）。另 `.autoos/specs.json`
  大 diff（7683/7670 行，疑 CRLF 归一）+`.next-id` 716→718 为簿记
  面，同样非本计划所有。

- **Q-6 Phase 2 执行协调（执行启动时核）**：①供⑨-a 勘定若定形为
  「stdlib 判型 `-> int`」——核对 int lane 位宽是否承载毫秒级值域
  （进程起点单调 ms<2^31 常态；epoch 级全宽值=701 字符串出口注记
  口径不回改）；②721 desktop 会话/桌面实例共栖窗口（P716-D1 环境
  前提）与本轮无耦合（纯代码面）——但 worktree lang-716 建树时
  避开其在途 worktree 惯例位；③下游回执时点=本轮 review 通过后
  （auto-edit probe_diffwin VM 形+T-03 探针臂重埋+撤钉回执——随
  下游件排程，非本轮 gate）。
  【2026-10-01 执行收口应答】：①frame 族定形 int lane——值域核对过
  （进程单调 ms<2^31；i32 封顶 ≈24.8 天 saturating 注记入 SD-B §3b）；
  time 族 epoch 值域 >2^31 不回改判型（截断 Hazard 活体实证+宽值
  合法出口=i64 变量/str 全宽，as-is 探针钉住）。②组内 worktree
  D:/autostack/.wt/lang-716/auto-lang 无位冲突（721 在途域未触及）；
  兄弟 auto-down@3373a5c 重建（Phase 1 复审绑定版）。③下游回执位
  三件（probe_diffwin VM 形/front 探针臂重埋/撤钉）注记在案——
  随下游件排程，非本轮 gate。另 Q-6(前) 主检出外来 WIP 项：
  back_proxy.rs +7 行与本轮启动预检不再在册（712 会话已处置）；
  本轮主检出改动仅本件簿记。
