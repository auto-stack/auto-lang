# PLAN-767 T-01 设计收据：AC 标量计算扩面（scalar-i32 profile）

- plan_revision: 2（含 §5.2.1 条件 for；合同 §5/§6 已冻结的可观察规则不在本文更改范围）
- 用户已指令实施当前 r2 合同（2026-10-12「实施它」），T-01 依赖满足。
- worktree: `D:/autostack/.wt/lang-767/auto-lang`，branch `plan-767-dev`
- base_commit: `a9225e8de6e13cc0bd0c519e7c4b380c7276ebfa`（v0.6-dev；规划基线
  `c64b71353e` → HEAD 仅计划簿记 diff，已核对）
- inputs 取证：`docs/plans/attachments/767/inputs.json` 全部条目 SHA256 复核零漂移
  （2026-10-12，本 worktree 内复核）。
- 本收据冻结节点字段、profile 随行 API、注册选择、旧 verify 兼容、canonical
  writer 身份冲突、诊断码与 case 清单。T-02..T-06 按此物化。

## 1. 身份与注册

| 域 | schema | revision | profile | descriptor 文件 |
|---|---|---|---|---|
| core（旧，不动） | `auto.hir.core.draft` | 1 | `core-i32-draft` | `schema/core-i32.atom` |
| scalar（新） | `auto.hir.scalar.draft` | 1 | `scalar-i32-draft` | `schema/scalar-i32.atom`（新增） |

- 源码 profile：旧 `auto.source.core-i32.draft` 保持；新增
  `auto.source.scalar-i32.draft`。CLI `--source-profile` 只接受这两个字面量，
  缺失/未知仍是用法错 2；流水线拒绝 1；成功 0（继承 745，不改）。
- `lib.rs` 注册表：`descriptor()`（core，既有名不变）+ `scalar_descriptor()`（新），
  均为 OnceLock 静态解析。新增 `bind_document(source)`：先 `atom_text::parse`
  读取根节点 `schema/revision/profile` 三字段，在注册表内按三重身份选择
  descriptor 再走既有 `descriptor::bind`；未知三重身份 → `bind.descriptor-unknown`
  （见 §6）。`main.rs` 的 `check`/`build`（HIR 文本模式）改用 `bind_document`；
  既有 `bind_source`（core 便捷入口）保持原语义，供旧测试使用。
- `Descriptor` 不内嵌域判断逻辑；`hir::HirDomain` 由 schema/revision/profile
  三重身份映射（仅 `auto.hir.scalar.draft/1/scalar-i32-draft` → Scalar，其余
  一律 Core——保守方向：未知身份永不升级为新 profile）。

## 2. HIR 节点与字段（hir.rs）

```rust
pub enum HirDomain { Core, Scalar }            // PartialEq/Eq，随 Bundle 走
pub struct Bundle { pub domain: HirDomain, pub requires, pub modules } // 新增首字段

pub enum BinOp {          // as_str
    AddI32,               // add_i32   ─ arithmetic，要求 overflow: trap
    SubI32,               // sub_i32   ─ arithmetic，要求 overflow: trap
    MulI32,               // mul_i32   ─ arithmetic，要求 overflow: trap
    DivI32,               // div_i32   ─ 固有 trap（0 / MIN/-1），无 overflow 字段
    RemI32,               // rem_i32   ─ 固有 trap（0 / MIN%-1），无 overflow 字段
    LtI32, LeI32, GtI32, GeI32, // lt_i32/le_i32/gt_i32/ge_i32，无 overflow
    Eq, Ne,               // eq / ne（i32×i32 或 bool×bool → bool），无 overflow
}
pub enum UnaryOp { NegI32, NotBool }           // neg_i32 / not_bool
pub enum LogicOp { And, Or }                   // and / or
pub enum ExprKind {
    ReadLocal{..}, Constant{..},
    Binary { op: BinOp, overflow: OverflowPolicy, lhs, rhs },  // eager
    Unary  { op: UnaryOp, operand: ExprId },                   // eager
    Logical{ op: LogicOp, lhs: ExprId, rhs: ExprId },          // lazy（短路）
    Call{..},
}
```

- 语义相等（PartialEq）继续排除 span；`domain` 参与相等（不同域的 Bundle 不相等）。
- 旧 core 域文本不可能绑定出新 op/节点（core descriptor 的 enum/branch 不含）；
  程序化混入由 verify 的 profile 检查拒绝（§4）。
- `schema/scalar-i32.atom`：core-i32.atom 全量复制 + enum 扩充（BinOp 11 例、
  `UnaryOp [neg_i32, not_bool]`、`LogicOp [and, or]`、ExprKind + `[unary, logical]`）
  + expr 节点新增字段 `operand: ExprRef` + 分支：
  - `expr.binary:{add,sub,mul}_i32`：require `[type, op, overflow, lhs, rhs]`；
  - `expr.binary:{div,rem}_i32`：require `[type, op, lhs, rhs]`，forbid `overflow`；
  - `expr.binary:{lt,le,gt,ge}_i32` 与 `expr.binary:{eq,ne}`：同上（forbid overflow）；
  - `expr.unary`：require `[type, op, operand]`，forbid 其余；
  - `expr.logical`：require `[type, op, lhs, rhs]`，forbid 其余；
  - 根身份行换成 scalar 三重身份。

## 3. descriptor.rs（binder + canonical writer）

- `bind()` 产出 `Bundle.domain = desc 三重身份映射`。
- `bind_expr` 新增 `unary`（`op`/`operand` 字段，op 走 `UnaryOp` enum 校验）与
  `logical`（`op`/`lhs`/`rhs`，op 走 `LogicOp`）分支；`binary` 的 op 词表扩至
  11 例；`branch_and_check` 的 disc2 机制不变（binary:op 键已覆盖）。
- `write_canonical(bundle, desc) -> Result<String, Diagnostic>`（签名变更，唯一
  测试调用点同步更新）：先校验 `desc` 三重身份映射与 `bundle.domain` 一致，
  冲突 → `canonical.identity-conflict`（Stage::Bind）。scalar 输出头写
  `hir(schema: "auto.hir.scalar.draft", revision: 1u32, profile: "scalar-i32-draft")`。
- canonical 表达式输出：`unary`/`logical` 节点按全命名形式；binary 的
  `overflow:` 仅对 add/sub/mul 发射（div/rem/比较/eq/ne 不发）。

## 4. verify.rs（profile 随行 + 新规则）

- 入口拆分：`verify(bundle)` 兼容语义限定 core 域——`bundle.domain != Core` 时
  整体拒绝（`verify.domain`）；新增 `verify_scalar(bundle)` 要求
  `domain == Scalar`（否则同样 `verify.domain`）。`CheckedModule` 增加
  `domain()` 访问器（内部即 bundle.domain）。无按运算集合猜测、无静默升级。
- core 域新算子防线（belt-and-braces，程序化混入时）：core verify 遍历到
  scalar-only BinOp（sub/div/rem/le/gt/ge/eq/ne）或 `Unary`/`Logical` 节点 →
  `verify.profile-op`（消息指明节点/op 与所属 profile）。scalar 域接收全部新 op。
- 类型规则（scalar 域内）：
  - `sub/div/rem`：两操作数 i32、声明 i32（同 add/mul，含 overflow 字段存在性由
    binder 分支保证）；
  - `le/gt/ge`：两操作数 i32、声明 bool（同 lt）；
  - `eq/ne`：两操作数同为 i32 或同为 bool（混合 → `verify.type-mismatch`）、
    声明 bool；
  - `unary neg_i32`：操作数 i32、声明 i32；`unary not_bool`：操作数 bool、
    声明 bool；
  - `logical and/or`：两操作数 bool、声明 bool。
- 图结构与初始化走查扩充：`check_expr_graph` 的 operand 边、`expr_type` 的
  Enter/Exit、`walk_value_in` 的初始化检查均覆盖 `Unary.operand` 与
  `Logical.lhs/rhs`。短路不改变「每个 ExprId 恰好一个语法求值位置」：Logical
  的 lhs/rhs 各占一个 operand 槽，RHS 未运行不影响静态归属/无环/初始化判定。
- 迭代性：新增节点的三处走查全部保持显式栈（深链不递归）。

## 5. native.rs（Windows x64 lowering）

- eager 二元（左→右求值次序不变，迭代后序机器扩展）：
  - `sub`：`isub` + 既有 checked_add 式护栏（(b>=0 && a<min-b)||(b<0 && a>max-b)
    → trap70 路径）；
  - `div`：目标指令前显式护栏——`safe_b = select(b==0, 1, select(b==-1, 1, b))`；
    `raw = sdiv(a, safe_b)`；`q = select(b==-1, ineg(a), raw)`；
    `bad = (b==0) || (b==-1 && a==MIN)` → `branch_to_trap_on(bad)`；结果 q。
    b==-1 且 a≠MIN 时 q=-a 正确；a==MIN/b==-1 落 trap70，不产生硬件 #DE；
  - `rem`：同构护栏——`safe_b = select(b==0, 1, select(b==-1, 1, b))`；
    `raw = srem(a, safe_b)`；`r = select(b==-1, 0, raw)`；
    `bad = (b==0) || (b==-1 && a==MIN)` → trap70。MIN%-1 按 §5.2 固定规则 trap。
    商向零截断/余数与被除数同号由 x64 idiv 语义给出，oracle 来自 §5.2 数学规则；
  - `le/gt/ge`：Signed* IntCC 比较经 `icmp_bool`（uextend 规范 0/1）；
  - `eq/ne`：`icmp_eq`/`icmp_ne`（i32 位宽下 bool 0/1 与 i32 值共用）。
- 一元：`neg_i32` = `(a == MIN)` → trap70，否则 `ineg(a)`；`not_bool` =
  `bxor(a, 1)`（bool 已是规范 0/1）。
- 短路（lazy）CFG：迭代机器新增两步——
  1. `LogicalLhs{dest,op,rhs}`：lhs 槽值就绪 → `rhs_b/merge_b = create_block`，
     `def_var(res, lhs)`，`and: brif(lhs, rhs_b, merge_b)` / `or: brif(lhs, merge_b, rhs_b)`，
     seal rhs_b，switch rhs_b；
  2. `LogicalRhs{dest,var,merge}`：rhs 槽值就绪 → `def_var(res, rhs)`，
     `jump(merge)`，seal merge，switch merge，`slots[dest]=use_var(res)`。
  变量合流走 FunctionBuilder var（自动 phi）；RHS 子树只发射进条件块，
  无条件执行路径不触碰可能 trap 的操作。LHS 恰好求值一次；RHS 仅按真值执行。
  命名实参先按源码顺序求值再按 bindings 落位（既有 Call 步骤不变，逻辑表达式
  作为任意实参/返回/初始化/赋值/条件时由同一 eval_expr 机器处理，不特判 if）。
- `find_entry`/`capabilities_check`/事务发布尾部（`main.rs::finish_build`）域无关，
  共享不改语义；收据 `receipt_extra` 增加 HIR 三重身份行（core 源码构建在既有
  `source-profile`/`source-input-hash` 行后追加 `hir-schema/revision/profile`，
  contains 断言兼容）。

## 6. 诊断码（新码全表；既有码复用不列）

| code | stage | 场景 |
|---|---|---|
| `bind.descriptor-unknown` | bind | `bind_document` 遇到未注册三重身份（文本头可解析但注册表无匹配） |
| `canonical.identity-conflict` | bind | `write_canonical` 的 descriptor 与 bundle.domain 不一致 |
| `verify.domain` | verify | `verify`/`verify_scalar` 入口的域不匹配（bundle.domain ≠ 入口域） |
| `verify.profile-op` | verify | core 域 verify 遇到 scalar-only op / unary / logical 节点 |

源码域复用：旧 profile 拒绝新操作符仍走既有 `source.unsupported`（"operator `X`
is not part of this source profile"）；`!1`/`-true`/`false&&(1+2)` 走
`source.type-mismatch`；`true||unknown()` 走 `source.undefined`；一元 `+` 走
`source.unsupported`（新消息）。所有负例诊断锚定原 `.at` 冒犯 token/EOF
（UTF-8 注释与 CRLF 场景沿 745 投影机制验证）。

## 7. source 前端（T-03 范围）

- `source::SourceProfile { Core, Scalar }`（`as_str`/HIR 域映射）。`SOURCE_PROFILE`
  常量保持；新增 `SCALAR_SOURCE_PROFILE`。`parse_program/resolve/typecheck/
  compile_parts/compile_source` 增带 profile 的路径（旧签名 = Core 委托，旧测试
  不动）：
  - `parser::parse_program_with(src, profile)`；
  - `resolve::resolve_with(program, caps, profile)`；
  - `typecheck::typecheck_with(program, profile)`；
  - `adapter::compile_parts_with(src, path, caps, profile)` /
    `compile_scalar_source(src, path, caps)`。
- lexer：新增 `for` 关键字（进 `Kw::For`；`for` 原已在 RESERVED_KEYWORDS，
  名称不可用性两域一致，词法面其余零改动——core 下由 parser 拒绝）。
- ast：`BinOp` 扩至 13 例（+Sub/Div/Rem/Le/Gt/Ge/Eq/Ne/And/Or）；
  新 `UnaryOp { Neg, Not }` 与 `ExprKind::Unary`；`Expr`/`Block` Drop 与各迭代
  走查同步覆盖 Unary。
- parser（迭代优先级攀爬机扩展）：
  - **条件 for（r2 §5.2.1）**：scalar 下 `Kw::For` → 解析条件表达式 + `{ body }`，
    复用既有条件循环 AST 结构（`Stmt::While` 形：cond 每轮入口重判、continue 回
    条件、break 离最近循环；HIR 仍降为 `loop { if cond { body } else { break } }`，
    不新增低层控制流模型；合成节点 span 指回 for 关键字/原条件，条件不 hoist）。
    scalar 下 `Kw::While` → `source.unsupported`（指向 while 关键字）。core 下
    `Kw::For` → `source.unsupported`（既有 keyword 拒绝路径），core 的 while 照旧。
    缺条件、缺闭合、`for item in xs`、范围/初始化/更新子句显式拒绝并定位原
    token（`in` 是 Reserved → unsupported；`for ;`/`for x =` 等按缺条件/
    非表达式 syntax 拒绝）。
  - scalar 优先级（低→高）：`||`=1，`&&`=2，`== !=`=3，`< > <= >=`=4，
    `+ -`=5，`* / %`=6，一元 neg/not（primary 位置框架，值完成即归约），
    调用/分组最高；同级左结合。
  - 帧类型 `OpFrame::Bin { op, prec }`（携带 op，不再由 prec 反推）；
    新 `OpFrame::Unary(UnaryOp)`。
  - primary 位置 `-`：peek2 为 Int → 既有负字面量折叠（域 [-2^31,-1]，含
    `- 2147483648` 空格形式）；否则 scalar 下推 Unary(Neg)（`-x`、`-(e)`、
    `--x` 右结合链），core 下保持既有 unsupported 消息。
  - `!`：scalar 下推 Unary(Not)（core 保持 unsupported）；一元 `+` 两域均
    `source.unsupported`。
  - operator 位置 `-`/`/`/`%`/`>`/`>=`/`==`/`!=`/`&&`/`||`：scalar 进入二元
    归约（`a-1` 正确解析为二元减，不误读负字面量）；core 保持 unsupported_op。
  - `in_bracket` 判定改为 Paren/Call 才算括号上下文（Unary 同 Bin 属顶层）；
    anchored 行检查对 Unary 后随 primary 同样生效（`-\n x` 拒绝）。
  - 值完成即归约全部 pending Unary 帧（在任何二元/闭合处理之前）——保证
    `-a*b = (-a)*b`、`!(a&&b)`、`- -x` 语义。
  - 换行/CRLF、参数逗号、括号收口、EOF 缺操作数/缺右括号规则逐字继承 745。
- resolve：`Combine` 的 vt 按 op 全表（算术→i32；比较/eq/ne/and/or→bool）；
  `Unary` 任务（操作数槽先行）；Drop 覆盖 Unary。
- typecheck：算术两Operand i32；lt/le/gt/ge 两 i32；eq/ne 同类（i32/i32 或
  bool/bool）；and/or 两 bool；neg→i32、not→bool。
- adapter：`EKind::Unary(UnaryOp, usize)`、`EKind::Logical(LogicOp, usize, usize)`
  （AST `Binary{And|Or}` → Logical；plan_expr 迭代任务扩展）；scalar 头行写
  scalar 三重身份；bool 字面量继续 `0<1`/`0<0` 子图（不加 bool constant 节点）；
  绑定用 `scalar_descriptor()`，`compile_scalar_source` 用 `verify_scalar`。

## 8. 测试与 fixture 布局（V-01..V-11 物化清单）

新目标：`tests/scalar_hir.rs`、`tests/scalar_source.rs`、`tests/scalar_native.rs`
（均 `#![cfg(windows)]` 于 native/进程类；纯静态类不设 cfg）。fixture 根：
`fixtures/scalar-hir/{valid,invalid}`、`fixtures/scalar-source/{valid,invalid}`。

| Case | 实现载体 | 要点 |
|---|---|---|
| V-01 | scalar_hir + scalar_native + scalar_source | 同一 `a-b`/短路源码双 profile CLI（旧 source.unsupported/新成功）；core Atom 加 sub_i32 → bind 拒；未知三重身份 → bind.descriptor-unknown；core bundle 混入新 op → verify.profile-op；write_canonical 换壳 → canonical.identity-conflict；verify/verify_scalar 域错位 → verify.domain |
| V-02 | scalar_native（HIR PE）+ scalar_source | -7/3,-7%-3 等 6 组除余 oracle（源码包装成 `if r != want { return 1 }` 自断言 exit0；HIR 域直接 exit=值）；不以 PE 负 exit 低字节当值 oracle |
| V-03 | scalar_native | MAX+1/MIN-1/MIN 取负/MIN/-1/MIN%-1/1/0/1%0；前驱 mark_a → exit70 且 stderr 精确 `a`；未执行分支无 trap |
| V-04 | scalar_native | false&&rhs、true||rhs（跳过：exit0/无 b/无 trap）；true&&rhs、false||rhs（执行：精确一次 b 或 trap70）；RHS 为真实 bool 函数 |
| V-05 | scalar_native（源码 PE） | 逻辑式入 return/let/assign/**条件for**/call/命名实参；循环 ≥2 轮每轮重算；trace 顺序不因命名绑定重排 |
| V-06 | scalar_source | MIN/MAX/负数全序；bool 四组 eq/ne；优先级组合；混合 bool/int、`!1`、`-true`、`true||unknown()`、`false&&(1+2)` 静态拒绝（未执行 RHS 仍查名/类型） |
| V-07 | scalar_hir | 程序化 Bundle 变异：错类型 unary/logical、共享 Expr、自环/二节点环、跨 body 引用、旧域新 op → verify 拒绝且无 Checked；输出后重 verify 不沿用旧凭证 |
| V-08 | scalar_source（正例 PE 于 scalar_native） | `a-1`、负 literal 邻接、二元实参逗号、跨行/CRLF、缺操作数/缺右括号、混合优先级；正例真实 PE；畸形 source.syntax 掉帧检查 |
| V-09 | scalar_source（1 例 PE 于 scalar_native） | 16384 项逻辑链→Checked（含 Drop）；20000 层 `!`；4096 层逻辑/调用混合 + 缺闭合反例；1000 层混合至少一例真实 PE |
| V-10 | scalar_native（源码 PE） | 新 profile 成功收据含 source-profile + hir 三重身份 + input-hash；旧成功制品（SENTINEL 内容）后新 source/HIR 拒绝 → 三件原 hash 不变；沿用既有事务执行器不重造 |
| V-11 | scalar_native（源码 PE） | Euclid gcd(48,18)=6（`for y != 0`）；十进制计数 12345=5；`is_digit` i32 码点比较改写；exit0 + 独立值断言；标注非原 lexer 编译 |
| V-12 | scalar_source + scalar_native | 新 profile `for y != 0` 正确降级（gcd=6）；初值 false 零次执行；嵌套 break/continue；带 trace 短路条件每轮重验（trace 次数精确）；新 for 接受/while 拒绝、旧 while 接受/for 拒绝；非 bool 条件/未闭合/for-in/初始化子句受控拒绝且原 token 定位 |

执行器/时限沿用：PE ≤60s、结构负例外层 5s、深结构 60s、构建门禁 600s。

## 9. 门禁（T-06）

- 迭代：`cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets`
  + `cargo fmt -- --check` + 三个 scalar 目标。
- 交审前一次完整：`cargo test --manifest-path experimental/ac-core/Cargo.toml --locked`
  + `scripts/verify-ac-scalar-767.ps1`（新增；case manifest + 串行真实 PE +
  收据落 `target/ac-verify-receipts/scalar-767`）+ 既有 `verify-ac-741.ps1`、
  `verify-ac-source-745.ps1` + `python scripts/acc_inventory.py --check
  --require-decisions`。
- 不触 `crates/`、根 Cargo/配置/旧 auto-lib；root 档不跑（AGENTS Category A/B 边界）。

## 10. 静态 preflight（对合同 §5/§6 逐项）

- §5.1 显式身份/兼容：注册表 + Bundle.domain + 双 verify 入口 + writer 冲突
  拒绝 + 收据追加身份行——均不改变旧域可观察行为（旧测试零修改预期通过）。
- §5.2 运算规则：除/余/一元/比较/eq-ne/短路的数学 oracle、字面量域、
  `a-1` 二元性、`-(2147483648)` 拒绝（孤立幅值超域在字面量检查点拒绝）、
  `-(-2147483648)` 可构建运行 trap70（Neg 于 Constant(-MIN)）——§5 native 规则
  与 §7 parser 规则一一对应。
- §5.2.1 条件 for：AST 复用/降级路线、两域 for/while 隔离与拒绝定位均落入
  §7 parser 与 §8 V-12；旧 profile while 行为零改动。
- §5.3 HIR/verify/native：新节点三走查迭代；短路 CFG 变量合流；div/rem 显式
  护栏不逃硬件异常；命名实参顺序；无递归栈回归。
- §6.1 矩阵：V-01..V-11 全部有载体（§8）；oracle 来源 §5.2/真值表。
- 禁改清单：未改可观察规则、未升级后端（Cranelift 0.126 lock 不动）、未触
  生产 crates、未加堆数据/bool constant 节点、未接旧 AST。
