# PLAN-815 T-05 R5 / T-06 / T-08 定向独立复审（2026-10-22）

**needs_fix，executing10/17。** 旧十项七项CLOSED，三项PARTIAL/OPEN；新增一项旧公开入口兼容回归。当前共四项：R1-01(P2，平台验收证据pending)、R4-04(P2，主声明第二真源)、R4-05(P1，捕获排水取消/终态)、R5-01(P2，旧构建返回语义)。T-05/T-06/T-08保持未勾，非T-09完整B终审。

## 冻结身份与独立性

- 代码 **8a08e84fe25ec81e164ba50b9f9e970e705505e0**，交接HEAD **10e4e1cc1a840132513317424afa7bba4dec5262**（其后仅文档，无生产差异），修复基面9ca1130f90b17dc0f3d19c475c7d5d7e47c9d9f1；R4受审基面02401f77cc31c0c4196f588b6b1ccfe2ae10d530。唯一树`D:/autostack/.wt/lang-815/auto-lang`、branch `codex/plan-815-dev`，开始clean、结束身份/漂移见final-baseline.json.txt。
- reviewer=Codex `/root`，延续审查上下文，有早期合同/账本参与限制，未参与当期修复。用`/auto-plan:review`从实际代码/有效输入重建结论；无新agent/第二树。autoui-verifier仅复用既有MCP客户端state/snapshot，不声称双端视觉验收。
- M814-B已发布祖先93a25c924d1beafab130588bad4d1041dc86e48c；Plan r3/验收合同保持。[baseline](baseline.json.txt)绑定锁/配置/所有修改和当前Specs；[冻结修复表](frozen-fixes.md)、[交接稿](frozen-handoff.md)、[SD核验](spec-delta-reviewed.md)均保留实际文字，未改canonical/live ledger。
- 独立重建bin SHA256 **ab88ccb440e007ad4ac243e28f693f05b431d809e4ef8f4e66ec3e9d0583d775**，与T-08交接逐字一致。14个净改Rust全部读审/格式相等。共享ExecutionSpec.env_remove新增字段及调用方已核对；受影响包定向测试通过。

## 十项关闭表

| 原finding | 裁定 | 实际证据与边界 |
|---|---|---|
| R1-01 非Windows清理 | PARTIAL/OPEN(P2) | 原阻塞read_until已改poll(2)，超时后不再无条件join；Windows held-reader真实分类/线程终态仍通过。Unix新测试尚未执行，修复表自己明确pending，不能称十项全闭。详下文。 |
| R1-07 consumer | CLOSED | 独立consumer --offline --locked 5/5；build_ui能力与generate不服务vm负控正确，保留实际进程与零副作用断言。 |
| R2-02 告警/格式 | CLOSED（修复触面） | runtime235条既有诊断，原新增两条消失；ui-tools22既有诊断；当前冻结14个Rust格式全相等。后续T-07 lifecycle测试已有5条诊断独立记账，未宣称全仓零warning。 |
| R4-01 串行上下文 | CLOSED | 标准MCP实际state：同宿主外部→本地第二请求=LOCAL_SECOND_SESSION，与新宿主同请求一致；每请求env_remove后追加、不写宿主内部键；主题None/Some的装配分支静态核查，不虚构像素主题实测。 |
| R4-02 缺DLL假成功 | CLOSED | 实际缺DLL case不呈现，RunOk.failed、launch=None、error含具体cdylib路径；真实SDK ABI外部后端成功正控与local对照保持。 |
| R4-03 输出根 | CLOSED（已验profile及布局） | 实际vue/jet/ark/vscode根存在，17/24/25/8文件；Tauri双根布局已入规范和源码，完整Tauri运行/发行仍T-07，不冒充各平台工具链均实跑。 |
| R4-04 第二真源 | PARTIAL/OPEN(P2) | 原VSCode身份字段现在正确request-name，Vue build项目上下文已走request；但同链pkg::resolve_impl仍读取当前cwd的pac.at，真实wire与磁盘冲突会执行磁盘指定PM。详下文。 |
| R4-05 owned取消 | PARTIAL/OPEN(P1) | 原Jet build sleeper树被杀、最终cancelled、EOF0；新ADB捕获stdout分支未检查任何停止/截止，持管道反例取消和EOF不能收口，释放后还completed。详下文。 |
| R4-06 构建失败 | CLOSED（协议原例） | 原exit7具名failed；exit0无APK具名failed，不再假completed。严格逻辑无区别落旧公开入口产生新兼容回归，另列R5-01。 |
| R4-07 Multi | CLOSED（原例及永久反例） | 实际[vue,jet]两根存在，41文件、共同根gen/front；新增具名永久测试同时通过。支持校验先于副作用，去重/arkts规范化源码已读；未宣称完整任意组合/发行矩阵已最终通过。 |

原已闭VM初始化q两例及已呈现q、真实VM Launched→Input q→第二会话cancel、原Vue owned树/合法大stdin/控制fatal和Rust取消本轮重放保持。完整T-07/T-09不因这些局部结果变完成。

## 剩余项与关闭oracle

### F-815-T05-R4-05 / P1 — 新ADB捕获分支仍不能停止，取消后还报completed

**AC-03/V-03，T-05/06，原生命周期完整不变量继续开放；本轮新捕获封装的相邻回归。** 与原build sleeper例区分：原例确已关闭，不将它回滚成失败。

- 重放：`python docs/plans/attachments/815/review-t05-r5/adb-held-pipe.py`，生成真实Jet工程，经真实run_ui→gradlew→ADB捕获链。仅Gradle/ADB为私有可控OS负控，不作为真实Android成功证明。ADB小程序源码由脚本原样落盘/rustc编译，通过DuplicateHandle将stdout句柄交给本次探针另起、非tool-owned holder，自己退出0；实际dup=true/非零handle证明目标路径到达。
- 自然EOF正控：相同受控ADB不复制管道，约358ms有RunOk.completed。负控：收到Cancelled回执，随后**14s没有最终结果**；Describe仍可响应，关闭stdin后再等2s，宿主仍活。只有显式释放本次私有holder后才得到RunOk，且**outcome=completed/error=None**（约16s）。远超代码10s清理界限，并非正常2s自然排水窗口。[原始帧/dup/PID/日志与正控](adb-held-pipe.json.txt)。
- 已确认：jet.rs的adb臂在wait_tracked之前调用proc_host.rs:1013 `drain_captured_stdout`，该loop没有cancelled/reader_stop/截止检查；capture_stdout分支没有注册forwarder，所以新的两阶段join也不覆盖它。非owned持管道者不归tool可杀范围，轮询本身不会自动退出。释放后has_device=false返回Ok，tool_ops.rs:287对Ok(Completed)不顾取消标志，导致错误终态。
- 关闭：包括caller-drained pipe在内所有实际读者都可响应本操作取消/EOF，真停止或有界具名失败，不能靠杀非owned进程/延长超时/静默detach；最终被取消操作须cancelled，下一请求可正常运行。覆盖自然EOF、owned/非owned持管道、取消与EOF各自入口、读取错误；保留旧build取消例。捕获数据不能无界收集且错误不能当完整输出，这是同分支需核查的边界，未声称本轮单独触发了超量案例。

### F-815-T05-R4-04 / P2 — Vue build同链包管理器仍从主pac取值

**AC-02/V-02，T-06，原单源不变量继续开放。** 原VSCode identity例关闭，完整项目读取仍不满足。

- 重放`package-source.py`：真实build_ui请求raw中pkg=npm，tagged Value通过真实codec；前端/身份等合法，wire两例一致，分别以磁盘pkg=npm作正控和磁盘pkg=pnpm作冲突负控。两个私有PM包装器立即写标记并exit7，不安装、不联网；二者都进入真实Vue build→npm_install链。
- 实测：同wire，正控执行npm，负控执行pnpm（fallback第二次仍pnpm），结构化错误也分别是npm install failed / pnpm install failed。[输入值/实际选择/完整日志](package-source.json.txt)。这是已到达生产执行分支的配置漂移，不是仅字符串检查。
- 已确认：pkg.rs:64-69 `resolve_impl`读取cwd/pac.at，CACHED_PM再将结果进程级缓存；新build_vue_project_with_request只替换了项目上下文，后续project.npm_install仍调用这条旧链。原计划§5.2/AC-02明确无主声明第二真源，T-04的pkg单算法owner偏差记录不等于授权主声明重复读取。
- 关闭：工具侧所有主声明字段（含未建模而保留在raw的pkg）经当前请求/明确adapter交给生产执行；legacy可按合同保留自扫。补同请求磁盘改变/移除、不同请求/不同cwd与串行PM选择正反控；不能删除字段/忽略原配置以绕过。对外global config/env覆盖保留已约定优先级并明确来源，不把当前cwd误当request.project_root。

### F-815-T05-R1-01 / P2 — Unix平台验收证据仍pending

**AC-03/V-03，T-05。** 源码修复成立：Unix poll就绪后read、10ms轮询，reader/input writer超时不再无条件join；不能继续声称旧read_until代码仍存在。新`unix_reader_truly_stops_when_non_owned_process_holds_pipe`有持管道与自然EOF对照，但本轮Windows编译cfg裁剪它，没有运行Unix代码。frozen-fixes.md第一行关闭表也明确“Unix真停测试未在本机执行”。

独立查得本机有WSL Ubuntu-22.04，但其中无cargo/rustc，只有cc；只读核查未安装工具链或改环境。因此不能把Windows78/78当Unix实测通过。本项是未满足的目标平台验收证据，不是声称已实测Unix产品失败，也不替代R4-05已证实的Windows capture bug。关闭需声明支持的Unix环境对实际冻结源码执行具名真停/自然EOF/有界清理测试及必要编译，绑定依赖/目标/命令/结果。源码阅读不等于执行证据；不把work-owned定向验收后移成merge整档pending。

### F-815-T05-R5-01 / P2 — 严格失败处理改变了旧公开构建返回语义

**AC-01/V-01、§5.4旧返回/错误语义不静默改；T-06，本轮修复回归。** R4-06要求协议诚实失败，其关闭oracle同时要求保留旧门面约定；不是允许把全部旧入口默认升级成新错误合同。

- 重放`legacy-build.py`：临时独立consumer直接调用**非托管**公开`auto_ui_tools::jet::build_jet_project`，真实Jet声明/前端生成成功后，私有gradlew exit7。当前实际输出REVIEW_LEGACY_RESULT:Err("jet build failed ... exit code:7")。该探针未install RunIo，也没有with_request，故明确到达旧入口。[完整编译/调用日志](legacy-build.log.txt)。
- 基面9ca1130f9同函数构建非零分支仅打印提示后Ok；8a08将其无条件改为Err，shared inner同时服务legacy/with_request。同类Ark缺wrapper/nonzero、VSCode缺工具/编译失败也无差别改变旧返回；这些siblings为源码确认，未逐项实际运行。交接稿“legacy直跑冻结”目前只冻结了Command形态，未保持结果语义。
- 关闭：按既有合同隔离协议严格结果与旧公开门面，补旧/新入口的成功/非零/缺工具对照；或由用户明确批准语义合同修订后按plan revision重新对账。不能靠修改旧oracle或把兼容回归标成无关预存来通过。具体API/编排实现由work决定。
- 首次探针沿用默认vue声明，未到构建退出分支，日志保留legacy-build-invalid-input.log.txt，不计finding；最终改成真实jet声明后才形成上述证据。

## T-08与验证账

T-08原具体缺口（真实source/bin/支持指纹、BuildOk取消分支、SD-07与正文）已补齐，独立bin hash一致；但“全部生产入口单源”“所有捕获读者有界”“legacy语义冻结”与上述反例冲突，Unix承诺缺平台实测，完整777/792/fresh/locked/资源仍T-07。按原依赖，不能勾T-08或发完整B规范。见[逐SD当前版本核验](spec-delta-reviewed.md)。

- build --offline成功；根定向**78/78（1既有DirectChildOnly负控LEAK）**、新四永久反例+真实VM交互**5/5**、Man兼容/媒体/请求缝**6/6**、独立consumer **5/5**，共**94个不同Rust测试通过**；不覆盖上文失败探针/Unix cfg裁剪分支。原始命令和日志在run-review.py、scoped-tests.log.txt、ignored-targeted.log.txt、targeted-checks.json.txt等。
- 当前冻结14个净改Rust rustfmt输出逐字相等；runtime235/ui-tools22既有诊断，原两新增已清。T-07 lifecycle的5条既有诊断尚归其后续审计，不虚构全仓零warning。代码修复diff卫生通过；包含交接证据的diff-check仅报告旧独立consumer日志末尾新增空行（非代码/不另编号产品finding，收据可整理）。
- 原脚本verify-targeted.py内部fmt仍写死R4 SHA，首轮fmt.json.txt因此在检查旧提交，不能用它断言当前仍红；已用check-final.py改为R5完整SHA独立检查，最终依据fmt-current.json.txt。该无效验证产物保留，未改变产品格式要求。
- ADB/Gradle/PM helper只作错误/所有权负控；真实SDK ABI backend和原有MCP state/snapshot才用于VM功能正控，二者不混充。所有辅助进程只按本次私有所有权清理。未做双端视觉/像素声明。
- 未跑整档t/tv/tt/tb/tu/taa/tf/docs_gen；777/792并集与完整发行资源矩阵仍T-07，不复用旧收据冒充本轮新依赖审计。未修生产代码/改canonical/live ledger/合入/归档/清树。

AC-01/02局部fail、AC-03fail且Unix必需证据pending、AC-04/05完整B审查pending（当期代码卫生修复通过）；A/L/T-04及未受影响既闭边界保持。原唯一树work修齐当前四项、补有效平台与相邻状态/字段/旧新入口对照、修正SD保证后提交新冻结SHA复审。计划r3 executing10/17，T-05/T-06/T-08继续未勾；最终pass后仍须merge-owned冻结裸t+实际触面档和发布收口，不提前交付M815-B。
