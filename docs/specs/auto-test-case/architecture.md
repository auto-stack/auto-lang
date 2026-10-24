# auto-test-case 架构（裁判来源与依赖约束）

> 状态：active（A 阶段）。为什么这个 crate 存在、依赖方向为什么是单向。

## 裁判来源：pinned vendored parser

- `vendor/autodown-test-parser/` 三文件快照（markdown_parser/ial/block_model），
  `PROVENANCE.md` 记录来源 revision 与 sha256；指纹守卫测试防漂移。
  不手改、不在 build/run 再生、不依赖浮动主检出。
- 树只提供标题层级/anchor/fence 语义；**载荷字节永远来自原始文档 bytes**，由
  `raw_ranges` 定位并与树逐 fence 校验（计数/语言/EOL 归一内容三向一致，任何
  分歧=硬错误）。不取 serializer 载荷，不另造 Markdown case 语义。
- 报告 toolchain 身份包含 vendored parser 的编译期指纹（`include_str!` 哈希），
  旧报告跨快照漂移可检测。

## 依赖方向（避免 GUI/自举环）

```
crates/auto-test-case  ──仅依赖──▶  vendor/autodown-test-parser
crates/auto-lang（adapters/orchestration/CLI）──▶  auto-test-case
```

- 协议核心零 GUI/零 auto-lang 运行时依赖（`cargo tree --edges normal` 仅 vendored
  parser；auto-atom/auto-val 仅 dev-dep，供解码器测试构造 Atom 值）。
- 执行 adapter（AVM run / a2r emit）与 materializer/编排/`auto-case` CLI 在
  auto-lang 侧 `src/test_cases/`，空 feature `test-cases` 门控、无默认 feature
  可无窗口构建运行。被测 a2r 不得在运行时再生裁判（parser 快照固定）。

## 阶段观测通道（类型边界，非文本）

- AVM：`run_with_capture_and_path_staged`——旧 API 的 staged 孪生，engine 陷阱点
  （typed `VMError` 仍存活处）记录变体名经 thread-local 中继出线程；旧 API 签名
  与行为原样。
- 单文件 a2r 发射：parse/CTEE 错误保持类型化传播（Lexer/Syntax/Type/…）。
- 分类规则（`classify_staged`）：类型化 trap → run；Lexer/Syntax → parse；
  Type/Name/Comptime/MsgWithSource → type；MultipleErrors 全递归取首个叶子；
  其余（Io/panic/合成超时/无名 Msg）= Infrastructure → ERROR，**永不满足语言
  负例**。
- 已知边界（F-773-R2-01 债务）：a2r project 模式两入口仍把 parse 错误包成无名
  Msg → 归 Infrastructure（fail-safe：永不假 PASS，但 project-mode 强错误
  oracle 不可满足）。

## B 阶段执行拓扑（PLAN-788：native worker 与全量父侧判定）

- 单管线复用：auto-lang `test_cases::ac` 编排，`experimental/ac-case-worker`
  独立 workspace 持有实验编译栈（ac-core 前端/verify/lower、783 store、787
  runtime——无第二执行器）；**auto-lang 零静态依赖实验编译器/Cranelift**
  （红线），经 `acworker/1` JSON-lines 有界会话 IPC 子进程调用（deny_unknown_
  fields、nonce/request_hash 回显+父侧比对、行限预分配）。
- **PASS 判定永在父侧**：typed admission（`validate_response`：schema/身份/
  形状/解码后流 cap）、manifest 分配前全校验+持久化（`admit_and_persist_
  artifacts`）、流比较（`compare_stream_field`）为可复用公共入口，周转 harness
  与 runner 使用同一入口；worker PASS 不受信。
- 父侧 I/O 生命周期统一：stdin 写入线程化并纳入请求墙钟；stderr 排空超限即
  时杀树（与响应/截止同 ≤10ms 轮询循环）；stdout 读取带界；产品消费 run 经
  同一 contained 执行器（不重编译、不裸 spawn）。
- 身份/stale 见 [profile-v2-native.md](profile-v2-native.md)（逐行适用集合）。

## 执行限制与试点边界

- 可信有限 pilot：无外部进程/网络/FFI/自举重型执行；未实现的 budget/fixture/env/
  resource 声明 = unsupported ERROR；超限 = resource ERROR（不伪装语法错误）。
- scratch 纪律：每 run/case/check 唯一 scratch 子目录；不改全局 cwd/env；不写
  旧目录 `.wrong.*`。
- 测试档位接线：`cargo t`（test_cases_tests 家族）/`cargo tv`
  （`vm_file_tests::pilot_aggregate_vm`）/`cargo tt`（`a2r_tests::pilot_aggregate_a2r`）。

> 来源：docs/plans/773-autodown-testcase-protocol-pilot.md §2/§5（Design 35 A）。

## probe 通道（PLAN-796：sema/query-v1 受限 adapter）

- 契约层：`Adapter::Probe × Run`（v2-only）+ 注册配方表（首个
  `sema/query-v1`；797 按同合同追加 ABI recipe，不改协议）+ 声明式步骤
  （query / edit-source / restart）+ atom-data 源类别。
- 执行层：语义 worker 子进程（`semaworker/1`，与 acworker 同形信封/回显/
  有界 IO/父侧墙钟）；restart = 统一 runner 内真实跨进程持久化；判定全父侧。
- worker 全生命周期：父侧 Job containment（suspended→assign→resume，子孙
  收割）、响应后仍待退出、stderr 越限即杀、4MiB 元数据上限；退出状态/版本
  镜像精确匹配；工具身份四轴（protocol/recipe/exe-sha256/worker-version）
  入报告与 stale 判定（ac/probe 分族）。
- 语言负例：parse 失败以 typed `diags`（stage=parse）返回并按声明 error
  completion 判定，不归 infra ERROR。
- case 永不携带 shell/任意 exe；worker 工具身份仅可信宿主配置绑定
  （SEMA_CASE_WORKER）。


## PLAN-798：已实现的AV消费边界

probe/run新增注册recipe ac-av/core-run-v1，av绑定entry/caps/可选session及run/edit-source步骤；case不能命名任意exe，跨recipe字段/步骤拒绝。保持sema/query-v1、abi/interop-v1和原v1/ac/syntax支持；ABI拒绝AV专用Run，混合报告身份按三recipe独立惰性解析，行顺序不影响结果，篡改某worker只使该recipe stale。AV worker avworker/1协议、avworker/1.0/av.core-i32/1镜像和nonce/request/source/产物身份由父侧准入后判定。worker准备/运行受限启动、Job杀树、双流cap和单截止，超时/截断/错actual为infra/失败，不把expected发给worker。技术计时和驻留增量的边界见[AV缓存验收](../auto-av/cache-and-conformance.md)。

PLAN-828 正式持续会话入口：`av_probe_worker session-serve`（一进程一会话：init/ready
恒定身份+初始派发身份基线、编辑流发布已验证当前值+分相计时+计数快照+派发载荷身份
（对象地址+代际）+worker 线内存峰值、close），父侧 judge 拥有测量窗（编辑入队→
父侧**全部校验完成后**为终点，IPC 与逐请求校验在窗内）与独立期望；每发布绑定期望
修订（revision==req_id）与确切安装目标（`expected_install`），未改函数派发身份跨修订
稳定；双门=P95<gate ∧ 全对 ∧ 结构（未改零阶段/零整模块/恰一确切安装 ∧ 修订/目标/身份
门）∧ 完整性，gate=0 必红；worker 中途死→缺样本红完整性，不静默缩战役；初始化排除
单列（init_ms）、ready 先判；完整观测（逐样本计数差分/诊断/身份/worker 相位与
输入/二进制身份）经 `JudgeReport::write_evidence` 落盘留存（成功与失败同留）；
`frontend-attribution` 子命令为双线预算 A 线隔离测量口。one-shot run/recipe 保留不变。

## PLAN-817：cte/scalar-query-v1 recipe（旧 recipe 零修改）

正式 recipe 表追加 `cte/scalar-query-v1`（v2 probe/run、atom-data 源、
steps=query|edit-source|restart 以 query 结尾、cte 绑定 entry/caps/session）；
worker 身份轴族 `cte-worker`（stale 第二登记点——配方分支外独立轴，混合 cte/av
行序独立+漂移即红）；outcome admission：judged 步骤上的隔离 resource/cancel/infra
态=ERROR 行（永非语言裁决）、语言类 diag 对照声明的 error completion（cte.*→
parse/verify/capability/run stage 投影+message matcher）、期望 success 默认 FAIL；
未知字段/recipe 拒绝；未注册 recipe lint 拒（列注册集）；坏 worker 工具身份=ERROR
不绿。runner/适配器见 `crates/auto-lang-core/src/test_cases/ac_cte.rs` 与语料
`crates/auto-lang/test/cases/ac-cte/`。
