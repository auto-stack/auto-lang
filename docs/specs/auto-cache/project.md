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

## 探针现状差距与实验/迁移边界（PLAN-783，实测清单）

1. **`BlobStore::put` 固定 tmp 名**：同哈希并发写共享同一 `*.tmp` 路径——结构性
   风险（无外层协调时竞争失败,PLAN-783 R1 F-08 反例）。探针以 per-chunk
   LockFileEx 有界等待串行化规避并实测**重叠双进程**同键（同内容/异内容）行为;
   **探针证据不构成生产 BlobStore 原生并发安全的证明**——生产化需修复或上移该
   串行化。
2. **元数据删除 vs 物理释放**：现有 GC 两条入口（`gc.rs::run_gc` 与
   `lib.rs::AutoCache::run_gc`）按记录大小计,无 lease/图保活/崩溃恢复/物理
   释放收据。探针实现了 tombstone→门锁→trash→删除 + 级联删行 + 幂等恢复
  （含已提交孤儿行与 trash 残留的恢复趟）,作为迁移目标形态的参考实现
   （非生产代码）。探针 store 以 `auto_vacuum=INCREMENTAL`+checkpoint 使
   SQLite 主文件在回收后**真实收缩**,GC 收敛决策用**沉降后的物理总量**
  （orphan/tmp/trash 全计）——原 64/48MiB 水位总量合同在 512-churn 上实测达成。
3. **`fingerprint::compute_content_hash` 直哈原源码**——各阶段细粒度键
  （profile/caps/target/backend 成分化）在探针 identity.rs 有完整参照。
4. **迁移边界**：探针不并入任何生产入口;退役/迁移由后续计划决策,防第三套
   生产缓存。探针详情 [auto-ac/artifact-probe.md](../auto-ac/artifact-probe.md)。

> 来源：PLAN-783（r6 pass,aa8c666ca）;探针 `experimental/ace/crates/artifacts/`。
