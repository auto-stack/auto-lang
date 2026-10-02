# PLAN-735 T-01/T-02 呈现/通知分离谱 + 交付节奏根因报告（2026-10-03）

> 数据源：`docs/plans/evidence/735/ladder-p735-base.jsonl`（改前基线谱，
> ladder735.py 可复跑）+ 临时 app.log 逐事件时间轴
> （analyze_base.py / 内联分析，协议同 731 T-05）。
> 二进制=本 worktree release @ 5da3b4727（插桩件：draw_end_ms +
> redraw_deliver + update trace——AUTO_FRAME_BENCH/AUTO_SCHED_DIAG 双门）。
> **钉版注记（T-00 钉版勘定）**：基线谱跑在 iced_winit **0.14.1**（worktree
> 无锁文件新解析；下游 auto-edit 同为 0.14.1——现象面同版）。731 带谱
> （0.41-2.47 pump/s）测于主检出锁 iced_winit **0.14.0**；两版差异=
> lib.rs ControlFlow 陈旧 WaitUntil 唤醒重调度（0.14.1 加 `current >
> Instant::now()` 守卫）——改前/改后双谱与带对照同版（0.14.1）内做，
> 跨版带对照注记于此。winit 0.30.13 / wgpu 27.0.1 / iced 0.14.0 两版一致；
> patches/iced_widget 本地补丁在位（PLAN-043，升级重审面）。

## 1. T-00 试验床复现（AC-01）

| 档.相位 | pump/s（本基线 0.14.1） | 731 release 带（0.14.0） |
|---|---|---|
| 5kb.type / 100kb.type / 1mb.type | 2.45 / 2.70 / 2.98 | 2.47 / — / 1.46 |
| 5kb.scroll / 100kb.scroll / 1mb.scroll | 0.55 / 0.42 / 0.68 | — / — / 0.41 |
| 5kb.typenl / 100kb.typenl / 1mb.typenl | 2.88 / 2.76 / 3.16 | — / — / 1.69-2.14 |

**带内成立**（0.4-3.2 ⊇ 731 带 0.4-2.5，本基线略高=0.14.1 唤醒修复+机时波动；
同版内改前/改后对照不受影响）。711 泵计数器四件
（queued_init/cpu_cont/parked_total/write_q）在 frame_pump enter 行全数采样
（全零——本例稳态键入无 Init demand/CPU continuation，见 §3 轴④）。
712 r2 残余零冲突：in-flight 五分支（719/722/723/730/732）对泵域五文件
（renderer.rs/frame_bench/frame_segments/sched_diag/dynamic）零提交（git log
master..branch 实勘）；frame-observability §4 边界注记与实况一致。

## 2. T-01 呈现 vs 通知分离谱（AC-02——决策件）

通道：`draw_end_ms`=根包装探针 draw() 括号结束的绝对墙钟（iced_winit
RedrawRequested 臂内，同步先行于 compositor present——**逐帧真实呈现**
代理）；`present>0` 行=泵通知消费配对（现行为 9918/9919 同语义）。

1mb 档相位谱（analyze_base.py，事件级）：

| 相位 | draws | 送达(deliv) | 驱动更新 | 行配对率 | draw→draw gap P50/P95 |
|---|---|---|---|---|---|
| type（30 键） | 33 | 33 | 32 | **3/33 = 9%** | 95ms / 300ms |
| scroll（25 步） | 19 | 18 | 26 | 5/19 = 26% | 3ms（突发）/ 17ms |
| typenl（30 键） | 30 | 30 | 32 | 3/30 = 10% | **107ms** / 114ms |

**定性结论（G-2 决策输出）**：

- **呈现真快**——present 完成随输入更新 +6-10ms 到达（SRC update →
  iced AboutToWait 无条件 request_redraw → 下一轮 RedrawRequested 臂
  同步 draw+present）。滚动突发期 deliver→deliver 2-3ms（循环能力
  数百 fps）；~95-110ms 的节拍=**驱动器的节拍**（ladder 60ms sleep
  +每调用 ~2-20ms MCP 时延 → 实测驱动周期 p50 62.5ms/最长 86ms，
  probe_type_period.py 逐调用计时在档），非渲染/通知天花板。
- **通知面双层伪影**——①行配对率 9-26%（与下游 13% 同现象同机理，
  §3 轴③）；②下游 8.1-9.5fps 实为其**驱动节奏的读数**（autoui 协议
  同源），非呈现能力。
- 731 §2 归因注记「呈现随键入节奏发生」的墙钟对照法在本件逐帧
  draw_end 通道下**修正**：呈现随**驱动节奏**发生（键入驱动≈键入
  节拍，滚动驱动≠键入节拍——同现象，机理=驱动器节拍传递）。

## 3. T-02 交付节奏四轴根因（AC-03）

**轴① RedrawRequested 发放/送达**：无门控旁路观测订阅
（redraw_diag_sub，恒 None 零消息面）——送达与更新同拍（每更新轮
1-2 次送达，+6-9ms）；711 D-2 门控的 frame_pump_sub **全程零装配**
（subF=0）零到达（enq=0）——稳态键入无 Init/CPU 工作，订阅门关。

**轴② winit/iced 重绘调度**：unconditional-rendering（Cargo features
在钉）使每个 AboutToWait 处理轮 request_redraw(NextFrame)=立即；
程序更新轮对全窗 request_redraw。送达粒度=轮询粒度——**无合并丢帧**
（draws=delivers 逐相等）。RedrawRequested 臂内序=draw → broadcast →
present（iced_winit lib.rs:924-940 实勘）。

**轴③ 泵消费滞后/配对（主根因）**：稳态通知的真实通道不是订阅，
是 **ready 唤醒链**（renderer.rs dispatch_app 尾——凡 update 致脏即
chain 一个 `Task::perform(ready)→__frame_pump`，下一轮到达消费）。
错位机理：链式泵消息在**同批或次轮**到达消费（先于本帧重建落 prev），
prev 空转→无配对行；紧随的 `__bounds_collected`（fit 测量链，每键入
一条）的 frame_begin 旋转把**已建完+已呈现的帧**以 `present=-1` 孤儿
落账，bounds 帧反而窃得配对——**87% 建帧的配对损耗=发布点错位的
测量通道伪影，非通知丢失、非呈现丢失**。

**轴④ 消息队列竞争**：泵计数器全零（无排队 demand/CPU/parked）；
write_q 零。MCP 订阅同队列非瓶颈（调用 p50 1.5-5ms；17-26ms 长尾
≈1/3 调用=shared 锁与重建竞争，次要轴）。

**修复方案选定（多臂并存）**：

- **臂 (a) 配对修复（frame_segments 发布语义升格）**：旋转发布点改
  truthful——`emit(stale, draw_end_ms>0 ? draw_end_ms : -1)`；泵发布点
  同规则（draw_end 优先，无 draw 回退消费时刻）。效果：已绘制帧
  100% 拿到真 present 行（13%→~100%）；`present` 列语义升格=真实
  呈现完成时刻（R-1 序障保持：draw 在消费轮之前，序仍成立）。
- **臂 (b) 呈现真相通道产品化**：frame_bench 增 DRAW_END 原子读数
  （draw 末点写入，门控零开销）+`frame_draw_end_ms()` 读侧；VM 9920
  内建/a2r 扩展**不在本件**（下游判定零触碰——回执列建议）。
- **不修面（证据驱动）**：不 fork iced_winit（无必要——无丢失通知
  可修）；不动 711 D-2 订阅门（其防自续帧职能不变）；不动驱动器
  协议（下游 022 frozen 域）。

## 4. 下游 8-9.5fps 定性（回执要点）

下游 frozen 口径的「scroll 25-30 distinct/3s=8.1-9.5fps FAIL」= 三层
叠加：①驱动节奏传递（其驱动周期 ~105-120ms/动作）；②行配对伪影
（§3 轴③——即便逐帧呈现，行通道也只报 9-26%）；③判定读 present
列（通知配对面）而非呈现真值。**修复后行通道即逐帧真值**——下游
重判可直接消费新行通道/回执结论，无需改其驱动协议。
