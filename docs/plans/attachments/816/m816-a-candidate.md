# PLAN-816 M816-A 阶段候选（T-12 交付，待 T-13 独立阶段审查冻结）

> 状态：**候选草案**（code @`0373cfa38` + baseline @`6b4bf9c2f`）。T-13
> 审查绑定冻结 SHA 后本文件 §1–§6 即事实清单；§7 为 merge-owned pending。
> 本阶段 ≠ 整 Plan 交付：T-03..T-11 未开（门见 §8 任务行），Plan 保持
> `executing`。

## 1. 身份

| 项 | 值 |
|---|---|
| plan / revision / phase | PLAN-816 / r2 / M816-A（T-12） |
| worktree / branch | `D:/autostack/.wt/lang-816/auto-lang` / `codex/plan-816-dev` |
| sync base（master 合入点） | `b65d0e963`（含 M815-A 交付 `e609caf1af`，祖先关系验证通过） |
| code_commit | `0373cfa388b9afc6b58e1e699fab1119af6dafd8` |
| baseline_commit | `6b4bf9c2fcf3e872b33f6cc8505cf9fa09aa550f`（777 基线刷新） |
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
| V-05 fixture（真实进程） | `cargo build -p auto-ui-tools -p auto-tool-execution && AUTO_UI_TOOL_BIN=target/debug/auto-ui-tool.exe cargo test --manifest-path test/fixtures/modularity/tool-client-consumer/Cargo.toml` | **10/10 通过（2.78s）**：describe 正控+EOF 码0+日志隔离（stderr 无控制帧）、validate 损耗敏感回显（name=外部 demo/not_checked 含 generation）、版本拒绝零副作用+流活、缺能力（generate）拒绝零副作用+流活、cancel 诚实（served→NotCancellable/未知→InvalidField）、**超时→收尸+reader join 后返回 Timeout+连接 Broken+shutdown 即时完成非 Success**、**孙进程 heartbeat 停止推进**（树真停观察）、**流污染→具名 StreamViolation+Broken**、cwd/env 经 spec 到达（probe 模式 stderr 证据）+干净 EOF、预算超限出口前 TooLarge 且流不污染 |
| V-05 fixture fresh/locked | `cargo generate-lockfile && cargo test --locked`（43 packages 锁定） | 10/10 通过（lockfile 按仓规不入库，流程与 792 fresh-lock 同构） |
| 闭包断言 | `bash test/fixtures/modularity/tool-client-consumer/verify-deps.sh` | PASS（client/protocol/execution/project-model 在闭包；auto-ui-tools/Man/lang/VM 零出现） |
| client crate 单测 | `cargo test -p auto-tool-client` | 4/4（纯数据面；真实进程面在 fixture——日常档不依赖外部二进制构建时序，镜像 815 分层） |
| 设施回归 | `cargo test -p auto-tool-execution` | 15/15（lib 2+集成 13，3.71s；te-child/take_* 增量无回归） |
| clippy | `cargo clippy -p auto-tool-client --all-targets` / `-p auto-tool-execution --all-targets` | 两 crate 0 警告 |
| CLI/Man 触面 | `cargo check -p auto -p auto-man` | exit 0（无路由改动，不切默认 CLI） |
| 777 | `python scripts/modularity-audit.py --refresh-baseline`（干净树 @0373cfa38）→ `--mode strict` | 基线 78 declared edges；**verdict=pass violations=0 grandfathered=2** |

## 5. Spec 候选（SD-01 A 子集；T-14 受管沉淀，本节为内容候选）

目标文件 `docs/specs/tool-execution/project.md`（新增），A 子集规则：

1. **owned 执行设施**（T-02 已落，随本阶段一并沉淀）：子进程执行的唯一设施 = `auto-tool-execution`（argv/cwd/env/stdio 策略/退出具名分类/deadline/cancel+收尸/reader 真终止/树退出）；`kill 发出 ≠ 已停`——取消/超时后观察到真停（收尸+reader join）才返回；只终止本次拥有的树。
2. **I-TOOL 客户端**：客户端经真实进程消费工具宿主（单源帧/信封），不静态链接工具实现；宿主拒绝是数据（`Reply::Host`），客户端故障具名分类；控制流（stdout）仅帧、日志仅 stderr，非帧字节 = 具名违规且流终态；请求 deadline 到期 = owned 树终止+收尸+reader join 后返回。
3. **发现边界（A 现状）**：客户端只接受显式工具路径，无隐式搜索；发现/bootstrap/安装 profile 规则等 M814-A 发布后的整链 probe（T-03/T-04）与 M815-B（T-04 完整正控）再定稿补入。
4. **测试分层**：crate 内联测试纯数据面；真实进程正反控在独立 fixture（driver 先构建二进制再以 env 钉定）；寿命反例用自带截止的具名测试子进程（te-child）。

SD-02..05（auto-cli/auto-man/http-service/generated-ui-distribution）不在 A 沉淀范围（薄 CLI 路由/服务分层/搬迁均未发生）。

## 6. 诚实限制（不因本候选宣称）

- **未交付**（B 阶段或门后）：工具发现/版本探测/bootstrap（T-04，门 = T-03+M814-A+M816-A）；管理服务/语言 adapter 与 auto-man-runtime（T-03）；薄 CLI 路由切换（T-05+，需 M814-B+M815-B）；service/serve/req/watch 接线（T-06）；完整 V-01..04 与 777/792 完整 profile fresh/locked/重定位（T-07）；792 `DELIVERY_CRATES` 增补——独立安装闭包变更随 profile 冻结在 B 落地（A 阶段客户端为库+消费证据，不改发行闭包；基线 §5 候选的定稿门 = M814-A 后整链 probe）。
- fixture 的 te-child 寿命反例是**具名测试子进程**（合同允许的异常/寿命控制），真实协议正控全部走实际 auto-ui-tool 进程，无 fake 充当正控。
- 814 未发布面零消费（M814-A 仍在审）。

## 7. merge-owned pending（T-13 pass 后 T-14 冻结门）

- 冻结提交上裸 `cargo t`（全日常面）；触面判定：diff 仅新增 crate/fixture+`auto-tool-execution` 增量（无 VM/trans/UI 生成代码触面 → tv/tt/tu 预计零触发，以最终 diff 复核）；fixture driver + verify-deps 复跑收据；master 祖先/来源终验；Spec 子集受管沉淀与 ledger 增量；不归档不清树。
