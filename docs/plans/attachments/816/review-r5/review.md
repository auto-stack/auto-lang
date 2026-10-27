# PLAN-816 r2 / M816-A / T-13 独立阶段复审 R5

2026-10-21。结论 **needs_fix**。本轮直接独立核验当前冻结提交，替代同提交 R4
的阶段 pass 作为当前结论。仍是 M816-A，不是整 Plan 最终验收；B 面未交付不作为
新增 finding。两个阻塞项：**F-816-R3-02 重开，F-816-R5-01 新增**。

## 基线、身份与证据边界

- reviewer：本聊天 Codex `/root`，独立于 aa2edc8fb/6e03f5f26 实施及其 R3/R4
  同会话审查；未创建 agent，具体模型/会话编号未暴露。结论来自源码、真实消费者、
  确定性注入，不把已有 pass、任务勾选或压力轮数当独立证据。
- review_scope=`phase`，对象 `PLAN-816:r2:M816-A`；唯一树
  `D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`。
- reviewed_commit=`6e03f5f268ee0eb4a3c2b9f64d4924c226977b72`；最近修复基面
  `aa2edc8fbc3889f72fcf3dcb4b82887dd28d3bd9`，三文件 +404/-143；整阶段基面
  `b65d0e963bbf39f97de5c1d6531997f6f21c760b`。
- M815-A=`e609caf1af511b96832f511ab9d1fa6c0c11f563`，三个上游 crate 相对发布点
  diff 为空，无 814 WIP；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
  入场/收尾工作树 clean，HEAD 一致，适用 AGENTS 无改动。
- [92项输入指纹](inputs.json)绑定代码、依赖锁、当前 Specs、候选、技能及回放程序。
  与 R2 相比，主检出 Plan/候选及五个实施文件、fixture锁发生变化；根锁/协议来源/
  旧CLI-Man源/配置未变。旧 CLI/Man check 仅按这些不变输入复用 R1/R2 证据。
  当前所有新包、fixture、健康、闭包和 777 重新运行。
- 原工具 exe SHA256 保持
  `a7c070596a74bfc56586e522b89e5cc68240803adc68c9e33117515847faaa6c`。
  本轮不修改实施源码、canonical Spec 或 live ledger，只写审查附件与计划簿记。

## F-816-R3-02 / P2 / OPEN：错误返回仍带活 writer，早退清理仅发 stop

**旧不变量未闭；T-02/T-12/T-13，AC-02 当期/AC-07/AC-08。** R3 的同一条
“任何返回路径不带活的自有线程”仍未成立；由实机证据补强，当前严重性 P2。

设施 `WriterWorker::spawn`（`lib.rs:694–730`）在同步 `write` 前检查 stop，
一旦进入满管道写就无法再读标志；`stop_and_join`（`:755–769`）到期只返回
PipesHeld，保留仍活的线程句柄，随后对象析构又将 JoinHandle 丢弃。
客户端 `write_frame_chunked`（`:840–867`）甚至不接收 stop，任务内无法停止。
就绪轮询/16KiB 分片并不保证同步 write 可被终止。

真实消费者 [writer-probe.rs](writer-probe.rs)：

1. 使用已公开的 **DirectChildOnly（设施默认值）**，根派生一个继承 stdin
   读端、但不读取的后代；后代有限寿命9秒，并有测试专属 release 文件。
2. 输入1MiB，deadline/request_timeout=250ms。根被正确终止；非owned后代
   不应被杀，观察其 heartbeat 保持推进。
3. 两个公共入口实际结果：

| 入口 | 返回与耗时 | own线程返回前后 | 释放测试后代读端后 |
|---|---|---|---|
| `run` | `Err(PipesHeld { after:3s })`，3254ms | **4→5** | 恢复4 |
| `ToolClient::request_unknown_operation`，随后 drop client | 同类错误，3272ms | **4→5** | 恢复4 |

两入口返回后各有一个 writer 仍活；release 后恢复基数，排除运行时后台线程误计。
这是 R2 已审“非owned持管道、自己的线程仍应退出”场景的写端，不是新规模或新
平台要求。**具名 PipesHeld 是准确错误，但不能证明线程已停止。** 当前永久测试
`writer_on_held_pipe_is_bounded_and_named` 明确接受这个带活 writer 的分支，随后
才 drop keeper 解除阻塞；它把真停要求缩成“有界+具名”，无 Plan revision/用户
批准依据，不能作为本条闭合 oracle。

早期失败分支也未完成 R3 所称的“stop+join 后传播”：

- `finish_early`（设施`:802–822`）忽略全部 stop_and_join 的失败；writer 仍可活。
- client `shutdown` 的 writer_done 超时/断开直接返回（`:742–753`）；三个
  wait/terminate 失败分支仅 `stop_all()` 后 return（`:764–795`），没有 drain/确认
  完成。stop 标志不等于线程终止，不能将函数调用次数当作清理成功。
- 设施超时分支 `:888` 先忽略 terminate 结果，再无界 child.wait（`:889`）；新的
  finish_early 只覆盖 wait 返回 Err 后，不能覆盖终止失败导致 wait 不返回。此点是
  错误分支源码证据，本轮未伪称实测了 OS terminate 权限错误。

修复应在所有返回出口保有并确认自己的 writer/reader 已结束，阻塞写须可中止，
清理失败与原错误准确传播；保持 DirectChildOnly 非owned进程存活规则。永久回归
必须观察返回时线程数/完成状态，而非只接受 PipesHeld 后再释放 keeper；补
wait/terminate失败的真实清理编排注入。无需为此修改原合同或提前做 B。

## F-816-R5-01 / P2 / OPEN：stop 排空新分支丢失读错及半帧终态

**R3 修复新增分支的错误传播回归；T-02/T-12，AC-02 当期/AC-07。**
R3-01 对已到达完整字节的排空修复有效，但新增 drain helpers 返回 `()`，只把
`Err(_)` 当 break；随后无条件 Stopped/Ok/Done，将流失败从结束语义中抹掉。

三个确定性测试均调用**原封不动的生产函数**：

| 控制 | 入口与输入 | 实际 | 原合同预期 |
|---|---|---|---|
| stop读错（设施） | CaptureWorker，通道同步保证先置 stop，再于 drain_readable 中 read=Err | `ReaderEnd::Stopped` | ReaderIo/ReaderEnd::Io，不能冒充正常停止 |
| stop读错（client日志） | stderr_session，stop=true、就绪=true、read=Err | `Ok(())` | 具名日志 I/O 错误 |
| stop半帧（client控制流） | reader_session，先置stop、closed=true，预置长度64的前缀+7字节不完整JSON | **仅 `[Done]`** | 保留 TruncatedEof 证据，由 natural_exit 消费方判违规；我方 kill 才可按既有规则忽略 |

定位：设施 `drain_readable :507–517` / CaptureWorker stop `:555–560`；client
`stderr_session :886–900`、`drain_readable_into :955–967`、`reader_run :1044–1048`。
normal 路径的 reader I/O/半帧测试仍通过，因为它们没有进入新 stop-drain 分支。
在 shutdown 的自然退出语境，半帧证据丢失就可能把无效控制流当 Success；这里不
要求实现任何 B progress/交互能力，只保持 A 已有的错误和流终态合同。

修复应让排空返回结构化结束状态（数据/EOF/停机/错误/截断），由所有生产消费
路径传播；保留自然EOF与我方kill的既有区分。把 stop=false/true、干净EOF/读错/
半帧/合法整帧放在同一回归矩阵，不能仅证明有效预置字节不丢。

## 验证方法与结果

- **53个不同现有定向测试全部通过**：client12 + execution9/17 + 真实fixture15。
  原 R1五失败输入全通过；R2四控制（双向大流量、关闭正反控、reader真停）全通过。
  [R1输出](r1-replay-output.txt)、[R2输出](r2-replay-output.txt)。这些结论保持，
  不将此前已闭的原例复活。
- 新实机 writer 两入口失败，[输出](probe-output.txt)；其后跑确定性注入，三项
  **全部失败**，[完整输出](injection-output.txt)，cargo exit101。
- 注入方式：TEMP 内复制 production `lib.rs` 完整内容及原平台模块，**只在文件末尾
  追加 reviewer 测试**。生产函数体未替换；假 Read/就绪源只提供
  异常输入。设施用通道栅栏保证 stop 先于读错，无 sleep 竞争。原代码树保持 clean。
  单元测试只过滤 reviewer 模块，无整档；结束线程均在测试内 join，无遗留注入线程。
- 回放统一入口 [replay.ps1](replay.ps1) 在既有树运行，消费者依赖实际crate；读错/
  半帧副本测试使用同一源码，带 `--no-fail-fast` 使两目标均执行。当前完整回放
  应非零（两入口失败+三个注入失败），不把输出 banner 当通过依据。
- fmt、fixture rustfmt、新包 clippy --no-deps、两crate rustdoc均通过/零告警；
  verify-deps PASS；777 strict=pass/78边/0违规/2grandfathered，见
  [原报告](modularity.json)；git diff --check 与来源核对通过。
- 实机平台 Windows；Unix仅源码检查。没有运行任何 t/tv/tt/tb/tu/taa/tf整档。

## 闭合、Spec 与后续

- R1/R2原输入及 01/02/04/05 的已验证行为保持；03完整坏帧处理主面保持，半帧
  stop相邻面归 R5-01；06正常路径错误/reader退出主面保持，writer及drain错误残项
  分别归 R3-02/R5-01。R3-01 的有效数据排空修复保持，不重复记为旧数据丢失。
- R4 N-2 broken拒绝文案注记不作为本轮额外阻塞项；N-1/压力轮数不能代替上述
  新分支的确定性错误注入。R4 的同上下文机器测试通过不覆盖本轮两个不变量缺口。
- AC-07 partial，AC-08/T-13未通过；T-02/T-12/T-13重开，保留T-01，
  `executing1/14`。原R4 pass保留为历史，新R5 needs_fix是当前结论。
- SD-05-A 仍对齐；冻结当前候选 [spec-delta-frozen.md](spec-delta-frozen.md)。
  SD-01-A 的真停/错误具名断言仍被实证反驳，不能发布。supersedes/new_spec路径及
  GOAL-003保持，无 canonical Spec/ledger写入；B任务/依赖门保持。
- 原树work先提交覆盖诊断：为什么“PipesHeld即过”和“stop drain返回()”会漏掉
  真停/错误终态；将本轮共享输入纳入永久回归，修齐两项并提交新SHA后再独立复审。
  不把相同同步API的限制当作未经批准的验收降级。
- T-14仍待阶段pass后的冻结裸cargo t/新包及fixture闭包门、实际master合入、SD/
  受管ledger/发布来源收据。当前diff无VM/trans/UI生成/AAVM触面，最终并集复核；
  tf仅主检出到期批量。未修实现、合入、归档或清树。
