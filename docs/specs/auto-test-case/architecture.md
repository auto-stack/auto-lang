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
