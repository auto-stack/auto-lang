# auto-a2x 架构（视图/准备/目标契约与依赖方向）

> 状态：active（AX-A＋AX-B 形态）。ADR 级约定，源自 PLAN-892（Design 51
> §3）与 PLAN-902（Design 51 §5.3）。

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

1. **不复制前端/执行器/ownership 分析**：共享 core 只消费已验证 HIR 与
   凭证；不通过运行 AV 取得转译产物；`emit_function_body` 只发 env 当前
   语义，编辑体在候选 parts（828 合同）。单函数片段视图按凭证准入
   （`SemanticView::from_parts`），其余 def 为签名-only ExternalFn（AX-B
   起 ExternalFn 携带验证 param modes）；**text/text-record 单体凭证=
   `verify_text_function`（携自身 BodyResourcePlan，排他种子来自保留环境），
   record 域沿 `verify_function_ref`（方法体 receiver=params[0] 同源）**。
   资源/模式事实出口见 [checked-resource-facts](checked-resource-facts.md)；
   凭证按声明上下文修订指纹绑定（`a2x.credential`）。
2. **身份按源名（名义）**：自由函数 Rust 名 `fn_<源名>`、方法 owner 限定
   `mt_<Owner>_<名>`——def 表索引跨整模块/单函数 bundle 漂移（F-828-R1），
   索引不可作跨 bundle 身份；模块内源名唯一；同布局异 owner record 是不同
   名义类型（上下文指纹与语义指纹均按名义+形状混入）。
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

## Collections 域与 AX-C（PLAN-904）

- `SemanticView` 准入扩至 collections 域（`Ty::List(u32)`/`Ty::Variant(u32)`/
  `Ty::Unit`；名义 variant 表/臂序=disc；受限 Str 面+record 构造/字段读+List/Variant
  节点集）。语义指纹增量 gated（902/892 字节不动）：list/variant op 标签、名义 digest
  闭包（变体臂+载荷传递图+list 元素）、`eval` 语句。
- 声明 required 闭包：候选 fragment 的 manifest/文件集绑定全部必需类型与片段；类型/
  元素/臂序/mode/被调事实变化失效，等价表序稳定。
- 配置分层失效：语义层 target 中立；目标 runtime/printer 变化→目标片段/manifest 失效；
  工具/认证配置变化→证书 STALE（execution_config 为证书键材）。

## PLAN-921：目标 provider 接缝（session 去目标耦合）

- `a2x-store` 不再 import 任何 `ace-a2x-rust`/`ace-a2x-c` 项：fragment
  key 目标尾字段、片段生产（miss 批量、miss_slots 保 loop 位序=冻结字节
  序）、runtime 文本选择、prelude seed（RT/COLL 记录表无损行式编解码）与
  fileset 装配全部经 `Box<dyn TargetProvider>` 委托。
- Rust 侧迁移是**字节同一**的（AXR1..3 键字段序 lang/edition/backend/
  backend_version/runtime_sha256/print_version 逐字保留；wrapper/manifest
  格式串原样迁移；RT/COLL prelude 经 seed 往返）；既有 Rust 全套
  （store_cache/incremental/902/904）是该迁移的字节同一 oracle。
- 语义层（AXS1..3）键不含目标字段；AXS1 consumers 注册增列
  `auto-a2x-c`（registry 元数据，键/字节不动）。
- 不支持族是命名拒绝：cold 入口 `ensure_target_supports`（C 仅
  Family::Scalar；RecordText/Collections 经 C 目标为 `a2x.domain`）。