# PLAN-708 T-00 决策报告（708-decision.md）

- `stage: work` | `task: T-00` | `plan_revision: 2` | 状态：**进行中（静态裁决已定，运行验证项标注 pending）**
- 用途：冻结 r2 §5/§6 要求 T-00 裁决的契约参数；未决项按 §6 有界处置（阻塞依赖它的 M 实施，基线/文档照常）。

## D-1 帧屏障（Q-04）——**裁定：listen_raw 帧通知 + 异步回环序障；不 fork iced**

事实链（静态证据，见 baseline §2.4）：
1. iced_winit 0.14.0 RedrawRequested 臂内：`interface.update` → frames 订阅广播（lib.rs:972）→ `compositor.present`（:981）。广播先于 present。
2. frames 订阅产生的 Message 经 runtime 通道**异步回环**，应用在后续事件循环轮消费 ⇒ 消费时该帧 present 已同步返回（present 失败臂除外，失败帧自行 request_redraw 重试）。
3. `event::listen()` 过滤 RedrawRequested；唯一公开通道是 `listen_raw()`。无 post-present 回执 API。

**裁定**：M-03/M-04 的"骨架已交付→启动 Init"序障采用 `listen_raw` 收 RedrawRequested：
- 应用层保证 = **第 N 帧消息消费发生在第 N 帧 present 返回之后**（异步回环序）。以"消费帧消息才运行有界 Init 泵"实现骨架先呈现。
- **非**硬在屏证明（无 fence 回读、present Ok ≠ 扫描完成）。措辞纪律：验收用"present 已返回的帧"而非"已扫描到屏幕"；AC-06 的在屏证据用运行期截图（skeleton 可见 + 时间戳）佐证。
- **防线**：listen_raw 无过滤会自我续帧（每条帧消息=重建+present→下一条）。约束三条：(a) 帧通知订阅**仅在存在未完成 demand 时激活**（零 demand → Subscription::none，同 702 has_parked_tasks 条件订阅家族）；(b) 帧消息消费即运行**有界**泵（预算片数上限，非全量 drain）；(c) 骨架帧构建必须命中 memo 廉价路径（S-01 联动），否则泵重建成新瓶颈。
- 运行验证（pending R-1）：实测帧消息到达时间 vs AUTO_MEMO_DIAG 构建时间戳 vs 截图可见性，确认序障在实践中成立；若实测矛盾（帧消息早于 present），本裁定作废并回 needs_replan（不得静默退回 sleep/tick）。

## D-2 CPU 片预算与写序（Q-05/Q-02）

**初值冻结（r2 §5 M-02 提议值，T-11 实施并按实测校准，不得为通过而下调指标）**：
- 每片 4ms / 4096 指令，至多每 64 指令查时钟；会话每轮 CPU pump 总预算 8ms。
- ready 集调度 FIFO 起步，本轮让出任务本轮不重跑（防饥饿自旋）；多 App 轮次预算共享。
- 同 App VM 写事件串行有界队列 128（input 代写/timer/props/MCP fixture/reload）；宿主滚动/resize/close 不入此队列。

**写序纪律裁定**：全部 CPU continuation 与写事件消费**只在 iced update 上下文串行执行**（= 702 泵既有单线程纪律的延伸，不引入第二执行线程）；共享堆不需要新增锁——序障由"唯一泵消费点"保证。Parked 凭据扩展：`ParkedWait` 增加 CPU Runnable 变体（保留既有 HttpRequest/Future/HttpStream 三凭据，不抹 707 分支）。累计 10M 指令护栏跨片累计（I/O 等待不计忙时）。

## D-3 707 配对（Q-07）

- 707 已落地于本计划基线（e1bab972e）：`ParkedWait::HttpStream`（engine.rs:339-343）、`stream_ready` 探测（vm_bridge.rs:1599-1601）、legacy busy-wait 流超时臂（engine.rs:2643-2656）、SSE decoder/http_stream FFI 全量在库。
- 708 承诺：wait 集合只增不改；`resume_fn_by_name_segment` 唤醒语义不动；最终 review 按 T-09 对 {HttpRequest, Future, HttpStream} 逐种串联验证（wait→CPU resume→cancel/close 资源一次终结）。

## D-4 memo 三态与失效（S-01 实施参数）

- 现状：`prop_memo || env=="1"`（aura_view_builder.rs:5140）。r2 三态表落地为：
  - `AUTO_OUTLET_MEMO=0` → 强制关（覆盖 prop=true——诊断逃生门优先级最高）；
  - `AUTO_OUTLET_MEMO=1` → 强制开；
  - 未设/其他值 → prop 未设=on（缺省面变更：现状缺省 off，r2 后 outlet 页缺省 on）、prop=false=off、prop=true=on。
- 046 并集契约变化点：显式 `memo:false` 与 env=1 组合（现状=off，r2=on）——测试矩阵必须含此组合。
- 失效面新增：局部 UI epoch（宿主级，进入 MemoKey 或保守失效）；`child_last_init_identity` 在 demand 化后语义变为"登记簿"——命中帧重放挂载/回调路由簿记保持现状（render_outlet_page_memo:5225-5239 已有）。

## D-5 归因与热点排序（运行探针——已闭环，见 baseline §3）

- **R-2 已裁决**：页构建秒级成本 ~100% 归一于 preview-card 臂每实例 `VueGenerator::new()`（~1.1–1.3s/卡，内容无关、跳过代码生成仍 ~9.0s/7 卡——判别实验四组）。热点排序：① preview-card 生成器重建（S-02 缓存/共享对象）② DataTable 静默 Degrade→memo 永不生效（S-01 失效面 + 诊断面 T-06）③ FileTree 恒 MISS（T-12 computed/依赖缓存）④ 每导航双构建乘数（M-03 通知合并核查）。
- **R-3 部分裁决**：gallery 加载路径无 native 主导慢段（build 成本=解释器+生成器）；重 native 单调用上界移交 T-11 夹具验证，不作为 gallery 基线项。
- **R-1 部分裁决**：press 往返 ≈ update+render 轮时长（action 等待渲染轮），实测与 build_ms 一致（9575ms press vs 9565ms build）——序障的"消费帧消息在 present 后"一侧由 MCP 通道实测佐证；`listen_raw` 应用层序障的最终实证在 T-04 实装时以帧消息时间戳 vs 构建时间戳闭环（T-00 预算外不再单独跑）。
- 附带实证：现状 `AUTO_OUTLET_MEMO=0` 不能关闭 prop `memo:true`（OR 逻辑，app.at:701 仍 HIT）——三态强制关语义确属缺失，S-01 实施对象。

## D-6 阻塞登记

| ID | 项 | 影响 | 处置 |
|---|---|---|---|
| R-1 | 帧序障实证 | T-04 开工前置 | 静态机制已证（异步回环序）+MCP 侧佐证；`listen_raw` 实装闭环移 T-04 首验 |
| R-2 | 热点归因矩阵 | T-02/T-12 选点 | **已闭环**（baseline §3.1-3.2） |
| R-3 | native 单调用成本 | M-02 native 超限处置 | gallery 无 native 主导段；上界验证移交 T-11 夹具 |

无 T-00 级阻塞——M 实施（T-01/T-02/T-11/T-12）可依本报告开工；探针失败项无。
