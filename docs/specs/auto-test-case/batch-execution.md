# ACE测试批量试点

> PLAN-888 r3；test-only，默认关闭feature；正式数据见计划内测量收据。

## 路径与推广边界

A为原逐项worker/编译/VM基线。B在显式 `test-case-batch` feature 下提供
`avworker-test-batch/1`，每组最多十项，逐项新建原driver/VM/store；只共享经证明不可变的
工具身份准备。父侧持expected，复用原准入与比较器；worker不收expected，不另造解释器。
旧case-serve/session-serve不改语义，缺batch能力明确失败，不回退A伪装B。

A/B/C功能与隔离控制已实施。历史候选完整矩阵的B k=3/k=10通过原门、k=1未通过，仅为该候选历史结果；889共享输入及后续计量producer变化后，不充当前候选收益证据。r3优先模块归组/数量及逐模块单轮资源计量，新的完整收益矩阵后置，不阻塞模块交付。公开test-case-pilot与单case均默认A；B仅显式--mode group，C仅显式--mode combined。其他recipe及原负诊断/FFI/资源/会话/单程序性能门保持。未来申请推广仍须以下完整合同，不以未完成矩阵假称技术负结果或加速。

C仅限原D3三条纯表达式，以现有av.owned/1路径一次compile/VM返回record三个actual，父侧
逐项原scalar判定；原表达式字节和包装byte/line映射分别记录。实际compile/VM调用点计数，
sequence只作执行器观测，不猜调用次数。C仅显式 `--mode combined --set d3`，不替代A/B、
原单文件编译/隔离或M1四路径认证。

## 准入和生命周期

白名单、schema/nonce/request/source/profile/worker身份、序号、逐项ACK、done/EOF/退出
必须闭合。可执行文件持只读共享lease，拒写/改名/删除；进程suspended创建、纳入Job再运行。
组截止从准备起直到比较和清理结束，最多60秒；逐帧4MiB、累计40MiB，原guest/输出限制不
放宽。读写有界，写线程、输出排空和清理均纳入生命周期；超限、截帧、额外结果、错序、
异常退出或不读stdin均有界杀树。普通判决错逐项保留，fatal后未派发项NOT_RUN，整组非零。

报告保留原case/check、源和契约路径/行、source/oracle hash、expected/actual或typed错误、
原父侧receipt及实际独立replay argv。单例筛选真只执行该例，不以重放覆盖首次失败。

## 未来收益推广所需正式测量（非本轮模块交付前置）

`measure_pilot.py` 消费冻结 `measurement-contract.json`：九cohort×两并行度×十正式轮，
每次原集合repeat3，另18完整预热。实际成功spawn点PID与原调用compile/VM计数共同验证
工作量；A/B15均45原判定/compile/VM，启动45/45/15/6；D3 A/B均9原判定/compile/VM、
启动9/9/3/3；C9原判定、3compile/VM、3spawn。重复invocation不充新增覆盖。

原生Job计父进程和全部后代CPU、观察器CPU、总CPU、整命令墙钟、进程/Job峰值；父侧准备、
传输、准入比较收据、清理分相互不重叠，worker阶段另列。源/配置/锁/输入/工具及生产者
前后绑定；A/B同一worker二进制。外部污染保留原尝试，整轮排除，按相同预定顺序最多重测
三次；机械/oracle失败立即停止，不能用环境理由掩盖。缺项/错误不充有效分母。

P50/P90采用nearest-rank ceil(pN)，噪声=max(1ms,2×A median absolute deviation)。推广须
墙钟或总CPU P50至少改善10%，另一P50和墙钟P90退化≤5%，差额超过噪声，内存及原guest
预算不越界，并有全部守恒/隔离/故障/重放控制。C只与匹配D3单列比较。数值门不自动推广；
决定和边界见[测量摘要](../../plans/attachments/888/pilot-measurement-decision-r10.md)。
