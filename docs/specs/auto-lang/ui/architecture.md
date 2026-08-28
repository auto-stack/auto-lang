# ui 架构

## 结构图

```mermaid
graph TD
  subgraph 前端解析
    DIALECT["dialect/ui.rs<br/>UiDialect（scenario=ui 才生效）"]
    PARSER["parser.rs<br/>WidgetDecl AST"]
  end

  subgraph AURA IR
    EXTRACT["aura/extract.rs<br/>view/state/handler 三元提取"]
    TYPES["aura/types.rs<br/>AuraWidget/AuraRoute/LogicPayload/AuraApp"]
    SCHEMA["aura/schema.rs + schema_loader.rs<br/>+ schema/aura.at 校验"]
  end

  subgraph 代码生成["ui_gen/（AOT codegen）"]
    VUE["vue.rs VueGenerator<br/>(a2vue 主力)"]
    JET["jet/ (Compose)"]
    ARK["ark/ (ArkTS)"]
    RUSTG["rust.rs (a2rust)"]
    BLOCK["block/<br/>BlockRegistry/BlockSpec"]
  end

  subgraph VM运行时["ui/（VM 渲染 + 桌面运行时）"]
    VNODE["vnode.rs / vnode_converter.rs"]
    EVT["event_router.rs EventRouter"]
    BRIDGE["vm_bridge.rs + interpreter/"]
    HOSTS["gpui/ · iced/ · headless/"]
    STYLE["style/ class·theme·iced_adapter<br/>(458 主题)"]
    ACT["action_config.rs<br/>(418/423/451 Action 配置层)"]
    EDITORS["code_editor/ · autodown_editor/<br/>(413-428 内建编辑器)"]
    SESSION["session.rs 双层会话<br/>DesktopSession/AppSession (453/459)"]
    WM["wm.rs WmState/VirtualWindow<br/>(462 虚拟桌面 WM)"]
    MCP["mcp_server.rs 调试服务"]
  end

  A2UI["a2ui/<br/>A2UIMessage JSON 协议"]
  API["api/ #[api] 契约<br/>typescript/tauri/axum"]
  EXT["外部：schema/aura.at · stdlib/aura/widgets · blocks/ · packages/widgets · examples/ui"]

  DIALECT --> PARSER --> EXTRACT --> TYPES
  SCHEMA --> EXTRACT
  TYPES --> VUE & JET & ARK & RUSTG
  BLOCK --> VUE
  TYPES --> VNODE --> EVT
  BRIDGE --> VNODE
  VNODE --> HOSTS
  STYLE & ACT & EDITORS --> HOSTS
  SESSION --> WM --> HOSTS
  MCP --> VNODE
  A2UI -.agent 协议.-> VUE
  API -.前后端契约.-> VUE
  EXT --> BLOCK
```

## ADR 日志

### ADR-01: AURA 作为唯一官方 UI-IR，结构与逻辑绝对解耦
- 日期 / 来源：docs/design/08-ui-systems.md §AURA（raw/aura.md）
- 决策：从 widget 声明提取三个纯元素——视图树（无逻辑）、状态定义（带类型的响应式签名）、事件处理器——组成 AURA IR，所有后端只消费 AURA。
- 备选：A. 每个后端各自解析 AST（pros：无中间层损耗；cons：N 后端 × M 语言特性，行为漂移）；B. AURA 统一 IR（pros：单一对齐点、schema 可校验；cons：IR 表达力成为天花板）。
- 后果：正面——vue/jet/ark/rust 共用 `AuraWidget`；负面——新语法须先扩展 IR。
- 状态：active（`aura/types.rs:AuraWidget` 在役）

### ADR-02: handler 保留为 LogicPayload 而非提取期转译
- 日期 / 来源：docs/design/08-ui-systems.md §Extraction pipeline；`aura/types.rs:LogicPayload`
- 决策：事件逻辑以 `AstBlock`（AOT 后端用）或 `Bytecode`（VM 动态执行用）原样保留在 IR 中。
- 备选：A. 提取期直接生成目标代码（pros：后端简单；cons：IR 与目标耦合，失去多后端）；B. 双形态载荷（pros：codegen 与 VM 渲染共用同一 IR；cons：两种载荷需保持一致）。
- 后果：正面——同一 widget 既能 `auto vue` 生成 SFC 也能 VM 直渲（plan-327/333 验证）。
- 状态：active

### ADR-03: scenario/dialect 条件关键字，UI 关键字不污染核心语言
- 日期 / 来源：docs/design/08-ui-systems.md §Scenario-Based Programming；docs/design/dialect-extension-diagnosis.md §6.1（`dialect/ui.rs` 头注引用）
- 决策：`widget`/`msg`/`model`/`view`/`on` 默认是普通标识符；`pac.at` 的 `scenario: "ui"` 激活 `UiDialect` 后才按关键字解析。实现从"parser 直接查 session"演进为 dialect 注册机制。
- 备选：A. 全局保留字（pros：实现直白；cons：core 场景 `let widget = ...` 被破坏）；B. 条件关键字（pros：零命名空间污染；cons：解析器依赖会话状态，LSP 需同步读 pac.at）。
- 后果：正面——core/shell 场景无冲突；负面——`view` 与 core 参数模式关键字复用同一 TokenKind，靠语句位置区分（dialect/ui.rs 注释）。
- 状态：active（supersede 了 08 文档描述的 parser 直查实现）

### ADR-04: 路由语法 `use module` + 懒加载（Plan 106 取代 Plan 105）
- 日期 / 来源：docs/plans/archive/106-router-use-syntax.md；docs/router.md §Plan History
- 决策：`routes { "/" => use index }` 映射 `@/pages/index.vue`，生成 `() => import(...)` 懒加载；旧语法 `"/" => HomePage {}`（组件名转小写、静态 import）保留兼容。
- 备选：A. Plan 105 组件名直引（pros：语义显式；cons：PascalCase 文件名、全量静态打包）；B. Plan 106 use 约定（pros：懒加载、文件约定小写统一；cons：隐式约定需文档化）。
- 后果：正面——首屏 bundle 减小、pages/ 约定稳定；负面——双语法并存，生成器需同时支持（`vue.rs` 内 Plan 105/106 分支）。
- 状态：active（106 为推荐路径）

### ADR-05: a2vue 双模式 API 层（Tauri IPC / Axios HTTP 运行时探测）
- 日期 / 来源：docs/design/08-ui-systems.md §Frontend-Backend Communication（raw/frontend-backend-communication.md）
- 决策：从 `#[api]` 声明生成 `api-interface.ts` + `api-tauri.ts` + `api-http.ts`，运行时 `api.ts` 探测环境选择实现。
- 备选：A. 构建期二选一（pros：产物小；cons：同一份前端无法同时发桌面与 web）；B. 双模式生成（pros：一份代码两形态；cons：三份生成文件需保持同步）。
- 后果：落地于 `src/api/targets/typescript.rs`（另有 tauri/axum 目标）；015-notes（plan-288/354）为首个真实消费者。
- 状态：active

### ADR-06: Block = Skill（spec + reference 双产物，AI 生成而非预烘焙库）
- 日期 / 来源：docs/design/blocks/blocks-first-class.md §2
- 决策：block 不是预烘焙组件库，而是"自然语言 spec + 结构化 frontmatter + 每 variant 一份 reference `.at` + gotchas"；`auto block add` 由 AI 按 spec 现场生成定制 `.at`，消费者拥有输出源码、可改可 eject。
- 备选：A. 黑盒高配置组件（pros：复用即所得；cons：变体空间高维，props 爆炸——低代码地狱）；B. 纯示例代码（pros：零维护；cons：不算复用）；C. Skill 模型（pros：订制走 NL、验收靠 acceptance 清单；cons：生成可复现性需 reference 锚定 + 编译回路收敛）。
- 后果：`ui_gen/block/registry.rs:BlockRegistry` + 顶层 `blocks/`（form/data-display/editor/navigation）已按包格式落地；Phase B 生成器 CLI 待做（plan-343）。
- 状态：active

### ADR-07: block kind 词汇表圈住订制自由，eject 为天花板
- 日期 / 来源：docs/design/blocks/blocks-first-class.md §4、§7
- 决策：不定"万能 block"，而定 kind 分类法（Form/Data-display/Feedback/Layout/Composite），每类固定扩展点词汇表；订制超出词汇表 → eject 接管源码。配色/间距归 design token，不进 block。
- 备选：A. 无限 props（cons：不可枚举、AI 无稳定目标）；B. kind 词汇表 + eject（pros：灵活且可文档化；cons：eject 后 spec 改进无法回流——开放问题）。
- 后果：loading/error/empty 成为数据型 block 的强制槽（对接 Rung 2 数据生命周期）。
- 状态：active

### ADR-08: app 生成走"能力阶梯 × 基准阶梯"，拒绝一键生成与反向转译
- 日期 / 来源：docs/design/16-app-generation-and-ai-authoring.md §3、§4、§7
- 决策：AI 生成完整 app 按 Rung 0-5 能力阶梯推进，每阶配 (编译器特性 + gallery 示例 + skill 条目 + 基准 app 评测) 四件套；基准 M1-M6 各覆盖一个互不重叠的能力簇，以"修复轮次 N"为度量。
- 备选：A. 一键生成整个 app（cons：错误无信号、不可迭代）；B. Vue→Auto 反向转译（cons：lossy，背离"AI 直写 Auto"初衷）；C. 阶梯式（pros：失败模式可定位；cons：周期长）。
- 后果：M1=015-notes 扩展（plan-338→354/357/360 系列）；widget 库扩容（plan-337 TODO-A）与 app 生成是同一攀登的两条腿。
- 状态：active

### ADR-09: 组件契约单源 schema/aura.at，数据流翻转
- 日期 / 来源：plan-435（2026-08）
- 决策：散落 8 处的组件定义收敛为一份 `.at` schema（扩展 aliases/tier/backends/deprecated 字段）+ 统一注册表；schema 管契约、Rust 管行为，render_support/vue import 映射从 schema 派生，CI 拦截漂移。canonical=kebab-case，变体进 aliases。
- 备选：A. Rust 硬编码为源（cons：漂移实证 8 处）；B. schema 单源派生（pros：一处改处处生效；cons：派生链需维护）。
- 后果：`schema/aura.at` 成为 widget/chart 契约唯一源；462 的 virtual_window 也经此登记（I4）。
- 状态：active

### ADR-10: DSL 现代化——widget 单轨 + setup 三相位 + msg 简写
- 日期 / 来源：plan-425/426/436/448（2026-08）
- 决策：component fn 降为 AST 级语法糖（parse 期产出等价 WidgetDecl，删 fragment 双轨）；widget 生命周期定版三相位 setup/.Init/.Destroy（setup 为每实例、首渲染前语句槽；vue 置于 script setup 顶层、解释器 L1 单实例、a2r 显式报错而非静默丢弃）；msg 声明去名 + 事件内联 lambda 简写（铸名提前到 parser，修复 decl 路径静默吞 HandlerNotFound）。
- 备选：双轨保留（cons：199 行重复与行为漂移实证）；a2r 静默丢 setup（cons：违背显式报错哲学）。
- 后果：`.at` 渲染路径唯一（Element+known_sub_widgets）；`use.web` composable 降为糖。
- 状态：active

### ADR-11: Action = 声明/绑定层，Event = 执行层
- 日期 / 来源：plan-418/423/451（2026-08）
- 决策：Action 定位为可寻址/可配置/可多路触发的语义事件（id 点分形式为 OS 键位层跨版本契约），最终仍派发为 on{} handler 事件，VM 分发零改动；actions{} 并入 widget DSL（外挂 .at 配置保留兼容），配置层支持热重载（Arc<RwLock<Arc<UiActionConfig>>> 零锁读、坏配置降级保旧值）、分层 keymap（app 内置→OS 用户层覆盖）、表达式条件（enabled_if/checked_if 走 Expr 求值）。
- 备选：Action 直连执行（cons：不可配置/不可多路）；外挂配置文件（cons：双源、LSP 不可见）。
- 后果：vue 侧补全链路（全局 keydown 回退层 + menubar/toolbar 组件树合成）；rfd 对话框/剪贴板/undo 等 natives 落 catalog。
- 状态：active

### ADR-12: VM 组件写法边界——无 props 读 store
- 日期 / 来源：plan-449 实测（2026-08）
- 决策：VM 后端组件回调 props 退化（on_xxx: msg 使组件整体 fallback）、快照组件子树不可见、片段参数化条件不求值三缺口登记后，确立 VM 组件一律无 props、经共享 store 通信、handler 留根视图的写法（"013 式"）。
- 备选：等待三缺口根治（cons：阻塞 041 组件化与后续桌面 App 拆分）。
- 后果：041-auto-edit 拆 app+store+三组件零回归；根治后可解除约束（债务簿在案）。
- 状态：active（约束式，待三缺口修复解除）

### ADR-13: 会话化运行时与 iced daemon 多窗口
- 日期 / 来源：plan-453/459（2026-08，蓝图 reports/453-t4c、459-t1）
- 决策：run_dynamic_iced 拆双层会话——进程级 DesktopSession + 每 App AppSession（运行循环 State 即 DesktopSession，renderer DynamicState 溶解拆借）；iced 入口迁 `iced::daemon`（view 带 window::Id、按 app_of_window 路由），开窗经 Event::Window(Opened) 自捕获；修饰键唯一源载荷化入 DesktopState；update 包 catch_unwind panic 边界；一窗一 App 不变式，全窗关闭显式 exit。
- 备选：application 单窗口 + PENDING_WINDOW_OPENS 通道（cons：多窗口非一等公民、通道竞态）。
- 后果：为 462 虚拟桌面（窗口内再分层）与多 OS 窗口形态同时铺路；Subscription map 受 const 检查约束改 fn 指针 + 自定义 Recipe。
- 状态：active（supersede 了 453 前的单会话形态）

### ADR-14: 虚拟桌面路线 A——单 OS 窗口 VirtualWindow z-stack
- 日期 / 来源：Design 23（docs/design/autoui/virtual-desktop.md R1–R7，2026-08-26 转正）+ plan-452/462
- 决策：一个 OS 窗口内 N 个 App：VirtualWindow widget（候选 B 定案：Stack/clip/mouse_area 组合 + 全局事件状态机）承载裁剪/事件路由/焦点分区；WmState/Wid 窗口注册表；DM::Wm 第四消息变体（Focus/Close/Move/Resize/Raise）；键盘路由改桌面层前置段（独立模式零回归=配置差异非分支）。**翻转** plan-365 的"Windows 非 compositor"裁定（R2：DWM 下虚拟窗口组合可行）。路线 B（386 分离渲染）后置，接缝已预留。
- 备选：A. 每 App 一 OS 窗口（459 已支持，但非"桌面"形态）；B. 分离渲染进程（386，内存优势但复杂度高、复活条件未满足）。
- 后果：463/464/465 全部构建于此；MCP 寻址 (AppId,widget) v1 指向焦点窗（T8 冻结）；组合中失焦 discard/preedit 两项遗留。
- 状态：active

### ADR-15: 桌面 shell 即普通 AutoUI App（启动=挂载）
- 日期 / 来源：Design 24/25（docs/design/autoui/desktop-shell-and-launcher.md R8–R12、25-autoshell-dsl-unified-shell.md）+ plan-463/464
- 决策：桌面 shell（全屏 borderless 宿主 + 任务栏 overlay 槽 + 排布）与 launcher 都是普通 AutoUI App：排布为纯函数（free/grid/master-stack，单测锁定）；应用注册表=扫描 apps 目录 pac.at（补 icon/category 字段）；DesktopBus v0 生命周期命令（LaunchApp 等）；launcher 经 Ctrl+Space 召唤进 overlay、`desktop.launch(name)` 真启动（R11：启动=挂载新虚拟窗，launcher 自隐匿）；内核/用户态分界与 workspace 驱动模型见 Design 25。
- 备选：shell 特权内建（cons：违背"shell 是 AutoUI"统一层初衷，双端不一致）。
- 后果：shell 可用 vue/vm 双端同一套声明；examples App 默认 render:"vue" 需注册表按 render 过滤 + 失败占位页。
- 状态：active（463 落地；464 设计待实施）

### ADR-16: vue 宿主 = 页面级虚拟桌面（设计裁定，待实施）
- 日期 / 来源：Design 23 R4/R5 + plan-465（2026-08，已立项未开工）
- 决策：一个 vue 页=一个虚拟桌面：每虚拟窗一个 `createApp().mount(container)` 实例级隔离；virtual_window 的 a2vue/DOM 实现为 absolute+clip+pointer 路由（WM 的第四叶）；registry 必须构建期生成（vite 动态 import 需静态可分析）；Web 永远是 A 形态（不搞 iframe/多进程/BroadcastChannel）。
- 备选：iframe 隔离（cons：样式/主题/通信割裂，出界）。
- 后果：vue codegen 的 modal fixed/teleport-to-body 页面级假设需改造（vue.rs:6310/3599/4057 已定位）；tauri 壳全屏打包。
- 状态：proposed（465 施工后转正）

### ADR-17: 双端 parity 与主题下沉为引擎规范
- 日期 / 来源：plan-455/458 + 450/451-image/452-login 系列（2026-08）
- 决策：parity 不再是逐例修复，而是引擎级标准：Status::Focused 2px ring、text 标签盒模型、全 margin 语义、按钮 14px/font-medium 等；theme/accent 成为一等配置（优先级=运行时切换>CLI/pac.at 播种>内置默认 dark+indigo），经 env（AUTO_UI_THEME/ACCENT）横切三 crate，不进 DesktopSession 字段；视觉差异报告必须经 vtree/snapshot/插桩结构数据交叉验证才立项（411 方法论）。
- 备选：parity 逐例修（cons：标准漂移、回归无锚）；主题进会话状态（cons：三 crate 横切复杂化）。
- 后果：455 跟踪器矩阵为验收基准（~9 绿/8+ 待审计）；"auto 跟随系统"显式非目标。
- 状态：active

