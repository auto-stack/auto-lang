# auto-ac artifact probe（产物契约与缓存可行性探针）

> **Status**: experimental（PLAN-783 交付;独立实验 workspace,非生产缓存入口）
> 路径：`experimental/ac-artifact-probe/`（独立 Cargo workspace + 独立 Cargo.lock）;
> 依赖：`auto-ac-prototype`(path) + `auto-cache` BlobStore(隔离适配) + blake3/rusqlite/windows-sys。
> 关联 [artifact-contract.md](artifact-contract.md);报告 `docs/reports/plan783/`。

消费 ac-core 公开接口跑真纵线（四 core oracle: fresh/跨进程对象命中(缺源注入)/
二进制 HIR 恢复重验/scalar 兼容控制）,验证 A00–A12 契约与缓存生命周期。
ac-core 零产品依赖由 dep_audit 永久断言;探针不改变语言面、不新增生产入口。

## 冻结预算（实验值,不得放宽换绿）

| 项 | 值 |
|---|---|
| 布局页 | 64 KiB;每布局 ≥128 MiB 实写;热缓存 ≤8 MiB;I/O 暂存 ≤2 页;小查询 ≤512 KiB |
| GC 水位 | 高 64 MiB / 低 48 MiB（总量=blocks+DB+WAL+staging+trash+locks+logs;收敛以沉降物理总量判定） |
| 临时空间 | **≤8 MiB = staging + CAS `.tmp` 双副本 + 全部失败/崩溃残留 tmp 之和**——跨进程独占预算锁横跨 staging 写入至 CAS 放置的原子预留（R3-01/R4-02/R5-01）;预留计入全部既有残留（`cas_tmp_bytes`）,失败出口在锁内清理自身 tmp,并发发布在配额窗口内串行;PEAK_TRANSIENT 只记成功预留 |

## 协议要点（R1–R6 反例迭代冻结）

- **发布=单临界区**：共享 liveness 覆盖 staging 分配→预算→CAS 投放→元数据提交;
  GC/恢复全程独占 liveness——在途发布不可见、注册时最终存活校验。
- **GC/恢复候选=完整状态矩阵**（R4-01）:keep 集=活 artifact 引用块;无行/
  deleted-tombstoned 行重现块/无 artifact 的 live 行全入回收,附行级联。
- **lease OS 真值判活**：锁文件被持即活（TTL/PID 不参与）;进程死即锁释放。
- **namespace 根 reparse 硬边界**（GC 与 recover 共拿 `check_namespace_roots`）。
- **SQLite 真实收缩**：`auto_vacuum=INCREMENTAL`+checkpoint;GC 决策用
  `settled_space` 沉降物理总量（NTFS 新写块可见性滞后的应对）。

## 运行面

`scripts/verify-ac-artifact-probe.ps1`:-Scope acceptance（18 族清单化校验:
摘要在场/最小通过数/ignored 白名单+已提交 perf 合同实证）;-Scope closeout
（merge-owned:512 churn 恒等校验恰 1 passed/0 failed+用例名行;30 样本 perf
再生→新临时产物校验（≥30 样本/六场景唯一/重叠删除>0/source_commit==冻结
HEAD）→发布后 SHA256 验证;校验或发布 IO 失败均传播为门禁失败且旧报告保留）。
注意：本机 sccache 端口被保留段阻塞时 cargo 需 `RUSTC_WRAPPER=` 空覆盖。

## 限制清单（不冒称的部分）

文件/锁级 reader 证明≠DLL/callback 运行代际安全;无 pack/compaction;布局人口
为确定性造数（证明分页/预算/共享机制,不冒充通用 CST）;无亚秒承诺（收益结构
由 rust-lld 链接支配 ~150-200ms,见 perf-report）;生产差距见
[auto-cache/project.md](../auto-cache/project.md) 探针边界节。
