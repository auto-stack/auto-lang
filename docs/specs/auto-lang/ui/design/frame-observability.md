# 帧观测通道（frame observability）

> SD-B（PLAN-716 组B 落账真源——M4 供料包供②「内核帧时间戳插桩」
> 承接）。before=.at 层无帧观测通道（PLAN-005 T-03 勘定）；after=本文
> 通道契约。基=auto-lang plan-716-dev。

## 1. 通道面

- 模块 `ui::frame_bench`（crates/auto-lang/src/ui/frame_bench.rs）。
- 两个只读单调时间戳（毫秒 i64，进程起点起算）：
  - **帧开始** `begin_ms`：桌面 update 入口到达时刻——锚点=
    `update_inner` 闭包入口（iced renderer.rs，VM 桌面路径全消息流经点）。
  - **呈现完成** `present_ms`：`__frame_pump` 消息消费时刻——711 帧通知
    泵（`listen_raw` 过滤 RedrawRequested）的 D-1/R-1 序障实证：**消费
    时刻 ≥ 该帧 present 同步返回**——present 的合法代理。硬 post-present
    屏障需 fork iced_winit（计划约束禁止，711 勘定在案）。
- 首帧段分解（steady_start）：spawn→首帧 begin→首帧 present 由读侧
  差分（present/begin 首个非零值）；type_latency=键入消息 begin→下一
  present 差；scroll_fps=相邻 present 差倒数。

## 2. 门控零开销纪律

- **`AUTO_FRAME_BENCH=1`** 门（AUTO_BENCH 族，PLAN-005 先例；711
  AUTO_SCHED_DIAG 同域先例）：OnceLock 读一次定格；门关=捕获路径
  一次布尔读+分支（实测 ns 级，60fps@1000msg/帧占比 <0.01%）——
  **未订阅/未标记零开销**（供②验收形）。
- 门开=两 AtomicI64 Relaxed store/帧锚点；读侧无条件可读
  （未捕获=0 语义：门关或帧未发生）。

## 3. .at 可达双轨（同源）

- **VM 内建**：9918 `auto.frame.begin_ms` / 9919
  `auto.frame.present_ms`（→I64；catalog+RET+NATIVE_ID_ENTRIES 钉扎
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

## 4. 边界注记

- **712 r2 帧泵域边界**：本通道只读观测（门控 store），不改调度
  语义（dirty/epoch/poll_frame_pump 泵臂零触碰）；712 r2 已
  delivered+archived（2026-09-30，451dc1401+ff32d7004），残余
  T-16..T-19 为桌面轨调查（无 renderer 域代码 WIP）。
- 多窗 runner 第二泵臂（renderer.rs:19871 __frame_pump 推送点）未
  插桩——多窗会话帧观测为待办注记（单窗主路径已覆盖；per-window
  路由精化归 711 T-06余 观测面复核）。
- 值语义：0=未捕获（门关/帧未发生），非 0 后单调递增；跨帧间隔
  直读可换算（ms）。
