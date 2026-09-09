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

## PLAN-591：methods pack 快路径 stale 通道与剔环类型级归因

- **快路径 stale 通道**（`compile_dep_methods`）：①manifest format 旧版拒用
  （format bump 一键失效）；②path 源**内容 hash + features 组合复合键**
  （`staleness_key`，源码变更/异 features 必重建；registry 源维持版本核对）。
  实勘动机：快路径此前只看 crate 版本，fixture 源码变更后陈旧 pack 永续复用。
- **剔环类型级归因**（`type_names_in_crate_errors`）：深模块路径类型
  （如 uuid 的 fmt 适配器）rustdoc 只给短名 → 生成器误径编译失败，错误文本
  无 `auto_` 符号——按 `crate::Type` 出现归因到 `auto__drop_<Type>` 类型级
  剔除，其余面可编译（DIV-DEP-17）。
- **to_cargo_line**：path 源渲染 features（此前漏发使 cfg 字段变体永不生效）。
- 自由函数 wrapper 缓存键 v3_{数量} 弱失效维持（P592-D1）+ 并发共享 sandbox
  竞态观察（P591-D3）——均登记 KNOWN-DEBT。

> 来源：PLAN-591（r1，9b3639122）；Anchor：crates/auto-cache/src/methods_pack.rs、sandbox.rs。
