# PLAN-740 downstream-handoff-v3（下游复判解锁预告 + 范围断言回执）

> 收件方：auto-edit PLAN-027 后继复判件（scroll_fps ≥54fps 正式判定
> ——027 协议复跑即得，**零工具改动**）。本件=上游交付能力谱面 +
> 复判参数预告 + 环境注记；不含下游任何判定执行（另件消费）。
> 判定阈值/语义 frozen 不变（54fps/面板×0.9——本件零触碰）。

## 1. 上游交付能力谱（PLAN-740 改后——T-03 双谱）

| 驱动形态 | 档 | 交付跟随（改前→改后） | 备注 |
|---|---|---|---|
| 内部通道精确驱动（与下游 in-process 驱动形态同构） | 34Hz scroll | 1.02 → **1.08** | 1mb 界内（下游形态） |
| 同上 | **54Hz scroll** | 0.79 → **1.06** | **±10% 带内——54fps 上游解锁** |
| 同上 | 54Hz scroll（5kb 钳制回声场景） | 0.73 → **1.04** | 回声放大场景同带 |
| HTTP 单发（027 协议驱动形态） | ≤47Hz type / ≤36.5Hz scroll | 1:1（全档） | HTTP 时延地板封顶驱动率（scroll p50 31-37ms）——≥54Hz 档 HTTP 驱动物理不可达，见 §2 |

- 修复面：MCP action 消费 16ms 轮询 → push 通道（到达即投递 +
  drain-to-empty）；定责链与证据=`T-00-T-01-report.md`。
- 契约沉淀：frame-pipeline-incremental.md **§4d 交付节奏持续率契约**
  （跟随判据=交付数/驱动数 ±10% 至 ≥54Hz 档——SD-01，merge 期正式
  沉淀）；观测通道=frame-observability.md §1/§4（SD-02）。

## 2. 复判参数预告（Q-2 重申——027 协议参数不变项）

- **驱动余量**：下游 MCP 调用地板 p50 ~14-16ms（027 实测）→ 54Hz
  驱动需 drive 睡眠 ≤3.5ms（复判件 drive 参数预告；如走 in-process
  直推则无此约束——740 探针同型）。
- **HTTP 时延注意**：scroll 单发在活动滚动期 p50 31-37ms（校验锁与
  帧同步块互斥）——HTTP 驱动 54Hz 档需验证实际达到率并如实记录
  （740 HTTP 档实测有效驱动封顶 ~36.5Hz——谱面不冒领）。
- **行通道**：735 配对修复起行通道=逐帧真值（100% 配对）——判定
  读行通道即呈现真值，无需改判定协议；`[P725-FRAME] draw_end_ms`
  列为呈现完成逐帧锚。
- **判定阈值语义 frozen**：54fps/面板×0.9/3s 窗——本件零触碰。

## 3. 环境注记（复判件执行时知悉）

- **偶发静默退出**：本机 2026-10-08 实录——app 进程无 panic 痕迹
  突然退出（rc=0/1 混合，改前改后二进制均出现，手工短复现不复现；
  同机他方会话重负载嫌疑）。复判跑如遇「textarea 未定位/连接拒绝」
  直接重试该档即可（740 采集协议同款——失败档重跑，不折算判定）。
- **state.at 损坏砖启动**：进程被杀于写半截时 `.am/state.at`（app
  侧持久化旁路文件，gitignored）可残留 NUL 填充 → 下次 boot
  lexer panic（parser.rs:404）**进程起不来**。处置=删除
  `examples/ui/041-auto-edit/.am/state.at` 即恢复。**债候选**
  （KNOWN-DEBT-and-RISKS）：boot 侧对损坏 state.at 的容错（解析
  失败→忽略重建）属 app 健壮性域，非本件范围。
- **ladder scrollable 定位修正**：`autoui_find kind=scrollable` 响应
  含祖先链 id（col/row）——旧 ladder 正则取祖先 id 被 MCP scroll
  校验恒拒（**731/735 谱的 scroll 相位实为无效驱动**，其 scroll 带
  谱=空闲节奏读数）。ladder740.py 已修（试错锁定真 Scrollable）；
  复判件若复用 731/735 脚本需同修。

## 4. 范围断言（AC-04——本件执行期实测）

- **auto-edit 零改动**：`git -C D:/autostack/auto-edit status --short`
  = 空（porcelain clean，2026-10-08 实测）。
- **712 r2 域零触碰**：`git diff master -- crates/auto-lang/src/ui/
  dynamic.rs frame_segments.rs frame_bench.rs sched_diag.rs` = 空
  （dirty/epoch/poll_frame_pump 泵臂语义零 diff）。
- **不 fork iced_winit**：本件 diff 全部落 app 侧（renderer.rs 订阅
  重排 + mcp_server.rs 临时诊断 + 探针线程）；iced_winit 0.14.1
  钉版零触碰。
- **零 .at 源改动**：041 例源零 diff（golden 基线 PNG 除外——
  `740-*.png` 新增，源文件零改动）。
- **master 分叉注记**：本件分支基面 c3ccd32c3，执行期 master 前移
  （PLAN-097 画布定位域 3 code commit + 736/738 docs）——域无交联
  （097=窗口定位/画布，740=帧管线订阅），fold 期 merge 自然合流。

## 5. 账本项

- **P740-1**（015 先例 merge 期项）：`.autoos/specs.json` upsert +
  module plans.md 回写 + `spec-index.py`——归 merge 技能执行。
- 台账快照（2026-10-08）：本件 worktree=lang-740（plan-740-dev）；
  证据=docs/plans/evidence/740/（双谱 JSONL×8 + 定责报告 +
  golden740.py + 本回执）；代码锚=3e1d8bb57（push 重排）/
  f3bf7f6b3（双谱+门禁）。
