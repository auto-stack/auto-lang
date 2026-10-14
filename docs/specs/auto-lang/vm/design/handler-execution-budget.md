# HTTP handler 有限执行预算合同

> PLAN-781 阶段 C 交付。决策全文（探针定量 + 修订 r1-r3）见
> `docs/plans/reports/781-budget-decision.md`；本文为持久层规范。

## 配置面（宿主可信）

- 环境变量 `AUTO_HTTP_STEP_BUDGET`：进程启动期读取的**宿主级**配置
  （.at 业务代码不可写、不入 pac.at）；每进程一份 → 实例天然隔离。
- 语义 = 单 HTTP 请求 dispatch 生命周期的**累计指令上限**（跨段
  park/resume、跨嵌套 fn 不重置）。
- 合法域：十进制 u64，`1 ..= 4_000_000_000`；默认不设 = 10,000,000
  （既有 Legacy 段语义**逐字节不变**）。
- 非法值（0/负/小数/非数/溢出/超上界）= 启动期指名配置错误 + 非零退出
  （`auto run` / `auto serve` 臂 fail-fast，先于任何业务 dispatch）。
- 解析：`vm/budget_policy.rs`（`parse_http_step_budget` 单源）；快照 =
  `AutoVM.http_step_budget`（构造期一次读取，运行期不可变）。

## 执行语义（engine `DriveBudget::LegacyHttp(cap)`）

- 段帽 = policy 本身（无 park 的单段 handler 需单段跑满请求预算）；
  多段由循环内累计检查执法（后段先触累计）。
- 累计精确到**逐指令**（N-1 耗尽 / N 通过边界可控）；I/O park 等待
  不计步；4 个 Parked 点 + 段尾统一计入任务 `cpu_steps_total`
  （resume 不重置）。
- runaway 基线跨段延续（与 CpuSlice 同理由——慢速跨段泄漏不漏检）。
- 耗尽 = **可识别资源拒绝**：`instruction budget exhausted in '<fn>'
  … (per-request cumulative budget N steps; AUTO_HTTP_STEP_BUDGET)`
  ——不得变成解析错、空成功响应或无返回值假成功；HTTP 面 = 500 +
  该 detail（与默认档文案可区分）。

## 入口矩阵（消费 policy）

| 入口 | 消费 |
|---|---|
| Axum 段派发·命名（`call_fn_by_name_segment_http`） | ✅ |
| Axum closure（`call_closure_segment`） | ✅ |
| stdnet 串行同步（`call_fn_by_name_http`） | ✅ |
| 段恢复（`resume_fn_by_name_segment_http`，http 三处 step 循环） | ✅ 累计延续 |
| SSE 生成器 next / UI bridge（CpuSlice）/ back_proxy / run_module_init | ❌ 维持既有 |
| middleware（`call_fn_by_name_segment`） | ❌ v1 边界（共享 UI 入口） |

## 取消/终态与长任务配套

- 零新机制：deadline/取消/断连/服务关闭沿既有 request scope；一次
  请求一个终态，无迟到成功。
- 长导入 profile = **双 env**：`AUTO_HTTP_STEP_BUDGET=1000000000` +
  既有 `AUTO_HTTP_REQUEST_TIMEOUT_MS=300000`（默认 30s 等待会先截断）。
- 已知边界（KNOWN-DEBT F-781-C01/C02）：段派发泵串行——长忙 handler
  占泵可饿死共居 split-VM 前端的内部调用，且 Reader 前端视 Init 失败
  为致命；公平性修复（泵并发化 / HTTP CpuSlice 化）需新计划。

## 验证面

- 单测：`cargo t plan781`（V-14 实例隔离/默认不变、V-15 配置表、
  V-16 N-1/N/N+1·嵌套 fn·park/resume 累计、V-20 closure 入口）。
- 真 TCP：`cargo th http_e2e_plan781`（低预算指名拒 + 服务继续 +
  断连不楔死）。
- 全链驱动：`scripts/tools/test_reader_import_budget.py` +
  `test/fixtures/reader-import-budget/`（V-14/15/17/18/19/20）。
