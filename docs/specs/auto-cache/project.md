# auto-cache

> **Status**: active
> 路径：`crates/auto-cache`  | 技术栈：Rust（rusqlite / blake3 / libloading / syn）

全局构建缓存：SQLite 存储 + blake3 内容指纹，支持跨工程产物共享；含 Rust FFI 沙盒执行。

## 目标与范围

- 以内容指纹（blake3）为键缓存构建产物，跨工程共享（SQLite 存储于用户目录）。
- 提供 GC、注册表（registry）、源码扫描（scanner）与签名扫描（sig_code，基于 syn AST）。
- Rust FFI 沙盒（sandbox，libloading 动态加载执行）。
- 与 auto-man（automan 模块）及转译流程（trans/aie_bridge）集成。
- 不做：不做构建调度本身（auto-man/builder）；不缓存语言编译中间态以外的通用文件。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| storage | SQLite 存储层 | active |
| fingerprint | blake3 内容指纹计算 | active |
| registry | 缓存条目注册表 | active |
| gc | 垃圾回收 | active |
| sandbox | Rust FFI 沙盒（libloading 动态加载） | active |
| scanner / sig_code | 源码扫描与 syn AST 签名扫描 | active |
| automan / trans / aie_bridge | 与 auto-man / 转译 / AIE 的集成桥 | active |
