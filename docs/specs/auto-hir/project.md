# auto-hir

> **Status**: experimental
> 路径：`experimental/ac-core`（独立 Cargo workspace，package `auto-ac-prototype`） | 技术栈：Rust（cranelift 0.126 栈消费方见 auto-ac）

core-i32 profile 的类型化 HIR 契约与实现：Schema-bound Atom 文本 → 绑定 → 语义校验 → `CheckedModule`。
由 PLAN-741 建立首个闭环；完整 HIR（全 A 子集、来源表、跨模块、泛型/所有权）不在现状内。

## 目标与范围

- **只接受** `auto.hir.core.draft` / revision 1 / `core-i32-draft` 三重身份（嵌入 descriptor `experimental/ac-core/schema/core-i32.atom`）；未知 tag/case/字段/分支违约一律 bind 阶段拒绝。
- 类型仅精确 `i32`/`bool`；运算 `add_i32`/`mul_i32`/`lt_i32`；加/乘必须声明 `overflow: trap`。
- 单模块内解析引用；跨模块引用、字符串/聚合/所有权/泛型/enum 值不在本 profile。

## 文本契约（atom_text + descriptor）

- 节点形态：`tag primary... (命名字段) { 正文字段/子节点 }`。primary 槽可用等价命名字段改写
  （`module m` ≡ `module(id: "m")`；expr 的 kind、local 的 type 同理）；同一槽双形态并写拒绝；
  同名字段跨 head/body 重复拒绝。字段位置互换不改变语义（表登记顺序仍决定 ID 分配）。
- 引用按**类别 + 作用域**绑定：类型/声明/函数体属模块表，local/place/expr/block/loop 属函数体表。
  文本 ID 只是本表局部标签；类别失配（如 rhs 写 local id）与悬空引用分别报 `bind.category-mismatch` /
  `bind.dangling-ref`。
- TypeUse：裸词 `i32`/`bool` = 内建简写；其余标识符/字符串解析到模块类型表。两种形态在**校验层**
  归一（内建引用≡内建简写），名义类型不因底层同为 i32 折叠。
- 分支契约由 descriptor 的 `branches` 声明（require/forbid）：如 `local.param` 必带 index 禁
  mutable、`expr.binary:add_i32` 必带 overflow、`expr.binary:lt_i32` 禁 overflow。
- 文件中的任何 `checked` 类标记**无授权作用**——descriptor 未知字段直接拒绝。

## 语义契约（verify → CheckedModule）

- `CheckedModule` 只能由 `verify::verify` 构造；后端不得消费其它来源。
- 类型：运算数/结果、调用实参/形参/结果、`return` 对函数签名、`if` 条件须 bool、param local 对签名
  逐位一致（index 界内唯一）。
- 归属唯一：每个函数的 `body` 字段必须指向 owner 恰为该函数的 body（共享 body、同签名共享、
  冒领 body 均在 verify 阶段报 `verify.owner-mismatch` 拒绝）；body→owner 反向一致性同此。
- 初始化：`let` 初始化；`assign` 目标须已初始化且 mutable（place 与 local 双查）；if 出口初始化 =
  可达分支交集；循环按可能零次迭代保守处理（体内初始化不外泄）。
- 块结构：非入口块恰有一个入边（唯一入口角色，禁共享块）；入口块零入边；入口须在所有路径 return；
  `return`/`break`/`continue` 之后不得有可执行尾语句；块自然收尾=交还宿主结构（if 汇合/循环重入）。
- 求值位置：每个 ExprId 恰有一个 owning 引用（语句位或操作数位；禁共享、禁死表达式）且表达式图无环；
  递归走 DefRef 不复制 body。`eval_args` 左→右求值存临时，`bindings` 仅做形参映射且必须为双射：
  param 全覆盖唯一、arg 越界/重复消费/未被任何 binding 消费均拒绝；实参类型按映射逐位核对
  （`params[b.param]` ≡ `eval_args[b.arg]`），不按求值下标对位。
- 诊断：阶段化（text/bind/verify/capability/backend/link/run），`file:line:col: error[stage/code]`
  渲染，span 指向冒犯 token。

## 边界与已知限制

- 不宣称完整 HIR/Auto 语言支持；`experimental/ac-core` 与 v0.5 parser/VM/A2X、auto_val::Value 无关。
- `loops`/`LoopId` 为函数体扁平表，嵌套合法性由 verify 的词法栈检查。
- 文档样例源：`docs/design/strategy/hir-examples/*.atom`（01-add / 01-add.explicit / 02-control /
  03-call-order）+ `experimental/ac-core/fixtures/{valid,invalid}/`（拒绝矩阵物化）。

## 阶段契约

公共 HIR 的阶段边界、pass 合同模板与非法变换负面清单见
[stage-contract.md](stage-contract.md)（planned；实现状态以本文件现状节为准）。

## 相关 plan

见 [plans.md](plans.md)。
