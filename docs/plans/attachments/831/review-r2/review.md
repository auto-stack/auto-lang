# PLAN-831 T-10 R2：needs_fix（修复复验与就绪核查）

2026-10-24，Plan r2。review_scope=final送审目标下的repair re-review；outcome=needs_fix。未发最终pass，也未重跑整套最终验收。交接自身承认AC-03的jet/ark仍为必需环境待验，按原合同不能以execution_done绕过；在此基础上按用户请求定向复验R1修复。检查者为当前Codex会话，未创建新独立上下文/agent；此限制延续，不冒称独立终审通过。

## 冻结输入

| 项 | 值 |
|---|---|
| 主检出簿记 | 83adc3419c41662fc3f28384b7784c2cde05c52c |
| lang树 / 分支 | D:/autostack/.wt/lang-831/auto-lang / codex/plan-831-dev |
| lang reviewed / repair base | fe8f3090a35c5720bcac873edba38d9f588a5663 / 1635835d377bd3faf12bb9ab642e14cebc150f94 |
| UI树 / 分支 | D:/autostack/.wt/lang-831/auto-ui / codex/plan-831-dev |
| UI reviewed / repair base | 425088520e402b1efae9479b9d0b5b3d44a56df4 / 464569d58d9d9c580616a811921b35bbac4265b5 |
| down只读依赖 | c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 |

两实现树tracked clean；lang唯一untracked为compile-probe/Cargo.lock，沿用允许的可再生产物处置。未写产品实现、依赖或配置；未跑整档Cargo门、未推进默认分支、未发布canonical/ledger、未清树。前次review-r1附件在主检出仍untracked，本轮保留，不隐含它们已入83adc3419。

## 已确认的闭合与证据复用

| R1 finding | 本轮复验 / 结论 |
|---|---|
| F831-09-01 blueprint同类oracle | **CLOSED**。期望不再回读被测函数；精确owner路径、真实spec.md枚举与错owner比较已恢复。UI树 `cargo test -p auto-ui-compiler --lib ui_gen::bp::registry::root_resolution_tests::`：5 selected/5 passed/0 ignored；覆盖env覆盖、cwd查找、fallback、实际扫描和锁恢复。原app_registry已绿的证据沿用，其输入/实现本轮未变。 |
| F831-09-05 同HEAD脏源admission | **CLOSED（本次R1范围）**。原 `replay-oracles.py --case admission-dirty`：合法clean=True，tracked source.rs 1→999后dirty=False，false_positive=false/exit0；新增tracked检查在共同准入路径。允许untracked锁的正控及S17既有收据保留。 |
| F831-R1-01 最终来源/SD | **CLOSED（材料刷新范围）**。freeze所列7份材料hash全匹配；4份原始收据hash与source-audit逐份匹配，sources绑定本轮完整SHA；全部canonical spec_inputs与当前主检出一致。独立ls-tree重算841基线文件：769相同、72变化、0丢失、1新增，与新台账一致。SD-01..07已改终态与840并集注记，旧865/867叙述纠正且tu唯一863保留。 |
| F831-09-03 两个旧假绿样例 | **旧例CLOSED，整体仍PARTIAL/OPEN**。原 `--case install-idle` 立即completed无安装被拒；原 `--case vm-idle` 无帧工具被拒；两者false_positive=false/exit0。真实链终源V06d/V06e收据存在且hash匹配，其各自观察范围保留。更完整不变量尚未闭合，见下节。 |

上述三个旧控制通过同一durable脚本重放：

```powershell
# cwd = D:/autostack/.wt/lang-831/auto-lang
python D:/autostack/auto-lang/docs/plans/attachments/831/review-r1/replay-oracles.py --lang-repo D:/autostack/.wt/lang-831/auto-lang --case install-idle
python D:/autostack/auto-lang/docs/plans/attachments/831/review-r1/replay-oracles.py --lang-repo D:/autostack/.wt/lang-831/auto-lang --case admission-dirty
python D:/autostack/auto-lang/docs/plans/attachments/831/review-r1/replay-oracles.py --lang-repo D:/autostack/.wt/lang-831/auto-lang --case vm-idle
```

未变证据复用：兼容compile-probe24/24、C2语义、777真实跨仓扫描、t/tu/ti/tn入口及既有选择面、LICENSE/告警归因；本轮未改这些产品路径，不重跑无关族。两仓修复diff的`git diff --check`均通过，前轮EOF空行已清。

## F831-09-03 · P1 · PARTIAL / OPEN

### A. 支持矩阵必需面仍未完成（已知就绪缺口）

原规则：Plan §5.3“环境缺口不可当绿，也不自动批准停用后端”，AC-03/V-03要求现行适用profile的真实构建运行。R1已明确不能把具名红改pass。

本轮新freeze同时记录open_findings=[]和open_environment_boundaries；handoff将该行标题仍写“pass（具名环境边界）”，正文/SD则承认named-partial。原始792 receipt确为38/39，V06c=false：jet gradlew exit9009，ark缺Hvigor wrapper、no build executed。vue/jet/ark交互运行、tauri/vscode宿主运行仍由dispositions记账而非成功证据。记录环境缺口是正确事实处置，不能使work-owned AC完成，也不能由T-10自行批准缩减支持。

处置：保持AC-03 partial和原门，不发final pass。应在适用环境取得约定成功证据，或由用户正式修订对应验收合同；没有此授权时不能把必需面转成非阻塞边界或merge-owned未来门。此为已知未完成项，不再花一次全量final-review确认它仍缺。

### B. 任意启动前进度仍被当作实际VM运行

原规则：AC-02/03、V-03/06及R1 F03闭合条件要求证明run已接受且实际运行、观察寿命收口。新的生成consumer driver `1274..1285`收集任意event并以`accepted = bool(run_events)`判接受；`1326`只再检查cancel回执、report.outcome=cancelled、describe_ok。

**已重放的区分性负控**：将同一V06e块的工具载体替为一个真实framed Python子进程。它只发送合法`ToolEvent::Progress {message:"resolve backend", done:1, total:3}`，然后服务cancel并返回完整RunReport `{profile:"vm", outcome:"cancelled", duration_ms:1, launch:null, error:null}`，describe返回当前真实发行工具的descriptor。没有Launched事件、不启动VM、不创建子应用/窗口。结果仍`V06e-vm-run-ui-real ok=True`，detail明确只见`['resolve backend']`，false_positive=true。

输入形态核验：Progress与RunReport字段来自当前auto-tool-protocol/message.rs/report.rs；describe body取当前交付二进制真实响应。生产run_vm.rs:292在运行前发`resolve backend`，:558在调用解释器前发`vm interpreter`；这些进度不能等价为首帧/运行中。该控制测试的是验证器对一种未启动即取消路径的辨别能力，不断言真实VM产品已坏，也不引入新后端/新规模承诺。

原idle反例修复只覆盖“完全无事件”；新控制覆盖同一运行接受不变量的“有准备事件、无真实启动”状态。当前run_ok/cancelled可以证明协议操作被取消，不能单独证明窗口已运行或终端/媒体完成关闭；不得在detail中无独立依据写“owned树回收后发出”当作外部观察结果。

**最小闭合范围**：保留无帧/立即终态旧反例与真实产品正控；增加启动前progress+取消的合法未运行控制，不能标为实际运行成功。实际运行证明须绑定真实启动/首帧/可用子应用等区分性观察，cancel/正常退出观察与其同一会话/对象关联；如复用协议Launched或既有MCP设施，应明确其真实含义与身份，不能只匹配任意event。只重跑受影响V06e及共享寿命/取消代表，未变发行/GC/全档不重跑。

R1/R2连续在F03同一不变量上未闭；下一work交接需先给出**旧例闭合为何未覆盖整个状态空间**的诊断、观察对象/实际路径及oracle调整，再提交下一候选，避免只再屏蔽一个负控输入。不要求新建泛化测试框架或重跑整个792长链来代替这个诊断。

## 复现与可保留证据

新控制的运行脚本：[replay-vm-progress.py](replay-vm-progress.py)，载体：[progress-only-server.py](progress-only-server.py)，结果：[controls.json](controls.json)。

```powershell
python D:/autostack/auto-lang/docs/plans/attachments/831/review-r2/replay-vm-progress.py --lang-repo D:/autostack/.wt/lang-831/auto-lang
```

本基线输出false_positive=true并退出2；其产品文件只读，在组内scratch生成自有输入并清理自己的进程。合法真实正控收据V06e为1s，事件为resolve backend/vm interpreter/None，cancelled与describe均绿；此历史事实保留，但不补足上述观察盲区。新的两种控制身份/断言范围在receipt.json中绑定，禁止把负控当成产品功能失败。

## AC / SD 与路由

AC-01来源/类型/DAG维持既有有效范围；AC-02仍有真实宿主寿命未齐；AC-03 partial（环境+运行证明）；AC-04现有资源/失败/安装活动及恢复证据保留其范围，不声称新鲜目录取消已覆盖所有旧安装升级事务；AC-05历史保全与drill保留、真实落地待T-11；AC-06材料/CI改进成立，整体验收仍随F03未闭；AC-07未发pass；AC-08 merge pending。

SD-01..07目标/owner/840并集及hash刷新检查通过，无新退休/支持缩减操作；SD-02/04/05涉及的完整发行/运行事实须受F03边界约束。保持GOAL-003/007与原spec-impact metadata。当前“环境partial但全部finding CLOSED/work complete”的结论不能成为规范事实。

最终：F01/F05/F831-R1-01本轮范围关闭，F03保持一个稳定ID下的未闭范围。T-09重开，Plan回executing 8/12，T-10不勾；无关T-01..08勾选及所有历史记录保留。work在同组原树继续，merge-owned裸cargo t/触面档与T-11/T-12仍pending，tf仍仅主检出到期批量。U831未成立，832不放行。
