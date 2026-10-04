# Atom Schema DSL：独立描述、组合与 primary / secondary

> 状态：Draft RFC，2026-10-04。Schema 以 Atom DSL 表达；具体语法与功能分期为待讨论建议。
> 用户已澄清：`info MyNode select { ... }` 等价于 `info(id: "MyNode", kind: select) { ... }`，
> 其中 select 是业务 enum 值，不是节点 tag，也不自动选择另一份 Schema。
> 关联：[Atom/Batom v2](atom-batom-v2-design.md)。仅补充设计，不修改 v0.5 parser/运行时，不分配正式 Plan 编号。
> 第一组消费者样例：[HIR Atom 文本](auto-hir-atom-text.md)；其中 ExprKind 需要下述受限分支约束。

## 1. 建议的职责与入口

Schema 是独立、可发布、可校验的数据契约；它本身也是 Atom 文档，不是可执行 Auto 程序。
它同时描述：字段/类型、稳定编号、约束与演进、Node 内容规则、表面语法绑定。

建议以独立 `.schema.atom` 文档为跨语言、跨仓库的主要入口；文件后缀尚未冻结。
源码标注不必废弃：`@primary` / `@secondary`、字段编号、rename 等可作为生成 Schema 的入口。
两种入口归一为同一 descriptor，不维护两套解析/校验规则。

一个已发布类型必须明确权威来源：

- schema-first：独立 Schema 是源，生成 Rust/Auto/C 类型、codec 与视图；源码标注只是消费映射。
- code-first：类型与显式标注是源，生成可发布 Schema；生成产物不与源同时手工改写。
- 需要映射现有类型时，可用独立 binding 文件描述字段对应关系，但不能偷偷改变公开数据契约。

同名类型的两套描述不能靠合并字段或最后加载者获胜解决；应报冲突或显式比较一致性。
标准库自动 derive 也必须产生稳定编号，不按源码字段顺序分配已发布编号。

## 2. 作者语法与完整描述

下面使用不依赖 primary/secondary 的显式 Atom 形式。它是建议 DSL 示例，不是现有编译器已支持的语法。

```atom
schema(namespace: "auto.dsl", family: "info", revision: 1u32) {
    enum(name: "InfoKind", type_id: 1u32) {
        case(name: "select", variant_id: 1u32) {}
        case(name: "input", variant_id: 2u32) {}
    }

    node(name: "info", type_id: 2u32) {
        field(name: "id", field_id: 1u32,
              type: :Text, presence: :required, lexical: :identifier) {}
        field(name: "kind", field_id: 2u32,
              type: :InfoKind, presence: :required) {}

        syntax {
            primary: :id
            secondary: :kind
        }

        identity {
            field: :id
            scope: :parent
        }
    }
}
```

`syntax.primary/secondary` 引用字段名；Schema 编译后将它们解析为稳定 FieldId。
Text 字段的 `lexical: :identifier` 显式允许裸标识符简写成字符串值：MyNode → "MyNode"。
这是一条声明的文本字面量规则，不是读取同名变量，也不是所有 Text 字段默认允许的转换。
枚举字段的裸词 select 由已声明的 InfoKind 解析成相应枚举值，不是一般 Symbol 或任意变量引用。

上例的 identity 是一个独立、可选的契约。声明它才按 parent 范围检查 id 唯一性；
仅声明 primary 不意味着它就是 ID。parent 范围指相同逻辑父节点，根节点使用约定的文档范围；
跨文档身份与引用策略另行声明。不论是否声明唯一性，内容容器仍保留子项，不能覆盖重名子节点。

为聚焦两槽语法，上例省略内容类型/基数、unknown 与演进策略，不能据此推断内容完全开放。
正式 Node 定义必须声明内容策略，例如叶节点禁止内容、只接受若干节点类型、允许 Text 混排。
未声明内容策略是否默认禁止，需在元 Schema 冻结时决定，不能由不同 reader 各自猜测。

### 2.1 元 Schema 与 bootstrap

`schema/enum/case/node/field/syntax/identity` 本身由版本化的元 Schema 定义。
Schema 的完整表达使用显式名称与受限数据 literal，不要求先加载它自己的语法糖才能解析。

建议路径：

```text
基础 Atom 数据 parser
    → 元 Schema 校验
    → 名称/引用解析
    → 组合展开与冲突检查
    → Schema descriptor
    → 访问表 / validator / codec / DSL binder
```

初版 Rust 实现固定少量 bootstrap 元结构；后续由同一元 Schema 验证其描述并做一致性测试。
不依赖 AutoC/AAC 完成后才能读 Schema，也不要求先自举 Schema 编译器。
Schema 导入由调用者提供已解析、版本锁定的 descriptor bundle；数据 parser 不隐式联网执行文件。

### 2.2 类型与约束的表达

建议先支持有限、声明式的核心，不引入一个可执行的第二编程语言：

| 范畴 | 内容 |
|---|---|
| 原子类型 | Text、Bool、宽度明确的整数/浮点、Bytes、Symbol 等 |
| 类型引用 | 命名 record/node/enum/variant 的引用，编译后解析稳定身份 |
| 复合描述 | Array、TypedArray、引用/Link、可判别 Variant；用 Atom 数据描述，不执行类型函数 |
| 字段状态 | presence = required/optional，与 nullable、default 分开 |
| 约束 | 范围、长度、数量、枚举成员、内容类型/基数、受控引用目标 |
| 演进 | alias、deprecated、reserved ID、unknown 策略、明确迁移版本 |

例如候选数组描述为 `type: { array: :Text }`，而不是调用 `Array(Text)` 函数。
具体嵌套类型产生式仍需元 Schema 定案。默认值不得通过任意 Auto 表达式计算，
也不能让“有 default”和“字段存在且被明确提供”混为一谈。
Auto 领域完整类型系统、ROS 语义、权限或任意复杂业务校验由消费方负责。

### 2.3 Schema 自身也可以使用两槽作者语法

加载固定版本元 Schema 后，作者不必始终书写完整命名形式。下面是 §2 的同义简写：

```atom
schema info(namespace: "auto.dsl", revision: 1u32) {
    enum InfoKind(type_id: 1u32) {
        case select(variant_id: 1u32) {}
        case input(variant_id: 2u32) {}
    }

    node info(type_id: 2u32) {
        field id Text(field_id: 1u32, presence: :required, lexical: :identifier) {}
        field kind InfoKind(field_id: 2u32, presence: :required) {}

        syntax { primary: :id; secondary: :kind }
        identity { field: :id; scope: :parent }
    }
}
```

元 Schema 明确规定：schema 的 primary 为 family，enum/node/case 的 primary 为 name，
field 的 primary 为 name、secondary 为 type。名称槽允许声明的 identifier→Text 简写；
type 槽将 Text/InfoKind 绑定为受限类型引用，不执行类型表达式或寻找普通变量。
类型引用的完整形式使用明确的符号/身份描述，基础 reader 不根据首字母大小写猜类型。

因此作者简写仍使用同一 binder；bootstrap、发布与调试可使用完整形式。
工具必须能在两种形式间保持语义往返，不能把“作者语法更短”变成另一套不可交换的协议。
用于选择元 Schema 版本的文件/调用上下文必须明确；这里不能靠 document 尚未绑定的 family
字段先猜它自己的元语法。该元描述可随 loader 固定版本提供，不要求动态联网发现。

## 3. primary / secondary：两个可选的文本绑定槽

建议支持 secondary，但把它限定为 Schema 明确声明的第二个头部简写位置。
它和 primary 一样，绑定已有字段，不新增 Atom 值种类，也不在 Batom 里额外存一份。

针对上例：

```auto
info MyNode select { ... }

info "MyNode" select { ... }

info(id: "MyNode", kind: select) { ... }

info { id: "MyNode"; kind: select; ... }
```

这些是相同 Schema 下的等价领域写法；`...` 是说明占位，不是 Atom literal。
归一后的节点保存：

```text
tag          = info
fields.id    = Text("MyNode")
fields.kind  = EnumValue(InfoKind, select)
content      = ...
```

这里 tag 是节点名称；kind 是用户字段名；InfoKind 是该字段的数据类型；select 是 enum case。
都不等于整个 Node 的 Schema 身份，也不等于语言变量绑定。

### 3.1 消歧规则

1. 未声明 secondary 的节点不接受第二个裸头部值；剩余属性用显式命名形式。
2. secondary 声明必须同时有 primary 声明，两者指向不同且存在的字段。
3. 固定从左向右绑定：第一个值是 primary，第二个是 secondary。只给一个值时不猜是哪一个。
4. 不提供任何裸槽位时，可以全部使用命名字段；必填字段照常校验。
5. 不能跳过第一个槽再提供第二个。例如 `info select {}` 把 select 绑定到 id，
   不因为它恰好是枚举项就解释成 kind；若 kind 未在其他位置提供，则诊断缺少 kind。
6. prime/secondary 与括号/正文同名属性冲突时诊断重复字段，即使值相同也不静默覆盖。
7. enum case 必须属于绑定字段声明的 enum；不按值反查其他字段或其他 Schema。
8. 第一阶段限制为至多两个头部字面量槽；列表/对象/复杂类型值使用命名字段。
   复杂 DSL 若确有需求，再定义显式且版本化的语法规则，不自动扩成任意裸参数列表。

数据 DSL 中 MyNode 是由词法规则接受的字符串简写，不定义变量。
若 Auto 可执行源码要同时支持变量定义/引用，必须由语言前端用明确的语法区分，
不能把“看起来像变量声明”当成已经执行了变量绑定。

### 3.2 typed enum 与自描述格式

`kind: select` 是 Schema-aware 的领域写法，其 enum 类型来自字段契约。
只读基础 Atom、没有 Schema 的 reader 不能把裸 select 自动解释为 InfoKind.select。

canonical 自描述文本应使用携带类型/分支身份的 Variant literal，或明确携带所需描述符；
该 literal 的词法属于 Atom 主设计的下一步。Batom/D 同样保留枚举身份与分支身份，
Batom/S 可用 Schema 和稳定编号缩短；都不能把 enum 不加说明地降成普通字符串/Symbol。
尚未绑定的 SyntaxNode 可以保留原词，但不能宣称它已是完成 typed 校验的 Atom 值。

### 3.3 kind 作为枚举不自动进行 Schema 分派

当前用户例子只是 info 的普通枚举字段，primary/secondary 足以表达。
若未来 select/input 决定不同的内容结构或必填字段，应另声明判别联合/分支约束。
不能因为字段恰好叫 kind 或 type，就启动特殊分派，也不能让 secondary 自带这种能力。

若引入分支 Schema，公共头部字段的类型、编号和语法绑定必须一致，先绑定判别字段，
再选择明确的分支验证 body；未知分支按声明策略诊断或保留。此功能需独立验收，
不因为 secondary 的例子而默认把通用 Schema 分派列为当前原型的必交付。

### 3.4 HIR 消费者补充：受限分支字段契约

[HIR 文本样例](auto-hir-atom-text.md) 使用 expr.kind 区分 constant/read_local/binary/call。
每个 case 有不同的 required/allowed 字段，这是一项明确的新消费者需求：
公共字段 id/type/kind 加上选中分支的字段契约，才构成该表达式的完整形状。

建议最小能力限于已声明的 enum 判别字段及有限 case；选择分支不执行 Auto 代码。
禁止分支重新定义公共字段，不用继承 override 解决冲突；整种类型的 FieldId 必须一致且可追踪。
字段形状属于 Schema 校验，操作数类型/调用绑定等仍属于 HIR verifier。
这个功能由 Schema 显式声明，不因出现 secondary 或字段名 kind 就自动启用。
具体元语法待定；它应进入 HIR 第一阶段的 Schema 验收，复杂嵌套联合与完整 Schema 分派仍另行分期。

## 4. 组合：建议 v0.6 核心支持

相关 Schema 确实需要复用；建议先用两种显式组合，而不是直接引入 OO 继承。

### 4.1 嵌套类型：保留边界

一个 style 字段引用 Appearance Record，是整体嵌套组合。
Appearance 有自己的 TypeId/FieldId；父节点只给 style 分配一个 FieldId。
不同类型的局部 FieldId 可以相同，因为它们属于不同类型身份。
这种方式不把 Appearance 的字段自动摊到父节点，也不制造字段名冲突。

### 4.2 fragment：显式字段复用

跨多种节点复用 Identified 等字段集，可定义不可独立编码的 fragment：

```atom
fragment(name: "Identified") {
    field(name: "id", type: :Text,
          presence: :required, lexical: :identifier) {}
}

node(name: "info", type_id: 2u32) {
    include(fragment: :Identified, ids: { id: 1u32 }) {}
    field(name: "kind", field_id: 2u32,
          type: :InfoKind, presence: :required) {}

    syntax { primary: :id; secondary: :kind }
    identity { field: :id; scope: :parent }
}
```

这个片段替代前例的 info 定义，不与它同时定义第二个同名类型。
fragment 复用字段说明和约束；宿主明确映射公开 FieldId，避免多个 fragment 争用编号。
fragment 不自动携带宿主 primary/secondary、identity、tag 或内容规则，这些由完整节点声明。
同一 fragment 在不同类型中使用不同字段编号是允许的，不能把编号当成全局字段名。

展开规则建议：

- 显式声明 include；不因字段名称/布局相同自动组合。
- 重复字段名、编号冲突、遗漏编号映射均报错；相同定义也不自动吞掉重复。
- 初版不支持覆盖 fragment 的类型/默认值，也不以 include 顺序决定胜者。
- 引用必须可解析、版本锁定；编译时展开，读值时不沿继承链查找字段。
- fragment 组合依赖不允许循环；领域数据类型的受控递归引用是另一件事，不一并禁止。
- 可复用 fragment 定义和文档来源保留在 descriptor 的来源信息中；展平语义必须确定。

约束的复用不等于任意交集运算。需要宿主细化时，应设计显式 refine 与可验证的规则，
不允许一个隐含 override 放宽原契约。第一原型可先要求完整字段与约束一致，不实现 refine。

### 4.3 编译与版本身份

Schema 编译后得到完整 FieldId/槽位/内容约束表，避免每次访问递归查找 fragment。
来源图、导入锁定版本与完整有效契约均需可追踪；指纹算法在格式冻结时定义。
不能只 hash 根文档的 include 名称、忽略被复用定义的改变。
fragment 改动不应静默升级所有已发布宿主 Schema；需显式更新依赖、检查兼容并发布。

## 5. 继承：与组合和子类型分别讨论

建议 v0.6 先交付组合，不把多继承或面向对象的完整替代规则带进通用格式。
“复用 Base 字段”“Derived 可作为 Base 使用”“Derived 与 Base 二进制兼容”是三个不同承诺。

即使某节点多了几个字段，也不能自动证明它能作为另一个 tag 的节点使用；
closed/unknown 策略、必填字段、内容类型和身份规则都会影响替代关系。
Schema-first 不应为了像 class 而默认许诺语言运行时继承/方法分派。

若未来提供 `extends`，需要先确定它是组合展开的便利语法，还是正式的类型替代关系。
作为便利语法时必须保持明确的编号/冲突规则，不能暗示子类型；作为正式继承时，
需另外定义字段/约束变化、内容可替代性、稳定编号、tag 关系与版本兼容。
v0.6 当前设计不依赖它；不预先采用隐含 override、菱形多继承或基类布局强转。

## 6. 原型次序与验收

1. 元 Schema 最小集、独立描述 loader、名称/ID/presence 校验。
2. 固定 Node/Record/enum descriptor 与字段/内容访问表。
3. primary/secondary/命名字段 binder，裸 identifier→Text 的声明式规则，enum case 解析。
4. 嵌套类型与 fragment 展平/编号冲突/依赖锁定。
5. 一条 schema-first codec/view；随后用同一 descriptor 接入 code-first 标注生成。
6. 有需求和独立验收后再加入复杂判别联合、refine、继承、扩展语法。

应覆盖：两槽简写与命名形式同值、缺失/重复属性、无 Schema 不猜 enum、
primary 不默认变 identity、重复 ID 不覆盖内容、组合编号稳定、冲突与循环诊断、
Schema 版本变更与 typed enum 的自描述往返。文档里的示例不是已实现的语法承诺。

## 7. 历史资料与本次范围

[旧 extending_atom 设计](../raw/extending_atom.md) 已提出 `@primary/@secondary/@args/@kids`，
也曾讨论按第一/第二成员隐式决定角色。本稿保留显式标注的意图，放弃按字段声明顺序猜角色。
旧稿里 secondary 在参数列表后的示例，与用户本次要求的两个连续头部槽不同，
不能自动视为相同语法；完整 v0.5 恢复后对实际用例建立 adapter/迁移矩阵。

本稿未更改 parser、derive、Schema 定义代码或测试。新旧语法共存和兼容迁移需正式 Plan。
