# AutoHIR 的 Atom 文本映射：第一组完整样例

> 状态：Draft 表达方案，2026-10-04。承接 [HIR 语义草案](auto-hir-design.md)、
> [Atom/Batom](atom-batom-v2-design.md) 和 [Schema DSL](atom-schema-dsl-design.md)。
> 本稿及 .atom 样例是新版 Schema-bound 数据语法的设计资产，尚未通过实现中的 parser/verifier。
> 不是可执行 Auto 源码，不声明现有 Atom reader 已支持这些形式，也不冻结公开 Schema 编号。

## 1. 用 Atom 组织 HIR

建议每份文本包含 HIR Schema/profile、一个或多个模块；模块保存类型表、声明表、函数体表。
函数体包含局部绑定、表达式和块；需要写入时显式增加 place 表。

```text
hir
  module
    types                 类型定义与函数签名
    declarations          已解析声明及其函数体/外部操作
    bodies
      body                owner、entry
        locals            参数和局部绑定
        places            写入/借用位置（按需）
        expressions       有类型的表达式图
        blocks            有序语句与结构化控制流
```

这里是公共高层表示：保留 if/loop 和明确的调用绑定，不规定机器寄存器、ABC 或 native ABI。
完整所有权、泛型、方言、来源表等能力按主设计继续扩展；首批样例只使用显式 i32/bool 核心。

## 2. primary / secondary 映射

| 节点 | primary | secondary | 例子 |
|---|---|---|---|
| module/body/block | id | 无 | `body b_add { ... }` |
| type | id | kind enum | `type t_i32 i32 {}` |
| function/intrinsic/local | id | HirTypeRef | `function d_add t_add { ... }` |
| expr | id | ExprKind enum | `expr e_add binary { ... }` |
| place | id | PlaceKind enum | `place p_i local { ... }` |
| types/declarations/bodies/locals/places/expressions/blocks | 无 | 无 | 固定结构容器 |
| let/return/if/loop/assign/break/continue | 无 | 无 | 首批使用命名属性，避免另增隐式规则 |

id 槽是 Text，并声明 identifier 字面量简写；类型引用槽先绑定标识符文本，再解析到对应类型表。
kind/op/role/access/effect/overflow 等裸词按对应字段的 enum 类型解释，不是读取 Auto 变量。
同一个词可以在不同语法位置表示枚举值或节点 tag；例如 `kind: function` 中 function 是枚举项，
`function d_add t_add {}` 中 function 是节点 tag，判别依据是语法位置与 Schema。

命名字段可以在括号/正文中互换；重复声明统一报错。比如：

```atom
expr e_add binary { type: "t_i32"; op: add_i32; overflow: trap; lhs: "e_a"; rhs: "e_b" }

expr(id: "e_add", kind: binary, type: "t_i32", op: add_i32,
     overflow: trap, lhs: "e_a", rhs: "e_b") {}
```

这是 Schema-bound 文本，不是已经冻结的全自描述 canonical 文本。调用者提供准确的 Draft
descriptor；root 的 schema/revision 是要求校验的契约标识，不是自动联网加载指令。
发布时必须补精确 descriptor 指纹，不能仅以 revision 数字证明身份。
没有 descriptor 可以保留原始 SyntaxNode，不能把裸 binary/i32 自动宣称为已解释的 enum。

## 3. 文本 ID 与真正引用

为了审阅，本稿使用 t_i32、d_add、b_add、e_add 等可读标签。
它们是本次导出的局部 ID，显示名称 add/sum 另存；不是跨版本永不变化的公共符号身份。
前缀只方便人阅读，不是校验器的类型检查依据。

| 引用属性 | 查找范围 |
|---|---|
| function/intrinsic.type、local.type、expr.type、place.type、签名 params/result | 当前模块的语言类型表 |
| body.owner、call.callee | 当前模块的声明表 |
| function.body | 当前模块的函数体表 |
| body.entry、if.then/else、loop.body | 当前函数体的块表 |
| read_local.local、let.local、local place.local | 当前函数体的局部表 |
| lhs/rhs、let.value、return.value、if.cond、assign.value、eval_args 的项 | 当前函数体的表达式表 |
| assign.place | 当前函数体的 place 表 |
| break/continue.target | 当前控制流嵌套中有效的 LoopId |

每张表内 ID 唯一，函数体内的 ExprId/LocalId/BlockId 分别属于各自类别；
不同模块或函数体可使用相同的文本标签。可读 name 可重复，不用它替代引用身份。

Schema binder 将这些字段绑定为不同的 HirTypeRef/DefRef/ExprRef 等类型化引用，
不能因它们在文本里都是字符串就允许互换。内存可用整数句柄；导出/导入需受控重映射。
当前模块/函数体提供局部 owner；跨模块引用需显式扩展模块/包身份，不只传一个短字符串。
这些域引用暂用 ID 数据表达，不先发明 Atom 核心 LocalRef/Link 的新词法。
以后是否用核心引用编码优化，应保持相同引用类别、作用域和身份规则。

参数 index 与调用 bindings 的 param 使用从零起的函数签名位置；不是 LocalId 或 FieldId。
local.mutable 未声明时在本 profile 中默认 false，与字段是否显式出现分开记录。
本稿的 HIR revision、语言类型 ID、Atom SchemaTypeId/FieldId、未来 Batom version 各自独立。

## 4. 完整样例一：两个参数、局部绑定、加法与 return

- [01-add.atom](hir-examples/01-add.atom)：双槽作者语法。
- [01-add.explicit.atom](hir-examples/01-add.explicit.atom)：同一份 HIR 的全命名形式。

语义是接收两个 i32 参数，计算 sum，然后返回 sum。主要段落如下，完整闭合文本见附件：

```atom
function d_add t_add { name: "add"; body: "b_add" }

expr e_add binary {
    type: "t_i32"
    op: add_i32
    overflow: trap
    lhs: "e_a"
    rhs: "e_b"
}

block k_entry {
    let(local: "l_sum", value: "e_add") {}
    return(value: "e_sum") {}
}
```

表达式表只登记计算结构，不按它的文本出现顺序执行。入口块按 let、return 的次序执行；
let 求值 e_add 时，按 lhs/rhs 的约定顺序读取参数，初始化 l_sum；return 随后读取它。
add_i32 + overflow: trap 是样例 profile 的显式数值语义，不裁定旧 Auto int 的宽度/溢出行为。

建议验收值：add(2, 3) = 5；i32 最大值 + 1 按该 profile trap。
这些是未来运行验收的预期，本次未生成/运行程序。

## 5. 完整样例二：可变局部、place、if 与循环目标

[02-control.atom](hir-examples/02-control.atom) 描述从 i = 0 开始计数直到 i >= n。

- 局部 l_i 明确 mutable；写入目标 p_i 是 place，不能与 read_local 表达式混用。
- loop 声明 q_count，循环体通过 if 转入 k_step 或 k_stop。
- k_step 更新局部后 continue(q_count)，k_stop 用 break(q_count) 返回 loop 后的语句。
- e_less 等节点描述一个求值位置，不是只算一次的 memoized 值；每轮执行到条件时重新求值。
- 内部 ExprId 的存在不意味着底层已是 SSA；continue/break 是结构化控制流操作。

预期 count(3) = 3、count(0) = 0、count(-2) = 0。代码没有后端执行结果声明。
本例每个块在该结构中有唯一入口角色；任意共享块/跨层跳转暂不接受，防止把结构化表示
悄悄扩成未定义的通用 CFG。loop 的子块树由 BlockId 引用承载，须验证无块包含环。

## 6. 完整样例三：参数求值顺序与形参绑定

[03-call-order.atom](hir-examples/03-call-order.atom) 给 pair(a,b) 定义 a * 10 + b。
测试专用 mark_a/mark_b 操作分别记录 a/b 并返回 1/2，调用者先计算 b，再计算 a：

```atom
expr e_call call {
    type: "t_i32"
    callee: "d_pair"
    eval_args: ["e_mark_b", "e_mark_a"]
    bindings: [
        { param: 0u32, arg: 1u32 },
        { param: 1u32, arg: 0u32 }
    ]
}
```

eval_args 从左向右求值并保存临时结果；bindings 只把这些结果映射到形参，不能再次求值。
因此预期 trace 是 ["b", "a"]，结果为 12。按形参顺序重新执行表达式会改变 trace，
即使结果仍碰巧为 12，也不符合契约。
当前 profile 只定义已解析静态 callee；动态 callee/receiver 的求值顺序需另有显式节点。

requires 声明 hir.test.trace.v1；intrinsic 是测试能力，不是 production ABI 或语言库承诺。
消费方无此能力应明确拒绝，不能忽略 observable 属性或随便生成一个同名函数调用。
未来可在隔离 HIR runner/后端夹具中实现它，不在旧 VM 加临时 native。

## 7. inventory、执行序列和表达式位置

types/declarations/expressions 等 inventory 按 ID 解释，文本顺序不触发执行。
签名 params、eval_args、block 的语句内容是有序的；if 只执行所选分支。
一般 binary 在本 profile 中按 lhs 再 rhs 求值；短路操作以后必须有独立明确规则。

参数对整个函数体可见；binding 的 inventory 记录不自动让它在任意位置可见。
let 所在块及其后续结构化子块决定该绑定的词法作用域；读取还须通过初始化检查。
当前 local place 不求值索引/receiver；assign 先确定位置，再求值 RHS，最后写入。
未来增加带计算的 place 时也必须显式保持该顺序，不能藏在字段集合的遍历里。

每个表达式记录一个结构化求值位置，在表达式/语句拥有关系中只有一个入口引用。
它可以因循环再次到达而多次执行；不能同时挂在多个语句/参数位置，导致执行次数含糊。
需要复用已求值结果时，用局部临时变量和新的 read_local 节点；共享常量表可另行加入。
表达式包含图须无环；函数递归用 DefRef，不用表达式包含环。

Atom 的 Node content 始终保存顺序。HIR 语义可以把某些 inventory 视为按 ID 的集合，
但 HIR 的图等价/缓存归一化应另定义，不能让通用 Atom writer 随意排序所有内容。
全局初始化顺序以后要有显式 module_init 序列，不能由声明表排版隐含决定；本 profile 不含全局初始化。

## 8. 对应 Schema 的最小职责

本次先确定字段绑定，不分配公开 Schema 编号/派生 Rust 布局：

- module、body、type、declaration、local、expr、place、block 各有明确的 owner、字段和内容约束。
- type.kind、expr.kind、place.kind 具有明确 enum 身份；每个 case 的 required/allowed 字段不同。
  read_local 必须有 local；binary 必须有 op/lhs/rhs；call 必须有 callee/eval_args/bindings。
- 数值操作还校验输入/输出类型；lt_i32 产出 bool；算术产出 i32，算术的 overflow 策略明确。
- type.kind=function 的 params/result 是类型引用；local.role=param 要有 index，binding 不可用 index 冒充参数。
- body.owner 与 function.body 相互一致；参数索引/类型与函数签名一致。
- return、let、if、assign 等节点有不同字段契约，不能把它们作为任意 property bag 接受。
- 表结构与 required 字段由 Schema 校验；目标身份、初始化、循环目标、类型和调用能力由 HIR verifier 校验。
- 本 profile 默认拒绝未知语义 tag/case；以后持久化 unknown 的能力不能绕过 Checked HIR 门禁。

这暴露出 Schema 的下一项需求：expr.kind 等判别字段选择字段形状的受限分支契约。
它不能由 secondary 自动推断，也不需要完整继承或任意代码执行。
统一 Expr 节点便于阅读；若分支契约成本不合适，也可以比较不同 ExprKind 独立 node tag 的方案，
在正式元 Schema 冻结前用同一批样例决定。

## 9. 错误样例的变更矩阵

以下对完整文件单独做一处变更即可形成反例；记录预期诊断，尚未有 verifier 的实际测试收据：

| 基础文件 | 单独变更 | 预期拒绝原因 |
|---|---|---|
| 01-add | 把 e_add.rhs 改为 "e_missing" | 悬空 ExprRef |
| 01-add | 把 rhs 改为 "l_b" | 引用类别错误，LocalId 不是 ExprId |
| 01-add | 在 return 前删去 let | 读取未初始化的 l_sum |
| 01-add | 同时在括号和正文声明 e_add.type | 跨区域重复字段 |
| 01-add | 把 e_b.type 改为未声明的 "t_bool" | 悬空 HirTypeRef |
| 02-control | 把 e_less.type 改为 "t_i32" | lt_i32 的结果类型错误；if 条件不为 bool |
| 02-control | 把 l_i.mutable 改为 false | 对不可变局部写入 |
| 02-control | 把 continue.target 改为 "q_missing" | 未绑定/不在当前嵌套范围的循环目标 |
| 03-call-order | 把第二项 bindings.param 改为 0u32 | 重复绑定形参 0，形参 1 缺失 |
| 03-call-order | 把 bindings.arg 改为 2u32 | 参数临时结果索引越界 |
| 03-call-order | 无 test.trace 能力的后端消费它 | 不支持 required 语义能力 |

跨 body 局部反例需在 b_caller 插入一个 read_local，引用只属于 b_pair 的 l_a；
这是额外的组合变更，不能宣称已由上表的单字段 mutation 自动覆盖。
schema/enum/ID/presence 的结构校验与 HIR 语义拒绝应分别报告，不合并成不明原因的 parse error。
尚未绑定的数据不能仅靠 root 写出 checked 就跳过这些校验。

## 10. 原型边界与下一步

本批覆盖：明确数值类型、函数签名/声明/局部身份、表达式图、有序执行、place、结构化控制流、
调用绑定和能力拒绝。尚未覆盖完整来源表、enum/Record 值构造、跨模块、泛型/接口、所有权、
异常/清理、动态值和 UI 方言；不能以这三个例子宣称 HIR 已完整支持 Auto 或 AAC。

接下来先对以上字段、简写和引用规则评审；补受限分支 Schema 与 enum/引用 literal 的规范。
正式 Plan 内实现独立 text binder/typed HIR/verifier，跑同义写法与反例，再验证往返和后端。
不在当前不可见完整 v0.5 的阶段直接替换 AST/旧 Atom/VM/A2X。

## 11. 本次文档核查

2026-10-04：四份样例的字符串和括号/方括号/花括号闭合检查通过；
01-add 的头部按声明映射展开后，与全命名样例去掉注释/空白的文本一致。
本检查只针对这些样例的头部与分隔符，不是通用 Atom parser 或语义往返验收。
本地文档链接、fence、空白及 Git diff 检查通过；未运行 cargo、HIR verifier 或后端程序。
