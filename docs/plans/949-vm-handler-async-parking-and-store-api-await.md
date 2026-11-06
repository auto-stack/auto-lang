---
plan_id: PLAN-949
status: executing
feature_name: VM handler 异步返回闭合与 JSON 值编码
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 2
current_step: 1
total_steps: 4
base_branch: master
depends_on: []
supersedes_spec_components: [docs/specs/stdlib/design/api-transport-contract.md, docs/specs/auto-lang/vm/back-proxy.md]

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
- [ ] T-01（work，依赖T00）：收口既有 per-app语言上下文、实现值编码与片段消费修复、永久最小反例/对照。
- [ ] T-02（work，依赖T01）：最终 SDK Todo VM 用户任务、真实返回与磁盘独立核对。
- [ ] T-03（work，依赖T01/02）：既有 boot/reentry/cpu-slice 与完整受影响职责组回归、Spec候选、提交execution_done回执。

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

## 10. 待澄清事项

无新增用户决定。T01..03 尚未执行完，不把旧 round15 的“全链闭合”当作已复审交付；原 Init-only 假设已由实际非 Init Parked 反证，不再作为待用户决策。
