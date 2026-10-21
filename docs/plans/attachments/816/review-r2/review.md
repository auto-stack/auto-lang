# PLAN-816 r2 / M816-A / T-13 独立阶段复审 R2

2026-10-21。结论 **needs_fix**。R1 六项中 **01/02/04 CLOSED，03/05/06 OPEN**。
原五个失败反例已全部转绿，不等于完整不变量闭合；本轮不增加 finding ID。
Plan 保持 `executing`，T-02/T-12 重开，T-01 完成保留，current_step=1/14。

## 冻结身份和复用边界

- scope=`phase`，`PLAN-816:r2:M816-A`，非整 Plan final。reviewer=本聊天 Codex
  `/root`，独立于实施会话；直接读代码、测试和实际反例。具体模型/会话编号未暴露，
  未创建 agent，未修改实施源码。
- 唯一实施树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`；
  reviewed_commit=`4372d56d90670210e90a5a7a97e672cec1867a3c`，repair base=
  `6b4bf9c2fcf3e872b33f6cc8505cf9fa09aa550f`，七文件 +1169/-227。
  整阶段 diff base 仍为 `b65d0e963bbf39f97de5c1d6531997f6f21c760b`。
- 上游 M815-A=`e609caf1af511b96832f511ab9d1fa6c0c11f563`；三个上游 crate
  对该发布点 diff 为空，无 814 WIP。组内 auto-down=
  `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 入场及验证后工作树 status 均为空、HEAD 未漂移。主检出其他工作保留；本轮只改
  Plan/债务簿记与本审查附件，无 canonical Spec 或 live ledger 写入。
- 与 R1 的 87 项输入 hash 对照仅 Plan、候选及上述七个实现文件变化；依赖锁、
  manifest、测试配置、上游来源和 canonical Specs 均未变。真实工具 exe SHA256=
  `a7c070596a74bfc56586e522b89e5cc68240803adc68c9e33117515847faaa6c`，逐字未变。
  本轮重新执行包测试/真实 fixture/闭包/健康/777；旧 CLI/Man check 可复用 R1：
  本次七文件均不在它们已连接的业务路径，旧源码/manifest/依赖/配置未变化。
  输入、技能、Spec 增量冻结 hash 见 [inputs.json](inputs.json)。

## 六项逐项闭合

| Finding | 原例/永久回归 | 不变量与当前结果 |
|---|---|---|
| F-816-R1-01 P1 | 原 F01a/F01b=251/255ms Timeout；blocked stdin/send 永久测试过 | **CLOSED**：run 先启动两个 reader，writer 线程化；client 发送与等待共享绝对截止。独立 C04 双向对照先写 stdout256KiB/stderr128KiB 再读 stdin512KiB，71ms Success，字节全保留。收尾失败残项归06，避免重复计数 |
| F-816-R1-02 P1 | 原 F02a/F02b=195/153ms；orphan 持管道两项永久回归过 | **CLOSED**：ProcessTree 根退出后在收集前终止剩余 owned 后代，正常 Success 保持；heartbeat 真停控制过。非owned持管道时误报收尾完成归06，不要求杀非owned后代 |
| F-816-R1-03 P2 | 原 F03=0ms StreamViolation，粘包/拆包/半帧/坏信封单测过 | **OPEN**：缓冲 drain 修复成立，但 shutdown 消费路径丢弃 Violation 并返回 Success，见下文。原例闭合，完整流错误合同未闭合 |
| F-816-R1-04 P1 | early_spawned_descendant、原树取消回归过 | **CLOSED**：源码确认 CREATE_SUSPENDED 在 spawn 前设置，Job assign 在 resume 前完成；准备/assign/resume 错误走终止挂起根/收尸。对正常受支持进程的入组前用户代码窗口已关闭；没有声称实测所有 OS 故障分支 |
| F-816-R1-05 P3 | 两 crate fmt、独立 rustfmt、两 crate rustdoc 均过 | **OPEN（修复回归）**：新增 fixture 测试产生 unused_mut，见下文；原格式/三条链接告警均已闭合 |
| F-816-R1-06 P2 | 正常 collect_capture 的 read error/panic 与 writer 断管注入过 | **OPEN**：run 超时分支仍吞读错/panic/未完成信号，client stderr 仍吞读错；完成通知代替资源停止的残项有真实反例，见下文 |

## 仍开放的具体证据与修复范围

### F-816-R1-03：关闭阶段坏帧被吞

`crates/auto-tool-client/src/lib.rs:618–641` 的 `join_all_bounded` 对所有
非 Done 消息 `Ok(_) => continue`，包括 `FromReader::Violation`。shutdown
先等根进程退出，再通过该函数收尾，最终仅按根码构造 Success。

共享反例 [probe.rs](probe.rs) 的 `close-bad`：真实子进程先对请求 id1 回答合法
UnknownOperation；客户端触发 EOF 后，子进程向 stdout 写 `ff ff ff ff` 再以0退出。
reader 能解析并排队 TooLarge，但 **shutdown 实际 `Ok(Shutdown { kind:Success })`**。
逐字相同流程、删除这四个字节的 `close-clean` 对照正确 Success。
这仍是“任意非帧字节须具名违规”的原 A 合同，不依赖 progress/并发/B 能力。

修复应在 shutdown/abort 的最终结果中保留控制流错误，保留已处理错误的状态与
重复关闭语义；补请求、EOF关闭、超时收尾三条消费路径的有效/坏帧与已报告错误对照。
只证明 reader helper 会发 Violation 不能证明消费者最终返回了该错误。

### F-816-R1-06：Timeout 返回时自己的 reader 仍活，失败继续被当拆除噪声

`crates/auto-tool-execution/src/lib.rs:409–413` 的
`drain_capture_after_stop` 忽略 `Capture.end`，把 recv timeout/panic 都变为 None；
`:515–528` 的 Timeout 路径忽略两个 reader 及 writer 的收尾失败，仍 `Ok(Timeout)`。
所有线程的 JoinHandle 都已丢弃，无法在该分支观察它们真正结束。

真实反例（同一 probe，Windows）：使用**已公开、且为设施默认值**的
DirectChildOnly；根派生一个继承输出管道、寿命9秒的测试后代，再保持运行，
run 配置 deadline=250ms。只终止直接根是正确 scope 行为，本反例不要求清理非owned
后代；需要停止/准确报告的是 run 自己拥有的两个 capture reader。

- 实际根约250ms被终止，随后每个 reader 各等3秒，**总耗时6268ms**。
- 返回 `Ok(ExecutionResult { kind:Timeout { after:250.8615ms }, stdout:None,
  stderr:None, elapsed:250.8615ms })`，没有 PipesHeld/收尾失败。
- 同一进程线程数在返回前后 **4→6**；非owned后代 heartbeat 仍推进。
  只给本次测试后代写 release 文件，线程数才恢复 **4**，证明新增两线程是悬留的
  capture readers。截图/计时不是依据：计数来自 OS 线程快照，并有释放后恢复对照。
- 反例全部有限寿命，另用 release 提前结束；未终止任意外部服务/无关进程。

即使只考虑 ProcessTree，源码的 recv timeout/disconnected 和 ReaderEnd::Io 分支
依旧会被掩盖；当前的注入测试只调用正常路径 collect_capture，没有覆盖这段生产分支。
此外 client `lib.rs:350–363` stderr reader 仍是 `Ok(0) | Err(_) => break`
随后发送 `()`，无从将日志 I/O 错误具名传播；R1 指出的原位置并未真正修掉。
本轮没有伪称诱发了内核 stderr 读错，后者是源码与通道类型的直接证据。

修复须区分干净 EOF、管道未收尾、读错/panic、终止失败，在正常/超时/关闭各路径
准确传播，保留足以结束并确认自己 reader/writer 的控制与寿命归属；使用实际返回
时刻计算 elapsed。DirectChildOnly 的非owned进程保留规则继续有效。
补正常 EOF、读错/panic、持管道、cancel失败的全路径故障注入；测试观察 own reader
确已停止，不能仅断言调用返回或 helper 输出具名错误。新 PipesHeld 类型本身不等于
原要求的资源已停止，也没有授权将“真退出”降级为带着活线程返回。

### F-816-R1-05：新增 fixture warning

`test/fixtures/modularity/tool-client-consumer/src/lib.rs:395`：

```text
warning: variable does not need to be mutable
395 | let mut client = ToolClient::spawn(spec).expect("spawn te-child");
warning: `tool-client-consumer` (lib test) generated 1 warning
```

来自新增 `shutdown_bounds_pipe_holding_descendants`，是当前 repair diff 新增，
不能归入旧依赖 warning。新增包 clippy 的根 workspace 选择不会包含被 exclude 的
独立 fixture，所以“两 crate 0 告警”不能覆盖它。删冗余 mut 并通过 fixture 本身的
check/test 告警检查即可；无需清仓旧 lint。

## 复现与测试证据

| 门 | 本轮实际结果 |
|---|---|
| `cargo test -p auto-tool-client -p auto-tool-execution` | **32/32**：client9，execution6+17；doc tests0 |
| 真实 fixture `cargo test --offline --locked --manifest-path test/fixtures/modularity/tool-client-consumer/Cargo.toml` | **12/12**，3.44s；上述一条新 warning |
| 原审方 R1 replay | **5/5** 原失败输入通过，exit0；[输出](r1-replay-output.txt) |
| R2 replay | 两个失败控制、两个通过对照，exit1；[输出](probe-output.txt)。C04双向完整流量、close-clean通过；close-bad/Timeout活reader失败 |
| fmt / 独立 fixture rustfmt / rustdoc | 通过，原 broken-link 警告0；`cargo clippy -p auto-tool-client -p auto-tool-execution --all-targets --no-deps` 新包无告警 |
| verify-deps（Windows Git Bash） | PASS；真实进程工具实现不在 client normal/build 静态闭包 |
| 777 strict | pass / violations0 / grandfathered2 / declared78；[原报告](modularity.json) |
| diff/来源 | `git diff --check` pass；上游三crate源与M815-A零漂移，锁/工具hash与R1一致 |

共 **44个不同现有定向测试通过**，反例另计；没有运行整档 t/tv/tt/tb/tu/taa/tf。
执行平台 Windows，Unix killpg/ESRCH 仅源码检查，不冒称 Linux 实机通过。
R2 replay 在原树中运行，TEMP 创建独立消费者项目，引用实际冻结 crate；源码和
返回码持久化在本目录，无实施代码修改：

```powershell
& D:/autostack/auto-lang/docs/plans/attachments/816/review-r2/replay.ps1
```

## AC / Spec / 下一步

- AC-07 仍 partial：真实已发布 tool、版本/能力/预算/独立闭包继续通过，退出与错误
  不变量尚未齐；AC-08/T-13 不通过。B 的原AC/启动门保持，未将缺814/815-B当新缺陷。
- 新增 SD-05-A 精确处置已经对齐：目标 distribution Spec 增真实进程消费边界，
  A安装/DELIVERY闭包不变，B完整profile保留。候选修订不是新的语义合同批准。
- SD-01-A 仍声称 reader 真停止、所有失败具名；以上06反驳这些断言，本轮不准发布。
  冻结当前§5候选于 [spec-delta-frozen.md](spec-delta-frozen.md)，源版本/hash见 inputs。
  原 supersedes/new_spec_components 路径及 GOAL-003 保留，无 live Spec/ledger写入。
- 原树 work 先诊断“helper 已报错，但生产收尾路径丢弃/读线程仍活”的覆盖断层，
  维护六项闭合表；一次修齐03/05/06，补完整错误消费和退出观察的永久控制后提交新SHA，
  再按受影响范围复审。01/02/04保持关闭，不重复立项。
- Plan executing1/14，T-02/T-12重开，T-13/T-14未完成。T-13阶段pass之后才由T-14
  执行冻结裸cargo t、新包/外部fixture/闭包门、master实际合入/Spec受管ledger/发布
  来源收据；当前diff无VM/trans/UI生成/AAVM触面，最终并集复核。tf仅主检出到期批量。
  未整档、修实现、合入、归档或清树。
