# native-fs（fs.* 原生族失败语义）

> **Status**: implemented（PLAN-659 SD-01，2026-09-19）

## 范围

VM 臂 `fs.*` 原生 shim（`crates/auto-lang/src/vm/native.rs`）的**失败返回语义**
契约。只约束"失败时返回什么"，不重复各操作的成功语义。

## 契约

- **`fs.canonical(path)`：失败返回 `""`**（`shim_fs_canonical` 失败臂
  `unwrap_or_default`）。
  - 权威依据：语料哨兵——027-file-manager NavTo 以 `can == ""` 判失败
    （"无法打开" toast 分支）。语料语义优先于 shim 实现便利；曾有的
    `unwrap_or(path)`（返回原路径）是 shim 实现侧漂移，会使哨兵分支
    不可达。
  - 族内一致：`Path.canonicalize`
    （`vm/ffi/stdlib.rs::shim_path_canonicalize`）同 `unwrap_or_default`
    ——失败返空是 fs 族统一语义。
  - 成功路径剥 Windows `\\?\` 扩展长度前缀（PLAN-016 T-05：面包屑/
    地址栏/拼接面不消费该形态）。
- 其余谓词型（`fs.exists` / `fs.is_dir` / `file.exists` 等）失败返回
  false——布尔哨兵自洽，不另行约定。
- 读型（`fs.read_dir` 等）失败抛 RuntimeError（中止 handler），非静默
  返回。

## 非目标

- 语料侧 `fs.exists` 前置门控形态（P642-D14② 的备选修向——因全语料
  零 `unwrap_or(path)` 依赖而未采纳，选边依据见归档计划 659 §5.1）。
- vue 臂 fs 语义（浏览器 stub 面另有契约，不在本模块）。

## 回归

`vm_tests.rs::test_plan659_fs_canonical_failure_returns_empty_sentinel`
（非法字符路径确定性失败，golden `[]`）。

## 动态整数列表字节写入（PLAN-988）

`File.write_bytes` 的 VM FFI 参数转换同时接受原生产生的
`ListData<i32>` 与语言数组、`List.new/push`、JSON 数组产生的
`ListData<Value>`。后者仅接受整数标签（Int/Uint/I64/USize/Byte/I8/U8），
并在进入写文件函数前检查 i32 范围；浮点、布尔、字符串等非整数或越界
整数报错，不跳过、不归零，不覆盖已有目标文件。非法句柄仍具名拒绝。

合法 0..255 字节逐字保持；写入函数既有 i32→u8 转换行为不因本修复改变，
这不是对任意范围整数新增字节合法性保证。该参数转换也用于 TCP/UDP 的
字节写入，消费者验收须分别覆盖，不以文件单测推断网络行为全部通过。

永久回归：`vm::ffi::convert::plan988_integer_bridge_tests`，含源码数组/
动态列表/JSON→真实文件、全256值、非法元素拒写与原文件保持。

Rust生成消费：嵌入式 `a2r_std::fs::write_bytes_list` 同时接受生成的整数
`Vec<i64>` 与JSON值列表；所有元素先验证为可读整数，再打开目标。非法
非整数返回既有失败码-1，目标不变；保留该Rust帮助函数原有i64取低8位
行为，不借本修复改变范围外整数兼容性。VM FFI仍按其Vec<i32>签名检查
范围。直接空数组在File.write_bytes调用点生成明确整数Vec类型，避免泛型
推断失败。0..255的承诺在两端一致；其他范围不宣称跨端新增等价。
