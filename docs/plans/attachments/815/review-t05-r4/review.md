# PLAN-815 T-05 R4 与 T-06/T-08 提前定向复审（2026-10-22）

**needs_fix。T-05/T-06未通过，T-08准备稿partial；Plan保持executing10/17。** R3外部模块与初始化q原例关闭，Windows持管道错误分类原例关闭；但非Windows清理不变量仍未闭。新增支持矩阵存在七项实质问题，既闭consumer/卫生项因本轮变化重新开放，共十项需处理。不是T-09完整B终审，也未复审后续T-07新提交。

## 冻结与证据边界

- reviewer=Codex `/root`，延续审查上下文，有早期合同/账本参与限制，未参与T-05/06实施；未创建agent/第二树。使用`/auto-plan:review`与autoui-verifier既有AutoUiMcpClient获取VM真实state/snapshot，不声称双端视觉一致性。
- 主检出Plan r3、唯一树`D:/autostack/.wt/lang-815/auto-lang`、branch `codex/plan-815-dev`；开始时clean HEAD **02401f77cc31c0c4196f588b6b1ccfe2ae10d530**。T-05修复50a743b7f33d82c2ec45a0be4c4adc58ee5c49da；T-06 347d09d281910f141c85b85b7e85d67832071166；修复基面64ca649ffe91d61efd20610ecc4b86fc78607c9d；M814-B祖先93a25c924d1beafab130588bad4d1041dc86e48c。13个净改文件读审，含新分派/协议/上下文和交接稿。
- [初始身份/输入hash](baseline.json.txt)、[终核与漂移对账](final-baseline.json.txt)、[冻结交接稿](frozen-handoff.md)、[规范核验](spec-delta-reviewed.md)。开始后实施方新增**9ca1130f90b17dc0f3d19c475c7d5d7e47c9d9f1**（T-07测试/fixture），另有tool_process格式WIP。生产源码与原冻结提交逐字一致，故本轮原bin/行为结论有效；新增T-07未获得本轮pass，脏改不能作为已提交修复。最终HEAD/status以终核为准。
- 冻结提交的tool_process负控已将jet改成unreal-engine，故74项根测试全过；独立consumer却仍保留旧断言，真实3/5，不能沿用工作交接5/5。

## 原例关闭与继续开放

| Finding | R4裁定 | 证据/边界 |
|---|---|---|
| F-815-T05-R1-01 / P2 | PARTIAL，继续OPEN | Windows held-reader持有者句柄复制成功，约563ms报告明确`reader for install stopped by teardown without a natural pipe EOF`；线程5→6，释放holder后仍6，原泄漏/EOF误分已闭。非Windows仍read_until阻塞，超时后无条件join，见下。 |
| F-815-T05-R3-01 / P1 | 原例CLOSED | 真实SDK ABI外部后端和本地同源对照都呈现R3_EXTERNAL_CONTEXT；模块根传递有效，keepalive寿命覆盖会话。相邻串行隔离/装载错误回归另编号R4-01/02。 |
| F-815-T05-R3-02 / P2 | CLOSED | 两次有效InputAck在Launched/first view前，最终completed（约1602/943ms）；已呈现q正控同样completed。重放VM启动cancel/EOF保持。 |
| F-815-T05-R1-07 / P2 | REOPENED，本轮能力扩展使consumer过期 | `verify-targeted.py`实际独立consumer3/5。build_ui是cancellable，旧match仍把除run_ui外全部断言为不可取消，line134 panic；零副作用负控仍用已服务jet，先读到Progress而非Error。修复断言、使用真实不服务目标并保留合法支持正控，再执行完整独立consumer，不能删掉负控。 |
| F-815-T05-R2-02 / P3 | REOPENED，本轮新增诊断/冻结格式红 | runtime由235→237 warning：renderer.rs:18978对语句用doc comment、:18980未读host_exit_since；冻结tool_process.rs长行rustfmt输出不相等（其他11文件相等）。后续未提交格式WIP不覆盖024收据。修齐并提交，受影响诊断/冻结格式复验。 |

其余已闭结论保持限定范围：R1合法大stdin控制可响应、原Vue owned子树取消、Rust取消分类、worker fatal/stdin开放退出等本轮实际重放。T-06直接Command新分支不被这些旧例自动覆盖。

### F-815-T05-R1-01 的剩余平台路径

AC-03/V-03、T-05，原合同内缺口。proc_host.rs:741在非Windows调用阻塞`read_until`，stop只能在读取前后检查；非owned持管道且无新行/EOF时不会返回。:252截止循环break之后仍无条件`handle.join()`，所以超过10s既不能披露最终错误也不能返回；输入writer :441同样需核查有界声明。此分支源码与R3相同，本轮未有Linux执行环境，不伪称实跑失败或闭合。关闭要求在声明支持的平台取得可停读取/真实结束和有界错误正反控，不能用Windows PeekNamedPipe结果覆盖Unix，也不以detach代替资源真停。

## 七项新增finding

### F-815-T05-R4-01 / P1 — 后一个请求沿用前一个外部后端

AC-02/03、T-05，R3上下文修复引入的会话隔离回归。入口`prepare-context.py`后`session-context.py`；真实SDK ABI后端，无伪响应替代成功链。

第一请求external_backend=../backend，实际state.title为R3_EXTERNAL_CONTEXT并Input q正常完成；同宿主第二请求external_backend=null、本地src/back/api.at明确返回LOCAL_SECOND_SESSION，却再次装载第一后端并显示R3_EXTERNAL_CONTEXT。完全相同第二请求换新宿主则显示LOCAL_SECOND_SESSION，两个案例都真实呈现并completed。原始四例/请求/状态/日志见[session-context](session-context.json.txt)。

已确认：run_vm.rs:374仅在Some且api存在时set_var AUTO_UI_TOOL_VM_BACK_ROOT，None/无效根不清理；helper继承宿主全局env。:507主题声明同样仅Some写入，是需核查的同因sibling，未声称主题已实测败。关闭：每个执行进程只获得本请求上下文，外部→本地/无后端、主题Some→None、失败后新会话均隔离且不重读主pac；重放第二请求及新宿主对照，不能删外部能力绕过。

### F-815-T05-R4-02 / P2 — 必需外部后端装载失败仍报告完成

AC-01/03、T-05。同一真实后端源，仅复制api.at/pac.at到新空目录，不提供target动态库；明确请求该目录。child日志确定`backend cdylib review_backend.dll not found`，但前端继续呈现，q后RunOk.completed/error=None。成功DLL装载正控同脚本第一例成立，故不是坏ABI被当成功证据。来源/观察仍[session-context](session-context.json.txt)。

bin/auto_ui_tool.rs:90把prepare失败降成eprintln后继续run_entry_file；旧run_vm非托管同分支则返回external backend load failed。关闭：保持既有声明后端失败传播，缺库/ABI不匹配等实际装载失败必须具名failed、不启动依赖它的会话；正常外部/local与取消保持。不能让日志警告代替结构化失败。

### F-815-T05-R4-03 / P2 — 三个生成profile报告不存在的输出目录

AC-03/04、T-06。`profile-contracts.py`对实际vue/jet/ark/vscode生成，四请求均GenerateOk；vue正控17文件且目录存在。Jet/Ark/VSCode却报告android、ohos、gen/vscode，均不存在且files_written=0；实际产物在gen/front/{jet,ark,vscode}。完整文件清单和回执见[profile-contracts](profile-contracts.json.txt)。

tool_ops.rs:255-257手写目录与各generator实际output_dir不符。关闭：从实际生成结果/一致的布局来源回报真实路径及计数，逐profile核对存在且文件walk一致；Tauri还需明确前/后端多输出的覆盖边界，不能只改测试预期接受零文件。

### F-815-T05-R4-04 / P2 — 新工具入口重新读取主pac，形成第二真源

AC-02/V-02、T-06。同`profile-contracts.py`：冻结请求identity.name=request-name，磁盘pac为disk-name；VSCode实际package.json.name/displayName/commands全使用disk-name。Vue同输入请求路径正控成功，完整请求和实际package均冻结。

tool_ops.rs:89调用无request的generate_vscode_project，vscode.rs:118-130读主pac并解析name/config；build/run也走同legacy链。另:139 Vue build调用build_vue_project→prepare_vue_sources→VueProject::from_workspace，源码确定同样绕过request。未把tauri_backend读取普通源代码误报为重读主pac。关闭：新工具所有生产入口消费已求值请求/明确source adapter；主声明修改/移除不改变快照字段，动态源/API等允许读取边界明确。VSCode全操作、Vue build及相邻profile须按字段来源表核查并补正反控，旧Man legacy读取可保留在授权适配边界。

### F-815-T05-R4-05 / P1 — 新构建子进程未纳入取消/EOF所有权

AC-03/V-03、T-06。`profile-contracts.py`生成真实Jet项目后运行`build-lifecycle.py`。仅Gradle包装器换成本次私有可控OS子进程（负控，不作为Android构建成功证明），记录自身PID后等待30s。真实build_ui到达包装器后Cancel得到Cancelled回执，5s内没有BuildOk，PID仍活；Describe可响应，关闭输入2s后宿主仍活。工具与受控后代均由探针仅按本次所有权清理。见[build-lifecycle](build-lifecycle.json.txt)。

Jet build jet.rs:1072直接Command::status，不经proc_host/auto-tool-execution，因此RunIo.children为空，cancel只置标志，不能中止status。Jet run/Ark/VSCode/Tauri的外部命令是相邻需逐点核查路径，未声称均已实际跑败。关闭：所有已宣称cancellable/interactive的实际路径使用统一owned树设施，保持日志/控制输入分离，取消/EOF后树真实终止、worker最终结果/宿主收口；串行下个请求可用。只改回执/在命令后补cancel检查不够。

### F-815-T05-R4-06 / P2 — 实际构建失败被包装为completed

AC-03/04、T-06，同脚本同生成工程，包装器exit0和exit7都已通过PID标记确认被执行，二者最终均BuildOk.completed/error=None（约191/202ms）。合法成功与确定失败仅退出码不同；不涉及Gradle/Android环境推测。

jet.rs:1092打印Build failed后Ok，tool_ops.rs:152/230把Ok直接映射completed。Ark的缺hvigor wrapper和非零退出同样源码返回Ok，是相邻错误传播路径；未把手工提示当真实构建成功。关闭：失败/缺工具/未真正执行等以结构化failed/具名拒绝诚实回报，实际成功另用产物oracle；必要时在保留旧门面语义的同时为协议层提供真实结果，不能只靠stderr或无条件Ok。

### F-815-T05-R4-07 / P2 — 多profile生成只执行第一项，却回报全部完成

AC-01/02/03、T-06。`multi-profile.py`传合法Multi [vue,jet]；GenerateOk.frontends=[vue,jet]，实际vue/package.json存在，jet/settings.gradle.kts不存在，完整文件清单只有Vue。单Jet正控前述已实际生成，故不是缺工具链。见[multi-profile](multi-profile.json.txt)及冻结wire。

tool_ops.rs:36仅取frontends[0]，最终却回报原frontends全集。关闭：按既有多profile语义完整生成并诚实聚合产物，保留顺序/支持校验；若有不支持组合须在副作用前具名拒绝，不能静默漏项或直接削减已约定能力。重放单目标及至少两种顺序/含不服务项负控。

## T-08准备稿与后续边界

交接稿不是可接受的完成交付：所谓“二进制”字段是.rs源路径，缺实际source SHA/bin SHA/支持指纹；Cancel只列RunOk遗漏BuildOk；称全profile真实运行/统一owned树与当前反例冲突；SD仅01..06索引，漏已登记07及全部增量正文。工作记录已承认正文待补，本轮将其记为T-08 readiness gap/partial，未重复增加产品finding。按原依赖T-07验收完成后，补齐准确身份/逐条SD正文和来源再送审；详细[规范核验](spec-delta-reviewed.md)。

T-07后来的新fixture/lifecycle测试未在本轮范围；独立consumer红属于已有受影响验收而非要求提前最终发行。新lifecycle_build_produces_dist源码接受任意Error/Failed且不检查dist，不能当实际构建成功正控；此为T-07后续审查输入提醒，不给本轮pass或另立已验完整T-07结论。

## 验证账与无效探针

- 实际bin build --offline成功；根定向nextest **74/74（1 LEAK，既有DirectChildOnly负控）**、Man兼容/媒体/请求缝 **6/6**，合计80个不同Rust测试通过。独立consumer **3/5，exit101**；不能汇总成全绿。命令/原始日志在本目录及targeted-checks.json.txt。
- 12个净改Rust冻结源码rustfmt输出逐字比较：11相等、tool_process 1不等；frozen diff-check通过。runtime明确新增两条warning，ui-tools仍22条既有诊断。
- 原R1协议/owned树/合法大stdin、Rust取消、真实VM取消/EOF、启动边界与stdin开放fatal重放的原始结果在replay-results、vm-initialization.json.txt、worker-fatal-open-input.json.txt。实际UI功能使用已有MCP客户端state/snapshot；未跑视觉截图或双端像素验收。
- 初次session-context误将Input编码成字符串，协议正确拒绝、第二请求busy；保留session-context-invalid-input.json.txt，不计产品缺陷。改为字节数组且取得InputAck/首会话终态后才使用最终串行反例。
- 初版Gradle Python helper继承协议stdin未在时限内写PID，wrapper_reached=false，不用于exit7/树存活判定；build-lifecycle-initial.json.txt保留。最终helper stdin=NUL，三例均marker=true后才判定。继承stdin本身应随R4-05核查，但本轮不编造具体Windows阻塞机制。
- 777/792并集、fresh/locked发行完整资源矩阵仍归待审T-07；既有清单hash保留，未把旧777结果充作新增fixture/HEAD并集结果。本轮没有整档t/tv/tt/tb/tu/taa/tf/docs_gen，也未修生产代码、改canonical/live ledger、合入、归档或清树。

## 当期AC和交回

AC-01/02局部fail，AC-03 fail，AC-04 fail/完整面pending，AC-05当期卫生fail；A/L及T-04历史已闭范围保持。T-05/T-06继续未勾，T-08准备稿partial；T-07新提交与T-09终审pending，Plan executing10/17。

原唯一树work一次修齐十项（R1-01、R1-07、R2-02、R4-01..07），补字段/多profile/失败与资源所有权的区分性矩阵及T-08正文后提交新冻结SHA复审。不能以旧反例关闭或测试总数代替完整不变量；不批准缩面/延期/把合同内缺口转非阻断债务。最终review pass之后才由merge在冻结提交统一执行裸cargo t及实际触面档、并集/实际发布；本轮不授权M815-B交付消费。
