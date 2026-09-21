---
plan_id: PLAN-674
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: rq-codeeditor-a2r-vocab
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 ui overview native queue 覆盖集 codeeditor 扩册, SD-02 shell-a2r-seams 词汇门扩容契约]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/design/shell-a2r-seams.md]
current_step: 6
total_steps: 7
---

# [PLAN-674] 生成器件：RQ 渲染臂 codeeditor 覆盖 + a2r codegen 词汇门扩容

> 来源：auto-edit 上游供料包（2026-09-21 M1 批）
> `docs/plans/attachments/673-674-m1-supply.md` §6/§7（auto-edit 仓
> plan-004-dev @ 44e3a74/ab57177 实勘登记）。两缺口同族——**生成/投影面的
> 覆盖词汇不足**，阻消费方 L1/L2 性能测量阶梯（RQ 路与 a2r 主形态）。

## 0. 变更摘要

两件生成面覆盖扩容：

1. **§6 RQ 渲染臂覆盖缺口**——`Coverage::native_queue_set()` 无
   `codeeditor`：`auto run -r vm -q`（VM+RQ）实例即退（拒绝渲染语义报
   `native queue 臂视图未覆盖: tag:codeeditor`），rqhost 侧
   `adoption 未达 Active`。扩册 + 投影臂落地，并勘定其余 not-yet kind
   全集（tree/menubar 族等）。
2. **§7 a2r codegen 词汇门**——`add_prop_to_builder` 识别面仅
   class/style/padding/spacing 四 prop（事件面 onclick/oncontextmenu/
   onchange 族），其余 prop 一律 `compile_error!`（PLAN-027 显式拒绝门）。
   消费方视图 DSL 的 `value`/`text`/`title` 等合法 prop 全被拒（实测 23
   错）——`auto build -r rust` merged 主形态被阻。按**机制扩容**（per-kind
   prop/event 表驱动），非消费方名单。

## 1. 目标

- **G1（codeeditor 过线）**：`native_queue_set` 增补 codeeditor +
   native_projector 投影/命中臂落地；examples 041 运行时拒收钉
   （coverage.rs:1399 `Some("tag:codeeditor")` 期望）翻 Covered；消费方
   `auto run -r vm -q` 实例达 Active 可渲染。
- **G2（not-yet 全集勘定）**：以 coverage 判据自动列出未覆盖 kind 全集，
   逐 kind 三态勘定（本批修/登记另立/降级归一），产决策档。
- **G3（词汇门扩容）**：prop/event 识别面按 View IR builder 能力表驱动
   扩容（单源纪律）；真未知仍显式拒绝（防静默缺件原则不回退）；消费方
   `auto build -r rust` 生成物可编译。
- **G4（交接）**：消费方复跑解阻通知（669 §10-4 模式；复验判据 =
   `perf.py smoke` exit 0 与 `perf.py a2r` exit 0——供料 §6/§7 各自
   复验条）。

**非目标（Non-goals）**：

- 不动 a2r **server 模板**缺口（`use api::Db` 硬编码 + 契约 fn 空体桩
  ——P670-D1 在册，供料 §4 另行）；词汇门与 P670-D1 三支路无重叠
  （那是 server 端点转译，这是 app 级视图 codegen）。
- 不动 vue 轨（PLAN-671 域）；不动编辑器内核（PLAN-673 域）。
- RQ 覆盖扩册只做**渲染/命中过线**；codeeditor 在 RQ 形态下的完整交互
  保真（IME/折叠/搜索面板等深水区）按投影策略允许显式降级登记（I3
  留痕纪律），不静默缺件。

## 2. 架构方案

| 件 | 现状锚点（本仓实读） | 目标形态 |
|---|---|---|
| RQ 覆盖 | `ui/desktop_protocol/coverage.rs:184` `native_queue_set` 现集 = text/button/form 族（input/textarea/checkbox/radio）/slider/select/image/progress/popover/mousearea/windowthumbnail/workspacepreview/tabs/canvas——**codeeditor 不在列**；scan 侧映射已在（:528 `View::CodeEditor{..} => "codeeditor"`）；启动覆盖门 `native_projector.rs:363-369` + `client_entry.rs:189`（judge 拒绝即退，AC-04 拒绝渲染语义）；041 拒收钉测试 `coverage.rs:1382-1399` | kind 入册 + 投影臂（策略 §10-1 裁定：canvas 先例〔PLAN-034 D4 位图快照通道〕vs 结构 DrawOps）+ 键入/命中回传面；041 钉翻 `None` |
| 词汇门 | `ui_gen/rust.rs:6177` `add_prop_to_builder` 识别 = class/className/style/padding/spacing（:6182-6215），余者 `compile_error!`（:6218-6227，PLAN-027 T-04 显式拒绝门——历史动机：shell 生成物防"看似编译过实缺件"）；事件 `add_event_to_builder`（:6263）识别 onclick 族/oncontextmenu/onchange 族，onmouseenter 族双轨同弃（:6275-6279），余者拒（:6281-6287） | 识别面从「四 prop 白名单」扩为**per-kind 词汇表**（源 = View IR builder 方法面/aura schema 单源驱动），映射到 iced builder 调用；未知 prop/event 拒绝门语义保留（真未知仍 compile_error） |

**机制红线**（沿 671 用户裁定同款）：扩容一律按机制（builder 能力表/
schema 单源/事件类型映射），**不针对消费方 prop 名单打补丁**——
`value`/`text`/`title` 只是通用机制的项目投影。

## 3. 技术栈

Rust（crates/auto-lang ui/desktop_protocol + ui_gen/rust.rs）；aura schema
（schema/aura.at）与 View IR（ui/view.rs builder 面）为词汇单源候选；
fixture 验证链（tests/ 通用 .at 语料 + a2r 生成 + cargo check）；消费方
复跑面 = auto-edit `tools/perf/perf.py`（smoke/a2r 两段，退出码 0/3/1
约定——3=blocked-on-upstream 归因即当前实测态）。

## 4. 需求分析与背景调查

**授权记录**：2026-09-21 用户指令（auto-edit 会话任务②）——供料包发往
本仓立上游计划，拆两件（本件=生成器件）。授权范围 = **立项起草**
（drafting）；执行另行授权。

**消费方证据**（供料包 §6/§7 原文 + 本仓独立复核）：

- §6 实测（auto-edit PLAN-004 T-06，2026-09-21 11:03/11:07 两轮）：
  `auto run -r vm -q` 双实例派发后 0/2 存活，app 日志死因
  `native queue 臂视图未覆盖: tag:codeeditor`；rqhost 侧
  `adopt App→…app-1/-2` 后 `adoption 未达 Active（预算耗尽弃置）`。
  编排链（rq-up/run/rq-down）验证在位——阻塞纯在渲染覆盖面。
- §7 实测（T-07，11:09/11:15 两轮）：`auto build -r rust` 生成物 23 错，
  全为词汇拒绝门 compile_error!（首错 prop `value`，另见 `text`/`title`
  ——消费方 menubar/tab/actions DSL prop 面）。

**本仓独立复核**（2026-09-21，master@20f79de62）：

- coverage.rs 现集逐 kind 核对（:184-231 kinds + :232-245 layouts）——
  codeeditor 确不在列；canvas 为最近扩册先例（PLAN-034 T-05 D4：位图
  快照通道 + on_hit 节点命中——codeeditor 投影策略的直接参照系）。
- coverage.rs:1379-1400 `native_gate_runtime_views_of_six` 六例运行时
  视图覆盖钉：041-auto-edit 期望 `Some("tag:codeeditor")` 拒收——
  **上游已把该缺口登记为已知 not-yet 家族**（注释 :1382「041 codeeditor
  （整 kind）两真 not-yet 家族运行时拒收」），本计划即其清偿件。
- ui_gen/rust.rs 词汇门两处（prop :6218 / event :6281）拒绝臂注释明示
  PLAN-027 T-04 设计（§3a-a4）——扩容不动该设计原则，只扩识别面。
- PLAN-027 rev2 spec 面（ui/overview.md:4 在册）：shell a2r 接缝面
  S1/S2 + 显式拒绝门/词汇门，详见 `design/shell-a2r-seams.md`
  （provisional）——SD-02 落点。

**先例与交集**：PLAN-025/026/028/029/032/034（native_queue_set 逐批
扩册史，每批 = 覆盖集 + 投影臂 + 命中面 + 钉测试同步）；PLAN-027（拒绝
门设计原案）；671（机制红线同款裁定 + T-06 单源注册表先例）；P670-D1
（server 端点面，非重叠，防误并）。

## 5. 详细设计

### 5.1 RQ 渲染臂 codeeditor 扩册

- **投影策略**（§10-1 裁定，T-00 产决策档）：(a) canvas 先例——场景
  栅格化位图快照 + 命中回传（过线快，文本交互经宿主合成）；(b) 结构
  DrawOps（文本 op 族逐行发射——保真好，行数大时 op 流量风险）。倾向
  (a) 起步 + 交互键入回传最小面（消费方 L1 结构探针只需结构+存活，
  L2 测量需稳定满帧）。
- **扩册四件套**（沿 025-034 惯例）：kind 入 `native_queue_set`；投影臂
  入 native_projector；键入/命中回传面；041 钉期望翻 `None` +
  capability-tests 增 RQ 形态 fixture。
- **not-yet 全集勘定**：以 scan_native_view × native_queue_set 判据跑
  examples/capability-tests 全语料，自动列出未覆盖 kind 全集（供料 §6
  推测 tree/menubar 族同缺——以勘定为准），逐 kind 三态分流（本批修/
  登记另立/降级归一），决策档入计划 §9。

### 5.2 a2r codegen 词汇门扩容

- **单源驱动**：per-kind prop 词汇表源 = View IR builder 方法面
  （ui/view.rs 各 View*Builder——解释态已消费的同一能力面）或 aura
  schema（schema/aura.at）——单源裁定 §10-2；发射映射 = prop → iced
  builder 调用（.value(...)→binding、.text(...)、.title(...) 等，逐
  kind 映射表）。
- **事件面同步扩**：`add_event_to_builder` 补 on_change 载荷族/
  on_contextmenu 等既有 View 槽（与 671 T-03 vue 侧形参装配同族——
  两侧机制各自落地，语义对齐）。
- **拒绝门保留**：词汇表外仍 compile_error!（PLAN-027 原则：未知 =
  显式缺件，不静默丢弃）；错误文案带 kind 名（现文案只报 prop 名，
  扩容后补 `on <kind>` 上下文——排障友好）。
- **互锁**：词汇表若源 aura schema，schema_drift 围栏同步（671 T-08
  先例）。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md（native queue 覆盖集段） | before：覆盖集 = 基础 kind + tabs/canvas（041 codeeditor 在 not-yet 案册）/ after：codeeditor 过线契约（投影策略、交互回传面、降级显式登记）+ not-yet 全集勘定档指针 | 覆盖集是 spec 级事实（025-034 每批同步），041 案册翻案须回写 | AC-01/02/06 |
| SD-02 | modify | docs/specs/auto-lang/ui/design/shell-a2r-seams.md（词汇门节；provisional 档随扩容转正） | before：词汇门 = 四 prop 白名单 + 拒绝语义（PLAN-027 rev2）/ after：per-kind 词汇表契约（单源、映射表、拒绝门保留、错误文案含 kind） | 词汇面契约化，防"识别面=名单"回潮 | AC-03/05/06 |

## 6. 测试设计

- **覆盖**：041 钉翻 `None` 后 `native_gate_runtime_views_of_six` 绿；
  全语料 scan×judge 勘定脚本收据入计划；capability-tests 增 codeeditor
  RQ fixture（VM+RQ 实跑达 Active）。
- **词汇门**：fixture .at 语料（含 value/text/title 等合法 prop + 一个
  真未知 prop）——合法面生成物 `cargo check` 过；真未知仍 compile_error
  且文案含 kind。逐 kind 映射表单测。
- **通用性静态门**：修复面 grep 零消费方特定标识符（auto-edit/
  editor_store/menubar_item 消费方命名等——机制投影除外，同 671 §6）。
- **回归**：`cargo t` + 改 a2r/trans 面按 AGENTS.md 作用域 `cargo tt`；
  PLAN-027 既有拒绝门测试零回退。

## 7. 验收标准

- **AC-01（codeeditor 过线）**：`native_queue_set` 含 codeeditor + 投影/
  命中臂落地；041 运行时钉翻 `None`；本机 `-q` 实例渲染存活（达 Active）。
- **AC-02（not-yet 全集决策档）**：全语料勘定清单在档，逐 kind 三态分流
  （修/另立/降级登记），无"未知"残留。
- **AC-03（词汇门机制扩容）**：per-kind 词汇表单源驱动落地；消费方
  `auto build -r rust` 生成物编译过（供料 §7 复验：23 错清零）。
- **AC-04（拒绝门不回退）**：真未知 prop/event 仍 compile_error!（文案
  含 kind）；PLAN-027 既有测试绿。
- **AC-05（fixture+grep 门）**：通用 fixture（零消费方业务名）双绿；
  grep 门零命中。
- **AC-06（spec+交接）**：SD-01/SD-02 落账；消费方复跑通知发出（复验判据
  perf.py smoke/a2r exit 0；669 §10-4 模式）。

## 8. 执行步骤

| ID | 任务 | 依赖 | 落点（文件/符号） | 意图 | AC | 验证命令/预期 |
|---|---|---|---|---|---|---|
| T-00 | 双勘定：①codeeditor 投影策略裁定（canvas 位图先例 vs 结构 DrawOps）+ not-yet 全集自动列举；②词汇表单源裁定（View builder 面 vs aura schema）——决策档入 §9 | — | coverage.rs 判据 + 本计划 §9 | 设计先行 | AC-01/02/03 | [x] 三裁定入 §9（A=结构 DrawOps·位图不适用证据=rasterize 无文本面+Plan 386 现成降层；B=builder 面·schema 残缺证据 code_editor 元素缺 value/font_size；C=批次边界）|
| T-01 | codeeditor 扩册四件套：入册 + 投影臂 + 键入/命中回传 + 041 钉翻 `None` | T-00 | coverage.rs:184 / native_projector.rs / coverage.rs:1399 | 过线 | AC-01 | [x]（复审 R-1 修正收口：on_cursor I3 注记+P674-D4 在册；余证据同前） `native_gate_runtime_views_of_six` 绿（041 翻 None）+ 矩阵钉/键入闭环 `codeeditor_rq_typing_roundtrip` 绿 + 实机 `-q`：adopt→window opened→**first frame**→稳定帧（净态复跑收据 2026-09-21 22:48）|
| T-02 | not-yet 全集三态分流执行：本批修项（按 T-00 分流）逐 kind 落臂；另立/降级项登记 KNOWN-DEBT | T-00 | 同 T-01 面扩 | 全集收口 | AC-02 | [x] 双根勘定仪器 `native_not_yet_kind_inventory` 在档：全集仅两项（style:native-unstyled ×12 capability-tests CSS 面 → P674-D1；tag:managed_content ×1 PLAN-656 合成件 → P674-D2），双双登记另立；tree/menubar 推测证伪（语料零出现）；examples/ui judged 24/24=100% |
| T-03 | 词汇表机制：单源抽取 + per-kind prop→iced builder 映射表 + `add_prop_to_builder` 改表驱动（拒绝臂保留，文案补 kind） | T-00 | ui_gen/rust.rs + view.rs 单源 | 机制扩容 | AC-03/04 | [x] view.rs `view_prop_vocab`/`ViewPropShape` 单源 + 表驱动臂 + menubar 族降层臂（合成变体+on() 臂+action_config 全局开合态）；`vocab_gate_per_kind_prop_table`/`menubar_family_lowers_to_popover_row` 绿；**消费方实测 23 compile_error! → 0**（auto-edit main@dc99328 重生成）|
| T-04 | 事件面同步扩容：`add_event_to_builder` 补 View 事件槽族（载荷语义与 vue 侧对齐注记） | T-03 | ui_gen/rust.rs + view.rs | 事件面 | AC-03/04 | [x] `view_event_vocab`（Closure/Msg 双槽形）+ 拒绝文案补 kind；`vocab_gate_unknown_keeps_rejection_with_kind` 绿（prop+event 双断言）|
| T-05 | fixture+回归+grep 门：tests/ 通用 fixture 双绿（codeeditor RQ + 非基础 prop）；cargo t/tt；PLAN-027 拒绝门测试零回退 | T-01..T-04 | tests/ + capability-tests | 证明集中 | AC-04/05 | [x] capability-tests/codeeditor-rq 入运行时门绿 + 实机 `-q`（adopt→window→**first frame**→稳定帧）；plan039 族 19/19 零回退；grep 门零消费方标识符（首跑 ActOpen 撞门已改泛化名）；**cargo t 5394/5415（21 红全数归因：musk 5=master 全等证 + ui::layout 15=在册环境豁免 + projector_counter=在册预存；顺带治愈 master 红 icon_component_class）；cargo tt 4084/4084 全绿**（test-trans 无 ui 组合编译修复=词汇表迁 ui_gen/vocab.rs 的组合律动因）|
| T-06 | spec 落账+交接：SD-01/SD-02 落盘；消费方复跑通知（perf.py smoke/a2r exit 0 判据） | T-05 | docs/specs/ + §9 记录 | 收口 | AC-06 | [x] SD-01（overview 覆盖集段 v1.9）/SD-02（seams 词汇门节转正）落盘；**消费方实测：perf.py smoke exit 0·双实例 2/2 存活**（供料 §6 的 0/2 清偿）；a2r 段 BLOCKED 首错移位 E0425（P670-D1 族=P674-D3 登记——**2026-09-22 用户裁定：扩 P670-D1 并入其清偿面**，债册两条已同步）|

## 9. 复审记录

- 2026-09-21 · stage: new · r1 起草 · 授权=用户指令（auto-edit 供料包
  发车拆两件，本件=生成器件；供料原件附件 `673-674-m1-supply.md`
  §6/§7）· outcome: **pass（待执行授权）** · next: **work**（授权后自
  T-00 起；§10-1 投影策略与 §10-2 单源为开工前待裁项）。
- 2026-09-21 · stage: work · T-00 双勘定（决策档）· 授权=用户指令
  「计划674: 实施它」（auto-plan-work 入场，drafting→executing）·
  outcome: **pass（裁定三项）** · 证据=master@f2e1aa9a1 实读 · next:
  T-01 起。
  - **裁定 A（§10-1 投影策略）= 结构 DrawOps**（否决计划倾向的位图
    起步）：①`rasterize_canvas_scene`（native_projector.rs:2422）纯
    矢量栅格（tiny_skia 图元族，**无文本整形面**）——codeeditor 主
    载荷是文本，位图通道产空帧，canvas 先例不迁移；②Plan 386 已有
    全链降层 `core::render::render(&core, fs, w, h, None)` →
    EditorDrawList → `lower_editor_frame`（editor_frame.rs:32，gutter/
    当前行/选区/搜索/文本 run/caret/preedit/滚动条全 op 面）——结构
    通道现成非新建；③流量风险消解：text_runs = "per visible run ×
    syntax span" 视口虚拟化，op 流天然有界；④键入回传走 core
    `handle_input` 全键面（含 IME/undo/箭标，EditorInput 族），RQ 臂
    与 inproc iced widget **同源**（CODE_EDITORS 注册表 keyed by
    storage_key + `with_font_system`——renderer.rs:25181 同款
    get-or-create + `code_editor_set_text` 外部值差分）。
  - **裁定 B（§10-2 词汇表单源）= View IR builder 方法面**：①schema/
    aura.at code_editor 元素 props 残缺（:335-351 缺
    value/font_size/highlight_current_line——convert_code_editor 实际
    消费面，"props TBD" 在案）——schema 今天**不是完备单源**；②
    builder 面 = a2r 发射语义本源（发射物即 builder 调用链，如
    ViewCodeEditorBuilder .value/.lang/.line_numbers/.wrap/.vi/
    .highlight_current_line/.tab_width/.font_size/.search，view.rs
    :3003-3081 实读），解释态 convert_* 同源消费；③schema 侧对齐随
    收口另行登记（SD-02 注记）。落地：view.rs 单源表
    `view_prop_vocab(kind)`（kind 归一走既有 tag_to_view_fn），
    `add_prop_to_builder` 改表驱动（拒绝门保留+文案补 kind）；**复合
    族（menubar 族 value/title/icon/shortcut/enabled/checked）非
    View kind**——由 a2r 族降层臂消费（降层契约 = 解释态
    convert_menubar_component（aura_view_builder.rs:7486）镜像：
    menubar→Row of Popover{BottomStart, open=action_config::
    menubar_open()==menu_id, on_dismiss=__MenubarClose}，trigger=
    Button(onclick=__MenubarToggle(menu_id))，item=menu_item_button
    同律——**开合态走 action_config 全局面**（MENUBAR_OPEN 进程级
    Mutex，a2r 生成物同可读写——与 VM 轨同机制非消费方补丁）），不
    经词汇门。
  - **裁定 C（§10-3 批次边界）**：本批修 = codeeditor（T-01）+
    not-yet 全集勘定后「examples 语料实际出现且同根顺手族」（T-02，
    worktree 跑 `native_flip_coverage_data_row` 勘定后定）+ 词汇门
    机制面（T-03/T-04，含 menubar 族降层臂——AC-03 编译过线必须）；
    登记另立 = 勘定出的深水区 kind（autodowneditor/terminal/
    imagesurface 等交互深水族，若非消费方解阻最小集）。


- 2026-09-21 · stage: work · r1 T-01..T-06 全量执行 · plan-674-dev
  @ worktree `D:/autostack/.wt/lang-674/auto-lang`（auto-down 依赖位
  `D:/autostack/.wt/lang-674/auto-down` @ fba6563ed detached）·
  outcome: **pass（execution_done 待 review）** · code_commit =
  worktree `feat(ui): PLAN-674 RQ codeeditor 覆盖…`（12 文件：coverage/
  native_projector/editor_frame/code_editor core+mod/ui_gen mod+rust+
  vocab 新增/specs 两件/capability-tests codeeditor-rq 新增）·
  task_ids = T-01..T-06 · blockers = 无（P674-D3 处置已裁：2026-09-22 用户「括P670」=扩 P670-D1 并入清偿面，债册 P670-D1/P674-D3 双条同步）·
  next: **review**。
  - **证据摘要**：041 运行时钉翻绿；矩阵钉/键入闭环绿；双根勘定
    （全集两项登记 P674-D1/D2）；消费方实测两连——**perf.py smoke
    exit 0 双实例 2/2 存活**（供料 §6 清偿）+ a2r 词汇门 23
    compile_error! → 0（供料 §7 判据达成；a2r 段整体 BLOCKED 首错
    移位 E0425 = P670-D1 族 = P674-D3 登记，全编译过需该族清偿——
    **消费方复跑通知**：auto-edit 侧可复验 smoke（已绿）；a2r exit 0
    须等 P670-D1 族清偿后复跑——处置已裁 2026-09-22：扩 P670-D1）；回归 cargo t 21 红全数预存
    归因（musk 5=master 全等 + layout 15=在册豁免 + projector=在册
    预存）+ tt 4084/4084 全绿；grep 门零消费方标识符。
  - **执行期裁定/调整记录**：①§10-1/§10-2/§10-3 三裁定（T-00，见上
    条）；②词汇表物理落位 view.rs → **ui_gen/vocab.rs**（组合律：view
    域整树挂 `ui` feature 门后，`test-trans` 无 ui 组合编译断——语义
    源仍在 builder 面，SD-02 文案同步）；③§10-4 交互回传深度落位
    review 待核面：core handle_input 全键面已达（键入/退格/回车/箭标/
    Home/End/PageUp·Down/Delete/Tab/IME/滚轮/光标点击定位）；vi 模式
    键面自然可达；**not-yet 边界**（I3 随注）= 剪贴板族
    （NullClipboard——Ctrl+C/V 经宿主剪贴板 not-yet）、折叠 gutter
    点击/搜索面板跳转（宿主面板族）、Esc 编辑器语义（select/popover
    关闭臂优先）；④menubar 族 a2r 降层 v1 边界：disabled 项隐藏非置灰
    （Button builder 无 disabled 通道）+ trigger/面板静态样式（VM 开态
    差分色 not-yet）。


- 2026-09-22 · stage: review · r1 复审（实施会话内复审——独立性声明：
  裁决自工件重建：diff/测试复跑/消费方双探针，不以执行者自述为凭）·
  plan_revision r1 · outcome: **needs_fix（仅 R-1 轻量）** ·
  reviewed_commit = 6939feb82（plan-674-dev）· base = f2e1aa9a1 ·
  dep = auto-down fba6563ed detached · spec_inputs = ui/overview.md
  （SD-01 段）+ ui/design/shell-a2r-seams.md（SD-02 段）·
  acceptance_results = AC-01 PASS / AC-02 PASS / AC-03 PASS（含已裁
  偏差：23 错清零双复现，整段编译过移 P670-D1——2026-09-22 用户裁定
  在案）/ AC-04 PASS / AC-05 PASS / AC-06 PASS ·
  findings = R-1（low·T-01）：RQ codeeditor 臂 `on_cursor: _` 静默
  弃置**消费方声明事件**（auto-edit app.at:176 `oncursor:
  .CursorMoved(i)`）——无 I3 注记无债登记，违计划 §0"允许显式降级
  登记（I3 留痕），不静默缺件"纪律。修正=臂内注释留痕 + P674-D4 债
  登记（§10-4 复审定边界：on_cursor = 最小键入面之外，降级+登记为
  规划内路径；接线随消费方需要另立）。R-2（观察·非阻塞）：tv 档含
  PLAN-596 dep_parity scratch 冷编译族（dep_parity_018 单测本机
  >30min——a2r 腿 env 门关/VM·oracle 腿不涉本批 diff；tf full 配置
  滤除）——预存环境项，无行动。· evidence = 作用域 7 测全绿复现
  （041 翻 None/矩阵/键入闭环/双根勘定/plan039 19/19/vocab 双测/
  menubar 降层）；**tf 3715/3715 全绿**；**tv 3857/3857 全绿**
  （排除域 -E 'not test(dep_parity)'，23.2s 贴基线刻度；排除理由
  如上）；tt 4084/4084 全绿（tt2 后仅 spec 文本变更——树等价复用，
  理由在案）；**消费方 HEAD 二进制双探针**：perf smoke exit 0 双
  实例 2/2（2026-09-22 00:51，auto.exe mtime 00:50=reviewed HEAD）+
  a2r 词汇门 0（grep compile_error 零 + "recognized vocabulary" 零
  命中；首错 E0425=P670-D1 族与记录一致）· next = **work（R-1
  修正）→ 复验 → reviewed**。


- 2026-09-22 · stage: review · r1 复审复裁（R-1 修正后）· outcome:
  **pass** · reviewed_commit = R-1 修正提交（plan-674-dev @
  6939feb82+1，仅 native_projector.rs 注释 4 行——行为零变化）·
  evidence = 修正后作用域三测复绿（键入闭环/矩阵/041 门）；全量证据
  沿 6939feb82 等价复用（树差异仅注释行——tf 3715/tv 3857/tt 4084/
  消费方双探针收据不失效）；P674-D4 已入册（KNOWN-DEBT :2595）·
  next = **merge**（/auto-plan:merge——SD-01/SD-02 随库落账）。

## 10. 待澄清事项

1. **codeeditor 投影策略**（阻塞 T-01，T-00 裁定）：canvas 位图快照通道
   （PLAN-034 D4 先例——过线快、文本交互合成）vs 结构 DrawOps（保真好、
   op 流量风险）。倾向位图起步（消费方 L1 只需结构+存活、L2 需稳定满帧）。
2. **词汇表单源**（阻塞 T-03，T-00 裁定）：View IR builder 方法面
   （能力即词汇，解释态同源）vs aura schema（声明即词汇，schema_drift
   围栏可依）。倾向 builder 面（a2r 发射的语义本源），schema 侧随收口
   对齐。
3. **not-yet 全集批次边界**（T-00 勘定后定）：全集中本批修多少 vs 登记
   另立——按消费方解阻最小集（codeeditor 必修）+ 顺手族（tree/menubar
   如同根）划界，其余登记。
4. **交互回传深度**：RQ 形态 codeeditor 键入回传最小面（消费方 smoke
   探针所需）vs 完整交互保真——允许显式降级登记（I3 留痕），review 时
   定边界。
