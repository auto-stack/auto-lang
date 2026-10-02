# PLAN-729 双端一致性报告（parity）

- worktree `D:/autostack/.wt/lang-729/auto-lang`
- VM 腿：`http_e2e_plan729_vm_*`（6 族 9 测，真 TCP）
- 生成 Rust 腿：`http_e2e_plan729_generated_service_wire_matrix`（auto-man，真实生成产物
  api.rs/main.rs/types.rs + 临时 crate cargo build + axum 0.7 + 共享 worktree target 缓存）

## 1. 同一共享代码的投影

| 层 | 单源 | VM 腿 | 生成腿 |
|---|---|---|---|
| 描述符/严格 options/Range/条件/If-Range/HTTP 日期/响应头 | `a2r_std::http::server_file` | ✅ | ✅ |
| 受限打开/准入/pump/watchdog/收口 | `auto_lang::http_file_service` | ✅（FileReply→axum 0.8） | ✅（FileReply→生成 crate 的 axum 0.7） |
| 构造 native/lowering | shim 9936 / trans `a2r_std::http::file_response` | ✅ | ✅（同一 api.at 转译体内联） |
| wire 矩阵 | §6.1 双端表 | 6 族全绿 | 矩阵子集全绿（见 protocol 报告） |

两代 axum（0.8 本仓 / 0.7 生成 workspace）经版本无关 `FileReply`（纯数据 + futures_core 流）
单源投影——生成 crate 无需新增命名依赖（决策报告 §3 冻结设计的实证）。

## 2. 差异面（如实）

- VM 腿响应含 CORS + X-Request-Id 追加（与其他 VM 回复形态一致）；生成腿 CORS 由路由层
  tower_http CorsLayer 承担（生产生成形态原样）。协议语义（status/headers/body）双端一致。
- VM 腿 scope/许可随 body 代持（FileBodyAdapter Drop/finish hook 恰一次）；生成腿无 VM
  scope——资源由 pump/watchdog 的服务级收口承担（同一次 serve 代码路径）。

## 3. 复现

```bash
# VM 腿
cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan729
# 生成腿（首编 ~14min 冷缓存；共享 worktree target 后 ~10s）
cargo test -p auto-man --lib --features test-http-e2e http_e2e_plan729 -- --test-threads=1
```
