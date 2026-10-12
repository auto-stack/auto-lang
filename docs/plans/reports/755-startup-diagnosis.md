# PLAN-755 T-02：e2e 启动诊断有界化与 120s 挂死分解（2026-10-10）

- 问题（承 [751-review-r2.md](751-review-r2.md) §3）：两种已实测形态无法互释——repo-root 有界
  真实 run 诊断 **0.10s 快返** `SIGNATURE_UNVERIFIED`；同一源在 th 测试进程内 **120.046s
  nextest TERMINATING**。契约错误与超时的因果未接通，"60s×2 read timeout" 旧归因待证伪/证实。
- 方法：在独立 worktree `D:/autostack/.wt/lang-755/auto-lang`（基面 `d6e4819ba`，契约缺口未修）
  给 plan730 夹具加临时计时观测（run 调用/返回、ready 轮询逐 10 次、客户端各相位；观测后已
  还原，未提交），单测串行 `--nocapture` 实跑。

## 1. 实测时间线（http_e2e_plan730_vm_plain_endpoint_untouched，port 18975）

```text
t=0        start_server enter
t=379µs    server thread: calling crate::run
t=32.7ms   server thread: crate::run returned Err(
             STDASSEMBLY.SIGNATURE_UNVERIFIED: http.upload_receive
             (stdlib/auto/http.at, native #9937): selected callee lacks an
             independent adapter contract)          ← 与 repo-root 探针同形态
t=2.059s   ready-poll err iter=0  (os error 10061 积极拒绝)
t=23.51s   ready-poll err iter=10
t=44.92s   ready-poll err iter=20                    ← 斜率恒定 ~2.15s/次
t=66.41s   ready-poll err iter=30
t=88.15s   ready-poll err iter=40
t=107.62s  start_server done（50 次轮询耗尽，原实现静默通过）
t=109.67s  raw_request connect err iter=0
t=120.03s  nextest TIMEOUT（TERMINATING）
```

## 2. 因果链（闭合，无第二缺陷）

1. 契约缺口使 `crate::run` 在测试进程内 **32ms** 即返回 Err（与 repo-root 探针完全同源；
   stdlib 根经 `CARGO_MANIFEST_DIR` 解析，与 CWD 无关——两种形态的入口差异被排除）。
2. 夹具 `let _ = crate::run(&code)` 吞掉 Err；服务从未 bind，端口无人监听。
3. **本机每次被拒连接实测 ~2.15s**（os error 10061；18975 不在 Windows excludedportrange
   内，2026-10-10 netsh 核过——属宿主 WinSock 行为，实测常量即证，过滤驱动层成因不追）。
   ready-poll 50 次 ≈ 107.6s 静默耗尽；客户端 60 次重试预算另有 ~129s。
4. `.config/nextest.toml` slow-timeout period=60s、terminate-after=2 → **120s SIGKILL**，
   报为 TIMEOUT。主线程死在 connect 爬行循环里，**从未到达任何 read**——
   "60s×2 read timeout"旧归因证伪；plan326 `e2e_a_redirect_302_with_location`（read
   timeout 5s，同样 50×~2.15s 轮询）120.033s 同机制，read 相位同样未达。
5. interop 109.988s FAIL ≈ ready-poll 耗尽(107.6s)+首次连接(~2.1s) 量级吻合，
   精确相位 T-05 全档对账时核实。

## 3. 修复（T-02 永久落点，先于 T-03 契约补齐提交）

三处夹具启动助手有界化（`catch_unwind` + channel 回传，ready 窗口内 `try_recv` 即时
panic 真实原因，轮询耗尽也报 run 线程状态）：

- `crates/auto-lang/src/tests/plan730_http_upload_tests.rs` `start_server`
- `crates/auto-lang/src/vm/ffi/http_server.rs` `start_server`
- `crates/auto-lang/src/vm/ffi/http_server.rs` `start_example_api_server`

契约未修基面上的验证：两项原反例由 120s TERMINATING 转为**秒级 FAIL 且失败信息含
`SIGNATURE_UNVERIFIED`**（证据见 worktree 提交与 T-04 复验）。

## 4. 结论

- P751-R2-02 的"因果关系尚未确定"**已接通**：契约错误完全解释超时（叠加本机 ~2.15s/次
  拒绝连接成本的放大器效应）；T-08 条件任务（第二缺陷处置）**不触发**。
- P751-D1 的挂死机制分诊修正完成：非服务端 accept 不响应，非负载 flake，非固定端口撞
  Hyper-V 段；是"启动错误被吞 × 拒绝连接慢速爬行"复合形态。
