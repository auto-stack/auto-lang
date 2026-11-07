# ACE 完整数值源码与四端一致语义

> PLAN-961；实现位于 `experimental/ace/grammars/auto-source.at`（numeric profile）、
> `core/src/source/cst/project.rs`（affix-num 解码/数值步进）、`core/src/source/resolve.rs`
> （同型/转换/移位规则）、`core/schema/numeric.atom` + `core/src/descriptor.rs`（schema 绑定）、
> `av-core/src/{mir,abc,mir_verify,av}.rs`（MIR 表示/执行）、`lowering`（HIR→MIR）、
> `native/src/lower.rs`（CLIF 发射）、`a2x-rust/src/{rt,artifacts}.rs`（RT_NUMERIC 运行时）、
> `a2x-c/src/runtime.rs`（C 运行时生成器）、`a2x-core`/`a2x-store`（通道/冷装配）。
> 本文件只记录**已验证**的支持、接口、拒绝与边界；未列入的能力不视为已支持。
> 既有 950/960 面（`ac.source.modules.scalar|packages/v1`）字节不变；i32/bool 旧子码路径不变。
> 语义基线为 959 分母 L-01/L-02/L-49 与本包 T-01 冻结表（计划 §9 收据），四端同一矩阵。

## 支持入口与身份

数值面经 `SourceProfile::Numeric` 显式源码 profile 进入：grammar `numeric` 裁剪（amp/vbar/caret
token 与数值类型关键字均 `profiles: [numeric]` 门控，旧 profile scanner 字节恒等）、
numeric-ops pratt 表（`& | ^` 同层，位阶 5）、`.shl(n)`/`.shr(n)`/`.as(T)`/`.to(T)` 方法形后缀步
（移位/转换**不入 pratt**：pratt 算子右操作数仅表达式、`>>` 与嵌套泛型闭合冲突——中缀 `as` 具名
延后）。消费入口 `ace_core::source::compile_numeric_source`；a2x 侧 `A2xSession` Numeric profile
冷装配；native 侧 driver 末位按面事实升级 Numeric profile。ctor-expr 不在 numeric 原子面
（聚合形归 962；closures 面预存缝 CI-closures-int 具名记录）。

## 类型与字面量

类型：`i8`、`u8`、`byte`（≡u8 别名）、`uint`（≡u32 别名）、`u32`、`i64`、`u64`、`float`（f32）、
`double`（f64，含 `f64` 别名拼写）。`int`/`i32` 维持已裁定 i32 宽度不变（禁止以 Rust 默认宽度
补语义）。

字面量（沿 legacy 词法纪律，affix-num 解码）：后缀无/`i`→int、`i32`/`i8`/`i64`、`u`/`u32`/`u8`/`u64`、
`b`→byte、`f`/`f32`→float、`d`/`f64`→double；前缀 `0x`/`0b`；`_` 数位分隔。浮点文本 `digits.digits`
（无前置/后置孤点），带点无后缀→f32。十进制文本按**目标宽度**正确舍入解析（f32 用 f32::from_str）；
解析溢出（得 inf）或整数超宽度 → 诊断 `source.literal-range`，不产生 inf 字面量。一元 `-` 作用于
字面量按折叠纪律（`-0.0` 得符号零位型；minus 感知 i64::MIN/i32::MIN 量级折叠）；NaN 无字面量语法。

## 运算语义（同型纪律；AV 为规范执行端，四端一致）

二元数值运算/比较要求两操作数**类型完全相同**；混合宽度必须显式 `as`/`to` 转换
（`source.type-mismatch`）。不采纳 legacy codegen 临时晋升矩阵（double>float>u64>i32 隐式插入），
具名差异。

整数族 T∈{i8,i32,u8,u32,i64,u64,byte}：
- `+ - *`：按宽度 checked，溢出→trap `IntOverflow`（checked trap 相位沿现行；legacy wrapping 是
  已登记差异 B13/B15）。
- `/`：除零→`DivByZero`；有符号 MIN/-1→`DivOverflow`；商向零截断。`%`：除零→`DivByZero`；
  有符号 MIN%-1→`RemOverflow`；余数随被除数符号。
- 比较：按符号性比较，→bool。
- 位运算 `& | ^`：同型整数；浮点/bool 拒绝。legacy 无位运算面——本表为新增冻结语义。
- 移位 `.shl(n)`/`.shr(n)`：两操作数同型；有符号 `>>` 算术右移、无符号逻辑右移；移位量 <0（有符号）
  或 ≥ 位宽 → trap `ShiftOutOfRange`（`av.trap.shift-out-of-range`）；移出位丢弃（值不 checked）；
  浮点移位拒绝。
- 一元 `-`：有符号 checked（MIN→`IntOverflow`）；浮点符号位翻转（0.0→-0.0、NaN 翻符号位）；
  无符号取负→诊断。

浮点 F∈{f32,f64}：
- `+ - *`：IEEE 754 逐位确定（NaN 传播按 IEEE、-0.0+(-0.0)=-0.0），不 trap。
- `/`：除数 == ±0.0 → trap `DivByZero`（沿 legacy DIV_F/DIV_D 引擎行为，非 IEEE inf）。
- `%`：fmod 语义；x % ±0.0 → NaN 结果**不 trap**（沿 legacy MOD_F/MOD_D 无零守卫）。
  该 ÷/% 不对称为 legacy 原样保留的具名差异。
- 比较：IEEE（NaN 与任何值不等/无序比较 false；-0.0 == +0.0 为 true）。

## 转换矩阵（`as` / `.as(T)` / `.to(T)`，数值目标）

8 数值型 i8,i32,i64,u8,u32,u64,f32,f64（byte/uint 别名）：
- 同型：恒等。
- 整→整加宽：符号扩展（有符号源）/零扩展（无符号源），不 trap。
- 整→整窄化或同宽变号（含 i64↔u64、i32↔u32）：**checked**，值不可表示→`IntOverflow`。
  不采纳 legacy `as` 截断/位重解释（TYPE_CAST_U32 位重解释、TYPE_CAST_I64 clamp），具名差异。
- 整→浮：可表示则精确，否则舍入到偶，不 trap。
- 浮→整：**checked**——NaN/±Inf→`FloatToInt`（`av.trap.float-to-int`）；截断值（向零）越目标
  范围→`FloatToInt`；-0.9→0 合法。四端先界检查后转换，C 端不落入 UB。
- f32→f64：精确加宽。f64→f32：舍入到偶；有限值舍入后超 f32 范围→`FloatOverflow`
  （`av.trap.float-overflow`）；NaN→静默 NaN（payload 不保证保留，具名记录）；±0 保持。
- 非数值目标（str 等）→ `source.unsupported`（972 职责）。

## MIR/wire 表示

- `MirType` 增 `I8/U8/U32`（wire tag 19/20/21）；布局 i8/u8 1/1、u32/f32 4/4、i64/u64/f64 8/8。
  `MirConst` 增 I8/U8/U32（tag 8/9/10）；浮点常量位精确 F32Bits/F64Bits。
- `BinOp::Num { op, ty }`（BIN subcode 12 + op 字节 + ty 字节；既有 AddI32..NeI32 子码 1..11 冻结，
  i32 算术/比较继续走旧子码保 golden 字节稳定；i32 位运算唯一编码走 `Num{…,I32}`）。
  NumOp∈{Add,Sub,Mul,Div,Rem,And,Or,Xor,Shl,Shr,Lt,Le,Gt,Ge,Eq,Ne}；NumTy∈八数值型。
- `UnOp::NegT { ty }`（subcode 3；NegI32/NotBool 1/2 不变）；ty∈{I8,I64,F32,F64}。
- `MirInstr::Conv { dst, src }`（opcode **0x3A**）；转换矩阵在 mir_verify 与四端执行一致实现。
- TrapKind 增 `ShiftOutOfRange`/`FloatToInt`/`FloatOverflow`；复用 IntOverflow/DivByZero/DivOverflow/
  RemOverflow。trap 相位维持运行时（未执行分支潜伏，B20 语义）。
- mir_verify：PLAN-820 数据 profile"无宽标量算术"门由本包**取代**——新数值面在
  data/owned/collections/dynamic profiles 准入；EqI32/NeI32 收紧为 (I32,I32)/(Bool,Bool)（原宽型
  准入缝具名关闭；宽型相等走 Num{Eq,ty}）。closures 族入口契约扩为数值标量参数/结果。
- compact_abc（av.core-i32/1 wire）维持拒收新面（子码 12/新 tag 解码即拒）；core profile 冻结不动。

## 四端发射（真实 dispatch，无 legacy fallback）

- **AV**：`av-core/src/av.rs` eval_num/eval_num_float/eval_conv 全矩阵（i64/u64 域数学、
  checked_reduce、num_trap_on；f32 原生精度、NaN/-0 位精确）——规范执行端。
- **native**（CLIF→COFF→PE 真实执行）：`LowerMode::Numeric`（= Legacy 行为 + 数值守者；无数值
  程序与 Legacy 恒等）。i8/u8/u32/f32 骑 I32 守者、i64/u64/f64 骑 I64；整溢出用 room 技术
  （加/减）与 smulhi/umulhi 高半检测（乘——MAX+1=MIN 型回绕不可由结果范围检出）；除/余零与
  MIN/-1 预守卫；移位量界检查；`%` 走 CRT fmod 导入；NaN/-0 位精确经 bitcast 守者往返。
- **a2x-rust**（文件集→真实 rustc）：RT_NUMERIC 运行时（各宽度 checked 整算术、移位界、fmod、
  转换矩阵、位算、字面量构造器；trap 类 `InvalidScalar` 通道）追加进每个 fileset。
- **a2x-c**（文件集→真实 MSVC cl /W4 /WX 零警告）：runtime.rs 按宽度/符号生成全套 helper
  （_mul128/_umul128 高半、i64/u64 room、窄宽度域范围检查、unsigned 源无下界检查防符号比较翻转、
  float→int 域检查 goto bad、typed result 通道按函数结果类型回传陷阱、显式窄化 cast 保 W4 清洁）。
- **金样/SHA pin**：a2c-hir-scalar-921 三套金样按新真实发射重冻结；`a2x-c/src/admitted_text.rs` 与
  `abi/src/source_foreign.rs` 的 C 运行时 SHA256 双 pin 重钉为
  `bd260a8b7d62f2b2956488fdeabffcf027605da89608ff81865cf8a8151de12d`。

## 验证责任与边界

永久入口（仓内 fixture）：core `numeric_source`（10）/`numeric_hir`（4）、av `av_numeric_semantics`
（9，含 0.1f32/0.1d 最近舍入位精确观测）、native `numeric_codegen_tier`（4，真实 COFF+PE 执行
独立 exit 断言）、a2x-worker `numeric_semantics`（Rust rustc 12/12 + C MSVC 12/12，独立
手算 oracle）。固定组：`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime`。

具名边界（非静默回退）：semantics 查询通道保持有限推断 profile——数值注解=具名拒绝
`numeric-annotation`（infer 面扩容属后继计划）；CTE 值面维持 I32/Bool（新类型进 CTE 为具名受控
拒绝）；comptime av_driver 数值常量执行面可执行但 admission 未扩（具名）；聚合/泛型/动态/控制
效应面归 962–966；i32 裁定与 diff-old-int-i64 差异沿用。其他平台/资源/标准库族不据此声明支持。
