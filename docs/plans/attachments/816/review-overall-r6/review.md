# PLAN-816 整体 B-R6 受影响复审

**结论：needs_fix。04 的 Windows watch 残项 CLOSED；Unix 残项因修复引入管道所有权回归仍 OPEN（沿用 F-816-B-R1-04 / P2）。07 保持 CLOSED。**

受审 `0e50bedd0e3775caf3be5ca172af2e16e125b718`，差分基线 `4b83659882901dae487d3c7e56a0bcbcbc11f1bf`，Plan r2。唯一实施树 `D:/autostack/.wt/lang-816/auto-lang` / `codex/plan-816-dev`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。本聊天 /root 独立核验实施交接；未建 agent/第二 checkout，未修改实施源码，模型及会话 ID 未暴露。review_scope=final 受影响核查；当前 work-owned 验收未完成，不能授整 Plan 最终 pass。

[输入与等价性](inputs.json)、[冻结 Plan](plan-frozen.md)、[冻结规范增量](spec-delta-reviewed.md)、[最终身份与产物](final-state.json)。本轮 diff 仅 `crates/auto/tests/plan816_cli_driver.rs`，其新源码 SHA256 为 `b5fda2448dde33448a175635221b5564216cb93a2887a51b7c5c9bb61a4e590c`。实施前后 HEAD/clean、依赖、配置与被测输入均核对，详见最终状态。

## 04：Windows 已闭，Unix 仍有一个 P2 残项

违反规则仍是 AC-02/04、V-02/04、T-07：测试 driver 必须保留子进程正常输出/退出语义，reader 真停且拥有有效管道；不能以提前关闭输入、破坏正控换取线程快照无残留。这里是验收 driver 回归，**不据此宣称生产 Unix CLI/service 实现故障**。

### Windows 与内容判据通过

- [watch 外部持写](watch-external-result.json)：整个新 driver 原样复制；探针仅按当前真实 watch 调用机械改为 `spawn_lines` / `drain` 字段。真实 DuplicateHandle 将 stdout 写端交给测试自己独立拥有的 receiver；转交 marker 确认后 release **5.5324ms**，Thread32 基线、返回后及100ms后均相同，`reader_residue=[]`。receiver 未被并入 owned 树、只由探针最终清理。R5 的5秒后残活例 CLOSED。
- [Windows run 外部持写](external-reader-result.json)：**217.9ms**、code=-1，原 stdout 保留、drains=[true,true]、OS线程回基线；保持 CLOSED。
- [owned 后代](reader-lifetime-result.json)：原200ms输入约266ms返回 code=-1/timeout，输出保留，后代心跳4→4、drains true；原23秒持管道 watch 在约20.05秒正确失败并收尾。
- [真实 CLI 四项](positive-driver-results.json.txt)：watch 内容再生、裸脚本 argv、serve/req、真实工具 gen 全过；新 driver 全文及断言未经修改。
- [旧 SFC 负控](watch-oracle-proof.json)：真实输入已改变，但故障宿主保留旧字节（input_changed=true、incremental_write=false、artifact_still_old=true），新测试正确 exit101；不拿旧冻结测试代替新 oracle。
- [总截止相邻控制](adjacent-watch-results.json)：合法 ready/内容更新 exit0；22秒持续噪声且不 ready 的宿主约20.12秒 exit101、没有变更 audit；持管道例有界。首次装配沿用脚本的 audit 文件未在两案间删除，导致噪声案读到上一案记录；原记录 [单列排除](adjacent-watch-stale-audit-excluded.json)，仅修探针每案清理 audit 后重放，不把旧 audit 作为本案发生变更的证据。

### Unix 管道所有权被提前释放

`plan816_cli_driver.rs:159–161` 的 Unix `build_drain` 闭包只捕获 `unix_fd/sink/flag`；`:219–223` 的 `build_lines` 同样只捕获 fd 等值。参数 `r`（实际为 ChildStdout/ChildStderr）没有移入线程，也没有保存在 Drain 中。创建函数返回时 `r` 被 drop，关闭真实读端；线程之后 poll/read 的只是失效或可复用的整数 fd。`Drain.fd` 保存数字不延长管道对象寿命。编译器的两条 `unused variable: r` 与此吻合；这不是尚待验证的 poll 调度猜想。

**原反例重放**：[unix-reader.py](unix-reader.py)、[原生 Linux 结果](unix-reader-result.json)。原整个新 driver 前缀不改，通过 WSL Ubuntu22.04 / native x86_64 Linux 和实际 execution crate运行。现在约81ms返回 `code=120, out="", err=""`，而原例应 `code=-1` 且保留 root 输出；线程虽回基线，测试仍 exit101。由于正常输入已损坏，不能把“没有残活线程”算作该反例的闭合证据。

**相邻正常正控**：[unix-output.py](unix-output.py)、[结果](unix-output-result.json)。同一新 driver 原样前缀，追加两个测试，不修改任一方法或原断言：

1. 普通受控子进程100ms后分别写 stdout/stderr 并正常退出。经实际 `run_bounded` 得 **code120、两流空**，期望 code0 与完整两条 marker，失败。
2. 同样正常子进程写一条完整 ready 行。经实际 `spawn_lines` / WatchGuard 收到 **Err(Disconnected)**，期望完整 ready 行，失败。守卫在断言前释放，探针拥有的子进程已收尾。
3. 同一 helper 直接在该 Linux 环境执行，实际 **exit0、stdout="R6-stdout-marker\n"、stderr="R6-stderr-marker\n"**。因此 helper、LF shebang与Linux运行环境正控有效；没有把装配 NotFound 当成产品失败。

复现入口（cwd 为唯一实施树）：`python D:/autostack/auto-lang/docs/plans/attachments/816/review-overall-r6/unix-output.py`；原例为同目录 `unix-reader.py`。脚本 JSON 中保存完整 native cargo argv、实际测试 exit101、源码前缀一致性与 stdout/stderr；Python脚本自身正常写证据退出不代表内部测试通过。

**闭合要求**：Unix 字节泵、行泵均须在读线程的整个实际读取期拥有有效管道；在此基础上重放原 SCM_RIGHTS 外部持写/OS线程快照、正常两流/ready 行、owned后代与实际超时出口。既要有界停止/收割，也要保留正常输出与退出语义，不能单纯忽略 `r` 的告警或接受子进程120。保留已正确的 Windows 路径。本轮不代 work 修改实现，也不新增产品 finding ID。

本次“新告警清零”的工作声明亦未成立：Windows 有新增 `BufRead` 未用；Unix另有两处 `r` 未用和 `Drain.fd` dead_code（共四条 driver 告警）。与同一04修复一起清偿；execution 依赖三条历史 Linux 告警、CLI历史 doc-comment/future-incompat 单列，不冒充本次新增。

## 07 与此前闭项

07 保持 CLOSED，不因本次测试 driver 回归重开安装缺陷。[R5 实际安装收据](../review-overall-r5/install-input-reviewed.json)及其 hash 被本轮 inputs 绑定；CLI/tool/SDK生产源、锁、配置与 auto-down 未变，R5 默认与显式 SDK producer 同源、双delivery资源verify、5539源输入、Core实际重建/脚本/缺UI具名拒绝与424包0禁项证据具名复用。R4 SDK服务生成与真实check/locked仍仅覆盖其已验范围。没有声称本轮重跑冷安装或完整HTTP链，也没有把旧工作 handoff hash 当本轮新发布产物。

此前关闭的生产服务 reader错误聚合本轮三态重放：[reader-error-results.json](reader-error-results.json)，READY前坏UTF8、READY后坏UTF8+exit0、+exit7 全 exit1且读错具名，最后一态保留7。JSON脚本return42和new均 exit0/整段单JSON，[结果](json-results.json.txt)；info因无pac.atom仍是旧无效成功样本，排除，不计本轮回归。

原生 Linux 生产service Ctrl+C转发/优雅退出与15秒强制兜底、真实Rust服务HTTP/config/port、seam与re-export单源、早晚取消及writer/reader残留错误传播，按不变源码/依赖/配置具名复用此前 R3/R5 记录。复用不覆盖新测试 drain；也不把 fake READY 寿命探针称为HTTP正控。consumer30与tool_process22+9ignored的R5证据同样按不变输入复用，未冒充本轮重跑或计入本轮测试总数。

## 健康、规范与路由

当前 locked 三包 bins 构建过；driver四项真实Windows正控、两个OS外部持写、owned寿命与内容/截止负控如上。777 strict **pass/0 violations/3 grandfathered/141边**，client依赖围栏、触碰文件rustfmt、差分check通过。首次777参数误用 `--strict` 的usage错误单列排除，改为原工具支持的 `--mode strict` 后才记录通过。新增测试告警仍未清。

SD候选与7项 canonical Spec 输入只读冻结，没有发布canonical/改live ledger。T08终稿仍需引用当期安装收据、消除旧handoff的延期R07片段，保持原合同；此前已闭07不因此重开。AC02/04受04影响、AC05未通过；AC01/03安装面及历史A/AC07..09保持此前已验范围；AC06为merge owner。没有因局部复审抹掉剩余任务或缩减原AC。

Plan保持 **executing 6/14**，T05..T09/当前T13不勾。下一步同树work完成04 Unix两种泵的所有权、正常与超时反例及新告警，提交后受影响复审，并完成既有T08最终交接/SD收口。最终pass后才执行merge-owned冻结裸cargo t与实际触面档/consumer-workspace门、Spec-ledger、合入归档guard；当前全部pending。未整档、未实施修复、未提交/合入/归档/清树。
