# ACE慢例成本归因（只读）

> PLAN-893 SD-02：13项慢测的当前成本归因合同。工具是
> [scripts/test-governance/ace_cost_report.py](../../../scripts/test-governance/ace_cost_report.py)
> （只读`cargo t --out`产物），治理测试 `scripts/tests/test_ace_cost_report.py`
> （固定组 `infra-ace-cost-report`）钉住其判定与敏感性（V-03/04/05）。

## 输入与只读边界

唯一输入是固定driver真实运行的`--out`目录：`results.json`（真实终态/选择/门结果/资源汇总）、
`*/discovery|inventory|run.command.json`（argv/exit/meter：wall/workload_cpu/meter_cpu/
peak_job_bytes/job_processes）、`run.stdout`的libtest-json-plus逐测试终态事件（含exec_time）、
TestCase报告引用。**不启动测试/Job/编译/计量，不改写任何run产物**；报告只写全新路径，旧报告
不覆写。无第二meter——CPU/峰值语义沿用原`process_capture`（Job树口径）。

## 阶段口径

| 阶段 | 观测来源 | 无观测时 |
|---|---|---|
| preparation_build_discovery | discovery/inventory.command（cargo构建检查+nextest list/inventory） | UNOBSERVED |
| tool_preparation | tools/*.command.json（case工具准备） | UNOBSERVED |
| requested_work | run.command（真实负载）+逐测试exec_time | UNOBSERVED |
| parent_side_verdict / persist_gc / copy / cleanup | run.command作业树内部，无命令粒度标记 | **UNOBSERVED+边界**，不填0 |

顺序命令wall可合成观察窗；CPU按命令分列不相加冒总墙；峰值取max不相加（嵌套Job共享树）。
测试体内部自带观测（如K-09生成器子进程自报峰值+PeakSampler采样）属原测试所有，报告只引用
存在性，不复刻。

## 判定完整性（不能铸绿）

- PASS主张必须有真实执行产物支撑：run.command存在、executed>0、（Rust行）每个执行测试有
  terminal `ok`事件；driver errors/零选择/伪PASS（PASS但无终态）一律红。
- 取消/超时（exit 124或meter error）记为canceled/timed-out，不绿；工具非零exit保留在阶段
  exits里可见。
- **STALE**：run记录的`source_state`（HEAD+diff+untracked复合hash）或`config_sha256`与当前
  树不一致 → 该run成本标STALE，不得作为当前有效证据复用；同source/dependency/config/input/
  tool/observer一致才可复用。
- 阶段敏感性：延迟必须落在真实发生的阶段桶（准备延迟不冒工作量、工作延迟不冒准备），总窗
  相应响应；重叠/缺phase/伪计数不可冒正确归因。

## 13项归因结论的形态

每项必交：当前有效单轮成本（或同身份合法复用+理由）、原因证据或具名未定位阶段与下一探针、
保慢原因/后继产品修复scope/owner建议/关闭oracle。单轮不冒正式P90（P90≤2000ms属各原
profile正式合同）；≤60秒是完整组成本目标不是本13项强杀门。产品算法/存储/ABI修复另案不预占号。

## PLAN-1060 增补（2026-11-15）：普通组/TestCase 入口的预算判定

- `--mode regular` 在**原 13 项 policy 归因之外**新增两类判定：① 完整普通职责组（截断或越界
  预算即判 OVER/FAIL，边界不取整）；② `domain=ace` 的 TestCase 入口组（同一 120/60 口径，
  逐 check 取 runner 的原生 `duration_ms`，期望 FAIL 的元控制按 `case-control-oracles.json`
  归因）。
- 阶段边界单列：准备（tools）/发现/运行/父判定/复制清理分别记录；`--cold-run` 把冷构建与
  工具构建单列为 T_cold，不混入 T_warm。
- 非本次执行的选择项（`#[ignore]`、平台排除）记为 reclaimed/裁定排除，不计入分母也不冒 PASS。

## 会话模式：跨组差集与新增 item 门（PLAN-1098 SD-03）

`--mode session` 在同一只读输入上判定多个模块，新增两件单组看不到的事：

- **身份差集**：`--baseline <ace-session-baseline/1>` 给出 `added` / `removed` 原生身份；
  `--baseline-out` 把本次身份写成下一次基线（可往返）。
- **新增 item ≤10 秒**：只对 `added` 身份执行 `--new-item-cap-seconds`（默认 10s）；
  老身份继续走组预算。无 `exec_time` 的新身份同样判违规。

退出码沿用 `policy`/`regular` 语义：0 = GREEN、1 = OVER_BUDGET/新增例超时、2 = 不可验证
（坏基线、缺终态、STALE 传播）。会话模式不建新 meter、不重跑、不改写产物；它复用
`regular_module` 的暖预算、终态完整性、STALE 与 Case 控制覆盖判定。

