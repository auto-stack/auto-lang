# PLAN-699 验证报告（T-08）

> worktree：`D:/autostack/.wt/lang-699/auto-lang`（`plan-699-dev`）
> **rebase 记录**：原基于 master `d119881c5`（链 `5a77d1972`→`3420af7ed`→`e798df2d7`→`5764db3e2`）；
> 执行期并行会话在 master 落地 `4dc4d581a`（017/018 前端 HTTP 重构，恰好修复基线上
> 018 的坏样）——四提交无冲突 rebase 至新 master，终链（本报告终态证据基线）：
> `d2de4d01b`（T-01 spike+决策）→ `01cd23192`（T-02 dispatch 核心）→
> `6de714eeb`（T-03/04/05 传输+预算+SSE+关闭接线）→ `d748e59be`（中间件 e2e 撤销）
> → 本报告提交。
> 依赖位：`.wt/lang-699/auto-down`（auto-down master `3373a5c` detach，仅 path 依赖解析用，零改动）。

## 1. 交付摘要

`auto run --server vm` 的 `#[api]` HTTP 入口从手写 HTTP/1 解析/响应拼接迁至
Axum/Hyper 承载的协议层（Design 33 阶段 B）。`!Send` AutoVM 保持单 owner 线程
（LocalSet），网络层在专属 `auto-http-net` 线程以 hyper-util auto Builder
（http1_only + TokioTimer + 64 KiB head 缓冲 + 10s header 读超时）驱动，桥只传
owned `Send` 类型（`ApiRequest`/`ApiReply`），无 usize 洗指针、无 spawn_blocking
调 VM。资源边界：body 10 MiB（413）/body 总期限 10s（408）/在途队列
AUTO_HTTP_MAX_INFLIGHT=64（满载 503+Retry-After）/回复等待
AUTO_HTTP_REQUEST_TIMEOUT_MS=30s（503）/graceful drain 10s。SSE 帧通道容量 1
（背压同 696），断连/关闭经 FrameStream 收流并在 owner 线程回收 iterator/task/698
订阅。手写解析/写串路径退出默认调用图（已删除），模块头旧误述（"VM 与 a2r 共用
Axum、spawn_blocking 直接调 VM"）已纠正。

## 2. AC 对账

| AC | 结论 | 关键证据 |
|---|---|---|
| AC-01 协议层=Axum/Hyper；keep-alive+chunked；纯 VM 编译 | **过** | `handle_connection_async` 已删除（调用图核对）；`e2e_plan699_chunked_request_body_accepted`/`…keepalive…`；`cargo check --no-default-features` 0 error |
| AC-02 owned Send 桥；单 owner；无洗白/spawn_blocking | **过** | ApiRequest/ApiReply 纯 owned 字段（http_server.rs 桥类型段）；`Rc<AutoVM>` 只在 owner 循环；grep 面无 `as usize` 指针转运；spike+实现均无 spawn_blocking |
| AC-03 实际生效上限；满载快速 503；内存有界 | **过** | 431（`…oversized_headers_431`）、慢头 10s 关（`…slow_headers_timeout_close` 实测 10.6s）、413/408（to_bytes+总期限）、队列 503（`bridge_tests::full_queue_rejects_with_503`）、body 以 limit 收集内存有界 |
| AC-04 慢 SSE 不碍健康请求；断连/关闭回收 | **过** | `e2e_plan696_slow_sse_does_not_block_health`、`e2e_plan696_sse_disconnect_cancels_generator`（新传输重跑绿）；698 订阅句柄回收单测在档（plan698_f1_cleanup_tests）；帧通道容量 1+FrameStream 关闭臂 |
| AC-05 注入关闭；端口复绑 | **过（实机 Ctrl+C 受环境限制，见 §4）** | `e2e_plan699_injected_shutdown_releases_port`（stop accept→drain→owner 退出→端口复绑）；CLI Ctrl+C/SIGTERM 注册进同一 watch（serve_async） |
| AC-06 语义无回退；多后台文档准确 | **过** | `docs/plans/reports/699-parity.md` 矩阵：015/017/023、Response、绑定、multipart、CORS/限流/request-id 全绿；中间件/`__axum:` 注记如实；SD-01..03 delta 见 plan §5 |
| AC-07 门禁+复审准备 | **过** | §3 门禁表；债项/遗漏扫描见 §4 |

## 3. 门禁与基线

| 门禁 | 结果 |
|---|---|
| `cargo check -p auto-lang` | 0 error；触碰文件零新告警（349 全仓预存告警不含本 diff 新增） |
| `cargo check -p auto-lang --no-default-features` | 0 error（纯 VM feature 形态，AC-01） |
| `cargo th` 基线（T-01 时点，旧 base） | 50/51（红=back_proxy/031 预存，master 同样红定责） |
| `cargo th` 终态（**rebase 至 master `4dc4d581a` 后**，56 测含 6 新探针） | **56/56 全绿**（并行会话的 CRUD back session heuristic 修复顺带收口了 back_proxy/031 旧红） |
| `cargo tv`（rebase 后） | 162/162 全绿 |
| `cargo tf`（rebase 后，5705 测） | **5704/5705 过；1 红=plan358_d1 stress 单跑双绿（master 与 worktree 各自单跑均绿）=并行负载抖动，与 698 红册 "ffi/c1 抖动单跑双绿" 同型**。同批 9 红比对：musk_vm_track×6 + plan606 029 + projector_counter_layout + a2vue_desktop_surface 在未改动 master 上逐一复红=全预存，零 PLAN-699 归因红；native_gate_runtime_views_of_six 被 4dc4d581a 的 018 重构修复（两侧一致转绿） |
| `git diff --check` | 干净（无空白错误） |

## 4. 已知边界/残留（非阻塞，均已单列）

1. **CLI Ctrl+C 实机验证受 harness 限制**：本会话为 headless pty 控制台，三种
   投递方式（进程组 CTRL_C_EVENT / AttachConsole+GenerateConsoleCtrlEvent /
   CREATE_NEW_PROCESS_GROUP+CTRL_BREAK_EVENT）均未能使 tokio handler 接管
   （CTRL_BREAK 落默认处理器 0xC000013A 终止=handler 未注册的表象，事件本身已
   到达）。注入信号（同一 watch 通道的下游全路径）已 E2E 证明；ctrl_c(+unix
   SIGTERM) 接线与仓内先例同构（`vm/task_system.rs:732`）。人工终端复现：
   `AUTO_HTTP_PORT=8080 auto examples/ui/015-notes/src/back/api.at` → Ctrl+C →
   期望 stderr 出现 `Ctrl+C — shutting down gracefully` 与 `VM owner loop exited`
   且进程 exit 0、端口可复绑。建议复审时由有交互终端的环境补一轮实机确认。
2. **同步 CPU handler 无抢占**（计划 §10.3 既定边界）：队列等待超时 503 只解除
   排队关系；handler 可中断/多 VM 并行属 Design 33 阶段 C。
3. **VM 中间件公共面缺口（预存）**：`http.server.middleware` shim 已注册但
   `http.at` 未声明；dispatch 逻辑逐行保留，补声明属后续计划。
4. **`__axum:` 合成路由**：dispatch 臂逐行保留；仓内无 Builder 形态语料走
   serve_async，未虚构语法造测（axum_adapter 单测在档）。
5. **body 慢滴单字节节流**：hyper-util 0.1.20 Http1Builder 无 body 空闲读超时
   暴露，以 body 总期限 10s 兜底（超期 408 断开），无无界占用。
6. **预存环境红**：`back_proxy_tests::http_e2e_back_proxy_real_031_native_ns_session`
   （"names non-empty: []"）master 同样红，与 back-proxy/031 装载面相关，建议
   单独立案勘定（非本计划范围）。

## 5. 遗漏/延期/Workaround 扫描（复审输入）

- 计划 T-01..T-08 全部执行；无未批准延期项。
- Workaround 检查：`e2e_ports_unique` 守卫下新增端口 18770-18776 无撞号；
  三处旧断言的大小写不敏感化是 hyper 规范化行为的正确适配（非绕过断言）；
  中间件 e2e 曾尝试后撤销（公共面缺失，见 §4.3）——撤销而非造假测。
- T-06 的 CLI 实机验证缺口见 §4.1——如实呈报，未声称完成。
