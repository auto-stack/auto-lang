# Plan 442: 跨平台合龙——musk 五域端口接线 + VM 渲染能力补缺 + 后端 AutoVM 激活

> **状态**: 🟡 执行中（2026-08-23 立项；Phase 0（P0-1/P0-2 ✅✅ 双侧复核含 musk
> 环境）+ A4/A6（✅,A4 经 musk canary 重放复核）已落地；**前置门已于 2026-08-25
> 全满足**（429–434 complete/436 ✅/musk-038 execution_done/auto-down-008 COMPLETE），
> gated 主体开工：**Phase A 全部完成** —— A1 ✅（server 核验回填 §2）/A2 ✅（store
> facade）/A3 ✅（ext link 平台桩）/A5 ✅（sched 定时器）,均本仓 worktree plan-442；
> 剩余 B（musk 五域 adapter 接线）/C（后端 AutoVM 激活）为 auto-musk 侧动作）
> **来源**: auto-musk PLAN-038 待澄清 #7（接线边界划出后无人承接）+ PLAN-041 裁定
> （web 轨退役等迁移完成）+ auto-musk KNOWN-DEBT-AND-RISKS 028 ③（VM 渲染目标
> "归 VM 渲染目标立项"）+ auto-musk pac.at 头注（"后端用 AutoVM 脚本运行"激活线）。
> **关联前置**: Plan 429–434（AAVM v2 移植/a2r 闭环）、Plan 436（setup 相位）、
> auto-musk PLAN-038（第三方库 Auto 版）、auto-down Plan 008（渲染库 Auto 化/
> markstream 消灭/编辑库）
> **跨仓说明**: 本计划主跟踪在 auto-lang，但缺口 A/C 的动作面在 auto-musk 仓
> （ports adapter 文件、pac.at 目标切换）——任务表显式标注仓库归属。

## 1. 目标与缺口

迁移线两侧基础计划（auto-lang AAVM 系列 + musk 前端单源化/第三方库）完成后的
**合龙段**：让 auto-musk 真正以 Auto/vm 或 Auto/rust 形态跑起来。三个缺口：

- **缺口 A（musk 侧动作）——五域端口 VM/Rust adapter 接线**：PLAN-037 落定的
  `ports/{platform,composables,icons,renderer,upload}` 目前只有 `.web.at` 适配器；
  各域需要 rust/vm 目标的同名 adapter 并切换绑定（musk 038 已划出、无计划承接）。
- **缺口 B（auto-lang 侧动作）——VM 渲染目标能力补缺**：KNOWN-DEBT 028 ③ 登记的
  store facade 概念缺失（VM 渲染对 `store` 合成物报 Undefined variable）+ TS ext
  显式 link 错误；以及两个 038 canary 将暴露的能力前置项——svg 节点（auto-icons
  渲染层）、调度/定时器原语（markstream 流式行为）。
- **缺口 C（musk 侧动作，gating 在 auto-lang）——后端 AutoVM 激活**：pac.at 头注
  "后端用 AutoVM 脚本运行（待 #[api] server 修复后激活）"；hw/ag 双轨中 ag 轨
  转正、`musk serve` 以 VM 跑后端的切换与验证。

## 2. 现状盘点（2026-08-23 立项时已核实）

- musk 五域端口调用面已纯 Auto（`use pac.ports.<域>: *`，调用面 `use.web` 非
  `.at` 目标零命中——PLAN-037/424 收口）；`resolve_at_adapter` 的目标门控机制
  （`X.at` 端口 → `X.<target>.at` adapter，缺失显式报错）已在 auto-man 落地并有
  platform 域拆分 proof（PLAN-037 Phase 6 T22-T23）。
- VM 渲染目标对 musk 源的现有报错清单见 auto-musk KNOWN-DEBT-AND-RISKS 028 行
  （store facade / ext link）；svg 与调度原语能力未实测（musk 038 T9/T16 canary
  会给出结论，届时回填本节）。
  - **回填（2026-08-23，musk 038 执行完毕）**：T9 svg canary 实证**不支持**——
    .at UI 的 `svg`/`path` 元素经 a2vue 退化为 `<div :viewBox=...><div :d=.../>`
    （SVG 语义丢失）；musk 侧已降级登记 KNOWN-DEBT（数据层 52 图标已交付），
    A4 的语言层 svg 节点支持成为解除条件。T16 高亮决策裁定 (a) syntect 原生
    （三引擎 token 级一致不可达，≤71%），衍生本计划 A6 的只读高亮原语需求。
- `#[api]` server 修复状态**待核验**（pac.at 注释可能已过时——auto-lang 429-434
  前进很多）；ag 轨休眠镜像清单见 auto-musk KNOWN-DEBT 018（tools/spec_tools/
  orch_tools/server_serve，已评估收益为零、不阻塞）。
  - **回填（2026-08-25，A1 核验完毕）**：pac.at 头注"待 #[api] server 修复"**已过
    时**——Rust 版 VM 的 `#[api]` server 路径已完整可用：015-notes（musk pac.at
    引用的参照结构）`auto run --server=vm` 实测起服（8 路由，`vm/ffi/http_server.rs`
    serve_async），GET/POST/DELETE 端点均返回正确数据（Plan 312–349 修复系列 +
    Plan 061 外部后端契约定位补遗 8b3789556 之后链路全通）。剩余缺口登记：
    ① **AAVM v2（Auto 版引擎）无 HTTP native**——431 移植规范明确剔除（66 条
    UI/异步 native），若"激活"要求 Auto 版引擎跑后端需另行立项，当前工具链以
    Rust VM 为准；② KNOWN-DEBT 346/317 两条 e2e flaky（端口竞态/serve_async
    无受控 shutdown）不阻塞激活；③ musk `src/back/api.at` 是**契约桩**（fn body
    全 `return None`，仅用于 codegen 前端客户端 + SSE 触发），真后端逻辑在
    `backend/crates/musk`（hw Rust + ag auto-src a2r 产物）——C1 切换试验的
    动作面是后端**实现面**的 .at 化，体量另计。
- auto-down Plan 008 Phase 3 的调度端口（VM adapter）与本计划缺口 B 的调度原语
  是同一能力的两个消费面。

## 3. 前置门（全部满足才开工）

| 前置 | 计划 | 状态 |
|---|---|---|
| AAVM v2 移植 + a2r 闭环 + AA2R | auto-lang 429–434 | ✅ complete（全部归档） |
| setup 相位解释器/a2r | auto-lang 436 | ✅ 完成（2026-08-23） |
| 第三方库 Auto 版（i18n/icons/渲染切换/高亮决策） | auto-musk 038 | execution_done（2026-08-23，15/16 任务 + T2 转责本计划 P0-1；待 /auto-plan:review） |
| 渲染库 Auto 化 + markstream 消灭 + 编辑库定版 | auto-down 008 | ✅ COMPLETE（Phase 1-4 本仓侧全毕；验收 4 musk 端到端延期至 DEBTS.md） |

> **Phase 0 不受上表前置门约束**——两项均为独立可执行的修复/瘦身（来自 musk 038
> 执行期的用户裁定），gated 主体（Phase A/B/C）仍等前置全满足。

> musk PLAN-041（web 轨退役）**不在前置门内**——它与本计划互为对侧：本计划合龙
> 完成 = 041 的"迁移完成"条件达成，041 随即解挂启动。

## 4. 任务分解（gated；仓库归属标注）

### Phase 0 — 独立修复项（**不 gated，可先行执行**；来自 musk PLAN-038 执行期裁定）

- **P0-1 auto-man 依赖按使用裁剪**（来源 musk-038 T2/待澄清 #9，裁定选项 (ii)）：
  `crates/auto-man/src/vue.rs` `generate_package_json` 的依赖为全量硬编码（仅
  router/i18n/npm_deps 条件化），生态内所有 vue app 均携带全量声明。改为**按
  生成代码的实际消费面条件发射**，映射（musk-038 执行期实测校准）：
  - `code_editor` widget → vue-codemirror + codemirror + @codemirror/{view,state,
    language,search,lang-rust,lang-python,lang-javascript,lang-markdown,lang-json}
  - `toast()` 调用 → vue-sonner（ui_gen/vue.rs 已有检测先例）
  - 脚手架 ui/Button 实际生成 → reka-ui + class-variance-authority（+既有 clsx/
    tailwind-merge）
  - 其余（vaul-vue / vee-validate / @vee-validate/zod / zod / embla-carousel-vue /
    @vueuse/core）→ 对应特性消费点存在才发射（无消费点的 app 零声明）
  - 验收：auto-musk（不用上述特性）fresh `auto build` 后 `gen/front/vue/
    package.json` 对 codemirror/reka-ui/vue-sonner/vee-validate/zod/embla/
    @vueuse/vaul grep 零命中（musk deps-guard 的 TRANSITIONAL 区随之清零），
    且 `cd gen/front/vue && pnpm install && pnpm build` 绿；widgets-gallery
    （用 toast/sonner 等）重生成后依赖仍在、构建绿。
  - **✅ 已落地（2026-08-23，worktree plan-442）**：`VueDependencyUsage`
    标记检测（App.vue+全组件 SFC 语料；ui/button 标记带结尾引号防
    button-group 误配）驱动 `OPTIONAL_DEPS` 表按组发射；CodeEditor.vue 壳
    usage 感知同步（未用即剪，防 vue-tsc 咬到未声明依赖的壳）；sync 路径
    `package_json_deps_drifted` 双向漂移检测；npm_deps 去重。仓内测试 29/29
    （vue 模块）。
  - **✅ musk 侧复核通过（2026-08-23，musk 会话,新 CLI=master ebbdc647）**：
    ①auto-musk fresh 全量 build（detached 干净 worktree,零既有 gen）→
    gen package.json 对 codemirror/vue-sonner/vee-validate/zod/embla/
    @vueuse/vaul **grep 零命中**（reka-ui/cva/clsx/tailwind-merge 为
    Button 脚手架真实使用,正确保留——验收原文把 reka-ui 列入零命中清单系
    措辞笔误,按映射语义以"未用依赖零命中"为准）；②CodeEditor 壳未生成,
    gen src codemirror 引用清零,musk deps-guard 的 TRANSITIONAL 过渡区
    已删除恢复严格白名单且 exit 0;③widgets-gallery `--render vue` 重生成
    →依赖发射正确（toast→vue-sonner、code_editor→codemirror×11 按使用
    保留）。**观察项（非阻塞,建议 follow-up）**：a) shadcn-vue CLI 添加
    button 时注入 `@lucide/vue ^1.33.0`（注册表 button 图标依赖,gen 源
    零引用——工具链痕迹,可接受或后续钉住）;b) gallery vue 轨构建红为
    **存量 demo 类型错**（toast.at `position:'center'` 不在 vue-sonner@
    2.0.9 Position 类型集;仅改 'top-center' 即 vue-tsc+vite 全绿,实测
    验证）——与 P0-1/P0-2 无关,1 行修复归本仓。
- **P0-2 CodeEditor 模板 setSearchEffect 类型错修复**（来源 musk-038 待澄清 #10）：
  `crates/auto-man/src/vue.rs` 模板（Plan 421 产物）发射 `import { setSearchEffect }
  from '@codemirror/search'`——该 API 在 @codemirror/search@6 实际导出面**不存在**
  （有 setSearchQuery 无 setSearchEffect）。后果：**任何新鲜 checkout 的
  `auto build` 全量 + `pnpm build`（vue-tsc）必炸**（既有 gen 目录因增量保留旧
  CodeEditor.vue 未暴露）。修复：改用 setSearchQuery（或等价改写 Ctrl+F 查询词
  注入路径），模板注释同步修正。验收：fresh scaffold 的 gen `pnpm build` 全绿
  （musk-038 #10 复核即本条验收记录）。
  - **✅ 已落地（2026-08-23，worktree plan-442）**：import/dispatch/注释/测试
    断言四处改 setSearchQuery。
  - **✅ musk 侧复核通过（2026-08-23,musk 会话）**：fresh detached worktree
    （零既有 gen）新 CLI 全量 `auto build` → `pnpm install` + `pnpm run build`
    （vue-tsc && vite）双绿（"Vue project built successfully"）,fresh scaffold
    不再复现 setSearchEffect 类型错（即 musk-038 #10 验收记录）。

### Phase A — VM 渲染能力补缺（auto-lang，先行）

- A1 核验 #[api] server 现状：AAVM 系列产物上重放 pac.at 后端激活路径，确认修复
  或登记剩余缺口（产出回填 §2）。
  - **✅ 已核验（2026-08-25，worktree plan-442）**：015-notes `auto run --server=vm`
    实测全通（起服/GET/POST/DELETE，见 §2 回填）。结论 = "待修复"注释过时，VM
    server 链路可用；剩余缺口三条已登记 §2（AAVM v2 无 HTTP native / 两条 e2e
    flaky / musk api.at 为契约桩、C1 动作面在后端实现面）。
- A2 store facade：VM 渲染目标引入 store 合成物概念（或显式报错指引改写），消除
  Undefined variable 警告；musk 30 widget 源作回归语料。
  - **✅ 已落地（2026-08-25，worktree plan-442）**：根因 = musk 的 legacy 形
    `use store: AuthStore`（模块名字面即 "store"）在 VM 装载层按 模块→`store.at`
    文件映射失败后**静默跳过**，StoreDecl 从未收集 → store 上下文为空 →
    handler/view 的 `store.X` 裸标识漏进 codegen 报 "Undefined variable:
    store"。修复 = `resolve_use_module` 公共解析函数（根 use 循环 +
    `collect_module_imports` 递归 + 测试镜像三处共用）：直接解析失败且模块为
    "store" 时,按命名约定（`snake_case(StoreName).at`,musk 九个 store 全命中）
    + 有界目录扫描（跳过 gen/node_modules 等,400 文件上限）定位 StoreDecl 文件,
    走既有 `collect_module_imports` → import_stmts StoreDecl → view-less child
    WidgetDecl 转换管道（Plan 370 机制）,含 legacy+unified 混用去重。回归语料
    `test/ui/plan442_store_facade/`（musk 形态：`store.Init()` 跨 widget 调用 +
    无点 `store.authenticated` view 引用 + store 文件自身 `use` 依赖）×3 测试,
    negativity 验证（禁用 fallback 即 3 FAILED）+ plan370 全组 18 绿。
- A3 ext link：TS ext 依赖在 VM 目标的显式 link 错误改为可配置跳过/挂平台桩。
  - **✅ 已落地（2026-08-25，worktree plan-442）**：根因 = VM 装载层对
    `Stmt::UseWeb` 完全无处理（scanner 走通用路径产出垃圾模块名 → 解析失败静默
    跳过）,纯 Auto 符号（`use.web platformInjectStyles from "…/platform.at"`）
    无定义,handler 调用留下未解析 CALL reloc → 整个 VmBridge link 失败
    （"Undefined symbol"）。修复 = `ui/ext_stubs.rs` + 生产公共函数
    `load_ext_imports_for_vm`：① `.at` ext 源经**端口 adapter 链**装载
    （`X.at → X.vm.at → X.web.at`,镜像 auto-man resolve_at_adapter 门控;项目根
    相对路径向上探三级）,adapter 纯 Auto fn 成真实符号（按文件 stem 限定名 +
    import_aliases 对齐调用点 reloc）,adapter 自身嵌套 use.web 递归收集;② 其余
    ext 符号（TS/npm 源）默认挂**平台桩**——按调用点扫描推断元数合成 no-op fn
    （VM RET 以 `bp - n_args` 展开,元数失配会坏调用者帧,必须精确）,warn 日志
    逐条可见,`AUTO_VM_EXT_STUBS=0` 恢复严格硬错。语料
    `test/ui/plan442_ext_link/`（port + web adapter + 嵌套 TS 依赖 + 直连 TS
    composable）×3 测试,negativity 验证通过。
- A4 svg 节点能力：按 musk 038 T9 canary 结论决定（语言层支持 / 挂账）。
  - **✅ 已落地（worktree plan-442：vue 轨原生直通 + 静态属性 / VM 轨 svgdoc
    文档渲染）+ ✅ musk 侧 canary 重放通过（2026-08-23,musk 会话）**：T9 同型
    canary（`svg { viewBox/fill/stroke/width/height, path { d } }`）产物为真实
    `<svg>/<path>`——静态属性原样发射、`:width/:height` 动态绑定保留,auto
    build exit 0。musk KNOWN-DEBT 038 条目已更新为"解除条件已达成,待 Icon
    widget 实现 + renderToString 对拍升级"（独立小任务）。
  - **✅ 语言层支持已落地（2026-08-23，worktree plan-442）**：vue 轨 map_tag
    SVG 直通臂（svg/path/circle/rect/line/polyline/polygon/ellipse/g/defs/
    use/stop/linearGradient/radialGradient/clipPath）+ 字面量属性静态发射
    （viewBox="…" 而非 :viewBox="'…'"，T9 退化根因修复）；VM 轨 svg 子树
    序列化 SVG 文档经 `View::Image{src:"svgdoc:…"}` 下发，renderer 复用
    svg::Handle 缓存渲染（单色 currentColor 文档走画时着色，多彩文档原色）。
    限制：动态 svg 属性/动画不支持（render_support 已登记 partial）；SVG
    text 子元素未支持（与 DSL text→span 冲突）。**musk 侧 T9 canary 重放 +
    icons 域解除条件验证待 auto-musk**。
- A5 调度/定时器原语：按 musk 038 T16 与 auto-down 008 Phase 3 的需求面定接口。
  - **✅ 已落地（2026-08-25，worktree plan-442）**：需求面收敛 = auto-down 008
    Phase 3 的 `SchedulerTimer`（单发可取消延迟回调,16ms 批节奏;DSL 已有
    `.Tick` interval,缺 one-shot）。接口定案 = `sched.set_timeout(callback,
    delay_ms) -> id` + `sched.clear_timeout(id) -> bool`（stdlib 三件套之
    sched.at/sched.vm.at;a2r 侧 rs.at 挂账待 B4 消费面明确后补）。实现：
    AutoVM 新增 `timers` DashMap 注册表（set_timer/clear_timer/due_timers/
    has_pending_timers,一次性语义 = 到期即取出）;native shim
    `auto.sched.set_timeout`(1206)/`auto.sched.clear_timeout`(1207) 接受
    **闭包值**（Int closure_id,满足 SchedulerTimer `setTimeout(fn,ms)` 契约;
    仅 by-value 捕获——by-ref 捕获读创建者已亡帧）或**事件名字符串**（走
    handler 派发,`.Tick` 相邻形态）;派发 = iced 渲染循环 `__timer_tick`
    subscription（16ms,**仅在有 pending timer 时订阅**,镜像 __toast_tick 门控）
    → `DynamicComponent::poll_timers`（事件→call_handler/闭包→call_closure,
    出错 warn 不致命）。回归 `test/ui/plan442_sched/` ×3（事件形式状态可观察/
    闭包形式计数可观察/clear_timeout 取消后永不触发）。
- A6 只读高亮渲染原语（musk 038 T16 决策 (a) 的落地需求）：041 code_editor 的
  highlight.rs（syntect 5 + two-face 0.4 内核）暴露 highlight-only API 或
  code_editor 只读模式——消费面 = VM 渲染目标的 markdown code_block 只读渲染
  （vue 轨继续 prismjs，双轨视觉近似已由 038 T15 矩阵背书）。
  - **✅ 已落地（2026-08-23，worktree plan-442）**：`highlight_segments(lang,
    text, dark, accent)`（syntect HighlightLines 走共享单例+预注册 autoui 主题，
    基前景色段为 None、相邻同色合并、未知语言退化单段）；语言通道 =
    `StyleClass::CodeLang`（lang-<token> class，惰性非视觉变体）由 codeblock
    分支携带；renderer `highlight_code` 有 lang 走 syntect（dark/accent 跟随
    theme_source），无 lang 保持手写 tokenizer（shell show 等零变化）。vue 轨
    按裁定继续 prismjs，未动。

### Phase B — 五域端口接线（auto-musk 动作，auto-lang 机制配合）

> **B 前置探针（2026-08-25，worktree plan-442）**：musk 53 文件全量语料经
> VM 渲染装载器 headless 探针实测——**全量走完 parse + codegen 直达 link
> 阶段**（A2/A3 修复在真实语料生效：九个 store 的 handler 均在编译,
> workspace_helpers 等 .at ext 源经 adapter 链装载）。据此本仓落地四项
> B 配合机制（详见下列 ✅）并产出剩余阻塞清单（auto-musk 侧动作）：
> ① `let` 重赋值 ×6（visual_store 的 initial/is_dark、workspace_helpers
> 的 has_sep/chosen、specs_helpers 的 max_num、settings_forge_helpers 的
> mode）——vue 轨语义宽松未暴露,VM 按语言语义正确报错,musk 源改 `var`
> （specs_helpers 未修 = 当前 link 致命阻塞的唯一残因）;
> ② `dom.focus_first/click_first`（app.at GlobalKeydown）与
> `location.reload()`（workspace_selector）浏览器全局直呼——归
> platform.vm.at adapter 桩;③ `self` 裸标识（specs_view SaveEditItem）
> ——musk 源修;④ 五域 `.vm.at` adapter 本体（web 侧均为 re-export 壳,
> i18n.at/icons_data.at 已是纯 .at 资产可直接直绑）。

- **✅ B 配合（auto-lang 侧,2026-08-25,worktree plan-442）——web 平台
  全局桥 ×4**：① `localStorage.getItem/setItem/removeItem` → Plan 401
  会话 KV 存储（getItem 缺失返回 None,对齐 musk `saved != None` 判定,
  native 2771-2773 + codegen localStorage 模块路由）;② `encodeURIComponent`
  bare 全局 → `auto.url.encode`（urlencoding crate,补齐 ID-map-only 的
  2000 条目缺 shim 绑定;此前是 musk 探针的 link 致命阻塞）;③ 子模块
  文件顶层 `use.web`（如 specs_view.at 引 specs_helpers.at）此前只在根
  AST 收集被漏——`load_ext_imports_for_vm` 对 visited 全部已装载模块做
  UseWeb 扫掠;④ adapter 别名配对修复（按 (adapter, symbols) 配对而非
  "任一 .at import",防符号误归属错误 adapter 限定名）。回归
  `test/ui/plan442_webcompat/` ×2（localStorage 往返+None 语义/
  encodeURIComponent JS 对齐）。
- B1 platform 域：`ports/platform.rust.at`（inject_styles 空实现/去化、
  setup_auth_fetch→rust fetch 注入、relay_command_runner rust 版）+ 构建双目标验证。
  （VM 轨：`ports/platform.vm.at`——dom/location 桩 + localStorage 已由
  本仓全局桥承接。）
- B2 composables 域：`ports/composables.rust.at`（useT→auto-i18n 直绑、gate_router
  rust 版）——依赖 musk 038 Phase 1 产物。
- B3 icons 域：`ports/icons.rust.at`（auto-icons 数据层直绑；渲染层依 A4 结论）。
- B4 renderer/upload 域：`ports/renderer.rust.at`（auto-down 008 产物）、
  `ports/upload.rust.at`（rust http 客户端版）。
- B5 musk `auto build` 双目标全绿（vue 产物对拍不变 + rust/vm 目标产物生成）。

### Phase C — 后端 AutoVM 激活（auto-musk 动作）

- C1 pac.at `api` 目标切换试验（rust→vm 路径），暴露的转译缺口登记回 auto-lang。
- C2 `musk serve` 以 VM 后端起服：HTTP/SSE 契约测试（复用既有 parity 测试面）
  对照 hw 后端全绿。
- C3 双后端并行观察期与切换/回滚开关（env 级），收口后 pac.at 头注的
  "待激活"改为已激活记录。

## 5. 验收标准

1. musk 前端 `auto build` 在 vue 与 rust/vm 双目标下全绿，五域端口各有非 web
   adapter（或显式降级登记）。
2. VM 渲染目标对 musk 30 widget 源零 Undefined variable 级报错（或每条有登记的
   能力缺口条目）。
3. `musk serve` VM 后端通过既有 HTTP/SSE 契约测试（与 hw 后端对照）。
4. 本计划完成 = musk PLAN-041 解挂条件达成（041 启动记录回填）。

## 6. 待澄清事项

1. **VM vs Rust 目标优先序**：B 阶段 adapter 先落 `rust.at`（a2r 路线成熟）还是
   直接 VM（AAVM 路线）——依 429-434 完成时的成熟度定，两个都做则排序待定。
2. **A4/A5 能力项归属**：若 auto-lang 决定不做 svg/调度原语，musk/auto-down 侧的
   降级路径为唯一路线——需要显式拍板而不是默认沉默。
3. **C 阶段 ag 轨休眠镜像**（KNOWN-DEBT 018：tools/spec_tools/orch_tools/
   server_serve）维持"不激活"结论还是借机激活——建议维持（收益为零结论仍成立）。
4. **观察期与回滚策略**：C3 的并行观察期长度与回滚开关形态。
