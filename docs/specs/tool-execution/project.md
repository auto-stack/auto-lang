# tool-execution — 统一 owned 子进程执行设施与 I-TOOL 客户端（M816-A 阶段子集）

> 状态：PLAN-816 M816-A 已发布子集（2026-10-21，T-13 R10 阶段审查通过，绑定
> `255146777`；并集冻结 `b522aad4a` 冻结门通过后落地）。本文件只陈述已发布的
> 执行设施/客户端子集；薄 CLI 路由、管理服务分层、工具发现/bootstrap/安装
> profile 属于 M816-B，不在本子集中。

## 执行设施（SD-01-A；AC-02，PLAN-816 T-02）

`crates/auto-tool-execution`（仅 std 零三方依赖，777 域 man-ecosystem）是子进程
执行的唯一设施：argv 数组（零 shell 解释）、显式 cwd/env、每流 stdio 策略
（继承/管道/null）、退出具名分类（Success/Failure/Timeout/Cancelled/Terminated/
SpawnError）、请求级 deadline、cancel 与 owned 收尸、捕获 reader 真终止、子进程
树退出（Windows Job Object / Unix 进程组）。分散的启动/等待语义
（main.rs run_command_live、pkg.rs run_script_live、git.rs、http_service.rs
serve_rust）在 B 面任务迁移消费本设施，不建第二套并存 runner。

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
- **reader 终态具名**：捕获 reader 的读错（ReaderIo）与线程 panic（ReaderPanicked）
  具名传播；停止排空按真实终态映射（EOF/读错/被停止），不把流失败抹成正常停止。
- **只终止本次拥有的进程（树）**；legacy run 链的端口清场不在设施内，亦不扩散到
  新工具/fixture 或 `auto service`（service 端口冲突非零、无假 ready、不 kill
  占用者保持）。

## I-TOOL 客户端（SD-01-A；AC-07，PLAN-816 T-12）

`crates/auto-tool-client` 经设施 spawn 真实工具宿主进程，以 auto-tool-protocol
单源帧/信封通信：`describe` / `validate_project` 窄能力、宿主拒绝（版本/能力/
字段/预算）以 `Reply::Host` 数据透传、客户端自身失败具名分类（ClientError）。
闭包 = data（protocol/project-model）+ man-ecosystem（execution），**无工具实现
静态链接**（独立 consumer `test/fixtures/modularity/tool-client-consumer/
verify-deps.sh` 钉住）。控制流（stdout）仅帧且先排空已缓冲内容（已到达完整帧与
坏前缀不等待新字节）；EOF 夹半帧保留 `TruncatedEof` 证据，由关闭语境定性（自然
退出=控制流违规；我方终止截断在途回复=拆除噪声）。请求 deadline 覆盖整次往返
（帧写入+回答等待）；优雅关闭（EOF）期待退出码 0，宽限到期则 cancel 收尸。工具
发现/discovery 归 M816-B（客户端只接受显式二进制路径，无隐式搜索）。

## 测试分层

crate 内联测试纯数据面 + 故障注入面（假 Read 驱动，无 sleep）；真实进程正反控
在独立 fixture（driver 先构建 auto-ui-tool/te-child 再以 env 钉定产物）；寿命与
取消反例用自带截止的具名测试子进程（te-child：sleep/grandchild/orphan/
stdin-holder/probe/proto-close 等模式）。审查期持久回放（probe/replay 脚本）冻结
于 `docs/plans/attachments/816/review-r*/`；代码/依赖/配置/输入变化时受影响证据
须重验。
