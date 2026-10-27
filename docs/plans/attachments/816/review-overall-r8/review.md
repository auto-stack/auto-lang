# PLAN-816 整体 R8 受影响复审

**结论：needs_fix。原 F-816-B-R1-04 最后残项 CLOSED；新增永久 fixture 的失败清理存在一项 P2：F-816-B-R8-01。07 与此前已闭范围保持。**

受审 `abc911befad74bd85234946517f077b57caa1683`，基线 `8c69caef752caf0f583aab5f5e0e4670eed7cf00`，Plan r2。唯一实施树 `D:/autostack/.wt/lang-816/auto-lang` / `codex/plan-816-dev`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。当前聊天/root独立核验实施交接，未建agent/第二checkout，模型/会话ID未暴露。review_scope=final受影响核查，非最终pass。差分只有测试driver与具名te-child；未修改实施源码。

[冻结输入](inputs.json)、[Plan](plan-frozen.md)、[SD候选](spec-delta-reviewed.md)、[最终状态](final-state.json)。本轮新driver SHA256 `cb99e0afd32690983bcbe819f323b12756c39e05916e7be8e776c2778e6e5d08`；te-child源码 SHA256 `27c2d39a2e258f9b97aa117d1b0853746222ae7c3a72f20a8178cdcf7892eb6f`。前后HEAD/clean/依赖与输入指纹核对。

## 原04闭合与正常控制

run_bounded四种出口现使用独立收尾宽限，实际停止不再从耗尽的deadline取1ms。原R7输入、整个新driver前缀和断言原样重放：

- [原生Linux真实超时](unix-timeout-result.json)：约 **323.4ms** 返回code=-1、输出保留、`err="timeout after 280ms"`，返回/1ms/100ms任务快照均回基线。真实SCM_RIGHTS转交marker成立，receiver全程仍活。原残活任务反例已修好。
- [根先退出](unix-reader-result.json)：约241.4ms、code=-1、drains=[true,true]、输出保留、任务回基线。
- [正常两流/ready行](unix-output-result.json)：两正控通过、helper独立执行对照通过；[Unix watch外部持写](unix-watch-external-result.json)约50.9ms真停。
- Windows [run](external-reader-result.json)约226.9ms、[watch](watch-external-result.json)约5.59ms，真实DuplicateHandle外部写端在场，reader均回基线。
- [真实CLI原四项](positive-driver-results.json.txt)4/4；[Windows新增两项](windows-permanent-tests.log.txt)2/2；新测试[旧SFC内容负控](watch-oracle-proof.json)保持实际输入改变/旧字节保留必红（exit101）。

新增永久两测在原生Linux亦通过；本轮独立以实际当前te-child重建，分别在独立测试进程中把唯一TEARDOWN常量300ms改成1ms：timeout测红于`own readers must be stopped before return`，holders测红于`drains=[false,true]`，都有实际转交，不是缺件红。[原样与突变/完整argv及输出](native-fixtures-result.json)、[装配及源码身份](native-fixtures-run.json)、[可重放脚本](native-fixtures.py)。当前测试源码具备对应区分性。

工作 `repair-r8/mutation-result.json` 的“双红”实际均为spawn fd receiver/NotFound，不能证明该区分性；本轮未采用该收据，而是按上面实际fixture重新验证。首次审方直接rustc装配遗漏te-child对本crate的引用，错误[单列排除](native-fixtures-setup-error-excluded.json)；改为私有Cargo包直接引用当前te-child原文件与实际execution依赖后才形成验收证据，未把装配错误算实现缺陷。

## F-816-B-R8-01 / P2：新receiver失败出口没有有界自退出与拥有者清理

**受影响**：AC-02/04、V-02、T-07。Plan测试合同要求具名子进程自带截止、测试清理其拥有子进程；不能在断言失败时留下无界fixture。此次是新增测试设施的失败生命周期回归，不重开已闭的run_bounded真停，也不据此宣称生产CLI服务故障。

**源码根因**：

1. `te-child.rs:143–144` 只有 `receive_fd()`完成后才sleep有限秒数。`:405` 的accept和`:426`的recvmsg均为阻塞调用，没有deadline、poll/timeout或其他看门机制。无人连接/连接后无payload时，“持有8秒”尚未开始，不构成整个receiver的自截止。
2. driver的两个新测试在多个assert之后才于`:732–733`、`:792–793` kill/wait；`spawn_holder_receiver`也在spawn后直接assert。没有RAII清理守卫。root在转交前失败或任一断言panic时，测试所拥有的receiver不会被收尾。

**实际当前源码失败控制**（详见native-fixtures-result.json）：

- 原样新driver及当前真实receiver。唯一故障宿主wrapper仅让`stdout-holdout`根在转交前exit7，receiver模式仍exec当前真实te-child；这是子进程失败/无转交出口，不改driver方法或assert。
- 测试正确exit101、红于“receiver must confirm actual write-fd transfer”，但测试结束后receiver **PID1201**仍活，**1.5秒后仍活**。它阻塞在accept，直到审方外层明确清理才退出；此处程序没有未来的握手超时。
- 对当前真实te-child分别做无人连接和连接后无payload控制，ARG3=1。就绪后1.5秒仍活，marker均不存在；内核等待点分别为`__skb_wait_for_more_packets`及`unix_stream_read_generic`。有限观测证明到达真实阻塞状态，源码无超时证明其不会靠声明的自截止离开；不是把1.5秒当新增性能要求。
- 上述合法1ms突变两案失败后，也分别遗留receiver1167/1184，说明永久负控自身的失败清理遗漏。与原样正控结束后receiver列表为空形成对照。

所有探针使用独立临时目录和唯一继承token识别自己拥有的receiver，核对实际te-child程序后才由外层kill；没有清理任何共享服务或无关进程。直接阻塞控制也总在finally kill/wait。wrapper、PID清理逻辑、原始输出与helper binary/source hash均在脚本/JSON中，输入可复放。

复现：cwd唯一实施树，执行 `python D:/autostack/auto-lang/docs/plans/attachments/816/review-overall-r8/native-fixtures.py`。该记录器保存预期红案后正常退出；内部测试code与残留列表是判定依据。

**闭合要求**：receiver从准备/握手起即有真实有界退出；两个测试及spawn后断言等失败出口均应回收其拥有receiver和临时socket/marker。保持原SCM_RIGHTS/DuplicateHandle持写语义，正控期间不可提前释放writer。关闭oracle应同时覆盖原样两绿、实际1ms突变两红、根转交前失败、无连接和静默连接，并在失败测试返回后核实无自有receiver残留；不能只在审方记录器里额外kill补绿。

## 既闭范围、规范与路由

07保持CLOSED，R5实际安装收据由inputs绑定hash。产品CLI/tool/runtime/SDK普通构建源、锁及配置未变，复用既验的full/Core/source安装、默认/显式SDK producer、资源verify、Core闭包及R4 SDK实际生成/check/locked范围。旧SDK快照含旧测试文件，未声称完整快照与当前源码逐字一致或作新发行身份。

生产service读错聚合/JSON、真实Unix关闭信号及强制兜底、HTTP/config/端口、seam/re-export单源、协议选项拒绝/泵取消与writer竞态按未变产品输入具名复用R3/R5/R6/R7证据。此次新fixture不在旧证据覆盖内。未把consumer30/tool_process22+9ignored的旧结果计为本轮运行。

当前locked三包bins、两触碰文件rustfmt、差分check、777 strict pass/0违规/3grandfathered/141边通过，新driver/te-child无新增告警，设施Linux三条与CLI历史告警单列。收尾常量是各操作分别使用的300ms，代码尚非共享300ms总截止；本轮不把注释的limit+300ms当作已证明严格上界，T08应据实际实现准确表述预算，现有1500ms断言只证明较宽有限界。

T08最终交接/当期收据/SD收口仍明确work-owned，旧handoff的旧hash/延期R07片段待更新，7项Spec来源只读冻结，不改canonical/ledger。原04与07运行实现缺陷已闭，但AC02/04受新fixture失败清理影响、AC05未过；T08未收口亦不能授整Plan最终pass。AC01/03与历史A/AC07..09保持已验范围，AC06为merge owner。

Plan保持executing6/14，T05..T09/当前T13不勾。下一步同树work修新fixture失败清理/真实截止、补对应正反控并提交；随后完成T08终稿与受影响最终复审。最终pass后merge-owned冻结裸cargo t/实际触面档/consumer-workspace门、Spec-ledger、合入归档guard仍pending。未实施修复、未整档、未提交/合入/归档/清树。
