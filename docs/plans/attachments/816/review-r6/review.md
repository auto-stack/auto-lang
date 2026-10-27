# PLAN-816 r2 / M816-A / T-13 独立阶段复审 R6

2026-10-21。结论 **needs_fix**。**F-816-R5-01 CLOSED；F-816-R3-02 OPEN；
新增 F-816-R6-01（Linux 编译回归）**。本轮是 M816-A 阶段复审，不是整 Plan
最终验收；B 的既有启动门保持，未把未发布 B 能力当新缺陷。

## 基线与身份

- reviewer=本聊天 Codex `/root`，独立于实施会话，具体模型/会话ID未暴露；未创建
  agent、未修实施源码。结论依据实际代码、原例、确定性调度与编译对照。
- 唯一树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`。
  reviewed_commit=`a972286fd5f0aa05338ef4ca0c09c54a56de0027`；repair base=
  `6e03f5f268ee0eb4a3c2b9f64d4924c226977b72`，七文件 +755/-140。
  整阶段 base=`b65d0e963bbf39f97de5c1d6531997f6f21c760b`。
- M815-A=`e609caf1af511b96832f511ab9d1fa6c0c11f563`，三个上游crate源零漂移，
  无814 WIP；同组 auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
  入场/收尾工作树 clean，HEAD一致。主检出其他工作保留。
- [inputs.json](inputs.json)冻结92项源码/锁/Specs/候选/技能/新回放指纹及实际
  Rust版本/本地目标清单，包含新增 win_stdin.rs。旧 CLI/Man源码、根manifest/
  根锁、上游和验证配置未变，适用旧check证据按R5理由复用；修改的新包/fixture/
  健康/闭包/777均重新验证。没有改 canonical Spec 或 live ledger。

## F-816-R5-01 / CLOSED：stop 排空的读错和半帧证据已保留

结构化 `DrainEnd::{Empty,Eof,Io}` 接入三个生产流；读错不再变为正常停止，半帧
在 EOF/owned_closed 语境保留 `TruncatedEof`，仍由 natural_exit 消费方判定。

原 R5 三注入直接重放，全过：CaptureWorker stop end=Io；stderr stop=Err；
reader stop消息=`[TruncatedEof { trailing:11 }, Done]`。
新增永久单测与原有效字节/完整帧控制也通过。原 R5 replay 仅机械增加新模块
win_stdin.rs 的复制，没有改变原探针和断言语义。

## F-816-R3-02 / P2 / OPEN：一次性取消仍漏掉“取消后才进入写”的合法调度

**原不变量残项，T-02/T-12，AC-02 当期/AC-07。** 原 R5 的“写已阻塞时取消”
两个实机入口现在确实通过（276/266ms，线程4→4、非owned持有者继续存活）。
这证明已挂起写的取消成立，未证明与后续写入的时序闭合。

定位：设施 `WriterWorker::spawn_with :875–889` 先读 stop，再调用
`StdinPipe::write_stoppable`；Windows 实现 `:757–762` **忽略 stop**，直接同步
write。`stop_and_join :949–963` 置标志后仅调用一次 CancelIoEx，随即 take/drop
取消凭证。若取消时写还没进入内核，返回 false 被忽略，随后写线程继续调用真实
write，既没有凭证重试、也没有针对这个窗口的停机协调。

确定性反例 [cancel-race-injection.rs](cancel-race-injection.rs)：

1. TEMP中复制实际生产源码，只追加测试。真实 te-child stdin-holder，
   DirectChildOnly，有限寿命5秒的非owned后代持读端；先成功写入64KiB。
2. 包装器仅以通道栅栏暂停在 **Worker 已读 stop=false 后、真实 write 前**，
   后续仍调用生产 `StdinPipe::write_stoppable`。这是合法调度延迟，没有替换
   write、cancel 或 Worker 函数体。
3. 按生产顺序先终止/收尸直接根（后代保留），再调用 stop_and_join；等待共享
   cancel槽已被消费关闭，才放行真实写。
4. 使用生产同一个 `TEARDOWN_GRACE=3s`，实际 **`Err(PipesHeld { after:3s })`，
   `writer_still_in_write=true`**，cargo exit101。不是较短测试宽限造成的误判。
5. 测试在断言前等待有限后代自停、保留并join writer，因此失败测试也没有留下线程。

原始结果见 [cancel-race-output.txt](cancel-race-output.txt)。成功预填是前置断言，
不作任意平台管道容量承诺；是否失败由真实 write 和线程状态判定。取消前没有挂起
写并不等于连接已安全停止，单次正常时序实测不能覆盖这个顺序。

修复须协调 stop 与写入进入内核之间的窗口，使**取消先到/写先到/写完成**三种
顺序都能确认真停。保留原R5 writer入口与该栅栏反例作永久控制。

R5已指出的异常返回编排也仍有残项（源码证据，未伪称 OS 故障实测）：

- client shutdown仍在 `wait_writer_done(writer_grace)?` 直接早退（`:766`），
  `wait_deadline Err` 仍是 stop_all 后直接 return（`:796–802`），没有走新增
  cleanup_then_error。不能凭该helper存在就宣称全部出口已覆盖。
- bounded_cancel虽限制自身等待，但若失败返回，run的 proc 尚未reaped，随后
  `OwnedProcess::drop :1397–1404` 又无界 `child.wait()`；终止失败时的外层生命周期
  仍可能悬挂。需要覆盖返回后的析构，不能只测试有界helper本身。
- 创建取消凭证的 `.ok()` 与 Clone失败变空句柄仍会静默降级取消能力，应纳入同一
  初始化/错误退出核查；本轮未诱发 DuplicateHandle 的 OS 失败，不另立新finding。

这些仍属于R3-02的全路径停止/失败传播，不批准将 PipesHeld 或有限等待当作真停。

## F-816-R6-01 / P1 / OPEN：本轮 Windows 类型泄漏至 Unix，Linux 无法编译

**修复引入的回归，T-02/T-12，AC-07/原跨平台支持面。** 本机已安装
`x86_64-unknown-linux-gnu` 标准库，离线 cargo check 实际报错，不是缺环境或链接器。

`win_stdin` 模块仅在 `#[cfg(windows)]` 下声明，却被无平台条件的签名引用：

```text
error[E0433]: cannot find module or crate `win_stdin` in this scope
  auto-tool-execution/src/lib.rs:407
  pub fn cancel_handle(&self) -> Option<win_stdin::CancelHandle>

error[E0433]: cannot find module or crate `win_stdin` in this scope
  auto-tool-execution/src/lib.rs:859
  cancel_slot: Option<Arc<Mutex<Option<win_stdin::CancelHandle>>>>
```

此外 client `writer_cancel`字段 `:280` 无条件引用仅Windows存在的
`auto_tool_execution::win_stdin_reexport::CancelHandle`；上游执行crate先失败，
本轮不伪称编译器已到该条client诊断，但修复范围必须覆盖两包。

对照：以 **上一冻结提交6e03f5f26的原执行crate源码** 作TEMP纯源码fixture，同目标
check **exit0**（三条旧warning）；当前真实consumer check **exit101**。
原始单次 [当前结果](linux-check-output.txt)、[基线结果](linux-baseline-output.txt)，
以及可直接重放的组合 [platform-replay-output.txt](platform-replay-output.txt)。
没有创建第二Git工作树或执行Linux程序，只作编译验收。

修复应让平台专属类型/取消句柄实现正确隔离或提供平台中立抽象，再对**真实两包/
独立consumer**做Linux check，避免只修第一个错误。保留Windows现有正反控及
Unix非阻塞停止路径。该要求是已有Unix支持的保持，不是新增跨OS生产实机认证。

## 验证与复用

| 检查 | 本轮结果 |
|---|---|
| 包定向测试 | **42/42**（client15，execution9+18） |
| 真实工具fixture | **16/16**，3.60s，0新告警 |
| R1 replay | 原五失败输入全部通过；[输出](r1-replay-output.txt) |
| R2 replay | 四控制全部通过；[输出](r2-replay-output.txt) |
| R5 replay | writer两入口及三注入全过，exit0；[输出](r5-replay-output.txt) |
| 新取消顺序注入 | 失败，exit101，3s后writer仍活；不是原R5示例复发 |
| Linux check当前/修复基线 | 当前exit101 / 基线exit0 |
| Windows fmt/fixture rustfmt/clippy --no-deps/rustdoc | 通过、零新增告警 |
| verify-deps / 777 strict | PASS / pass（78边、0违规、2grandfathered） |

共 **58个不同Windows现有测试通过**，不能覆盖新增反例。没有运行任何整档
t/tv/tt/tb/tu/taa/tf；没有Linux运行通过声明。阶段Spec候选冻结见
[spec-delta-frozen.md](spec-delta-frozen.md)，SD-05-A边界仍对齐；SD-01-A跨平台/
真停/全出口错误断言未成立，不能发布。路径元数据/GOAL-003保持，无live知识写入。

回放入口：

```powershell
& D:/autostack/auto-lang/docs/plans/attachments/816/review-r6/replay-cancel-race.ps1
& D:/autostack/auto-lang/docs/plans/attachments/816/review-r6/replay-platform.ps1
```

下一步：原树work一次修齐取消时序/全生命周期清理及Unix编译回归，先补“已挂起
取消≠未来写安全停止”与cfg覆盖缺口的诊断/永久控制，提交新SHA后独立复审。
F-R5-01保持关闭；AC07 partial、AC08 fail，T02/T12重开，T13未完成，T01保留，
executing1/14。T14冻结裸t/新包fixture闭包/实际合入/SD/受管ledger/来源发布仍
pending；当前无VM/trans/UI生成/AAVM触面，最终并集复核，tf仅主检出到期批量。
本轮未修实现、合入、归档或清树。
