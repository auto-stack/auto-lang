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

## 执行限制与试点边界

- 可信有限 pilot：无外部进程/网络/FFI/自举重型执行；未实现的 budget/fixture/env/
  resource 声明 = unsupported ERROR；超限 = resource ERROR（不伪装语法错误）。
- scratch 纪律：每 run/case/check 唯一 scratch 子目录；不改全局 cwd/env；不写
  旧目录 `.wrong.*`。
- 测试档位接线：`cargo t`（test_cases_tests 家族）/`cargo tv`
  （`vm_file_tests::pilot_aggregate_vm`）/`cargo tt`（`a2r_tests::pilot_aggregate_a2r`）。

> 来源：docs/plans/773-autodown-testcase-protocol-pilot.md §2/§5（Design 35 A）。
