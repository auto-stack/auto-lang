# PLAN-816 整体 R10 受影响复审

**本轮受影响代码验收通过：F-816-B-R9-01/P1 CLOSED，无新增实现finding。整体最终outcome仍为needs_fix，仅因既有T-08最终交接/当前身份与规范终稿尚未完成；下一步转T-08收口，不再要求重复本轮代码修复。**

受审 `6d15d25a4a317bb028b160fa74be95ee90d9c788`，基线 `04bb00a460d6c11f375ce9ccb1a95f900067c672`；主检出交接 `7dd6eaa3e9f80a95efb5c4c153a1a90c0f5e2abc`，Plan r2。唯一实施树 `D:/autostack/.wt/lang-816/auto-lang` / `codex/plan-816-dev`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。当前/root聊天独立核验实施交接，无agent/第二checkout，模型/会话ID未暴露。review_scope=整体R10受影响核查及最终readiness检查，不冒充整Plan最终pass。未修改实施源码。

[冻结输入/等价性](inputs.json)、[Plan](plan-frozen.md)、[SD候选](spec-delta-reviewed.md)、[历史交接快照](work-handoff.md)、[最终状态/产物](final-state.json)。前后HEAD/clean/依赖/配置及7项当前Spec来源核对。三个变更文件为unix_pg、te-child、exec_lifecycle；driver/协议/服务主体未改。

## R9-01/P1 CLOSED

signal_group与kill_group现在传负PGID，保留空组ESRCH收尾语义；既有进程组准入/pre_exec不变，不扩大到全系统或同名进程。以下均在原生Linux使用实际当前库、当前helper、当前生命周期测试全文前缀重建。

[原反例及相邻控制](native-lifecycle-result.json)、[可重放脚本](native-lifecycle.py)、[源码身份](native-lifecycle-run.json)：9个独立定向测试均exit0，测试后自有live helper均为空：

- 三条R9原失败测试全部转绿：cancel_tree_scope_kills_grandchild、root_exit_terminates_pipe_holding_descendants、early_spawned_descendant_belongs_to_owned_tree。覆盖根活着取消与根已经退出仍有owned后代两状态。
- 原心跳对照：root/verified PGID=1710，真实cancel后 **0→0**，不再推进。补充外部组清理返回-1/errno3（ESRCH），心跳仍0；组已由真实cancel清空。
- 上述探针仅机械更新补充清理的前提：允许0或ESRCH，真实 `before==after` 关闭断言不变。修复后再要求“外部补杀必须成功”会误报；R9冻结红输入/结果未被覆盖。
- 当前永久 `interrupt_signal_reaches_group_descendants` 正控exit0：双端handler已安装握手后，后代受控退出42，由仍活的根传播，证明送达后代而非默认杀根。
- DirectChildOnly正反控通过：其后代继续推进并按自身截止退出。独立新增无关进程保护控制：owned heartbeat **0→0**，独立helper heartbeat **0→300**且仍活；记录后才由探针清理该helper。
- R8-01无人连接/静默连接两生命周期测试均具名自退4，保持通过。

**新信号oracle区分性**：[signal-negative.py](signal-negative.py)、[结果](signal-negative-result.json)。只在私有诊断库副本中把signal_group的一处负PGID恢复成正PID；kill_group保留修复，当前真实helper及生命周期测试源路径不改。实际后代10秒截止退出 **7**，测试在 `Some(7) != Some(42)` 上正确exit101。不是缺件、编译失败或固定睡眠假红。私有副本只有unix_pg一处输入变化，不是第二checkout，实施树未修改。

## 实际消费者与既闭项

- [原生服务关闭消费](unix-signal-result.json)：当前serve_rust实际执行尾部全文提取、真实设施、受控READY子进程与真实SIGINT handler；向服务父前台组发SIGINT后，ProcessTree实际路径约 **264ms** 干净关闭、exit0、graceful marker在场，继承组控制亦通过。它验证生产关闭代码→当前设施的消费组合；不是冷构建整套服务、真实HTTP负载或在途请求排空的新证明。原已验HTTP/config/端口证据只在未变对应代码范围复用。
- [原生独立execution consumer](native-consumer-result.json) **5/5**，消费当前库及在同次调用重建的当前helper，覆盖真实argv/cwd/exit/deadline/tree/spawn/终止原语。首次尝试复用上一原生调用的/tmp helper路径失效，五项均NotFound；[原错误单列排除](native-consumer-missing-helper-excluded.json)，未视为实现失败或有效负控。随后同调用构建并绑定实际helper hash才记通过。
- Windows当前execution **11+23**、client **17**，见[execution](windows-execution-tests.log.txt)、[client](windows-client-tests.log.txt)；当前原样CLI driver四项+永久三项共 **7/7**，见[原四项](positive-driver-results.json.txt)、[永久三项](windows-permanent-tests.log.txt)。没有把0选择或ignored充数。
- [R8-01正反控](native-fixtures-result.json)：三个永久正控绿、两条有效1ms突变真红、根转交前失败红；六案测试后receiver均空。仍存在的独立临时目录内，守卫socket/marker剩余均0；无peer/静默peer自退。文件清理在同一原生调用内观测，不拿已经消失的/tmp目录充当清理证明。
- [实际Unix超时](unix-timeout-result.json)322.8ms、[根先退外部写端](unix-reader-result.json)219.7ms，输出保留、线程回基线；[正常两流/ready行](unix-output-result.json)与[Unix watch](unix-watch-external-result.json)50.4ms通过。Windows [run](external-reader-result.json)228ms、[watch](watch-external-result.json)5.20ms真停保持。
- [新driver旧SFC负控](watch-oracle-proof.json)：输入实际改变但旧字节保留，新测试正确101。旧协议拒绝、seam/re-export、JSON/读错聚合等不变路径按对应R5/R6/R9证据具名复用。

07安装/来源实现缺陷保持CLOSED。R5安装收据只作为其准确历史输入与已验结构证据，布局/发现/producer接线未改；本轮Unix执行代码确有变化，不能说SDK源码或发行hash与R5完全相同。新原语行为由上述原生设施、服务与独立consumer实测；当前完整安装源码/二进制身份刷新仍归T08终稿，不拿旧快照冒充当前发行包。

## 最终readiness：T-08仍未完成

Plan §8明确T09依赖T08；续12工作记录也明确“随后T08最终交接/SD收口仍work-owned”。当前交接快照仍有旧二进制SHA、旧“完整profile未交付/延期R07需裁定”片段，与已闭07不一致；SD仍为概要候选，未形成完整当前终稿。因此不将本轮代码闭合等同整Plan最终审查通过，也不新增代码finding或重开07来表示这项既有工作。

下一步T08需完成具体收口：

1. 对齐当前冻结代码、命令/验收矩阵与所有已闭项，移除作为当前结论使用的旧未交付/待裁定片段；保留历史收据与复用边界。
2. 更新并读回当前full/Core/SDK源输入和二进制身份，明确本轮Unix源变化与已有安装证据的适用范围，不沿用旧hash作新产物身份；无需为未变生成算法再做无关冷构建。
3. 完成SD01..05及protocol/tools七目标的持久规则终稿、当前Spec来源/增量及Plan完成记录。真实源码向既有Unix进程组规范收敛，不能修改规范来降低已验树语义。

本轮只冻结候选，未发布canonical或改live ledger。完成上述既有work-owned收口后，再以输入等价性复用本轮通过的代码证据，审查终稿/最终readiness，不要求重复制造代码修复轮。

## 健康与状态

locked三包bins、触碰文件rustfmt、差分check、client依赖围栏、777 strict pass/0v/3g/141边通过。Linux设施三条、consumer既有非测试lib告警及CLI历史告警单列，新变更无新增告警。未跑整档；merge-owned裸cargo t及实际触面档/consumer-workspace门、Spec-ledger/合入归档guard全部pending。

本轮AC02受影响执行/取消范围通过；AC01/03及AC04既验行为范围保持，但AC04当前身份/SD交接和AC05最终readiness仍待T08。AC06未来merge，历史A/AC07..09事实保留。T02恢复完成，Plan **executing6/14**；T05..T09/当前T13未据局部核验勾整项。最终outcome **needs_fix（仅既有T08收口；无未闭代码finding）**。未实施修复、未整档、未提交/合入/canonical/ledger/归档/清树。
