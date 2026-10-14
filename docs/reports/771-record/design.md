# PLAN-771 T-01 设计冻结（design.md）

- 计划：PLAN-771 r2 · T-01（work）；record profile `auto.source.record.draft`
  → HIR `auto.hir.record.draft / 1 / record-i32-draft`（774 interface-contract §7.2 唯一身份表）。
- 本文件是 T-02..07 的唯一语法/规则/码/Case/预算来源；与 774 冻结合同冲突时，
  **先修订本文件再实现**，不得以实现方便改语法。
- 基线：worktree `D:/autostack/.wt/lang-771/auto-lang` @ `plan-771-dev`，基面
  master `199d6509e`（774 已合入；770 execution_done 未合入，无实现依赖）。

## 1. 输入核对（T-01 事实差异表）

| # | 材料 | 核对结果 | 消费 |
|---|---|---|---|
| I-01 | 774 interface-contract.md（六轮复审 pass 后随 930d05fae 合入） | §5/§7.2/§8：771 消费表 = D-01 int 精度、D-04 receiver 规则、D-05 ABI A/D/E + record 布局、D-06 无堆交互、身份三元组 | 全文遵循 |
| I-02 | 774 decisions.md §D-04/§D-05 | receiver（只读实例方法 = view 借用 receiver；显式 mut 自由函数参数；`mut fn` 首批不支持）；布局 = 声明序、自对齐 4、size=4n、offset(fᵢ)=4i、无重排；无 >8B 聚合返回，record 返回走 out-slot（caller 预置存储 + callee 全字段写入） | §3/§4 |
| I-03 | 774 模型拒绝码 | 与 771 相关：`verify.mut-conflict`（mut 排他）、`verify.ownership-required`（调用点缺 `.mut`）；其余 6 码为 text 侧，771 不注册 | §5 |
| I-04 | 真实 parser（crates/auto-lang/src/parser.rs） | `fn_params`（parser.rs:9678 一带）：参数模式**前缀于名字** `mut name Type`（`view` 默认省略）；方法体隐式注册 `self`（parser.rs:9258-9270）；`Ident(已注册类型)+(` → 位置参数 node 实例（parser.rs:12483+ is_constructor 分流） | §2 |
| I-05 | 自举 parser（packages/aavm/lib/parser.at） | `parse_type_decl`（1769）：`type Name {` + 成员 `name [":"] Type`（冒号可选）+ `static fn`/`fn` 方法 + `}`；字段默认值/泛型/继承/is/as/has 均 v2 unsupported；`ctor_node`（972）：命名构造 `Name { field: value, ... }`（**冒号必需**，逗号/换行分隔）；`expr_pratt` 前导 `.field` = `Dot(self, name)`（1211） | §2 |
| I-06 | Design 10 §OOP/§Parameter | `type` 块内 `static fn`/`fn`（无 impl 块）；`self` 可选、`.field` 简写；`mut fn` 存在但首批不支持；参数 `ref`/`mut` 注解。**与 I-04/I-05 的差异：Design 04 文档写 `u mut User`（名前模式后），真实 parser 两处一致为 `mut u User`（模式前名后）——以真实 parser 为准** | §2 |
| I-07 | 772（archived） | scalar 管线（772 修复后）为 record 实现平台；`verify_domain`/`HirDomain`/descriptor 分域模式照抄 | T-02..05 |
| I-08 | 745 numeric-boundary（MD403/404）、743 | lexer Token 五字段与 parser P/CG/static new 证明需要；本批只交付标量子集，不覆盖 enum/text/List 字段（SD-05 边界） | SD-05 |
| I-09 | ac-core 现状（master@199d6509e） | source 七模块（lexer 528/parser 1413/resolve 1187/typecheck 233/adapter 1738/ast 290/mod 94）；hir.rs TypeKind={I32,Bool,Function}、PlaceKind={Local}；native.rs Vt={I32,Bool,Function}；`type`/`static`/`mut`/`self` 在 lexer RESERVED_KEYWORDS（767 的 `for` 先例：profile 消费关键字按档提升） | T-02..05 |
| I-10 | 774 探针（PE 实证） | `p774_rec_view`/`p774_rec_mut`/`p774_rec_out`/`p774_rec2_view`（含嵌套 Inner{x@0,y@4}+flag@8 size12）在真实 COFF/PE 61→64 探针全绿（gate-20261014-022118）= ABI-D/E + 嵌套布局 + out-slot 返回的机器可行性已证 | §6 probe 引证 |

**语法差异裁定（§10.1 待澄清事项 1 的解答）**：774 D04/D05 已冻结且与真实语法核对无冲突；
命名构造/参数修饰拼写以真实 parser 为准——命名构造 `Name { field: value }`（冒号必需）；
参数修饰为**模式前缀** `fn f(mut r Pair)`（view 默认省略；Design 04 文档的 `u mut User`
顺序不采用）。调用点模式后缀 `f(p.mut)`。冲突时无需修订 r2 合同：774 §7.4 的
"显式 `mut Str`"表述与模式前缀拼写（`mut` 修饰该参数）语义一致，仅词序以 parser 为准。

## 2. 冻结语法（record source profile）

仅 `--source-profile auto.source.record.draft` 下接受；core/scalar 档接受/拒绝集合零改动
（`type`/`static`/`mut`/`self` 在两旧档仍 Reserved → `source.unsupported`）。

### 2.1 声明（对齐 I-05 parse_type_decl）

```auto
type Pair {
    left int          // 字段：name [":"] Type；分隔 = 换行或 ";"（expect_eos）
    right: bool       // 冒号可选（两种写法都接受）
    static fn new(l int, r bool) Pair { return Pair { left: l, right: r } }
    fn left_is(l int) bool { return .left == l }   // 只读实例方法
}
```

- 字段类型 ∈ {`int`/`i32`（同义，精确 i32）、`bool`、已声明 record 名}；**前向引用合法**
  （预登记后闭合检查，声明顺序独立于名字解析依赖顺序）。
- 空 record（零字段）首批显式拒绝（plan §5.2；真实 parser 允许，但本批不支持）。
- 字段默认值/泛型/继承/is/as/has/ext/成员属性（`#`）：拒绝（真实 parser v2 亦 unsupported）。
- 字段名重复、字段与同名方法、方法名重复（同 type 内 static/instance 之间）→ 拒绝。
- 方法首批：显式非 unit 返回类型 + 显式 `return`；`mut fn` 拒绝（774 D-04 F14）；
  实例方法 receiver = 只读 view 借用（ABI-D `*const`），无显式 receiver 参数写法。

### 2.2 表达式

| 形态 | 拼写 | 语义 |
|---|---|---|
| 位置构造 | `Pair(1, true)` | 按声明序逐位绑定；实参数 = 字段数，逐位类型检查 |
| 命名构造 | `Pair { left: 1, right: true }` | **按源码序先求值全部实参，再映射到声明字段**（禁止按声明序重排求值）；缺/重/未知字段拒绝 |
| 字段读 | `p.left` / `self.left` / `.left`（方法体内 = `self.left`） | rvalue；路径逐段 owner 验证（`a.b.c` 每段按实际字段归属） |
| 方法调用 | `p.left_is(3)` / `Pair.new(1, true)` | receiver **先求值且只求值一次**，随后显式实参按源码序 |
| 自由函数调用 | `f(p)` / `f(p.mut)` / `f(b: 2, a: 1)` | 命名实参重排合法（绑定按名）；`.mut` 调用点后缀 = mut 传参 |

### 2.3 语句与可变性

- `let` / `var` 沿用；`var b = a`（record）= **值拷贝**（4n 字节平拷），此后 `b.f = ..` 不影响 `a`。
- 写字段仅两入口：`var` 局部的字段（`var p Pair; p.left = 1`）与显式 `mut` 参数
  （`fn f(mut r Pair) { r.left = 1 }`，调用点必须 `f(p.mut)`，缺 `.mut` →
  `verify.ownership-required`）。
- `let`/view 传参/只读 receiver 写字段 → `verify.immutable-write`。
- 同一对象两 mut 参数（`f(p.mut, p.mut)`）→ `verify.mut-conflict`（前门，native 调用计数 0）。
- 分支初始化规则不放宽：构造即全字段初始化；无默认字段值。

### 2.4 HIR 形态（Atom 文本与 schema/record.atom）

- `TypeKind` 增 `record`（slots: 字段表 `[field(name, TypeUse)]` 声明序）；名义身份 =
  模块内声明（TypeId）；结构相同不同名不互换（verify 名义相等）。
- `ExprKind` 增 `record_ctor`（按源码序求值表 eval + 逐字段绑定映射）、
  `field_get`（base ExprId + 逐段 FieldId 路径）。
- `PlaceKind` 增 `field_path`（local 根 + FieldId 逐段路径，typed place）。
- `DefKind::Function` 增方法归属（owner record + `static`/`instance`）；实例方法签名
  隐含 receiver view 参数（ABI-D），static 无 receiver。
- 参数模式：`LocalDef` 增 `param_mode`（view/mut；record 参数专用，i32/bool 参数
  首批仍按值 ABI-A，模式位只对 record 参数有意义——非 record 参数写 `mut` 拒绝）。
- 通用 pass（跨域 lower/通用优化）仍 planned；record 域有自己的 verify/native 入口。

## 3. 布局与内部 ABI（774 D-05 全文适用，此处为 771 落点）

- 字段按声明序、自对齐 4、size = 4×n、对齐 4、offset(fᵢ) = 4·i、无重排无尾随填充
  （首批全部字段 4 字节，布局为纯算术；checked u64 算术防未来扩面溢出）。
- ABI-A：i32/bool 按值（I32，bool 规范 0/1）。
- ABI-D：record view 参数 = 单参 `*const Rec`（I64）；callee 只读。
- ABI-E：record mut 参数 = `*mut Rec`（排他借用）；record 返回 = **out-slot**：
  caller 预置存储 + callee 全字段写入（函数机器签名追加尾参 `out: *mut Rec`、
  机器返回 void）。**不使用 >8B 聚合 RAX/sret 隐藏参**。
- 嵌套 record：无递归值树——布局平铺（探针 P2：1024 层嵌套 size=4100），
  字段访问 = 偏移算术（P3：4096 段路径 = 平坦偏移累计）。
- 值拷贝 = 4n 字节平拷（栈槽/参数位间）；构造临时、赋值拷贝、返回 out-slot
  三者存储互不重叠（caller 分配序：实参槽 → out-slot → 临时，V-05 断言
  sret 不覆盖存活实参）。
- Drop：首批 i32/bool record 无资源，无析构调用；退出路径无 Drop 代码（774
  interface-contract §5："i32/bool record 无拥有 buffer；清理规则不触达"）。

## 4. verifier 规则（前门；拒于 verify 层，native 调用计数 0）

| 规则 | 检查 | 码 |
|---|---|---|
| 名义类型 | 同结构不同名赋值/传参拒绝 | `verify.type-mismatch` |
| 类型含图 | 自环/互环/断开环全拒绝（迭代 worklist，探针 P1）；图不闭合（未知引用）拒绝 | `verify.record-cycle`、`verify.bad-reference` |
| 字段归属 | 路径逐段 owner（经标量续走/record 结尾当值用均拒绝，探针 P3） | `verify.field-owner` |
| 构造完整性 | 位置/命名构造缺/重/未知字段、类型不匹配 | `verify.ctor-arity`、`verify.type-mismatch` |
| 可变性 | let/view/receiver 写 → 拒；mut 参数写 = 唯二合法写入口 | `verify.immutable-write` |
| mut 排他 | 同一 local 两活跃 mut 借用（含经嵌套路径同根）→ 前门拒绝 | `verify.mut-conflict` |
| `.mut` 调用点 | mut 参数调用缺 `.mut` | `verify.ownership-required` |
| receiver 一次 | 方法调用 receiver 求值恰一次；随后实参源码序 | HIR 结构保证 + trace 断言 |
| 声明冲突 | 字段/方法/类型名重复、类型名被局部遮蔽后用作类型 | `verify.duplicate-decl`、`verify.shadowed-type` |
| 签名/身份 | record 参数/返回/字段的 TypeUse 界限；text 域类型不可漏入 | `verify.domain`、`bind.*` 既有族 |

诊断带真实 span 与原 token 文本；不拼接字符串制造唯一性。

## 5. 冻结诊断码表（771 新增；命名空间与既有 source.*/verify.*/bind.* 一致）

| 码 | 阶段 | 语义 |
|---|---|---|
| `source.record-decl` | text | record 声明语法错误（泛型/默认值/空体等，真实 parser 同类拒绝） |
| `source.unsupported`（扩展） | text | `mut fn`、`move`/`view`/`take`/`copy` 参数模式、record 参数外的 `mut` 等 |
| `source.syntax`（扩展） | text | 构造/路径/调用拼写错误 |
| `verify.record-cycle` | verify | 类型含图含环（自环/互环/断开环） |
| `verify.field-owner` | verify | 字段路径段归属错误（经标量续走等） |
| `verify.ctor-arity` | verify | 构造字段缺/重/未知 |
| `verify.duplicate-decl` | verify | 字段/方法/同名声明冲突 |
| `verify.shadowed-type` | verify | 类型名被局部遮蔽后在类型位使用 |
| `verify.mut-conflict` | verify | 同对象多活跃 mut（774 冻结名） |
| `verify.ownership-required` | verify | mut 参数调用缺 `.mut`（774 冻结名） |
| 既有复用 | — | `verify.type-mismatch`/`verify.immutable-write`/`verify.domain`/`bind.*`/`source.*`/`backend.*` |

机器层状态码：771 record 无堆无 buffer，**不新增**机器负码（774 -1..-11 不触及）。

## 6. 支持预算与 whole-path probe 收据（V-09 冻结）

**预算冻结**：类型含图深度 1024 层（构造/排序/verify/layout 全路径）；混合字段路径
4096 段；均为**验证规模而非语言上限**——无固定语法深度排除；环境耗尽（如实际栈配额
低于探针基线）必须以受控诊断失败，不得截断合法嵌套语法。

**probe 收据**（worktree `docs/reports/771-record/probe/`，rustc --edition 2021 -O，
exit 0）：

```text
PROBE-OK P1(cycle:1024chain+self+mutual+disconnected) P2(size=4100 align=4 checked-u64) P3(path4096 owner-rule eval-order copy)
```

- P1：迭代 worklist 环检测——1024 深无环链通过；自环/互环/断开环（A→B→C→B 与
  D→E 并存）全部定位拒绝；零递归（实现形态裁定：verify 的图检查**必须**用此形态）。
- P2：1024 链布局 size=4100/align=4（In1=8，每层 +4）；checked u64 加法在
  u32::MAX 哨兵边界不回绕。
- P3：4096 段路径平坦偏移解析；经标量字段续走拒绝（owner 逐段规则）；命名构造
  源码序求值 trace（`ab`，禁声明序重排）；`var b=a` 拷贝后写 b 不改 a。
- ABI 半边引证 774 PE 探针（I-10）：rec_view/rec_mut/rec_out/rec2_view 已证
  ABI-D/E + 嵌套布局 + out-slot 在真实 COFF/PE 可行；管线内 PE 证明归 T-05
  `record_native.rs`。
- 文件 SHA256：`whole_path_probe.rs` =
  `e167d970aa3413ef88e3d7797b7afa3f3ca105e05f5301d46ed9275445b2dd36`，
  `run-output.txt` =
  `3561cb4c9a6e1ba740144133aba6e8df8dde3b1572210c5d50bb9b886a1e5d90`。

**Drop 裁定**：i32/bool record 无资源 → 无析构、无清理调用、无泄漏面；"Drop 全路径
无栈崩溃"由"无 Drop 代码 + 布局平坦化（无递归值树）"结构性满足（V-09 断言面）。

## 7. Case 映射（V-01..V-10 → 实际入口；T-02..06 物化）

| Case | 实际入口 | 关键断言（区分性） |
|---|---|---|
| V-01 | `tests/record_hir.rs`：直接 Atom 文本经 descriptor::bind（record 域）三维身份/未知根/错域/text 字段混入 | 正确绑定 + `bind.*` 受控拒绝；不产 Checked |
| V-02 | `tests/record_source.rs` + `record_native.rs` PE：异质字段/嵌套/`var b=a` 写副本 | 返回值 + trace 精确；原对象不变 |
| V-03 | `record_hir.rs`：同构异名、前向引用、自环/互环/断开环 | 异名拒绝；前向嵌套通过；含环全拒 + 无布局递归崩溃 |
| V-04 | `record_hir.rs` 程序化 Bundle 变异（TypeId/FieldId/path owner/签名 params/result/receiver） | 真实 verify 拒绝、无 panic；合法兄弟例通过 |
| V-05 | `record_native.rs` PE：混合参数/命名实参重排/嵌套调用/record 返回/递归标量+record | 独立值 oracle；receiver 一次 + 源码序 trace；out-slot 不覆盖存活实参 |
| V-06 | `record_source.rs`：mut 局部/mut 自由函数/let/view/receiver 写/同对象双 mut | 合法 mut 只改目标；非法在 source/verify 拒绝并带 span |
| V-07 | `record_source.rs` + 直接 HIR 同义：声明 left/right，源码序 right-then-left + mark_a/mark_b | trace `ab` 按源码序；字段按名落位；缺/重/错字段确切码 |
| V-08 | `record_source.rs`：static new、readonly self/.field、同名遮蔽矩阵、错 receiver | 绑定正确 + 冲突/错类确切 source/span/code |
| V-09 | `record_hir.rs`/`record_source.rs`/`record_native.rs` 深输入 + malformed partial + 原 scalar 深输入重跑 | §6 预算内全路径无栈崩溃；耗尽受控 |
| V-10 | `scripts/verify-ac-record-771.ps1`：新 gate + 741/745/772 旧 gates（770 若已合入联跑其 gate）+ 失败 build 三件 SENTINEL | 实际 gates 通过；旧制品字节不变；新基线 hash |

## 8. 共享热点与合并顺序登记（774 interface-contract §6 采纳）

- 共享热点（两分支都改；语义定义一处 = 774 decisions/interface-contract）：
  `src/source/`（ast/lexer/parser/resolve/typecheck/adapter/mod）、`src/hir.rs`、
  `src/descriptor.rs`、`src/verify.rs`、`src/native.rs`、`src/main.rs`。
- 冲突解法：优先以 774 合同语义为准；非文本合并冲突（身份/签名/能力门不兼容）
  → 暂停依赖实现回 new 修订，不用宽松 verifier/跨域 fallback 绕开。
- 顺序：774 已合入 ✓ → 770/771 并行（本计划基面 = 774 后 master，无 770 代码）→
  先完成者先合入 → 后完成者在**新整合基线**重放自身全门禁 + 首个已合 profile
  回归面（770 先合 → 771 联跑 770 gate；771 先合 → 770 联跑本计划 gate）。
- text-record 组合：后续 owner；两分支单独通过不构成组合验收（第二合入者重新绑定
  受审基线，plan §10.3）。

## 9. T-02..T-05 落点清单（实现范围冻结）

| Task | 文件 | 增量 |
|---|---|---|
| T-02 | `src/hir.rs` | TypeKind::Record、ExprKind::{RecordCtor,FieldGet}、PlaceKind::FieldPath、DefKind 方法归属、param_mode、HirDomain::Record（`auto.hir.record.draft/1/record-i32-draft`） |
| T-02 | `schema/record.atom` | record 域 descriptor（enums 增 record_ctor/field_get/field_path/param_mode cases；node type 增字段表） |
| T-02 | `src/descriptor.rs` | record descriptor 注册 + bind 分域 + write_canonical |
| T-02 | `src/verify.rs` | `verify_record` 域入口：含图/名义/字段/构造/可变性/mut 排他（§4 全表） |
| T-02 | `tests/record_hir.rs` | V-01/03/04 |
| T-03 | `src/source/` 七模块 | `type` 声明/构造/字段路径/方法调用/static/readonly self/mut 参数（§2 全文）；profile `auto.source.record.draft` |
| T-03 | `fixtures/record-{source,hir}` + `tests/record_source.rs` | V-02/06..09 源码腿 |
| T-04 | `src/native.rs` | Vt::Record、布局表（§3）、ABI-D/E lowering、out-slot 返回、值拷贝、构造临时 |
| T-05 | `src/main.rs` + `tests/record_native.rs` | `--source-profile auto.source.record.draft` CLI 面；source/HIR PE（V-02/05..09） |
| T-06 | `scripts/verify-ac-record-771.ps1` | §7 V-10 门禁 + case manifest |
| T-07 | `docs/reports/771-record/{verification,proposed-spec-delta}.md`、README、inputs hash | 收据 |

**明确不在范围**（拒绝面即验收面）：str/Str/char 字段、List/enum/Option/Result、
泛型/继承/接口/ext、递归/指针字段、move-only/堆 record、可变实例方法、unit 返回、
use/跨模块、全局变量、生产 ABI、text-record 组合。
