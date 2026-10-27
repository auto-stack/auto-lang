# PLAN-816 整体 B-R7 受影响复审

**结论：needs_fix。R6 的 Unix 管道保活缺陷及新测试告警已闭；04 的“运行至超时返回前 reader 真停”仍有一项 P2 残项，沿用 F-816-B-R1-04。07 与 Windows 已闭范围保持。**

受审 `8c69caef752caf0f583aab5f5e0e4670eed7cf00`，差分基线 `0e50bedd0e3775caf3be5ca172af2e16e125b718`，Plan r2。唯一实施树 `D:/autostack/.wt/lang-816/auto-lang` / `codex/plan-816-dev`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。review_scope=final受影响核查，非最终pass。当前聊天 /root 独立核验实施交接；未创建agent/第二checkout，模型/会话ID未暴露。未修改实施源码。

[输入/等价性](inputs.json)、[冻结Plan](plan-frozen.md)、[SD候选](spec-delta-reviewed.md)、[最终身份/产物/清洁状态](final-state.json)。差分只有 driver 19行；48项受监控输入中47项与R6相同。新driver全文SHA256 `8561504edaef617993d0e6adbe02f1e856a2f3cf988315bdc5c9483cc58198d9`。所有新driver探针保留其全文前缀，只追加测试；原测试及方法未被替换。

## 04 已闭范围

Unix两种泵的闭包现在持有 `r`，读取期间管道对象不会提前drop。R6两条原正控、旧外部持写反例均重放通过：

- [正常输出/行泵](unix-output-result.json)：实际run_bounded得到 code0、完整stdout/stderr两条marker；实际spawn_lines/WatchGuard收到完整ready行。相同helper直接执行亦code0/两流完整。
- [原SCM_RIGHTS反例](unix-reader-result.json)：根正常先退出、真实stdout写端交给独立receiver。当前约 **220.1ms** 返回code=-1，保留root输出、drains=[true,true]；/proc任务返回基线，无残活reader；没有靠释放外部writer补绿。
- [新增Unix行泵相邻控制](unix-watch-external-result.json)：同一实际停止模块、实际WatchGuard和真实SCM_RIGHTS外部写端，先收到root完整行再release；约 **50.6ms** 收尾，任务回基线，转交marker成立、receiver在快照时仍活。证明行泵并非靠提前关读端结束。
- [Windows watch](watch-external-result.json)：真实DuplicateHandle外部持写，release约 **5.24ms**，Thread32基线与返回后相同。
- [Windows run](external-reader-result.json)：约 **228.3ms**、code=-1、root输出保留、drains true、线程回基线。
- [真实CLI正控](positive-driver-results.json.txt)：当前原样driver的裸脚本argv、serve/req、watch内容再生、真实tool gen **4/4**。
- [新测试内容负控](watch-oracle-proof.json)：新driver与私有副本SHA相同，实际input_changed=true、incremental_write=false、旧SFC字节仍在，正确exit101。没有重跑旧测试代替新oracle。

新driver在Windows/native Linux均无新增告警。Linux依赖设施仍有三条基线告警；CLI既有doc-comment及proc-macro未来兼容告警仍单列，未声称全仓零告警。

## 04 / P2 残项：根仍运行直至deadline，reader在caller返回后才停止

**既有规则**：AC-02/04、V-02/04、T-07要求超时后自有进程/reader实际退出，不能只证明parent返回。R5闭合要求已覆盖提前超时/等待失败出口，R6亦要求实际超时出口。这里检查测试driver自身收尾，不宣称生产Unix服务回归，不新增平台或性能承诺。

**原因**：`plan816_cli_driver.rs:484–487` 的 `left(deadline)` 在到期后取1ms；`:491–508` 等待根到期后，才调用cancel与两个reader的 `stop_bounded(left(deadline))`。Unix reader在`:305` poll中最长等待50ms。实际剩余收尾时间不足时，`:96` 把JoinHandle移入RESIDUE并返回false，run_bounded随后带“drain threads still alive”返回。登记归属有效，但不等于返回前线程已停。这一分支不受旧“根先退、尚余约100ms清理预算”正控覆盖。

**输入与真实入口**：[unix-timeout.py](unix-timeout.py)、[最终结果](unix-timeout-result.json)、[首次相同机制结果](unix-timeout-first-result.json)。使用当前整个driver不改，仍由实际auto-tool-execution设施承载；同一个原SCM_RIGHTS helper仅增加root转交/输出后继续运行3秒的模式，run_bounded限280ms。外部receiver有5秒自截止、marker确认实际收到写fd；不属于被取消树。只在全部快照完成后显式清理该探针拥有的receiver。

实际结果：

```text
elapsed=288.430942ms code=-1 out="root output before exit\n"
err="timeout after 280ms; drain threads still alive (RESIDUE registry)"
transfer_confirmed=true receiver_alive=true receiver_alive_later=true
baseline={718,719}
after_return={718,719,722}
at_one_ms={718,719,722}
reader_residue=[722]
after_100ms={718,719}
test exit101: reader still alive after timeout returned
```

首次独立执行同样约288.6ms返回、任务725在返回与1ms后仍活，100ms后回基线。之后仅把超时诊断断言从具体“timeout after”放宽为分类“timeout”，避免固定未来修复的文本；真正的线程退出断言不变，重放仍失败。两个原始结果均保留。

因此这是**短暂但实际的返回后reader残活**，100ms内已自行结束；不是无期限阻塞。输出保留、超时具名与RESIDUE归属本次都正确，但仍未满足“返回前真停”的原闭合规则。不能把老例转绿概括为整个04闭合。

复现命令（cwd为唯一实施树）：

```text
python D:/autostack/auto-lang/docs/plans/attachments/816/review-overall-r7/unix-timeout.py
```

JSON含完整native Linux cargo argv、源码前缀一致性及helper hash。Python记录器正常退出只表示保存证据，内部Rust测试exit101才是判定。没有修改生产方法或注入暂停屏障，OS线程ID属于测试进程；外部receiver为独立进程，不能混入该线程快照。

**闭合oracle**：保持已过的正常两流/ready行、根先退/外部持写、Windows和watch控制；同一实际run_bounded在根运行至timeout时，也须有界完成其自有reader停止与收割再返回。修复应协调真实取消/清理所需时间与总预算，不能只调整错误文本、把线程登记进全局表，或等待测试释放外部writer才宣称通过。需补此真实超时出口的永久定向回归；对共享的wait失败及根退出后预算不足收尾分支同步核对，不能再只验证根先退样例。当前未对OS wait失败注入作实测声明。

## 07、此前闭项与验收范围

07保持CLOSED。[R5实际安装收据](../review-overall-r5/install-input-reviewed.json)由本轮inputs记录hash。R5以来产品CLI/tool/runtime/SDK普通构建所用源、Cargo锁、配置及兄弟依赖不变，只有测试driver变化；复用R5默认/显式SDK producer同源、双资源verify、实际full/Core产物与Core闭包证据，以及R4SDK服务生成/真实check/locked的既有覆盖。旧安装SDK快照中的测试文件属于旧快照，未声称它与当前完整源码逐字一致，亦未把旧安装hash冒充新发行身份。

R6重放已过的生产service三读错状态、JSON脚本/new，R3生产Unix Ctrl+C真实转发/优雅与强制兜底，已有真实Rust HTTP/config/port、seam/re-export单源、协议选项拒绝/泵取消与writer完成竞态，均按未变产品源/依赖/配置具名复用。R5 consumer30、tool_process22+9ignored同理，未计作本轮重跑。R6 owned后代/20s噪声总截止证据在其机制不变范围保持；新Unix timeout分支未被这类复用覆盖。

T08最终交接/SD收口仍为work-owned：旧handoff仍有旧产物hash及“发行闭包延期R07/待裁定”片段，应引用当期已审核安装收据并对齐已闭事实；本次不重开07产品缺陷，也不把该已知收口任务当作新产品finding。SD01..05及protocol/tools候选只冻结；7项canonical Spec来源核对，无canonical/ledger写入。

## 健康与路由

当前locked CLI/tool/execution bins构建、触碰文件rustfmt、差分check、client依赖围栏通过；777 strict **pass / 0 violations / 3 grandfathered / 141边**。本轮不跑整档；上述私有原样driver与失败探针按实际结果分别记录，未用全绿测试总数替代关闭判据。

AC02/04仍受04影响，AC05未过；AC01/03及历史A/AC07..09按既验范围保持；AC06由merge负责。Plan **executing 6/14**，T05..T09/当前T13不勾。下一步同树work收口真实超时出口与永久回归，提交后受影响复审，并完成既有T08最终交接/SD收口。最终pass后再执行merge-owned冻结裸cargo t及实际触面档/consumer-workspace门、Spec-ledger、合入归档guard；均pending。未实施修复、未整档、未提交/合入/归档/清树。
