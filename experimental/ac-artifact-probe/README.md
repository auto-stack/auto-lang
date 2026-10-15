# ac-artifact-probe — PLAN-783 编译产物契约与缓存可行性探针

独立实验 workspace(不进根 workspace,不成为生产缓存入口)。消费
ac-core 公开接口跑真纵线,隔离复用 `auto-cache` 的 `BlobStore` 作块层。

```
源码(.at) ──parse──► AST ──A04 摘要──► …
   │
   └─compile_parts_with─► Bundle ──A07 二进制 codec─► store
                              │─(恢复后重过真 verify)─┘
                          verify ──lower_object──► A09 COFF ──link_object_staged──► 运行
```

- 层登记 A00–A12:`schema.rs`(实际 codec 子集 = A00/A04/A07/A09/A10)。
- 身份/引用:`identity.rs`(域分离 blake3;QueryKey/ArtifactId/ChunkId;
  LocalRef/ExternalRef 带界校验)。
- codec:`codec.rs`(chunk 框架自校验;HIR core+scalar 受限子集,解码
  全边界校验;恢复的 Bundle **不带信任**,一律重过真 verifier)。
- 布局:`layout/`(同一逻辑数据两种冻结布局,64KiB 页、8MiB 热预算、
  2 页 I/O 暂存、小查询 ≤512KiB、页级 CAS 跨版本共享)。
- 仓:`store/`(SQLite 元数据 + BlobStore 适配 + 跨进程锁;发布三故障点、
  lease OS 真值判活、tombstone 门锁、水位 GC 64/48MiB、8MiB staging 上限)。
- 驱动:`driver.rs` + `bin/node.rs`(计数器证明跳过;缺源注入)。

## 运行

```powershell
cargo test --manifest-path experimental/ac-artifact-probe/Cargo.toml --release --test native_resume_
# 全矩阵(acceptance)与 closeout(512 churn + 30 样本 perf):
powershell scripts/verify-ac-artifact-probe.ps1 -Scope acceptance
powershell scripts/verify-ac-artifact-probe.ps1 -Scope closeout
```

报告:`docs/reports/plan783/`。预算是实验冻结值,不得放宽换绿。
