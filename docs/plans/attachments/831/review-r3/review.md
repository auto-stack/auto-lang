# PLAN-831 T-10 R3：needs_fix（Plan revision 3）

2026-10-24；review_scope=final目标下的受影响修复复审。接受用户本轮对r3合同修订已完成的直接说明，并以当前Plan §4/§5.3/AC-03/V-03的有界文本核对；**不再将jet/ark缺工具链本身判作本Plan阻塞**。本轮发现的是修订后仍保留的失败传播规则与新端点收口oracle的偏差。

reviewer：当前Codex会话，非新独立上下文；未创建agent/第二checkout。outcome=needs_fix，未发最终pass。

## 输入与范围

| 输入 | 绑定 |
|---|---|
| 用户指明簿记提交 | 1210008cc6f67f25f2de6dbd9b6dabe84c83d1af（当前master祖先） |
| 实际主检出审查位 | 92f709da857e66598a3dec4cbc247258edd6c073 |
| lang树 / branch | D:/autostack/.wt/lang-831/auto-lang / codex/plan-831-dev |
| lang reviewed / base | 2d47e236eb7de6c9d275bbe7629b822c788f1820 / fe8f3090a35c5720bcac873edba38d9f588a5663 |
| UI树 / branch | D:/autostack/.wt/lang-831/auto-ui / codex/plan-831-dev |
| UI reviewed（未变） | 425088520e402b1efae9479b9d0b5b3d44a56df4 |
| down只读 | c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 |

本轮lang仅两个提交、一个driver的188行差异；UI实现/配置/依赖未变。两实现树tracked clean，lang可再生untracked compile-probe/Cargo.lock不变。freeze中7份材料hash匹配，4份原始收据hash及三仓sources匹配；当前canonical spec_inputs无漂移。SD-01..07/r3修订文本已读，其目标/owner/840并集方向保持。

未改产品代码/依赖/配置、未跑整档Cargo/792全链、未合入/发布canonical-ledger/清树。本轮检查范围为V06c、V06e与r3边界；原兼容24/24、blueprints5/5、777真双根、许可证/测试选择面、净迁移和其他未变范围按R1/R2证据复用。

## 闭合确认

F03-A旧问题（缺jet/ark工具链仍阻塞）：**按r3合同修订关闭**，并保持生成六profile不缩面。jet/ark所有真实尝试仍须记录；不冒称它们已构建成功。后续环境门控承接项应在KNOWN-DEBT保留，本轮补记为经用户批准的非阻塞延期，不把它与下述合同内缺陷混淆。

F03-B旧progress-only反例：**原例关闭**。复跑R2的replay-vm-progress.py，只有合法Progress(resolve backend)而无Launched/端点的载体现为false_positive=false、退出0。运行证明从任意进度改绑Launched+活性探针，方向正确；新版完整收口不变量仍有下述窄缺口。

终源原始正控：792 39/39；V06c的vue/rust/vscode completed、jet9009/ark缺wrapper逐项登记；V06e真实Launched→HTTP200→cancelled→ConnectionRefused→describe_ok（3s），其hash/source均匹配。distribution5/5、drill19/19等未变证据保留。这些有效正控不覆盖下面两个失败状态。

## F831-09-03 · P1 · PARTIAL / OPEN（同一ID，两个受影响子面）

### A. V06c错误地忽略jet/ark的一切失败

**既有r3规则**：Plan §4及driver自己的注释均保留“jet/ark工具链在场且构建失败仍判红”；被批准转后的是缺环境的真实构建验证，未批准吞掉已执行的编译错误。

代码：driver `required_ok = all(ok for p, ok, _ in build_results if p in BUILD_REQUIRED_PROFILES)`只观察vue/rust/vscode。jet/ark的env_blocked仅用于detail字符串，不参与最终判据；因此这两项无论缺环境、编译错误还是会话异常，均不影响case通过。

**原样V06c代码块+真实framed子进程的五个控制**（所有profile均实际发送build_ui请求，返回完整协议RunReport）：

| 输入 | 按r3应判 | 实判 |
|---|---|---|
| 五profile全completed | 绿 | 绿 |
| required completed；jet9009/ark缺wrapper | 绿（环境转后、逐项记录） | 绿 |
| required completed；jet=Kotlin编译错误“unresolved reference Renderer” | 红 | **绿** |
| required completed；ark=ArkTS编译错误“incompatible assignment” | 红 | **绿** |
| vue=真实构建错误，其余completed | 红 | 红 |

负控报告不是宣称本机已安装Android/DevEco；它直接验证汇总器对**有效的真实编译失败报告**是否传播失败。其合法环境正控同时证明修复不能简单把jet/ark所有红恢复为阻塞，从而撤销r3。

最小闭合：区分经r3授权的环境缺失与在场执行后的真实失败/协议失败，后者传播红；required三个profile仍必须全完成，五profile全尝试/全记录。补上述兄弟控制与实际已存在的缺环境正控即可，不要求为修这个汇总器安装被r3移出的工具链或重跑全部发行长链。

### B. HTTP超时被当作同端点已关闭

**既有规则**：F03-B修复声明以同一MCP端点在cancel后的独立关闭观察证明资源收口；本Plan AC-02/V-06与原R2要求区分运行/关闭真实状态，不能只相信报告文字。

代码：`_mcp_alive`将所有异常（连接拒绝、读取超时、其他网络错误）统一返回False；cancel循环的`if not still: endpoint_closed=True`把未知或无应答等同端点已不存在。查询从开始HTTP200到结束超时，不能推出TCP listener已关闭。

**已实测控制**：一个真实自有TCP监听socket在Launched后正确回答HTTP200；收到cancel后继续listen/accept，只令HTTP响应阻塞。工具仍返回完整合法cancelled/RunReport/describe。原样V06e最终：

```text
Launched=http://127.0.0.1:1907 探活[HTTP 200]
outcome=cancelled；同端点收口=True（TimeoutError('timed out')）
describe复验=True；V06e ok=True
```

在V06e报告完成之后、测试清理工具进程之前，独立`socket.create_connection`对**同端口**成功，确认监听者仍在。负控随后只终止自己创建的进程，无残留产品进程。端口由OS动态分配，1907为该次实测值，重放不依赖固定端口。

最初的诊断脚本因Windows路径转义破坏而未启动，不计作产品或oracle证据；修正为有效路径/HTTP载体后才得到上述结果。有效控制stderr空；错误初探记录保留在scratch，本报告及controls只引用已确认输入。

最小闭合：把“连接被拒绝/端点已确证关闭”和“超时/未知探测失败”分开，未知不能直接变closed绿；保持当前真实MCP可达正控、真实cancel后ConnectionRefused正控、启动前progress反例，再加这个仍监听但不回答的状态。无需强化成新规模/新平台承诺，也不能通过延长固定sleep掩盖判断错误。

此finding是验收器缺口，不声称真实VM当前有泄漏；终源真实正控确实ConnectionRefused的事实保留。完整资源/寿命结论须有区分力的验收观察支持。

## 重放、AC/SD与下一步

可复制入口：`replay.py --case build-matrix`与`--case endpoint-timeout`，在lang权威树运行，`--lang-repo`显式指定该树；脚本只读当前driver，写自有组内scratch并回收自有进程。详controls.json及receipt.json。build控制包含合法全完成/获准转后/required失败，endpoint控制包含前段真实HTTP200与后段独立TCP证据。

本轮AC-03仍partial（**汇总失败传播缺口，不再是jet/ark缺环境本身**），AC-02/06仍受V06e收口oracle限制；AC-01/04/05其余未变范围与SD-01..07既有方向/来源证据保留，AC-07未发final pass、AC-08 merge pending。SD的r3边界被接受，旧R1 named-partial句子应在下一终稿标明历史，避免与r3当前规则混读，不另立新阻塞ID。

结论：r3合同有效且应保留，F03原两例分别由合同修订/实现修复闭合，但两个变更路径的完整不变量仍未闭。继续沿用F831-09-03一个finding，T-09重开，Plan维持revision3并回executing8/12。修复只做受影响正反控和必要正控重验，不重启其他已闭族/整档。下一轮交接应说明“获准延期/真失败/未知探测”三类如何在判据中区分，不能只给原样例总数。

merge-owned裸cargo t与实际触面档仍待最终pass后执行；tf主检出到期批量，T-11/T-12均pending；U831未成立，832不放行。当前会话独立性限制保持。
