# PLAN-735 下游回执 v2（帧两行重判解锁——呈现事实定谳+交付节奏终态）

> 承 731 `downstream-handoff.md`（669 模式第二段：上游出谱与回执，判定
> 归下游重判件——auto-edit 022 frozen 判定口径零触碰）。消费方=auto-edit
> 帧两行重判收口件（PLAN-026 delivered@252028a 归档件预告的位点）。
> 上游代码面=auto-lang plan-735-dev（5da3b4727 插桩+7b9b2f8f5 配对修复）。

## 1. 呈现事实定谳（026 回执要点①的答案）

- **下游 8.1-9.5fps 是驱动节奏读数+通道伪影，不是呈现能力上限。**
  上游逐帧 draw_end 通道（真实呈现时戳）实证：
  - 呈现随输入更新 +6-10ms（更新轮 AboutToWait 无条件 request_redraw
    → 下一轮 RedrawRequested 臂同步 draw+present——unconditional-
    rendering 钉版语义）；滚动突发期送达间 2-3ms（事件循环能力
    数百 fps）。
  - 「~105-120ms 恒定节拍」=MCP 驱动调用周期（60ms sleep+调用时延
    p50 62.5ms+长尾 ~20ms@1/3 调用≈95-107ms 有效周期）的**传递
    读数**。驱动器停则呈现停；帧率类判定必须连同驱动协议周期归因。
  - 731「present=-1 孤儿=测量配对面」定性正确但机理细化：配对损耗
    根因=**发布点错位**（稳态通知走 ready 唤醒链每致脏 update 一条，
    `__bounds_collected` 更新旋转把已建已呈现帧以 -1 落账、bounds 帧
    窃得配对）——**非通知丢失、非呈现丢失、无需 fork iced_winit**。
- 分离谱：改前行配对率 9-33%（与下游 13% 同现象同机理）；~100ms
  节拍在 scroll 相位（驱动周期 ~265ms+Tick 批化）同样传递。

## 2. 上游改后终态（重判依据面）

| 面 | 改前 | 改后 |
|---|---|---|
| 行配对率（键入/滚动/typenl ×三档） | 9-33% | **100%（184/184）** |
| `present` 列语义 | 泵消费配对（错位） | **真实呈现完成时刻**（draw_end 优先） |
| `present=-1` | 配对面伪影（87% 建帧） | 真未呈现帧（本谱零出现） |
| R-1 序障 | 消费 ≥ present | **保持**（404 行 0 违例） |
| 呈现真相通道 | 无 | 行列 `draw_end_ms` + `frame_bench::frame_draw_end_ms()` |
| S5/segsum 谱 | 731 带 | 同带（零回退）；golden 三形 0.00% |

- 双谱在档：`ladder-p735-{base,after}.jsonl`；复跑=
  `python docs/plans/evidence/735/ladder735.py --label <x> --out <p>`。
- 钉版注记：谱系 iced_winit 0.14.1（下游同版）；731 带谱测于 0.14.0
  （两版 ControlFlow 陈旧唤醒差异——0.14.1 加 `current > Instant::now()`
  守卫；跨版带对照在 T-00 报告注记）。

## 3. 通道语义建议（重判件读回适配——非本件域）

1. **帧率/时延类判定读呈现面**：行通道 `draw_end_ms` 列（逐帧）或
   `frame_draw_end_ms()`（末值）；`present_ms`/行 `present` 列改后
   已对齐真值（draw_end 优先），但**驱动周期仍需并入归因**——
   「distinct present ≥ 面板率×0.9」判据在驱动周期 > 1/面板率时
   物理不可达（驱动器本身是帧率上限）。
2. **VM 9920 内建（auto.frame.draw_end_ms）**：建议下游重判件立项
   接线（同 9918/9919 模式——stdlib/auto/frame.vm.at+codegen 路由
   +白名单+a2r_std::frame 同源），上游已备 Rust 读侧单源。
3. **驱动协议建议**：若重判需测「应用能力上限」，驱动节拍应小于
   被测率（如 16ms 节拍驱动测 60Hz 面板）——现 60ms 节拍+调用时延
   只能测到 ~9-16Hz 有效驱动率。

## 4. 重判件预告（消费清单）

1. 下游 bench 帧档复跑（022 协议原样），判定读数切换呈现面通道
   （§3.1）后重驱动帧两行；「scroll 25-30 distinct/3s」读数预期
   随行通道修复自动对齐（无下游代码改动场景）+驱动周期归因注记。
2. type_latency=帧内 P95 ≤1 帧：行 `total_ms`（present-begin，现在
   =draw_end-begin 真实帧时）P95 读数；上游 1mb type total P50
   ~0.01-0.11ms（余量 ~8×）。
3. 若下游仍有独立于驱动的帧率面（自驱动画/动画计时器），走
   draw_end 谱与 sched_diag 送达轴（AUTO_SCHED_DIAG=1——新增
   `redraw_deliver`/`update`/`frame_msg enqueue` 三行同 t0 对读）。
4. anchors --verify+render --check 收口（024 同款）。

## 5. 判定口径（frozen——零重申变更）

- type_latency / scroll_fps 定义原样；本件零下游判定面改动
  （auto-edit 仓 porcelain 零改动——其证据为输入）。
