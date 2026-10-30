# Auto 源码 record profile

> **Status**: experimental；`auto.source.record.draft` revision 1（PLAN-771）。
> 实现：`experimental/ace/crates/core/src/source/`（与 core/scalar 共用前端骨架，profile
> 显式选择）；消费端见 [project.md](project.md)；姊妹 profile 见
> [source-core-i32.md](source-core-i32.md) 与 [source-scalar-i32.md](source-scalar-i32.md)。

本 profile 在 scalar-i32 之上交付**名义 record**的首批有限值语义：类型声明
（含方法）、位置/命名构造、字段路径读写、嵌套 record、聚合参数/返回、
`static fn` 与只读实例方法、显式 `mut` 自由函数参数。共同类型/文本/借拥有
前门/内部 ABI 以 774 冻结合同
（[data-resource-contract.md](data-resource-contract.md)）为唯一接口来源；
本文件只定义 record 源码域的语法/语义/诊断合同。

## 身份

- source `auto.source.record.draft` → HIR `auto.hir.record.draft / 1 /
  record-i32-draft`（774 interface-contract §7.2 唯一身份表；descriptor
  `experimental/ace/crates/core/schema/record.atom`）。
- 身份映射为**精确可失败映射**：未注册三元组由 bind 整体拒绝
  （`bind.descriptor-unregistered`）；canonical writer 对未注册/错域一律拒绝。
- verify 分域入口 `verify_record` 限本域；core/scalar 入口拒绝本域文档
  （`verify.domain`），本域对 core/scalar-only 面防御（`verify.profile-op`
  双向生效）——三域支持/拒绝集合互不扩大。

## 类型与声明

- `type Name { 字段 [;] ... [static] fn 方法 }`：字段 `name [":"] Type`，
  类型限 `int`/`bool`/已声明 record 名；**前向引用合法**（声明顺序独立于
  名字解析）；字段类型包含图必须无环（自环/互环/断开环均拒绝，
  `verify.record-cycle`，迭代检测不递归）。
- 名义类型：`VT::Rec(i)` 按声明索引；**结构相同不同名不互换**
  （`source.type-mismatch`，诊断两侧均具名）。
- 空 record、默认字段值/默认构造参数、泛型/继承/接口/ext/成员属性、
  `mut fn`（可变实例方法）、递归/指针字段显式拒绝。
- 命名冲突：局部/参数不得与 record 类型同名（`source.duplicate`）；
  函数与 record 同名时调用形归函数（命名空间分离），类型位仍解析 record。

## 构造

- **命名构造** `Pair { left: 1, right: true }`：字段值按**源码序求值**，
  再按名映射到声明字段（禁止按声明序重排求值——trace `ba` oracle）；
  冒号必需，逗号/换行分隔；缺/重/未知字段拒绝
  （`source.argument`/`source.duplicate`/`source.undefined`）。
- **位置构造** `Pair(1, true)`：按声明序逐位绑定，实参即源码序；元数/
  命名实参/逐参名义类型在源码域检查（`source.argument`/
  `source.type-mismatch`）。解析层 `Name(args)` 与函数调用同形，由
  resolver 在局部/函数查名后按 record 表路由。
- **值捕获不变式（771 R2 冻结）**：record 值操作数在其**自身源码序求值位**
  捕获——更晚兄弟实参（含 `.mut` 调用）的副作用不得回溯改写已捕获字段；
  重排写法中更晚的读观察变异后的值。借位实参（view/mut）与此不同态：
  传活指针，读发生在被调方。

## 字段路径与赋值

- 读 `p.inner.left` 为 rvalue；record 字段读=**值拷贝**（改副本不改原对象）。
- 写 `p.left = v` 仅两入口：`var` 局部与显式 `mut` 参数；`let`/view 借位
  写拒绝（`verify.immutable-write`）。嵌套路径按实际字段 owner 逐段验证
  （`verify.field-owner`）；路径终点标量存值、record 字段整体值拷贝。
- 整体赋值 `p = <record 值>` 为值拷贝；**目的地不相交不变式（771 R1 冻结）**：
  赋值目的地存储与 RHS 触集不相交时可直写（copy elision），否则 RHS 先完整
  物化再提交——`p = P { a: p.b, b: p.a }`、`p = swap(p)` 类别名自赋值必须
  保持读-后-写正确。

## 方法与调用

- `static fn new(...) Pair` 无 receiver，调用 `Pair.new(...)`；类型名 receiver
  不求值。
- 只读实例方法：体内 `self.x` 与前导 `.x`（隐式 self）；receiver 视图借用、
  **只求值一次、先于显式实参**；写字段受控拒绝。
- 参数模式前缀拼写 `fn f(mut r Pair)`；调用点 `.mut` 后缀；缺 `.mut` 源码级
  `source.type-mismatch` + HIR 级 `verify.ownership-required` 双重门。
- 借位互斥：同一调用的**合并实参树**（含经构造操作数的读）内 mut 借位与
  任何同对象访问冲突即拒绝（`verify.mut-conflict`，774 D-04）。
- 方法首批必须显式非 unit 返回类型及 `return`；unit/隐式尾返回不在本域。

## 排除与拒绝（与支持面同为一等合同）

str/Str/char 字段、List/enum/Option/Result、use/跨模块、全局变量、
递归含指针类型、move-only/堆 record、text 类型漏入、`mut fn`、unit 返回。

## HIR 形态与可信门

- 增 `record_ctor`（源码序求值表 eval_args + 按名/按位 ctor_bindings）、
  `field_get`（base + path）、`borrow_view`/`borrow_mut`、`field_path` place、
  `method` 声明（DefKind::Method + ReceiverKind：static/view）、param
  `mode`（view/mut）。
- verify 前门（record 域）：名义类型相等、构造完整性（每字段恰一次、
  每实参恰消费一次，`verify.ctor-arity`）、逐段字段归属、借位位置/模式
  （borrow 仅直接调用实参位，`verify.eval-position`）、per-call mut 排他、
  全部表引用先界检后走查（`verify.bad-reference`；binding 内索引越界
  `verify.binding-invalid` 受控拒绝，不 panic）。
- 全路径（类型含图/字段路径/构造/解析/求值/Drop）迭代实现，深输入无栈
  崩溃；malformed partial 结构化拒绝。

## CLI、制品与测试能力

`check-source` 留在 `auto-ac-prototype`；native 构建面在 `ac-native`：
`build-source --source-profile record|auto.source.record.draft --entry <名> --output <exe>`
（未知值用法错 2；0/1/2 语义不变；事务性发布/硬截止进程约束/trace 能力三域一致）。
一键门禁 `scripts/verify-ac-record-771.ps1`（23 步 23 例：check/fmt/全测、源码旗舰与
求值序 trace、别名赋值/位置构造/ctor 操作数快照族、HIR r01/r03/r04 直连腿与强制失败
控制、10 例钉码负向、profile 隔离、SENTINEL、收据身份、旧 gates 741/745/767）。

## 验收与范围来源

测试族 record_hir/record_source（旧 record_native PE 套件随 PLAN-871 旧 lowering 退役；
其冻结 oracle 由脚本 ac-native 腿、`ac-mir-native/tests/record_method_tier.rs` 与
`gap_closure_871.rs`（HIR 纯 id 方法/前向 t_MutRet/ctor 快照）承接）；实现收据
[verification.md](../../reports/771-record/verification.md)（含 R1/R2 修复轮
与两连败诊断）；设计收据 [design.md](../../reports/771-record/design.md)。
后续 owner：text-record 组合（770 合入后第二合入者重绑基线）、可变实例
方法、unit/资源字段/List/use、通用 pass（跨域 lowering/优化保持 planned）。
