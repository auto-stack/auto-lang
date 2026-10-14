# PLAN-771 规范增量提案（proposed-spec-delta）

> 状态：**提案**——本文是 T-07 冻结稿，未发布 canonical；merge 阶段
> （T-09）经独立复审通过后按 `/auto-plan:merge` 沉淀，与 design.md
> §规范增量表对应。冻结内容以本文件为准；与 774 已冻结合同冲突处，
> 774 优先（语义定义一处）。

## SD-01（add）docs/specs/auto-ac/source-record.md

新文件：record 源码 profile 的有限语言合同。要点：

- 身份：source `auto.source.record.draft` → HIR `auto.hir.record.draft /
  1 / record-i32-draft`（774 interface-contract §7.2 唯一身份表）。
- 支持面（首批，全部经真实 PE 实证）：
  - `type Name { 字段 [;] ... [static] fn 方法 }`：字段
    `name [":"] Type`（i32/bool/已声明 record 名），前向引用合法；
    空 record、默认值、泛型/继承/ext/成员属性、`mut fn` 显式拒绝。
  - 构造：位置 `Pair(1, true)`（按声明序逐位）与命名
    `Pair { left: 1, right: true }`（**按源码序求值**，再按名映射；
    冒号必需，逗号/换行分隔；缺/重/未知字段拒绝）。
  - 字段路径：读 `p.inner.left`（rvalue，record 字段=值拷贝）、写
    `p.left = v`（仅 `var` 局部与显式 `mut` 参数两入口）。
  - 方法：`static fn`（`Pair.new(...)`，无 receiver）；只读实例方法
    （`self.x` / 前导 `.x` = 隐式 self；receiver 视图借用、只求值一次、
    先于显式实参求值）。
  - 参数：模式前缀拼写 `fn f(mut r Pair)`（真实 parser 语义，Design 04
    文档词序不采用）；调用点 `.mut` 后缀；缺 `.mut` →
    `source.type-mismatch`（源码级）+ `verify.ownership-required`
    （HIR 级）双重门。
- 拒绝面（与支持面同为一等合同）：str/Str/char 字段、List/enum/
  Option/Result、use/跨模块、全局变量、unit/隐式尾返回、递归/指针
  字段、text 类型漏入。
- 诊断码（771 新增，命名空间与既有一致）：`verify.record-cycle`、
  `verify.field-owner`、`verify.ctor-arity`、`verify.duplicate-decl`、
  `verify.shadowed-type`、`verify.record-empty` + 774 冻结名
  `verify.mut-conflict`、`verify.ownership-required`；机器层不新增
  状态码（无堆无 buffer）。
- 验收锚：V-01..V-09（本计划 §6 Case 表）。

## SD-02（modify）docs/specs/auto-hir/project.md

追加：record 名义类型（TypeKind::Record 声明序字段表；结构相同不同名
不互换）、字段 place（PlaceKind::FieldPath 逐段归属）、构造
（record_ctor：源码序求值表 + 按名绑定）、借位表达式
（borrow_view/borrow_mut）与方法声明（DefKind::Method + ReceiverKind）
的 HIR 形态与可信门（含图迭代环检测、名义相等、mut 排他）。旧
core/scalar profiles 的接受/拒绝集合零改动（域防御
`verify.profile-op` 双向生效）。通用 pass（跨域 lower/优化）仍 planned。

## SD-03（modify）docs/specs/auto-hir/stage-contract.md

追加：record 域阶段责任——binder 绑定 record/field/method 结构与
CategoryRef；verify 前门执行 774 D-04 借用/所有权规则（拒绝时目标
操作 native 调用计数为 0）与构造完整性；native 阶段按 D-05 布局
（声明序、4 对齐、size=Σ字段尺寸）与 ABI-D/E lowering、copy-elision
与 out-slot 返回的存储不重叠义务。未实现部分（通用聚合优化、公开
ABI）保持 planned 标注。

## SD-04（modify）docs/specs/auto-ac/project.md

追加：record profile 实验性支持面（source/HIR→native 闭环已实证，
V-01..V-10 全绿）；内部 ABI 边界重申（774 D-05：非生产模块 ABI、非
ACC Cranelift 桥）；text-record 组合未立项（后续 owner）。

## SD-05（modify）docs/specs/auto-acc/project.md

追加：MD403/404 的有限子集取证（lexer 关键字位、parser type 块/
static fn/方法调用/构造拼写、名义 record 语义）在 ac-core record
profile 中有真实实现证据；P/CG/Token 剩余 text/enum/List 字段与
可变实例方法仍为前置（后续扩面 owner），不关闭 Gen1/GOAL-017。
