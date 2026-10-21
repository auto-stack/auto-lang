# PLAN-816 r2 / M816-A / T-13 独立阶段复审 R4

2026-10-21。结论 **pass（M816-A 阶段）**。R3 两项 findings（01/02）**CLOSED**；
受影响面外证据按 R3 裁定复用（未变代码/依赖/测试配置）。附 2 项**非阻塞**
质量注记。Plan 保持 `executing`；T-14 merge-owned 门 pending。

## 审查身份与限制（如实声明）

- reviewer=本聊天会话，与 R3 审查及修复同会话（用户指定）；不能声称独立
  上下文。对冲手段：结论从工件重建——R3 修复 diff（3 文件 +404/−143）
  逐行精读、三面测试+8 次全套压力重跑、双 replay 重放、全门禁重跑；未
  采信实施侧摘要。pass 的机器可复现部分（replay/测试/门禁）独立于审查者
  身份；T-14 冻结门（裸 cargo t 等）在合入前提供又一重机器核验。
- review_scope=`phase`（M816-A/T-13，非整 Plan final）；未创建 agent、
  未修改实施源码。

## 冻结身份

- worktree `D:/autostack/.wt/lang-816/auto-lang`，branch `codex/plan-816-dev`，
  reviewed_commit=`6e03f5f268ee0eb4a3c2b9f64d4924c226977b72`（入场/收场
  clean）；R3 修复 diff=aa2edc8fb→6e03f5f26（3 文件 +404/−143）。
- 祖先链 b65d0e963→…→aa2edc8fb→6e03f5f26 验证；上游 M815-A
  `e609caf1af` 三 crate 零漂移；无 814 消费。

## 指派范围核验（R3-01/02 闭合 + 三个重点）

### F-816-R3-01（stop 先停后排空）— **CLOSED**

- 代码：设施 `CaptureWorker` stop→`drain_readable`（poll 至假，DRAIN_CAP
  500ms 防御）；client `parse_drain` 提取（stop 路径=排空 OS 管道→**重解析**
  →退出——已到达帧/违规不丢）、`stderr_session` 提取（stop 就地排空进
  sink，日志不缺字）。逐行核实与设计一致。
- 实测：R3 实机间歇场景（consumer_describes stderr 缺字/shutdown_bounds）
  在修复后 **15+8 次全套连跑 0 红**（修复前约 1/6~2/12 复现率）。

### 重点①三个注入单测的确定性 — **合格（附注记）**

- client 两测（`stop_drains_preset_frames_before_stopping`、
  `stderr_stop_drains_preset_bytes`）以 `stop.store(true)` **先行置 stop**
  +预置数据——确定性最坏情形（stop 先于任何读），无 sleep/无负载依赖 ✓。
- 设施测（`stop_drains_readable_bytes_before_stopping`）：stop 由
  `stop_and_join` 内部置位（`CaptureWorker` 拥有私有标志，无法外部预置），
  覆盖的交错是概率性的，但**断言结果确定**（bytes==payload、Stopped）。
  最坏情形由结构同构（同一 drain-on-stop 模式）+client 两测覆盖——
  **注记 N-1**：接受，理由为 API 边界所限与同构覆盖；若后续给
  CaptureWorker 增测试钩子可补预置形态。

### F-816-R3-02（错误路径不停线程）— **CLOSED**

- 设施：`finish_early`（stop+有界 join 后传播）覆盖全部四处早退
  （wait_deadline Err / 超时分支收尸 Err / 无 deadline wait Err /
  terminate-after-root-exit Err——grep 计 4 个调用点核实）；三个 finish
  顺序化（任一错误返回时三线程均已终结）。
- client：`stop_all` + shutdown 三个错误分支先停后返回；
  `drain_until_done` 重构为**全收集后传播**（writer→stderr→stdout 三结局
  收齐、stdout 坏帧记下不中断 drain 到 Done，之后按序传播）——逐行核实。

### 重点②drain 全收集后传播结构 — **合格（附注记）**

结构正确：错误传播时三条线程均已终结；坏帧按 natural_exit 语境定性保持
（close-bad=StreamViolation / kill 截断=噪声）。**注记 N-2（非阻塞）**：
`stdout_err` 为 PipesHeld/Panicked 时 `broken` 也被置
`BrokenReason::StreamViolated`——仅影响已终结连接上后续请求的拒绝文案
（返回给调用方的错误本身准确；枚举无 PipesHeld/Panicked 变体）。建议
下次触码轮（T-14 修复或 B）一行清理：仅 StreamViolation 情形置该标记。

### 重点③两处测试时序修正的正当性 — **成立**

- `shutdown_bounds`：heartbeat 观察移至 shutdown **之前**——shutdown 的
  树终止会杀死持管道后代，关停后观察"启动"在设计上不可能；先证存活
  （持有管道的事实成立）再验有界关停+真停，是唯一自洽顺序。
- `consumer_describes`：stderr 断言移至 shutdown drain 完成后——stdout
  回帧与 stderr 读取是两条异步流，即时快照必输竞争；R3-01 的
  stop-排空保证 shutdown 时日志完整，断言落在被保证的状态上。

## 回归与门禁（重跑 @6e03f5f26）

| 门 | 结果 |
|---|---|
| crates（client 12 / execution 9+17） | 全绿 |
| fixture 真实进程 15/15 + 全套连跑 8 次 | 0 红 |
| R1 replay（F01a/F01b/F02a/F02b/F03） | 5×contract_pass（01/02/04 无回归保持） |
| R2 replay（C04/close-clean/close-bad/timeout-held-readers） | 4×contract_pass（close-bad=StreamViolation、threads 4→4、251ms——03/05/06 主面闭合保持） |
| clippy --no-deps / rustdoc / fmt+fixture rustfmt | 0 / 0 / 过 |
| verify-deps / 777 strict / `cargo check -p auto -p auto-man` | PASS / pass（78 边 0 违规 2 grandfathered）/ exit 0 |

未整档（时序裁定）；tf 归主检出到期批量。

## 结论与后续

- **AC-08 pass（绑定 6e03f5f26）**；AC-07 主面闭合保持；阶段 pass ≠ 整
  Plan 交付——B 面任务与 AC-01..06 按既有门 pending。
- 注记 N-1/N-2 为非阻塞质量项（N-2 建议随下次触码清理），不构成本轮
  finding、不授权转债延期。
- **next：T-14（merge-owned 阶段发布）**——冻结提交 `6e03f5f26` 上串行：
  裸 `cargo t`（全日常面）；diff 触面复核（预计 tv/tt/tu 零触发）；fixture
  driver+verify-deps 复跑收据；master 实际合入（用户授权的 Git 合入）；
  Spec 子集受管沉淀（SD-01-A + SD-05-A 补文）与受管 ledger 增量；祖先/
  来源终验；不归档不清树。
