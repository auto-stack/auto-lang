# PLAN-708 T-00 基线报告（708-baseline.md）

- `stage: work` | `task: T-00` | `plan_revision: 2` | 状态：**进行中 → 运行探针补齐后冻结**
- 修订轮次：第 1 轮调查（§6 预算内；纠正复测轮未使用）

## 1. 环境与版本固定

| 项 | 值 |
|---|---|
| OS | Windows 11 专业版 10.0.26200 |
| CPU | 13th Gen Intel Core i5-13600KF |
| GPU | NVIDIA GeForce RTX 4060 Ti；**另注册 Todesk Virtual Display Adapter**（远程桌面虚拟显示——present/刷新计时可能经由虚拟适配器，见 §5 警示） |
| 分辨率/刷新率 | 2560x1600 @ 59Hz |
| 系统 DPI | 96（100%） |
| auto-lang 源码基线 | plan-708-dev @ `e1bab972eb7ac3fcda8cc7bd8793f2791f0a04c6`（=master，含 707 全部落地接口） |
| iced 锁定版本 | iced 0.14.0 / iced_winit 0.14.0 / iced_core 0.14.0 / iced_futures 0.14.0（crates.io，Cargo.lock 锁定）；本地补丁 `patches/iced_widget`（PLAN-043 布局补丁） |
| iced features | `unconditional-rendering` 已启用（auto-lang Cargo.toml:217） |
| gallery 语料 | auto-os `widgets-gallery` @ `93050a6a19231238cd5d4478909e9d6efd30be95`（= 计划钉定值，干净） |
| release 二进制 | `target/release/auto.exe`（构建于 worktree，SHA 见 §4 冻结值） |
| MCP | AUTOUI_MCP_PORT 动态分配；热重载关闭；F12/probe 按用例开关 |

## 2. 静态探针结论（源码证据，行号基于 e1bab972e）

### 2.1 引擎分段驱动（T-11 靶面）——`vm/engine.rs`

- `drive_handler_segment`（:2519）：步预算 `10_000_000`；**预算耗尽路径（:2691-2731）返回 `Completed(Ok(()))`——静默假成功**：task 停在函数中部（ip 指向函数内、bp≠saved_bp），无结果值，调用方无从分辨。这是"不可续跑的错误 Completed 语义"的实锤，也是 M-02 契约要替换的核心。
- ParkedWait 凭据集（:330-344）：`HttpRequest(u64)` / `Future(u32)` / `HttpStream(u64)`（707 T-05 新增）。非等待 Yield（SLEEP/SSE generator retry）在预算内 busy-continue（:2614-2616）——长 SLEEP 烧预算但仍同步占线程。
- 入口分野：`call_fn_by_name`（:2305，legacy busy-wait，非 UI 保留位）/ `call_fn_by_name_segment`（:2339，park-on-wait）/ `call_fn_by_name_segment`→`dispatch_fn_by_name`→`drive_handler_segment`。
- `resume_fn_by_name_segment`（:2361）：唤醒源 6 镜像 + 段续驱；重复 park→resume 链支持（chained await）。
- runaway 守卫：每 `RUNAWAY_CHECK_EVERY` 步核对字符串池/堆增量（:2529-2552），段独立基线。
- API 忙等预算报告（PLAN-026）：`api_budget_accumulate/api_budget_report`（:2637/:2711）。

### 2.2 UI 桥同步契约（T-12/M 靶面）——`ui/vm_bridge.rs`

- `call_vm_fn`（:1704）：**每次新开 `AutoTask::new(0, 4096, 0)` + legacy 同步驱动**（"保留位①"，:1766-1771）；结果 Value 契约 + RC stake 接管（:1777-1789）。视图 helper 求值即此——冷 miss 同步重算。
- `call_computed_fn`（:2105）：同型（"保留位②"，:2112-2116）。
- `call_handler_for`（:2127）：段派发（park-on-wait）；**parked 在途重入直接忽略**（:2153-2156）；Completed 后整栈清账（:2203）。
- `child_init_should_fire`（:1447）：身份表（组件名+key prop）**判定即写**——Init 派发决策在检查时消费身份，与真实完成无关。M-01 deferred Init 必须重排此语义（登记≠完成）。
- `run_module_init`（:1310）：段派发可 park（`__module_init`）。
- 702 泵：`has_parked_tasks`（:1524）/`resume_ready_parked`（:1544，串行于 iced update 上下文）/`parked_wait_ready`（:1587，凭据探测）/`register_parked`（:1606）+ `sync_busy_flag`（:1635，`__busy_handlers` 根态镜像）。

### 2.3 渲染构建与 memo（S-01/S-02/M-01 靶面）——`ui/aura_view_builder.rs`、`ui/dynamic.rs`

- `render_outlet_impl`（:5073）：memo 门现状 = `prop_memo || env AUTO_OUTLET_MEMO=="1"`（:5140-5141）——**二值 opt-in，无 env=0 强制关**；r2 三态表（env=0 覆盖 prop=true）是本计划改动。
- `render_outlet_page_memo`（:5182）：memo 键 = (ctx_state_obj, SITE_OUTLET_PAGE=6, skeleton_fp, probe_on)（:5257-5267）；命中条件 = seq 快路径 + globals 指纹（theme_epoch/menubar/popover/action_config ptr，:5270-5279）+ dyn_fp 慢路径。Degrade 臂走 `note_degraded`+全量（:5250-5255）。
- **同步 Init 派发在渲染路径内**：outlet 页身份变化帧 `render_outlet_page_memo` 内直接 `call_handler_for(Init)`（:5205-5219）；子件通用路径 `fire_child_init_if_any`（:6560-6600）同为渲染期同步派发（HandlerNotFound 静默）。M-01 demand 化的两个登记点即此。
- `child_init_identity`（:6603）：组件名 + 调用位 `key:` prop。
- `DynamicComponent::view`（dynamic.rs:2049）/`view_with_debug_gated`（:1553）：每帧全树重解释；AUTO_MEMO_DIAG 打点已存在（build_ms）。
- gallery 语料 memo 旗标实况：`app.at:183`（sidebar memo）、`:696`（sidebar memo）、`:701`（outlet memo）——与计划静态调查一致；r2 表中"已有开关只作基线核查"对应的对象即这三处。
- preview-card 代码生成税：aura_view_builder.rs:4414 在 VM 视图构建内调 `VueGenerator::gen_previewcard_code`（vue_gen/vue.rs:8783）——显示构建内的同步 codegen，T-02/T-12 热点候选。

### 2.4 帧屏障与交付顺序（Q-04 裁决材料）——iced_winit 0.14.0

- RedrawRequested 臂（iced_winit lib.rs :767-1034）：`interface.update`（布局）→ **`runtime.broadcast(Interaction{RedrawRequested})`（:972-978）→ `compositor.present`（:981）**。frames 广播在 present 之前——证实计划判断。
- frames 订阅消息经 runtime 通道异步回环：应用在**后续事件循环轮**才收到该消息 ⇒ 收到时该帧的 present 已同步返回（除非 present 失败重试）。可作"上一帧已交付"的应用层序障，**非硬在屏屏障**（无 GPU fence 回读；present Ok=swapchain 入队）。
- 公开 API 面：`iced_futures::event::listen()` **显式过滤 RedrawRequested**（iced_futures event.rs `listen_with` filter_map 第一臂返回 None）；收帧通知必须 `listen_raw()`（文档自警可能无限循环）。无公开 post-present 回执；硬屏障需 fork iced_winit——按计划约束不允许暗加。
- `unconditional-rendering` 语义（lib.rs :1165-1168）：AboutToWait 有 events/messages 即 `request_redraw(NextFrame)`；消息非空/UI 过期则 `program.update`+`build_user_interfaces`（**每批消息全量重建视图**，:1228）。空闲零事件则 `continue`（:1130-1132）不空转。
- **推论（加载期成本模型）**：帧通知订阅活跃期间，每条帧消息=一次完整 view 重建+layout+draw+present。骨架帧必须命中 memo（廉价重建）否则泵本身成为重建税放大器——M-03 的"仅唤醒一次/无任务不运行泵"与 S-01 memo 是同一枚硬币的两面。

### 2.5 会话/update 漏斗（M-03 接线面）——`ui/iced/renderer.rs`

- `run_session`（:15906）：Desktop 模式装配点（rqhost daemon、主题播种、panic hook）；iced 循环启动。
- `update_inner` 闭包（:16597）：`state.split_mut(app_id)` 缺 App 早退（:16604）；**大量提前 return 臂**：`__mcp_fixture`（:16619）、`OnEditorFocus`（:16649）、`OnColResize`（:16682）、`__mcp_resize_col`（:16714）、`__mcp_click`（:16736）……——M-03 完成通知/dirty 传播必须逐臂覆盖。
- 702 泵订阅：`__parked_resume_tick` 16ms 条件 tick（:17204-17209 消费臂；:21558-21561 订阅臂，`has_parked_tasks()` 门控）——**时间驱动轮询**，r2 的 CPU continuation 泵不沿用 tick 形态（帧通知驱动 + 有界预算），但注册表/串行纪律复用。

### 2.6 MCP 路径

- MCP `autoui_state`（mcp_server.rs:2256）读 `SharedStateHandle` 锁内快照；树快照消费渲染期发布的 `SendViewTemplate`（:465-474, :1305-1318）——MCP 热路径**不独立重建视图**（probe 门注释 :1547-1552 同证）。"MCP 旁路"主要成本在快照发布与锁竞争，T-00 运行探针以 MCP 开/关对照量化。

## 3. 运行探针结果（2026-09-30 实测，release 980304eb693f…，gallery 93050a6a）

驱动：autoui-verifier `AutoUiMcpClient` + 时间戳 stderr tap（tmp/708/，worktree gitignored）；AUTO_MEMO_DIAG=1。press 延迟 = MCP action 往返 ≈ UI 线程处理该输入的 update+render 轮时长（action 工具等待渲染轮完成，实测与 build_ms 一致）。

### 3.1 页级构建成本（单帧 build_ms，outlet 页构建）

| 页面 | preview-card 数 | 冷构建 | 复访 | memo 行为（site=6） |
|---|---|---|---|---|
| Home/Index | 0 | 1ms | 1ms/0ms | FILL→HIT fast ✓ |
| Row | 7 | 6.4–8.1s（跨启动方差） | **0ms** | FILL→HIT fast ✓ |
| DataTable | 9 | **9.5s**（9565/9573 两次实测） | **11.5s + 9.9s 双构建** | **零 site=6 行——静默 Degrade，memo 从未生效** |
| AreaChart | 1 | 1.2s | **1ms** | FILL→HIT fast/version-fast ✓ |
| FileTree | 1 | 1.4s | 1.2s×2 构建 | **每次 FILL，从不 HIT** |

### 3.2 病灶归因（判别实验，bench 语料 tmp/708/bench/）

| 变体 | 内容 | 构建 |
|---|---|---|
| cards7 | 7×preview-card（每卡 1 row 3 square） | **9406ms** |
| bare7 | 同内容无 preview-card 包装 | 0ms |
| cards1 | 1×preview-card | **1274ms** |
| sq70 | 70×裸 row（210 squares） | 0ms |
| cards7-prop | 7×preview-card + 显式 `auto:`/`vue:` prop（跳过代码生成字符串化） | **9021ms** |

**结论：preview-card 每实例 ~1.1–1.3s 恒定成本，与内容无关、与代码生成字符串化无关**——落在该臂独有的 `VueGenerator::new()`（aura_view_builder.rs:4413-4415）→ `WidgetRegistry::with_defaults()`（registry.rs:43：register_defaults 全量规格表 + apply_schema_vue_mappings 全 schema 折叠，schema 475 元素 × 数百 spec 每次重跑）。页级成本全部由此解释：row=7 卡、datatable=9 卡、area/filetree=1 卡。精确内部拆分（register_defaults vs schema fold vs resolve_tag）留 T-02 实施时钉定。

### 3.3 结构性观察

- **DataTable 静默 Degrade**：静态扫描不可证明 → `note_degraded`+全量路径无任何诊断输出（aura_view_builder.rs:5250-5255 无 eprintln）→ 每帧 9.5s 重建且不可观测。T-06 需补 Degrade 诊断面。
- **FileTree 恒 MISS**：每次访问 FILL（扁平化 computed 依赖/树态每帧变 seq），1.2s×2/次。
- **每次导航双构建**：FILL 行成对出现（路由写 + 页渲染两轮 dirty）；未命中页成本 ×2。
- **加载期输入突发**：巨构建期间 press 入队即时返回，UI 线程连续被占 10.8s→37.7s（11.5s+14.9s 两构建背靠背）≈**26.4s 连续占用**；输入延迟由巨构建支配。
- **env=0 无强制关语义**：`AUTO_OUTLET_MEMO=0` 下 prop `memo:true`（app.at:183/696/701）仍命中（OR 逻辑）——**现状无法对 gallery 关 memo**，r2 三态表（S-01）正是缺失面（A/B 对照改为由 bench 语料 bare7 承担：同内容无 memo 包装 0ms）。
- 启动：MCP ready ≈1.7s；boot 首帧 IndexPage 1ms。编译/装载耗时未计入（本次无病感；T-09 单列）。

## 4. 冻结值

- 二进制：`target/release/auto.exe` SHA256 `980304eb693fe769725c1f1bfea530881775ba2f88728a71a870ef4a9c17cebd`（plan-708-dev @ e1bab972e，release）。
- 基线热点排序（优化对象）：① preview-card 臂 VueGenerator 重建（页构建成本 ~100%，S-02/M 缓存对象）；② DataTable 静默 Degrade（S-01 失效/诊断 + T-12）；③ FileTree 恒 MISS（T-12 computed 缓存）；④ 双构建乘数（M-03 通知合并时核查）。
- 最大连续 UI 占用基线：单轮 9.5–11.5s；突发输入窗口 26.4s（§7 门禁 max≤50ms 的对照基线）。
- native 单调用：gallery 加载路径无 FFI 主导段（构建成本为解释器+生成器，trace 无 native 慢调用迹象）；重 native 单调用上界验证移交 T-11 夹具（plan708 测试族），不作为 gallery 基线项。

## 5. 警示与限制

- **Todesk 虚拟显示适配器**在本机 GPU 枚举中注册；若被测窗口落在其管理的显示路径上，present 计时含远程编码开销，p95 可能整体抬升。对策：窗口固定在物理显示器（RTX 4060 Ti 主输出）；如 p95 异常，另采 `--windowed` 移窗对照样本，并在报告中标注采样所在适配器。
- 59Hz 刷新率：vsync 节拍 ~16.9ms；16.7ms 预算在该屏上等于 1 vsync 槽，骨架延迟按帧取整解读。
- 冷构建存在跨启动方差（row 6.4–8.1s，±12%）；T-09 终验按 §7 ≥20 样本报 p50/p95/max。
- MCP press 延迟 ≈ update+render 轮时长（action 等待渲染轮）；不能当作"输入→可见"上屏证明（在屏证据用截图，AC-06）。
- 本报告静态部分为源码证据（e1bab972e）；运行数据已冻结如上，构成 T-00 归因证据；AC-06/09 的验收仍以 T-09 的 §7 完整矩阵为准。
