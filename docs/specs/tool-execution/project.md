# tool-execution — 统一 owned 子进程执行设施与 I-TOOL 客户端

> 状态：PLAN-816 整 Plan 交付（M816-A 2026-10-21 发布；M816-B 2026-10-24 整体
> R10 代码验收通过，绑定 `6d15d25a4`——分散站点迁移、daemon/终止原语与全部
> 真停/树语义实测闭合）。M816-A 子集事实保留于下文各节；B 面新增合同见
> §寿命与真停合同的 B 增补与 §daemon/终止原语。

## 执行设施（SD-01；AC-02，PLAN-816 T-02/T-06）

`crates/auto-tool-execution`（仅 std 零三方依赖，777 域 man-ecosystem）是子进程
执行的唯一设施：argv 数组（零 shell 解释）、显式 cwd/env、每流 stdio 策略
（继承/管道/null）、退出具名分类（Success/Failure/Timeout/Cancelled/Terminated/
SpawnError）、请求级 deadline、cancel 与 owned 收尸、捕获 reader 真终止、子进程
树退出（Windows Job Object / Unix 进程组）。M816-B 已完成分散站点迁移
（main.rs run_command_live、pkg.rs run_script_live、git.rs（git_capture：显式
cwd/DirectChildOnly/退出分类）、http_service.rs serve_rust 轨）——不建第二套
并存 runner。

### 寿命与真停合同

- **deadline 覆盖整次执行**：stdin 写入（经 writer 线程分片）、根等待与资源收尾
  共用同一绝对截止；对端不读导致的管道写阻塞不越过截止。
- **任何返回路径都不带活的自有线程**：超时/关闭先请求停止、终止 owned 树、确认
  reader/writer 线程真停（轮询切片 + join 收割）之后才返回；错误返回路径统一
  先收尾后传播。写端准入（取消凭证创建/复制）先于任何自有线程启动——失败时
  具名拒绝（`create/clone stdin cancel credential`），无 reader/writer 需要清理，
  宿主进程由 Drop 有界终止回收。
- **kill 发出 ≠ 已停**：取消/超时后观察到真停（收尸 + join）才返回；根退出后仍
  持管道的 owned 后代先终止，reader 有界收尾；管道被非本次拥有的进程持有时具名
  `PipesHeld`，不以成功码或空输出掩盖。
- **取消语义（Windows）**：同步匿名管道的阻塞写只能以文件对象级 `CancelIoEx`
  取消（`CancelSynchronousIo` 对管道无效）；停止侧持有取消凭证（DuplicateHandle
  副本）并在等待线程结束期间每切片重发——任意时刻进入内核的写都会被随后的重发
  中止；写线程正常完成即关闭凭证副本（EOF 不被延迟）。凭证创建/复制失败具名
  拒绝，不静默降级。Unix 由 `O_NONBLOCK` + poll 轮询承担同语义。
- **树归属（Windows）**：树范围子进程以 `CREATE_SUSPENDED` 启动，挂起中完成
  Job 归属再恢复执行——后代不可能在归属建立前逃出；失败路径对挂起根终止收尸。
- **树终止（Unix，M816-B R10 定稿）**：`signal_group`/`kill_group` 必须传**负
  PGID**（killpg 语义）——正值只命中同号 PID（根），owned 后代收不到（R9-01
  已修缺陷）。进程组在任一成员存活期间持续存在（根退出/已收尸不撤销 PGID），
  因此"根已退出但 owned 后代仍在"时组信号仍然送达；ESRCH=无此进程组视为收尾
  成功。`interrupt()` Unix=向 owned 组发 SIGINT（ProcessTree 子进程经 pre_exec
  setpgid 移出父前台组——终端 Ctrl+C 不再直达，父进程收到终止意图须显式转发，
  子进程才能汇入自身优雅关闭）；Windows 控制台事件本就独立送达同控制台子进程，
  成功 no-op。组内送达由受控 handler 永久回归钉住（te-child signal-tree：
  双端安装就绪握手后 interrupt，后代干净退 42 并由存活根传播）。
- **reader 终态具名**：捕获 reader 的读错（ReaderIo）与线程 panic（ReaderPanicked）
  具名传播；停止排空按真实终态映射（EOF/读错/被停止），不把流失败抹成正常停止。
- **只终止本次拥有的进程（树）**；DirectChildOnly 只终止直接子进程，其后代按
  自身寿命退出（正反控在案）；legacy run 链的端口清场不在设施内，亦不扩散到
  新工具/fixture 或 `auto service`（service 端口冲突非零、无假 ready、不 kill
  占用者保持）。

### daemon 分离与终止原语（SD-01 B 增补；PLAN-816 T-06）

- `spawn_detached`：daemon 分离 spawn——无归属/不收尸；`windows_creation_flags`
  透传（CREATE_NO_WINDOW 等创建语义）；非 Windows 平台非零 flags=具名
  `InvalidSpec`。`auto serve` 后台自身 spawn 消费（daemon 协议不动）。
- `terminate` 原语：`install()` 安装 OS 处理器（Windows 控制台 ctrl 事件 /
  Unix SIGINT+SIGTERM）把"操作员终止"落到进程级标志，`requested()` 观察；
  `CREATE_NEW_PROCESS_GROUP` 子进程默认禁 Ctrl+C——install 先 NULL/FALSE 重启
  用再挂处理器。与 VM tokio ctrl_c 互斥（service -s vm 不安装，VM serve_async
  自持优雅关闭）。watch 以终止标志在唤醒粒度内干净退出（watcher 常规 Drop
  回收，缓存已存盘）。

## I-TOOL 客户端（SD-01；AC-07，PLAN-816 T-12）

`crates/auto-tool-client` 经设施 spawn 真实工具宿主进程，以 auto-tool-protocol
单源帧/信封通信：`describe` / `validate_project` 窄能力、宿主拒绝（版本/能力/
字段/预算）以 `Reply::Host` 数据透传、客户端自身失败具名分类（ClientError）。
闭包 = data（protocol/project-model）+ man-ecosystem（execution），**无工具实现
静态链接**（独立 consumer `test/fixtures/modularity/tool-client-consumer/
verify-deps.sh` 钉住）。控制流（stdout）仅帧且先排空已缓冲内容（已到达完整帧与
坏前缀不等待新字节）；EOF 夹半帧保留 `TruncatedEof` 证据，由关闭语境定性（自然
退出=控制流违规；我方终止截断在途回复=拆除噪声）。请求 deadline 覆盖整次往返
（帧写入+回答等待）；优雅关闭（EOF）期待退出码 0，宽限到期则 cancel 收尸。

M816-B 发现序（T-04）：env `AUTO_UI_TOOL_BIN`（指定即必须存在）→ workspace
target/{debug,release} → 邻接安装；经真实 describe 核验 tool_name/
protocol_version/required_operations；缺件（EnvPathMissing/NotFound）/版本错
（VersionMismatch）/缺能力（CapabilityMissing）具名拒绝，无隐式 UI fallback；
合法 describe 后的收尾失败（坏帧/非零退出）= `TeardownFailed` 具名携带原因，
不返回成功发现、不回落其他候选。

## 测试分层

crate 内联测试纯数据面 + 故障注入面（假 Read 驱动，无 sleep）；真实进程正反控
在独立 fixture（driver 先构建 auto-ui-tool/te-child 再以 env 钉定产物）；寿命与
取消反例用自带截止的具名测试子进程（te-child：sleep/grandchild/orphan/
stdin-holder/probe/proto-close/terminate-wait(可选 ready-marker)/stdout-holdout
（打印 marker→Unix SCM_RIGHTS 经 unix socket / Windows DuplicateHandle 转交写端
→存活 N 秒）/fd-receiver(Unix，握手 TE_CHILD_HANDSHAKE_SECS 截止自退)/
signal-tree（组信号送达证明对端）等模式；te-child 定位 env `AUTO_TE_CHILD_BIN`
覆盖优先，两路径均存在性断言）。真停/超时/树终止/握手自退/组送达由
exec_lifecycle 与 plan816_cli_driver 永久回归锁定；审查期持久回放（probe/replay
脚本）冻结于 `docs/plans/attachments/816/review-r*/`；代码/依赖/配置/输入变化
时受影响证据须重验。

## PLAN-832 client 仓属（SD-03，as-built）

`auto-tool-execution` 继续作为语言仓 std-only 中立设施，UI 与 Man 双消费，不迁成 Man 独占。`auto-tool-client`（含 discovery）已迁 auto-man 仓；真实工具宿主、协议和 owned 寿命合同不变。`plan816_cli_driver` 与 tool-client-consumer 从 Man 固定 source 重放，UI tool 不静态链接。

## PLAN-896：既有Command捕获与真实编译消费者

`run_command_capture(Command, TreeScope, Option<Duration>)`直接消费原Command并进入同一个spawn/run/reader/owned收尾实现；不重建环境，因此env_clear、显式env覆盖、argv与cwd保留。该output式入口将stdin设为null、stdout/stderr设为捕获；ProcessTree使用设施的Windows挂起/入Job/恢复创建策略，不承诺保留调用方自定Windows创建标志。没有隐式缩短deadline，None仍由原调用方预算约束。

语言FFI的auto-cache编译与rustdoc、ACE A2X测试中的真实rustc/程序、资源门的Cargo构建及artifact兼容包装复用此原语；不新增一套执行器，不引入ACE/UI/Man runtime依赖到语言侧。auto-cache的实际构建/rustdoc显式绑定自己的build_dir/target，operator的外层target不泄入该私有编译面。

artifact包装保留原RunResult code/timed_out/partial stdout/stderr；完整截止含清理，reader/containment/清理错误仍为Err。原先忽略错误/可选Job/返回后读者残留的紧凑执行器退出。字符串协议拒绝无效UTF8，不把损坏字节替换成可接受JSON。外层测试Job对工作负载残留仍严格失败，MSVC辅助进程仅在真实编译producer持有的owned树内回收，未增工作负载豁免。
