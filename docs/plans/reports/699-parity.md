# PLAN-699 对拍与回归矩阵（T-07）

> 基线：master `d119881c5` → 执行期 rebase 至 `4dc4d581a`；worktree
> `D:/autostack/.wt/lang-699/auto-lang`（`plan-699-dev`，终链
> `d2de4d01b` → `01cd23192` → `6de714eeb` → `d748e59be`）。
> 验证二进制：`cargo th`（test-http-e2e，串行真 TCP）；门禁数字均为 2026-09-24 本机实测。

## 1. AC-06 语义面 → 证据映射

| 语义面 | 证据（全部在新传输上重跑） | 结果 |
|---|---|---|
| 015-notes 真实 CRUD | `e2e_plan696_real_015_notes_crud_parity`（真实 `examples/ui/015-notes/src/back/api.at`；列表/创建/持久化） | PASS |
| 017-chat 真实 SSE+CRUD | `e2e_plan696_real_017_chat_crud_parity`；SSE producer/断连回收另有 696 双测 | PASS |
| 023-realworld 认证面 | `e2e_plan696_real_023_auth_parity`、`e2e_b1_realworld_token_auth`（meta 参数绑定/token） | PASS |
| `Response` 对象（status/headers/redirect） | `e2e_a_redirect_302_with_location`、`e2e_musk_response_constructors`（302/Location、自定状态与头经 ApiReply 头透传） | PASS |
| 参数绑定（by-name/path/query/缺参 400/typed） | `http_e2e_api_missing_param_400`、`http_e2e_api_typed_query_int`、`http_e2e_api_query_by_name`、`http_e2e_api_post_body_by_name`、`http_e2e_api_raw_body_single_param`、`e2e_int_path_param_handler` | PASS |
| multipart（字段+文件） | `e2e_b6_multipart_upload_field_and_file`（同一 framed body 进 parse_multipart，语义零改动） | PASS |
| CORS preflight/响应头 | `e2e_value_accessors` 等（preflight 短路在 dispatch 保留；响应经 typed header 写出） | PASS |
| 限流 429（Retry-After+请求 id） | `e2e_zz_rate_limit_429_after_quota` | PASS |
| request id（生成/透传） | `e2e_b6_request_id_generated_and_echoed`、`e2e_b6_request_id_incoming_passthrough`（断言改大小写不敏感——hyper 规范化响应头名为小写，值不变） | PASS |
| 中间件短路面 | dispatch 内 Plan 352 链逐行保留；**公共 `.at` 面缺口（预存）**：`http.server.middleware` shim 已注册（stdlib.rs:8735）但 `http.at` 未声明该 API，用户面无法触达——非本计划回归，登记待后续计划补声明 | 矩阵注记 |
| `__axum:` 合成路由（Router Builder） | dispatch 内 Plan 442/383 臂逐行保留（route_by_synthetic_name + push_extractor_args 未动）；仓内无 Builder 形态语料可走 serve_async，不虚构 .at 语法造测；axum_adapter 单测（path_conversion/extractor_kinds）在档 | 矩阵注记 |
| 合并/IPC 抽样 | VM merge 为进程内函数调用，无 HTTP status/header 面，本计划未触碰其语义（计划 §1 非目标）；split 模式走同一 #[api] 服务器（上表 CRUD 即其服务端） | 抽样确认 |

## 2. 新传输协议探针（本计划新增 E2E，全绿）

| 探针 | 断言 | 耗时 |
|---|---|---|
| `e2e_plan699_chunked_request_body_accepted` | chunked 请求体被解码并正确绑定（旧手写解析器直接拒 TE） | 0.6s |
| `e2e_plan699_keepalive_two_requests_one_connection` | 单连接顺序两请求两响应（旧服务器每响应后关连接） | 0.6s |
| `e2e_plan699_oversized_headers_431` | >64 KiB 请求头 → hyper 原生 431 | 0.6s |
| `e2e_plan699_slow_headers_timeout_close` | 慢头在 10s header 读超时处确定性关闭（实测 10.6s） | 10.6s |
| `e2e_plan699_injected_shutdown_releases_port` | 注入关闭旗标 → accept 停 → owner 排空退出 → 端口可复绑 | 0.6s |
| `bridge_tests::full_queue_rejects_with_503` | 队列满 → 立即 503 + Retry-After（桥边界单测，无时序竞态） | 0.07s |

## 3. 已知行为差异（设计内，均记录于 699-bridge-decision.md）

1. **chunked 请求体从"拒绝"变"支持"**（AC-01 要求的放松；上限 10 MiB 不变）。
2. **keep-alive 成为缺省**（旧服务器每响应后 `Connection: close`）；错误响应同样保持连接。
3. **响应头名小写规范化**（hyper 行为；HTTP/1.1 头名大小写不敏感，值不变——三个旧 e2e 断言已同步改为大小写不敏感匹配）。
4. **错误/边界的确定性提升**：task-slot 缺失（原实现挂起连接）现回确定 200 空体；SSE 头写失败边界的 handler task 也回收（原实现漏）。

## 4. 生成 Axum 轨

本计划零改动 `auto-man/src/api_gen.rs` 与 `back/api.at`（Cargo.toml 仅动 auto-lang 依赖表）；生成轨与 VM 轨是独立服务实现（SD-02 澄清的正是这一点）。015/017/023 的 VM/生成行为对拍基线由 PLAN-696 建立且其 e2e 在新传输上全绿复现，行为面无漂移信号。

## 5. 门禁汇总（2026-09-24 实测）

| 门禁 | 结果 |
|---|---|
| `cargo check -p auto-lang` | 0 error / 触碰文件 0 新告警 |
| `cargo check -p auto-lang --no-default-features`（AC-01 纯 VM 形态） | PASS |
| `cargo th`（56 测，rebase 后） | **56/56 全绿**（旧 base 的 back_proxy/031 环境红被 master `4dc4d581a` 的修复顺带收口） |
| `cargo tv`（语料 162） | 162/162 全绿 |
| `cargo tf`（5705 测） | 5704/5705：唯一红=plan358_d1 stress 并行抖动（单跑双绿定责）；同批 9 红在未改动 master 逐一复红=全预存，零本计划归因红 |
