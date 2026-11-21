# comptime（编译期求值）

> **Status**: partial（语句级 `#if`/`#for`/`#is`/`#{}` 已实现并接入全部编译管线；表达式级 `#{expr}` 的编译期替换、确定性沙箱与资源限额未落地，见"已知坑"）

## 职责

编译期执行（Compile-Time Execution）：在 parse 之后、type-check/codegen 之前对 AST 做 Stage-1 变换——
`#if` 条件编译裁剪、`#for` 循环展开、`#is` 编译期匹配、`#{}` 求值替换。求值复用现有 `VmInterpreter`，
不另造解释器。设计哲学：**显式优于隐式**——`#` 前缀让编译期代码一眼可辨（docs/design/raw/compile-time-execution.md）。

## 现状

- CTEE 模块（plan-095，✅）位于 `crates/auto-lang/src/comptime/`，仅 `mod.rs` + `transformer.rs` 两个文件，
  核心类型 `CTEE` 内嵌一个 `VmInterpreter` 实例和 builtins 表（`OS`/`ARCH`/`DEBUG`/`VERSION`）。
- 七处管线集成点均在 parse 之后立即调用 `CTEE::transform`：AutoVM 执行、测试、调试、`create_vm_from_source`、
  a2r session 路径（lib.rs 五处），以及 `transpile_c`、`transpile_rust`（trans/ 两处）。
- AST 节点在 `ast/comptime.rs`：`HashIf`/`HashFor`/`HashIs`/`HashBrace`；表达式级 `#{expr}` 解析为
  `Expr::Comptime(Box<HashBrace>)`（parser.rs:2337，2026-09-07 快照）。
- 错误类型 `ComptimeError`（error.rs:1164，2026-09-07 快照），诊断码 `auto_comptime_E0401`–`E0404` 等。
- 单元测试随 `transformer.rs` 内联（builtins、truthy、相等性、`compile_error`）；
  `test/comptime/` 为三级示例语料（plan-137），未接入自动化测试运行器。

## 关键入口

- `crates/auto-lang/src/comptime/transformer.rs:CTEE` — 引擎主体（builtins、target_os/arch、内嵌 VM）
- `crates/auto-lang/src/comptime/transformer.rs:CTEE::transform` — Stage-1 变换入口（逐语句重写 `Code.stmts`）
- `crates/auto-lang/src/ast/comptime.rs:HashIf` / `HashFor` / `HashIs` / `HashBrace` — comptime AST 节点
- `crates/auto-lang/src/error.rs:ComptimeError` — 编译期错误（E0401–E0404）
- 集成点：`crates/auto-lang/src/lib.rs:execute_autovm_with_path`、`lib.rs:test_code`、`lib.rs:debug_file`、
  `lib.rs:create_vm_from_source`、`lib.rs:trans_rust_with_session`、
  `crates/auto-lang/src/trans/c.rs:transpile_c`、`crates/auto-lang/src/trans/rust.rs:transpile_rust`
- 表达式级 `#{}` 现状：`crates/auto-lang/src/vm/codegen.rs` 编译 `Expr::Comptime` 分支（2026-09-07 快照 10254 行，TODO 未做编译期替换）

## 使用示例

```auto
#if OS == "windows" {
    fn init() { init_win32() }
} else {
    fn init() { init_linux() }
}

// 语句级求值块：编译期算好，替换为字面量语句
#{ 1 + 2 }

// 编译期报错
#if ARCH == "unknown" {
    compile_error("Unsupported ARCH")
}
```

## 已知坑

- **表达式级 `#{expr}` 不做编译期替换**：CTEE 只处理语句级 `Stmt::HashBrace`；嵌在表达式里的
  `Expr::Comptime` 由 codegen 直接编译内层表达式、运行时求值（vm/codegen.rs:10254 附近（2026-09-07 快照），代码内 TODO 自述）。
  结果数值正确，但"编译期算好"不成立；`test/comptime/` 多数示例因此实为运行时等价（plan-137 文中已注明）。
- **`comptime_mode` 标志不存在**：`comptime/mod.rs` 文档注释声称用该标志区分编译期/运行时，
  但 `VmInterpreter` 源码中无此字段（plan-095 Task 4.1 设计未落地）；确定性沙箱、`CTEELimits` 资源限额同样未实现。
- **`#for` 仅支持整数上界**：可迭代值只认 `Int`（展开 `0..N`）；`Array` 分支返回空Vec并标注
  "needs proper array iteration"（transformer.rs `value_to_iter`）。
- **builtins 只在裸标识符时命中**：`eval_expr` 仅当整个表达式是 `Expr::Ident` 时查 builtins 表；
  复合表达式一律 `format!` 成源码交 `VmInterpreter` 求值，该 VM 环境未注入 builtins（transformer.rs:246-269）。
- **值回写能力有限**：`value_to_expr` 只覆盖 Nil/Int/Bool/Str/Float，其余值（数组、对象）退化为 `Expr::Nil`；
  `Expr::I64` 求值截断为 i32（注释自述 "Truncate for now"）。
- **`compile_error()` 拦截面窄**：仅当调用直接出现在 comptime 条件/目标表达式位置时触发（transformer.rs `eval_expr` 开头）。

## 蒸馏来源（Phase 1）

- `docs/design/09-compiler.md`（"Compile-Time Execution (Comptime)" 节）
- `docs/design/raw/compile-time-execution.md`（定稿设计文档 v1.0，语法与原理的主要来源）
- `docs/features/comptime-metaprogramming.md`（注意：其 `comptime fn` 语法与实现不符，见蒸馏报告分歧清单）
- `docs/plans/archive/095-compile-time-execution-engine.md`、`docs/plans/archive/137-comptime-examples.md`
- `test/comptime/`（示例语料）
- 代码核对：`crates/auto-lang/src/comptime/`、`ast/comptime.rs`、`error.rs`、`lib.rs`、`trans/`、`vm/codegen.rs`

## 普通 AV 的编译期执行（PLAN-1091）

本节适用于普通 AV 源码入口 `ac-av/language-source-run-v1`。装配先处理项位和语句位选择，源查询绑定包闭包并产生具体实例。source-cte 执行 `#{expr}` 的受检调用闭包，经 MIR/ABC 验证和 AV 执行得到冻结标量，再用原位常量替换需求。受检运行代码中已无需求调用。只读包常量可在 CT 内求值，无权读取或修改运行时的可变存储。

`int/i32` 和 `bool` 可回灌。文本或组合字面量作为冻结结果会返回 `cte.type/value-domain`。CT 继承单根时限、AV 帧/资源预算和取消分类，外部作用不获运行授权。错误定位为需求的原源码跨度。管理存储用源码/闭包身份、目标事实、cap/provider 和 AV 版本区分执行页。按原输入重开可读取已验证的成功页，失败需求不能覆盖该页。

实现、复现和原 AutoVM CTEE 的适用范围见 [PLAN-1091](../../../plans/archive/1091-av-parity-comptime-and-macros.md)。
