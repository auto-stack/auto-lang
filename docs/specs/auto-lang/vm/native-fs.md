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
