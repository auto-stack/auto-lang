# 原生验收通道（native acceptance channel）合同

> PLAN-781 阶段 A 交付。通道冻结合同全文见
> `docs/plans/reports/781-channel-decision.md`（含 §12 修订 r1-r3）；
> 本文为持久层规范。

## 目的与形态

为原生桌面 consumer（Reader 类）提供可信可见性 oracle：真实 split
VM+HTTP 实例上的**真实动作、当前帧裁剪几何、隔离库与冷启动可重放**
证据。不以 headless、预置终态或 fixture 注入替代。

## 启动与身份

- 启动形态：`auto run -r vm --no-merge -B <port>`（单进程 iced 前端 +
  VM HTTP 后端线程）。
- 身份三件套：pid ↔ 端口（netstat）+ stderr 监听行 + 内容级断言；
  每 run 建隔离数据目录（`reader001-review-*`），已有端口实例不能混用。
- 就绪 = 控制面 `/__auto/health/live` 200（legacy VM 轨无 `/health`
  app 路由）；就绪前不做任何断言。

## 观测工具（MCP，只读）

- `autoui_scroll_state`：pane 真实几何（viewport rect 含原点 / offset /
  content extent），键 = iced widget id（controller pane =
  `scroll_ctl_<name>`）；`frame` = 布局测量代数（仅新鲜采集后增长）。
- `autoui_visibility`：目标 rect 与 pane 链的有效 clip 的正面积交集；
  flags ∈ `unmeasured/zero_size/clipped_out/outside_window/no_pane`。
  坐标合同：采集 bounds 为 **content-space**（滚动后不变）；窗口化 =
  仅减 offset（决策报告 r4 勘定——不回加 viewport 原点）。

## 动作与证据纪律

- 动作发现自当前 VTree（`autoui_vtree`），经生产事件派发
  （`autoui_action press`）；ack 与随后状态变化需证据链。
- 几何与实例/渲染代际绑定：action 前后记录 freshness；未测量/帧未
  更新/节点失效/隐藏/零尺寸都不可判 visible。
- 恢复/观察阶段禁止驱动侧滚动（可在建立前提时滚动，两阶段隔离记录）。
- 真实冷启动 = 停止并 reap 本 run 拥有的进程（按验证过的命令行/port
  归属，不按进程名）→ 同库重启 → 新 pid。
- required 病例任何 FAIL/SKIP → 驱动非零退出；收尾 owned-port sweep
  （本 run 全部实例/重启端口不得仍被本 run pid 持有）。

## 验证面

- 驱动：`scripts/tools/test_native_reader_acceptance.py`（v07~v13 病例族
  + fixture `test/fixtures/native-reader-acceptance/`）。
