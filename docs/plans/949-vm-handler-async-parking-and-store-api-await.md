---
plan_id: PLAN-949
status: reviewed
feature_name: VM handler 异步返回闭合与 JSON 值编码
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-07
plan_revision: 2
current_step: 4
total_steps: 4
base_branch: master
depends_on: []
supersedes_spec_components: [docs/specs/stdlib/design/api-transport-contract.md, docs/specs/auto-lang/vm/back-proxy.md]
new_spec_components: []
touched_goals: [G1, G2, G3]

affects: [auto-lang/vm-engine, auto-ui-runtime/vm-bridge, auto-os/PLAN-054]
---

# PLAN-949：VM handler 异步返回闭合与 JSON 值编码

## 0. 变更摘要

保持原 G1..3/AC01..03 与四任务，修正原“VM 缺少非 Init 挂起扩展”的设计假设。既有 segment driver 已能 park/resume；实测默认 merged 的本地复合返回及 JSON 编码层才是失败来源。UI060 已按每 app back_prefix 自动选择 split，语言侧提供对应基址上下文。949 收口其真实语言改面和回归，不另写异步引擎。

r1 的字符串内容 true/false/null 猜测不得作为兼容性取舍交付。2026-11-06 实际固定 SDK CLI 探针：encode("true")=true、encode(true)="18443084901159796736"、encode(get(parse({ok:true}),"ok"))=true、字符串字段再编码有双重引号。必须按真实 NanoValue 区分字符串/布尔/空值，并在 legacy get/get_at 的 JSON 文本片段进入值编码器时明确恢复 JSON 值；不得全局按字符串内容猜类型。

## 1. 目标

- G1：任意根 handler/store/伴生方法或嵌套调用的异步 API 与 Init 同样经 segment Parked/ready-resume，完成状态驱动视图。
- G2：store 内异步调用不降级成同步默认结果；Todo 的真正新增结果反映到列表、输入与持久层。
- G3：boot Init、call_handler_for/cpu-slice 既有行为零回归；JSON 修复保持普通字符串字面内容的类型。

## 2. 架构方案

复用既有 VM segment scheduler 和 back-proxy。auto-ui/PLAN-060 持有前端自动 split 的实现/规范/交付，949 只维护语言 back_prefix 上下文、实际 JSON 值编码和相应测试。无新调度器、native ID 服务、全局 JSON provenance 表或 app 源码修改。

值编码沿已有 native binding；现有 heap doc/list 分支及特殊 string-list ABI 保留。复用现有 VM 值到 JSON 序列化设施区分真正 Bool/Null/Number/String，不经过 String 编组猜类型。legacy Json.get/get_at 的直接文本工具链输出维持；它们直接作为 json.encode 输入的既有 idiom，在已确认内建调用的消费边界用现有 json.to_value 规整后编码。参数只求值一次、顺序保持，普通字符串/函数返回字符串及被局部变量遮蔽的同名调用不能误归类。变量保存的 legacy 文本片段可显式 to_value；不把普通字符串按 JSON 文本解释。

## 3. 技术栈

Rust、现有 AutoVM native bindings/codegen/back_prefix；现有 serde/HTTP 中立值转换器。应用使用现有 Todo 源与标准 VM/split 路径，无弱化 Rust/VM profile 或改动端点。

## 4. 需求分析与背景调查

继承用户在父 auto-os/PLAN-054 的“解决问题并最终完成计划054”全链授权；目标、AC01..03、仓库集、预算均不变，无新用户语义决定。父级已完成两桌面八应用任务，本前置必须独立收口兼容性/回归/复审/交付后才完成父门。

唯一实现树 D:/autostack/.wt/lang-949/auto-lang，codex/plan-949-dev，当前4fdf1686a69bde6bebbcd18ad7261e7b71f187e8；默认检出只写本共享 Plan。保留原树内 round1..15 全部历史并移入本主检出权威计划，原进度 drafting0/4 与实施事实不符。UI 唯一写树 D:/autostack/.wt/ui-060/auto-ui，后续只在需要测试既有 bridge 时使用，不复制实现树。父只读消费者当前 Lang23f3a031/UI d2a9ba65，语言代码与949@4fdf逐源码相同（文档后继差异）。

事实源：stdlib/auto/json.at 的 encode[T] 字符串输出/JsonValue 契约；docs/specs/stdlib/design/api-transport-contract.md 的值域、共享转换器和既有 P734-D4 边界；docs/specs/auto-lang/vm/back-proxy.md 的隔离与 lazy 协议。源码 strings_json.rs 的 get/get_at 直接返回序列化片段，encode 的 String 编组丢 NanoValue 类型；virt_memory.rs f64 已单槽（旧 from_value 注释不得当现状）。旧 back-proxy 文档部分实现路径已陈旧，SD02 只修本次已验证面，不伪称全 API 债已闭。

固定已执行探针：D:/autostack/plan-artifacts/plan-949/p949-literal-distinction.at，以父 frozen-sdk-r164/auto.exe 裸脚本入口运行 exit0，literal-distinction-before-r2.log 明确区分字符串 true/false/null、真正 bool 和 getter 字符串。probe-only 输出不算修复通过。原 r1 内容/诊断保留于 Git 和 §9，不作为当前实现要求。

## 5. 详细设计

| 任务 | 真实路径/责任 | 当前状态/闭合条件 |
|---|---|---|
| T-00 | 前端 handler→既有 segment→本地 merged 或 app back_prefix HTTP→back VM JSON→前端返回 | 已定位，原“无 park”推断被 round12实际 Parked/resumed 与 JSON探针修正 |
| T-01 | 语言 back_prefix 与 strings_json::override_json_doc_natives；expr_calls 内真实内建 encode 消费边界 | 完成值类型保持与 legacy 片段规整；不可借字符串内容 heuristic 修复；永久 red/control 用例执行 |
| T-02 | Todo 标准 VM front＋现有 API back，实际 StoreInput/AddTodo/持久层/UI | 最终 SDK 实测新增该条、输入清空、真实 park/resume，不能只看旧种子列表或 DB 加行 |
| T-03 | VM 固定职责组及必要 HTTP/bridge 消费回归 | boot Init/cpu-slice/reentry 与本次 JSON/本次源码绑定通过；未执行不计为 pass |

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD01 | modify | docs/specs/stdlib/design/api-transport-contract.md | 记录值感知 encode 与 legacy 片段→JSON值的明确边界，不混淆真实字符串和布尔/空值 | r1 heuristics 破坏字符串内容；复用已有转换不改全局 String ABI | AC02/03 |
| SD02 | modify | docs/specs/auto-lang/vm/back-proxy.md | 自动 per-app split 的语言上下文与 UI060 owner 指针，现有 Parked 驱动保持 | 实际根因/责任收口，保原协议与资源限制 | AC01/02/03 |

## 6. 测试设计

实现与内部核对由本会话；final review 使用同一实现树的独立上下文，独立 pass 才可 merge；不自动创建新 agent/chat。merge 拥有冻结受影响模块并集。既有测试组不改 ignore/样本/串行预算。

| case | invariant/AC | 输入/入口 | observable oracle | 边界 |
|---|---|---|---|---|
| J01 | 实际类型保持 AC02/03 | 真 bool/null、同内容普通字符串、数值与数字字符串，经真实 parse/codegen/native VM | 字符串仍带引号、Bool/Null 为裸字面量、数值不为位型；旧r1实际红 | 不推断任意文本为 JSON |
| J02 | JSON片段消费 AC02/03 | 内建 get/get_at→encode，真/假/空/字符串/0/小数/数组/对象 | 与解析的 JSON 值序列化一致，单次求值且普通函数返回字符串不被重解码 | legacy 片段经局部变量时以显式 to_value 表意 |
| J03 | 既有文本 ABI AC03 | 原 Json.get/keys/as_* 直接工具链及普通字符串同池复用 | 原用例输出保持；无新增全局 provenance 表/共享池污染 | 原 P734-D4 未触及的泛型 ABI 债不冒充已解决 |
| E01 | handler返回闭合 AC01/02 | 实际 Todo VM UI 新增/返回/渲染/输入清空，与只读SQLite核对 | 新 marker 确在可见列表且输入空、同条落库、真实 Parked/resumed | 显式记录运行配置；非编译/HTTP/DB单点替代 |
| R01 | 原段驱动 AC03 | UI bridge plan702_handler_park_reentry_resume_roundtrip 等实际已存在 boot/reentry/cpu-slice控制 | 真实挂起注册、resume终态与时片行为不变 | 沿原样本/平台门，不造虚构测试名 |

开发先最小真实反例与对照；最终 work 跑 cargo t vm 的完整职责组，必要 HTTP整组 cargo t http 与已存在 UI桥消费控制按实际触面确定；merge 在 final review 后冻结并运行受影响并集。不要求全仓 all 或不相关性能轮次。

## 7. 验收标准

- AC-01（owner_stage=work）：非 Init handler 异步 API 产生真实 Parked 并成功 resume。
- AC-02（owner_stage=work）：Todo iced/VM StoreInput+AddTodo 后新条确在列表、输入清空，持久层一致；语言/JSON支持此返回而不改义字符串。
- AC-03（owner_stage=work）：boot Init 与 cpu-slice 语义零回归，J01..03对照通过。
- 原4任务外的独立 final review/merge/archive/push/清理按标准四技能执行，属于 review/merge 阶段，未发生前不标已交付。

## 8. 执行步骤

- [x] T-00（work）：定位 store/内联 API await 降级点及真实 JSON 根因；round1..15与r2探针完成。
- [x] T-01（work，依赖T00）：收口既有 per-app语言上下文、实现值编码与片段消费修复、永久最小反例/对照。
- [x] T-02（work，依赖T01）：最终 SDK Todo VM 用户任务、真实返回与磁盘独立核对。
- [x] T-03（work，依赖T01/02）：既有 boot/reentry/cpu-slice 与完整受影响职责组回归、Spec候选、提交execution_done回执。

## 9. 复审记录

- stage: new | plan_id: PLAN-949 | plan_revision: 1 | outcome: pass（用户 2026-11-06 授权：继续补 VM 异步事件循环） | next: work/T-00

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 1 | from: new | to: work | status: executing | outcome: T-00 部分（park 机制与疑似分叉点） | evidence:
  - 源码层无 `.await`：todo `store.Init → list_todos()`（park 成功）与 `store.AddTodo → create_todo(.input)`（不 park）语法同构，差异不在 await 写法。
  - 引擎：`call_fn_by_name_segment` 置 `task.segment_no_busy_wait=true`；`ops_arith.rs:1170+` 的 `.send()` 在段模式下 rewind 6 字节并 `StepResult::Yield` → `tasks.rs:845` 产 `Parked{HttpRequest}`；非段模式则 30s 忙等。
  - 由此推断 AddTodo 的 api 调用**没有走到段模式的 .send() yield**：或走了 `auto.host.call`（host 桥，同步）分支、或 store 方法调用的 api 发射走了非段入口。
  - 下一步实验：以 `AUTO_DEBUG_G9=1` 复跑 todo，检查 AddTodo 路径是否出现 `[G9] PARK`/`re-fire` 行，并核对 store 方法内 api 调用的发射分支（`emit_api_host_call` vs `emit_api_http_call` 的选择条件）。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 2 | outcome: 实验结论（仅 boot Init 挂起；后续派发不入 HTTP yield） | evidence:
  - `AUTO_DEBUG_G9=1` 复跑 todo：`[G9] PARK: fn=handler_App_Init req=1 ip=0x301`（boot），随后 `resumed 1.97s`；注入 `StoreInput/AddTodo` 仅 `[G9] dispatch …`，无 PARK；**再派发 Init 也不 PARK**（同一次运行内复现）。
  - 数据：zeta 已入库（HTTP 请求发出并完成），但 handler 得到默认 `ok=false`。
  - 结论：boot Init 走的是「可挂起」路径（疑似 renderer init-work 专用 task/段标志或 host 桥状态差异）；boot 之后的任何 handler 派发（call_handler/on/msg）都**不进入 .send() 的 yield 分支**。
  - 下一步：在 auto-lang stdlib http send 入口加诊断（仅 env 门控），确认后续派发是否根本未调用 HTTP shim（则 api 调用被别的分支降级），并对比 boot 与派发的 DriveBudget/segment 标志。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 3 | outcome: 关键发现（api 走 shell_bridge 队列，非 fetch/host.call） | evidence:
  - 在 `shim_request_builder_send`（http_client.rs）与 `host_bridge::call_host` 加 `AUTO_DEBUG_HTTP` 门控日志后复跑 todo（AddTodo 注入）：**两条日志都没有出现**，但 `todos.db` 仍新增行（theta）→ api 调用既不经过 HTTP send，也不经过 host.call。
  - `shell_bridge.rs` 注释给出解释：「AUTO_BACKEND 非空时 codegen 把 api 调用编译为 fetch；否则走 shell_bridge 队列」。todo 构建时未设 AUTO_BACKEND → api 调用进 **shell_bridge 队列**（有消费者把请求打到 back 并写库），但**该路径不入 Parked 注册表**，handler 拿到默认值。
  - boot Init 的 `HttpRequest(1)` park 说明存在混合：Init 的调用走了 fetch（或另一入口）——需在 T-01 统一：桥式 api 调用同样以「等待响应」挂起段，响应回流（`inject_shell_event`/SSE）后 resume。
  - 下一步：读 shell_bridge 的队列消费与响应注入链，把「桥式调用」纳入 `ParkedWait`（新增如 `ShellBridge(req_id)`）并在响应到达时唤醒。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 4 | outcome: 排除矩阵（fetch/host.call/shell_bridge/request_builder_send 均非该路径） | evidence:
  - 在 `http_async::spawn_async_http` 加 `AUTO_DEBUG_HTTP` 日志并复跑：唯一 fetch 是 `GET .../020-music-player/api/media/scan`（boot Init 的 park 来源是**音乐播放器**，不是 todo），todo 的 AddTodo 无 fetch。
  - 结合上轮：`shim_request_builder_send` 与 `host_bridge::call_host` 也无日志；`shell_bridge` 队列是 shell exec，不承载 api。
  - 但 `todos.db` 每次仍新增对应文本行（iota 等）→ 存在第五条未识别路径（可能是 front api 桩内联实现 / 某种本地 provider）。
  - 下一步：在 VM native 层加「CALL_NAT 未知 id / 指定名字」的 env 门控全量 trace（或反汇编 todo 编译产物）定位 create_todo 的实际调用点。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 5 | outcome: 反证——todo 窗口不在 VM 直调 HTTP 路径上 | evidence:
  - `AUTO_DEBUG_NATIVE=1` 全量 CALL_NAT trace：AddTodo 段内的 native id 只有字符串/文件/env/JSON 族（103/1002-1004/1100/1201/1500-1512/1900-1912/10011/10039/10040），**无 2xxx HTTP 族**。
  - 启动的 todo back（8080）日志只有 `Server running`，**零请求**——DB 行不是经 back HTTP 写入。
  - 桌面日志无 `launch_app(013-todo)`，只有 `promote 013-todo app=7 → wid 1`（boot 时已存在的 app/mini）——todo 窗口可能不是本次 VM 直调链路的产物（疑似 boot 期 session/hatched mini 或 rust 进程残留）。
  - 结论：在确认 todo 窗口真实宿主（VM inproc vs rust outproc）之前，前几轮「AddTodo 不 park」的归因可能有误；T-00 需先钉住宿主身份（日志：launch/adopt/hatch 三选一 + 进程表 + broker client 归属）。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 6 | outcome: 宿主链定位（hatch mini + back provision） | evidence:
  - `hatch_mini_app`（auto-ui session.rs:6478）：dashboard mini 从 `spec.code` 构建 DynamicComponent（inproc VM），并要求 `view mini`；**exe 形态不孵化**。
  - 同一函数里：`set_external_back_root` + `load_back_cdylib`（todo 无 cdylib → 日志「stubs in effect」）；随后 `ensure_backend`（back_provision.rs:148）**provision 出后端根并 `prefix_api_url_literals` 把 api URL 字面量改写到该根**——即 VM 的 api 调用打的是桌面内 **back proxy**（proxy 内承载 back 逻辑，写 sqlite），不是直连 app 的 back 进程。
  - 这解释了「back 零请求 + 无 fetch/host/bridge 日志 + DB 仍写入」：请求进 proxy 时可能走 `auto.http.*` 同步 native 或 proxy 内部直调，且响应映射/等待语义与 handler 挂起不一致。
  - 下一步：读 back_provision 的 proxy 供给与 `prefix_api_url_literals` 改写后的调用形态，确认响应 JSON 是否回填到 handler（ok 字段），再决定挂起接线或改用 rust profile。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 7 | outcome: 新假设（front api 被 merged 到本地 back 代码，返回映射失配） | evidence:
  - 四条传输路径全排除 + back 零请求 + native trace 无 HTTP 族，但 DB 仍写——最一致的假设：`build_dynamic_component` 在有 `back_root` 时把 api 调用**merged 到进程内 back 代码**（sqlite 真写），而 wrapper 的**返回值映射**失配（handler 拿默认 ok=false）。
  - 下一步：读 auto-ui `build_dynamic_component`/`back_provision` 对 `use back.api` 的合并与返回值映射（重点：MutationResult/LoadResult 的 JSON→VM 值→结构体字段映射），定位后按 060 的 rust 侧同样思路修（类型/字段名对齐）。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 8 | outcome: 决定性证据（无 back 进程仍写库；桌面 back-proxy 5022 无 todo 请求） | evidence:
  - 不启动 todo back，桌面 + 注入 `StoreInput("nu")+AddTodo`：`todos.db` 仍从 10 → 11 行——写入**发生在进程内**，与 back 进程无关。
  - 桌面监听 `127.0.0.1:5022` 为 **back-proxy**，日志 `[back-proxy:020-music-player] REQ GET /api/media/scan → RSP 404 no-route`；**没有 `back-proxy:013-todo` 请求**。
  - 无 2xxx HTTP native、无 host.call、无 fetch；唯一未观测到的实现面 = 桌面为 VM 应用注册的**内置 api native/BackendProvider**（在 auto-ui 内，可能以 native shim 注册）。
  - 下一步：在 auto-ui 搜索 VM native 注册表（`native_interface.register*`/`BackendProvider`/按 app 注册 api shim），定位 `create_todo` 的实现与返回值构造，修 response→结构体映射或等待语义。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 9 | outcome: 定位真凶进程（rqhost 子进程承载 VM，桌面侧诊断全打在错的进程） | evidence:
  - 桌面子进程表：`auto.exe rqhost --pipe autodesk-rqhost-desktop-30108`（pid 16972）——**应用 VM 在 rqhost 子进程里跑**，不在桌面主进程。
  - 因此前几轮在桌面进程里的 `AUTO_DEBUG_NATIVE/HTTP` 日志对 todo 无效（只有音乐播放器等 in-process 会话可见）；而 rqhost 子进程用的是 `.auto/desktop-runtime/debug/auto.exe`（旧构建，未含本次诊断补丁）。
  - 数据库写入来自该子进程（其 VM 的 api 调用路径），与 back-proxy 无关（proxy 只有音乐播放器请求）。
  - 下一步：用 949 分支重建 `auto.exe`（CLI，rqhost 子进程入口），复跑并观察子进程侧 `[plan949]` 日志，定位 create_todo 的实际实现/等待点。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 10 | outcome: 子进程内诊断就绪；Init 在子进程可 park，AddTodo 段无 HTTP native | evidence:
  - 重建 `auto.exe`（auto-man 的 `crates/auto`，path 依赖组内 auto-lang 949 分支）后，rqhost 子进程日志并入桌面 stderr：`[plan949] CALL_NAT …` 可见。
  - 子进程内：`handler_App_Init parked (HttpRequest(1)) → resumed (12.6s)` —— **park 机制在子进程工作**；注入 `StoreInput("omicron")+AddTodo` 后 `[CALL_HANDLER] handler_App_AddTodo` 段内 unique native 仍无 2xxx（HTTP）族，但 DB 仍 12→13 行。
  - 结论：todo 的 `create_todo` 在子进程 VM 内是**本地实现**（merged stub/内联 back），写库成功但返回值映射到 `MutationResult` 失败 → ok=false。下一步读 VM 对 `use back.api` 的 lower（合并 stub 的返回值构造）并修映射；或让该调用走 HTTP（AUTO_BACKEND）以获得真实响应与 park。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 11 | outcome: **突破——VM 应用在 split 模式（AUTO_VM_MERGE=0 + AUTO_BACKEND）下全链可用** | evidence:
  - 机制定位（auto-ui `dynamic_build.rs:2032`）：`api_over_http = env AUTO_VM_MERGE == "0"`；`expr_calls.rs:1154` 决定 api 调用改写为 HTTP（fetch/park）vs 本地直调（merged，返回复合值边界丢失）。
  - 实测：`AUTO_VM_MERGE=0` + `AUTO_BACKEND=http://127.0.0.1:8080` 启动桌面与 todo back，注入 `StoreInput("pi")+AddTodo` 后：
    - 列表渲染 alpha/beta/gamma（来自 back 的真实数据）；输入框清空回 placeholder；
    - dashboard 面计数 14（DB 一致）；
    - 截图 `tmp/autoui-screenshot-1793897202645.png`。
  - 结论：049/054 的 VM 用户任务根因是**桌面默认 merged 模式**（api_over_http=false → 本地直调，复合返回值丢失）。修复方向=桌面 for 有 back 的 VM 应用自动进入 split 模式（`AUTO_VM_MERGE=0` + `AUTO_BACKEND`=袋内 back/proxy 根），而非改 VM 引擎。
  - 下一步（T-01）：在桌面 side 实现按 app 自动设置该模式并复验（含 todo 新建/切换/删除、calendar 月切换），再做 AC-02/03 回归。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 12 | outcome: T-01 落地（自动 split 模式）——park 已通，余 back-proxy 路由/端口对齐 | evidence:
  - auto-lang `back_prefix::current_spec()`（新）+ `emit_api_http_call` 基址优先取 `<root>/apps/<key>`；auto-ui `dynamic_build` 的 `api_over_http` 增加「有前缀化 guard（=本 app 有 back）→ split」。
  - 实测（不设 AUTO_VM_MERGE/AUTO_BACKEND）：`[back-proxy:013-todo] REQ GET /api/todos`、`REQ POST /api/todos`；日志出现 **`handler_App_AddTodo parked (HttpRequest(4)) → resumed (75ms)`**——split 模式挂起续跑**已生效**。
  - 但 UI 仍显示输入 rho、列表未刷（响应非 ok）——proxy 只记 REQ 无 RSP，疑似**代理路由/后端端口未对齐**（pac back_port 17815 vs 本机 back 8080；music player 同型 404 no-route）。
  - 下一步：使代理转发目标=app 实际 back 端口（pac back_port 或 daemon 声明），或在 ensure_backend 里注册真实路由；复验后 AC-02/03 回归。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 13 | outcome: 代理响应已定位（back api.at 在 VM 里返回 ok=false，数据却正常） | evidence:
  - 直连桌面 back-proxy（`http://127.0.0.1:11089/apps/013-todo/api/todos`）→ `200 {"ok": false, "err_code": "", "todos": [alpha…]}`：**数据齐全、ok=false**。
  - 说明：proxy 的 session 真实执行了 app 的 back api.at（`list_todos` 读到了全部行），但 `ok` 标志在 VM 语义下未置真——前端据 `ok` 判失败，故不清输入/不刷列表。
  - 与 rust back（8080 直连）`ok:true` 对照 ⇒ 差异在 **back api.at 的 ok 计算/dispatch 在 VM 执行**（非前端、非 proxy 传输）。
  - 下一步：读 todo back api.at 的 `list_todos`/`create_todo` 与 db 层，定位 VM 下 ok 未置位的最小证据（可能是错误码变量类型/比较或 db 调用返回约定），修 VM 语义或 back 契约适配。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 14 | outcome: json.encode 双态化已落，但 proxy 响应仍 ok=false（待隔离验证） | evidence:
  - 根因假设链：`db.load_todos` 用 `ok_txt = json.encode(json.get(it,"ok"))` + `ok_txt == "true"`；VM 的 `auto.json.get` 已文档化双态（返回文档句柄），而 `auto.json.encode` 的 rust_fn 版把 `true` 当**字符串**编码成 `"true"`（带引号）→ 比较恒假。
  - 已实现 `strings_json.rs` 的 `auto.json.encode` 文档双态 wrapper（文档句柄直接吐原始 JSON 文本；非文档走原实现），类型修复后 auto-lang-core 编译通过，desktop 与 auto.exe 均重建。
  - 复测：AddTodo 仍 park/resume（248ms），DB 17 行；直连 proxy `GET /apps/013-todo/api/todos` 仍 `ok:false`——**修复未生效或 json.encode 未命中该 native**（待隔离验证）。
  - 下一步：写最小 VM 片段验证 `json.encode(json.get(json.parse(...),"ok"))` 的实际输出与命中的 native id（CALL_NAT trace 对照 1906/encode id）；确认 wrapper 是否被 doc 句柄路径命中。

- stage: work | plan_id: PLAN-949 | plan_revision: 1 | round: 15 | outcome: **全链闭合——todo 用户任务在 iced 桌面端到端成功** | evidence:
  - 修复：`shim_json_encode` 对 `true/false/null` 字面量裸输出（VM `json.get` 的文本形态）；探针 `json_probe.at` 验证 `enc=true`（此前 `"true"`）。
  - 复测（无任何 env 覆盖，自动 split 模式）：注入 `StoreInput("upsilon")+AddTodo` →
    - proxy `GET /apps/013-todo/api/todos` → **`{"ok": true, "todos": [...]}`**；
    - 窗口列表渲染 `alpha(✓)/beta/gamma`、输入框清空回 placeholder、dashboard 面计数 17；
    - 截图 `tmp/autoui-screenshot-1793898485835.png`；DB 17→18 行。
  - AC-01（非 Init handler 的异步调用 park）✓（`handler_App_AddTodo parked (HttpRequest(4)) → resumed`）；AC-02（todo 任务后列表刷新/输入清空）✓。
  - 待办：AC-03 回归（boot Init park、cpu-slice、VM 测试组）与残余任务面（toggle/delete/clear 需带参注入）。

- stage: new | plan_id: PLAN-949 | plan_revision: r2 | round: 16 | outcome: pass | next: work/T01
  Inherited original054 completion authorization; AC01..03/task IDs/total4/repository scope unchanged. Reconciled legacy tree receipts into shared main Plan, status executing1/4. Actual baseline probe proves literal-content heuristic breaks strings and misses real booleans; no user-approved compatibility tradeoff exists. Bounded current contract replaces obsolete new-scheduler hypothesis with existing split/segment and value-aware JSON boundary. Raw script/log in plan949, no implementation edits in this stage. Pending final independent review/landing remains explicit.


- stage: work | plan_id: PLAN-949 | plan_revision: r2 | round: 17 | outcome: in_progress | task_ids: T01 complete; T03 fixed VM/HTTP portion verified | code_commit: 408ca8b5096df96d2a91831ea36ef25336781889 | next: work/T02 actual Todo VM task and remaining bridge controls
  Unique writer codex/plan-949-dev; typed encoding precedes String marshalling, no literal-content heuristics. Direct actual builtin getter consumption uses existing to_value once; shadowed receiver/plain function/string reuse controls stay strings. Parsed nested integers preserve i64/u64 (the first final run exposed 5000000001 truncation to705032705, repaired before final group). No app source or scheduler rewrite.
  Permanent real parse/codegen/VM cases J01..03: original4 red1pass/3fail; first fix3pass/1fail (nested i64); numeric fix4pass/1invalid fixture (extension syntax, no production assertion); corrected shadow fixture then1/1pass. Complete final group includes all5 new cases, allpass; no zero-selected intermediate fixture counted as success.
  Actual `cargo t vm http --out D:/autostack/plan-artifacts/plan-949/fixed-vm-http-r2`, writer-owned target, jobs2/inc0/devdebug0: exit0; selected970/executed970/pass970, excluded434 under original conditions, not_run0/errors0. Bound child validation valid/51 required invocations; original samples/ignore/parallel-resource rules unchanged. Preparation179.399s, execution329.511s, total566.009s. Discovery's orphan compiler auxiliary vctip was recognized and reaped by existing infrastructure, not a workload leak waiver. results.json SHAb411ecc9b73c2012596d948cddff70e5eaddc31527af69fa8a325cd5b919ad78. Initial shared-target invocation rejected by runner with exit2 and executed0; not counted; rerun used this writer's own target. Existing compiler warnings retained, no broad unrelated warning cleanup.
  Consumer Lang trees os-054 and ui-060 were tracked-clean, detached from23f3a031/4fdf1686 to code408ca8b5 for current SDK/bridge verification. Original source binding of the passing group was unchanged throughout; implementation committed after terminal0. Proposed SD01/02 prepared in writer and scratch spec-delta-r2.md, not published. Main's unrelated docs/plans/design WIP untouched. T02 and remaining T03 controls remain open; no execution_done/review/landing claim.


- stage: work | plan_id: PLAN-949 | plan_revision: r2 | round: 18 | outcome: pass | status: execution_done4/4 | worktree: D:/autostack/.wt/lang-949/auto-lang | branch: codex/plan-949-dev | code_commit: 408ca8b5096df96d2a91831ea36ef25336781889 | spec_candidate_commit: 40ab612ea | next: independent final review on this same tree
  T02/E01/AC01/02 actual public iced desktop: candidate root916a1426, UI d2a9ba65, Man03bb65ef, Lang408ca8b5; immutable SDK r167 built three programs exit0 (CLI60.01s/tool59.56s/host50.47s). SHA CLI1cf419bc546a789f09dfc95f3e5e7b1e742014694375908a14e44ecfe21a68d6, toolbcc481f588528d10efc58c515a583d482872df85e43907aa85ff40e1792caf19, host7be01796f593ff2d0c487e9e3174f555fb0afbc062eee48c37c32261e8cd5bcb. This is not old23f3 heuristic output. Replayed the actual pre-fix CLI probe: string-true="true", bool-true=true, get-bool=true, string-false="false", string-null="null", getter-string="true"; exit0.
  Named949 VM probe only: temporarily selected Todo launch=vm in owned candidate manifest, no pac/app-source change and no global AUTO_VM_MERGE/AUTO_BACKEND override. Default dashboard hatched Todo as VM app7 and launch promoted it to wid1. Per-app VM API back-proxy executed original api.at; no Rust Todo server or API/DB prewrite. Empty before-state/DB; actual stable-id013-todo StoreInput("U949-真正VM新增-中文54.25-r2")+AddTodo -> real AddTodo Parked HttpRequest4, POST; followup HttpRequest5 GET; resumed to completion132.1604ms. Actual screenshot inspected: new text visible in list, input placeholder restored, 1itemsleft and dashboard1. Snapshot state input="", active_count1/action_err=""; independent read-onlySQLite exactly[(1,marker,0)]. PNG SHA86f9c7081f0c2d315e107edda34e03dea91080ac2066cc19bb17d64b78059993; oracle e34e529022852d15908e80e3668dcf1257c69c8184a8c73f9e1cb3df0ea4dfdf in plan949. Initial snapshot-name auto-todo used as input target queued but executed no handler; excluded, corrected to actual stable dispatch ID. Snapshot selector remains auto-todo, source-template child nodes are not treated as rendered-list proof; actual PNG provides that leg.
  Normal app bus shutdown: wrapper finally restored manifest byte SHA2036c3983b98c783f845baabdb15bda210305db79942bca97341eb1c2d1cb137 and returned0; product logs host exit0, Chat service graceful0, owned private helper34936 controlled-cancel1/reaped. All captured owned processes gone and9247/17823 absent; independent Blog47236 original10:15:07 identity preserved. OS Get-Process receipt exit fields are null (handles were not retained early), so no individual CLI/Cargo exit0 claim; product/wrapper terminal0 and actual resource disappearance are the bounded evidence. Raw normal-stop-r2.json/owned-before-stop-r2.json retain this limitation. This VM profile does not replace parent054's separate accepted Rust Todo profile.
  T03/R01/AC03: UI writer286e0397 with its clean detached Lang408ca8b5 dependency, `cargo nextest run --locked -p auto-ui-runtime --lib --features mpv-widget -E 'test(/^ui::vm_bridge::/) or test(/ui_tests::plan711_cpu_slice_tests::/) or test(/ui_tests::plan437_child_init_tests::/)' --no-tests fail --no-fail-fast`: exit0, selected/executed79/pass79, unselected2344 not pass; compile92s, run5.825s. Covers actual plan702_handler_park_reentry_resume_roundtrip, init snapshot/child completion, cpu-slice parity/guard/error cases. Log SHAd3d134ef5207709690ebf748a41d9cc29441c4881d55dc8ffd43a8d1f9027e28. Full fixed VM/HTTP970 already passed r17 unchanged implementation, not repeated for documentation. SD01/02 prepared40ab612ea, no legacy ledger/index writes. plan949 scratch registered in-flight in retention registry; parent054 common SDK/review/merge remain separate and incomplete.


- stage: review | review_scope: final | reviewer_identity: independent delegated context `/root/review_lang949`, separate from executor; actual model/session ID not exposed, no fabricated identity | plan_id: PLAN-949 | plan_revision: r2 | outcome: needs_fix | next: work/T-01,T-03 in the same tree; no merge
  - Baseline: `D:/autostack/.wt/lang-949/auto-lang`, registered via `git worktree list --porcelain`, branch `codex/plan-949-dev`, clean at entry and verdict; reviewed_commit `8e242408ce452658f64a0ab26299405fcab07306`; implementation `408ca8b5096df96d2a91831ea36ef25336781889`; candidate Spec `40ab612ea08e368296eede587c685682e519ae6c`; diff base `6fae9d3c8c7063eff14fb0a4db376bb1a805c2b4`. Full original Plan diff inspected, including earlier back_prefix/base selection and env-gated diagnostic changes, not only r2 JSON patch. No implementation or Spec writes by reviewer.
  - Dependencies/evidence binding: existing UI bridge controls UI `286e039744a3ebd6a8e726cbe54f062df2c775fe` and its clean detached Lang `408ca8b5096df96d2a91831ea36ef25336781889`; actual Todo runtime root916a1426/UI d2a9ba65/Man03bb65ef, Todo source153710720964c9c4b4fcb302b5f401474f1d606c and frozen SDKr167. Reviewer CLI SHA256 independently matched `1cf419bc546a789f09dfc95f3e5e7b1e742014694375908a14e44ecfe21a68d6`. No group auto-down exists; fallback main root resolved read-only at `58ea33f699efa5cb1194473a2d45dcec51006842`; this is an observed dependency identity, not an assertion that full tests were freshly rebuilt against it.
  - AC-01 → T00/T02: **pass on frozen baseline**, independently read raw Todo logs2967..2975 (AddTodo→HttpRequest4 POST→HttpRequest5 GET→resumed132.1604ms), original app entry/config and owner wrapper. Existing segment mechanism and UI controls assertions inspected; no GUI or global pipe touched. AC-02 → T01/T02: **partial/fail**: actual visible PNG independently inspected (new Chinese54.25 marker, empty input and1itemsleft), before/after state inspected and independent read-only SQLite query returned exactly `[(1,"U949-真正VM新增-中文54.25-r2",0)]`; language value encoding fails F949-01/02 below. AC-03 → T01/T03: **partial/fail**: reused unchanged full VM/HTTP970 and UI79 controls after inspecting original selected cases/atomic result/51 valid bound children, plus code assertions for init/reentry/cpu slice; J01..03 miss representative scalar-collision and named-argument cases.
  - Accepted existing evidence: `fixed-vm-http-r2/results.json` SHA256 `b411ecc9b73c2012596d948cddff70e5eaddc31527af69fa8a325cd5b919ad78`, `cargo t vm http` exit0, selected/executed/pass970, original excluded434 (not PASS), no errors/not_run,51 bound calls valid; all five p949 cases actually selected. `ui-bridge-controls-r2.log` SHA256 `d3d134ef5207709690ebf748a41d9cc29441c4881d55dc8ffd43a8d1f9027e28`,79run79pass2344skipped. Existing config matches test-groups SHA8ac1f4a0592798b55dc655fb2fd339659f86a8a1ab5d46d21001ba097524b9f9; nextest-full SHAf0de57eb9eba703580c81b9555f3a87bb3d7d2a8832dbd04fceb3796a55b4967. These totals do not override new direct production failures; no repeated complete group/global suite.
  - **F949-01 (P1, original r2 requirement gap, T01/T03, AC02/03): plain integer misclassified as existing document.** Existing rule §2/J01/SD01 and stdlib encode[T]: actual Number remains JSON number independent of unrelated allocations. Repro source `review-number-collision-controls.at`: `print(Json.encode(4000000)); let first=Json.parse("{\"ok\":true}"); let second=Json.parse("[1,2]"); print(Json.encode(4000000)); print(Json.encode(4000001))`. Expected first/second numeric output4000000 and final4000001; observed4000000,`{"ok":true}`,`[1,2]`. Execute `D:/autostack/plan-artifacts/plan-054/t04/frozen-sdk-r167/auto.exe D:/autostack/plan-artifacts/plan-949/review-number-collision-controls.at` from writer; exit0 is a product wrong-value result. Established cause: `strings_json.rs::json_doc_heap_id` accepts TAG_I32 contents as heap IDs and encode prioritizes it; reused `http_server::nv_to_json` also probes i32 contents. Ordinary number4000000 before allocation and string"4000000" controls stay correct; direct getter-root numeric controls4000000/4000001 remain numbers. Scope is actual numeric inputs versus legacy handle compatibility, not a new size promise; this mechanism predates r2 but violates its explicit type-preservation claim. Minimal closing oracle: true i32 numeric4000000/4000001 before/after allocating live object+array remain numbers, typed documents/string-list compatibility remains correct, ordinary strings and repeated document uses remain unchanged; independently examine ownership/stake behavior when i32 contents equal a live heap ID. Do not blindly remove legacy handle support or move the identical heuristic to the fallback converter.
  - **F949-02 (P1, repair omission, T01/T03, AC02/03): named encode argument skips direct builtin getter normalization.** Existing rule §2/J02/SD01 says actual builtin getter expression directly consumed by builtin encode is restored once, with no positional-only restriction. Valid production source `let doc=Json.parse("{\"ok\":true,\"s\":\"true\"}"); print(Json.encode(value: true)); print(Json.encode(value: "true")); print(Json.encode(value: Json.get(doc,"ok"))); print(Json.encode(value: Json.get(doc,"s")))` yields `true`, `"true"`, `"true"`, `"\"true\""`; expected `true`, `"true"`, `true`, `"true"`. Entry same frozen CLI with `review-named-argument.at`, exit0. Controls prove named args reach actual encode; getter text is incorrectly encoded again. Established cause expr_calls.rs normalization checks `Arg::Pos(Expr::Call(...))` only, while native argument compiler accepts `Arg::Pair`. Minimal closure: positional/named get and get_at across bool/null/string/array/object, side-effectful input evaluated once/in order, shadowed ordinary receiver and plain function-returned strings unchanged; no global provenance or content guessing.
  - Raw independent probes live only `D:/autostack/plan-artifacts/plan-949/`. Collision controls source SHA256 `4b3792189e4bc436e942e2a76d7b2270f81396b44a6cd2189e15a1ebd6f0bf7e`, log `026c4de9e113f153f1a53bbe24cec6206c6f264937f048b981af531e1d4f6868`; named source `b64a4dbc0e17b0e36a874e1fde84590cdc1c5dcd4c8b3e5d18b4f6911047bb3a`, log `b7caf05fe946b9107230e905e39b36a6a735f6b2427d1547e52525ea4f359b28`. One exploratory `review-static-shadow.at` did not establish a valid user-method production path (direct call already returned empty), so its null output is not classified as a finding and creates no new promise.
  - Knowledge delta: SD01/02 candidate text reviewed, canonical source not published. Candidate API contract SHA256 `0e36f65651af20099fb7fecb67688022e3ff32dddcb3305bb612be2420b1c046`, back-proxy `7f552b302e3bc121126f79c4f1db23bd20234de21903cf0307eec6c27767535a`; frozen scratch spec-delta-r2.md SHA256 `30c251d048111eacc4621c5cddc998c4c59db733898c0bea30b9b2dd4c917e95`. Targets/owner split/RAII behavior align with actual code; SD01 type guarantee currently fails both findings. No legacy JSON/index writes. `new_spec_components: []` because both operations modify existing Specs; touched goals G1/G2/G3 are existing §1 IDs. Fixing current omissions need not weaken/increment r2 semantic contract.
  - Boundary/route: inspected native tags/BigInt nested conversion, existing string-list path, original getter ABI, wrapper ownership guards and shadow checks; independent production counterexamples cover sibling/failure paths beyond executor's five examples. No claim of exhaustive resource-object/generic ABI coverage. T00 and actual T02 evidence retained; T01/T03 reopened, current_step2/4. Their downstream final SDK/runtime evidence must be assessed for validity after repair, rather than rerunning every family automatically. Frozen affected union for future **merge-owned pending** gate is Lang `vm http` plus separately UI-owned vm_bridge/Init/cpu-slice controls and necessary public Todo integration; unify with UI060/parent054 consumer groups at merge. Current failing implementation cannot enter that gate or land. No merge/push/cleanup/new tree, other Plans untouched.

- stage: work | plan_id: PLAN-949 | plan_revision: r2 | round: 20 | outcome: execution_done | findings: F949-01/F949-02 repaired | code_commit: 68dfbc864 | next: independent affected re-review
  F949-01: encode consumes the existing stack ownership shadow before decoding. Plain i32 with no owned stake is encoded as a number without heap lookup or releasing an unrelated live object; legacy owned i32 string-list handles retain their existing ABI. Tagged document/primitive ownership and fallback marshalling remain guarded. No new provenance table or global String/list ABI change. Permanent single-VM controls cover 4000000/4000001 before and after live document/array allocation, numeric local/function/record/array paths, continued live document and keys/string-split list use, and numeric strings.
  F949-02: the already resolved builtin encode/getter normalization accepts both positional and named Pair argument forms; receiver shadow protection and single evaluation remain. Getter/get_at matrix now covers both forms across 17 values (68 real compile/VM executions), with side-effect, plain function-return and shadowed receiver controls.
  Actual permanent seven-case red: 4pass/3fail (review-repair-red-r2.log). Final seven cases: 7pass/0fail/0ignored, execution2.91s (review-repair-final-r2.log). Changed VM responsibility fixed group `cargo t vm --out D:/autostack/plan-artifacts/plan-949/review-repair-vm-r2`: actual exit0, gate_passed=true, executed922, results SHA256 d7f6c7e4efb5ac36ed6a0b22bbc110b10751dc854bfbe0497f0703c1008a113d. Source frozen at68dfbc864 during this run; original configuration/resource/ignore rules unchanged. SD01 now states ownership identity and named-argument boundary.
  T00/T02 and unchanged scheduler/UI controls from round18 remain identified evidence, not a claim that SDK167 includes this repair. Final consumer SDK, necessary Todo integration and frozen merge-owned vm/http/UI affected union remain pending at delivery; do not rerun unrelated product families solely for a documentation descendant. T01/T03 implementation and work gate now closed, 4/4 execution_done; independent review still required. No merge/push/archive/cleanup performed.


- stage: review | review_scope: final | reviewer_identity: same independent delegated context `/root/review_lang949` as prior final review, separate from repair executor; no model/session ID exposed | plan_id: PLAN-949 | plan_revision: r2 | outcome: pass | findings: F949-01 closed, F949-02 closed | next: merge-owned frozen affected gate, then authorized delivery; no landing performed by review
  - Frozen baseline verified via worktree registration/status before and after review: `D:/autostack/.wt/lang-949/auto-lang`, branch `codex/plan-949-dev`, clean HEAD `d73b1295f58871cd43baf1bb38b9dafafc78798e`; repair implementation `68dfbc864a8785d9f3893ca8427d15cd750fa474`, repair diff base prior reviewed `8e242408ce452658f64a0ab26299405fcab07306` (whole-Plan base remains6fae9d3c8c7063eff14fb0a4db376bb1a805c2b4). No implementation/Spec edits, consumer movement, merge/push/cleanup or GUI access by reviewer. Only main authoritative Plan changed.
  - Scope/check→invariant: independently inspected `take_stake_at` transfer/clear, `pop_arg_nv` slot clearing, rc_push_slot's established PLAN-667 scalar-versus-owned-handle rule, consumed StakeGuard and CALL_NAT dead-slot settlement. Unowned i32 returns its numeric encoding before either document lookup or nv_to_json's legacy i32 lookup, and creates no heap-release guard. Owned legacy i32 uses the transferred owned_id guard; tagged heap/string values keep the existing guard and result slot owns only its new string. Fallback restores the transferred stake. This reuses existing ownership metadata, with no new identity table/global type guessing. Named argument change handles Arg::Pair alongside Pos after builtin encode native-ID resolution and retains existing receiver shadow guards; does not recompile or reevaluate getter expressions.
  - Independent actual execution: built a scratch-only driver against writer's current `target/debug/deps/libauto_lang_core-0d6b36e179a9bf1e.rlib` (SHA256 `11bb003b2037087824eb10406cbf6b70cdfd29ab56878e9bac911e418adfe788`) using existing native outputs, calling production `execute_with_engine_capture(AutoVM, source)` (real parse/codegen/native VM). Driver SHA256 `4fdbd6b9ff95494d317fcc7656ce667cec02605a008db43a9913a570e1ae4db5`; this is not stale SDK167. First direct rustc link lacked windows.0.52.0.lib; supplied existing cargo registry native lib path and linked successfully, an infrastructure correction with no product failure/implementation change.
  - **F949-01 closed:** replayed unchanged `review-number-collision-controls.at` (original source SHA4b3792189e4bc436e942e2a76d7b2270f81396b44a6cd2189e15a1ebd6f0bf7e) against repaired production engine: exact eight output values/types `[4000000,4000000,4000001,"4000000",4000000,4000001,{"ok":true},[1,2]]`, exit0; no more object/list substitution. Added independent sibling input `review-repair-siblings.at`: keys alias and List<str> function parameter, repeated keys and still-live doc encodes, numeric function argument, computed integer4000001, negative integer/zero, named nested array getter and escaped quoted-string get_at; all eleven exact JSON values/types pass. Thus sibling checks cover ownership copying/call transfer/repeated use beyond the executor's original example. Source SHA256 `8cead9aabd6190c59122dfb7c4d7666566bc6adc63b48b0a56168d4ebcf264c8`, final log535fe91a021a990378115aab22e2a498e1810519ff1b00d5f7adcfa9b3bc5a79. No claim of exhaustive arbitrary-resource or allocator-scale support.
  - **F949-02 closed:** replayed unchanged `review-named-argument.at` (original source SHAb64a4dbc0e17b0e36a874e1fde84590cdc1c5dcd4c8b3e5d18b4f6911047bb3a), exact output values/types `[true,"true",true,"true"]`, exit0. Independently reran committed seven `p949_tests` through current writer's `auto_lang_core-078bff18ed265d8d.exe p949_tests --nocapture`:7pass/0fail/0ignored,1637filtered,2.67s; inspected actual assertions, including 17-value get/get_at×Pos/Pair68 real executions, side-effect once, plain function strings and shadowed receiver controls. These close both old examples and shared invariants, not merely a renamed helper assertion.
  - Replay evidence all in `D:/autostack/plan-artifacts/plan-949/`: `review-repair-independent-oracle.py` checks exact parsed values AND Python JSON types (avoids true==1 masking), returns0; original collision final log SHA2569d04f721c953c817a8053afffbb4057bbfc7159b0032ab4b7b1f14dc7b0773d9; named final logc0cc14b488bb588a429dda730967d49f4e5fe407c6d9faae28cef96864b587f3; independent-seven log734bfb3d7b0e633bf0755469e46a08f983d5cbdd349e6f1fa4acc22d277a45c6. No raw evidence committed.
  - Accepted affected work-group evidence independently parsed: `review-repair-vm-r2/results.json` SHAd7f6c7e4efb5ac36ed6a0b22bbc110b10751dc854bfbe0497f0703c1008a113d, groups=[vm], gate_passed=true, selected/executed/pass922, excluded434 under original conditions (not pass), not_run/errors empty,51required bound children valid; includes actual RC lifecycle atomic results. Configuration still SHA8ac1f4a0592798b55dc655fb2fd339659f86a8a1ab5d46d21001ba097524b9f9. No repeated full group or whole-repo run by reviewer. Unchanged prior HTTP/UI scheduler controls remain identified historical evidence; this review does not relabel SDK167 or those consumer builds as containing68dfbc864.
  - Acceptance map: AC01→T00/T02 **pass** (prior independently confirmed real Todo Parked4/5→resumed132.1604ms; scheduler/transport not changed by repair). AC02→T01/T02 **pass** (language scalar/getter gaps closed by independent real executions; prior actual PNG/inputclear/read-onlySQLite oracle retained). AC03→T01/T03 **pass** (J01..03+both findings closed; current full VM922 including RC, prior unchanged Init/cpu-slice79 controls retained). All four work tasks complete; semantic r2 contract unchanged. Updated shared SDK/public Todo closure on final delivery baseline remains an explicitly merge-owned integration check, not a claim that old binary already contains repair.
  - Spec delta final review: existing SD01/02 paths and goals unchanged; API contract candidate SHA256 `c9b70c04d4c3f4e39f9ff76aad915d81ccf7cd188804647d4df3dd7e6bff876f` now documents existing stake identity and both argument forms consistent with verified behavior, not a weakened rule. Back-proxy candidate SHA256 `7f552b302e3bc121126f79c4f1db23bd20234de21903cf0307eec6c27767535a` unchanged. No new Spec components; G1/G2/G3 remain touched. No canonical publication or legacy JSON/index write during review. Code diff whitespace check exit0; existing unrelated compiler warnings are unchanged.
  - pending_closeout_gates: **merge-owned frozen union still pending**, Lang `vm http` (reuse current VM evidence only if its complete source/dependency/config/input identity remains valid), UI-owned vm_bridge/Init/cpu-slice controls on chosen common Lang revision, final common SDK and necessary public Todo integration; combine with UI060/parent054 owned groups at merge. Pass→reviewed grants acceptance readiness only, and cannot authorize landing with unmet group/integration gates. Original obsolete diagnostic hypotheses and both prior finding records remain history.


- stage: review | review_scope: final (post-rebase integration) | reviewer_identity: original independent `/root/review_lang949` context | plan_id: PLAN-949 | plan_revision: r2 | outcome: pass retained and rebound to integrated baseline | reviewed_commit: f03226e1063e191af61dd119bc066ec60bf848ca | base_commit: 8b9c3813c9706aefcfe45b5ba16212b669ac23a3 | next: remaining cross-repository merge-owned integration; no merge/push by reviewer
  - Entry/verdict: original registered writer `D:/autostack/.wt/lang-949/auto-lang`, branch codex/plan-949-dev, frozen/clean f03226e1063e191af61dd119bc066ec60bf848ca. Independently recomputed 949's old diff6fae9d3c8→d73b1295f and new diff8b9c3813c→f03226e10, restricting all crates/Spec files: exactly10files' added/deleted patch lines equal, canonical JSON SHA2567743e8c4701f23b9dbd77d75f957b0fd810c03550b5531d7d0c7f1cb0f1bfe48, matching existing rebase-equivalence-r2.json and range-diff. Plan-only historical conflicts retain current authoritative main Plan; no lost implementation or Spec delta. Later master changes beyond8b9c inspected as no crates/test-group changes at this audit. Worktree unchanged by reviewer; only main Plan updated.
  - Upstream dependency audit (oldbase→8b9c): inspected actual relevant diffs, not just equal Plan patch. `convert.rs` adds checked `ListData<Value>` integer decoding to Vec<i32> (reject bool/float/out-of-range), leaves String marshalling and stack ownership helpers intact. `codegen.rs` corrects inferred type of dynamic json.parse local to Unknown, preventing `.bytes` from being incorrectly dispatched as str.bytes; does not alter encode argument consumption. `strings_json.rs` upstream only delegates UUID generator to existing a2r_std implementation, outside JSON wrapper. `a2r-std/json.rs` adds JsonValue alias; serde encode/parse functions unchanged. Remaining Reader/dialog/zip/assembly/import changes do not replace 949's encode/ownership/scheduler paths. The shared-list and dynamic-field changes justify current-source execution below, rather than automatic reuse of old binaries.
  - Necessary independent minimal controls: relinked scratch `review-rebase-driver.exe` against current writer `libauto_lang_core-f501db547a858294.rlib` SHA256c84ef4b8c4afd88dde4ae3fe96cbceff95e42feb6fbe0d5299361b94745b0c30; driver SHAb84892b13f9c933c1f4c6ed602a9283ffffdfd4ee526344cbb25d1ff231d57e1. Same production parse/codegen/AutoVM entry as prior review; no Cargo invocation or consumer update by reviewer. `review-rebase-independent-oracle.py` reruns unchanged original collision8/named4/ownership-siblings11 exact parsed JSON outputs and types, all exit0/pass23. Added `review-rebase-dynamic-field.at`: actual `Json.parse` object field named bytes contains `[4000000,4000001,true,"true"]`; field→encode, named getter bool, keys list, same-value numeric4000000, and full still-live object all produce expected values/types,5/5pass. Input SHA256bf961e7364a80c167628296df54c40e298d89f674dc91ad45421e45343f507f5; log6ccf5e12f5f88ae359ba501fdf0b54de3a395e1e975f987d27c7627081949eb4. Unchanged original replays retain exact previous log hashes9d04f721…/c0cc14b4…/535fe91a…. All material remains in plan949 scratch, no implementation test edits/raw evidence in Git.
  - **Lang merge-owned fixed union complete on f032:** independently parsed `D:/autostack/plan-artifacts/plan-949/merge-fixed-vm-http-r2/results.json`, SHA256be2b5d89f5b2c64df76f48c0019b802696888396eba713cb091a9f865795b51a; selected groups vm+http; config SHA33ee32cc65b074a55cceba5b4a181ffbd502b398aa4dca7da95b89fa0524343d; gate_passed=true, native command exit0, selected/executed/pass987, original-condition excluded434 (not pass), not_run/errors empty,51required bound invocations valid, wall537.728045s. Selection comparison against old complete970: no removed cases,17 additions (two949 repair cases, ten988 integer bridge cases, four988 dialog controls, oneReader parsed-kind parity). All7current949 cases selected and successful, including direct Pos/Pair and ownership controls; no mock/zero-selection result. Original conditions/resource rules retained. Reviewer reused this completed real group without mechanical repeat; prior922/970 records remain historical, not claimed to cover new dependencies.
  - AC map retained with new integration binding: AC01→T00/T02 pass (scheduler contract unchanged; historical independently checked actual Parked/resume retained); AC02→T01/T02 pass (value/legacy compatibility independently replayed on current code/deps; prior visible Todo/DB evidence retains its exact old runtime identity); AC03→T01/T03 pass (current987 includes7JSON controls plus originalVM/HTTP duties, independent28outputs confirm affected invariant). F949-01/02 remain closed. SD01/02 candidate bytes unchanged from prior final review (API SHAc9b70c04d4c3f4e39f9ff76aad915d81ccf7cd188804647d4df3dd7e6bff876f; back-proxy SHA7f552b302e3bc121126f79c4f1db23bd20234de21903cf0307eec6c27767535a). No plan_revision change, weakened AC or new debt; reviewed4/4 remains.
  - Remaining merge-owned union boundary: UI060/common UI baseline7de47df5 and frozen C2 Lang f032 consumers, common SDK and necessary actual Todo integration are still pending owner validation. No claim that existing SDK167 or old79UI controls ran on this rebased dependency set. Lang987 gate is valid only for its frozen source/dependency/config/input identity; later meaningful changes require affected revalidation. Final pass here does not close parent054 or permit landing across an unmet cross-repository gate.


- stage: review | review_scope: final (SDK176 actual Todo integration evidence) | reviewer_identity: original independent `/root/review_lang949` context | plan_id: PLAN-949 | plan_revision: r2 | outcome: pass retained; named common SDK/Todo integration now verified | reviewed_commit: f03226e1063e191af61dd119bc066ec60bf848ca | next: remaining cross-repository final gates and delivery owner; no source publication/merge by reviewer
  - Source/SDK binding independently read: SDK176 source-roots selects Lang `f03226e1063e191af61dd119bc066ec60bf848ca`, UI `a1db3d7940c344c69cea08cf469b562d15a31eb8`, Man `2a9b16e50ec60bf8e591fcf5dae14226567ac32c`, OS host `5e186935b716aa279adfa6636ae4972c16de1112`. All three SDK binary hashes independently recomputed matching original results: CLI84ffcd0782475f4a9f77ac626042c7a1354b23270930b985531f74212602425b; tool56e50b44b3ff33c44a7fd61c1860e0fefeaa0efe7dd55fd6d1e7c9cf60382ea7; desktop1b701a71c68c800e5acae1f20eb1987b73ea66da61d51a8c9403871de3a61988. CLI/tool are explicitly reused r174 builds with exactly unchanged Lang/UI/Man inputs; OS lock change does not enter those workspace builds; desktop was built at5e, all recorded build exits0. Actual runtime OS `fa93c48b44ced8a8ea4592dee30ceae179bf067a` diff from5e independently inspected: only three app gitlinks and assembly Spec, zero host-code/manifest changes. Lang writer still clean f032, consumer Lang/UI/Man clean at these same selected revisions.
  - Todo source pin `e915894d57d739719e06f016488f89989f68e843` matches actual candidate gitlink. Compared old runtime source1537107→e915 over src/pac: only `repository.at::fs_delete` wrapper return annotation bool→int, no API/handler semantic rewrite. This app-owned compatibility change does not change 949's Lang987 gate identity. SDK source-roots/results local paths in parent054/t04, SHA256 respectively8fa028582c9018e8dbcf7d0a954d76ad9365e0e592b81e93d8dc24efda5b534e/78af58f35a06cdd5e183222c277797e92efd79ab2d358261c4a03027d2195278. No assertion that whole parent054 is accepted merely because these programs built.
  - Actual frozen CLI original findings controls: independently parsed both raw logs, checked original source/log hashes and exact JSON values+types against original 8numeric/4named expectations:12/12pass, actual command exits0/0. `plan-054/t07/typed-json-cli-r176.json` SHA256adc9245b1d99fe78cfa81ff8ac53805b96fe7461e3eb5436f65419803c4657b8, actual CLI hash above. F949-01/02 remain closed on the shared executable, not only the scratch production engine driver.
  - Actual Todo VM front/back r3: read run/execute/stop scripts and raw before/after/log, independently viewed `todo-vm-visible-r3.png` (SHA256c84e5913bd5644859bafee4cac37157da2ba35ffc6ad8b3c75553f474aa1696b). New marker `U949-真正VM新增-中文54.25-r3` is visibly in the rendered list; empty input placeholder and1itemsleft shown, dashboard1. Before todos=[]/active0; after input=""/active_count1/action_err=""/preview1=new marker. Raw log contains exactly one StoreInput and one AddTodo handler; real AddTodo Parked HttpRequest4→POST /api/todos→HttpRequest5→GET→resumed124.8048ms. Initial unpublished-surface retry sent no input; repeated launch only promoted the same app (image has two Todo windows), not a second AddTodo/data mutation. Independent SQLite mode=ro query found exactly `[(1,"U949-真正VM新增-中文54.25-r3",0)]`. Source-template snapshot children are not counted as rendered-row proof; independent PNG supplies that leg. No HTTP/DB prewrite or Rust Todo backend: original api.at runs in VM per-app proxy.
  - Wrapper temporarily changes only candidate Todo launch native→vm with explicit r3 isolated data root and SDK176; clears global AUTO_BACKEND/AUTO_VM_MERGE overrides, restores exact manifest in finally. Both run receipt and current read-only file hash match2036c3983b98c783f845baabdb15bda210305db79942bca97341eb1c2d1cb137, wrapper exit0. Stop script binds root30808 path+creation time, captures process handles before graceful app-bus shutdown and waits/reaps owned descendants; original terminal records root/wrapper/CLI/Cargo/service0, private helper41708 actual controlled-cancel1, all reaped. Script checks ports9247/17823 absent and foreign Blog47236 original2026-11-06T10:15:07 identity unchanged. Reviewer inspected original terminal records and product shutdown log, did not touch live GUI/global pipe/processes or relabel helper1 as0.
  - Evidence local hashes (plan949 scratch): todo-task-oracle-r3.json10068da1db66ec2398c15d4fe8a1242a4312344b20ffc13094398d0cb1d00070; raw todo-vm-r3.err.log6b5219b5edd2a0d4946eecfe237df6448381672eb4dcdafa7eac2da9478f24ec; normal-stop-r3.jsonbab4c0b2feda9dc259242d1cbeac547d39ac67063dd6d5149711d7d6efae0a18. Before/after image hashes checked against original oracle. Raw evidence stays scratch; this concise receipt is durable. No new implementation/Cargo/test execution, consumer update, Spec publication or unrelated WIP edits by reviewer.
  - Acceptance/gate scope: AC01→T00/T02 actual non-Init park/resume pass on SDK176; AC02→T01/T02 actual input→visible list→cleared input→same persistence plus actual CLI type controls pass; AC03→T01/T03 retains independently accepted7permanent cases and real Lang vm/http987/987. Existing merge-fixed-vm-http-r2/results.json independently rehashed unchanged be2b5d89f5b2c64df76f48c0019b802696888396eba713cb091a9f865795b51a (51valid bound invocations/config33ee32cc…); unchanged frozen Lang/dependency/config/input means reuse, no repeated Cargo gate. Canonical Specs/949source unchanged; reviewed4/4 remains. **Remaining pending** is cross-repository final UI affected union on selected UIa1db/common baseline and parent054's other Vue/native task/overall gate plus actual delivery/archive/cleanup. This closes the named949 SDK/Todo integration gap only; does not make pending cross-repository gates green or publish source.

## 10. 待澄清事项

F949-01/02 已关闭，f03226e10 整合 final pass，reviewed4/4；Lang vm/http987/987与 SDK176实际 CLI/Todo VM集成已独立核验通过。最终跨仓 UI受影响并集、父054其余任务/总门、默认分支交付、归档与清理仍待 owner 完成。
