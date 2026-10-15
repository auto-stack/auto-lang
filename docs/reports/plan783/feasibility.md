# PLAN-783 可行性收据(T-01 冻结决策 + 实测)

日期:2026-10-15(work 阶段,基面 `ec934b000`,分支 `plan-783-dev`)。
探针:`experimental/ac-artifact-probe/`(独立 Cargo workspace,`[workspace]` 空表)。
本报告为 T-01 冻结决策的落地收据;实测数字见 [perf-report.md](perf-report.md)。

## 1. 冻结决策(T-01)

| 决策点 | 冻结结果 | 依据/实测 |
|---|---|---|
| 依赖 | `auto-ac-prototype`(path ../ac-core)、`auto-cache`(path ../../crates/auto-cache,仅隔离适配 `BlobStore`)、`blake3 1.5`、`rusqlite 0.30 bundled`、`serde/serde_json 1`、`windows-sys 0.59` | 全部仓内已用版本,离线解析一次通过;独立 `Cargo.lock` 已生成并提交 |
| 块层 | `auto-cache::storage::BlobStore` 经隔离适配复用:同 chunk 跨进程写由 `locks/chunk-<id>.lock`(LockFileEx)串行——**实测**复现了固定 tmp 名并发风险的存在面(同哈希写共享同一 `*.tmp` 路径),不加锁必然互毁;加锁后 0 复现 | V-07 |
| 元数据层 | SQLite(WAL + busy_timeout 8s);artifacts/chunks/edges/roots/leases/txns 六表;发布=单事务 | 三故障点恢复全部幂等(V-08) |
| HIR codec | 二进制定长标签格式(v1):core+scalar 受限可执行子集;解码全索引边界校验;chunk 框架(48B 头+blake3)自带校验 | 14 项 lib 单测 + V-02 |
| 布局 | A=32B 定长记录表+payload 独立页流(算术索引);B=变长记录+10B 目录项分页;页 64KiB 内容寻址,版本=页清单,未改页逐块共享 | V-03(≥128MiB/布局实测写入) |
| 发布协议 | staging 写入 →[F1]→ chunk 锁+CAS 投放 →[F2]→ SQLite 单事务;GC tombstone →[F3]→ 独占门锁→trash→删除 | `AC_PROBE_FAULT` 注入,`std::process::abort()` 模拟硬崩 |
| 子进程 | 60s 默认截止(测试经 `AC_PROBE_RUN_DEADLINE_MS` 收紧);Job Object 树杀;管道并发排干 | `subproc.rs`;挂死 exe 用例 3s 内受控失败 |
| 目标面 | 仅 `x86_64-pc-windows-msvc`(宿主);其余受控拒绝且零产物 | V-05 |

## 2. 最小纵线可行性结论(T-01 门槛)

四条 core oracle fixture(p01=14/p04=3/p12=0/p13=70)全部通过:

1. **fresh**:真源码 → `compile_parts_with` → 真 verify → 真 `lower_object` → 真 `link_object_staged` → 运行,exit 逐条相符,stdout/stderr 空。
2. **对象持久化 + 新进程只链接执行**:对象 chunk 落盘后,新进程(且**源文件已被移除**——禁调失败注入)仅 link+run 即复现 exit;parse/bind/verify/lower 计数全 0(V-04)。
3. **二进制 HIR 恢复 → 重 verify → native**:purge 对象层后跨进程恢复二进制 Bundle(`hir_restore=1`),真 verify 通过后重新 lower/link/run,payload 无 `hir(`/`module(` 文本标记——非文本 roundtrip(V-04 + lib 单测)。
4. **scalar 兼容控制**:同一 core∩scalar 面源码在 scalar profile 下全层 miss 重建、语义一致(V-05)。

结论:**二进制 HIR 恢复/公开 API/旧 BlobStore 并发语义三项前提全部成立**,T-01 有界可行性门槛通过,无反例需要回规划。

## 3. 关键实测发现(work 阶段)

- **A10 回执内容必须是链接输入的纯函数**:首版把 HIR provenance 放进回执字节,hit 路径与全建路径产生不同内容 → 同键 Conflict。修正:provenance 走图边(metadata),回执内容=对象 chunk+target+backend+entry+producer。这是"内容身份 ≠ 元数据"的实证。
- **摘要内容身份不得含 span**:含 span 时 body-only 编辑(签名不变)改变摘要内容,破坏共享;span 属于快照/HIR 工件。
- **僵尸行**:GC 删 chunk 后若保留 artifact 行,同键重发布永远 Conflict;GC 级联删行 + put 时死行替换,双向收敛。
- **LockFileEx 竞争返回 ERROR_LOCK_VIOLATION(33)** 而非 32:两者都必须映射为"忙",否则 lease 判活直接 IO 错误。
- **GC 自身 WAL 增长可能超过小仓的物理释放量**:收据必须分列 blocks/DB/WAL/trash,断言按层断(收据字段已分列)。
- **对象键以 HIR 查询键哈希为成分**(而非 HIR artifact id):对象命中查询必须在不编译的前提下可计算——缺源恢复的必要条件;body-only 编辑经快照链仍然失效对象。

## 4. 限制与后续(不做即声明的边界)

- reader 安全性证明止于**文件/锁级**;不冒称 DLL/callback 运行代际安全(A12 Future)。
- 无 pack/compaction;稀疏 reclaim 后的空间归还以 blocks 层字节数为证,NTFS 簇级 slack 未计量(已声明局限)。
- 布局人口为**确定性造数**,证明分页/预算/共享机制,不冒充通用 CST。
- 亚秒周转未宣称;实测距离见 perf 报告。
- 生产差距与迁移路线(现有两条 GC 入口、fingerprint 粗粒度、无 single-flight 服务化)见 spec-delta(SD-04)。
