---
plan_id: PLAN-507
status: archived                 # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: desktop-protocol-stage5-coverage
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "desktop_protocol/coverage.rs::Coverage::target_set（500 建立）: 扩容至 Tier1+2 全量（17 kind + 32 layout + typography 折叠）——auto 探测可投影集与元素登记表一致性由单测钉"
  - ".cargo/config.toml::cargo t 别名: 日常档携带 --features ui-iced（desktop_protocol 套件进日常；tf 语义不动）"
  - "tests/schema_drift.rs: 围栏新增 queue 覆盖维度（queue_coverage_drift_fence——aura.at ↔ 元素登记表双向同步）"
new_spec_components:
  - "aura/element_coverage.rs: 元素级覆盖登记表——388 element 单一事实源（covered/not-yet(reason)/not-consumed(reason) + 覆盖率计数），无 feature 门（日常档围栏可读）"
  - "desktop_protocol/client_runtime.rs: Tier1 臂族（icon/badge/avatar/progress/divider/separator/spacer + checkbox/switch/radio/textarea Toggle 命中区〔handler 拥有状态变更〕+ grid cols 等宽网格 + card 表面缺省档）+ 焦点 accent/禁用乘暗差分 + Tier2 typography 缺省档（small/heading 字号档、pre 族底盒、blockquote 引用条）+ 容器 z 序修正（bg 先于子级）"
  - "test/parity/matrix/: parity 金样矩阵——覆盖表驱动 5 夹具 × 两阶段（初帧→规范交互→复帧）全精度锁 + 防漏钉（夹具扫描并集 ⊇ target_set）"
  - "examples/ui/p507-tier-coverage: Tier1+2 全家福构造示例——实机 queue e2e 语料（真实双进程 broker 端到端）"
touched_goals:
  - "GOAL-009: 虚拟桌面桌面协议 Stage 5——RenderQueue 覆盖爬坡 Tier1+2 全量（69/388）+ 第三端 parity 债自动闸门（未登记即红）"
  - "GOAL-007: AutoUI 跨端视觉一致——parity 金样矩阵化（防漏钉）+ 容器 z 序缺陷修复（500 逃逸）"

affects: [auto-lang/ui]
current_step: 10
total_steps: 10
---

# [PLAN-507] 桌面协议 Stage 5——RenderQueue 全 widget 覆盖爬坡 + parity 日常门禁

## 变更摘要

承接 500（Stage 4，已归档：载荷二态/三态开关 `desktop_render:`/AppProjector
块流布局引擎/001–005 子集端到端/parity_001 三臂金样，77/77 绿）。本期两件事：

1. **覆盖爬坡**：`schema/aura.at` 全量 **388 个 element** → 分级降维——
   **Tier 1**（display/form/layout 常用族，~40–60 kind）+ **Tier 2**
   （typography/语义容器，块流简单映射）全量覆盖；**Tier 3**（长尾与复合
   组件：编辑器/chart/diagram 家族）显式 not-yet 表 + auto 降级长期共存，
   复合组件是否永久 not-yet 逐项裁定。
2. **parity 门禁进日常档**：金样从 parity_001 单条扩为**覆盖表驱动的抽样
   矩阵**；并把 `cargo t desktop_protocol --features ui-iced` 纳入日常档
   （500 复审实证 `cargo tf` 不带 ui-iced——门禁盲区家族，本期收口）；
   **覆盖表漂移检测**：registry/aura.at 变更 ↔ Coverage 表同步告警——
   新 widget 默认 not-yet 必须显式登记，防"第三端 parity 债"烂尾。

## 目标

- **G1 分级清单**：388 element 三级清单成文（T1 产出，按使用频率×家族依赖
  排序），Tier1/Tier2 覆盖目标集、Tier3 not-yet 政策逐项注记。
- **G2 Tier1 覆盖**：常用 display/form/layout 族在 queue 模式渲染正确
  （含交互命中区/焦点态/禁用态），抽样示例 re-exec 端到端绿。
- **G3 Tier2 覆盖**：typography（h1–h6/b/em/i/code 等）与语义容器
  （header/footer/article/aside/nav/main 等）块流映射覆盖。
- **G4 Tier3 政策**：not-yet 显式表 + `auto` 降级链路复核（覆盖不足 →
  independent + 观测行）；复合组件（code_editor/autodown_editor/markdown/
  chart/diagram 家族）逐项裁定"永久 not-yet / 后续专项"。
- **G5 parity 门禁日常化**：金样矩阵按覆盖表抽样扩容；desktop_protocol
  （ui-iced 特性）进 `cargo t` 日常档；覆盖表漂移检测入 schema 三件套
  同款治理位。
- **非目标**：Stage 6（默认策略/远程端消费）；复合组件的实现级 lowering
  （仅裁定政策）；性能调优专项；vue 端行为对齐（I4' 登记义务不变，行为
  对齐仍在 vue 侧自身线）。

## 架构方案

```
aura.at(388) ──T1 分级──▶ Tier1(~50) / Tier2(~80) / Tier3(not-yet 表)
AppProjector(client_runtime.rs, 500 块流引擎) ──爬坡──▶ Tier1+2 臂+命中区
Coverage 表 ──驱动──▶ auto 降级判定(既有) + parity 金样抽样矩阵(T4)
门禁: cargo t 日常档 ⊕ desktop_protocol(--features ui-iced) ⊕ 漂移检测
```

- **爬坡机制**：每 widget 臂 = 投影 lowering + 命中区推导 + 覆盖表登记 +
  金样（家族抽样而非逐 widget 全量金样——矩阵规模可控）。
- **防烂尾刹车**：漂移检测 = aura.at element 集 vs Coverage 表 diff 测试
  （新增未登记 → 红；登记 not-yet → 过）——parity 债从此有自动闸门，
  这是"第三端纪律"的机制化落地。

## 技术栈

既有 AppProjector/协议栈/金样体系。零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 500 归档复审记录 + 现场核验 2026-08-31）

- **Stage 4 落点**（爬坡起点）：pac.at `desktop_render:` 字段定名；
  AppProjector 已重写为**块流布局引擎**（NodeStyle 复用 `ui/style`——D2
  定案为"自有块流+样式系统复用"）；T3 五真源 App 同宿主 re-exec；T4 三臂
  基线（queue 臂自动档+实机档）；复审门禁事实 `cargo tf` 不带 ui-iced、
  套件须 `cargo t desktop_protocol --features ui-iced`（77/77）。
- **目标集现状**：`schema/aura.at` 388 个 element（含别名/分层
  tier/builtin_widget 与 web_component 混合）——T1 分级需同时吃 registry
  的 tier/类别元数据（WidgetCategory 已有 Display/Form 分类基础）。
- **I4' 纪律依据**：设计文档 §1.3 三臂 parity 条款（覆盖表 not-yet 显式、
  禁止静默错绘）；I4 双端金样体系为形态先例。
- **排程**：503（reviewed 待合并）/505（execution_done 待复审）/506
  （drafting）在途或待领——本期改动面 = `ui/desktop_protocol/
  client_runtime.rs`（爬坡）+ Coverage/金样/门禁配置，与 503 视觉面、
  505 事件泵段、506 示例批**零交叠**；499（charts v2）若动 svgdoc 渲染
  臂与 projector 无共享文件。可并行。

## 详细设计

### 1. 分级清单（T1 产出，回写本计划）

- 依据：registry WidgetCategory + examples/实际使用频率扫描 + 家族依赖
  （如 button 被 card/dialog 族引用）；
- 预期形态（T1 定稿）：Tier1 = text/button/input/checkbox/switch/image/
  icon/badge/avatar/divider/container/column-row/scrollable/card/list/
  tooltip/menu 族 ≈ 40–60；Tier2 = typography + 语义容器 ≈ 60–90；
  Tier3 = 复合编辑器/markdown/chart/diagram/nav 系/menubar 等 ≈ 其余。

#### T1 定稿（2026-09-01，扫描 registry/aura.at 388 element + examples 93 前端源频率）

全集构成：`schema/aura.at` 388 element = builtin_widget 62 / native_html 50 /
unclassified 21 / web_component 255。examples 高频实证：text 509 / col 398 /
row 331 / span 267 / button 243 / spacer 32 / input 26 / link 20 / center 18 /
grid 18 / checkbox 15 / textarea 9 / icon 7 / divider 6 / avatar 6 / svg 6 /
badge 6 / nav-item 5 / nav-group 5 / image 4 / progress 4（card/tooltip/menu/
switch/select/list 均 0 用例——card 由 col+bg-card 样式组合表达）。

- **Tier 1（40 element，本期 covered）**
  - 文本族 10（归一 text）：text h1 h2 h3 h4 h5 h6 p span label
  - display 11：button image img a icon badge avatar progress divider
    separator spacer
  - form 5：input checkbox switch radio textarea
  - layout 7：col row center container grid grid-item scroll
  - card 族 7（web_component 升格）：card card-action cardcontent
    carddescription cardfooter cardheader cardtitle
- **Tier 2（29 element，本期 covered，块流映射）**
  - typography 13：b em i strong small code pre blockquote quote heading
    codeblock codepane figcaption
  - 语义容器 16：article aside footer header main nav section figure
    details summary ul ol li dl dt dd
- **Tier 3 not-yet（73 element，显式裁定见步骤 7）**：复合编辑器（code_editor
  autodown_editor markdown dyn embed_block math_block query_block callout
  toast-provider toaster）、chart/diagram/SVG 部件族 17（svg canvas circle
  ellipse line path polygon polyline rect g defs mask use stop
  linearGradient radialGradient clipPath）、overlay 弹层族（tooltip×4
  popover hover_card select spinner …）、nav 系（nav-group nav-item nav-link
  taskbar menubar link）、表格族（table tbody thead tfoot tr td th）、
  其余 builtin/native 长尾。
- **n/a（246 element）**：web_component（shadcn 长尾）不由 queue 臂消费——
  card 族/card 外 2 员（grid-item switch）升格除外；升格需显式登记（漂移
  闸门强制）。
- 覆盖基线：covered 69 / 388（17.8%）；组件 kind 口径（文本族归一后）
  Tier1=31 kind + Tier2 两族。Tier1 定稿 40 ≤ 70 阈，不切半。
- 与预期形态的差异注记：tooltip/menu 族按扫描裁定降入 Tier 3（overlay
  弹层，块流静态帧不可保真；examples 零用例）；list/list_item 不入
  Tier1（iced=fallback 且 examples 零用例，`for` 构造才是真实列表路径，
  归 Stage 6）；`switch`/`radio` 零用例但同 checkbox 机廉价覆盖，保留。

### 2. 投影器爬坡（client_runtime.rs）

- 每 widget：NodeStyle 映射扩展 + lowering 臂 + 交互命中区（form 族：
  焦点/禁用态的命令差分）+ Coverage 登记；
- 块流引擎不动结构——只加臂与样式映射（500 的 NodeStyle 管线扩展点）。

### 3. 覆盖表与漂移检测

- Coverage 表（500 已有结构）扩为 element 级全集视图：covered | not-yet
  (reason) | n/a（web_component 类不由 queue 臂消费的）；
- 漂移测试：`aura.at element 集 ⊆ Coverage 表`（未登记即红）——落
  `crates/auto-lang/tests/` 既有 schema 三件套同族位置。

### 4. parity 金样矩阵

- 抽样规则：Tier1 每家族 ≥1 条全交互金样、Tier2 每语义组 1 条、not-yet
  不产金样（覆盖表即证据）；矩阵由覆盖表生成清单（防漏）。

### 5. 门禁日常化

- `cargo t` 日常档纳入 desktop_protocol 套件（`.cargo/config.toml` 别名
  增补 `--features ui-iced` 的 scoped 用法或独立 gate 行——以 nextest
  profile 实际能力定，T6 定案）；复审档 `cargo tf` 的 ui-iced 缺口以
  文档注记 + 日常档覆盖达成（不动 tf 语义）。

## 测试设计

1. **T1 清单成文**：分级表回写本计划（复审对照物）。
2. **T2 爬坡单测**：每 widget lowering 臂的 round-trip/渲染快照断言
   （家族参数矩阵：状态×prop 关键集）。
3. **T3 re-exec 集成**：Tier1/Tier2 抽样示例（001–005 + 新构造覆盖示例）
   queue 模式端到端；auto 降级链路（构造 not-yet app → independent +
   观测行）。
4. **T4 parity 矩阵**：金样抽样全绿（三臂对拍）。
5. **T5 漂移检测**：表与 aura.at 同步（红→登记→绿演练一次）。
6. **T6 门禁演练**：`cargo t` 日常档一跑即含 desktop_protocol 结果；
   覆盖率数字（covered/total）输出可读。

## 验收标准

1. Tier1+Tier2 目标集 covered（覆盖率 ≥ 计划定稿值，复审对照 T1 清单）；
   Tier3 not-yet 表逐项有裁定。
2. T2–T5 全绿；T6 门禁演练留痕（日常档含套件 + 覆盖率输出）。
3. 漂移检测红→绿演练成文；schema 三件套不回归。
4. `cargo t ui`、`cargo t desktop_protocol --features ui-iced` 不回归；
   `cargo check -p auto-lang` 零警告。
5. 既有 001–005 与 auto 降级行为零变化（500 行为回归）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **分级清单**：扫描 registry/aura.at + examples 使用频率 → Tier1/2/3
   清单回写本计划 + 覆盖基线数字。
   验证：清单成文（T1 对照物）。
   [✅ 已完成] 详细设计 §1「T1 定稿」回写：Tier1=40/Tier2=29/not-yet=73/n-a=246，
   覆盖基线 69/388（registry+aura.at 四 tier 计数 + examples 93 源频率扫描实证）。
2. **漂移检测**：Coverage 表扩 element 全集视图 +
   `crates/auto-lang/tests/` 漂移测试（未登记即红）。
   验证：`cargo test -p auto-lang --test schema_drift`（或新测试文件）绿 + 红演练。
   [✅ 已完成] `src/aura/element_coverage.rs`（388 条登记：covered 17 /
   not-yet 125 / not-consumed 246）+ `tests/schema_drift.rs::
   queue_coverage_drift_fence` 双向同步 + 覆盖率输出；绿跑 2/2 + 红演练
   （注释 text 条目→红列出未登记名→恢复→绿）实证；零新增警告。
3. **Tier1 爬坡（一）display 族**：`ui/desktop_protocol/client_runtime.rs`
   text/image/icon/badge/avatar/divider/container/scrollable 臂 + T2 单测。
   验证：`cargo t desktop_protocol --features ui-iced`。
   [✅ 已完成] 85/85 绿（nextest 档）：icon/badge/avatar/progress/divider/
   separator/spacer 七臂 + container/scroll 登记（catch-all 容器臂）+ img
   入 kinds；折叠键匹配贯通（normalize_kind/layout_node 同源）；
   target_set +8 kind/+2 layout；登记表翻 covered 27/388；T2 单测 6 条
   （家族参数矩阵）+ covered⊆target_set 一致性钉。注：libtest 并行档
   demo::counter_loopback_demo_parity 有 wid 计数器竞态（pre-existing，
   单跑两版本均绿——复审记 KNOWN-DEBT）。
4. **Tier1 爬坡（二）form/交互族**：input/checkbox/switch/button 族含
   焦点/禁用态差分与命中区 + T2。
   验证：`cargo t desktop_protocol --features ui-iced`。
   [✅ 已完成] 93/93 绿：checkbox/switch/radio/textarea 四臂 + Toggle 命中
   区（Bool 翻转/radio 恒真/onclick+onchange 双键——013/024 真源 onclick
   优先）；焦点态 accent 描边差分（input/textarea）；禁用态乘暗 +
   命中区不登记（button/input/checkbox/switch/radio/textarea 全族）；
   target_set +4 kind；登记表 covered 31/388；T2 单测 8 条；500 降级
   夹具 checkbox→select/svg 更新（checkbox 已 covered）。
5. **Tier1 爬坡（三）layout/复合容器**：column-row/card/list/tooltip/menu
   族 + T2。
   验证：`cargo t desktop_protocol --features ui-iced`。
   [✅ 已完成] 97/97 绿：grid cols 等宽网格臂（cols/gap prop 真源口径——
   011/016）+ card 表面缺省档（底色+边线+16 内边距）+ cardheader/title
   等块流容器；list/tooltip/menu 按 T1 定稿裁定 Tier3（零用例 + 弹层
   不可保真，步骤 7 裁定）；**容器 z 序修正**（500 逃逸：bg 排子级后 →
   顺序栅格化盖住子级，003 卡片实机 = 空底板——修正为 bg 先于子级，
   幂等重排先例；parity_001 无 bg 容器金样不变）；target_set +9
   layout；登记表 covered 40/388（Tier1 全量）；T2 单测 4 条含 z 序
   回归钉。
6. **Tier2 爬坡**：typography + 语义容器批量臂（块流映射为主）+ T2。
   验证：`cargo t desktop_protocol --features ui-iced`。
   [✅ 已完成] 100/100 绿：typography 13 员文本缺省档（b/strong——bold
   字重不载保真边界同 500 font_bold；small 12px/heading 24px/code 档/
   pre 族底盒+换行保留/blockquote 引用条/figcaption）+ 语义容器 16 员
   块流登记（列表标记不载——保真边界）；normalize_kind 折叠
   typography→text；target_set +16 layout；登记表 covered 69/388
   （Tier1+Tier2 目标集全量达成）；T2 单测 3 条。
7. **Tier3 政策**：not-yet 表逐项裁定（复合编辑器/chart/diagram/nav 系）
   + auto 降级链路复核用例。
   验证：`cargo t desktop_protocol --features ui-iced`（降级用例绿）。
   [✅ 已完成] 101/101 绿（降级用例绿）。裁定表（入 element_coverage
   reason 串）：**永久 not-yet** = chart/diagram SVG/canvas 族 17 员（已对表
   499：canvas/svgdoc 自有管线承接，drafting 中、独立渲染面成立）+ 复合
   编辑器 3 员（code_editor/autodown_editor/markdown——editor_frame/
   pixels 独立通道）+ 瞬态浮层（toast/toaster/notification——宿主通道）；
   **后续专项** = 内容复合块 5 员（dyn/embed_block/math_block/
   query_block/callout）+ nav 系 9 员 + 表格族 7 员 + 树形 2 员 + 媒体
   5 员 + overlay 弹层族（待 Stage 6+ 弹层支持）+ 长尾按需升格。
   auto 降级复核：6 代表族（overlay/chart/编辑器/nav/表格/浮层）→
   Pixels + 观测行缺项证词逐一断言。
8. **parity 矩阵**：金样抽样扩容（覆盖表生成清单）+ 三臂对拍。
   验证：金样套件绿（parity_001 同档）。
   [✅ 已完成] 103/103 绿（金样套件含矩阵）：覆盖表驱动 5 夹具（Tier1
   display/form/layout-grid-card + Tier2 typography/semantic）× 两阶段
   （初帧 → 规范交互[首命中区点击+输入] → 复帧）全精度锁，
   test/parity/matrix/ 5 金样 + parity_001 序列化共用化；
   **防漏钉**：夹具扫描标签并集 ⊇ target_set（kinds+layouts，if 构造
   除外）——扩容必带夹具否则红（cardaction 缺夹具当场红实证）。三臂
   口径同 500：queue 金样（本套）+ vue 臂 a2vue 同族 + iced 实机档。
9. **门禁日常化**：`.cargo/config.toml` 或 nextest profile 调整 +
   覆盖率输出 + T6 演练。
   验证：`cargo t` 一跑含 desktop_protocol（留痕输出）。
   [✅ 已完成] T9 定案：nextest 单调用不能混 feature 集 → `cargo t` 整档
   携带 `--features ui-iced`（desktop_protocol 进日常；演练 4304/4304
   绿 49.5s，desktop_protocol 用例在留痕输出可见）；tf 语义不动（盲区
   注记入 alias 注释——复审清单须另跑 scoped 套件）；覆盖率输出：
   `cargo nextest run -p auto-lang --test schema_drift queue_coverage
   --success-output immediate` → `[queue-coverage] covered 69 / not-yet
   73 / not-consumed 246 / total 388（69/388 = 17.8%）`（命令入档
   .cargo/config.toml 注释）。
10. **re-exec 全量 + 实机 + 收尾**：T3 抽样示例全绿；实机 queue 模式跑
    Tier1+2 示例集；健康检查；状态翻 execution_done。
    验证：`cargo t desktop_protocol --features ui-iced && cargo t ui`。
    [✅ 已完成] `cargo t desktop_protocol --features ui-iced` 103/103 绿
    + `cargo t ui` 1641/1641 绿 + schema_drift 2/2 + docs_gen 4/4 + 触面
    文件零警告。**实机 queue 模式**：新构造示例 `examples/ui/
    p507-tier-coverage`（Tier1+2 全家福）入 `t3_examples_queue_end_to_end`
    ——真实双进程（子进程 `t3_child_body`）× 命名管道 broker × 会话窗，
    queue 模式端到端：首帧全家族文本到位（heading/badge/card/grid/
    typography/语义容器）+ checkbox 点击 → `.ToggleOk`（真源 if/else 自翻
    模式）→ if 块 feat:ON→OFF 帧差断言。**Toggle 语义修正**（实机实证
    逃逸）：handler 在场 = handler 拥有状态变更（024 真源自翻模式；
    投影器自动翻转与之对冲成双翻），无 handler 才自动翻转——e2e 首跑
    红当场暴露。001–005 行为回归绿（climb 测试 + parity_001 金样不变）。

## 复审记录

**复审人**：zcode（/auto-plan:review，2026-09-01）；分支 `plan-507-dev`
@5db2e12a1（10 提交，merge-base 96cb06782；master 期间前进至 b4c670643
——文件交集仅 stage3.rs 且为异地追加，零冲突预判）。

**验收标准逐条重验**（verify, don't trust——全部命令复跑）：

1. **Tier1+Tier2 目标集 covered** ✅ —— 登记表独立重算（不复用执行期
   数字）：covered=69 / not-yet=73 / not-consumed=246 / 总 388；T1 定稿
   40 员与 T2 29 员逐一 ⊆ covered，零计划外 covered，登记集 = aura.at
   全集，not-yet 无空理由（18 员长尾默认理由在案）。Tier3 73 员裁定
   逐项入 reason 串（永久 not-yet 25 / 后续专项 / 按需升格分层）。
2. **T2–T5 全绿 + T6 门禁演练留痕** ✅ —— `cargo t desktop_protocol
   --features ui-iced` 103/103；漂移红→绿演练（步骤 2 证据）+ 本次
   fence 复跑绿；`cargo t`（含 ui-iced）4304/4304，desktop_protocol
   用例在留痕输出可见；覆盖率命令输出 `[queue-coverage] covered 69 /
   not-yet 73 / not-consumed 246 / total 388（17.8%）`。
3. **漂移检测成文 + schema 三件套不回归** ✅ —— `cargo tf`（复审全量
   门禁）**3337/3337**（含 1M churn 档 + schema_drift 2/2 + docs_gen
   4/4 + component_registry）。
4. **scoped 套件不回归 + 零警告** ✅（口径注记）—— `cargo t ui`
   1641/1641；`cargo check` 触面文件 5 处 warning 经 merge-base 溯源
   全部为 500 期存量（PUMP_T0/t0/PageFaultCount/unused-e——行号漂移
   非新增）；「零警告」实义 = 零新增（仓库基线 160 条存量）。
5. **001–005 与 auto 降级零变化** ✅（一处显式偏差记录）——
   parity_001 金样文件零字节变动（diff 实证）；001–005 climb 测试绿；
   auto 降级机制不变（500 测试夹具 checkbox→select/svg 更新 = 覆盖
   增长的预期结果，机制断言等价）。**显式偏差**：容器 z 序修正改变
   003/004 容器 bg 的算子顺序（bg 先于子级）——这是修复 500 逃逸缺陷
   （顺序栅格化下 bg 盖子级，003 实机表现 = 空底板），经 e2e/金样/
   z 序回归钉三重锁定，非行为回归；Toggle 语义（handler 在场 = handler
   拥有状态变更）为 T4 新增语义，500 无此面。

**遗漏/延后/workaround 清点**：无 silent deferral——Tier 边界三处调整
（tooltip/menu 降 Tier3、list/list_item 不入 Tier1、chart/diagram 永久
not-yet 对表 499）均在 T1 定稿节成文且由计划「待澄清 Tier 边界」条款
sanction；TODO/HACK/FIXME 扫描零命中；复审清场动作：scratch/p507
一次性生成器撤出分支（repo 惯例 scratch 不入库）。债务入账 P507-1
（保真边界集）/P507-2（demo wid 计数器 libtest 竞态，pre-existing）/
P507-3（覆盖率数字默认档不可见）→ KNOWN-DEBT-AND-RISKS.md。

**复审过程事故记录（透明）**：复审中一次 `git stash pop` 误弹共享栈中
Plan 473 遗留 parked stash（`plan-473-merge-park`）致 5 文件冲突——
立即 `git reset --hard` 复位工作树至 5db2e12a1，**stash 条目完整保留
在栈中**（未丢失），工作树复验（check + fence 绿）。教训：共享 stash
栈的仓里裸 `git stash pop` 前必须先 `git stash list` 核对。

**结论**：五条验收全过、无阻断债务 → `status: reviewed`，移交
/auto-plan:merge。

## 待澄清事项

- **Tier 边界**：T1 清单定稿数值（预期 Tier1 40–60/Tier2 60–90）以扫描
  结果为准；若 Tier1 超 70 需再切半防单计划膨胀（可拆两批折叠）。
- **复合组件裁定基准**：chart/diagram 家族的 queue 化若被 499/502 的
  canvas/SVG v2 路线覆盖（独立渲染面），则永久 not-yet 合理化——T7 时
  与 499 落地形态对表一次。
- **门禁形态**：`cargo t` 别名能否 scoped 增 feature（nextest profile
  实际能力）——不能则退路 = 独立 gate 别名（`cargo td`？）+ 复审清单
  注记，T9 定案。
- **金样成本**：三臂对拍的 vue 臂依赖 vue 侧渲染稳定性——若 vue 臂抖动
  （字体/AA 差异）允许像素容差档位调整，沿用 a2vue 金样容差先例。
- 排程：与 503/505/506 零交叠核对在 T2 时复查一次（它们可能先合入移动
  行号——grep 重定位）。
