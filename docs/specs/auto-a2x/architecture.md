# auto-a2x 架构（视图/准备/目标契约与依赖方向）

> 状态：active（AX-A 形态）。ADR 级约定，源自 PLAN-892（Design 51 §3）。

## 数据流（AX-A 交付形态）

```
.at 源 + 显式 source profile
  → ac-core 前端（compile_source / compile_scalar_source / 暖路径 FnCandidate）
  → SemanticView（双域准入 + 声明事实 + 逐 body 防御走查）
  → FunctionPlan / 语义指纹（target 中立；span/名字/表序排除）
  → 目标 provider（a2x-rust：lower → print → 四文件集 + manifest）
  → 父侧判决（golden 逐字节 / 显式 certify 真编译运行）
```

## 关键裁定（ADR）

1. **不复制前端/执行器**：共享 core 只消费已验证 HIR 与凭证；不通过运行 AV
   取得转译产物；`emit_function_body` 只发 env 当前语义，编辑体在候选 parts
   （828 合同）。单函数片段视图按 `verify_function_ref` 凭证准入
   （`SemanticView::from_parts`），其余 def 为签名-only ExternalFn。
2. **身份按源名**：Rust 函数名 `fn_<源名>`——def 表索引跨整模块/单函数 bundle
   漂移（F-828-R1），索引不可作跨 bundle 身份；模块内源名唯一。
3. **求值顺序是事实不是配置**：`Call.eval_args` 为源序（各求值一次），
   `bindings` 为落位图；目标按源序钉临时后按 binding 落位，不得反推。
   短路 `Logical` 降为惰性 if 表达式（RHS 临时在分支内）。
4. **具名拒绝优先**：一切准入失败是 `A2xReject`（`a2x.*` 稳定码），与
   resource/infra 分列；防御深度预算具名 `a2x.too-deep`（400）。
5. **span 是可观测非身份**：source map v1（def origin → 生成注释 + manifest
   functions[]）；span/空白/表序不入语义指纹。

## 依赖方向（机械断言：boundaries 测试）

```
a2x-core   → auto-ac-prototype（HIR/verify/source 只读）
a2x-rust   → a2x-core
a2x-store  → a2x-core, a2x-rust, ac-artifact-probe（783 store 只读复用）
a2x-probe  → 以上全部（+windows-sys 仅 cfg(windows) 内存计数、serde_json 协议）
根 workspace ⇏ 任何 a2x crate（实验栈不入产品 workspace；消费经预构建 worker 进程）
```
