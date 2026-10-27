# 给 GLM 的下一轮指示：PLAN-828 r3 / T-01 修正与最小首例

按当前已授权范围继续 work。先读主检出的 [Plan828](D:/autostack/auto-lang/docs/plans/828-av-function-incremental-session.md) §§5.6–8、最新§9收据与 [合同就绪报告](D:/autostack/auto-lang/docs/plans/attachments/828/contract-r3/readiness-20261021/report.md)。实施树固定 D:/autostack/.wt/lang-828/auto-lang，分支 plan-828-dev，交接基线 3e60af2c8aa2c287633f4700b377714e94c4ff4f。进入先核 path/branch/HEAD、dirty状态及当前主计划；若出现新提交/未提交实现，先与本基线对账，不覆盖。

**本轮范围：T-01/D-01..03 的修正与真实最小可行性证明。** F-01/F-02 均 OPEN，其他开放项保持；T-01尚未验收，不直接展开T-02或T-05/06的全面替换，不合入。本指示沿用既有授权，不要求重新申请相同范围批准。

**同提交正式复审补充（needs_fix）**：先读[完整复审包](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/review.md)。T01-C01/P2对应R-01，T01-C02/P2对应R-02，均已有真实失败与合法控制；新增T01-C03/P2为CheckedEnv.sigs/FnSig可见性不一致的private_interfaces warning，须同一轮修正并用编译输出闭合，不能归入既有F-14老warning。原contract5/5不抵消独立控制2pass/2fail；关闭原样9/9保持。下文D-02/D-03前置证据仍需补齐。

若D-01确实受未定接口阻塞，记录具体前置并继续无依赖的D-02调查；不让待定设计扩散成全面换接，也不停止已有授权的独立工作。

## 按顺序完成

1. **先关闭报告 R-01/R-02（D-01）**。stub_resolved 目前先深clone整个ResolvedFn再丢body；plan_module_scoped对全部函数plan，render/bind仍生产全部stub bodies。去掉这条隐藏全量工作，不得把它改名接口目录工作。可读取/更新必要签名和绑定元数据；复用既有语义规则、producer/binder并抽取真正单body上下文入口。只编changed body，未改body不clone/计划/发射/绑定重建；结果仍经适用的既有verifier构造私有凭证。保留完整source/ABC入口与原支持面。具体API由本轮有界选择；若DefRef消费仍阻塞，给源码事实/候选方案/失败首例，不用另一层桩包装掩盖。
2. **补真正的同门正负控**。真实producer/consumer观察changed和unchanged函数，元数据单列；固定real_body_plans=1和调用者设N/N不作为已执行工作证据。合法单body路径值正确且门绿；在同一候选/会话测试配置中强制真实旧compile_program_parts全stub路径、保留实际观察器，值可对但同门因额外工作红；缺事件另测。保留冷编HIR逐字节对拍、类型错/签名漂移反控及caller只读接口控制，不再手动重查caller来替代“它未被处理”的断言。
3. **冻结 R-04 的候选环境责任（D-01/D-03）**。recheck成功会替换env条目；prepare成功也不能提前发布当前环境。说明选用候选/overlay或有界临时换入后恢复等实际机制，禁止全env深clone。T-01先做同基面双候选取消/合法继续，以及一次后续拒绝的最小事务首例：当前环境/根/source/revision/来源不提前改变，成功commit才一致发布。迟到/foreign/closed与完整失败出口列明实际消费接缝，随T-02/T-06按原C01/C02/C05矩阵完成，不冒称本首例已全部验证。新增环境body的close释放和预算责任写入拥有关系表，保留C05既有语义。
4. **再补 R-03（D-02）可执行最小首例**。写清单元局部type/const/slot/PC、跨函数/import及自递归绑定、snapshot归属和trap func/来源映射分别在哪消费。运行main→helper的新单元dispatch首例：两函数不同常量，改helper长度/槽数后值正确，未改main实际payload同一；普通/steps/resume共用实际取数接缝，真实暂停旧链保留旧snapshot；错签名/缺目标在安装前拒绝且旧状态不变。强制复制实际被AV消费的未改payload，值可对但同结构门红。允许最小实施证明接缝，不借此勾完整C02矩阵；不要把当前merged缺陷已量化当成新机制已跑通。
5. **维护一份当前合同与关闭表**。更新主检出的interface-decision.md，撤销“当前脚手架等于纯接口遍历”等超过代码证据的结论；历史原件由本报告目录保存。关闭表逐项覆盖T01-C01/C02/C03及R-03/04剩余决策：原输入/合法对照、实际机制、符号/入口、断言、预期与实际、原始输出、残余边界。按实际需要收窄字段/类型可见性清理T01-C03，不盲目扩大公开API。校正frontend4与“5/5”矛盾；维护§6已有真实测试映射，追加本轮实际入口/断言/raw。未完成的C01/C02矩阵、A07/依赖/F09/持久化/正式门不标通过。

## 验证与交接

先做上述实际修正，再运行相关反例和定向模块检查；不因单条finding、提交或再次送审运行整档t/tv/tt/tb/tu/taa/tf或docs_gen。可按本轮实际触面选：

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_contract contract_828_frontend -- --test-threads=1
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_contract contract_828_dispatch -- --test-threads=1

这两个现有filter不覆盖全部必需断言；按新增实际test名选择R-01..04所需控制，确认非零选择数、失败不吞和实际mutation。不只重放初始5例就宣称本轮闭合。相关cold差分、旧会话/source-map控制、r23a关闭/候选/PausedRun控制按改动重放；未改证据可具名说明代码/依赖/配置/输入未变后复用，不机械重跑所有旧harness。src/caps/格式/常量空间/实际消费改变的控制不可直接复用。

提交后交：冻结完整SHA、clean树、依赖/配置及输入身份、实际命令/选择数/退出码/raw路径、R-01..04关闭表、C01/C02已测/未测边界、下一责任人。维持executing 0/11，不将局部交付设整份execution_done；先做T-01受影响面复核。通过后再按原依赖顺序推进T-02/03与T-04→T-05/06。无需本轮就做正式IPC/P95/全内存门；这些仍是原任务后续必需项。

如果有界首例仍不能满足某条原不变量，如实保留OPEN并给实际阻碍/设计备选回new；本轮成功标准是机制与证据成立，不是增加绿测试总数。禁止直接降低验收或把必需项转债。

