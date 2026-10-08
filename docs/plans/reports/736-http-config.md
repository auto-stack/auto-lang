# PLAN-736 T-02 报告：配置与 CLI service 入口（AC-01/02, SD-01/02/04/05）

> 绑定：plan_revision 1 · 代码提交 `2dbae4685`（后续 T-03..T-06 增量见 git log）·
> 本报告为 per-AC 证据索引；门禁复现命令全部在 worktree
> `D:/autostack/.wt/lang-736/auto-lang` 执行。

## AC-01 证据：两轨独立服务/配置真实生效

| 项 | 命令/入口 | 实测 |
|---|---|---|
| VM 轨启动（无 UI/Vite/桌面） | `auto service --server vm --dir examples/http_server/api_contract --http-config-inline '{"profile":"development","listen":{"port":18321}}'` | loopback 监听 + 5 routes + wire（echo 400 诊断/plain 734734/404）+ `AUTO_SERVICE_READY` 含 profile+config_hash |
| 生成轨启动 | `auto service --server rust --dir examples/http_server/uploads --http-config-inline ...` | 生成→`cargo run` 子进程→`AUTO_SERVICE_READY {"bound":...,"profile":...,"config_hash":...}` |
| 默认 loopback | service.json 缺省 | `127.0.0.1:8080`（VM 旧 `auto run` 默认 0.0.0.0 **不动**——legacy 面零变化，迁移路径=决策报告 §3.1） |
| 显式监听/port0 | `listen.addr=0.0.0.0` / `port=0` | 合法；port=0 ready 报真实 bound 地址（serve_network ready 载荷） |
| 坏配置非零 | `--http-config-inline '{"...","max_connections":0}'` | EXIT 1，0.1s，指名 `range [1, 4096]` |
| 端口冲突非零（VM） | python socket 占口 + serve | EXIT 1，0.3s，`os error 10048` 指名 |
| 端口冲突非零（rust） | 同上（子进程） | EXIT 1（子进程 bind 诊断→非零传播） |
| 未知 server 非零 | `--server vue` | EXIT 1 |
| 端口/URL/legacy 优先级 | `-B` > config listen.port；legacy 链不动 | `cli_override` 来源记录（单测锁定） |

## AC-02 证据：ready 身份贯通（T-05 完成面）

- ready 判据链 bind→初始化→Ready（`serve_with` Ok 臂；compile 失败=run_file Err=非零）。
- 身份四元组 `instance_id/bound/profile/config_hash`：READY 行=health/ready 体=快照同一事实源（`http_service_observability::set_service_identity`）。
- 坏配置=子进程 exit 2（生成轨）/Err（VM 轨）；bind 失败=exit 1；**不输出 ready**。
- port=0：真实 local_addr 上报（ready 载荷实测）。
- rust_ui 探针升级：health 优先（503 不再假 ready；legacy 404=可继续），TCP 兜底。

## 单测与门禁

- `cargo t plan736`：T-02 面 10 测（范围/必填性/CORS 矩阵/代理身份/限速器/hash 稳定/round-trip/转发壳/seam）。
- `cargo test -p auto-man --lib http_service`：5 测。
- `cargo test -p auto --bin auto cli_service`：2 测（CLI 形状）。
- 决策与冻结：[736-http-decision.md](736-http-decision.md) §3.1（schema/范围表）。

## 偏差记录（语义调整，均在计划授权内）

1. `auto serve` → **`auto service`**（与 Plan 269 AutoVM daemon 命令撞名；daemon 零改动）。
2. serve 面不 kill 占端口进程（legacy run/split 的 kill 语义保留）——端口冲突=显式非零。
3. 预存修（非 736 回归，顺手修复并登记）：
   - `auto_lang::a2r_std::http` 缺 730 upload 转发壳（qualify 后生成上传后端 E0425）；
   - auto-man 0 字节 `src/main.rs`（E0601 阻塞 `cargo check -p auto-man`，bb708320f 误回归）。
