# 帧观测通道（frame observability）

> SD-B（PLAN-716 组B 落账真源——M4 供料包供②「内核帧时间戳插桩」
> 承接）。before=.at 层无帧观测通道（PLAN-005 T-03 勘定）；after=本文
> 通道契约。基=auto-lang plan-716-dev。PLAN-735 升格：呈现/通知分离
> 语义+呈现真相通道（§1/§4/§5）。

## 1. 通道面

- 模块 `ui::frame_bench`（crates/auto-lang/src/ui/frame_bench.rs）。
- 两个只读单调时间戳（毫秒 i64，进程起点起算）：
  - **帧开始** `begin_ms`：桌面 update 入口到达时刻——锚点=
    `update_inner` 闭包入口（iced renderer.rs，VM 桌面路径全消息流经点）。
  - **呈现完成（通知面）** `present_ms`：`__frame_pump` 消息消费时刻
    ——通知消费时戳（R-1 序障实证：**消费时刻 ≥ 该帧 present 同步
    返回**）。735 定谳（§5 分离谱）：稳态通知面走 ready 唤醒链
    （每致脏 update 链一条 `__frame_pump`，711 D-2 订阅门控仅在
    Init/CPU 工作窗口在册）——本字段语义=**通知消费时刻**，非呈现
    真值；硬 post-present 屏障需 fork iced_winit（计划约束禁止，
    711 勘定在案）。
  - **呈现完成（呈现面真值）** `draw_end_ms`（PLAN-735）：根包装探针
    （`ui::iced::frame_probe`）draw() 括号结束的绝对墙钟——iced_winit
    RedrawRequested 臂内同步先行于 compositor present（同调用栈，
    µs 级间隙），**逐帧真实呈现节奏**的直接代理。Rust 读侧
    `frame_draw_end_ms()`；逐帧行通道见 §5 `[P725-FRAME]`
    `draw_end_ms` 列。VM 9920 内建/a2r 扩展=下游重判件建议项
    （本件下游判定零触碰不接线）。
- 首帧段分解（steady_start）：spawn→首帧 begin→首帧 present 由读侧
  差分（present/begin 首个非零值）；type_latency=键入消息 begin→下一
  present 差；scroll_fps=相邻 present 差倒数。**735 注记**：帧率/时延
  类判定的呈现面读数应取 `draw_end_ms` 族（§5 分离谱——`present_ms`
  为通知面，受配对/驱动节奏双层伪影，见 §4 交付节奏契约行）。
- **交付节奏三轴（PLAN-740 扩轴——AUTO_SCHED_DIAG 门控 stderr 行，
  与上列时间戳同一 `sched_diag_t0` 基准）**：
  - **消费/到达轴** `mcp_recv t=<ms> got=1`：MCP action push 通道
    （PLAN-740 T-02 起，转发线程）逐动作打点=逐动作消费时刻。
    改前形态 `mcp_poll t=<ms> gap=<ms> got=<0|1>`（16ms tick 真实
    触发节奏+本拍消费标记——740 T-00 勘定轴；push 重排后退役，
    谱解析双形态兼容，gap=-1 语义=无节拍轴）。
  - **发出代理轴** `update_end t=<ms> app=<id>`：`update_inner`
    返回时刻——iced_winit AboutToWait 轮内逐消息 update 后对全窗
    无条件 request_redraw（735 轴②实勘），本点即每消息的发出节拍。
  - **到达轴** `redraw_deliver t=<ms> app=<id>`（735 既有）：旁路
    观测订阅（恒 None 零消息面，仅门开时装配）。
  四轴 + `[P725-FRAME] draw_end_ms`（呈现真值）同 t0 对读=交付
  节奏归因链（消费间隔结构 → 发出节拍 → 到达 → 呈现交付）。

## 2. 门控零开销纪律

- **`AUTO_FRAME_BENCH=1`** 门（AUTO_BENCH 族，PLAN-005 先例；711
  AUTO_SCHED_DIAG 同域先例）：OnceLock 读一次定格；门关=捕获路径
  一次布尔读+分支（实测 ns 级，60fps@1000msg/帧占比 <0.01%）——
  **未订阅/未标记零开销**（供②验收形）。
- 门开=两 AtomicI64 Relaxed store/帧锚点；读侧无条件可读
  （未捕获=0 语义：门关或帧未发生）。

## 3. .at 可达双轨（同源）

- **VM 内建**：9918 `auto.frame.begin_ms` / 9919
  `auto.frame.present_ms`（→**int**；catalog+RET+NATIVE_ID_ENTRIES 钉扎
  ——`register_vm_declarations` 按 NATIVE_ID_MAP 取固定 id）。
  编译器面=stdlib/auto/frame.vm.at（#[vm] 声明）+codegen
  (module,method) 路由对+内置模块名白名单 `frame`（勘定：缺白名单=
  「Undefined variable: frame」）；.at 调用形=两段名 `frame.begin_ms()`
  （canonical 归一 auto.frame.begin_ms）。
- **a2r 臂**：trans/rust.rs 裸名映射（模块 fn 体）+
  ui_gen/rust.rs handler 直调臂（实现体单源 `ui::frame_bench`，
  710 code_editor_delta 直调纪律同款）+`a2r_std::frame::{begin_ms,
  present_ms}`（与 VM shim 同源读——对拍测试锚）。
- 探针族 `plan716_supply_probes`：VM 读回+a2r 臂 grep 锚+a2r_std
  同源对拍+门开序（note begin→present 单调）+门关零捕获+开销两态
  微基准。实机帧序 live-fire 归下游 bench 档（供②验收分工原文）。

## 3b. 出口形与值流契约（PLAN-716 Phase 2 供⑨-a 修订）

> 下游 auto-edit PLAN-022 消费首跑实录三断（.at 侧赋值落 0/`.str()`
> 出 None/json 出 0——供料档 §8 供⑨）触发的出口形定形。勘定+
> 修复=plan-716-dev 5fe510539。

- **出口 lane=int（i32 单槽）**：frame.vm.at 判型 `-> int`（原
  `-> i64`——I64 声明驱动编译器 2 槽 Uint 消费路径，与 shim 单槽
  nv 推栈错配，是三断的类型面根因）；shim 出口 `push_i32`
  （原 `push_i64_vm` 推 TAG_I64 nv）。int 承接/`.str()`/复合
  obj 字段读三面保真由探针钉住（`frame_value_flow_gate_on`
  门开真值+`frame_begin_ms_str_fingerprint` 门态无关）。
- **值域边界（i32 封顶 ≈24.8 天进程存活）**：帧时间戳=进程单调
  毫秒，常态 <2^31；越界 saturating 截断（`min(i32::MAX)`）——
  超长存活会话的帧观测需求出现时另档（届时出口形需重勘，
  i64 变量承接形已在案）。
- **时源坐标注记（勘定实录）**：`process_start` 定格于模块首次
  触及（非进程起点）——首个 note 恒存 ~0；spawn→首帧段分解的
  消费方须在同坐标系内自行标记 spawn 时刻（如进程入口先读一次
  `frame_begin_ms()` 定格坐标系）。SD-B 首版「进程起点起算」
  措辞按实测修正。
- **time 族同根因面（PLAN-005 登记的现势复核）**：`var int =
  time.now_ms()` 落 32 位伪影（epoch ms 1790847058147 →
  4140720794 实测——「返 0」形的真身=截断 Hazard）。time 族
  **不改**（epoch 值域 >2^31，改判型必截真值）；宽值消费契约=
  **i64 变量承接或 `.str()` 全宽出口**（701 全宽值字符串出口
  先例的值域边界——`time_family_vm_readback_recheck` 探针
  as-is 形钉住）；now_sec 值域 <2^31，int 承接正确。
- **下游回执位（随下游件排程，非上游 gate）**：front 探针臂
  重埋（handler 体 `frame.begin_ms()` regen exit 0——供⑨-b
  补臂后解封，12e49f78c）+T-03 帧两行判定面（type_latency=
  帧内 P95≤1 帧/scroll_fps=distinct present 计数≥面板率×0.9）
  重驱动。

## 4. 边界注记

- **712 r2 帧泵域边界**：本通道只读观测（门控 store），不改调度
  语义（dirty/epoch/poll_frame_pump 泵臂零触碰）；712 r2 已
  delivered+archived（2026-09-30，451dc1401+ff32d7004），残余
  T-16..T-19 为桌面轨调查（无 renderer 域代码 WIP）。
- **交付节奏契约（PLAN-735 升格）**：呈现节奏=**驱动器节奏传递**
  （每输入更新一轮 AboutToWait 无条件 request_redraw（unconditional-
  rendering 钉版）→ 下一轮 RedrawRequested 臂同步 draw+present，
  更新→呈现 +6-10ms 实测）；驱动停摆则呈现停摆，呈现速率上限=
  事件循环轮速率（滚动突发实测 2-3ms/轮，数百 fps 能力）。~100ms
  级「恒定节拍」观测=驱动器调用周期+服务端锁竞争长尾的传递读数，
  **非渲染/通知天花板**——帧率类判定必须连同驱动协议周期一起归因
  （evidence/735/T-01-T-02-separation-and-rootcause.md §2/§3）。
  通知面（`present_ms`+行配对）735 起配对修复（§5），`draw_end_ms`
  为呈现面真值；两通道读数分离对读，单通道不作呈现事实断言。
- **呈现真相通道边界**：`draw_end_ms` 单窗精确（多窗会话=最后结束
  窗口值，多窗帧观测仍为待办注记——renderer.rs 第二泵臂同域）；
  `frame_probe` 覆盖 dynamic_view_impl 主路径，`view_named(face)`
  早退路径（dashboard face）不包装（731 R1 F-5 注记沿用）。
- 多窗 runner 第二泵臂（renderer.rs:19871 __frame_pump 推送点）未
  插桩——多窗会话帧观测为待办注记（单窗主路径已覆盖；per-window
  路由精化归 711 T-06余 观测面复核）。
- **交付节奏观测边界（PLAN-740）**：三轴均为 stderr 门控行（门关
  =零输出零写入零订阅装配）；`mcp_recv` 在转发线程（非主线程）
  打点，与 update_end/主线程轴对读容许亚毫秒级跨线程钟差；push
  重排后消费无固定节拍——间隔语义由 `mcp_recv` 逐动作时刻差承接
  （**间隔结构**才是定责杠杆，740 T-01：got 间隔亚槽宽聚簇=呈现
  槽合并的配对燃料）。三轴不进任何判定阈值语义（frozen——下游
  54fps 判定面零触碰），仅归因对读。
- 值语义：0=未捕获（门关/帧未发生），非 0 后单调递增；跨帧间隔
  直读可换算（ms）。

## 5. 上游阶梯谱基准（PLAN-725 T-05——通道的上游消费面）

> 下游 live-fire（bench 档两行判定）归下游件（§3 分工原文）；本节=
> 上游对偶面——infra 效果度量（编辑路径键入→present 增量重建谱）。

- **分段探针**：`ui::frame_segments`（与本文通道**同一 AUTO_FRAME_BENCH
  门**——单一门语义不引第二门）：`[P725-FRAME]` 行随帧吐出五段
  （S1 载荷/S2 VM/S3a MCP 同步/S3b 主重建/S4 Element）+builds+dirty；
  prev/cur 双槽配对（泵序 begin→present）。**PLAN-735 配对修复（§5
  配对面口径升格）**：发布点 present 列=真实呈现完成时刻
  （`draw_end_ms` 优先）；`present=-1` 仅剩真未呈现帧（建后零 draw
  ——例：最小化/零尺寸窗早退）。731 的「-1 孤儿=测量配对面」伪影
  已修（发布点错位——建帧 87% 配对损耗根因，§4 交付节奏契约行）。
  门关零分支零写入（§2 纪律同款）。
- **S5 子段插桩扩展（PLAN-731）**：`[P725-FRAME]` 行增
  `s5_layout_us/s5_shaping_us/s5_draw_us` 三时间戳——**残差口径
  （total−segsum）退役**，S5 升分段口径（S5a layout=根包装探针
  `ui::iced::frame_probe` layout() 括号；S5b shaping=编辑器核心
  render 内 cosmic-text 整形括号；S5c draw=根包装探针 draw() 括号）；
  present 侧 wgpu flush 以 gpu_residual 单列（配对帧）。门控纪律同款
  （门关不取 Instant——`.then(Instant::now)` 形）。**覆盖边界（R1 F-5
  注记）**：探针包装 dynamic_view_impl 主路径（含防御快道）；
  `view_named(face)` 早退路径（dashboard face 面）未包装——face 会话
  的 S5 观测为待办注记（帧档 bench/ladder 主路径不受影响）。
- **谱面**：文档尺寸阶梯 5KB/100KB/1MB × VM 轨 × MCP autoui_type 驱动
  （单字符 30 键 60ms 节拍——下游 stage_frame 协议同源）；判据
  **segsum（S1..S4 和）P50/P95**（present 配对受 bounds 回路竞争不稳定
  ——total 口径弱化注记；S5=残差）。
- **脚本与双态谱**：`docs/plans/evidence/725/ladder.py` +
  `ladder-{baseline,after}.jsonl`（同负载同脚本可复跑）。改后判据门：
  5KB segsum P95 ≤16.7ms@60Hz；1MB/5KB P50 比 ≤1.6（尺寸缩放清零）。
- **基准 app**：`examples/ui/041-auto-edit`（现役编辑器例复用——Q-5
  默认形）；文件装载走 MCP fixture 通道（AUTOUI_TEST_FIXTURES=1 写
  auto_open_path 状态+触发 App.Tick→ConsumeOpen——本例无 env 直读臂，
  fixture 是零改例等价驱动）。
