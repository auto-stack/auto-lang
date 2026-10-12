# PLAN-740 T-00/T-01 交付节奏三轴定责报告（2026-10-08）

> 数据源：`ladder-p740-pre-rungs-5kb.jsonl`（HTTP 驱动四档）、
> `ladder-p740-pre-probe.jsonl`（内部通道探针 34/54Hz，5kb）、
> `ladder-p740-pre-probe-1mb.jsonl`（探针 34/54Hz，1mb 判别跑）、
> `ladder-p740-pre-735proto.jsonl`（735 协议改前对照）。
> 二进制=本 worktree release @ 插桩件（T-00 三轴打点：
> mcp_poll/update_end 新轴 + redraw_deliver/draw_end 既有轴）。
> 跟随判据=follow_ratio_counts（对齐窗内 distinct draw_end 交付数 /
> 驱动数；AC-02 ±10% 带材料）。

## 0. 结论速览（定谳）

- **消费轴与 tick 轴全档健康**：16ms tokio tick 真实触发节奏
  p50=16ms（全档 10-54Hz 无 90ms 聚簇）；单条 try_recv 消费
  got==驱动数（零丢失零积压）。
- **~93ms 恒定 pacing 在上游 041 不复现**（027 下游证据链形态——
  两跑一致的 91-95ms——在本机 041 全档未出现；scroll@34 探针档
  单次 321ms 离群除外）。
- **交付轴在 ≥34Hz 档跟随退化**（探针精确速率驱动下）：34Hz/1mb
  =1.02 ✓；54Hz/1mb=**0.79**（<0.9 带）；5kb 档因钳制回声放大
  更低（0.73-0.77——协议面伪影，见 §3）。
- **责任机制（候选逐项判定见 §2）**：16ms tick 消费网格把 54Hz
  原生间隔（18.5ms）重排为亚 16ms 消息对（配对燃料：got 间隔
  8ms bin×49/127），消息对落入 16.7ms 呈现槽/批处理窗 → 单帧
  合并 → 交付 38.9/s（0.79×驱动）。交付间隔谱=严格 16/32ms
  双步阶梯（直证 tick 网格+槽合并）。
- **修复面（T-02，计划 (c) 分支预授权的 Poll 面最小绕行）**：
  MCP action 消费 16ms 轮询 → **push 通道**（到达即唤醒投递），
  内含 drain-to-empty 语义；原生间隔恢复后无配对 → 跟随回带。

## 1. 阶梯跟随性谱（改前）

### 1.1 探针档（内部通道——精确速率，下游 in-process 驱动形态同构）

| 档.相位 | 驱动 | follow | de_gap p50/p95 | got | 备注 |
|---|---|---|---|---|---|
| 5kb.scroll@34 | 34.0Hz | 0.77 | 31/35ms（max 321） | 102/102 | 钳制回声放大 |
| 5kb.scroll@54 | 54.0Hz | 0.73 | 16/33ms | 162/162 | 同上 |
| **1mb.scroll@34** | 34.0Hz | **1.02** ✓ | 19/40ms（max 55） | 102/102 | 界内滚动（判别跑） |
| **1mb.scroll@54** | 54.0Hz | **0.79** ✗ | 16/32ms | 162/162 | 残余=槽合并 |
| type@34 | 33.5Hz | 1.37 | 16/28ms | 102/102 | bounds 回声（每键入 1 额外帧） |
| type@54 | 49.6Hz | 0.95 | 17/24ms | 162/162 | 回声在高频并入主帧 |

### 1.2 HTTP 档（单发驱动——下游 027 协议形态）

| 档.相位 | 有效驱动 | follow | 备注 |
|---|---|---|---|
| 5kb.scroll@10/16 | 9.96/15.86Hz | 1.07/0.98 | 带内 |
| 5kb.scroll@34/54 | 27.9/36.5Hz | 0.92/0.94 | **HTTP 时延地板**（scroll p50 31-37ms）封顶驱动率——54Hz 档物理不可达，谱面如实记录达到率 |
| 5kb.type@10-54 | 9.95-46.6Hz | 1.93→1.08 | type 单发 p50 15-18ms 可达 ~47Hz；回声稀释 |

**驱动机制勘定（T-00 内建项）**：单发 call 在 type 相位可达
~47-50Hz（时延 p50 15-18ms），scroll 相位 31-37ms（校验锁+发送
与帧同步块互斥）→ ≥54Hz 档只能走内部通道（本件探针=计划预案的
「test-only action 注入」，AUTOUI_DRIVE_PROBE 触发文件协议）。

### 1.3 判别实验（钳制回声 vs 槽合并）

5kb fixture 内容高 ~1-2k px，scroll y=900×i 步进两步后全程钳制；
P656 控制器对钳制命令发校正回声（`__mcp_scroll` 回发——scrollupd
计数 2×动作数直证）→ 消息流 2×驱动率 → 配对率放大。1mb
（18k 行，全程界内）34Hz 回 1.02——**回声成分剥离成立**；54Hz 残余
0.79 为纯槽合并（下游复判滚动大文档界内——本残余即其 54fps 判定
的真实上游余量）。

## 2. 候选逐项判定（AC-01——五项不预设的定谳）

| 候选 | 判定 | 证据 |
|---|---|---|
| (a) 16ms tokio tick 真实触发 ~90ms（runtime×winit 唤醒/0.14.1 守卫副作用） | **证伪** | mcp_poll gap p50=16ms/p95 17-31ms 全档（10-54Hz）；`MissedTickBehavior::Skip` 下无聚簇；027 形态（91-95ms 恒定+突发连建）未出现 |
| (b) poll_mcp_actions 单条 try_recv 供给耦合 | **细化** | 消费零丢失（got==驱动数全档）；但 16ms 网格**重排消费间隔**——got 间隔分布 8ms×49/16ms×72/24-32ms×35（1mb.scroll@54）：原生 18.5ms 间隔被网格+抖动重排出大量亚 16ms 对 → 交付配对燃料。单条→drain 不改节奏（队列常态 0-1），**间隔结构**才是杠杆 |
| (c) request_redraw→RedrawRequested 合并/ControlFlow | **证实（合并落点）** | iced_winit AboutToWait 批处理：同批消息合并一次 request_redraw；pending-redraw 幂等——在制帧窗口内到达的第二消息并入该帧。交付间隔谱 16/32ms 双步直证 |
| (d) 驱动期多订阅竞争 | **证伪** | 低档（10-16Hz）全轴 1:1 跟随无退化；驱动期无 timer/parked/media 负载（041 稳态滚动不依赖泵——735 同勘） |
| (e) vsync/compositor | **证实（量子源）** | 呈现槽 ~16.7ms：交付 p50 16ms/p95 32ms 严格槽阶梯；原生间隔 18.5ms>槽宽则无配对（34Hz 档 1.02 直证）；16ms tick 网格把部分消息对压入同槽 |

**责任链定谳**：驱动 54Hz（18.5ms 原生）→ 16ms tick 消费网格重排
（配对燃料）→ 消息对落入 16.7ms 呈现槽（合并）→ 交付 0.79×驱动。
**027 下游 ~93ms 恒定 pacing 在上游不复现**——其定责属下游域
（版本偏差/工作负载差异），本件 handoff-v3 供零改动复判通道。

## 3. T-00 勘定的环境修正（如实记录）

1. **735/731 ladder 的 scroll 相位自始无效驱动**：旧
   `find_scrollable` 正则抓取祖先链 id（col/row），MCP scroll 校验
   （kind==Scrollable）恒拒——735 谱的 scroll 带实为**空闲节奏读数**
   （探针三候选实测：仅第 3 个 kind=Scrollable，前两个
   Column/Row）。ladder740.py 已修（试错锁定真 Scrollable）。
   → 731/735 的 scroll 相位带对照在本件**不可比**（驱动从无效变
   有效），零回退判据改由 type/typenl 带承担（见 T-03）。
2. **P656 控制器钳制回声**：scroll 命令出界时控制器回发校正命令
   （F-4 读回投影语义）——5kb 阶梯协议面伪影；下游界内滚动不受
   影响。阶梯谱以 1mb（界内）为主判据档。
3. **探针调度 Duration 下溢实录**：朴素 `deadline - Instant::now()
   - 2ms` 在检查与减法间时钟推进时 panic（X9-PANIC 实录，杀全进程）
   ——`checked_duration_since` 修复（本件 T-00 提交内）。

## 4. T-02 修复方案选定（决策输出）

**方案 P（选定）**：`mcp_action_subscription` 的 16ms AppTick 轮询
→ **push 通道订阅**（后台转发线程：`std mpsc recv()` 阻塞 →
futures channel → 到达即产消息），唤醒后 `try_recv` 循环取空
（**drain-to-empty 语义内含**——畸形/Disconnected 臂原样保留）。
预期：消息间隔恢复原生 ±转发抖动（>16.7ms 槽宽）→ 无配对 →
54Hz 档跟随回带。

- 依据：候选 (b) 细化+（c）（e）证实——杠杆在**消息间隔结构**，
  非吞吐；计划 §5 T-01 (c) 分支「即时绘制臂/Poll 面——不 fork
  约束下的最小绕行」预授权。
- 护栏：不 fork iced_winit ✓（纯 app 侧订阅重排）；712 r2 域零
  触碰 ✓（泵/dirty/epoch 语义不动）；门关零开销 ✓（无新门，复用
  既有 AUTO_SCHED_DIAG 轴）；heartbeat 2s 空闲拍保留（快照新鲜度
  语义不变）；Recipe 身份换新 id（订阅表去重安全——453 T4 教训）。
- 备选否决记录：改 unconditional-rendering feature（ Blast radius
  全应用——735 在案语义依赖，非最小面）；AppTick 加密至 8ms
  （网格仍在，配对燃料不减）；fork iced_winit 逐消息重绘（约束②
  禁止——且 push 后无必要）。

## 5. 下游复判解锁预告（回执要点——v3 详单另档）

- 上游交付能力谱（本报告 §1.1）：34Hz 界内 1.02、54Hz 界内
  0.79（T-02 后复测）——027 协议复跑即得判定；驱动余量参数
  （Q-2：54Hz 需 drive 睡眠 ≤3.5ms）不变。
- 行通道配对语义 735 已升格（100% 配对）——下游判定读行通道
  即逐帧真值，无需改其驱动协议。
