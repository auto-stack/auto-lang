# auto-hir

> PLAN-896当前组织：上述ACE相关实现统一属于`experimental/ace/Cargo.toml`独立workspace（单Cargo.lock/同树target），各子目录为成员包，不再是独立workspace。Checked HIR仍由experimental/ace/crates/core的ace-core提供；不因目录调整改变IR及验证边界。普通语言cargo t不构建ACE；cargo t ace选择常规集合，完整包/all及ace-performance保原专项和适用条件。

> **Status**: experimental
> 路径：`experimental/ace/crates/core`（ACE workspace成员，package `ace-core`） | 技术栈：Rust（cranelift 0.126 栈消费方见 auto-ac）

core-i32 / scalar-i32 双 profile 的类型化 HIR 契约与实现：Schema-bound Atom 文本 →
绑定 → 语义校验 → `CheckedModule`。由 PLAN-741 建立首个闭环，PLAN-772 增设
scalar 身份域与运算扩面；完整 HIR（全 A 子集、来源表、跨模块、泛型/所有权）
不在现状内。

## 目标与范围

- **已注册双身份域**（精确可失败映射，未注册三元组不归类任何域）：
  - `auto.hir.core.draft` / revision 1 / `core-i32-draft`（descriptor
    `experimental/ace/crates/core/schema/core-i32.atom`）；未知 tag/case/字段/分支违约
    一律 bind 阶段拒绝；
  - `auto.hir.scalar.draft` / revision 1 / `scalar-i32-draft`（descriptor
    `schema/scalar-i32.atom`，core 的超集）。
- 类型仅精确 `i32`/`bool`。运算面：core 域 `add_i32`/`mul_i32`/`lt_i32`（加/乘
  必须声明 `overflow: trap`）；scalar 域另有 `sub/div/rem_i32`（div/rem 无
  overflow 字段——trap 固有：除数 0、MIN/-1、MIN%-1）、全序
  `le/gt/ge_i32`、同类 `eq/ne`（i32×i32 或 bool×bool → bool）、一元
  `neg_i32/not_bool` 与 **lazy** `Logical{and,or}`（与 eager `Binary` 分离表示；
  禁止映射为双参 intrinsic 或无条件发射 RHS）。程序化混入跨域运算/节点由
  verify 拒绝（`verify.profile-op`），扩大的共享 enum 不扩大旧 profile。
- `hir::Bundle` 携带 `HirDomain`（由 descriptor 三重身份精确导出）；verify 入口
  按域分设：`verify` 限 core、`verify_scalar` 限 scalar（域错位 `verify.domain`）；
  未注册 descriptor 的 bind 整体拒绝（`bind.descriptor-unregistered`），
  canonical writer 对未注册/错域一律拒绝（`canonical.identity-conflict`）。
- 单模块内解析引用；跨模块引用、字符串/泛型/enum 值不在任何已注册域内。
- **record 域（PLAN-771）**：`auto.hir.record.draft / 1 / record-i32-draft`
  （descriptor `schema/record.atom`，scalar 的超集）增设名义 record 类型
  （`TypeKind::Record` 声明序字段表；结构相同不同名不互换）、字段 place
  （`PlaceKind::FieldPath` 逐段归属）、构造表达式 `record_ctor`（源码序
  求值表 + 按名/按位 ctor_bindings 双射）、借位表达式 `borrow_view`/
  `borrow_mut`（仅直接调用实参位）与方法声明（`DefKind::Method` +
  ReceiverKind static/view，param `mode` view/mut）。可信门：类型包含图
  迭代环检测（`verify.record-cycle`）、名义 ValueType::Record(tid) 相等、
  构造完整性（`verify.ctor-arity`）、逐段字段 owner（`verify.field-owner`）、
  借位位置/模式与 per-call mut 排他（`verify.mut-conflict`，774 D-04）；
  全部表引用（含 binding 内索引）先界检后走查，变异 Bundle 受控拒绝不
  panic。旧 core/scalar profiles 的接受/拒绝集合零改动（`verify.profile-op`
  域防御双向生效）；verify 入口 `verify_record` 限本域。合同细则见
  [../auto-ac/source-record.md](../auto-ac/source-record.md)。
- **text 域（PLAN-770）**：`auto.hir.text.draft / 1 / text-i32-draft`
  （descriptor `schema/text.atom`，scalar 的超集）增设 str/Str/char 类型
  （`BuiltinType::StrSlice/StrOwned/Char`）与 12 个文本表达式节点
  （str_lit/char_lit/text_len/text_len_bytes/text_char_at/text_slice/
  text_concat/text_clone/text_to_owned/text_to_borrowed/char_to_codepoint/
  char_from_codepoint）、`Stmt::Drop`、`CallBinding.moved` 与参数所有权模式
  `ParamMode`（View/MutBorrow/MoveOwn——771 record 的 `mut` 即 MutBorrow，
  实现树统一枚举）。可信门：verify_text 分域入口 + 文本资源前门
  TextResourcePlan（owned 块词法作用域/借用 provenance 与构造代际/
  `.move` 转移与双移除/清理责任 incoming 三态/参数对排他 fixpoint 传播
  /派生 runtime 需求含清理义务）；各域 profile 防线互拒（`move` 仅
  text、`mut` 仅 text+record）。合同细则见
  [../auto-ac/source-text.md](../auto-ac/source-text.md)。

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

- `CheckedModule` 只能由 `verify`（core 域）/`verify_scalar`（scalar 域）构造；
  后端不得消费其它来源；Checked 记录所属域。
- 引用边界门先于索引走查：body 内 local/place/expr/block/loop 引用与模块级
  函数签名 params/result 的 `TypeUse::Ref` 越界均受控拒绝
  （`verify.bad-reference`，签名门先于签名求值——非法 TypeId 不得默认成 i32
  后进后端 panic）；声明 ty 越界沿 `verify.callee-type` 受控拒绝。
- 类型：运算数/结果（算术→i32、比较/eq/ne/logical→bool 且 eq/ne 限同类、
  一元 neg i32→i32 / not bool→bool）、调用实参/形参/结果、`return` 对函数签名、
  `if` 条件须 bool、param local 对签名逐位一致（index 界内唯一）。
- 归属唯一：每个函数的 `body` 字段必须指向 owner 恰为该函数的 body（共享 body、同签名共享、
  冒领 body 均在 verify 阶段报 `verify.owner-mismatch` 拒绝）；body→owner 反向一致性同此。
  短路 `Logical` 的 lhs/rhs 各占一个操作数位——RHS 条件执行不改变「每个 ExprId
  恰好一个语法求值位置」；未执行 RHS 仍受静态类型/初始化走查。
- 初始化：`let` 初始化；`assign` 目标须已初始化且 mutable（place 与 local 双查）；if 出口初始化 =
  可达分支交集；循环按可能零次迭代保守处理（体内初始化不外泄）。
- 块结构：块包含边仅来自 `if` 的 then/else 与 `loop` 的 body（运行时 loop 重入、break/continue 的
  LoopId 目标、递归调用**不是**包含边）；入口块零入边、非入口块恰有一个入边（唯一入口角色，禁共享块）；
  全部块从入口可达且包含图无环——结构检查（显式栈 DFS）先于数据流走查，结构失败不构造 `CheckedModule`
  （自环/断开环分别报入口角色唯一与不可达定位诊断，不 panic）；入口须在所有路径 return；
  `return`/`break`/`continue` 之后不得有可执行尾语句；块自然收尾=交还宿主结构（if 汇合/循环重入）。
- 求值位置：每个 ExprId 恰有一个 owning 引用（语句位或操作数位；禁共享、禁死表达式）且表达式图无环；
  递归走 DefRef 不复制 body。`eval_args` 左→右求值存临时，`bindings` 仅做形参映射且必须为双射：
  param 全覆盖唯一、arg 越界/重复消费/未被任何 binding 消费均拒绝；实参类型按映射逐位核对
  （`params[b.param]` ≡ `eval_args[b.arg]`），不按求值下标对位。
- 诊断：阶段化（text/bind/verify/capability/backend/link/run），`file:line:col: error[stage/code]`
  渲染，span 指向冒犯 token。

## 边界与已知限制

- 不宣称完整 HIR/Auto 语言支持；`experimental/ace/crates/core` 与 v0.5 parser/VM/A2X、auto_val::Value 无关。
- `loops`/`LoopId` 为函数体扁平表，嵌套合法性由 verify 的词法栈检查。
- 文档样例源：`docs/design/strategy/hir-examples/*.atom`（01-add / 01-add.explicit / 02-control /
  03-call-order）+ `experimental/ace/crates/core/fixtures/{valid,invalid}/`（拒绝矩阵物化）。

## text-record 组合域（PLAN-782）

- `HirDomain::TextRecord`（schema `auto.hir.text-record.draft` / revision 1 /
  profile `text-record-i32-draft`；descriptor = text ∪ record 全节点集 +
  `expr.record_clone {record, base}` 与 `expr.move {inner}` 包装两新 case，
  旧四 descriptor 零改动）。
- 名义 record 类型在组合域可含 Str/char 字段；field place 组合段
  （field-owner 逐段验证扩资源段）；资源分类为 verify 从 RecordShape 推导
  的**派生属性**——输入 Bundle 的"自称"（copyable/owned/checked）不参与，
  变异面（V-08）实证。
- move 的单一表示：let/assign/return/实参位整体根转移统一走 `expr.move`
  包装（`CallBinding.moved` 在本域禁用）；`RecordClone` 程序化 Bundle 可
  直接构造。
- 拥有形态 allowlist（let/assign/return/ctor 字段位）：RecordCtor /
  RecordClone / Call / Move——裸局部与 FieldGet（视图）前门拒。
- 结构/引用界检**先于**语义规则（畸形 Move.inner/binding 越界等 →
  `verify.bad-reference` 受控拒，不 panic）；`check_profile_ops` 四域化
  （旧四域对新节点零接受）。旧四域接受/拒绝集合零改动。

## 阶段契约

公共 HIR 的阶段边界、pass 合同模板与非法变换负面清单见
[stage-contract.md](stage-contract.md)（planned；实现状态以本文件现状节为准）。

## 相关 plan

见 [plans.md](plans.md)。
