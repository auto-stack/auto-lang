# ui（AURA / UI 引擎 / 桌面运行时）

> **Status**: active（主战场：vue 轨 codegen 成熟化 + VM 轨视觉 parity + 虚拟桌面线推进中）
> 最近刷新：2026-09-18（PLAN-027 rev2 回写：shell a2r 接缝面 S1/S2 落地——codegen 臂族 + 显式拒绝门/词汇门 + ShellProjection/DesktopBusHandle typed 载体，详见 design/shell-a2r-seams.md（provisional）；design 文档 desktop-shell-a2r.md 裁定落定 = B 形态 + 双轨常驻）
> 2026-09-03（Plan 527 归档回写：VM 轨 Tailwind v3.4 清单驱动全量覆盖契约——清单锚定/静默丢弃关闭/三家族补全/变体管道/对拍审计台常驻；2026-09-02：Plan 522 helper fn 进 vue SFC、516 vue 桌面远程窗、518 桌面视觉二期）
>
> **资产位置注记（PLAN-590，Stage B P-5，2026-09-07）**：桌面域资产已随迁
> auto-os——`ui-gallery`/`widgets-gallery` 在 **auto-os 顶层**（框架侧
> schema_drift/docs_gen/gallery_golden 等语料锚改 `resolve_os_top_dir`
> 解析序定位）；`025-sys-monitor`/`028-launcher`/`038-minesweeper` 与
> `common/settings` 在 **auto-os/apps/**。本页历史段落中的
> `examples/ui-gallery`、`examples/widgets-gallery` 表述为落成时位置，
> 现状以本注记为准（去向详表 `docs/plans/INDEX.md`）。

## 职责

Auto 的 UI 子系统，围绕 **AURA**（UI-IR）组织，2026-08 起扩展为**桌面运行时**：

- **前端解析**：UI 方言（`widget`/`msg`/`model`/`view`/`on`/`setup`/`actions`，dialect 机制）。
- **AURA 提取与校验**：WidgetDecl → 视图树/状态/事件三元 IR，按 `schema/aura.at`（唯一契约源，plan-435）校验。
- **代码生成**（`ui_gen/`）：a2vue（主力）、a2rust、a2jet/a2ark、ts adapter、widget 契约表、block 层。
- **VM 运行时渲染**（`ui/`）：vnode/事件路由/VM 桥接、iced/gpui/headless 后端、code_editor 与
  autodown_editor 内建 widget、热重载、MCP 调试服务。
- **桌面运行时**（2026-08 新增）：`session.rs`（DesktopSession/AppSession 双层会话 + WmState/
  WmCommand 桌面消息）、`ui/iced/virtual_window.rs`（VirtualWindow 渲染）、排布纯函数与
  DesktopBus v0——单 OS 窗口内多 App 虚拟桌面。
- **a2ui 协议** 与 **`#[api]` 前后端契约**（`src/api/`）。

## 现状（2026-09-18）——Select Anything（PLAN-646）

任意 AutoUI 基面框选 → 结构化 Auto/JSON 回吐（知识采集地基）：选择语义
纯函数（中心点包含命中 + 顶层修剪 + 文档序，`ui/selection/`）、结果信封
三格式（Auto 切片/JSON/Atom，`ui/selection/output.rs`）。挂点：VM 端
Alt+拖拽 marquee（`GlobalPress` 臂起笔 + canvas 蒙层 + `__bounds_collected`
延迟计算 + DevTools「采集」标签页三视图+复制）；Vue 端 dev overlay
（`node_to_html` 注入 `data-auto-{tag,src,id,span}`，`AUTOUI_SELECT_MARKERS=1`
门控；脚手架 `auto-sources.ts` 源映射 + `auto-select/overlay.ts`，
main.ts `import.meta.env.DEV` 动态引用）；MCP 工具 `autoui_select_rect`
（共享 styled_vtree + layout_bounds + 随帧 source_code）。契约：
`design/select-anything.md`。`__hot_reload` 泵臂代消费 `needs_bounds`
（静默会话 bounds 采集闭环，Plan 282 补线）。

## 现状（2026-09-18）

**VM 空转渲染减负 easy wins（PLAN-650 落地）**：timer `when` 订阅层门控
（P499-1 调度器半边清偿）+ dirty=false 非 debug 帧旁路（live_vtree/
needs_bounds/input_ids）+ hot_reload 非 debug 默认 2000ms（`AUTOUI_HOT_RELOAD`
0/1 门控）+ MCP 捕获按 dirty/近 30s 活跃收紧。架构级 Element 帧间缓存等
D-1..D-5 延期，设计见 `docs/design/autoui/vm-frame-budget.md`。

**嵌套组件 TimeSource + mounted 过滤订阅（PLAN-652 阶段 1）**：订阅不再
只扫根 `tick_interval()`——装载期收集 root/child/store 的 `.Tick`+`timer`
进 `timesources`；装配期 `AuraViewBuilder` 把实例化 child 名写入
`mounted_types`；renderer 只对「挂载中且 when 通过」的源建
`widget_event_tick` 订阅。条件分支切走后对应 Tick/timer **停订**（类型级
D-2；同类型 for 多实例仍共一条，path 级=阶段 2）。详见
`docs/specs/auto-lang/ui/design/nested-timesource.md` 与设计
`docs/design/autoui/component-time-and-events.md`。

**012-clock 现代时钟应用重构与传统手表表盘（PLAN-644 落地）**：
`examples/ui/012-stopwatch` 升级并重命名为 `examples/ui/012-clock`（Clock 现代时钟应用），深度重构为五大完整功能模块（时钟、世界时钟、闹钟、秒表、倒计时），首页呈现 SVG 矢量传统机械手表 ⌚ 指针表盘与动态角度换算，桌面小组件 `view mini` 升级为迷你手表表盘 + 数字时钟，全面适配 AutoUI Design Tokens 并消除 P642-D6 遗留横幅按钮债务。

## 现状（2026-09-15）

**编译 exe 桌面客户端面（PLAN-020/025/026，provisional）**：desktop_protocol
客户端臂自解释态 `DynamicComponent` 泛化到 `Component` seam——a2r 编译 exe
经 `NativeProjector<C>`（View 运行期投影）作 compositor 一等客户端，native
覆盖集（v1.8：form/payload 族 input/textarea/checkbox/radio/slider/select
+ display 族 image/progress 占位保真 + layouts scroll/grid + 样式降级放行
批（flex-1/shadow/overflow-/min-w-/min-h-/leading- 等——逐类随注
`native_queue_set`）；icon/badge/avatar/divider/separator/spacer/a 经 a2r
codegen 降级归一——分表非缺口；imagesurface 整 kind not-yet）与
`auto`=independent 缺省裁定（026 复测 judged 76.2% < 95%，翻转点已备）、
宿主 `desktop_exe:` 孵化分流、输入路由两端（投影器右键/滚轮/聚焦编辑/IME
闭环消费 + 宿主 `broker_key_event/broker_char/broker_scroll/broker_ime_*`
生产）随册。权威正文 =
`docs/design/autoui/desktop-protocol-v1.md` §1.6–§1.8（本节仅指针，不
重复）；翻转数据行 =
`docs/plans/reports/p026-native-flip-data-row.md`；度量 =
`docs/plans/reports/020-rust-exe-compositor-metrics.md`。

**图像 DrawOp 通道（PLAN-028，provisional）**：`DrawOp::Image{rect, src,
fit}`（tag 6 追加式，`ImageFit` v1 = Stretch，`PROTOCOL_VERSION` 仍 1）
——src 引用 + 宿主侧解析（零位图字节过线、child 免解码）：词汇表 =
本地文件 / `builtin:` / `data:` / `http(s)`（3s 超时）/ `thumbnail://{wid}`
虚拟引用（snapshot SWR 每帧直查，不进永久句柄缓存）；宿主
`broker_surface` 解析面 = 进程级 Handle 缓存（key=src 含负缓存）+
`Frame::draw_image` 首用 + http miss 占位先行/后台解码/下帧翻真（零新增
触发器）+ 未解析占位 + `[drawlist-image]` 观测去重（I3）；解释态
`layout_image`（src 绑定 read_state 代入）与 native `View::Image` 臂同刻度
真图升级（rect 推导零变化；icon lucide 降级形态随臂入线——宿主字形解析
not-yet，P026-D1 字形半句维持）；TS decode tag 6 必达 + 占位渲染 +
web 真位图 not-yet。权威正文 =
`docs/design/autoui/desktop-protocol-v1.md` §1.9（本节仅指针）；测试面 =
`t028_*` 宿主解析单测族 + `p028_image_arm`（AUTO_DESKTOP_E2E）+
`IMAGE_FRAME_HEX` 双侧 golden。

## 现状（2026-09-17）

**examples/ui 全量 Style Recipe 配方化 + 裸调色板色门禁（plan-637 落地，GOAL-007）**：
`examples/ui/stylekit` 为 **canonical 共享配方库**（`hint_text`/`caption_text`/
`input_field`/`icon_base`/`pill` 族等），全部 33 个有源码 demo + 045 完成
Phase A 去重与 Phase B 语义 token 化；demo 经 635 通道消费
（`dep stylekit { path: "../stylekit" }` + `use stylekit.styles: <name>`，
16 demo / 100+ 位点）。**规范纪律**：已完成 rollout 的 demo 禁新增裸调色板色——
门禁 `scripts/style_palette_guard.py` + `scripts/style_palette_manifest.json`
（completed 清单驱动 + 豁免表逐键定性，四类口径：显式品牌色、装饰性插画/
渐变色、语义状态色 registry 无键、图像叠字恒白）。**已知限制**：括号形属性
中的配方引用（`icon (..., style: 配方名)`）在 VM 原生轨判 undefined（组件静默
跳过）——大括号形全轨可用；475 组件包形态（widget 内 `use { package }` 块）
与顶层 stylekit use 解析互斥——两场景暂留字面量，待框架收敛后回迁。

## 现状（2026-09-11）

**声明式样式配方语言层（plan-607 落地，Design 29 Phase 3，GOAL-007）**：
AutoUI 顶层引入 `style <name> = "<classes>"` 常量配方与 `style <name>(<params>...) = "<f-string>"` 参数化配方语法；在 widget 的 `style:` 属性中支持裸标识符引用、参数化调用与数组/条件/插值混用；通过在 AST → AuraNode（`aura_view_builder.rs` / `extract.rs`）单一入口处编译期 Desugar 展开为标准 class 字符串，实现 Vue 与 VM Iced 双端同源、运行时零开销、零视觉漂移；配套硬编码调色板色（如 `bg-blue-500`）lint 警告，并在 013-todo 与 015-notes 中示范重构，收敛存量重复 class 字符串。

**跨包 style recipe 引用（plan-635 落地，GOAL-007）**：recipe 作用域从单编译单元扩展为可经 `use` 符号导入跨包消费——依赖包以 `pub style` 导出（非 pub 仅包内可用），宿主 `use <dep>.<module>: <name>` 命名导入或 `: *` glob 导入（只导 pub），与 widget/fn/store 的 use 同构零新语法；命名导入命中非 pub 配方为编译错误，宿主与导入源同名撞名为编译错误（诊断含双源名）；导入模块自身的 use 传递贡献其 pub recipe（visited 集防环；注：传递收集不校验中间模块 use 的 pub 性——v1 语义，v2 视需要收紧）。实现为「预导入通道」：宿主 parse 前按源码扫描 use 收集并预注册（parser 符号检查钩子只认活注册表），parse 后 clear+按 source 标记受控重放（`design_tokens/recipe.rs` prepare_style_recipe_imports / load_and_validate_style_recipes_with_imports）；VM（build_dynamic_component_inner）与 Vue（ui_gen::api::generate_component_from_file）四注册点同函数族单点，desugar 单点与双端同源承诺不变。示范=examples/ui/stylekit（共享包）+ examples/ui/045-style-import（消费者）。

**vue as-cast 整型降级（plan-604 落地，KD-VM4 双端一致）**：view/handler 内
`expr.as(int)`（及 i64/uint/u64/usize/byte）vue 侧降级 `Math.trunc(...)`，对齐
VM TYPE_CAST_I32 的 Rust `f as i32` 截断语义；浮点目标 JS 原生 f64 直通，其余
类型保持值不变。此前 `Expr::Cast` 三处 emit 无降级（handler 内落 `undefined`、
绑定位硬错误、文本位 R046 占位符）。单测
`ui_gen::vue::tests::test_as_cast_int_lowers_to_math_trunc`。

**ui-gallery 画廊与应用内嵌架构（plan-549 落地）**：对齐 widgets-gallery 交互体验，建立 examples/ui-gallery 示例画廊应用；构建期自动扫描与元数据提炼（auto-man generate_gallery_host + demos-registry 动态装配），左栏导航聚合全部 43 个 UI 示例（分类折叠与实时过滤），右侧上部提供真实可交互的 AppViewport 沙盒视口（独立 createApp 挂载隔离、异常边界 errorHandler、状态一键重置与 Desktop 100% / 1024px / Tablet 768px / Mobile 375px 多端尺寸切换），右侧下部提供基于 AutoDown/Markdown 的教程与源码逐行剖析；形成跨目录应用沙盒化内嵌的通用规范。

**029-photo-gallery（plan-537 落地）**：image widget 首个应用级双端示范
（picsum 固定 seed 网络图源，缩略 cover/查看 contain）；单组件+平行列表
数据流形态第四例。执行期实证两基建缺口（P537-D1 VM lucide 闭集 85 项（**2026-09-13 已根治：改为 lucide 官方数据生成的全量表 1401 项，见 `ui/iced/lucide_generated.rs` + `scripts/gen-lucide-table.mjs`**）——
icon 名单受限；P537-D2 语义 grid 的 cols/class 状态绑定不解析——密度
三臂静态 grid 绕开），详见 KNOWN-DEBT-AND-RISKS.md P537 节。

**use 导入 helper fn 发射（plan-522 落地）**：`use` 导入的模块级 fn 按需转译
进消费方 SFC（池收集全模块 fn + 入口门控与 VM import_aliases 同口径 + 闭包
不动点拉取 + ext_imports/命名冲突/转译边界三态闸 R013 回退）——437 §0.6.E-3
缺口关闭，computed/handler/lifecycle 体裸名调用双端同源；配套：VM 装载对包
组件文件 use 依赖的收集+别名、auto-man components/ 通道挂池、
`transpile_body_as_return` 尾表达式 return 化。语料：016 computed 化迁移
（store ×4 重算链删除）、024 donut dc/ds 回正。机制见
[design/vue-use-fn-emission.md](design/vue-use-fn-emission.md)。

**组件线（plan-437 落地，ADR-18/19）**：chart 族（line/bar/area/donut）schema 契约落库
+ vue 发射 spec 驱动化（特判臂退役）；四类图官方 Auto 组件（AutoLineChart 等四件，
widget-parens props + Init 几何 + 段记录打包，载体 widgets-gallery components/）；
VM 轨子组件 Init 渲染期补发（props 播种→Init→build，vue onMounted 对齐）——
派生计算型组件双轨可用的地基。契约细节见 [design/chart-components.md](design/chart-components.md)。
**tag 双态归属（PLAN-643）**：chart 四裸名 tag（area/bar/line/donut-chart）schema 分类
`package_origin`——声明面只登记名与契约（palette/lsp 词汇面），实现面归 official 组件包，
不参与 builtin 压制（`is_builtin_fold`/`resolve` 排除，Plan 408/435 通用 shadow 规则不动）；
合并臂 demo 适配器装载链补 package 分支（`load_ext_imports_for_vm` sweep，P642-D1 核销）；
bp `palette_drift` 合法集 = WidgetRegistry ∪ schema package-origin tags。
**交互态（plan-498 落地）**：四图族 emphasis 二态（line/area 图例悬停高亮+转折点浮现/
bar 分组描边/donut 扇区中角外移）+ legend onclick 点击显隐（mouse-area on_click 引擎臂，
iced on_press/vue @click）；悬停态字段图族专属（hovLn/hovAr/hovBr/hovDn 无悬停哨兵 9——
负数字面量 view 比较缺陷 P498-1 与 VM 单态串扰 P498-2 均已挂账）。
**交互 v2——指针移动流（plan-499 落地）**：mouse-area `onmousemove`+`coords:"WxH"` 通用
指针原语（事件携带 viewBox 逻辑坐标,引擎层完成屏幕→逻辑换算;VM 臂 PointerArea 自定义
widget 33ms 时间闸+0.5px 量化限频,vue 臂 DOM 原生不限频）;line axisPointer 十字线索引
吸附+tooltip 跟随,donut 极坐标扇区直接命中（svgdoc 静态限制经代码命中解除）,hover
动画双轨（timer 数值插值/CSS transition 类）;统一原语设计与 P-list 图元协议草案见
[design/autoui/canvas-pointer-events.md](../../../design/autoui/canvas-pointer-events.md)。
**diagram 家族开篇（plan-502 落地）**：flow-diagram v1（widgets-gallery Diagrams 分组
/flow-diagram 页）——数据轨 props `nodes`/`edges`/`direction`（td/lr 经转置）+
**Sugiyama-lite 分层布局纯 Auto**（DFS 回边剥离开环→最长路径分层→barycenter 双向
2 轮降交叉→层内等距/父居中）+ v1 SVG 渲染（484 charts 同通路）+ **svg `text` 直通
标签**（M1 对照定案胜出:vue 轨 in_svg_subtree 上下文分流/vm 轨 svgdoc text 序列化臂,
resvg 原生栅格化——svg 无 text 约束自此解除）+ hover emphasis/锚定 tooltip（498
三段式,哨兵 999）+ 边路由 bbox 交点直线 + head/tail 字形（arrow/diamond/circle）+
line dash/thick。契约见 [design/diagram-components.md](design/diagram-components.md)；
group 平铺/focus 模型归 Phase 2a，DSL 静态糖归 Phase 3。
**tree 组件族（plan-614 落地）**：TreeView（通用受控树，Ant Tree/MUI
RichTreeView 数据轨：nodes 嵌套 record + expanded/selected 受控 +
on_toggle/on_select 回调契约，PLAN-037 T3 通道 payload=首实参）+ FileTree
（文件系统树自包含预设：default_expanded 播种、folder/folder-open/扩展名
图标、badge）+ TreeIcon 有界调色板（vue 轨 icon 字面量名约束的分支式
真 lucide 发射）。渲染 = flatten-to-rows（显式栈迭代 DFS 拍平成可见行序列，
纯 Auto 经 Plan 522 use-fn 双端同源）+ pl-4 阶梯缩进（depth>8 钳制，字面量
随 SFC 转译进 JIT 扫描面）。载体 widgets-gallery components/
{tree_util,treeview,filetree,tree_icon}.at + treeview/filetree 两页
（Display 分组，63 Widgets）。VM 轨组件侧纪律成文：纯 fn 列表遍历 while+索引
（for-in 参数列表零次迭代）、Obj 字面量形状锁定全键书写、同名子组件全画廊
单实例（P320）、view f-string 禁方法调用；P614-C1（MCP press 子组件行崩溃）
挂账待引擎立项。契约见 [design/tree-components.md](design/tree-components.md)。

**导航组件线（plan-482 落地，✅ plan-562 退役）**：nav/nav-group/nav-item/nav-link
族已全部迁移至 sidebar_* 族并退役——仓内 015-notes/018-book-reader/019-video-app +
widgets-gallery 外壳与 navitem/navlink 两页 + 外仓 auto-musk(052)/auto-os-config(012)
零残留；schema/aura.at 四元素加 `superseded_by` 标注（schema_loader/ElementMeta
新增该字段解析，docs_gen 生成物 core.md/kitchen-sink.at 过滤带标注元素，本机制
首个应用即本族）；gallery 侧栏滚动随 562 增补改 AutoUI `scroll` 组件（ScrollArea/
Scrollable 双端）。nav 族实现（nav_contract.rs/脚手架 Nav*.vue/渲染臂）保留一个
观察期，移除小计划要点（含 P548-D3 清理、Plain 臂 type="button" 补全、
sidebar_input VM 缺口裁定）登记 KNOWN-DEBT。原 482 契约细节见归档计划。

**sidebar 组件族（plan-548 落地，Vue 端先行）**：shadcn-vue Sidebar 1:1 复刻的
sidebar_* 23 元素族——schema/aura.at 手写扩全 props（collapsible/variant/side/
collapsed_size 等，`SCHEMA_DRIFT_UPDATE_BASELINE=1` 更基线）+ Vue 端逐元素接线
（gen/vue.rs shadcn 发射臂，脚手架组件模板对齐 shadcn 原版）+ `to:`/`active` D2
扩展（免 RouterLink 手写）+ sidebar 裸用自动包 Provider + widgets-gallery sidebar
页重写；设计/退役路线见 [design/autoui/sidebar-family-and-nav-retirement.md](../../design/autoui/sidebar-family-and-nav-retirement.md)。
VM 端契约子集已由 plan-561 落地（见下条）；nav 族迁移与退役已由 plan-562
完成（见上条）；债务 P548-D1..D3（schema.rs 双侧分化/tooltip 未实现/vue.rs 旧臂
死代码）台账 KNOWN-DEBT。

**sidebar VM 契约子集（plan-561 落地，结构等价口径）**：`ui_gen/sidebar_contract.rs`
契约模块（28 常量 VM 可解析子集 + VM_ADAPTED 适配清单 + 逐 token 锚 shadcn
scaffold 资产的防漂移测试）+ aura_view_builder 全族构建臂（容器/分区/分组折叠
复用 nav_group_states 通道/menu_button 三态 + `to:` 路由自动探测，双拼写 tag
分发）+ color.rs sidebar 语义色板 8 映射 + render_support 37 臂与 aura.at
aliases/backends.iced 四表同步（baseline +52/−13，nav-item 先例）。子集外
（rail/trigger/input/skeleton、collapsible=icon 轨道、side 放置）按设计 §3.3
不做；widgets-gallery sidebar 页 VM 实跑与 Vue 端结构等价对拍证据
`docs/reports/p561-sidebar-contract-evidence/`。

**015-notes 示例现状（plan-616 清爽化重做，GOAL-007/010）**：示例从「卡片化 + 模态编辑」
改为**扁平双栏 + 始终可编辑**：顶栏（`border-b`）+ 列表栏（sidebar 族，`border-r`）+ 编辑栏三区用
发丝线分隔，无 `rounded-xl shadow-sm` 卡片外壳；图标全走 lucide `icon`（置顶用**文本标签**
`Pin note`/`Unpin`，因 VM 字形闭集无 pin）；筛选为 All/Pinned/文件夹/标签 胶囊（VM 不支持
`flex-wrap`，作用域与标签分两行）；**过滤下沉 store 产索引表**（`visible_pinned`/`visible_notes`，
视图只做下标解引用）——视图条件里的方法调用在 VM 端恒假，不能在视图里写 `contains` 过滤；
草稿常驻 store（切换笔记/筛选/新建前自动落盘，编辑中被切走不丢内容），保存为显式 `Save`
（仅 dirty 时出现），删除两步确认，置顶经 `toggle_pin` 落库；搜索走后端 `search_notes`
（大小写不敏感，旧契约的 known-gap 已消除）；标签/文件夹词表由笔记数据派生
（`all_tags`/`all_folders` 为**模型字段**，旧 `computed all_tags => []` 恒空写法退役）；
**外观面板由示例自带**（主题/暗色/5 色板），退出跨仓 `deps/settings` 依赖——原 `deps/settings`
指向已删除的 `examples/ui/common/settings`（靠 auto-os 回退解析的悬空链接）。
契约与 19 条 MCP 场景见 `examples/ui/015-notes/tests/{acceptance.atd,015-notes.autotest}`；
双端 evidence 见 `docs/plans/attachments/616/`。

**slot 替换（plan-476 落地）**：VM 轨 widget 插座/填充与 vue 轨语义对齐——调用位
`slot(name:X){..}`/裸子节点渲染到子 widget outlet，父作用域求值+父事件路由+逐帧重求值；
机制为构建期 `SlotFills` 父 builder 捕获 + 五容器×双胎兄弟拼接，
详见 [design/slot-substitution.md](design/slot-substitution.md)。

**桌面视觉体系（plan-503 落地）**：stella-os 风格移植——accent 玫瑰粉（coral 校准
#c4706a）、dock 图标格/激活竖条/运行圆点、弹层 glass 三件套（bg-card/80+细边+柔影，
无 blur 降格 parity 条款）、壁纸 scrim、窗口 chrome（36px 标题栏/16px 圆角/三色圆点）、
launcher 品牌色图标底块；引擎补齐 style 串循环成员插值 `${member.field}` 双端。
详见 [design/desktop-shell.md](design/desktop-shell.md)。

**桌面视觉二期（plan-518 落地）**：stella 对齐收口——双主题 token 全组（light 暖纸/
dark 精修蓝黑,可切换：set_theme 热切换动词+shell.appearance.theme 持久化+boot 读回）、
内嵌宣纸壁纸（builtin:ricepaper/inkwash,深浅主题共用浅色=壁纸主题解耦）、dock 去常驻
accent 底+贴底宽条运行指示、per-app 徽标色图标格（badge_color_for 8 色板）、窗口标题
居中+阴影双主题+透明度三档（shell.desktop.transparency）、settings Appearance 分区、
backdrop 毛玻璃词汇声明冻结（三臂：vue 直通/渲染 no-op/queue 放行;真模糊挂 RenderQueue
planned-debt）。T3 双主题对表逐通道像素断言过权威图（reports/518-t3-visual-parity.md）。

**桌面线（452→459→462→463→465 落地，464 未开工，386 暂缓）**：
452 翻转"Windows 非 compositor"裁定并验证 IME/焦点分区可行 → 459 iced daemon 多 OS 窗口 +
会话化（453 的 DesktopSession/AppSession 拆分）→ **462 路线 A 地基**：VirtualWindow widget
（Stack/clip/mouse_area 组合）+ WmState/Wid + DM::Wm 消息 + 桌面级键盘路由 → 463 桌面 shell：
全屏 borderless 宿主 + 任务栏 overlay + free/grid/master-stack 排布纯函数 + pac.at 应用注册表 +
DesktopBus v0。**465 vue 端虚拟桌面已落地**（M4）：`auto run --desktop` 宿主 scaffold
（auto-man `generate_desktop_host`：scan_apps 过滤 + `src/apps-registry.ts` 静态 import 注册表 +
子组件/stores 合并）+ WM 运行时（auto-man `assets/wm/`：store.ts WmStore、layout.ts 布局 TS
直译〔I6 与 layout.rs 共享期望值表 layout_cases.json〕、keyboard.ts R12 热键捕获段、
VirtualWindow/Taskbar.vue DOM 叶——schema/aura.at `vue:` 登记源两端同源）；E1 `(AppId,event)`
注入形状与 E2 AppWindow 叶枚举成文（`docs/plans/reports/465-t4-wm-dom-leaf.md`）；
tauri 全屏壳复用同一宿主页。**464 launcher 已落地**（SummonLauncher 懒挂载 + 真注册表平行串列注入 + windowless 特权 App 拆借垫片）。**472 AutoShell 地基已落地（shell-track M1）**：DesktopBus v1 对账定案（候选 B 传输 + `desktop.*` 动词词表 8 动词，Design 25 §3 注记回写）+ 投影协议 v1 合同 （`schema/projection-protocol-v1.md`：`__wm_*` 六字段全集/`__wm_workspaces`/指纹门控，双端对拍基线）+ workspace 分区驱动（WmState 加法增域、默认 2 分区、过滤六点）+ shell.at 升格 `widget Desktop` dock（图标化/pinned activate/切换条/`shell.dock.*` 数据级配置，pinned 宿主解析 {id,icon} 注入）。M2（switcher/pager）消费面就绪。**478 shell-track M2 已落地**：投影协议升版 v1.1（`__wm_mru` 当前分区 MRU 序投影 + `__wm_workspaces.label` 1 基标签 + 指纹扩段 + 动词词表增 `workspace_add`/`workspace_close`/`send_to`，vue 端对拍基线）+ switcher overlay（`assets/switcher.at` 进程内嵌第二枚 overlay 槽：Ctrl+Tab 召唤/推进、MRU 快照平行串列注入 + RebuildMru、Tab/←→/Enter/Esc 键盘流、点击聚焦、键盘独占 + Esc 仲裁 + 仅 visible 推层）+ dock 切换条升格 pager（1 基 label、当前分区高亮、每分区 × 删除——宿主 toast 门（非空不删/末分区保底）、尾部 + 增分区即入）+ send_to 跨区发送（Ctrl+Alt+Shift+←/→，WmCommand::SendFocusedTo）+ 驱动配套（remove_workspace 重排相邻前驱/下标压实/clamp/焦点让渡、move_win_to_workspace、mru_in_workspace）。热键表：Ctrl+Tab 改道 switcher（**490 起 Alt+Tab 退役**——Win11 系统保留死键位；分区切换迁 Ctrl+Alt+[ / ]，launcher Ctrl+Space+Ctrl+Alt+Space 双收，见下 490 条目）。**479 shell-track M3 已落地**：463 瞬时 toast 升格「浮现+历史聚合」双面（`push_notification` 单入口——入史+未读+落盘+浮现+面板活更新，既有 8 处 toast 调用点改道）+ 通知中心（dock 铃铛+未读 badge——`__wm_notes_unread` 条件消费空串/零双守卫 + 第三枚 overlay 槽 `assets/notification_center.at` 右下锚定面板：懒挂载/快照注入 note_ids·kinds·msgs·ats+RebuildNotes、开面板未读清零、逐条 ×/全部清除/Esc 关、键盘独占+Esc 仲裁+仅 visible 推层）+ `notify`/`notes_toggle`/`notes_clear`/`notes_dismiss` 动词（词表 v1.2）+ storage 定长槽 `shell.notes.0..9` 持久化（persist_notes 全量重写/restore_notifications boot 恢复，NOTES_CAP=50 内存 FIFO MRU）+ 投影协议升版 v1.2（`__wm_notes` 全量 {id,kind,msg,at} + `__wm_notes_unread` 未读串 + 指纹尾接 `|notes:{len}:{front_id}:{unread};` 段，vue 端对拍基线）。**487 shell-track M4 已落地（系统设置面板 S7）**：设置面板 overlay（`assets/settings.at` 第四枚 overlay 槽：左列 Dock/通知/关于三分区导航 + 右列内容卡；DesktopState.settings_app + HostCtx.settings_fields + split_mut windowless 第五路/split_ref_settings/settings_visible + toggle_settings 懒挂载/二态翻转/配置快照注入（cfg_dock_position/cfg_dock_enabled/cfg_notes_enabled + pinned_ids 平行列表〔B12 规避〕+ about_host/about_version 常量）+ call_handler RebuildPinned + 仅 visible 推层装配 + Esc 仲裁链第五路/键盘独占/订阅第五块）+ dock 几何驱动动词与执行臂（`open_settings`/`set_dock_position`/`set_dock_enabled` 词表 v1.4——execute_set_dock_position/enabled → apply_dock_edges_now 三联动：storage 键写回 → dock_edges 键重推导〔boot 同函数，I9〕→ apply_layout relayout + 槽位排水 + shell `__dock_*` 投影热同步；enabled=false 零预留位置键保留，重开按原位恢复）+ dock 配置链升格读写闭环（`shell.dock.*` 由 472 boot 单向读 → 驱动写回 + pinned UI 写手——面板行内增删 storage.set 直写逗号拼接 = load_dock_pinned 格式，boot 生效）+ 通知持久化开关（`shell.notes.enabled` "false"=关——479 消费链 push_notification 单点门控，notify 全链路短路；缺席/其余=开向后兼容）+ 投影协议升版 **v1.4**（486 先合占 v1.3、487 按并行协调叠 v1.4——纯增量动词/storage 键，零新投影字段零指纹变化）+ shell.at 双任务栏分支齿轮入口（OpenSettingsPanel → open_settings，铃铛邻位）。os-config 跨仓深桥/主题分区待续（壁纸已由 M5 落地，见下）。**496 shell-track M5 已落地（桌面本体 S9）**：第五面 `assets/desktop.at`（常驻不召唤——boot 装载挂 463 预留桌面层 z 槽：view 装配 Stack 先于 z_order 虚拟窗推层=壁纸之上/App 窗口之下；DesktopState.desktop_app/desktop_wallpaper + HostCtx.desktop_fields + windowless 第六路拆借/split_ref_desktop）+ 壁纸双径（#hex 由 desktop.at 根 bg 插值实铺〔`__desktop_bg` 注入〕/图片路径由宿主壁纸图层铺底——DSL 无重叠布局 z 序宿主兑现；boot 解析回退 #090e1a 默认色）+ 图标网格（pinned ∪ 自定义合并去重 hidden 排除，icon/label 注册表解析；storage 三键 shell.desktop.wallpaper/icons/hidden——487 非几何无动词判定同款，boot 生效）+ ondblclick VM 全链（View::MouseArea.on_double_click 双映射→iced mouse_area；convert_view_messages 补 MouseArea 显式臂——此前 VM 动态路径落 Empty 兜底，484 图表族经 Rust codegen 未暴露）+ 右键三项本地面板（打开=activate 472 两臂/移除=hidden 直写/更换壁纸=open_settings）+ 空白点击 463 语义+ settings 四分区（+外观：壁纸输入 storage 直写 + cfg_wallpaper 召唤快照）+ 投影协议 v1.4 内字段扩展（§2.1 `__desktop_*` 三字段族，boot 一次注入无指纹门控，零新动词）+ a2vue 真资产同源金样（插值 class 缺口修复：`${.field}` 静态段+`:class` 拼接表达式五点落码）。**497 shell-track S3 已落地（Status 栏——桌面特性线收官）**：每窗口真缩略三件套——①快照核心 `ui/iced/snapshot.rs`（**T1 定案裁剪式整窗快照**：headless no-op/iced 无公开子树离屏 API 双证伪 → `iced::window::screenshot` 整窗 RGBA〔自带 scale_factor，官方支持 widget-bounds 裁剪语义〕按 `VWinState.rect×scale_factor` 裁剪 + box 降采样长边≤256；进程级 TTL 2s 缓存〔惰性过期〕+ 抓取请求队列〔500ms 冷却防风暴〕+ 事件失效三点接线〔CloseWindow/SetLayout/apply_dock_edges_now〕）②`window_thumbnail` widget 七表登记（aura.at〔vue: @/wm/WindowThumbnail I4 同源〕+registry+schema.rs+view_builder 双臂+View 变体+渲染臂+render_support Full；渲染臂命中 `Handle::from_rgba` 直绘/miss→lucide fallback+request_capture——native "N<slot>" parse 失败天然回退）③消费者三面（switcher 行缩略〔`mru_thumbs` 平行就绪标记合同面〕+ dock 条目 hover popover〔422 先例 mouse-area+open 表达式〕+ pager 分区hover 网格〔该区窗口缩略+标题〕）+ 宿主抓取编排闭环（ServiceTick 排空队列→一次整窗截图服务全部请求〔`SnapshotShot` 事件回调按 pending wid 集裁剪入缓存+switcher/shell dirty 驱动 miss→真缩略一帧升级〕）；dock 时钟（`__wm_clock` HH:MM 本地——ServiceTick 分钟变化才写，**唯一非门控注入字段**不进指纹〔投影协议 v1.4 内字段扩展注记〕）+ 托盘组右置（挂载点容器 v1 占位+铃铛+齿轮+时钟两态）；**缺陷修复**：untracked convert_element 补 popover 臂（tracked 兜底委托路径下 popover 落容器直通锚/overlay 语义全失）+ invalidate_all 清冷却表 + SnapshotShot 补 shell dirty；T5 实机六项 PASS（switcher 真像素缩略/dock hover/pager 网格/时钟走字/顶底两态/冷缓存升级链）；tf 3316+desktop_mcp 3+t2_snapshot 4+a2vue 14 绿；债务 P497-1（pager ≤4 截断——.at 无过滤后截断原语）/P497-2（a2vue props 不透传，465 先例一致）。**501 shell-track S7 已落地（系统设置接通统一 settings center——487 预留的 os-config 跨仓深桥兑现）**：daemon 生命周期管理器 `ui/osconfig_daemon.rs`（DaemonStatus 三态 Running/Spawning/Offline + DaemonIo 注入式检活〔std TCP 裸 HTTP `/api/health`——reqwest blocking 在 tokio 上下文 panic 故弃〕+ detached spawn〔Win DETACHED_PROCESS|CREATE_NEW_PROCESS_GROUP + stdio null，桌面退出不杀——共享服务语义，待澄清② v1〕+ 就绪轮询 ≤5s + badge_projection 三态徽标投影；端口约定 17701，spawn 期 `AUTOOS_BACK_PORT` 覆盖 daemon 缺省 17901；发现序 storage `shell.osconfig.daemon` > 相邻仓 `auto-os-config-back/target/release`〔二进制实名 `auto-os-config-back-server`，计划原文路径现场核验修正〕> PATH 留扩展位〔P501-1——安装态立项时接宿主 which 语义〕）+ app 注册表多扫描根（`aggregate_scan`：主根 examples 优先按 id 去重；extra 根 = storage `shell.apps.extra_dirs`〔分号分隔 `id=path`/`path`，`parse_extra_dirs`〕+ 相邻仓探测缺省 `../auto-os-config/auto` → id `os-config`〔`shell.apps.scan_siblings=false` 可关；boot 期 `host_extra_roots` 包装〕）+ launch 执行臂依赖面（pac 可选字段 `daemon: autoos`〔跨仓 os-config 0e81196〕→ `ensure_ready` 检活/懒起 → `AUTOOS_DAEMON` env 进程注入〔VM Env.get 同源〕；**Offline 不阻断 launch**〔App 自带 daemon_view 连接测试 UX〕，原因记 `DesktopState.osconfig_status` 供徽标；pac `back: { project }` 外部 back cdylib 桩桥装载〔Plan 061 链桌面补齐——`set_external_back_root` + `load_back_cdylib` auto-man rust_ui 同型，句柄驻 `DesktopState.back_keepalive` 丢弃即卸载〕）+ settings.at 五分区（+系统：「系统设置（全部模块）」入口卡 `OpenSystemSettings` → `launch	os-config`；offline 置灰「重试并打开」双态——launch 每次重新探活零额外动词；召唤注入 `osconfig_state`/`osconfig_hint`）；T3 集成档 `tests/osconfig_integration.rs`（材料门控跳过 + 90s 看门狗 + 六段面包屑：daemon 起〔USERPROFILE/HOME 重定向配置根零污染——待澄清③跨仓 config root env 因此非必需〕→ 就绪 ping → 真相邻仓条目 launch → App Init 真数据 sys_host → GET /api/modules ≥7 → PUT ai-daemon.at 落盘断言）；合并后 master 全量 3323/3323 + scoped 189/189 + T3 1/1 绿；债务 P501-1（PATH 级）/P501-2（人手点击链残差）；P501-1..6 台账。**473 native dock 假洞 Phase 1 已落地**：`ui/native_dock/`（NativeSlot 模型+状态机+策略纯逻辑 + Win32 适配层 #[cfg(windows)]/非 Windows no-op——EnumWindows+PID 发现/DWM ext-frame 几何写读回/GWL_STYLE 剥离还原/DWMWCP_DONOTROUND 直角/sink_desktop_below z 序〔insertAfter 语义实测勘误：对 desktop 调 SetWindowPos 沉到 slot 之下〕/WinEventHook 五事件 OUTOFCONTEXT 钩子线程〔RwLock 数据槽×占用 AtomicBool 分立防回调死锁〕）；DesktopBus 动词增 `dock_native`（pid=/hwnd=）/`undock_native`；WmState.native_slots 注册表 + 槽位伪 Wid 进 apply_layout 同轮排布（min-size 扩张 C3/free 恒等）+ sync_native_geometry DPI 排水（CoordMapper 桌面原点×GetDpiForWindow+标题条客户区内缩）+ 槽位框 chrome（标题条 min/close）+ C2 独占全屏拒绝 + C4 拖走/B7 回收事件臂 + B8 退出批量恢复；feature 阶梯 native-dock（windows optional+target 双门控，默认档零开销）/test-native-dock + tools/native-fixture 夹具（JSON-lines start/bounds/close）+ 真第三方进程 E2E 六测试；**486 native dock Phase 1.5 触发面已落地**：DragWatch 拖入手势会话（`ui/native_dock/mod.rs` 纯逻辑状态机 Idle→Watching→Over——注入式落点计算〔含 free-cell 命中/最近〕+30Hz 节流+rect 变化即时重发+T1 七测；win32 钩子增 MOVESIZESTART 六事件+GetCursorPos 光标采样）+ session 接线（`DesktopEvent::NativeDragOver` 物理域消息面〔E2E/headless 注入用〕+`DesktopSession.native_drag_watch/native_drag_over` 字段+renderer `drive_drag_watch`〔START 起会话/被拖窗 LOCATIONCHANGE 采样/END 终态→dock 执行臂或清 overlay〕+`native_candidate_logical` 级联占位抽取〔高亮即落点不变量〕）+ 落点高亮 overlay（`native_drag_over_element` 主色 18% 半透明+2px 描边，view 层栈槽位 chrome 之上）+ 投影协议升版 **v1.3**（`__wm_wins` 纖入 native 槽位条目 {wid:"N<slot>",title,native,icon,focused}——仅 Docked 态/App 条目 native 恒空串统一/指纹窗段扩 "N{slot}:0,"，vue 端对拍基线）+ 任务栏动词 `focus_native`/`close_native`（三处落点+N 前缀 arg 双形态容收；执行臂 SW_RESTORE+SetForegroundWindow best-effort / WM_CLOSE→DESTROY 自然回收 B7）+ shell.at dock 区 native 分支（title 文本按钮 max-w-32 truncate+×）+ T4 E2E 拖入/拖出（`drag_sim` 合成拖拽——SendInput caption 真拖主路径〔TOPMOST 置顶+AttachThreadInput 前台化+激活结算+70% 宽大窗抓点四要素，4K@200% 小窗 caption 按钮占宽过半实测教训〕+SC_MOVE|HTCAPTION 注入退路〔同入真实 move-size 循环〕）+ T5 实机冒烟（t5_smoke #[ignore] 手动驱动：B1/B5/B8 实机留痕、D1 ◐ Chrome 自移触发 C4、G2 附带实证；B6/C1/B9 仍待用户）+ P473 债务行清偿回写；债务 P486-1 事件泵吞吐（16ms 单事件/拍，系统噪声下 dock 落位秒级延迟）；**485 原生互操作 Phase 2 剪贴板已落地（三族全通）**：`ui/clipboard_native.rs`（纯 codec 层 DROPFILES/DIBV5↔RGBA 全平台可测 + Win32 双门控层——files_get/files_set（CF_HDROP，DragQueryFileW/GMEM_MOVEABLE DROPFILES）/image_get（CF_DIBV5→CF_DIB→registered PNG 三退路→temp PNG，64MP 防爆）/image_set（DIBV5+PNG 双挂）；feature native-clipboard=[dep:windows,dep:image]（ui-iced 隐含，windows dep features 与 native-dock 共条目扩列）；四 VM natives auto.clipboard.files_get/files_set/image_get/image_set（catalog 2934-2937，降级臂空表/false/null——.at 零平台分支）+ codegen bare-name intrinsics；GlobalClipboardTestLock 跨进程命名互斥（nextest 多进程剪贴板测试互清根治）；示例 043-clipboard-bridge（三卡实机演示，Explorer/截图工具/画图往返留痕）；OLE 拖放 Phase 3/真洞 Phase 4 待续（真人清单顺延 KD-P473-2；P481-6 实机 Ctrl+C 末步复验受阻于合成输入不达 winit raw-input 流，债务开放）。**494 原生真洞 Phase 4 已落地（Region 机制替换形态）**：双 spike 证伪原设计——透明 swapchain 本机不可行（wgpu HWND surface 仅 [Opaque] alpha；DxgiFromVisual/DirectComposition 能力解锁但内容不上屏〔疑 ToDesk 远程显示环境〕；色键分层破坏 flip-model 呈现）+ HTTRANSPARENT 跨进程证伪（MSDN 同线程文义——WindowFromPoint 不跳层、真实点击被丢弃）→ 机制替换为 **SetWindowRgn 洞排除**：`raise_desktop_above`（SetWindowPos(slot,desktop) 单步 z 翻转，473 sink 参数对调）+ `apply_hole_regions`（CreateRectRgn+RGN_DIFF 逐洞扣除+SetWindowRgn(berase=false)，空表复位）——洞区窗口不存在=视觉透出 z 下层+点击直达（OS 区域语义，无同线程限制）；模式位 `shell.native.hole` storage（默认 off，DesktopOptions 程序位取或）+ sync_native_geometry hole 分支（z 翻转+refresh_hole_regions 洞集重建=全部 Docked 槽位 slot_rect）+ 失败自动回退假洞（hole_mode 翻 off+473 z 全量重申+日志，`refresh_hole_regions_at` 可测核心+stale hwnd 实测）+ 退出清 Region；T1 纯逻辑（window_local_holes 裁剪换算+z 序插入模型）+ 真实测试（z 不变量〔首可见邻居断言——IME 伴随窗楔位实测〕/Region 穿透/复位）+ T3 E2E 铁证（洞心 SendInput 跨进程精确穿透 ±6/洞外零泄漏；夹具增 click 坐标日志+win32::test_support scratch 设施）；G4 覆盖层洞边裁剪（Region 代价）+T5 实机清单+透明路径复验=已批准债务 P494-1/2/3（物理机复验，AUTO_DESKTOP_HOLE=1 钩子）。**386 路线 B 桌面协议 Stage 1+2 已落地（v1.1）**：`ui/desktop_protocol/`
（五通道消息 + 二进制编解码 + 双端状态机 + 命名管道/共享内存传输 + broker
入口裁决 + L2 detach-attach，re-exec 两进程集成验证，状态保持经 revision
连续性证明）；spawn-client 双态启动 + broker 孵化中转就绪，live-iced 渲染
消费面换接与 Stage 3 多 App 内存实测归 shell-track/后续。设计源：
[Design 23/24/25](../../../design/autoui/README.md)、[桌面协议 v1.1](../../../design/autoui/desktop-protocol-v1.md)。。**480 路线 B Stage 3 已落地（v1.2，收官）**：真桌面壳孵化通道（`ui/desktop_protocol/client_runtime.rs` AppProjector 投影器 v1——AuraNode→DrawList text/button+线性堆叠+button 命中区+prop/FStr 插值代入 VM 状态；ClientPump 协议泵 step/run 双形态；`auto run --autodesk-client=<pipe>`/`--autodesk-incubate`/`--app386=<name>` 双模入口，无标记行为零改动；`DesktopSession::enable_broker` serve 线程 + ServiceTick 帧泵周期落地）+ BrokerClient 驻留多 App 宿主（`broker_clients` 表，N=3/5 压测全 Active/逐 App 点击帧递增/30s 存活）+ 弹性重连（EOF→预算内重试连回，VM 状态/revision 原地）+ L1 换窗（`detach_surface_to_os_window`/`attach_surface_back` 登记翻转，App/VM 原地）+ L3 v2a 快照迁移（`ControlMsg::StateSnapshot` tag 11 注入恢复，count/revision 连续）+ 内存边际增量基线（Private 4.81MiB/App 临界达标·WS 23.17MiB/App 未达标——度量+判定形态，`docs/plans/reports/480-memory-baseline.md`）+ 修复 recv_wait 丢消息/shm 段名跨进程撞名两真缺陷（`autodesk-shm-<pid>-<surface>`）。像素级投影保真与 live-iced 渲染器换接归后续。**500 Stage 4 已落地（v1.3，RenderQueue 命令帧臂）**：帧载荷二态（queue=DrawList 命令帧宿主栅格化 / independent=shm 像素帧纹理上传，同宿主同屏并存）+ per-App 三态开关 `desktop_render:`（spawn > manifest > auto 覆盖探测降级+观测行）+ `coverage.rs` 能力表/judge——详见 [desktop-protocol-v1](../../../design/autoui/desktop-protocol-v1.md) §1.3 与 specs.json P500-x。**507 Stage 5 已落地（覆盖爬坡 + parity 债闸门）**：aura.at 388 element 三级分级定稿（Tier1 40/Tier2 29/Tier3 not-yet 73 逐项裁定/n-a 246），AppProjector 爬坡至 Tier1+2 全量（**covered 69/388=17.8%**）——display 七员（icon/badge/avatar/progress/divider/separator/spacer）、form 四员（checkbox/switch/radio/textarea——Toggle 命中区〔handler 在场=handler 拥有状态变更〕+焦点 accent 描边+禁用乘暗不登记）、grid cols 等宽网格+card 表面缺省档、typography 缺省档（pre 族底盒/blockquote 引用条/small/heading 档；bold/italic 字重边界）+语义容器 16 员块流（折叠键贯通 normalize_kind/layout_node）；**第三端 parity 债自动闸门**：`aura/element_coverage.rs` 元素级登记表（单一事实源，无 feature 门）+ schema_drift 双向同步围栏（未登记即红/陈旧即红/覆盖率输出）+ covered⊆target_set 一致性钉 + parity 金样矩阵（覆盖表驱动 5 夹具×两阶段+防漏钉夹具并集⊇target_set，`test/parity/matrix/`）；**500 逃逸修正二枚**：容器 z 序（bg 先于子级——顺序栅格化下 bg 盖子级）与 Toggle 双翻对冲（实机 e2e 实证）；**日常门禁**：`cargo t` 携 `--features ui-iced`（tf 盲区收口）；构造示例 `examples/ui/p507-tier-coverage` 入实机 queue e2e。specs.json P507-1..6；债务 P507-1..3。**508 Stage 6 已落地（默认策略裁定 + 远程 command 流，RenderQueue 线收官）**：`shell.apps.process_model` 配置位（inproc|outproc，缺省 inproc 零变化；outproc=launch_app broker 孵化真子进程——re-exec spawn→broker 受理→同步泵 attach→registry_id 回填）+ G2 对比实测**裁定维持 inproc 缺省**（总边际 0.86 vs 7.64MiB/App、启动 2–17ms vs 25–250ms、交互 0.07 vs 1.54ms；outproc 留隔离选项，翻转三闸 T-覆盖/T-稳定性/T-远程，`docs/plans/reports/508-process-model-verdict.md`）+ `WsTransport`（Transport 第五实现：tokio-tungstenite 0.30 no-default-features，WS Binary=codec 信封原样，token 升级期 query 校验 401 终态拒收）+ 远程镜像会话（`remote.rs`：`127.0.0.1:17800` 缺省不监听、`shell.remote.token` 缺省拒绝；Hello→Welcome+HitTable〔tag9 纯追加〕→帧推送，输入路由同 broker 收尾；`PROTOCOL_VERSION` 仍 1，pipe/loopback 零改动）+ `packages/drawlist-renderer/`（TS/Canvas2D：codec/messages/render/connect 四模块，Rust↔TS golden 双侧对拍防漂移，vitest 20 绿）+ `examples/remote/viewer/`（vite demo + e2e：T4 Playwright 浏览器渲染 002-counter 点击闭环 PASS、T6 真桌面 outproc↔headed Chromium 双向闭环截图×3）。specs.json P508-1..6。**516 vue 桌面远程窗已落地（508 远程渲染解锁的用户出口）**：远程 App 作为 `kind:"remote"` 虚拟窗进 465 vue 宿主 WM（store kind 判别 + `remote.ts` 会话切片〔reactive 状态/帧缓存 rAF 末帧/输入回发/断线保留〕+ `RemoteWindow.vue` wm 叶〔canvas 位图=Welcome 尺寸×DPR/四态状态面/指针 HitTable 命中回发/可打印字符回发〕）；渲染器包经 auto-man `wm_assets` 编译期 include_str! 拷贝物化 `src/wm/remote-renderer/`（包零改动）；配置双通道 `<apps_dir>/remote-apps.json` + URL 注入（?remote/app/title/rbudget），boot 建连失败降级状态面不阻断；尺寸协商 v1 以 vue 侧为源不回传 resize（协议文档注记）；wm-test vitest 测试位 14 绿（resolveId 直指渲染器包源双向钉拷贝漂移）+ Playwright 全链 e2e（帧渲染/点击闭环/拖动/任务栏聚焦/断线保留 + 无配置零变化门禁）。specs.json P516-1..6；债务候选 P516-1 onLog 行文耦合/P516-2 键盘 printable-only（=计划待澄清⑤）。

**vue 轨（codegen 正确性 + 工程化）**：444 修五类 vue-tsc 缺陷（回调通道/emits 名册派生/变体断言）；
443 defineModel 降级收窄（bound_model_channels 预扫）；451 actions/menubar/toolbar/快捷键消费补进
codegen；457 ~60 个 shadcn 组件编译期内嵌 auto-man（冷启动离线化）；437 chart 族声明驱动发射
（SVG 直通三端同源，借 442 A4）。

**VM 轨（视觉 parity + 运行时对齐）**：450/451-image/452-login 逐项拉平 shadcn 语义（圆角 SDF 掩膜、
Text 盒模型、按钮 14px/500、input 透明背景）；455 双端 parity 跟踪器把标准下沉为引擎规范（focus
ring 2px、margin 盒模型），矩阵约 9 绿 / 8+ 待审计；458 theme/accent 一等配置（CLI/pac.at/env
三通道，双端默认 dark+indigo）；446 清偿实战上报的渲染薄弱点（A1 多 store 消歧编译期报错、
J1/J2 渲染器子树，批五转正中）。

**DSL 与内建 widget**：425 component fn 糖化退役双轨（widget 单轨）；426/436 setup{} 三相位语义
（vue 落地、解释器 L1、a2r 显式报错）；448 msg 去名 + 内联 lambda 简写；413–428 code_editor 全链
（自研 cosmic-text ViEditor、折叠逐 run 管线、多 tab、vue CodeMirror 契约对齐）；449 确立 VM 组件
写法边界（三缺口登记：回调 props 退化/快照子树不可见/片段条件不求值）。

**示例轨道**：examples/ui 024-charts（437/445）、025-dashboard（438）已交付；026-database（439）、
027-file-manager（440）草案可领取；028-launcher 归 464。

**448 AutoUI 语法改进滚动收集已落地（七条目）**：三轮全量走查 examples/ui 收集并实施——A `msg {…}` 无名正字法（兼容窗口保留旧名）·B 事件内联 lambda `onclick: () => {…}`（parser 铸名 `__evt_*` 单点 desugar，三后端同源）·C 裸 `value:` 两向绑定（`__bind_<W>_<n>` 空体铸名——VM input_state_map 预写回/原生 Rust input_fields 注入/Vue v-model 折叠+噪音抑制；输入框免 msg/on 三件套）·D style 组合（数组形态 `["基座", if…]` 显式类段 join；class-safe 拼接结构判定→:class 三发射点齐备）·I grid `cols:/gap:` 动态值（VM 重建期求值+失败落默认；Vue 内联 `grid-template-columns: repeat(N,…)` 绕 JIT 扫描）·H1/H2 computed 块体（Vue 尾 return+`computed<T>` 推断；VM `__computed_<W>_<p>` 隐藏函数经 handler 合成机制执行）。H3（模块级 helper fn 进 vue SFC）划出独立 Plan 522。复审 tf 3357/3358+12/12 条目测试；债务 P448-1..3（__evt_ 跨兄弟冲撞面/plain 生成器 cols 死属性/computed store 消歧未接）。
**481 展示型文字选择/复制已落地**：text/label 增 `selectable` 属性（bool,默认 false——opt-in,缺省渲染路径逐行零变化）;VM 端自研 `ui/iced/selectable_text.rs` SelectableText widget(advanced Widget——绘制复用 iced `text` 同参同路径保证逐像素一致,命中走 iced_graphics Paragraph 公开 `buffer()`;手势集 v1=拖选/双击词选(字符类分段词界,UAX#29 默认 CJK 连字)/Ctrl+C 写剪贴板(有选区才捕获)/Esc 清除不夺全局流;选区为 widget 本地状态,零桌面集成改动) + `selection.rs` 选区纯逻辑(全平台单测);vue 端显式化 `style="user-select: text"`(plain/shadcn 双路径);a2vue 金样 011 锁 prop 往返;001-helloworld/004-profile-card 点亮。边界:font-mono 代码文本 v1 保持 Rich 高亮不可选;arboard 兜底未启用(iced 剪贴板恒可用)。

**504 示例桌面化三件套（011-calculator 样板）**：pac.at `window: "fit"` 自适应窗口——独立 VM 窗首帧按内容 shrink 测量后 resize（clamp [200, 可用区]）、桌面虚拟窗以测量值替代"可用区 60%"写死初值（`register_window` 覆盖保留 fit_pending 为关键修复）；title/settings 上移 os-config per-app 配置（`~/.config/autoos/apps/<app>/config.at`，launch 直读文件注入 theme/accent——启动期 daemon 可能未起，不经 daemon；`modules.d/<id>.at` 注册后通用编辑器零手写获得设置 UI）；应用内 `ExampleHeader` 退役，标题由 pac.at `title:` + 桌面 chrome 提供。债项 P504-1..4 见 KNOWN-DEBT。
**506 示例桌面化批一（011 样板批量展开 7 例）**：三件套向首批示例批量兑现——header 退役线（008/009/010）：`common/header.at`（ExampleHeader 组件包）整体删除，app.at 无 header/settings，`dark_mode`/`accent_color` 声明保留作 os-config/env 播种挂钩（宿主 `seed_app_config` 与 renderer env 播种均要求已声明变量，删声明即静默失效）；theme/accent 注册 os-config per-app 配置（`~/.config/autoos/modules.d/auto-<app>.at` + `apps/<app>/config.at`，shape 循 calculator 先例 theme/accent 两键），优先级链 CLI > os-config > pac.at > 内置。fit/title 线（002/003/012/038）：pac.at 补 `title:` + `window: "fit"`，根容器 `center` 居中外壳拆除改"内容即页面"（四 app 实测均有 center，012 另删 min-h-screen），VM 独立窗实测收缩 400x400/400x720/550x774/647x878（默认 1293x836），fit 断言范式 = VM 截图 PNG 像素尺寸 < 900。测试改法范式：双端脚本删 settings 交互、增"无 header 元素 + 内容标记"断言（循 504 test_011）。债项 P506-1（038 Reveal 触发 VM RC use-after-free，master 预存疑 511 回归）/P506-2（MCP rendered-vtree 快照无事件注记，038 改 label 定位法）见 KNOWN-DEBT；批二 = 剩余 20 例 title 债务（001/004-007/013-025/028/042）。
**512 示例桌面化批二（fit 动态重测机制 + 批二 20 例）**：`window: "fit"` 语义补全——由首帧一次性测量升级为**动态重测双向跟随**：app view 重建（dispatch 漏斗 view_dirty）落到 fit 窗条目打 `fit_dirty` 标 → ServiceTick 400ms 节拍发起内容测量（**standalone 订阅补齐**：原仅 desktop 门控且宿主窗句柄恒 None，现 desktop 或 standalone 有脏标即订、测量目标取待测窗自身 id）→ `decide_fit_resize` 滞回 8px 决策 → standalone `window::resize` / desktop vwin rect 更新双路径；用户手动 resize 一次性锁定（程序化回波 ±2px 豁免）。**量测法关键修正**：活树布局受当前窗口钳制（内容自然高超出窗口即被裁，增长方向量不到；504 首测成功仅因默认窗大于内容）——`fit_aware_root` 锚点外套 vertical scrollable（内部无限高约束排版），锚点量真实自然尺寸；副作用=内容超窗瞬态出滚动条而非裁剪，宽度方向 v1 仍受视口钳制（P512-1）。实证：011 Scientific +44px/回缩基线、005 校验错误行 +39px/回缩（双探针 `tests/test_512_fit_remeasure.py`，win32 物理尺寸断言，阈值 >24px=3×滞回有实测依据）。迁移：fit 线 4 例（001/004/005/016）拆 `center` 外壳 + 卡片固定宽（**实证：`w-[28rem]` rem 任意值 iced 端不支持塌缩 213px，改 Tailwind 刻度 w-96/w-112 双端兼容**）实测 213x236/445x451/573x535/541x441 截图留痕；title-only 线 16 例 pac.at 补 `title:`。债项 P512-1（宽度钳制）/P512-2（锁定后余量视觉语义）/P512-3（p508_g2_outproc_arm 并发偶红）见 KNOWN-DEBT；P504-2 已清偿，P506-1（038 Reveal UAF）核对仍在（511 归档未修）。

**552 桌面应用策展已落地（desktop 字段 opt-in + 测试探针清退）**：R10 注册表条目增 `desktop_visible`（pac `desktop:` 字段——主根 examples/ui 缺省 false=opt-in、外部自含根缺省 true=opt-out，坏值静默回退；`entry_for_dir(default_visible)`）；boot 两分——`app_resolver` 捕获全量（按名启动/自定义图标启动不受策展限制），`registry_entries`=策展过滤（launcher/图标格/dock 三消费面只见策展集，boot 日志双计数）。C 档 19 例加 `desktop: "true"`（045 已由 551 退役，20→19）；8 个测试探针（overlay-probe/p051/p493/p507/p515/p518/459-dual-app/042-two-inputs-child）迁 `examples/capability-tests/`（stage3 e2e 双根解析、ui_desktop/ui_dual_app include_str 改指、`scan_examples_ui_curation_set` 恰等断言防今后 demo 悄悄上架/掉字段）；examples/ui README 总览表增"桌面"列。债务 P552（customs 非策展 id 元数据回退 app-window/裸 id）。
**527 VM 轨 Tailwind 全量覆盖契约**：样式子系统从「Tailwind-inspired 按需子集」升级为 **v3.4 清单驱动的全量覆盖契约**——①清单锚定：Tailwind v3.4 core 展开清单 vendor 入库（`tests/fixtures/tailwind-v34-utilities.txt` 8861 类×15 families，`tools/gen_tailwind_manifest.py` 零依赖再生）+`Style::parse_reported` 报告通道（未映射类按原文名报告，**静默丢弃通道关闭**）；②对拍审计台常驻（`tests/style_parity.rs` 已挂 `cargo t`）：白名单外零 missing + 布局/视觉/文本三家族 iced applied 门 + PARSED_ONLY_ALLOWED 豁免台账 + 覆盖率表 `docs/style-coverage.md` 同源再生（基线 applied 3807/parsed-only 276/unsupported 4778/missing 0）；③三家族补全：布局 1582/视觉 1901/文本 308 applied（SizeValue::Fraction 分数 Fill-ratio 口径、四色板 lime/violet/fuchsia/stone 补全+950 真值行、渐变三 stop+位置百分比真消费、彩色阴影、object-fit→ContentFit、全字重 9 档、leading 双轨、line-clamp；顺修 from-100 三位 hex 误吞/min-h 未知命名误落 0.0 等隐性假映射）；④Variant 管道泛化：hover/focus/active/disabled 同构（按钮状态面真消费+opacity 乘法降级）、responsive 五断点解析期按 `theme::window_width` 门控（resize→view 重建→重解析既有回路）、`dark:` 按 `theme::dark_mode` 门控——未命中态登记 variant_classes 可见不静默；⑤不做/受限台账 KNOWN-DEBT P527-1..5（永久不做族/宿主上限/分数近似口径/变体分期消费/存字段类）。复审 tf 3397/3398（唯一红=在案存量）零新增红。
**593 语义 token 值单源化（Design 29 Phase 1，GOAL-007）**：样式值从四处置手抄
互锁（vue.rs base_css 模板/theme.rs match 臂/code_editor accent 副本 + accent 双份）
收敛为 **`design_tokens/registry.rs` 单一事实源**——①对账先行：七发射点实勘
（E1-E7，含重大发现「ui_gen base_css 仅测试消费、真实 Vue 路径=auto-man
generate_index_css」→ V1 扩双改造）；②零漂移方法论「先钉后改」：期望表
（resolve_semantic_rgb 全语义色×双态 RGB 字面量）+ 双金样（base_css/index_css
模板逐字提取）先行钉绿，迁移后同组断言即等价证明；③四处置迁移：V2 match 臂→
color_token 投影+stella 查表、V1×2 色变量块→registry zinc/scaffold 装配、V4-a
code_editor accent 表删（L 分量漂移 3 处但编辑器只消费 H/S，归一输出中性）、
accent 表单源+ACCENT_DEFAULT 常量；④证据门裁定：`bg-accent` 81 处在用→
accent 投影坍缩钉 Phase 2（P593-D1）；⑤债 P593-D1..D5（accent 投影/+4vs+10
提亮分叉/ui_gen 遗留/auto-os 跨仓/auto CLI 双副本）。复审 R1 抓 tf 档 cfg 门
缺失（S10 只跑日常档的漏检面）、R2 回退字面量收口；F-env kitchen_sink 跨仓
竞态（auto-os@84ff922 并行推进）环境分离非回归。Phase 2（主题声明与热切换）/
Phase 3（style recipe 语言层）见 Design 29 §7。
**601 主题声明与热切换（Design 29 Phase 2 落地，GOAL-007）**：主题从「dark/light
二值 + accent 单槽」升级为**可命名、可派生、可热切换的整套色板**——①registry
双面统一：五内置主题（zinc/scaffold/stella/tauri/cli-vue）结构化 `ThemeSpec`
（`ColorLit` HSL 原文/RGB 真值同源互转），canonical CSS 渲染（金样口径升级
value-pinned）；②声明面：pac.at `theme: { extends/mode/colors }` 块解析
（extends 链深 ≤4 防环、本声明最后胜、mode 沿链最近声明胜，未知 token 编译期
错误；对象冒号形态），合成体 `decl::compose` 产物 `ComposedTheme` 与 builtin
同类型，经 auto-man **双端消费**〔PLAN-609 双端消费对齐〕：vue 腿（index.css
双 mode 块装配 + index.html `__AUTO_COMPOSED_THEME__` 运行时种子
write-if-unset）+ VM 腿 boot（`run_vm_ui` 首帧前 `set_theme_composed` 激活
〔env 种子块之后——声明只换 ACTIVE_THEME 色板槽，不清写 DARK_MODE/
ACCENT_NAME；优先级链 CLI env > os-config/宿主 > pac.at 声明 > 内置缺省；
无声明/坏声明回退内置缺省〕）；③切换面：VM
`set_theme(name)`/`SetThemeName` 动词（ACTIVE_THEME 槽 + THEME_EPOCH 失效回路 +
desktop_config `theme_name` 持久化 + **boot 读回激活**〔open_desktop 首帧前，
复审 R1 补〕），vue `applyTheme` 运行时全变量写入（html inline light + `.dark`
元素 dark 值 + 光照模式陈值清理，accent overlay 内聚末位，storage `auto-theme`
boot 恢复），零 accent 面 app 零注入；④`dark:` 门控与 set_theme(bool) mode
链路正交零回归；⑤accent 降维覆盖层（primary 槽，dark 提亮 +10 双端归一）；
⑥债收口 P593-D1..D5 全关（D1 accent 独立投影——81 处使用面有意视觉对齐、
D3 ui_gen generate_base_css 退役、D5 CLI 双副本 registry 装配）+ **P601-T11
开放债收窄〔PLAN-609 勘验修正〕**：settings-popover 类 `use` 引用的包组件
SFC 断链已由 PLAN-609 收口（根因=dep 源死指非发射链缺口，auto-os 镜像
回退+import/落盘一致性守卫成文）；开放债仅余 SVG 图形属性 token 通道缺失
——图表主题跟随示范面受阻，charts 裸名 `<div :data>` 占位为 484 M4 有意
形态（KNOWN-DEBT P601-T11）。settings 选择器 UI 属 auto-os 资产面移交。

**615 calc 修复与增强（按钮行盒/主题传播链 OS 跟随）**：①**按钮标签行盒契约
（SD-03）**——按钮标签（纯文本/样式路径/hicon+lucide 图标行）行高钳
`Relative(1.0)`（iced 0.14 文本默认 1.3，额外 leading 全落字形上方，无高度类
按钮〔shrink 高=行盒高〕字形系统性偏下 ~0.15em）；显式 `leading-*` 类优先；
高度类按钮 Plan 414 容器居中正交；回归锚
`layout_tests::button_label_line_box_clamped_to_font_size`。②**主题传播链
（SD-02）**——OS 系统主题（Windows `AppsUseLightTheme` 注册表，
`ui/system_theme.rs` reg query 零 feature 耦合，非 Windows 回退 dark）→
`DesktopConfig.theme_source`（**system 缺省**=`load()` 每次 OS 派生
`dark_theme`〔含 mtime 外写热应用轮询同链〕；**manual**=设置面板 set_theme
用户显式切换即置+持久化终结跟随；存量配置文件缺键按 system=即时获得跟随）→
应用 `dark_mode`（boot 播种〔518 在案〕+ launch 播种缺省臂 + 独立 VM 窗
`AUTO_UI_THEME` 环境链〔CLI>os-config>pac〕未解析时 OS 回退）；语义 token 类
（bg-card/bg-muted/bg-primary…）随 set_dark_mode 自动双档，应用级零手工分支。
③bind 键盘直输：parse_bind_block 零参约束（仅收 `.Name`）——模式感知零参
Key 处理器族范式（带参 bind 扩展=KNOWN-DEBT P615-D1）；`dom.copy_text` 双端
剪贴板内建（VM native 2926 复用 418 面/vue navigator.clipboard）。

**619 015-notes 双端 parity 收敛（GOAL-007）**：同一份 `.at` 在 VM(iced) 与 Vue 两端
肉眼可见的四处差异被量化、定位到引擎侧根因并修掉，同时把「双端一致」落成可复跑门禁。
①**权威色板成文（SD-01）**：`lucide_svg()`…（见下条）之外，VM 语义 token 的缺省解析由
「隐式 stella 单源」改为 **与生成端 `index.css` 同源的那张表**（`registry::SCAFFOLD`——
auto-man 生成 Vue CSS 时逐 token 渲染的就是它），未声明 `theme{}` 的应用两端取同一表；
**桌面宿主必须显式声明 stella**（`DesktopConfig::default().theme_name = Some("stella")`），
规则成文为「宿主 = stella、pac 应用 = scaffold」。②四个根因修复：`with_class_prop` 补
`style:` 键回落（DSL 主写法此前被整串丢弃）、`IcedStyle::effective_padding/effective_margin`
统一「单侧 > 轴 > 统一 > legacy」覆盖次序（`mx-*` 整族此前无处消费）、Text 臂补 `height`
（契约 `h-8` 不落盒）、icon 的 `.at size:` 两端贯通（合并 PLAN-617 后统一为
「显式 `w-*`/`h-*`/`size-*` 类 > `size:` > 缺省 20px」双端口径：VM `with_icon_size` 折
Width/Height，Vue 生成器折内联 `style="width:Npx;height:Npx"` 并撤缺省 `w-5 h-5`）。
③**根因级文档坑**：lucide 文档构造必须**单层包装 fragment**——PLAN-617 全量表
（`lucide_generated.rs`，生成自 lucide-vue-next v0.312，1401 项）给出 24×24 内部 markup，
`lucide_svg_doc_with` 从 fragment 单层包 `<svg>`；若把完整 16×16 文档（如 `lucide_svg`
的缓存产物）再套一层 24×24 `<svg>`，嵌套 viewport 会按 16/24 二次缩放（跨端 icon ink
系统性偏小 ≈33%，即「声明 18px 实测 ≈12px 盒」——619 实证根因，617 换表时一度复发，
由 619 回归锚守住）。④门禁：`tools/parity_shot_diff.py` 预算化（语义面色 ≤2/通道、图标 ink 比
∈[0.9,1.1]、左缩进 ≤1px，违规退出码 1）+ 015-notes acceptance **T14/C-PARITY-1** +
autoui-verifier 技能「步骤 3.5 像素预算对拍」。实机（1280x800）：面色 Δ0/Δ1、图标 ink 比
1.03/1.03、左缩进 Δ0/Δ1/Δ0；19 MCP + 18 Vue 场景全绿；`cargo t`/`tv` 零新增红。

⑤**icon `state` 契约（PLAN-621，SD-01）**：`icon` 元素增 `state` prop（`"on"`/`"off"`
字面量或 bool 绑定）——on 注入 `text-primary`（**运行时 accent 预设主色**：
`resolve_semantic_rgb` 对 `Color::Primary` 专臂查 `registry::accent_hsl`，dark L+10
双端统一；token `accent` 是 hover 高亮面非强调色族，不可用）+ 描边加重基档+0.5
（`StyleClass::StrokeWidth` 承载绝对值，builder `with_state_tint` 单点计算，renderer
lucide 路径只读消费；基档与 Plan 518 G4② 同式 ≥48px→1.5 否则 2.0）；off 注入
`text-muted-foreground`（`Color::OnSurface` 双盘 dim）；未声明零注入（输出逐字节
不变，既有金样零扰动）。优先级「显式 `text-*` 类 > state > 继承」镜像尺寸口径。
亮度型 on/off（显著 vs dim）对色盲安全，色相型不可用（WCAG 仅靠色相禁令）。
适用面仅独立 `icon` 元素（button/nav-item 内嵌 icon 的 active 语义独立存在）。
门禁：`plan621` 10 测（VM 6 + Web 4）+ `test/a2vue/012_icon_state` 金样（五形态）。
⑥**图库策略（PLAN-621，SD-02）**：lucide 为 AutoUI 主力图库（fill 变体官方不做，
满/空双态由 ⑤ 的 state 契约承载）；品牌图标/真填充需求走 IconifyJSON 补位管线
（`set:name` 双段名 + `@iconify-json/*` 锁版本离线可复现），playbook 与调研数据见
[icon-state-and-library-policy](../../../design/autoui/icon-state-and-library-policy.md)，
不整体迁移（Remix 同名交集 0 + 反向缺名，波及面基线 29 名）。
⑦**progress `onseek` 契约（PLAN-620）**：`progress (value:, max:, onseek: .H($0))`
双端可拖拽 seek——handler 收**条内横向比例 0..1**（作者写 `SeekTo(.duration * $0)`
无须知道像素或 max）；按下即 seek、**按住期间的移动**才继续 seek（悬停不 scrub）；
VM 侧新 widget `ui/iced/seek_area.rs`（iced `mouse_area.on_press` 不带坐标，由
`SeekArea` 自持按下态并在事件现场换算比例，拖动用绝对坐标防拖出条外断流），
`View::ProgressBar.on_seek` 复用 `PointerMoveHandler` 装配；Web 侧生成器输出
pointer 三包装（`setPointerCapture`，拖出条外仍跟手，判据取 `e.currentTarget`）；
与 `PointerArea`（Plan 499，坐标流原语）刻意分离——比例语义不塞坐标契约。**兼容
锚**：未声明 `onseek` 的 `progress` 输出逐字节不变（测试反断言在案）。
⑧**图标数据源与生成器（PLAN-620）**：VM 端 lucide 字形由
`scripts/gen-lucide-table.mjs` 从本地已安装的 lucide-vue-next（离线、幂等、零新
依赖）生成全量表 `ui/iced/lucide_generated.rs`（当前 1401 条/24×24 markup），
`lucide_svg` 查表 + 遗留别名（`sidebar`→已更名、`file-icon`→已并入）；重跑
`node scripts/gen-lucide-table.mjs`（自动发现 `examples/**` 已装包，`--src` 可显式
并向上探测版本）。**防漂移门禁**：产物内嵌 `LUCIDE_SOURCE_VERSION` 常量 +
`source_version_matches_installed_package` 对拍测试（升包未再生即红并指路重生，
找不到安装包/版本未知时跳过）。口径细节见
[icon-data-source-and-parity](../../../design/autoui/icon-data-source-and-parity.md)。
尺寸口径见 ⑤ 与 617/619 段（显式类 > `size:` > 默认 20px），此处不重复。

## ui-gallery VM 内嵌健康度契约（PLAN-642）

VM 画廊内嵌形态（AppViewport.vm.at + demos/*.at 适配器 + registry.at）的 enduring 契约，2026-09-18 落地：

1. **registry `loadable` 语义（modify）**：registry.at 的 `loadable` = "VM 臂内嵌可交互"（`loadable || fullstack`）；web 臂 `demos-registry.ts.loadable` = Vue 动态挂载能力——两臂数据源分离为 enduring 决策（P633-D2 核销）。
2. **适配器 stylekit 配方内联（add）**：`emit_gallery_vm_demos` 发射期把跨包 `use stylekit.styles` 配方内联为适配器本地 `pub style`（画廊上下文无跨包解析通道）；教程/源码 tab 展示语料原文。
3. **组件包目录级联（add）**：475 组件包（`use { package: ... from "dir" }`）目录随适配器级联拷贝进 demos/（package.at 清单跳过；包内 fn 模块链同链收集；同名异容 kept-first 告警）。
4. **parse_package_widgets recipe 预注册义务（modify）**：475 包装载器 parse 前必须执行 `prepare_style_recipe_imports`（与编译入口同契约；tree_icon 修复实证）。
5. **distributed 列 grow 剥离（add，T-11）**：justify-between/around/evenly 列的直接子剥 Flex1/FlexAuto/Grow 且不补 Height(Full)——iced 0.14 flex 对 FillPortion 子 min=max=份额硬钉 + 列内子项按剩余量配给，grow 子与垫片竞争时内容 0×0 隐没（008 特性行实证）；CSS grow 的 min-content 钳制 iced 无对应，distributed 列 grow 让渡给垫片。
6. **frame scroll 兜底 + 内嵌语料 Fill 高度约定（add，T-12）**：overflow-y:hidden + justify-Center/End 列的内容包 Shrink 高度 Scrollable——短内容被容器 center_y 垂直居中、长内容封顶滚动（009/016）。配套语料约定：**内嵌 demo 避免 Fill 高度技巧**（items-stretch 等高拉伸、定高滚动上下文中的 flex-1——Fill 在 scroll 无界主轴下解析塌缩，008 实证）；等高需求待 PLAN-655 StretchLine 原语（P642-D12 近期处置）。
7. **已知开放项**：子件主题魔法变量统一状态覆写链（016 打开翻转宿主主题持久）在案未修（T-13 needs_replan，见债账 P642-D9/D12）；008 卡片 scroll 折叠线下滚轮可达性待人工复验。

## items-stretch 两阶段行语义（PLAN-655）

`items-stretch` 行的 enduring 渲染契约，2026-09-18 落地（P642-D12 路线 A）：

1. **等高原语（add）**：stretch 行由 `ui/iced/stretch_line.rs` 的 `StretchLine` 控件承载——两阶段布局：measure 遍按各子项**最终份额宽** + 子项主轴 compression 取内容自然高（`h_line = max(子项内容高)`）；final 遍以 `effective = 有界父上下文 min(h_line, 入射上限) / 无界 h_line` 落位。行高 = max 子项内容高，子项等高 = 行高（CSS `align-items:stretch` 同构），**任意祖先上下文成立**（scroll 内容臂无界下不再塌缩——P642-D12 008 定价卡消失根修）。
2. **交叉轴拉伸载体（add）**：Shrink 高子项以 `min_h = effective` 拉伸到行高（CSS auto 高 flex 项语义；justify-between 列因此在行高内分布内容，008 卡底 CTA 对齐实证）；Fixed 高子项不拉伸（CSS：显式高不参与 stretch），自然钳制；Fill 高子项解析到行高。
3. **主轴配给（add）**：行内 Fill/FillPortion 子项按 third-pass 数学均分剩余宽（justify 垫片即 FillPortion Space，并入同一配给）；宽度探测先于测高（文本换行行数依赖最终宽——宽度未定时测高会把最高卡测短，内容下溢钳裁）。
4. **旧 Fill 包装形态退役（retire）**：build_row 不再用「子项包 height:Fill 容器」模拟 stretch（该形态依赖有界祖先，无界下塌缩 0 高）；上节第 6 条「内嵌 demo 避免 items-stretch」约定自本节起解除，008 语料还原 items-stretch。scroll 兜底本身（overflow-hidden 列包 Shrink Scrollable）不受影响。
5. **非目标**：012-clock 横向 stretch 行（列交叉轴=宽度，现实现无害）；iced 引擎级 flex 补丁（P642-D12 远期路线 B，iced 升级时处理）。

## 关键入口

- `dialect/ui.rs:UiDialect` · `aura/extract.rs` · `aura/schema_loader.rs`（契约源自 `schema/aura.at`）
- `ui_gen/vue.rs:VueGenerator` · `ui_gen/api.rs` · `ui_gen/widget/registry.rs`（widget/chart 契约表；
  **PLAN-609 后 use 引用的包组件必须落盘 `components/*.vue` 与 import 发射一致**
  ——dep 源经 `resolve_dep_os_mirror` 按 resolve_os_top_dir 解析序回退 auto-os 镜像，
  `auto run` 增量路径 Phase 1c 同步落盘，from_workspace 一致性守卫缺者 strict 硬错/非 strict 告警）
- `ui/widget_registry.rs` · `ui/render_support.rs` · `ui/event_router.rs` · `ui/aura_view_builder.rs`
- 桌面线：`ui/session.rs`（DesktopSession/AppSession + WmState/WmCommand/DM::Wm）·
  `ui/iced/virtual_window.rs`（VirtualWindow）· `ui/iced/renderer.rs`（view_desktop_fn/run_dynamic_iced_multi）·
  `ui/desktop_protocol/`（路线 B 桌面协议 v1.1：五通道/传输/shm/broker/状态机）
- 样式与主题：`ui/style/`（class/color/theme/iced/headless/gpui 适配 + **Plan 527 v3.4 清单驱动全量
  覆盖契约**——parse_reported 报告通道/对拍审计台 style_parity/Variant 管道〔hover/focus/active/
  disabled/responsive 五断点窗口宽门控/dark 主题态门控〕，覆盖矩阵 docs/style-coverage.md；
  **Plan 593 后语义 token 值单源于 `design_tokens/registry.rs`**——theme/ 目录模块化，
  resolve_semantic_rgb 投影查表零字面色值）·
  `design_tokens/`（**Plan 593 新增无 feature 门基础层**：registry 单一事实源——31 键封闭
  词表〔19 shadcn+8 sidebar+4 扩展〕；**Plan 601 后五内置主题〔zinc/scaffold/stella/
  tauri/cli-vue〕结构化 ThemeSpec 双面**（ColorLit HSL 原文/RGB 真值同源互转，canonical
  CSS 渲染）+ theme{} 声明合成〔decl.rs：extends 链/mode 继承/值规范化，ComposedTheme
  与 builtin 同类型〕+ accent 表单源；ui_gen〔无门〕/ui::style/code_editor 三方直达消费，
  theme re-export 保路径稳定——放 ui 外因 ui_gen 不能引 feature="ui" 实体）·
  `ui/action_config.rs`（actions 配置层，热重载/OS keymap/表达式条件）
- 内建编辑器：`ui/code_editor/`（**Plan 601 后编辑器主题从活动主题派生（V4 完整）**——
  bg/fg/caret/syntax 取 registry Background/Foreground 真值入编辑器色域，syntax 主题键
  `autoui-{theme}-{mode}-{accent}`〔内置预烘焙+boot 期合成体烘焙〕，切主题 THEME_EPOCH
  失效翻转）· `ui/autodown_editor/` · `ui/handler_codegen.rs` · `ui/hot_reload.rs`
- `ui/mcp_server.rs`（AutoUI MCP 调试服务）· `a2ui/schema.rs:A2UIMessage`

## 使用示例

```auto
widget Counter {
    setup { greet = fn() { print("hi") } }   // 每实例前导（426）
    msg { Inc, Dec }                          // 448 起可去名
    model { count int = 0 }
    view { col { button + { onclick: .Inc } h2 > Count: ${.count} } }
    on { .Inc => { .count += 1 } }
}
```

## 媒体元素与媒体服务（PLAN-617）

### `video` 受控媒体契约（SD-01）

`.at` 作者对 `video` 声明**任一**受控下行 prop（`paused`/`position`/`volume`/`muted`/`rate`）
即进入受控模式：状态 → 元素属性下行同步，原生媒体事件 → 状态上行回灌。

- **下行**（状态 → 元素；两端同名同单位，与 `ui/mpv/contract.rs` 同形）：
  `paused`（bool，false → `play()`）、`position`（秒；**目标值变化才 seek**，换片作废）、
  `volume`（**作者面 0..100**，Vue 生成器翻成元素 0..1）、`muted`（bool）、
  `rate`（float；≤0 被忽略）。受控 prop **不作为元素属性发射**（`:paused` 会打到
  只读 DOM 属性、`volume` 单位不同）。
- **上行**（元素 → 状态）：`ontimeupdate($0: float 秒)`、
  `onloadedmetadata($0: float 秒)`、`onplaystatechange($0: bool，play/pause 合成)`、
  `onended`、`onmediaerror($0: str 真实文案)`、`onaudiotrack($0: bool 音轨可用性)`。
- **`onaudiotrack` 语义（AC-16b）**：仅 Vue/Chromium 发射——元素实际播放超过 1s
  后读 `webkitAudioDecodedByteCount`（Chromium 专有探针），恒 0 ⇒ `false`
  （音轨编码无浏览器解码器，如 Dolby Digital Plus），应用**必须如实标注
  「音轨不支持」且不得假装有声**；探针缺失（非 Chromium）不调用 handler
  （不主张任何结论）；VM/mpv 端无此事件（mpv 自带 Dolby 解码）。
- **兼容边界**：未声明任一受控下行 prop 的 `video`，生成结果与引入契约前
  **逐字节一致**（裸 `<video :src :class />`）；由
  `test_uncontrolled_video_is_byte_identical` 钉住。
- **现状**：Vue 端已实现（`ui_gen/vue.rs` 的 `try_generate_controlled_video_html`
  / `video_script_block`）；iced 端为 **partial**（见「已知坑」的 SD-05 块）。

### 媒体文件服务（SD-02）

`crates/auto-lang/src/ui/media_service.rs` 提供**平台级**本地媒体索引与字节流，
生成后端经原始路由（`auto_media` 先例）暴露：

- `GET /api/media/scan` → `{entries:[MediaEntry], root_missing}`；
  `GET/HEAD /api/media/stream/:id` → HTTP Range 字节流
  （206 + `Accept-Ranges` + `Content-Range`；无 Range → 200；越界/畸形 → 416）。
- **`MediaEntry` 契约**：`{id, name, rel_dir, relative_path, extension, bytes,
  size_str, video_url, title}`；`id = blake3(relative_path)` **令牌**——
  流式端点只按 `id` 反查文件，**绝不接受请求方传入的路径**（杜绝任意文件读取），
  **绝对路径不出后端**。
- **递归语义**：`index_directory` 递归遍历媒体根（**不跟随 symlink/junction**，
  防环），扩展名白名单（`mp4/m4v/webm/mkv/mov/avi`——只是「候选」，真实可播性
  由解码端决定），组内 `natural_sort_key` 自然排序。
- **大文件约束**：流式**必须惰性分块**（`ReaderStream`），单文件可到 GB 级，
  **禁止整文件读入内存**。
- **根目录解析序**：`AUTO_MEDIA_ROOT` env（显式，优先）→ pac.at `media_root`
  （`auto run` 仅在 env 未设时注入子进程）→ 无（诚实空库；
  `root_missing=true` 让前端区分「目录不存在」与「目录为空」）。

### 030-video-player 示范（SD-03）

本仓视频类权威示范的**当前形态**：扁平发丝线布局（h-12 顶栏 + 视口 +
w-72 队列栏 + h-14 播控条）+ 真实目录队列（递归扫描、按 `rel_dir` 分组）+
受控 `<video>`（真实起播/暂停/seek/音量/静音/倍速/上下曲）+ 本地文件选择
（Web 端 File API → object URL）+ 逐项真实播放状态（音轨不可解时主动标注）。
取代 Plan 542 的「沉浸式大视口 + 预置远程 URL 队列 + 模拟状态」描述。

### 已知约束（SD-04）

1. **iced 端 `video` 降级是有信息的**：未开 `mpv-*` feature 或缺运行库时，
   渲染面画**诚实降级面板**（说明未启用/无解码能力），不是纯黑也不再静默
   （默认档）。
2. **本仓默认档没有任何视频解码依赖**：原生播放是可选 feature（`mpv-native`
   /`mpv-gpu`/`mpv-widget`），CI 不安装系统媒体包；库缺失走降级不 panic。
3. **容器可解 ≠ 全部可播（实测）**：Matroska 容器 Chromium **能**解
   （4K HEVC 画面实测可解，「Chrome 不支持 MKV」是误判）；但 Chromium 的
   FFmpeg 构建**不含 Dolby Digital Plus (E-AC-3/Atmos) 解码器**，该类文件
   表现为「有画面无声音」并被判为 video-only（失焦时 `play()` 抛
   `AbortError`）。**规范要求界面如实标注音轨不可用（onaudiotrack 契约），
   不得假装有声，也不得把「MKV 播不了」或「所有文件都能播」当默认假设。**

## store facade 消费位语义（PLAN-622）

### 五消费位契约（SD-01）

`store Name { ... }` + `use store: Name` facade（web 轨发射 Pinia，VM 轨原生解释）
的 VM 侧消费位语义钉死如下（语料：`plan622_store_facade_gap_tests` +
`test/ui/plan622_store_facade/`；实证基线 auto-lang master @ plan-622-dev）：

1. **handler 读**：widget handler 内按裸字段名读已合并的 store 字段（标量、
   嵌套对象字段）→ 读得 store 当前值（442 的 `store.`-前缀 musk 形态之外，
   裸名形态同义）。
2. **跨 facade 带参 msg 派发**：`store.Msg({key: v})` 单 map 载荷派发 → store
   handler 以 `args.key` 收参执行（载荷单类型约束不变）。
3. **视图响应回读**：视图绑定 store 字段（裸名与前缀两形态）→ store msg 变异
   后快照/重渲染跟随新值。
4. **store 内 #[api] 调用**：store handler 体内的契约 fn 调用与 widget handler
   同一 340 面——split 模式（api_over_http=true，AUTO_BACKEND）改写为 HTTP，
   merged 模式走宿主分派；不再落入模块内 stub 体。
5. **变异原语**：`List.splice(start, count)`（2071）——JS 移除形语义（负 start
   自尾计数、双向钳制、返回移除元素的新列表；元素 stake 由源容器转移到返回
   列表）。JS splice 的插入形变体 v1 不设，调用方以 insert 组合（规格注记，
   需要时另案扩参）。

### 闭包捕获编址契约（SD-02）

`compile_closure` 的捕获槽位元数据与 `emit_store_loc`/`emit_load_loc` 的编址
**同源**：local 域 `slot = idx - fn_scope_start - n_args`、参数域打 0x8000 旗标
（`real = idx - fn_scope_start`，运行期负向寻址）。禁用裸 scope idx 当槽位
——widget handler（fn_scope_start=1）内两者差一位，by-ref 读邻槽垃圾且遮蔽
env 值（此前 385/454 语料 fss=0 未暴露）。`self` 在闭包体内特判捕获所在
handler 的 `__state` 状态对象（此前 CONST_0，`self.field` 全部静默 nil）。
守卫语料：`plan622_a/a2/a3/b`（facade 读/派发/视图跟随）、
`plan622_e1/e2`（捕获两形态）。

## store facade 跨状态与短路语义（PLAN-624）

### 跨状态字段读消费位增补（SD-01）

PLAN-622 五消费位契约的增补（语料：`plan624_cross_state_tests` +
`test/ui/plan624_cross_state/`；split/merged 双模守卫）：

6. **跨状态字段读**：widget handler 内直读 store 字段（含嵌套对象字段、
   map 字面量实参表达式内读取、`?str` 双态——None 读回 nil 不崩、Some
   读回值本体）→ 读得 store 当前值。jade 实机「0 参 handler 派发带 λ 的
   store msg 后读写全断（Invalid object ID）」的真因在闭包帧协议
   （见下 SD-03），与 `?str` 编码无关。
7. **列表原生面增补（SD-02）**：`auto.list.find_index(λ)`（2072）——
   谓词消费与 find（2063）对齐，返回首命中下标，未命中 -1（此前静默
   失效求值为 Nil）。

### `&&`/`||` 短路值语义（SD-01 附，PLAN-624 rev 2 用户裁定②）

`a && b` ≡ a 真值 ? b : a；`a || b` ≡ a 真值 ? a : b——与 web 轨 TS/JS
对齐，`.at` 惯用法 `fm && fm.title || fallback` 逐字可用（falsy 链值透传
fallback 本体，不再静默布尔归一写成 `true`）。真值判定沿用 JMP_IF_Z/NZ
既有判定面（Plan 406 nv_truthy：tagged bool 权威、null/0/哨兵假）。
纯布尔用法值不变；存量 .at 语料 335 处 `&&`/`||` 用法清点均为纯布尔面
（tv 全量 3706/3706 绿证零语义依赖）。

### 闭包激活帧协议（SD-03，执行期新增）

闭包激活（`call_closure` 方法与 `CALL_CLOSURE` 码两路）必须入 call_stack
恰好一帧，闭包体末端 RET 弹恰好一帧——CALL 帧协议对闭包闭合。违反形态
（激活不入帧）下闭包 RET 偷走**外层函数**的帧：call_stack 错位使最深被调
帧的 `current_fn_n_args` 泄漏进外层，参数寻址（`0x80+idx` 负偏移）越界走
NULL 守卫——0 参 widget handler 派发带 λ 的 store msg 后读 `__state`
即得 NULL 哨兵（SET_FIELD "Invalid object ID: 0xFFFFFFFF80000001"，
jade facade 切换实机 P2 面真因）。带参 handler 的泄漏值恰等自身故不可见；
守卫语料须覆盖「0 参 handler × store λ 派发」组合（plan624 p2 臂）。

## icon 字符串协议族（PLAN-018）

icon 通道是**字符串协议**：pac `icon:` → 注册表 `AppRegistryEntry.icon` →
shell 注入 → 渲染臂按前缀分发。三个后端 + 占位，按回退链求值：

| 前缀 | 后端 | 实现 |
|---|---|---|
| `iconfile:<stem>` | 双主题位图文件 | `iced/icon_file.rs`：`{root}/{light,dark}/<stem>.png` → `Handle::from_bytes`，(stem,dark) 键全局缓存（失败占位防重试） |
| `hicon:<slot>` | native 窗口真图标 | `iced/native_icon.rs`（Plan 515 D1，HICON→RGBA） |
| `lucide:<name>` / 裸名 | 内嵌 SVG 静态资源 | `lucide_svg_doc` 族 |

- **回退链**：`iconfile:` → `hicon:` → `lucide:` → 占位。任一级未命中
  （前缀不符 / 资产根缺席 / 文件缺失 / 提取失败）即下沉下一级——纯 lucide
  名的 app 渲染路径恒不变（零回归边界）。
- **资产根解析序**：`AUTO_OS_ICON_ROOT` env（桌面 boot 注入）→
  `AUTO_OS_ROOT/assets/icons` → 缺席。资产与映射表（`mapping.json`，
  registry id → stem）归 auto-os `assets/icons/`（PLAN-018 SD-02）；切片
  工具 `auto-os scripts/slice_icons.py --verify` 再生成。
- **主题**：dark 位取进程级 `theme::dark_mode()`（boot 期 `set_dark_mode`，
  PLAN-615 链）→ 选 `{dark,light}` 子目录。
- **接线**：桌面 boot 注册表快照组装后 `apply_icon_mapping` 把命中 id 的
  `entry.icon` 改写为 `iconfile:<stem>`（未映射原样），同时注入
  `AUTO_OS_ICON_ROOT`。vue 轨同前缀 → 双 `<img>` 对 + SFC 三行切换规则
  （宿主根 `.dark` class；无该机制宿主恒浅表）。
- **stem 白名单** `[A-Za-z0-9_-]`（拼路径前拒收穿越/杂字符）。

## api 模块形态与限定名调用（PLAN-627）

`use back.api` 两形态与限定名调用的发射契约（rust/vue 两发射器对称；
VM/桌面动态编译路径不在其列——qualified 直落本地字节码为既有行为）：

1. **模块形态抽取**：`use back.api`（无符号清单）与符号形态
   （`use back.api: a, b`）等价——函数清单自 `resolve_back_api` 定位契约
   文件的 `#[api]` 注解 fn 枚举（plain pub fn 不入清单；契约缺席/解析
   失败宽容降级空清单）。消费点为 `ui_gen/api.rs` 与 `auto-man/rust_ui.rs`
   抽取双写；布局寻得支持 `<root>/app.at` 与 `<root>/src/front/*.at`
   向上三层，外部后端 `back: { project }` 形态不支持（现网消费者均为
   符号形态，按需再补）。
2. **限定名发射等价**：`api.X()`（X ∈ 清单）在 rust/vue 两发射器与裸名
   `X()` 产物逐字节一致——vue：`await X(...)` 客户端调用 + SFC 头
   usage-driven `import { … } from '@/lib/api'`；a2r：裸名调用 + 单语句
   `.Init` async-Init 同走 `__InitLoaded` 形态。金样
   `plan627_qualified_api_tests`（模块 vs 裸名全产物对拍，含清单门与
   `__InitLoaded` 形态锚）。
3. **清单门守卫**：清单外 `api.` head 原样透传（防用户自建同名 `api`
   对象误伤）。
4. **VM 语义不变**：裸名 merged no-op 桩语义维持（PLAN-053，plan622
   守卫在案）；限定名直落本地字节码为 merged 进程内唯一可达形态
   （auto-term PLAN-017 实证）。

## 交互原语三件：hover 样式对 / popover pointer 定位 / 转换缓存（PLAN-631）

> 契约全文 = docs/design/29-autoui-style-theme-system.md §10（canonical）；
> 本节为现状摘要。剖析与 A/B 证据：docs/plans/archive/631-*/evidence/。

### MouseArea hover 样式对（SD-01）

`mouse-area` class 串支持 `hover:` 变体（消费 `Style::variant_classes`
既有面，零新增解析）：声明即构造 HoverFlag + `HoverArea` 包裹 + 容器样式
闭包 base/hover 二选一——**零 VM 消息零视图重建**（只 request_redraw）；
无声明零开销路径不变；`onmouseenter/onmouseleave` 事件臂保留兼容。消费示范
027 树形折叠箭头（`hover:bg-accent/60`）。

### popover `placement: "pointer"`（SD-02）

面板 open 翻真时原点 = 渲染器会话记忆的**最近一次指针按下位置**（左/右键
同记；触发件与面板可分离，单实例菜单挂视图根）。机制：窗口根单包装
`PointerPressArea`（`iced/right_press_area.rs`，ButtonPressed 事件现场读
cursor 记账，进程级单槽 f32 位型）→ `pointer_panel_anchor` 归一（未记账
回退 BottomStart 锚件语义；snap/翻转钳制沿用）。无 x/y 的 pointer popover
由转换器合成原点点锚；坐标不进 VM 状态、不经消息回路。已知边界：跨 App
窗口共享单槽（最近写入即正确锚）；触屏/无指针设备不做。

### 动态视图转换缓存（SD-03）

`Style::parse_reported` 类串 intern 缓存（`ui/style/mod.rs parse_cache`）：
键 = (类串原文, 6 位主题门控槽 = sm/md/lg/xl/2xl 命中位 + dark 位)——键对
parse 输出完备（responsive/dark 门控只做布尔比较），窗口 resize 同断点域内
恒命中；命中克隆已解析 Style；类串表上限 4096 兜底清空。A/B 开关
`AUTO_STYLE_CACHE=0`（默认开）。剖析仪表 `P631_PROFILE=1`（renderer 逐重建
帧吐 `[P631-PROFILE]` 行）。量化（027 67 行选中，debug）：parse p90
13.2→1.4ms（~9x）、整重建 15.8→9.8ms；opt 档噪声内。结构 diffing 评估：
不立项（行级 memo 为第一候选，触发门槛见 evidence diffing-eval.md）。

## terminal iced 绘制契约与像素门禁（PLAN-634）

> 契约全文 = docs/specs/widgets/terminal-iced-draw.md（canonical）；
> 归因全文 = docs/plans/evidence/634/pixel-golden-drift-attribution.md。

### draw 期段落强引用规则

iced wgpu `fill_paragraph` 排队 `Arc::downgrade` 弱引用，flush `upgrade()`
失败**静默丢弃**——draw 闭包内局部段落 fill 后析构，文字必不进像素产物
（quad 值拷贝不受影响，表现为"底色块在、字没有"）。三处范式（terminal
widget.rs）：行文本 `ROW_CACHES`（digest 门控）、菜单标签 `MENU_PARAS`
（静态串 OnceLock）、badge/preedit `PLAIN_PARAS`（键 (text,width)，封顶
64 溢出清空）。不变式：强引用存活到 flush；guard 持有跨越 fill 调用；
动态键缓存只许封顶清空、不许悬垂。a2r 侧同族规则（语句位置块尾恒补
`;`）见 a2r-std/project.md（#18 E0308 实证）。

### 像素金样环境契约

headless 像素产物字节面绑定三张环境敏感牌：wgpu 适配器/后端枚举
（`Renderer::name()` 恒 `"wgpu"`，后端翻转不换金样后缀）、MSAA×4 合成、
系统字体栅格（cosmic-text fontdb）。漂移=环境态翻转非代码（015 同日晨绿
午后红跨树复红 + 634 三后端探针零漂移互证）。门禁语义：硬断言=墨水占比
/同进程双渲染字节一致/层容差差分（阈值按金样实测标定，分离度 ≥2×）；
金样留档+审计制（超 5% 预算仅告警）。像素/可见性测试一律 nextest 跑
（裸 cargo test 单进程共享注册表/静态缓存会互踩——634 实证）。

## 已知坑

- **`video` 元素：Vue 是原生 `<video>`，iced 是原生命中播放面（PLAN-617；SD-05）**：
  - **支持级别**：`schema/aura.at` 的 `video.backends.iced` 为 **`partial`**
    （由 `fallback` 提升）。`render_support` 同步为 partial，`ignored` 列
    `poster/preload/playsinline/autoplay/controls`（浏览器专有语义，iced 端无对应）。
  - **实现位置**：`crates/auto-lang/src/ui/mpv/`（`engine` 生命周期 / `channel` 帧上屏 /
    `contract` 受控契约 / `widget` iced 渲染面）。帧通道**不经 iced 的图像/atlas 通道**
    （`Handle::from_rgba` 每帧新 id、同 id 命中即不再上传、大帧超 `MAX_SYNC_SIZE` 与
    atlas 2048 上限——那条路必然闪烁），而是自持**持久纹理**、每帧原地更新。
  - **可选 feature（可降级）**：native 播放挂 `mpv-native` / `mpv-gpu` / `mpv-widget`
    三层 feature（`auto` 侧有同名透传与 `mpv` 别名）；**默认不开**——打开后任何带
    `video` 的示例都会真解码并开音频设备，故取显式开启
    （`cargo build -p auto --features mpv`）。**零构建期原生依赖**：libmpv 只在运行时
    `LoadLibrary`，因此 11 个 `ubuntu-latest` CI 任务**不需要任何系统媒体包**。
  - **运行库解析序**：`AUTO_MPV_LIB`（显式文件路径）→ 可执行文件同目录的
    `libmpv-2.dll` → **都没有即降级**。注意**不回落系统搜索路径**：显式路径给了但
    文件不存在时直接判为「无库」（否则「路径写错了」会表现成「莫名用了别的版本」）。
  - **缺失时的行为**：`MpvEngine::new()` 返回 `MpvUnavailable::NoLibrary`，**不 panic、
    不黑屏**；渲染面改画诚实降级面板（说明未启用/无解码能力）。
  - **架构约束**：mpv 的 render API **只有 OpenGL 与 Software 两个后端，没有
    Vulkan/wgpu**；本机 iced 实选 Vulkan，故走 **SW 后端**（帧写进映射内存再
    `copy_buffer_to_texture` 上屏）。GL 路径经评估不可达——wgpu 不公开外部内存导入。
  - **帧由谁驱动**：应用需声明 tick（`timer { XxxTick (every_ms: N) }`）驱动重绘，
    否则画面停在首帧。上行事件（`ontimeupdate` 等）由渲染面按帧采集，应用侧分发
    见 `ui/mpv/widget.rs`。
  - **受控媒体契约（§2.3）**：下行 `paused`/`position`/`volume`(0..100)/`muted`/`rate`
    + 上行 `ontimeupdate`/`onloadedmetadata`/`onplaystatechange`/`onended`/`onmediaerror`
    （`onaudiotrack` 仅 Vue 端，见「媒体元素与媒体服务」SD-01 节），
    两端同名同单位（Vue 侧生成器把 `volume` 再翻成元素的 0..1）。
  - **实测**：VM 实机（`test/ui/plan617_video_vm`）1080p 本地文件真实播放，
    seek 生效（`position: 8.0` → 播放中 `time-pos` 由 8 递增）；4K 端到端 38 fps、
    1080p 258–385 fps（含逐帧读回，为保守下界）。
- - **示例可依赖的 DSL/VM 子集（plan-616 实证；写 `.at` 前先看这条）**：① 文本不要写
  `text "…${x}…"`（两端都渲染成字面量）→ 用 `text <ref>` / `text <prop.field>`；② `view fn`
  只传对象 prop + 点路径（标量 prop 在 VM 端不参与文本绑定）；③ **禁用** `.field = []` 与
  「局部 `[]str`/`[]Note` → 状态字段」整赋值（VM codegen 抛 `Assignment to complex LHS`；
  需要重建列表时用类型化局部量构建后整体赋值，或 `push` 到状态字段）；④ **视图条件里不写方法调用**
  （`x.contains(y)` 在 VM 端恒假，`resolve_binding_path` 不支持方法调用）→ 过滤下沉 store 产索引表；
  ⑤ VM 不支持 `flex-wrap` / `transition-*` / `group-hover:` / 任意 rem 值 / `aspect-*` /
  `text-transform`；⑥ 按钮无显式宽度会按 Fill 撑开布局，需 `w-auto`；⑦ 需要按 label 寻址的控件
  （MCP 场景）必须有文字标签——纯图标按钮 `autoui_find` 找不到。完整探针方法与清单见
  `examples/ui/015-notes/tests/acceptance.atd`「示例侧 DSL/VM 使用约束」节。
- **测试 storage/管道全局态卫生（Plan 489，P487-2 收敛）**：两条铁律——
  ①凡断言「键缺席回退默认」或落盘链的测试，一律 `t2_isolate_storage` 隔离
  （`AUTO_VM_STORAGE_FILE` 指临时文件；`storage_raw_remove` 只清内存，
  `storage_load` 会把盘上键并回——实机桌面用过设置面板后 store 必含
  `shell.dock.*`，487 写回链生效即打破无隔离测试的前提）；②broker/管道类
  测试一律 pid 后缀管道（`adjudicate_on` 参数化缝 / `Broker::on_pipe`），
  禁止依赖生产固定管道 `autodesk-broker` 的全局命名空间状态（本机任何
  桌面宿主 listen 即打穿 Standalone 断言——间歇红根源）。i18n corpus 的
  `front/i18n/{lang}.json` 属必备资产（.gitignore `*.json` 母规则需
  `!test/**/i18n/*.json` 否定）——落测不入库则 fresh clone 必红。
- **VM input 焦点寻址（Plan 483）**：VM 轨 text_input 每框派生唯一稳定 Id
  （`derive_input_id`：on_change 的 widget+event 主键、placeholder/width/
  password 三元组兜底），渲染期 `collect_input_ids` 依 DFS 序登记进
  per-App `devtools.input_ids`；自动聚焦路径（`__focus_input` 约定/未捕获
  Tab→`__focus_prompt`/PromptBar refocus/launcher 召唤）一律按登记表取首个
  input Id 寻址——**严禁**回退「全窗固定 `prompt_input`」写法（iced Focus
  operation 对同 Id 的全部 focusable 一次全置焦 ⇒ 多 input 视图双焦点+
  键盘双投递，上游 auto-musk 011）。焦点跨重建由 iced Tree 槽位 diff 保持
  （Plan 047 on_submit 语义不受影响）。
- **VM input 焦点环遍历（Plan 491，483 登记表之上的扩展）**：未捕获 Tab/
  Shift+Tab 由 `keyboard_event_message` 按 `modifiers.shift()` 分派
  `__focus_next_input`/`__focus_prev_input`（Named 臂无修饰前缀——Shift+Tab
  与 Tab 同命中 "Tab"，必须在此分流）；update 遍历臂走两段链
  `operate(FindFocusedInput).then(focus_traverse→内建 focus)`——探针经
  Operation 遍历读**实际**持焦者（含点击直聚；无聚焦恒出 `Some(None)`，
  异于内建 `find_focused` 的 `Outcome::None` 断链），`focus_traverse` 按登记
  表 DFS 序回环求址（不在表内/无聚焦→首项，空表 None）。**登记表空（纯
  textarea 视图，ash-gui/028）回落 057 `__focus_prompt` textarea 优先链**。
  机制级 p491 七测（iced_test）；真键盘实机复验在 P483-3 真人清单（环境
  OS 键盘注入对 winit 不可达）。
- **桌面热键表数据级可配置（Plan 490）**：桌面级热键改查 `HotkeyTable`
  （session.rs——HotkeyAction 11 动作/KeySpec 解析/builtin 新默认 +
  `shell.keys.<action>` storage 覆盖，472 dock 配置同型；boot 期
  load_hotkey_overrides 读入，坏值静默回退）。**新默认**：Alt+Tab 退役
  （G1，Win11 系统保留；`shell.keys.cycle_window` 显式配置可复活）；
  分区切换 Ctrl+Alt+←/→ 迁 **Ctrl+Alt+[ / ]**（G2，Intel 核显旋转键
  冲突族；方向键可覆盖恢复）；launcher 主键 Ctrl+Space + **别名
  Ctrl+Alt+Space 双收**（中文 IME 抢键机实测——入口按钮文案标注）。
  订阅臂链 = 纯函数 `desktop_hotkey_message`（可测化内核）。
- **VM 轨布局件点击 parity（Plan 490 G4）**：`row/col/div(container)`
  挂 `onclick` 在 VM 轨此前被转换层**静默丢弃**（Vue 轨 onclick→@click
  泛映射已通）——028 launcher 候选行鼠标点不中的根因。修复三层：
  `View::Row/Column/Container` +`onclick: Option<M>`（map/convert 语义
  穿透）+ aura 分发点 `set_layout_onclick` 提取（tracked/untracked 六臂，
  沿 text onclick→Button 先例）+ `wrap_layout_onclick` mouse_area 包装
  （on_release 发射；inspect 模式自守卫）。**严禁**在转换层丢弃布局件
  事件声明——新增布局事件（hover/右键）走 View::MouseArea（484）或
  同型扩展。
- VM 组件三缺口（449 实测）：回调 props 退化（组件一律无 props 读 store 规避）、快照组件子树不可见、
  片段参数化条件不求值——修好前 041 组件化受限。
- 446 批五（U2–U6）在 worktree 待 merge；463 合入后需实测内存给 386 提供数据。
- 455 矩阵多数示例（008–025）仍 Pending Audit；示例矩阵编号与目录有漂移（todo 在计划内）。
- 桌面线由并行会话活跃推进：WM 无独立 wm.rs 文件（WmState/WmCommand 实体在 session.rs，Plan 471 实测
  校正）；virtual_window 的 schema/aura.at 契约登记（462-I4）master 暂未见，随桌面线收尾合入。
- 465 未落地前，vue codegen 的 modal `position:fixed`/teleport-to-body 假设与虚拟桌面容器冲突（改造点已定位 vue.rs:6310/3599/4057）。
- a2vue 工程庞大（vue.rs 万行级），缺陷修复走 444 式"五类分簇"模式；013/015/011 构建失败系 master 预存（R006/R007）。
- router 双语法并存（Plan 105/106，见 docs/router.md）。

**505 桌面 DEBT 批处理一期（四族清偿）**：A 交互时序——原生槽位事件泵单发
16ms try_recv 改 `drain_slot_events` 每拍排空 + MoveSizeStart/End 稳定分区
前置（快甩同批即判；`NativeSlotEvents` 批消息形态）；B shell 面五瑕疵——
shell.at 任务栏双分支收敛 flex-col-reverse 单份 + `__dock_border` 宿主投影
边线、投影协议 v1.5（`pager` 旗标 + `more` "+N" 派生面）、a2vue 注册件
props 透传、daemon 发现序三级 PATH、`shutdown_broker` 五退出点；C 实机
验收通道——`autoui_desktop` MCP 注入（DesktopInject 队列走真实按钮同一
消费臂，AUTOUI_ACCEPTANCE=1 门控）+ ADR 规程 + acceptance_channel.py 统一
入口，P487-1/P496-1/P501-2 三债实机照补拍归档；D P488-D4 on_dnd_finished
发起方锚定 + 壁纸热切换定案（天然支持）。债项 P505-1/2 见 KNOWN-DEBT。

**559 vue 双端嵌入债并案已落地（GOAL-007/009 收尾面）**：W2 四件上收——①`vue_event_param` 单点收窄 `$event.target.{value,checked}`（gen 树 TS2339/18047 清零）②store 组合式跨 store 限定调用 facade 化（`AuraStore.sibling_stores` + 组合式发 sibling 导入/reactive facade 常量 + ts_adapter `store_bare_heads` 自限定裸发——vm A1 契约）③项目供给 TS 粘合安装 `install_project_api_glue`（契约抽取零端点=实现式 back/api.at 时 src/back/api.ts 孪生装入 gen lib/api.ts+dist；os-config 首试点：孪生真源签入 auto/src/back/api.ts，regen.sh 镜像 host）④`use back.api` 排除出 Plan 522 use-fn 拉取（TS2440/TS2304 根修）。W3 desktop-host api-client 守卫放开（gen 粘合/项目孪生择先，run 内先到先得+每次覆写防陈旧属主）+`desktop_extra_app_roots`（默认探测 `../auto-os-config/auto`，id=os-config 与 vm `extra_roots_from` 对齐，`AUTO_DESKTOP_APPS_EXTRA` 可覆）。W4 Taskbar ⚙️（emit settings，registry 在场性门控）+宿主 `launchSettings` 聚焦-或-启动（vm 551 T2 对齐）。W6 通用编辑器字段级挂载（`entryAtW`×2——vm merged 真源 auto-os-config-back/api.at 同步，漏改即 launch 不可用——+ConfigEditor `widgets` prop（Modules.active_widgets 装载一次零额外 HTTP）+wallpaper_picker 渲染分支；drop-in 夹具 p559-fixture 双端实证点选落盘）。W7 `autoui_desktop` handler 增 (app,widget) 子组件定位维度（`DesktopInject::Handler.widget` + `DynamicComponent.call_widget_handler` namespaced 派发 onclick 同管线——Plan 320 单 VM 统一态恒根 state id）+验收场景 p559（Pick→config.at 断言→已应用，幂等基线 PUT 重置）。对拍 Desktop 页双端三 shots（顶部标签/外观壁纸卡/Settings 卡同源）。门禁 tf 3426/3427（唯一红=charts 存量甄别）+desktop_protocol ui-iced 120/120+auto-man 245/245。债 P559-D1 证伪（AUTO_HTTP_PROXY 实际透传正常——404 系陈旧 vite 占港+auto-increment 漂移假象）/P559-D2 regen.sh 两族已上游化 sed 已清；P559-1..6 台账。

**573 ui-gallery 左栏 sidebar 族化（GOAL-010/007）**：549 示例画廊左侧导航从手搓 `aside+button+style-if` 迁移 sidebar 族（provider/header 包 pills 筛选器 + scroll(ScrollArea) 包 menu/menu_button(active: 契约)），双行卡片按 015-notes 惯用法（menu_button 内单 col 孩子 + h-auto 覆盖契约 h-8），手搓 active/hover 类串零残留；执行期根修一处约束链缺口——aside 只写 `flex-col` 无 display:flex（Tailwind `flex-col` 不隐含 display）→ provider flex-1 塌缩、ScrollArea 不受约束窗口级滚动，对齐 widgets-gallery `md:flex` 先例补 `flex min-h-0`（**惯用法：aside 外壳挂 sidebar_provider 时必须带 display:flex + min-h-0**）；VM 端结构树/pill press 正常，demo 列表空为预存限制（registry 全系 Vue-only TS extern fn `demos.ts`，master 基线同败）——跨端化留待后续立项。**625 ui-gallery VM 视口实装（registry 双产物跨端化 + AppViewport VM 形态，清偿 573 待澄清①②与 P573-D1）**：`generate_gallery_host` 升级双产物——TS registry（web 臂，含 `load` 动态导入，不变）+ `src/front/registry.at`（VM 臂纯数据表：id/title/category/icon/description/tags/doc/source/pac_text/loadable + 生成期预计算 `search_lc` 小写搜索堆料；字段避 `.at` 关键字 `pac`→`pac_text`），app.at 以 `use registry: filter_demos, demo_title…` 消费（tree_util PLAN-522/614 双端同源先例），`filteredDemos`/标题/描述/教程/源码七项 computed 全部 VM 可执行；`run_vm_ui` 增 registry 刷新 hook（vue 臂每次 run 刷 generate_gallery_host 先例）。**侧栏 aside 惯用法扩全（T-04）**：aside 外壳挂 sidebar_provider 必须带 `display:flex + min-h-0 + h-full` 三件——缺显式高度时 provider 的 `h-full` 在 iced 布局塌缩 0×0（Row 交叉轴无 CSS stretch 语义），整子树"树在/像素无"（headless 守卫 `p625_uigallery_sidebar_pills_visible` 双形态钉住：无高度类=0×0 哨兵 + h-full=正尺寸）。**`bg-clip-text` 渐变裁剪文字降级（T-05，声明式）**：iced 无文字填充渐变——`BgClipText` 标记 + text-transparent 清空回落继承可读色 + 渐变底盒抑制；from-/via-/to- 补语义色回落（`from-primary`/`to-primary/60` 原不可映射，`parse_color_with_alpha` 自带 /N 与主题解析）。**循环体事件惯用法定案（T-06）**：for 内 onclick 用 msg 带参形式（`onclick: .SelectDemo(demo.id)`，027 `OpenItem(item.id)` 同型——循环变量分发期求值）；lambda 捕获循环变量为 handler 合成不支持形态（编译器能力缺口，见 KNOWN-DEBT）。**R002 校验器字符串剥离**：`store.` 引用检测前经 `blank_string_literals` 剥离字符串/模板字面量（registry 内嵌示例源码 11 处字面量 "store.x" 曾误杀构建；模板 `${}` 插值保守保留）。**AppViewport VM 实装（T-10，ext 链收口）**：`.vue` Component 导入探测同名 `.vm.at` 适配器（ext_stubs Component 臂——嵌套 `.at` component 随装随注册，fn/.ts 沿旧路 stub）；`AppViewport.vm.at`（生成产物）内 `use.web component Demo<Pascal> from "src/gallery/demos/<id>.at"` 导入改名子 widget 源并按 `.app` 条件实例化；发射过滤=loadable 且自包含（单 `widget ` 行首声明、无 .at 导入、无模块级 use——模块 use 指向示例自有模块，拷贝后 link 致命 016/026 实证，v1 跳过上报）。**实证**：VM 端侧栏 33 项渲染、分类/搜索过滤、条目点击切换详情、三 tab 内容非空、视口实时渲染 002-counter 且按钮交互联动（MCP 点 "+" ×2 → Counter: 2）；vue 构建 vite 全资产绿。**差异登记**：Rust 臂（--render rust）转译器对 Plan 522 模块 use 未支持（E0425 实测）+ .vm.at 适配链为 VM 加载器专属——Rust 臂实装转独立计划；VM 进程 AppHang 静默退出两类终态结论见 KNOWN-DEBT P625-D1。


**P619 容器盒模型与图标尺寸两条语言层规则 + 一个文档写法坑（GOAL-007）**：
1. **padding 覆盖次序 = 单侧 > 轴 > 统一 > legacy**：Tailwind 的 `p-2` 与 `py-1.5` 同时存在时
   按 CSS 源码序后者胜（VM 端此前 uniform 命中即 early-return，per-axis 覆盖被丢）；
   margin 同理，且 `m-*`/`mx-*`/`my-*` **整族**必须折算（iced 无 margin，折外部 padding 模拟）
   ——此前只读单侧字段，`mx-3` 等静默丢失（搜索结果行左缩进少 12px 的根因）。
2. **icon 尺寸权威 = `.at` 的 `size:`，优先级 = 显式 `w-*`/`h-*`/`size-*` 类 > `size:` > 缺省
   20px**（双端同一套：VM `aura_view_builder::with_icon_size` 折 Width/Height，Vue 生成器折
   内联 `style="width:Npx;height:Npx"`；块式 `style:` 与动态 class 同入判据——PLAN-617 并轨，
   PLAN-619 最初的 `:size` 绑定发射由此取代）。**已知残余**：无显式尺寸的**组件内**图标仍被
   shadcn 资产 `[&>svg]:size-4` 钉死 16px（缺省 `w-5 h-5` 类打不过资产选择器；带显式
   `size:` 的图标走内联样式、特异性可压过资产 CSS）——普通 div 内的图标不受影响
   （KNOWN-DEBT P619-D6）。
3. **lucide 文档构造必须单层包装**：字形真源 = `lucide_generated.rs` 全量 fragment 表
   （生成自 lucide-vue-next v0.312，1401 项，`lucide_fragment` 二分查找 + sidebar/file-icon
   两条遗留别名），`lucide_svg_doc_with` 必须从 fragment 按目标尺寸**单层**包装——把完整
   文档再套一层 24×24 `<svg>` 会按 16/24 二次缩放（619 实证：跨端 ink 偏小 ≈33%；617 换表
   时该坑复发过一次）。回归锚 `plan619_lucide_doc_renders_geometric_ink`（纯 CPU 栅格化，
   断言 ink 充满度 = 几何值 0.833）。
## 蒸馏来源

- 本模块 spec 于 2026-08-28 由 Plan 471 刷新：蒸馏 437–465 活跃计划 + 4xx 归档计划 + 365–428 早期 UI 计划。
- 设计层：[Design 20（分离架构）](../../../design/20-autoui-separation-architecture.md)、
  [Design 16（App 生成战略）](../../../design/16-app-generation-and-ai-authoring.md)、
  [design/autoui/](../../../design/autoui/README.md)（虚拟桌面三部曲）。
- 过程记录：`docs/plans/plans.md 索引表` + `docs/plans/KNOWN-DEBT-AND-RISKS.md`（445/449/414/422/444 条目）。
**571 button 缺省 variant 一等化（GOAL-007，Design 22 §1.2/§3 修订）**：`button` 缺省从"primary 填充别名"升一等 `default` variant——UA stylesheet 显式等价物（Web 裸 `<button>` 有浏览器预填兜底，VM(iced) 无此层，故以 variant 表显式承载）：中性填充 `bg-muted` + `border-border` 发丝描边（dark 下 muted 对 background 对比度 ~1.15:1，纯填充不可辨）；`primary`（+行为语义 `submit`）为显式醒目 CTA 档；`secondary` 深一档纯填充无边框（"有边框"归 outline 专属），token 与 muted 分档（dark slate-700 #334155 / light 暖灰 #e3ddd1）。单源 `ui/style/variants.rs`（VM 臂 convert_button 与 rust codegen 臂 with_button_preset 共用；preset 前置、user class 后类胜；动态 class 不注入无回归；ui feature 门控）——Vue 侧三源（ui_gen cva 模板、auto-man 烘焙资产 button/index.ts〔PLAN-457 烘焙补丁先例〕、auto CLI 内嵌模板）与 CSS 变量层（auto-man generate_index_css 等）逐源互锁测试锚定，复审首轮 fail 抓出 CSS 变量层第三源未分档（T10 收敛）。顺修 web 端 `variant:"primary"` 落空（cva 无 primary 键）。specs.json P571-1..6；债：iced `Color::Accent` 无解析臂（ghost/outline `hover:bg-accent` VM no-op）、Vue cva 多真源维护面（互锁已防漂移）。

**P011 mouse_area 命中带=内容盒（KNOWN-DEBT 候选：转换器级尺寸类语义）**：iced `mouse_area` 无自带 width/height，命中带=其内容盒；DSL `mouse-area` 的显式尺寸类（w-full/h-full/w-N/h-N）由转换臂（renderer.rs MouseArea 双臂）落在**外层包装 container** 上，对命中几何是 no-op——需要大命中带时必须让**内容件自身 Fill**。P010-F1 先例：desktop.at 空白菜单 popover（T36）以全桌面 mouse-area 为锚件，锚的命中带仅图标网格条带高，条带以下全桌面为 BlankPress/BlankMenu 命中死区（levitate/悬垂假设经 AUTO_STACK_PROBE 五配置矩阵+格条带几何微测证伪；修复=内容包 `w-full h-full` col，锚件几何/popover 放置语义/视觉零变化，auto-os `46a07cf`）。同族规则：P007-1"shrink 上下文（Popover 锚/行内）Fill 解析零高"。转换器级根治（style 尺寸类进 mouse_area 命中带）波及全 app 命中面，登记 KNOWN-DEBT 候选（PLAN-011 SD-01）。

**632 画廊内嵌 demo 模块组件桥接（ADR-20，清偿 625"6 模块 use 降级"中的组件/store 族）**：T-01 standalone 对照实验修订两处预设——016 store 桥接 standalone 全通（store→child 转换+模型并根+计算属性全绿），内嵌降级根因=use.web 适配器链（Demo*.at）携带的 StoreDecl 在 load_ext_imports_for_vm 才进 import_stmts、晚于 store→child 转换位——ext 装载后按名去重补转换（F1）；006 双缺口=dep 目录 item 命名文件（settings_popover.at）解析永不命中（F2：resolve_use_module dotted-module 探测，`deps/{dep}/{item_snake}.at` 候选复用，snake_case 优先）+适配器模块 use 链 widget 从不注册（F3：扫 visited 按显式 items 注册进 registry/child_decls，P545 bare use 不触发）；F4=适配器链符号别名补齐（视图 computed 内 `month_name(...)` 裸名查 exports miss → raw 模板回退根因；or_insert 根环优先）。**实证**：006 弹层开合（settings_open 状态断言+截图）、016 June/42 格填充网格/Selected 点击更新、002/003/011 回归巡检——内嵌 MCP 10/10 双轮（实现期+复审期）；plan632_demo_bridge_tests 5 测（T-02 红→绿）；tv 3715/3715、tf 3568/3569（唯一红=P615-D3 预存并发抖动，隔离恒绿）。**债**：P632-D1 App.Init 兜底告警（宿主根件无 Init 的既有形态）；006 standalone 的 deps/settings 空目录为 vue 臂 dep 管线产物缺口（环境面，非运行时范围）。


**641 tabs variant 形态词表（default/enclosed 连通形态）**：`tabs` 根节点 `variant` 两值词表——`default`=按钮托盘（零回归），`enclosed`=连通形态（激活 tab 与内容面板共享背景无缝、非激活扁平等高 cell 仅背景色差、条与面板层次分明）；Chrome（顶圆角）/IDE（直角）观感差异归 token 层（Vue `rounded-t-*` class、VM 根 style `effective_border_radius` 顶角），不扩词表（SD-02 治理：新增取值须双端同落+gallery 覆盖）。VM 轨从无 tabs 到全链：`convert_tabs` 复合标签折叠（受控最小集——`active`/`value`/`defaultvalue` 解析序 + `resolve_expr_to_value` 状态引用穿透 + `onselect`/trigger `onclick` 注参索引回调）、`convert_view_messages` 显式 Tabs 臂（除名 `_ => Empty` 折叠）、iced 渲染臂双形态（theme token + 顶角半径 + button 命中面透明化）；A2UI 线协议 `Tabs.variant`（Option 缺省省略，向后兼容）。Vue 轨：registry 四件 variant provide/inject（`autoTabsVariant`）——顺修两处既有隐患（wrapper 自闭合丢 slot、`defineEmits()` 未类型化断 v-model 转发）；**tabs SFC 三源同步纪律**（registry 手工源 / vue.rs WidgetTemplate 库模式 / auto-man assets 脚手架快照〔`@/lib/utils`+reactiveOmit 风格，patch 记 SNAPSHOT.md〕）。schema/aura.at + schema.rs 双源 prop 词表（value/active/defaultvalue/variant/onselect，iced partial），五表漂移围栏过栅（baseline 裁剪 tabs 孤儿两条）。契约细节见 [design/tabs-components.md](design/tabs-components.md)；双端验证 046-tabs-variants + vue-gallery tabs 页。
