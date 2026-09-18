---
plan_id: PLAN-641
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: tabs-variant（Tabs 组件 variant 扩展：enclosed 连通形态）
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/design/tabs-components.md
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径
current_step: 6
total_steps: 6
---

# [PLAN-641] tabs-variant —— Tabs 组件新增 enclosed（连通）形态

## 0. 变更摘要

Tabs 组件新增 `variant` 属性，提供第二种形态：

- `variant: "default"`（默认）——现状按钮托盘观感，**零回归**；
- `variant: "enclosed"`——连通形态：激活 tab 与下方内容区共享背景、无接缝；
  非激活 tab 为扁平方块，仅以背景色深浅区分（非按钮）。

Chrome 观感（激活 tab 顶部大圆角、tab 间留缝）与 JetBrains IDE 观感（直角、1px
竖分隔线）是**同一 variant 的装饰差异**，走既有圆角/边框 token（`rounded-t-*`
vs 无），不扩充词表。

落地面：Rust `View::Tabs` 单节点 + A2UI 线协议 + iced 渲染臂 + aura schema /
Vue 生成器 / registry 组件两端同步；gallery 增示例覆盖 default / enclosed 直角 /
enclosed 圆角三种观感并做双端一致性验证。

## 1. 目标

- **G1**：`tabs` 根节点（复合标签 `tabs` 与 Rust `View::Tabs`）接受
  `variant` ∈ {`default`, `enclosed`}，默认 `default`，非法值回退 `default` 不 panic。
- **G2**：两端（Vue 模式 `auto run` / VM 模式 `auto run -r vm`）对 `enclosed`
  渲染出同一结构契约：①激活 tab 与内容面板共享背景、中间无接缝；②非激活 tab
  扁平（非按钮/浮起芯片）、仅背景色差；③tab 条与内容区层次分明。
- **G3**：装饰差异由既有样式 token 承载：顶部圆角 token → Chrome 观感，
  无圆角 → IDE 观感；不新增 variant 取值。
- **G4**：gallery 增示例覆盖三种观感（default / enclosed 直角 / enclosed 圆角），
  双端一致性通过 autoui-verifier。
- **G5**：spec 沉淀：tabs variant 契约与词表治理规则入
  `docs/specs/auto-lang/ui/design/tree-components.md`。

**非目标**：

- gpui 后端的 enclosed 完整实现（仅编译适配，降级保持现状渲染）；
- jet/ark（Android）codegen 的 variant 映射（`ui_gen/widget/registry.rs:1009`
  Tabs 注册无 props，本期不动）；
- tabs 结构性新能力（可关闭 ×、新增 +、拖拽重排）——未来以独立 props 承载；
- `render_support.rs` 其余 fallback 族（accordion/modal/select 等）的补全。

## 2. 架构方案

**词汇裁定**（用户 2026-09-18 会话确认）：形态差异用 `variant`——与仓库既有一级
词汇一致（Button/Badge/Card/Toggle/Alert/Sidebar 均以 variant 承载形态级差异，
`aura/schema.rs:514` Plan 435 P2 注记：variant/size/icon 是实现与 gallery 实际
消费的 prop）。不用 `kind`（blueprint 分类学/语义内容类型）、不用 `position`
（方位轴，正交）、不拆新组件（行为重复）。

**variant 挂载点**：tabs 根节点（复合标签 `tabs` / Rust `View::Tabs`），不挂
`tabs-list`——连通契约涉及内容面板，整个复合组件需感知；与 Card/Sidebar 把
variant 放根上一致。

**数据流**：

```
.at 复合标签 tabs(+tabs-list/tabs-trigger/tabs-content, variant prop)
  ├─ Vue 模式: ui_gen/vue.rs 生成 → registry 组件(tabs/*.vue, reka-ui) → variant 切 class 结构
  └─ VM 模式: a2ui 线协议 Tabs{variant} → View::Tabs{variant} → iced renderer 臂 → 连通结构
```

**enclosed 结构契约**（两端共享的语义，装饰参数归 token 层）：

| # | 契约 | token 归属 |
|---|---|---|
| 1 | 激活 tab 与内容面板共享背景、无接缝 | 颜色（bg=content bg） |
| 2 | 非激活 tab 扁平等高 cell，仅背景色差 | 颜色 |
| 3 | 激活 tab 顶部圆角（有→Chrome / 无→IDE） | `rounded-t-*` / border-radius |
| 4 | tab 间分隔（缝隙 vs 1px 竖线） | 边框/间距 token |

**依赖关系**：PLAN-640（drafting）动的是 blueprints/ + Block-Gallery 自动化，
与本案 examples 侧 gallery 示例不同子树，无合并顺序硬依赖；示例落位若与
640 的 gallery 路径交集，在 T-01 勘定后协调（见 Q3）。

## 3. 技术栈

- Rust：`crates/auto-lang/src/ui/view.rs`（View 枚举/Builder）、
  `crates/auto-lang/src/a2ui/{schema,import,export}.rs`（线协议）、
  `crates/auto-lang/src/ui/iced/renderer.rs`（渲染臂）、
  `crates/auto-lang/src/ui/aura/schema.rs`（元素注册表）。
- Vue/TS：`packages/widgets/registry/tabs/*.vue`（reka-ui 派生注册件）、
  `crates/auto-lang/src/ui_gen/vue.rs`（生成器）。
- 验证：cargo check / cargo t（iced/ui 模块）/ cargo tv（VM 语料 golden）/
  cargo tf（终门，触及 a2ui 协议定义）；autoui-verifier 双端一致性。

## 4. 需求分析与背景调查

**授权记录**（2026-09-18 用户会话）：用户确认 variant 属性方案（default +
enclosed；Chrome/IDE 差异走装饰 token 不扩词表），批准口径="tabs 根节点加
variant、两端渲染器落实连通结构、gallery 加示例覆盖两种观感"。L1 级任务。
无特别预算/自动续跑限制授权。

**现状调查**（证据均实测于本仓 master@776fe8f6c）：

1. **Rust 单节点**：`View::Tabs`（`ui/view.rs:767`：labels/contents/selected/
   position/on_select/style）+ `TabsBuilder`（view.rs:3105）。`AbstractView`
   即 `View` 别名（`ui/iced/renderer.rs:8`）。iced 渲染臂
   `ui/iced/renderer.rs:4871` 现状为裸 `button(text(...))` + 激活项 `[label]`
   括号——"多个按钮"观感的来源；`position` 字段被忽略（`position: _`）。
2. **VM 运行时缺口**：复合标签 `tabs`/`tab`/`tabs-list`/`tabs-trigger`/
   `tabs-content` 在 VM/iced 侧为 fallback 渲染成 Column
   （`ui/render_support.rs:278`"tabs component not implemented"）；A2UI 线协议
   `Tabs` 节点（`a2ui/schema.rs:161`）经 `a2ui/import.rs:511` 折叠成 `tabs`/`tab`
   标签后同样落入该 fallback。
3. **Vue 端已实现复合路径**：生成器 `tabs`→Tabs、`tab`→TabsTrigger
   （`ui_gen/vue.rs:75-76`、tabs 臂 12586+），class fallback 表 9229-9232，
   registry 组件 `packages/widgets/registry/tabs/{Tabs,TabsList,TabsTrigger,
   TabsContent}.vue`（shadcn-vue 派生：muted 托盘 + 激活浮起芯片）。
4. **aura schema**：`tabs` ElementDef（`aura/schema.rs:2954`）现仅
   `defaultvalue`/`class` 两个 prop——无 variant。
5. **token 层就绪**：`IcedStyle` 已解析 `rounded-*`/border-radius
   （renderer.rs:1162 `has_border_radius` / 1172 `effective_border_radius`）；
   Vue 端 tailwind class 直出。
6. **variant 双端消费先例**：iced `convert_button` 消费 variant；vue 生成器消费
   size/variant（schema.rs:515 注记）。
7. **现有使用面**：`examples/ui` 中无 first-class tabs 用例（019-video-app 等
   以手搓 row+chip 模拟 tabs）；`examples/unified-demo/front/pages/tabs.at`
   使用 TabRow/Tab/TabsContent（default 形态零回归的回归样张）。
8. **机械穿透点清单**（View::Tabs 加字段的全部 match 臂）：snapshot_builder.rs:580、
   desktop_protocol/coverage.rs:400/451、vnode_converter.rs:452/564/604、
   gpui/renderer.rs:596、gpui/auto_render.rs:692/1250、view.rs 内 Clone/Debug 臂。

## 5. 详细设计

### 5.1 Rust 类型（T-02）

- `ui/view.rs`：新增 `pub enum TabsVariant { Default, Enclosed }`（derive
  Clone/Debug/PartialEq/Default，置于 `TabsPosition`（view.rs:185）旁）。
- `View::Tabs` 加字段 `variant: TabsVariant`；`TabsBuilder` 加 `variant` 字段与
  `.variant(TabsVariant)` fluent 方法，`build()` 默认 `TabsVariant::Default`；
  view.rs:2274 Clone 臂同步。
- 线协议：`a2ui/schema.rs` Tabs 节点加
  `#[serde(default, skip_serializing_if = "Option::is_none")] variant: Option<String>`
  （向后兼容：旧 JSON 无此字段照常解析）；`import.rs:511` 折叠为 `tabs` 根 prop
  （`AuraPropValue`，值直传字符串）；`export.rs:369` 回传 round-trip。
  非法字符串在渲染端回退 Default（不 panic，见 AC-01）。
- 机械穿透（编译适配，行为不变）：§4-8 清单各臂补 `variant` 绑定或 `..` 展开；
  gpui 两臂明确降级——读字段但按现状渲染（注释注明 PLAN-641 非目标）。

### 5.2 iced 渲染臂（T-03）

- `renderer.rs:4871` Tabs 臂重写：
  - `Default`：保持现状按钮托盘观感（允许顺带把 `[label]` 括号 hack 换为
    等价但更合理的选中标记，以截图比对确认视觉等效；若无法等效则原样保留）；
  - `Enclosed`：条 = 等高 cell 行（复用 row + container 定制背景）；激活 cell
    背景 = 内容面板背景、底部无接缝（条与面板之间不画分隔）；非激活 cell 扁平、
    仅背景色差（取 style muted 语义色）；激活 cell 顶部圆角取
    `effective_border_radius`（圆角 token 有值→Chrome 观感，0→IDE 观感）。
- `render_support.rs:278` tabs 族支持级别 fallback → partial（支持 props 按
  T-01 勘定的最小集：`style`/`variant`/`active`/`value`/`defaultvalue`），
  描述更新。
- 复合标签 → `View::Tabs` 折叠：VM 侧把 `tabs`(+子标签) 折叠为有状态 Tabs
  （最小契约：`defaultvalue` 定初始选中、`value` 标识 tab、点击切选中；
  激活态外部受控绑定若非 variant 所需则不做，防范围膨胀——T-01 定案）。

### 5.3 Vue 端（T-04）

- `aura/schema.rs:2954` tabs ElementDef 加
  `PropDef { name: "variant", type_: PropType::OneOf(["default","enclosed"]),
  required: false, default: Some("default") }`。
- `ui_gen/vue.rs`：tabs 臂（12586）把 variant 下发到生成产物；class fallback
  表（9229-9232）增 enclosed 分支。
- registry `packages/widgets/registry/tabs/*.vue` 加 `variant` prop（默认
  `"default"`）；enclosed 结构：TabsList 托盘背景去除（透明、无缝）、
  TabsTrigger 扁平等高 cell（去浮起阴影，`data-[state=active]:bg-background`
  + 顶部圆角 class 由 token/class 决定）、TabsContent 顶部无圆角无边框。
  文件头注明"Generated by AutoUI from widgets/tabs.at"但 `widgets/tabs.at`
  不在仓内——再生成来源由 T-01 勘定（Q1），若已脱钩则直接维护 registry 文件
  并更新头注 provenance 说明。

### 5.4 gallery 示例（T-05）

- 落位 `examples/component-gallery`（build/vue 结构已存在）或 T-01 勘定的更
  合适位置：一组三块——`variant:"default"`、`variant:"enclosed"`（直角，IDE
  观感）、`variant:"enclosed"` + 顶部圆角 class（Chrome 观感）。
- autoui-verifier 双端跑通 + 截图存档（双端模式命令：`auto run` / `auto run -r vm`）。

### 5.5 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/design/tree-components.md（tabs 节） | before：tabs 仅形态描述、无 variant；after：tabs 根节点 `variant: default\|enclosed` 契约——enclosed 三条结构契约（连通/扁平/层次）+ 装饰差异归 token 层（圆角/边框/颜色） | 形态差异统一走 variant 一级词汇（Plan 435 P2 词表），Chrome/IDE 观感不扩词表 | AC-01,02,03,04 |
| SD-02 | add | docs/specs/auto-lang/ui/design/tree-components.md#tabs-variant | before：无；after：词表治理规则——tabs variant 新增取值须 Vue/iced 双端同落 + gallery 示例覆盖，装饰差异优先走既有 token 不新增取值 | 防 variant 词表膨胀与单端漂移 | AC-08 |

## 6. 测试设计

- **Rust 单测**（随 T-02/T-03 落）：
  - a2ui round-trip：带/不带 variant 的 Tabs JSON export→import 保真
    （AC-05）；
  - `TabsBuilder` 默认 variant = Default；`.variant(Enclosed)` 生效；
  - 非法 variant 字符串 → 渲染回退 Default（AC-01 的 panic 兜底）。
- **VM 语料 golden**：改渲染臂后跑 `cargo tv`（现档 3578 测 / ~20s；纯 .at
  语料，不含 aavm）。
- **双端一致性**：autoui-verifier 技能（AC-06），三观感截图比对。
- **终门**：`cargo tf` 一次（触及 a2ui 协议定义，属 core protocol 范畴，
  按 AGENTS.md Category B 合入前档）。
- **taa 零触发声明**：本案不触及 `auto/lib/*.at`、`test/vm/aavm2/**`、
  `parity/**`、aavm2 测试基建——不跑 `cargo taa`。
- **Category C**：若 T-04 勘定 aura schema 变更牵动文档生成器/schema 参考
  文档，则补跑 `cargo test -p auto-lang --test docs_gen`（T-04 内判断）。

## 7. 验收标准

| ID | 验收标准 | 验证方法与预期 |
|---|---|---|
| AC-01 | tabs 根接受 `variant`，非法值安全回退 | .at 示例写 `variant:"enclosed"`/`variant:"bogus"`；前者生效、后者回退 default 且不 panic；Rust 单测覆盖 |
| AC-02 | VM/iced 模式 enclosed 连通结构 | `auto run -r vm` gallery 示例截图：激活 tab 与内容区同背景无缝、非激活扁平仅色差 |
| AC-03 | Vue 模式 enclosed 同语义 | `auto run` 同示例截图：结构契约三条均满足 |
| AC-04 | 圆角 token 两观感 | gallery 中 enclosed 直角（IDE）与 enclosed+rounded-t（Chrome）两例截图成立 |
| AC-05 | 线协议向后兼容 + round-trip | 旧 JSON（无 variant）解析通过；带 variant export→import 保真；Rust 单测绿 |
| AC-06 | 双端一致性 | autoui-verifier 双端跑通，无阻断差异 |
| AC-07 | 零回归 | unified-demo pages/tabs.at（default）双端渲染与改前一致；`cargo tf` 全绿 |
| AC-08 | spec 沉淀 | SD-01/SD-02 落 tree-components.md；`.autoos/specs.json` upsert + `python scripts/spec-index.py` 通过 |

## 8. 执行步骤

> 执行序：T-01 → T-02 → T-03 → T-04 →（T-05 可与 T-04 后半并行）→ T-06。
> 全程在 worktree `D:/autostack/.wt/lang-641/auto-lang`（branch `plan-641-dev`）内进行；
> 计划簿记留在 master。

### T-01 侦察定案（bounded investigation）[x]

- 操作：
  1. 勘定 VM 侧复合 tabs 标签→有状态 Tabs 的最小折叠契约（激活态绑定：
     `defaultvalue`/`value`/`active` 何者入最小集；`render_support.rs:278`
     fallback 的调用链定位）；
  2. 勘定 `packages/widgets/registry/tabs/*.vue` 的再生成来源
     （`packages/widgets/cli/src` 是否消费某处 `widgets/tabs.at`；源头在仓外
     则记录 provenance 规则）；
  3. 勘定 gallery 示例落位（examples/component-gallery vs 其他）及与 PLAN-640
     gallery 自动化的路径交集。
- 决断产物：结论写入 §9 复审记录（T-01 行）+ 若推翻 §5.2/5.3 假设则按
  修订规则增量 plan_revision。
- 依赖：无。关联：AC-02/03/04/06 的实现前提。
- [✅ 已完成] 2026-09-18：三项定案 + convert_view_messages 新发现见 §9 work
  收执行；gallery 落位改 vue-gallery + examples/ui/046（等价实现，scope 内）。

### T-02 Rust 类型 + 线协议 [x]

- 操作：§5.1 全部（TabsVariant 枚举、View::Tabs/TabsBuilder/Clone 臂、
  a2ui schema/import/export、§4-8 机械穿透清单、gpui 降级注释）。
- 文件：`crates/auto-lang/src/ui/view.rs`、`src/a2ui/{schema,import,export}.rs`、
  `src/ui/{snapshot_builder,vnode_converter}.rs`、`src/ui/desktop_protocol/coverage.rs`、
  `src/ui/gpui/{renderer,auto_render}.rs`。
- 验证：`cargo check -p auto-lang` 零警告零错误。
- 依赖：T-01（仅 Q1 结论不影响本任务则可并行启动）。关联：AC-01/05。
- [✅ 已完成] worktree 75cfcd47c：TabsVariant(parse/as_str/Default) +
  View::Tabs/TabsBuilder.variant() + A2UI serde（None 省略，旧 JSON 兼容）+
  import 折叠 tabs 根 prop/export round-trip + 机械穿透 8 处（gpui 降级注释）；
  单测 4 过（parse 兜底/builder 缺省/wire round-trip/向后兼容）。触碰文件
  cargo check 零新增警告。

### T-03 iced 渲染臂 [x]

- 操作：§5.2（renderer.rs:4871 重写双形态、render_support 支持级别升级、
  VM 侧复合标签折叠）；Rust 单测（round-trip/builder/非法值回退）。
- 验证：`cargo check -p auto-lang`；`cargo tv`；`cargo t iced`（或 ui 模块滤串）；
  VM 模式示例目检截图（AC-02 初验）。
- 依赖：T-02。关联：AC-01/02/05/07。
- [✅ 已完成] worktree ee5a666f0：双形态臂（default 零回归 + enclosed 连通
  结构/theme token/顶角半径/透明化命中面）+ convert_view_messages 显式臂 +
  convert_tabs 折叠（tracked/untracked 双臂镜像）+ render_support partial +
  折叠单测×3。tv 3746/3747（唯一红=real_sidebar_at_parses_with_navtree
  基线预存，主检出对照复证）。VM 实机截图
  examples/ui/046-tabs-variants/src/front/tests/screenshots/p641_vm_v2.png
  （三形态 + 受控切换断言 s2 a→b）。

### T-04 Vue 端 [x]

- 操作：§5.3（aura schema prop、vue.rs 生成器与 class fallback、registry 四件
  variant 化；docs_gen 触发判断）。
- 验证：`cargo check -p auto-lang`；Vue 模式示例目检截图（AC-03 初验）。
- 依赖：T-01（Q1 provenance 结论）、T-02（TabsVariant 无需——Vue 侧独立字符串
  prop，可并行）。关联：AC-01/03/04/07。
- [✅ 已完成] worktree c98fcbb36 + 5bba77fbe + 4271b4e5b：schema.rs tabs
  prop 词表（variant/active/onselect/value）+ vue.rs tabs 臂
  variant/defaultvalue 直传 + registry 四件 variant provide/inject + vue.rs
  内嵌 WidgetTemplate 同步（Q1 补全：库模式生成源头）+ registry 三件补
  slot（顺修自闭和丢子件隐患）+ typed emits 修 update:modelValue 转发；
  vue-tsc 零错误。

### T-05 gallery 示例 + 双端验证 [x]

- 操作：§5.4（三观感示例、autoui-verifier 双端跑通、截图存档）。
- 验证：AC-02/03/04/06 终验（截图）。
- 依赖：T-03、T-04。关联：AC-02/03/04/06。
- [✅ 已完成] worktree 4271b4e5b + 0be62de85：046-tabs-variants 双端示例
  （default/enclosed 直角/enclosed+rounded-t-lg Chrome）+ vue-gallery tabs
  页三观感 demo。双端实机：VM 截图（三形态 + 按钮受控切换
  s2 "a"→"b" 断言 + Beta 激活连通视觉）；Vue 截图（shots/p641_vue_final2.png
  + 点击切换断言 + computed bg rgb(9,14,26)/白字）。schema/aura.at tabs 族
  同步（variant/active/onselect/value props；五件 iced partial）+
  auto-man assets baked patch（第三拷贝源，SNAPSHOT.md 记录）。执行期新发现
  已回写 §9。

### T-06 spec 沉淀 + 复审收口 [x]

- 操作：SD-01/SD-02 落 tree-components.md；`.autoos/specs.json` upsert +
  `python scripts/spec-index.py`；§3 独立复审（checklist audit / 遗漏与
  workaround 扫描 / 零警告 health check）+ 终门 `cargo tf`。
- 验证：AC-07/08。
- 依赖：T-05。关联：AC-07/08。
- [✅ 已完成] worktree e111554e0：规格落位调整——tree-components.md 为树
  组件专文，tabs 契约改落新 design 文档
  `docs/specs/auto-lang/ui/design/tabs-components.md`（variant 词表/VM 折叠
  契约/三源同步纪律/验证基线）+ overview.md 641 条目（等价落位，SD 目标
  面不变）。docs_gen core.md 再生（tab iced partial，Category C 门禁抓漏
  闭环）。`.autoos/specs.json` upsert 留待 /auto-plan:merge 沉淀（本技能
  不动 live ledger）。终门 tf 3600/3601（唯一红=基线预存）。

## 9. 复审记录

- 2026-09-18 draft/rev1 交底（/auto-plan:new）：stage: new，PLAN-641 rev1，
  outcome: pass，next: work。调查基线见 §4；T-01 三个决断点（Q1/Q2/Q3）为
  执行期首要产物，若推翻 §5 假设按修订规则处理。
- 2026-09-18 work 收执（/auto-plan:work）T-01 定案（worktree
  `D:/autostack/.wt/lang-641/auto-lang` @ e352437b0，branch `plan-641-dev`）：
  - **Q2 定案**：VM 侧折叠走**状态受控最小集**——`tabs`（+透明包装
    `tabs-list`/`tabrow`）折叠为 `View::Tabs`；labels 取 `tab`/`tabs-trigger`
    的 `text`/`label`/文本子件，contents 取 `tabs-content` 子件；selected 解析
    序 `active`/`value`（索引或值串匹配）→ `defaultvalue`/`default` → 0；
    点击经既有 `DynamicMessage::Typed{widget_name, event_name, args:[idx]}`
    事件路由回调 store handler（`aura_view_builder.rs:2838` details_onclick
    先例），**不引入 widget 本地状态基建**（对齐 unified-demo tabs.at 的
    状态驱动惯用法）。无 handler 绑定时点击不切换（与受控语义一致）。
  - **新发现（落入 T-03 范围）**：`convert_view_messages`（iced
    renderer.rs:6388）对 Tabs 落 `_ => AbstractView::Empty` 通配臂
    （renderer.rs:6830 注释）——单节点 Tabs 带回调在 VM live 树也被折空。
    须补显式 Tabs 臂：`on_select` 为 Arc 回调，按 Select 同款
    `IcedMessage::from_dynamic` 包装转换。§5.2 随行补此臂。
  - **Q1 定案**：`packages/widgets/registry/tabs/*.vue` 在仓内手工维护
    （Plan 331 CLI 为 copy-out 工具，无再生生成器；头注"Generated by
    AutoUI from widgets/tabs.at"为历史遗留）。T-04 直接改 registry 文件并
    刷新头注 provenance 说明。
  - **Q3 定案**：`examples/component-gallery` 已不被 git track（基线
    e352437b0 无此目录，master 磁盘为未跟踪残留；另有他分支 WIP 改名
    commit 不在本计划范围）。widget 级 demo 落位：
    (a) `examples/vue-gallery/src/pages/tabs.vue` 增三观感 demo 块
    （default / enclosed 直角 / enclosed 圆角）；
    (b) 双端验证新建 `examples/ui/046-tabs-variants/`（空闲号已核）走
    autoui-verifier。PLAN-640 的 `blueprints/` 为页面级 gallery，无交集。
  - outcome: pass（scope 内调整：convert_view_messages 显式臂 + gallery
    落位改 vue-gallery/ui-046，均属等价实现，不增 plan_revision）；
    next: T-02。
- 2026-09-18 work 收执（execution_done）：stage: work | PLAN-641 | rev1 |
  outcome: **pass** | code_commit: 4271b4e5b+0be62de85+e111554e0（worktree
  `D:/autostack/.wt/lang-641/auto-lang`，branch `plan-641-dev`，基线
  e352437b0）| task_ids: T-01..T-06 全勾 | evidence:
  - AC-01 variant 语义+非法回退：单测 test_tabs_variant_parse_and_default
    等 4 测 + 折叠单测×3 绿；
  - AC-02 VM enclosed：046 VM 截图 p641_vm_v2.png（三形态+连通结构）；
  - AC-03 Vue enclosed：p641_vue_final2.png（结构契约三条 + computed
    bg=rgb(9,14,26)/白字断言）；
  - AC-04 圆角 token 两观感：enclosed 直角 vs rounded-t-lg 双例截图；
  - AC-05 线协议：test_tabs_variant_roundtrip/test_tabs_no_variant_backward_compat 绿；
  - AC-06 双端一致性：VM 受控切换断言（press Beta → s2 a→b）+ Vue trigger
    点击切换断言，autoui-verifier 脚本双端跑通；
  - AC-07 零回归：default 形态原样（含 [label] 标记）；tf 3600/3601，唯一
    红=real_sidebar_at_parses_with_navtree 基线预存（主检出对照复证）；
  - AC-08 spec 沉淀：design/tabs-components.md + overview.md 条目
    （specs.json upsert 留 merge）。
  | blockers: 无 | next: review（/auto-plan:review 独立复审；worktree 留存）。
  执行期新增债/发现：①VM 轨大写标签（Col/H1/Row/Button）子树丢失（既有
  行为，046 以小写规避，KNOWN-DEBT 候选）；②docs SFC 三源同步纪律入
  tabs-components.md（registry/vue.rs 模板/auto-man assets，Q1 完整答案）。
- 2026-09-18 独立复审（/auto-plan:review，实现同会话——按技能要求从工件
  重建结论，不采信执行期总结）：stage: review | PLAN-641 | rev1 |
  reviewed_commit: 49d8be5d1（含 R1 修复）| base_commit: e352437b0 |
  dependency_revisions: auto-down detached@362d75b（组内兄弟，路径解析
  只读）| spec_inputs: design/tabs-components.md（新增）+ overview.md 641
  条目 + schema/aura.at + docs/components/core.md（再生）。
  **复审发现与修复循环**：
  - **P641-R1（fail → 已修复）**：trigger 文本子件形态
    （`tabstrigger (value:"a") { text "Alpha" }`）标签静默回退 "Tab N"——
    fold 浅层 Text 匹配漏掉 text-like Element 形态；执行期最后一轮改
    app.at 后未重跑 VM 视觉验证，漏网。修复=复用 convert_text_element
    折叠链（TEXT_LIKE_TAGS + child_element_text props 链下钻一层）+
    回归单测 plan641_tabs_fold_trigger_text_child_element。受影响任务
    T-03 重开即修（修复循环 1/3），commit 49d8be5d1。
  - P641-R2（flake，非回归）：tv 偶发 c1_future_all 红——双端单跑皆绿
    （主检出 PASS / worktree PASS），并发抖动在案。
  **AC 复证结果（全 pass）**：AC-01 单测 parse 兜底 7/7；AC-02/04 VM 全新
  实机复现 p641_review_fix.png（三形态标签正确+连通+圆角对比）+ 受控切换
  断言（press Beta → s2 "b"）；AC-03 Vue computed 样式断言（bg
  rgb(9,14,26)/白字）+ 点击切换；AC-05 wire 单测；AC-06 双端脚本跑通；
  AC-07 tf 3600/3601（唯一红=real_sidebar_at_parses_with_navtree 基线
  预存，主检出对照复证）；AC-08 spec 沉淀在案（specs.json 留 merge）。
  frontmatter new_spec_components 落位修正（tree-components.md#tabs-variant
  → design/tabs-components.md，T-06 落位调整后的簿记同步）。
  规范增量核验：SD-01/SD-02 与 design/tabs-components.md 实文一致，无
  超范围承诺。**outcome: pass** | next: merge。

## 10. 待澄清事项

- **Q1（T-01 决断）**：registry `tabs/*.vue` 的再生成来源——文件头注释指向
  `widgets/tabs.at`，仓内未找到该源文件；若源头在 auto-os 仓或已脱钩为手工
  维护，需记录 provenance 规则（决定 T-04 改 registry 文件的方式与头注写法）。
- **Q2（T-01 决断）**：VM 侧复合 tabs 标签折叠的最小激活态契约——只做
  variant 连通渲染所需最小集（defaultvalue/value/点击切换），外部受控绑定
  （如 store 驱动 activeTab）如需另立任务，防止本计划范围膨胀。
- **Q3（T-01 决断）**：gallery 示例落位与 PLAN-640 的 Block-Gallery 自动化
  是否有路径/构建交集（examples/component-gallery vs vue-gallery vs
  blueprints/）；有交集则与 640 协调落地顺序。
- **Q4（低风险，执行期观察）**：iced `Default` 形态现状激活标记为 `[label]`
  括号 hack——若替换为等效选中标记则需截图证明视觉等效，否则原样保留
  （AC-07 零回归口径以"原样保留"为最稳路径）。


## spec-sync 回写记录

- 2026-09-18（/auto-plan:merge）：SD-01/SD-02 落
  `docs/specs/auto-lang/ui/design/tabs-components.md`（新建，variant 词表/VM
  折叠契约/三源同步纪律/验证基线）+ `ui/overview.md` 641 条目 +
  `ui/plans.md` 行 + INDEX 再生（26 projects）+ `.autoos/specs.json`
  upsert（P641-1 architecture / P641-2 reviews，运行时账本本地投影）。
- 合并收据：master `727b7c9b1`（merge plan-641-dev，基线 e352437b0，实现
  提交 75cfcd47c..49d8be5d1 共 9 个）；主检出冒烟：tabs 单测 8/8 +
  schema_drift 2/2 + docs_gen 4/4。
- 清理：wt-guard clean 后移除 worktree `D:/autostack/.wt/lang-641/auto-lang`
  与分支 `plan-641-dev`（组内 auto-down detached 兄弟一并移除）。
