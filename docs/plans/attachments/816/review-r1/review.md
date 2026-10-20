# PLAN-816 r2 / M816-A / T-13 独立阶段审查 R1

2026-10-20。结论 **needs_fix**，不是阶段 pass 或整 Plan final。Plan 保持
`executing`；T-02/T-12 重开，T-01 保留完成，current_step=1/14。

## 审查基线与范围

- reviewer_identity：本聊天 Codex `/root`，独立于实施会话；未创建子 agent，未修改实施源码。
  未暴露具体模型/会话编号，不推定身份。本轮直接从合同、代码与反例重建结论。
- review_scope=`phase`，对象 `PLAN-816:r2:M816-A`；主检出 Plan r2，实施唯一树
  `D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`。
- reviewed_commit=`6b4bf9c2fcf3e872b33f6cc8505cf9fa09aa550f`；代码提交
  `0373cfa388b9afc6b58e1e699fab1119af6dafd8`，T-02 来自 `5fcdabe14`。
- diff base/sync base=`b65d0e963bbf39f97de5c1d6531997f6f21c760b`（与当前 master
  的 merge-base）。该范围 14 文件、2356 增/2 删，只新增执行设施、客户端、fixture、
  workspace/777 接线；旧 CLI/Man 默认路由无修改。
- 原始 T-02 base=`9e80ae0fb6f907ab5b0bc5320c28c0fa6f75acbb`；上游 M815-A
  delivery=`e609caf1af511b96832f511ab9d1fa6c0c11f563`，均验证为实施 HEAD 祖先。
  `git diff e609caf1af..HEAD -- crates/auto-ui-tools crates/auto-tool-protocol
  crates/auto-project-model` 为空；没有消费 814 WIP。
- 实施树入场及检查结束 `git status --short` 均为空，HEAD 未变化；锁、target 均为忽略项。
  主检出原有其他用户/并行工作保留，本轮仅写本报告、反例和 Plan/债务簿记。
- 真实工具 exe SHA256=`a7c070596a74bfc56586e522b89e5cc68240803adc68c9e33117515847faaa6c`，
  与 work 候选逐字相同。输入、依赖锁、Spec、技能和源码 hash 见 [inputs.json](inputs.json)。

## Findings（全部 OPEN，不授权转债延期）

### F-816-R1-01 / P1：deadline 没覆盖阻塞写入

**原要求缺口；T-02/T-12，AC-02 当期/AC-07，V-02/V-05。**
`auto-tool-execution/src/lib.rs:311–338` 在建立 reader/开始限时等待前同步
`write_all(stdin_data)`；`auto-tool-client/src/lib.rs:373–389` 在
`recv_timeout` 前同步发送整帧。对端不读 stdin 时，管道满后两个入口都失去 deadline
控制；双向大输出还会因 reader 尚未启动而相互等待。

反例 F01a/F01b：owned 自带 1800ms 截止的对端不读输入，1MiB stdin/协议帧
（后者低于 8MiB）+250ms deadline。实际 **1805/1804ms** 才返回
`Io(write stdin, BrokenPipe)` / `SendFrame(Io(BrokenPipe))`，不是 Timeout。
对照 C02：同一对端的小 describe 正确在 264ms Timeout。
这不是新规模承诺，而是已公开 deadline 在合法输入上的失效。

修复须让同一绝对 deadline 覆盖发送、等待及资源收尾，reader 及时并发读取，
堵塞 writer 能随 owned 树退出并被 join；不能只把 recv_timeout 再缩短。
永久回归应包含小/大输入、不读输入、先写 stdout/stderr 再读 stdin 的相邻组合。

### F-816-R1-02 / P1：根进程先退出使 reader join 越过 deadline/关闭宽限

**原要求缺口；T-02/T-12，AC-02 当期/AC-07，V-02/V-05。**
执行设施 `lib.rs:335–364` 仅对根 Child 等待设限，根退出后直接无界 join；客户端
`lib.rs:440–455` 同样只限根进程，join 期间不再监督 owned 后代。Windows Job
仍由 `proc` 持有，直到 join 返回、对象析构才关闭；后代持有继承 stdout/stderr 时
EOF 不会到来。Unix 路径亦不能用根 reaped 当整个组已结束，Drop 还会因 reaped
直接跳过 terminate。

F02a/F02b：根派生继承 stdout/stderr 的后代后 120ms 退出，后代 1800ms 自停，
run deadline / shutdown_timeout=250ms。实际 **1818/1810ms** 才返回，均报
Success。若后代长运行，join 可无限等；本反例特意使用有限寿命以免污染机器。

修复须在根退出、超时、关闭和错误路径分别处理 owned 树及管道寿命，并在 reader
join 前解决仍持管道的后代；保留真实退出分类，不将 root wait 等同树停止。
永久回归应覆盖后代持 stdout、持 stderr、根正常/非零退出与关闭/超时组合。

### F-816-R1-03 / P2：已缓冲的坏帧被下一次阻塞 read 遮住

**原要求缺口；T-12，AC-07，V-05 控制流违规具名分类。**
客户端 `lib.rs:494–525` 解析出一帧后压实 buffer，却无条件再 read，不先处理剩余
完整帧/坏前缀。解析结果由 OS 分块时序决定。

F03：一次 write 发出合法 `UnknownOperation` 回答 id=1，再接 `ff ff ff ff`，
随后保持存活 1800ms。首请求正确得到 Host 拒绝；第二请求实际 **255ms Timeout**，
而应报告已经收到的 `StreamViolation(TooLarge)`，进入 StreamViolated 终态。
C03 使用逐字相同内容、将两部分相隔 75ms 写入，正确得到
`StreamViolation(TooLarge { limit:8388608, observed:4294967295 })`。
本反例不要求 A 实现 progress/并行请求，首帧只是当前已有的合法错误回答。

修复须 drain 所有已可解析内容，仅 Incomplete 才读取更多字节；补合法帧+坏前缀、
粘包/拆包和 EOF 的对照，不能靠 sleep 调整测试时序补绿。

### F-816-R1-04 / P1：Windows 子进程开始运行后才加入 Job，树归属存在竞态

**原要求缺口；T-02/T-12，AC-02 当期/AC-07；源码证明，未声称压力复现。**
执行设施 `lib.rs:270` 普通 `Command::spawn()` 后，才在 `:287` 调用
`win_job::create_and_assign`，后者 `win_job.rs:92–109` 还需先创建/配置 Job。
没有让子进程在入 Job 前暂停执行的机制。合法调度可先执行子代码并派生孙进程，
随后只将根进程加入新 Job；先前后代没有被该代码追加入 Job。若根先退、assign
失败，错误路径也只 kill/wait 根，无法清理已逃出的后代。

`:284–286` 的“本调用先于子进程用户代码执行”注释不成立；现有 heartbeat
测试在 spawn 返回后等孙进程存活，只证明它当时存在，不能倒推出在 assign 后才创建。
Unix 的 pre_exec setpgid 没有同一种父侧入组窗口；本项仅针对 Windows。

修复须在允许子用户代码运行前建立本次 owned 树归属，并涵盖准备/入组失败的清理。
新增确定性同步/早派生反控验证归属建立时序，保留无关进程不被清理的对照；
不得用等 heartbeat 或扩大等待时间宣称消除竞态。

### F-816-R1-05 / P3：新增代码格式与公开文档告警未清

**收口缺口；T-02/T-12/T-13，AC-08 健康检查。**
`cargo fmt -p auto-tool-client -p auto-tool-execution --check` exit1，差异在新增
client/src/lib.rs；独立 fixture 的 `rustfmt --check .../src/lib.rs` 亦 exit1。
`cargo doc -p auto-tool-execution --no-deps` 新增三个 broken intra-doc-link warning：
`:17 TreeScope::ProcessGroup` 不存在，`:70/:95 argv[0]` 被当文档链接。
修格式并改正公开 API 文档引用；对新增包及独立 fixture 重放，不要求格式化全仓。

### F-816-R1-06 / P2：捕获 reader/清理失败被吞掉，仍可报告成功或已收尸

**原要求缺口；T-02/T-12，AC-02 当期/AC-07，V-02 reader 错误面。**
执行设施 `lib.rs:379–390` 将所有 read error 当 EOF，join panic 则默认空 Vec，
`run` 仍按根状态返回 Success，捕获内容可被无诊断地截断。客户端 stderr reader
`:249–251` 同样合并 EOF/error，`:428–433` 丢弃 join 结果；`:422` 更忽略 cancel
结果，却无条件生成声称“terminated, reaped, readers joined”的 Timeout。
本项依据完整错误分支检查，不伪称本机发生了真实内核读错/终止权限错误。

修复须把实际 I/O、reader panic 和取消/收尸失败具名传播，清理其他 owned 资源后
准确返回；不能以成功码/空输出/固定 Timeout 文案掩盖失败。应通过可控故障注入
覆盖 reader 读错、join 失败及终止失败的传播，并有正常 EOF 对照。这正是合同原列
reader 错误与资源退出面，不是额外新增的 B 能力。

## 实跑证据与覆盖

| 检查 | 结果 |
|---|---|
| `cargo test -p auto-tool-client -p auto-tool-execution` | 19 个不同测试全过：client4、execution lib2+integration13；doc tests0 |
| `cargo build -p auto-ui-tools -p auto-tool-execution` | 通过；真实工具 SHA 与候选一致 |
| fixture 离线 fresh lock → `cargo test --offline --locked --manifest-path test/fixtures/modularity/tool-client-consumer/Cargo.toml` | fresh43包；真实工具10/10，2.78s |
| `verify-deps.sh`（Windows Git Bash） | PASS，normal/build 无静态 auto-ui-tools、Man/lang/VM/gen/media；client/protocol/execution/project-model 均存在 |
| `cargo clippy -p auto-tool-client -p auto-tool-execution --all-targets --no-deps` | 新包零告警 |
| `cargo check -p auto -p auto-man` | exit0；现有依赖/旧包有预存告警，其源在本 diff 未变；不能宣称全仓零告警 |
| fmt / fixture rustfmt / rustdoc | 上述 F05 未过；`git diff --check` 通过 |
| `python scripts/modularity-audit.py --mode strict --output .../modularity.json` | pass / 0 violations / 2 grandfathered；declared78，metadata cross-check一致 |
| 独立反例 [replay.ps1](replay.ps1) / [probe.rs](probe.rs) | 5 个失败观察，exit1；[完整输出](probe-output.txt)。F01两入口、F02两入口、F03粘包，含 C01/C02/C03 对照 |

最初直接 `--locked` 因 fixture 锁未保留而诚实失败；按 fresh→locked 补跑通过，未把
缺锁作为产品 finding。系统默认 bash 是 WSL，未配置 Linux cargo，脚本初次无输出
exit1；改用 `C:/Program Files/Git/bin/bash.exe` 后脚本通过，不是闭包回归。
额外 `clippy -- -D warnings` 被未改动 auto-val 的六条旧 lint 拦住；以 --no-deps
单独核对新增包，不把依赖旧 lint 冒认为新包缺陷。

反例重放入口（在既有实施树运行）：

```powershell
& D:/autostack/auto-lang/docs/plans/attachments/816/review-r1/replay.ps1
```

脚本只在 TEMP 创建独立 Cargo 项目、引用同一实施树的真实 crate；子进程均有限寿命。
1000ms 断言是给 250ms 限期预留的审查容差，用来区分等待 1800ms 自停，
不是新增 1000ms 支持等级。此观察器不替代正常包测试或永久回归；F04/F06 仍需
实施方增加确定性时序/故障注入测试，不能以这五项变绿宣称六条 finding 都闭合。
执行平台 Windows；Unix 仅源码检查，无 Linux 实机 pass 声明。未运行任何整档。

## AC → T → 交付物 → 当前判定

| AC / 范围 | 任务/交付物 | 结果 |
|---|---|---|
| AC-07 真 M815-A 客户端/来源/独立 Core consumer | T-12；client、fixture、777 | 正控/来源/闭包 pass，寿命与流处理 fail，整体 partial |
| AC-02 当期 argv/cwd/env/stdio/exit/取消/deadline/树/reader | T-02/T-12；execution/client | 原正反控 pass；F01/02/04/06 fail，不完成 |
| AC-03 当期协议版本/缺能力/预算/日志隔离 | T-12；真实 auto-ui-tool consumer | 原控制 pass；混合控制流 F03 fail |
| AC-08 独立阶段验收 | T-13；本次复审 | fail；不能勾选或发阶段 pass |
| AC-01..04 的 B 面及 AC-05 最终审查 | T-03..T-09 | 不在本轮；按既有发布门 pending，不新造上游缺失 finding |
| AC-09 阶段合入/冻结门/Spec/ledger/来源 | T-14 | merge-owned pending，且 T-13 当前尚未通过 |

## 规范增量与遗漏扫描

冻结候选 §5 见 [spec-delta-frozen.md](spec-delta-frozen.md)，其精确 hash 及当前目标
版本见 inputs.json。目标 `docs/specs/tool-execution/project.md` 尚不存在，SD-01-A
可新增；“deadline/owned 树/reader 真终止”目前被 F01/02/04/06 反驳，不能当既成事实
发布。候选明确旧 CLI/服务/发现/bootstrap/profile 仍属 B，与实际无路由切换相符。
SD-02/03/04 本阶段无实现变化，保持后续；SD-05 原 Plan §5.5 称 A 会沉淀实际
协议消费子集，候选 §5 却将 SD-02..05 整体排除。下次送审须给 SD-05-A 的明确
处置（现有 distribution Spec 是否已充分覆盖显式进程消费及尚无安装变化、所需
精确补文），不要用“CLI/服务未搬迁”替代该项本身的理由。未批准砍原 SD-05/B 合同。
这项作为 T-12 规范候选收口检查保留，不要求本阶段提前完成 B 发行 profile。

supersedes_spec_components / new_spec_components / touched_goals 的原路径与
GOAL-003 保留；没有发布 canonical Spec 或写 live ledger。未发现已有 CLI
功能被替换、静态 UI fallback 或 814 未发布接口消费；原命令/adapter 映射及
bootstrap 后置门有明确归属，不当作本阶段已交付。新增调试输出只在具名 fixture
和审查观察器，不在生产 client。六项均为本阶段合同缺口/健康项，登记债务不等于完成。

## 后续与门禁

原树 work 一次修齐上述 findings，加入永久定向控制并回放共享反例，提交新 SHA 后
再作受影响阶段复审。Plan revision 仍为2，不改变合同语义。T-13 pass 后，T-14
才在冻结提交串行执行裸 `cargo t`、必要新包/fixture/闭包门；当前 diff 无
VM/trans/UI-generator/AAVM 触面，tv/tt/tu/taa 不触发（最终并集需复核）。
`cargo tf` 仅主检出到期批量；实际 master 合入/Spec/受管 ledger/发布来源仍 pending。
本轮未实施修复、整档、合入、归档或清树。
