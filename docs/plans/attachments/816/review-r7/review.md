# PLAN-816 T-13 R7 — needs_fix

2026-10-21，revision 2，**review_scope=phase / M816-A**。本聊天 Codex /root
独立于实施会话，从冻结代码和共享输入重建结论；具体模型/会话 ID 未暴露，未创建 agent。
本轮是既有 R1–R6 序列的 R7，不因 R3/R4 实施会话内的检查而重置历史。

- 工作树：`D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`；入场及收尾均 clean。
- 冻结提交：`05d3940c6f57254bf74027079d4d00ac70ca8bac`。
- 修复基线：`a972286fd5f0aa05338ef4ca0c09c54a56de0027`；两文件 +100/-77。
- 阶段基线：`b65d0e963bbf39f97de5c1d6531997f6f21c760b`。
- M815-A：`e609caf1af511b96832f511ab9d1fa6c0c11f563` 为祖先，tool/protocol/model 三包源码零漂移。
- auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- [94 项输入指纹](inputs.json)，[冻结阶段规范增量](spec-delta-frozen.md)。Plan 指纹取自本轮簿记之前。

| Finding | 本轮结论 |
|---|---|
| F-816-R6-01 / P1 Linux 无法编译 | **CLOSED**：真实两包及独立 consumer 的 Linux target check exit 0；不是仅检查第一个错误 |
| F-816-R5-01 / stop 读错、半帧被丢弃 | 保持 CLOSED：三原注入通过 |
| F-816-R3-02 / P2 自有线程与错误退出 | **OPEN**：原 R6 示例闭合，但内层检查到入核之间仍可漏取消；客户端终止失败仍可能进入无界 wait |
| F-816-R7-01 / P3 新增平台告警 | **OPEN**：Linux 比 6e03 基线新增 unused variable / dead field 两条 |

## F-816-R3-02：入口检查不等于与实际写入协调

影响 T-02/T-12、AC-02 当期/AC-07；这是原停止不变量残项，未增加新寿命或规模合同。

R6 共享反例暂停在调用 `StdinPipe::write_stoppable` **之前**。新增入口检查确实使
该示例通过：`Stopped{written:0}`、`writer_still_in_write=false`，见
[原例重放](r6-race-output.txt)。R5 已挂起写的两个公共入口亦通过，线程 4→4。

但生产 `crates/auto-tool-execution/src/lib.rs:763–767` 仍是两个独立动作：

```text
if stop.load(SeqCst) { return Ok(0); }
self.inner.write(buf)
```

线程可在读到 false 后、实际 WriteFile 之前被调度暂停。停止侧
`:963–973` 置 stop、取消一次、take/drop 凭证；取消时没有挂起写，随后放行的
真实 write 便没有取消覆盖。SeqCst 不把检查和同步系统调用合成原子操作。
源码注释“未观察到标志的入核写必已被随后的取消覆盖”遗漏了这一顺序。

独立确定性反例 [replay-after-check.ps1](replay-after-check.ps1) 沿用 R6 的输入、
生产 `TEARDOWN_GRACE=3s`、终止根后再停 writer 的顺序和断言，只将调度点移到
**生产 Windows 内层 stop 检查之后、同一条真实 write 之前**：

1. TEMP 复制冻结源码，仅插入两行测试调度 hook，再追加测试模块。
   [完整插入 diff](instrumentation.diff)；没有替换 stop/write/cancel 的实现，未修改工作树。
2. 真实 te-child、DirectChildOnly、有限寿命 5 秒的非 owned 后代持读端；预先成功写入 64KiB。
3. writer 到达 hook 后，先终止/收尸直接根，再执行生产 stop_and_join；确认取消槽
   已被消费关闭后才放行。等待通道只模拟合法的线程调度延迟，不改变返回分类。
4. 实际 **3000ms 后 `Err(PipesHeld { after:3s })`，`writer_alive_at_return=true`**，
   测试断言失败 / cargo exit 101。[原始输出](after-check-output.txt)。
5. 失败测试在断言之前等有限后代退出并 join writer，不留下测试线程。

这证明旧示例修好，不证明完整不变量修好；未把任意管道容量当作跨平台承诺。
客户端也通过同一 StdinPipe 原语写入，停止侧同样只作单次取消，故同一设计缺口影响两消费方；
本轮新调度注入的实际运行入口为设施 WriterWorker，不伪称另跑过客户端内部 hook。

### 同一 finding 的错误路径核查

本轮已修：client shutdown 的两处早退现在调用 cleanup_then_error；OwnedProcess Drop
使用有界 wait_deadline，原析构无界 wait 已移除。WriterWorker 的 outcome 经 join 返回，
完成通知不再被 stop_and_join 二次消费；普通完成、停止与旧错误控制通过。

仍有源码可定位的未闭路径：`auto-tool-client/src/lib.rs:618` 的超时收尾，以及
shutdown 超宽限分支 `:790`，均调用 `OwnedProcess::cancel`。后者
`auto-tool-execution/src/lib.rs:1369–1373` 先保存 terminate 的失败，再无界
`child.wait()`，最后才传播终止错误。若终止失败且子进程继续运行，超时调用仍可
一直等待，尚未进入本轮改过的 Drop。run 内部的 bounded_cancel 未被这两条客户端路径复用。
这是失败分支的源码证据，本轮未诱发真实 OS 终止失败，不把它写成实机错误注入结果。

R6 已要求的取消凭证初始化失败也未改：take_stdin 的 `.ok()` 和 Clone 失败返回空句柄
仍会静默丢掉取消能力；未实测 DuplicateHandle 失败，不另立独立 finding。

修复应以可证明的写入/取消协调闭合窗口，或使用可停止的写入机制；不能只再加一次
stop 检查或把 PipesHeld 当作线程已停。保留“入口前暂停”正控、“内层检查后暂停”
负控、已挂起取消、正常写完/EOF，并覆盖两消费方的超时和关闭路径。客户端 deadline
路径的终止失败也须有界且具名传播。永久控制应断言真实线程退出，不只断言调用返回。

## F-816-R7-01：Linux 修复新增两条告警

影响 T-02/T-12、AC-08 健康检查；修复引入的 P3 回归，原 Windows 格式/文档问题仍关闭。

[Linux 新旧同目标输出](platform-output.txt) 中，6e03 原执行源码为 3 条旧告警；
当前真实执行包为 5 条，其中仅以下两条为本轮新增：

- `auto-tool-execution/src/lib.rs:1117`：`cancel` 在非 Windows 下未使用。
- 同文件 `:837`：WriterWorker.cancel 在非 Windows 下从不读取。

平台中立签名修复了 E0433，但两处 Windows 专用状态没有同步隔离。应按平台收敛
状态/局部变量；不要求清除旧的 unused_mut、POLLIN、POLLOUT 三条，也不把 CLI/Man
旧告警和依赖 future-incompatibility 提示计入本项。Linux check 通过并不代表零新增告警。

## 验收与验证矩阵

| AC / 任务 | 当前代码/证据 | 结论 |
|---|---|---|
| AC-02 当期 / T-02 | 执行设施 argv/cwd/env/stdio、正常/非零/双向输出、deadline、owned 树、reader/writer 故障及退出；原例通过，新取消时序失败 | partial |
| AC-07 / T-12 / V-05 | 真实 auto-ui-tool describe/validate、版本/能力前置拒绝、cancel 诚实、预算、控制流/日志、独立闭包、寿命；58 现有测试和旧反例通过，停止不变量仍未闭 | partial |
| AC-08 / T-13 | 冻结代码、来源/规范/遗漏/健康独立检查；F-R3-02、F-R7-01 未闭 | fail |
| AC-09 / T-14 | 需阶段 pass 后冻结门、实际发布和知识沉淀 | pending / merge-owned |
| 原 AC-01..04 B 其余内容、AC-05/06 | CLI 改薄、发现/bootstrap/profile、管理服务/watch 及整 Plan 收口 | 原阶段门 pending，未当新增 finding |

| 检查 | 本轮原始结果 |
|---|---|
| 包定向测试 | [42/42](package-tests.txt)：client 15，execution 9+18 |
| 真实工具 consumer | [16/16](fixture-tests.txt)，driver 先构建实际二进制，锁定消费 |
| R1 原五反例 / R2 四控制 | [全通过](r1-replay-output.txt) / [全通过](r2-replay-output.txt)，含正常/小发送/分次坏帧对照 |
| R5 两 writer 入口 + 三 stop 注入 | [全通过](r5-replay-output.txt)：run/client 326/256ms，线程4→4，非owned持有者存活 |
| R6 原取消栅栏 / Linux 编译 | [通过](r6-race-output.txt) / [exit0；新增2告警](platform-output.txt) |
| R7 检查后取消栅栏 | [失败](after-check-output.txt)，writer 在返回时仍活 |
| Windows fmt/fixture rustfmt/clippy --no-deps/rustdoc | [全部 exit0](gates.json)，新两包零告警 |
| CLI/Man 定向 check | [exit0](cli-man-check.txt)，旧告警不判为本轮新增 |
| consumer 闭包 / 777 strict | [PASS](deps.txt) / [pass，78边、0违规、2grandfathered](modularity.json) |

共 58 个不同的现有 Windows 测试通过，不将重复重放或 R7 失败反例算入该数字。
本轮未运行整档 t/tv/tt/tb/tu/taa/tf；Linux 只编译，未声称 Linux 运行认证。
原输入指纹比较只变 Plan、候选以及本轮两源码，依赖/锁/配置/其余测试未变。
当前两包、真实工具消费、健康/闭包/边界及 CLI/Man 检查均重跑，未仅复用旧总数。

## 规范增量与下一步

SD-05-A 进程消费边界与当前闭包一致；A 未改发行安装清单，B 的 profile/bootstrap
等内容仍按原门。SD-01-A 的“真停/有界错误清理”断言尚不能发布，不能削弱候选
来绕过已批准合同。SD-02/03/04 仍属 B；元数据路径和 GOAL-003 保持原合同。
冻结规范候选 SHA256：`cbc62a16540f5476868fb8eb9eb68b5cbd6d3999d81920d4875f4f49fdd64e59`。
未改 canonical Specs 或 live ledger。

连续失败诊断：R5 只验证“已挂起→取消”，R6 包装栅栏位于被新增检查覆盖的入口外，
R7 将实际检查后仍存在的窗口具体化；需要调整取消设计及覆盖矩阵，不能将单个旧例
转绿当作全顺序闭合。平台恢复编译后须同时比较净新增告警。

返回原树 work 一次修齐 F-R3-02 和 F-R7-01，补永久控制并提交新 SHA 后再受影响复审。
T-02/T-12 重开，T-13 未完成；T-01 保留，`executing / current_step=1 / total_steps=14`。
F-R6-01 和 F-R5-01 保持关闭。通过阶段审查后，T-14 才在冻结提交执行裸 cargo t、
实际触面/新包与 consumer 门，以及实际合入/SD/受管 ledger/发布来源读回；当前无
VM/trans/UI生成/AAVM触面，最终并集复核，tf 仅主检出到期批量。
本轮没有修改实施、合入、归档或清树。
