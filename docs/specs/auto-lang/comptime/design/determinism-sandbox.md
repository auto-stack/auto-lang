# 确定性与资源限额（设计，未实现）

## 范围

plan-095 Part A 为编译期执行定下的确定性沙箱与资源限额设计，及其与代码现状的差距。
本文档几乎全部内容描述**尚不存在**的机制，保留它是为了给后续落地提供冻结的决策依据。

## 原则

编译期执行必须**可复现**：同源码同输出——这是缓存失效（hash）与可复现构建的前提（plan-095 "Why Deterministic"）。

## 确定性规则（设计）

| 操作 | 编译期模式 | 运行时 |
|---|---|---|
| `Time.now()` / `Random.*` / `File.read()` / `Process.spawn()` / `Env.get()` | ❌ 报错（E0402） | ✅ 允许 |
| 纯计算 | ✅ | ✅ |

承载机制设计为 `VmInterpreter.comptime_mode: bool` + `set_comptime_mode()/is_comptime_mode()`
（plan-095 Task 4.1 给出了字段与方法的完整签名）。

## 资源限额（设计）

```rust
CTEELimits {
    max_time_ms: 5000,              // 5 秒
    max_memory: 100 * 1024 * 1024,  // 100 MB
    max_recursion: 256,             // 256 帧
    max_native_calls: 10000,        // 1 万次 native 调用
}
```

越限报 `ComptimeError::ResourceLimit`（E0403）。动机是编译期无限循环/内存耗尽不应拖死编译器
（plan-095 Risks 表）。

## 现状差距（代码核对结论）

- `VmInterpreter` 中**不存在** `comptime_mode` 字段（grep 全 `src/` 仅命中 `comptime/mod.rs` 的文档注释
  与 plan 文本）——`mod.rs` 注释"Distinguishes compile-time vs runtime via `comptime_mode` flag"与代码不符。
- `CTEELimits` 类型不存在；`ComptimeError::NonDeterministic`/`ResourceLimit` 两个变体已定义但无任何抛出点。
- CTEE 内嵌的是普通 `VmInterpreter::new()`，编译期代码拥有完整 I/O 能力，且每次 `transform` 新建实例，
  与运行时 VM 天然隔离（这一点设计与实现一致）。

## 显式非目标

- 本文档不把该设计标记为"已实现"；overview 的 Status 为 partial 主要由此而来。
- 不设计二级缓存/指纹：确定性是缓存的**前提**，缓存本身归 AIE/AutoCache 模块（09-compiler.md 另述）。

> 来源: docs/plans/old/095-compile-time-execution-engine.md（Part A "Deterministic Execution"/"Resource Limits"、Task 4.1）、crates/auto-lang/src/comptime/mod.rs、crates/auto-lang/src/interpreter/（核对无该字段）
