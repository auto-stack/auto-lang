# PLAN-816 M816-A 阶段候选（T-12 交付，待 T-13 独立阶段审查冻结）

> 状态：**R1 修复后候选**（T-13 R1 needs_fix 六项 findings 已全部修复；
> 冻结 SHA = 修复提交 `4372d56d9`，见 §8）。T-13 审查绑定该 SHA 复审后
> 本文件 §1–§7 即事实清单；§7 为 merge-owned pending。
> 本阶段 ≠ 整 Plan 交付：T-03..T-11 未开（门见 §8 任务行），Plan 保持
> `executing`。

## 1. 身份

| 项 | 值 |
|---|---|
| plan / revision / phase | PLAN-816 / r2 / M816-A（T-12） |
| worktree / branch | `D:/autostack/.wt/lang-816/auto-lang` / `codex/plan-816-dev` |
| sync base（master 合入点） | `b65d0e963`（含 M815-A 交付 `e609caf1af`，祖先关系验证通过） |
| code_commit（R1 修复后冻结候选） | `4372d56d9`（R1 前历史：`0373cfa38` 实现 + `6b4bf9c2f` 777 基线） |
| 消费的上游交付 | **M815-A**（master `e609caf1af`）：`git diff e609caf1af..b65d0e963 -- crates/auto-ui-tools crates/auto-tool-protocol crates/auto-project-model` 为空（发布源零漂移） |
| 上游工具二进制（本树构建） | `target/debug/auto-ui-tool.exe` SHA256 `a7c070596a74bfc56586e522b89e5cc68240803adc68c9e33117515847faaa6c`（debug 构建含路径信息，逐字节 hash 随构建环境变；源身份以上行 commit 钉定，消费合同见 §2） |
| 依赖修订 | 无 814 依赖（未消费其未发布面）；未切任何默认 CLI 路由 |

## 2. 交付范围（A 阶段实际面）

- **`crates/auto-tool-client`（新，workspace 成员，777 域 man-ecosystem）**
  ——I-TOOL 真实客户端：
  - `ToolClientSpec{tool_path 显式, argv, cwd, env, protocol_version, request_timeout, frame_budget, tree_scope=ProcessTree, shutdown_timeout}`；
  - `ToolClient::{spawn, describe, validate_project, cancel_operation, request, logs, host_pid, shutdown}`；
  - 负控通道 `request_with_version` / `request_unknown_operation`（合同验证用）；
  - `Reply{Describe,Validate,Event,Host(ToolError)}`——宿主拒绝是数据不是异常；
  - `ClientError{Spawn,InvalidSpec,Io,SendFrame,StreamViolation,StreamClosed,IdMismatch,MalformedReply,Timeout,Broken}` 具名分类；
  - **寿命合同**：请求超时 = owned 树终止 + 收尸 + reader 线程 join（真停）之后才返回 `Timeout`；EOF 优雅关闭按宿主合同期待码 0，宽限到期则 cancel；控制流任何非帧字节 = 具名 `StreamViolation`，流终态 `Broken`；
  - 闭包 = data（auto-tool-protocol/auto-project-model）+ man-ecosystem（auto-tool-execution），**无 auto-ui-tools 静态链接**；`ExitKind` 经 client 再导出。
- **`auto-tool-execution` 增量**（同一设施演化，非第二 runner）：`OwnedProcess::take_stdin/take_stdout/take_stderr`（管道所有权移交消费方 reader 线程）；te-child 增 `probe` 模式（cwd+env→stderr 身份证据）与 `stdout-kib` ARG2 写后延时（污染反例时序确定化）。
- **独立 consumer fixture `test/fixtures/modularity/tool-client-consumer/`**（根 Cargo.toml exclude，空 `[workspace]`）：依赖仅 auto-tool-client/auto-project-model/auto-tool-protocol；`verify-deps.sh` 钉闭包（禁 auto-ui-tools/auto-man/auto-lang/auto-vm/auto-gen/auto-media-core）——"Core 无 UI 安装可消费"的编译期证明。
- 777：`auto-tool-execution`（补 T-02 欠账）与 `auto-tool-client` 登记 man-ecosystem；基线刷新 78 declared edges。

## 3. 消费合同（对 M815-A 已发布窄能力）

经真实进程消费协议 v1：`describe`（ToolDescriptor：auto-ui-tool/协议 1/能力=[describe, validate_project]）、`validate_project`（GenerationRequest 同源 wire、损耗敏感回显）、`cancel` 控制消息（NotCancellable/InvalidField 诚实回答）；版本/能力拒绝在副作用前、流保持可用；帧预算 8MiB 出口前拒绝（TooLarge）；日志仅 stderr、控制流仅帧；EOF→退出码 0。客户端不做工具发现（显式路径；discovery 归 T-04）。

## 4. 验证证据（work 树 @0373cfa38→6b4bf9c2f 实跑）

| 门 | 命令 | 结果 |
|---|---|---|
| V-05 fixture（真实进程） | `cargo build -p auto-ui-tools -p auto-tool-execution && AUTO_UI_TOOL_BIN=target/debug/auto-ui-tool.exe cargo test --manifest-path test/fixtures/modularity/tool-client-consumer/Cargo.toml` | **R1 后 12/12 通过（3.7s；原 10 测 + F01b/F02b 两新增）**：describe 正控+EOF 码0+日志隔离（stderr 无控制帧）、validate 损耗敏感回显（name=外部 demo/not_checked 含 generation）、版本拒绝零副作用+流活、缺能力（generate）拒绝零副作用+流活、cancel 诚实（served→NotCancellable/未知→InvalidField）、**超时→收尸+reader join 后返回 Timeout+连接 Broken+shutdown 即时完成非 Success**、**孙进程 heartbeat 停止推进**（树真停观察）、**流污染→具名 StreamViolation+Broken**、cwd/env 经 spec 到达（probe 模式 stderr 证据）+干净 EOF、预算超限出口前 TooLarge 且流不污染 |
| V-05 fixture fresh/locked | `cargo generate-lockfile && cargo test --locked`（43 packages 锁定） | 通过（lockfile 按仓规不入库，流程与 792 fresh-lock 同构） |
| 闭包断言 | `bash test/fixtures/modularity/tool-client-consumer/verify-deps.sh` | PASS（client/protocol/execution/project-model 在闭包；auto-ui-tools/Man/lang/VM 零出现） |
| client crate 单测 | `cargo test -p auto-tool-client` | **R1 后 9/9**（4 数据面 + 5 故障注入：缓冲坏前缀/粘包/reader Io/半帧 EOF/非类型化信封；真实进程面在 fixture——日常档不依赖外部二进制构建时序） |
| 设施回归 | `cargo test -p auto-tool-execution` | **R1 后 23/23**（lib 6 含 4 注入 + 集成 17 含 4 新反例，3.7s） |
| clippy / rustdoc / fmt | 两 crate clippy 0 警告（--no-deps）；`cargo doc` 0 告警；`fmt --check` + fixture rustfmt 过 | 全绿（F-05 闭合） |
| CLI/Man 触面 | `cargo check -p auto -p auto-man` | exit 0（无路由改动，不切默认 CLI） |
| 777 | `python scripts/modularity-audit.py --mode strict`（@4372d56d9 复核） | 78 declared edges；**verdict=pass violations=0 grandfathered=2** |
| 审方反例回放 | `powershell .../review-r1/replay.ps1`（@4372d56d9） | **5/5 失败反例全过**（F01a/F01b/F02a/F02b/F03；C01/C02/C03 对照保持；REVIEW_PROBE_FAILED_CONTROLS=0） |

## 5. Spec 候选（SD-01 A 子集；T-14 受管沉淀，本节为内容候选）

目标文件 `docs/specs/tool-execution/project.md`（新增），A 子集规则：

1. **owned 执行设施**（T-02 已落，随本阶段一并沉淀；R1 修复后语义）：子进程执行的唯一设施 = `auto-tool-execution`（argv/cwd/env/stdio 策略/退出具名分类/deadline/cancel+收尸/reader 真终止/树退出）；deadline 覆盖整次执行（stdin 写入经 writer 线程、根等待与收尾共用同一绝对截止）；根退出后仍持管道的 owned 后代先终止、reader 有界收尾；`kill 发出 ≠ 已停`——观察到真停（收尸+reader 有界收尾）才返回；reader 读错/panic 与 writer/终止失败具名传播（`ReaderIo`/`ReaderPanicked`/`PipesHeld`），Windows 树归属在子进程任何用户代码运行前建立（挂起启动+入 Job+恢复）。
2. **I-TOOL 客户端**：客户端经真实进程消费工具宿主（单源帧/信封），不静态链接工具实现；宿主拒绝是数据（`Reply::Host`），客户端故障具名分类；控制流（stdout）仅帧且先 drain 已缓冲内容（完整帧与坏前缀不等待新字节）、日志仅 stderr，非帧字节 = 具名违规且流终态；请求 deadline 覆盖发送+等待，到期 = owned 树终止+收尸+writer/reader 有界收尾后才返回 `Timeout`（终止/收尸失败与线程 panic、管道被非拥有进程持有具名传播，不谎报）。
3. **发现边界（A 现状）**：客户端只接受显式工具路径，无隐式搜索；发现/bootstrap/安装 profile 规则等 M814-A 发布后的整链 probe（T-03/T-04）与 M815-B（T-04 完整正控）再定稿补入。
4. **测试分层**：crate 内联测试纯数据面+故障注入面（假 Read 驱动，无 sleep）；真实进程正反控在独立 fixture（driver 先构建二进制再以 env 钉定）；寿命反例用自带截止的具名测试子进程（te-child：sleep/grandchild/orphan/probe）。

**SD-05-A 处置**（R1 复审要求明确给出，替代此前"SD-02..05 整体排除"的一揽子表述）：
目标 `docs/specs/repository-boundaries/generated-ui-distribution.md` 增一段精确补文（T-14 受管沉淀），内容候选：

> **工具消费方式（PLAN-816 M816-A 起）**：`auto-ui-tool` 及后续 I-TOOL 工具宿主只能经**真实子进程 + auto-tool-protocol 帧**消费（客户端 `auto-tool-client`；闭包由 `test/fixtures/modularity/tool-client-consumer/verify-deps.sh` 钉住：无 auto-ui-tools 静态链接）。M816-A **不改变任何发行/安装闭包**：792 `DELIVERY_CRATES` 与生成 UI 分发清单维持 M815-A 现状——客户端是 workspace 库，供后续薄 CLI 组合（T-05+）消费，不作为独立发行件；独立安装 profile 的闭包增补属 B（T-07，profile 冻结门 = M814-A 后整链 probe）。

即：现有 distribution Spec **未覆盖**"显式进程消费 vs 静态链接"这条边界（需上述补文），但**已充分覆盖**"A 尚无安装变化"（分发清单未动，如实陈述即可）。SD-02/03/04 仍不在 A 沉淀范围（CLI 路由/服务分层/搬迁均未发生——这是各 delta 自身的现状理由，不是 SD-05 的替代理由）。

## 6. 诚实限制（不因本候选宣称）

- **未交付**（B 阶段或门后）：工具发现/版本探测/bootstrap（T-04，门 = T-03+M814-A+M816-A）；管理服务/语言 adapter 与 auto-man-runtime（T-03）；薄 CLI 路由切换（T-05+，需 M814-B+M815-B）；service/serve/req/watch 接线（T-06）；完整 V-01..04 与 777/792 完整 profile fresh/locked/重定位（T-07）；792 `DELIVERY_CRATES` 增补——独立安装闭包变更随 profile 冻结在 B 落地（A 阶段客户端为库+消费证据，不改发行闭包；基线 §5 候选的定稿门 = M814-A 后整链 probe）。
- fixture 的 te-child 寿命反例是**具名测试子进程**（合同允许的异常/寿命控制），真实协议正控全部走实际 auto-ui-tool 进程，无 fake 充当正控。
- 814 未发布面零消费（M814-A 仍在审）。

## 7. merge-owned pending（T-13 pass 后 T-14 冻结门）

- 冻结提交上裸 `cargo t`（全日常面）；触面判定：diff 仅新增 crate/fixture+`auto-tool-execution` 增量（无 VM/trans/UI 生成代码触面 → tv/tt/tu 预计零触发，以最终 diff 复核）；fixture driver + verify-deps 复跑收据；master 祖先/来源终验；Spec 子集受管沉淀（含 §5 SD-05-A 补文）与 ledger 增量；不归档不清树。

## 8. R1 修复轮记录（T-13 R1 needs_fix → work 修复，@4372d56d9）

T-13 R1（reviewed 6b4bf9c2f）六项 findings 全部修复，修复提交 `4372d56d9`
（+1169/−227，7 文件；设施 crate 语义增强 + client 内部重构，公开 API 只
增不改）：

| Finding | 修复 | 永久回归 |
|---|---|---|
| F-01(P1) 阻塞写越过 deadline | `run()` 重构：reader 线程先于任何写入启动；stdin 写入经 writer 线程，与根等待/收尾共用同一绝对截止；client `send_deadline` 同一截止覆盖排队+阻塞写+等待，超时先终止 owned 树解除写阻塞 | 集测 `deadline_covers_blocked_stdin_write`（251ms）；fixture `request_deadline_covers_blocked_send`；审方 F01a/F01b 回放 251/254ms |
| F-02(P1) 持管道后代拖过期限 | 根退出（真实分类保留）后、reader 收尾前显式终止 owned 树（win_job terminate / killpg，Unix 组空 ESRCH 容忍）；reader/writer 有界收尾（TEARDOWN_GRACE），宽限耗尽 = 具名 `PipesHeld` 不谎报；client abort 先 cancel 再结束 idle writer | 集测 `root_exit_terminates_pipe_holding_descendants`（167ms+heartbeat 真停）；fixture `shutdown_bounds_pipe_holding_descendants`（127ms）；单测 `pipes_held_is_named_after_grace`；审方 F02a/F02b 回放 |
| F-03(P2) 缓冲坏帧被下一次 read 遮住 | reader 内层 drain 循环：Complete 后继续解析缓冲（含坏前缀 TooLarge/Malformed），仅 Incomplete 才 read；`reader_session` 泛化 `Read` | 注入单测 `buffered_bad_prefix_reported_without_more_reads`（坏前缀在零次额外 read 下报 TooLarge）/`coalesced_frames_all_drained_before_eof`；审方 F03 回放 0ms |
| F-04(P1) Windows 入 Job 前执行窗口 | `CREATE_SUSPENDED` 启动 → 挂起中建 Job+配置+`AssignProcessToJobObject` → Toolhelp 线程快照恢复主线程：后代只可能在归属建立后创建；失败路径对挂起根终止+收尸（不可能已有后代） | 集测 `early_spawned_descendant_belongs_to_owned_tree`（orphan 早派生+heartbeat 随树真停，确定性） |
| F-06(P2) 错误被吞 | `ExecutionError::{ReaderIo,ReaderPanicked,PipesHeld}` 与 `ClientError::{PipesHeld,Panicked}`+`BrokenReason::WriterFailed`；`Timeout` 仅在 cancel 成功+有界收尾完成后返回；`Done` 不变量（无 Done 断开=panic）；预算拒绝（pre-IO TooLarge）不再误杀连接 | 注入单测 4（设施）+5（client）；集测 `stdin_writer_failure_is_named`；干净 EOF 对照 |
| F-05(P3) 格式/文档 | 两 crate + fixture rustfmt 全过；rustdoc 3 条 broken-link 清零（ProcessTree/argv 转义） | fmt --check/rustdoc 0 告警入证据矩阵 |

验证矩阵（@4372d56d9 实跑）：设施 6 单测+17 集测（原 15 全保持）、client
9 单测、fixture 12/12 真实进程；审方 `replay.ps1` **5/5 失败反例全过**
（C01/C02/C03 对照保持，`REVIEW_PROBE_FAILED_CONTROLS=0`）；clippy 两
crate 0 警告、rustdoc 0、fmt 过；verify-deps PASS、777 strict pass（78
边/0 违规/2 grandfathered）、`cargo check -p auto -p auto-man` exit 0。
