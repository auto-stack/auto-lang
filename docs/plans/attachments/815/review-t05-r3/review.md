# PLAN-815 T-05 R3 定向独立复审（2026-10-22）

**needs_fix；T-05未勾，Plan executing10/17。** R2的六项中五项关闭，F01仅“reader真停”部分关闭，停止/EOF错误分类仍开放；新增两项进程隔离/交互启动回归。当前三项：F-815-T05-R1-01(P2)、F-815-T05-R3-01(P1)、F-815-T05-R3-02(P2)。

## 基线与范围

- 主检出Plan r3/簿记573e4c3b2已读；唯一树 `D:/autostack/.wt/lang-815/auto-lang`、branch `codex/plan-815-dev`、clean HEAD **64ca649ffe91d61efd20610ecc4b86fc78607c9d**，repair_base **54aadefadc803af91f6095f8e594b938848d84bd**；原T-05基面13b41e9d47cfe62768f68f73b658c3c92b9d3d48。九个净修改文件全部独立读审。
- review_scope=phase/T-05提前定向，不是M815-B/T-09终审。reviewer=Codex `/root`，有早期合同/账本参与限制、未参与实施，从源码/真实入口重建结论；未创建agent/第二树。
- M814-B已发布祖先93a25c924d1beafab130588bad4d1041dc86e48c保持。新子进程接线、轮询reader、代计数器是实质替换路径，旧单函数回放不能代替相邻状态核查。
- [baseline](baseline.json.txt)绑定实现/清单/锁/配置、最新计划/合同/current Specs；[终核](final-baseline.json.txt)零漂移且tree clean。重建真实bin SHA256 **2850357aba75d6eaffea5f07fcc22cdc995e15bce225bfc4602f90e4193ac8f9**。
- [冻结SD-01..07核验](spec-delta-reviewed.md)保持原验收、增量尚不具完整发布事实；未改canonical/live ledger。复用autoui-verifier的原有AutoUiMcpClient获取VM State/snapshot，仅作本次功能oracle，不声称Vue/VM视觉或像素一致性。

## R2六项关闭表

| Finding | R3裁定 | 可重放证据与边界 |
|---|---|---|
| R1-F01 | PARTIAL/OPEN(P2) | owned树/reader资源真停改善成立：held-reader线程终态6→释放holder后仍6，旧7→6泄漏消失；但holder仍持管道时被当成自然EOF，报告遗漏收尾失败，详下文 |
| R1-F03 | CLOSED（原实际cancel/EOF） | vm-initialization的cancel-init两例均约76/80ms返回cancelled，eof-init两例/已呈现EOF均exit0。由终止owned VM子进程保证；不是代计数API初始化语义本身被证明正确，新q反例归R3-02 |
| R1-F04 | CLOSED | stdin保持开放、真实控制读端关闭的Describe/worker Generate两例均直接exit1，不再等EOF；源码progress/Launched/input writer均走fatal关闭，正常EOF/坏frame正控保持 |
| R2-01 | CLOSED（原基本连续VM会话） | 同一真实宿主第一次已呈现后取消，再启动第二个VM无RecreationAttempt；真实交互测试另覆盖自然q结束后第二会话cancel/Describe/EOF0。外部后端跨进程丢状态另归R3-01 |
| R2-02 | CLOSED | 净九文件中所有Rust含fixture的rustfmt逐个exit0；launched新增dead_code消失，ui-tools回22条旧诊断；diff-check通过 |
| R2-03 | CLOSED（真实VM profile正控） | 新具名test真实VM→Launched→Input q→RunOk(completed/error=None)→第二会话cancel→Describe→EOF0，独立1/1；独立探针等first view后q同样completed/launch准确，MCP现有客户端实连动态端口取得State/snapshot。不是伪服务成功。完整Vue/其他profile/媒体/发行矩阵仍T-06/07 |

R1早已关闭的F02/05/06/07保持：合法大stdin下控制响应/树退出、Rust取消及普通exit7失败对照、777 strict、独立消费均复验，未回滚A/L与T-04无关结论。

## 未闭项

### F-815-T05-R1-01 / P2 — 强制停止被误标自然EOF，收尾错误仍未披露

**类型：原reader生命周期/错误传播不变量继续开放；AC-03/V-03，T-05。** 不再声称旧reader泄漏仍存在；新问题是“停止成功”被包装成“自然EOF”。

- 入口：`python docs/plans/attachments/815/review-t05-r3/held-reader.py`，与R2同语义的真实工具/安装等待点。仅安装器为受控OS子进程，将stdout句柄复制到本次探针私有、非tool Job的holder并正常退出；复制成功error0，holder持续持管道直到最终报告后的显式释放。fake仅此负控，不作真实UI成功证据。
- 应观察：reader真停且非自然EOF具名披露；不能杀非owned holder，不能把其存在伪称pipe EOF。
- 实测：约0.45s得到RunReport，唯一error仍是后续 `vue backend start failed: backend Cargo.toml missing...`，没有reader/EOF收尾错误。线程数初始5、终态6、holder释放后仍6，说明旧残活reader已停，但收尾失败被漏报。原始结果held-reader.json.txt/log.txt；这次“线程真停”是通过项。
- 已确认源码原因：`read_available`在stop=true时返回None（proc_host.rs:676）；`forward_pipe`将任意None设置reached_eof=true（:588-591）。forced stop/IO错误/自然EOF复用了同一个None，join_readers据此误认正常EOF；tool_ops合并错误的分支因未记录原因而无效。不是要求对正常取消加虚假错误，而是要求保留真实读取终态。
- 相邻静态分支：非Windows仍是阻塞read_until；join_readers截止后仍执行无界handle.join。本轮未在Linux实跑，不能把Windows通过外推这些分支；它们同属reader真停/具名有界收尾闭合范围。
- 关闭oracle：自然EOF成功、请求停止、非owned持管道、实际读错/panic分别具名且reader/writer确已结束；报告不能丢错误或把stop伪装EOF。用线程终态/OS管道持有者作为区分性观察，不能只断言reader Vec为空；重放Windows并核查声明支持的非Windows实现或取得明确的合同调整。

### F-815-T05-R3-01 / P1 — VM子进程丢失已求值的外部后端/模块上下文

**类型：执行隔离修复回归；AC-01/02/03、V-02/03，T-05。** 这是已服务VM路径替换引入的语义丢失，不是要求本轮提前完成所有T-06 profile。

- 输入/入口：`prepare-context.py`（重建真实SDK ABI后端），再 `vm-context.py`。真实backend cdylib使用本树auto-lang-core::vm::backend_abi::BackendRegistry，ABI版本1、注册入口真实调用reg.log；同工具链/target构建成功，source/清单在context-backend-source，binary hash与rustc版本在终核。没有用伪ABI/空壳响应替代实际后端装载。
- 同源前端：`use back.api: greeting`；widget .Init调用greeting写入title；backend/api.at的真实函数返回 `R3_EXTERNAL_CONTEXT`。负例用已支持request.external_backend指向外部backend；正控仅把相同api.at放本地src/back，前端源码逐字相同。两请求/源文件、实际状态/快照均冻结。
- 观察：外部例父进程明确记录 external backend linked、`R3 real backend ABI registered`、external backend loaded，证明有效输入已进入目标分支；子进程却报P-15 `back.api解析失败`，随后 `Undefined symbol: greeting in module App`，最终RunReport.failed、launch=None。正控实际呈现，标准MCP客户端state得到 `title: "R3_EXTERNAL_CONTEXT"`，rendered snapshot也包含该文本。
- 已确认原因：run_vm_flow在父进程装载/注册后端、set_external_back_root，host.rs:541-558却只把entry路径交给current_exe --internal-vm-exec；父进程SDK的EXTERNAL_BACK_ROOT/host_bridge等非env状态不会出现在新进程。子进程直接run_entry_file，无该request或backend装配。已实测模块映射丢失；主题/其他thread_local与注册状态是同因需核查的sibling，不冒称它们逐项都已测败。
- 关闭oracle：在执行进程用单源请求正确装配必要后端/模块/主题等上下文，保留backend对象寿命；无外部backend、本地/外部/拆分后端及代表请求声明/来源正反控保持。不能只复制某个测试文件、让子进程重读主pac制造第二真源，或把已支持外部路径改为拒绝以消红。具体跨进程载体/编排位置由work验证。
- 原始功能探针曾在model字段初始化直接调用函数，外部与本地都nil；该值oracle无效，model-init-*原件保留，不用作回归依据。最终使用仓库真实.Init语法后，本地标记成功、外部明确Undefined symbol才计finding。

### F-815-T05-R3-02 / P2 — 初始化期间已接收的Input q被启动快照忽略

**类型：新交互执行通道的启动边界回归；AC-03/V-03，T-05。** 不声称InputAck代表命令已执行，也不新增秒级SLA；问题是已写入并由内部watcher处理的退出命令会被启动状态设计永久忽略，窗口需要另一次取消/EOF才收口。

- 入口：`python .../input-initialization.py`，真实最小VM工程，在vm interpreter progress后重试直到真实InputAck（缺foreground的具名拒绝不计反例）。两次有效输入均InputAck.bytes=2，Ack时尚无Launched/first view，随后真实窗口呈现，却没有RunReport；探针观察2s后由EOF收口。原始frames/时序标记在input-initialization.json.txt。
- 正控：相同工程等first view后再发q，收到InputAck，约1s全程RunReport.completed/error=None，launch为真实MCP URL。取消/EOF原例在相同新架构均有效，因此不是缺runtime、坏frame或虚假的交互成功。
- 源码机制：内部bin先起stdin watcher，q调request_host_exit递增代数，再run_entry_file构建组件；renderer.rs:18950到run_session才快照当前代数。若q在此之前到达，该次请求已被纳入since，订阅只认>since，效果与旧清零丢请求相同。没有子进程插桩直接采generation值，具体线程交错由源码读写顺序与前述双对照推断；实测“有效输入后未退出/后来呈现/新退出请求可解除”已确定。
- 关闭oracle：输入允许前明确就绪/缓冲至本会话可消费，或把退出代绑定到操作真正开始的边界；已接受q在组件构建/会话启动中和已呈现后均有效，历史退出不误杀下一会话。不能只把测试睡久些等Launched，或让q必须重发才能生效；普通输入/取消/EOF与连续会话保持。
- 初版有一个案例未拿到InputAck，该case不计产品缺陷；initial-*保留。最终两个案例都到达有效Ack后仍失败，只有最终记录用于结论。

## 验证账与范围

- scoped nextest **74/74通过（1 LEAK）**，含19协议、20常规tool_process、5proc_host、30当前选择面的共享执行设施。LEAK仍是既有DirectChildOnly/非owned持管道负控，未说无泄漏全绿。
- 新真实VM交互/再启 **1/1**；独立consumer **5/5**；真Vue生成具名ignored单条 **1/1**；Man公开兼容/媒体/请求缝 **6/6**。不同Rust测试共**87项通过**，不覆盖以上失败反例，也不冒称整个包或整档全绿。
- 777 strict **0 violations/3 grandfathered**，配置/清单/锚输入未改变；原生cargo tree重放现有consumer闭包断言：零禁列、零必需项缺失。fmt全部净改动Rust及fixture通过，diff-check通过，新launched告警清偿。
- 原R1 replay.py all以及R2四脚本原例均独立运行，逐字段检查而非只看脚本exit0；新的holder EOF分类/外部context/初始化输入反例另行记账。
- 外部后端真实构建、同源api/前端、标准VM MCP state/snapshot是功能证据；无截图/像素或Vue双端视觉声明。本轮仅调用autoui-verifier已有客户端，无任意重写UI操作驱动。
- 脚本均在本目录，使用fresh私有target根，原始stderr在raw-process；内部VM窗口和helper均按本次进程所有权清理。context-scratch私有根可用prepare-context.py重建；不复制DLL入Git，保留源码/hash/命令。
- 未运行整档t/tv/tt/tb/tu/taa/tf/docs_gen；未修实施/改canonical或live ledger、合入、归档、清树。

## 当期AC与下一步

AC-01/02：局部fail（R3-01实际语义丢失），完整后续矩阵仍T-06/07；AC-03 fail（三项）；AC-04 partial（777/consumer/闭包通过，完整发行/792未验）；AC-05当期卫生pass、T-09完整终审pending；AC-06与A/L历史按原阶段合同保持。

原唯一树work一次修齐当前三项，补reader终态、跨进程上下文和初始化交互的永久区分性正反控，更新实际来源/规范草稿后提交新SHA复审。已关闭五项与更早关闭证据按未变边界保留，不把失败转成非阻断债务。T-05继续未勾、executing10/17；T-06..08/T-09及最终pass后的merge-owned冻结裸t+必要触面档/实际发布收口pending，未授权删能力/后移验收。
