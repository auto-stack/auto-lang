# PLAN-783 性能与资源基线(V-11,T-06;R5 修复后再生)

- 机器:见 `perf.json` `machine`(Windows,核心数/物理内存随文件);参考机数据,非产品承诺。
- fixture:`p04-while-retest.at`(oracle exit 3);每场景 **30 样本**(plan §5.5);构建 profile=release;`source_commit` 随报告(本文件对应 R5 修复轮工作区,closeout 将以 `AC_PROBE_SOURCE_COMMIT` 再生绑定冻结提交)。
- 语义正确性:每样本断言 exit oracle(`semantic_failures=0`),不减验证换时延。
- GC 场景证据:22 趟 GC,**77 个 chunk 物理删除,其中 73 个发生在被计时查询期间**(overlap 计数);0 worker 失败;触发为稳态基准之上的 hair-pin 高水位+无地板(每趟回收全部死候选)。真实水位边界(64/48MiB)由 512 churn 证明。

| 场景 | 含义 | P50 | P95 | P99 | 全对 |
|---|---|---|---|---|---|
| fresh | 冷仓:parse→HIR→verify→lower→link→run(每样本不同源) | 255 ms | 265 | 266 | 30/30 |
| same-process-hot | 同进程对象命中:仅真 link+run | 271 ms | 297 | 354 | 30/30 |
| restart-disk-hot | 新进程对象命中(进程生发+开仓+link+run) | 219 ms | 250 | 264 | 30/30 |
| cross-project | 同内容他工程:全命中,零新块 | 272 ms | 316 | 322 | 30/30 |
| body-modified | 同签名改 body:对象 miss→全重编 | 340 ms | 357 | 367 | 30/30 |
| gc-concurrent-query | GC 真实并发回收时的命中查询(22趟/77块删/73重叠) | 428 ms | 621 | 632 | 30/30 |

## 读数与瓶颈(诚实归因)

1. **对象命中路径由链接支配**:fresh≈255ms,其中 `link_object_staged` 的
   rust-lld 子进程约 150–200ms。R5 轮加入残留 tmp 计账(每次预留走查 CAS
   `.tmp`)后有可测的协调开销——本轮机器同时承压(512 churn 同机并行),
   hot/cross 场景 p50 抬到 ~270ms,如实记录为"正确性代价+负载噪声",
   生产化需按路径分摊并缓存残留清单。
2. **跨进程重启代价小**:restart-hot P50 219ms,与同进程 hot 的 271ms 同量级
   (restart 场景的子进程冷启动反而避开了同进程锁竞争窗口),磁盘热路径成立。
3. **GC 并发尾延迟**:P99 632ms(本轮同机负载下),来源为 liveness 互锁排队+
   预算锁串行化+残留 tmp 走查+删除 I/O 与查询交错;GC 删除真实发生且与计时
   查询重叠(73/77)。生产化需 GC 限速/有界批次让路(Design40 §7.2),
   探针未实现调度——见 D-783-A。
4. **磁盘占用**:单版本五工件约 90KB;fresh/body-modified 每样本新增约 150KB;
   cross-project 0 新块(内容共享)。
5. **内存与临时空间**:in-process 场景记录 OS 级 private-bytes 差量(随 JSON,
   四场景实测,restart 跨进程报告 0 并注明)。`peak_staging`(churn 收据)是
   **PEAK_TRANSIENT 的成功预留上界**——含 staging 与 CAS `.tmp` 双副本与
   残留 tmp 的预留合计,只在预算检查通过后记录,**不是实测驻留 staging 字节**;
   实测磁盘临时字节由回归测试直接走查断言(r4_f02/r5_f01:staging+全部 tmp ≤8MiB)。
   逐场景 per-process I/O 计数器不可用(已声明局限);布局证据(V-03)的
   I/O 为真实页读计数。

## 亚秒距离(不宣称达标)

冷编译 P95 265ms;命中路径本轮 P95 297ms(同机承压)——收益结构(链接支配)
决定更大节省需增量链接/对象直出(D-783-B);多轮修复的协调开销是正确性代价,
生产化时需分摊优化;本计划只测距离。

## 下一阶段建议(不在本计划实施)

- 链接支配→增量链接/对象缓存直出 PE 的增量发布(A10 拆阶段)。
- GC 互锁竞争→低优先级 GC 队列+批次让路(尾延迟归因已有重叠证据)。
- 大编译单元下重测 fresh-vs-hit 收益差与 A04 摘要免体装载。
