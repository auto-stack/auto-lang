# PLAN-783 性能与资源基线(V-11,T-06)

- 机器:见 `perf.json` `machine`(Windows,核心数/物理内存随文件);参考机数据,非产品承诺。
- fixture:`p04-while-retest.at`(oracle exit 3);每场景 **30 样本**(plan §5.5);预热另列于注记。
- profile:release;`node.exe`(真实进程)承载跨进程场景;restart 场景含 OS 进程生发,**不冒称 OS 全冷**。
- 语义正确性:每样本断言 exit oracle(`semantic_failures=0`),不减验证换时延。

| 场景 | 含义 | P50 | P95 | P99 | 全对 |
|---|---|---|---|---|---|
| fresh | 冷仓:parse→HIR→verify→lower→link→run(每样本不同源) | 351.1 ms | 399.3 | 402.1 | 30/30 |
| same-process-hot | 同进程对象命中:仅真 link+run | 258.4 ms | 288.3 | 289.5 | 30/30 |
| restart-disk-hot | 新进程对象命中(进程生发+开仓+link+run) | 279.0 ms | 356.0 | 357.0 | 30/30 |
| cross-project | 同内容他工程:全命中,零新块 | 255.9 ms | 358.5 | 392.0 | 30/30 |
| body-modified | 同签名改 body:对象 miss→全重编 | 350.4 ms | 381.6 | 388.9 | 30/30 |
| gc-concurrent-query | GC 并发回收垃圾版本时的命中查询 | 333.2 ms | 523.3 | 694.9 | 30/30 |

## 读数与瓶颈(诚实归因)

1. **对象命中路径由链接支配**:fresh≈hot≈350/258 ms,其中 `link_object_staged`
   的 rust-lld 子进程调用约 200–230 ms(冒烟实测 212–215 ms)。前端+verify+lower
   合计仅 ~90 ms——本 fixture 规模下缓存省的是少数项;**亚秒已达**(P95<400 ms),
   但"亚秒靠缓存"的收益面要到更大编译单元才成立。
2. **跨进程重启代价小**:restart-hot 比同进程 hot 仅 +21 ms P50(进程生发+SQLite
   开仓+chunk 读),磁盘热路径成立。
3. **GC 并发尾延迟可见**:P95/P99 抬升(523/695 ms),来源为 SQLite 写锁竞争与
   删除 I/O 与查询交错——生产化需要 GC 限速/批次退避(设计 §7.2 的有界批次)。
4. **磁盘占用**:单版本五工件(A00/A04/A07/A09/A10)约 90 KB;fresh/body-modified
   每样本新增 ~150 KB;cross-project 0 新块(内容共享实证)。

## 内存/IO(V-03 布局证据,同机)

- 128 MiB×2 布局写入后,**单记录小查询 I/O ≤128 KiB**(上限 512 KiB),400 页扫描后
  热缓存 ≤8 MiB、OS 级 private bytes 增量 <32 MiB(全量装载必 ≥128 MiB)——按需读取成立。
- I/O 证据为探针计数(真实 pread 页读)+ OS 级 `GetProcessMemoryInfo`
  private/working set;无 per-process IO 计数器可用,已声明该局限。

## 下一阶段建议(不在本计划实施)

- 链接支配问题→增量链接/对象缓存直出 PE 的增量发布路径(A10 拆阶段)。
- GC 写锁竞争→WAL 之外的 GC 专用低优先级队列 + 批次让路。
- 大编译单元(多模块/多函数)下重测 fresh-vs-hit 收益差与 A04 摘要免体装载。
