# PLAN-783 性能与资源基线(V-11,T-06;R3 修复后再生)

- 机器:见 `perf.json` `machine`(Windows,核心数/物理内存随文件);参考机数据,非产品承诺。
- fixture:`p04-while-retest.at`(oracle exit 3);每场景 **30 样本**(plan §5.5);构建 profile=release;`source_commit` 随报告(本文件对应 R3 修复轮工作区,closeout 将以 `AC_PROBE_SOURCE_COMMIT` 再生绑定冻结提交)。
- 语义正确性:每样本断言 exit oracle(`semantic_failures=0`),不减验证换时延。
- GC 场景证据:21 趟 GC,**73 个 chunk 物理删除,其中 69 个发生在被计时查询期间**(overlap 计数,R3 补充);0 worker 失败;触发为稳态基准之上的 hair-pin 高水位+无地板(每趟回收全部死候选)。真实水位边界(64/48MiB)由 512 churn 证明。

| 场景 | 含义 | P50 | P95 | P99 | 全对 |
|---|---|---|---|---|---|
| fresh | 冷仓:parse→HIR→verify→lower→link→run(每样本不同源) | 298 ms | 363 | 394 | 30/30 |
| same-process-hot | 同进程对象命中:仅真 link+run | 272 ms | 322 | 328 | 30/30 |
| restart-disk-hot | 新进程对象命中(进程生发+开仓+link+run) | 217 ms | 280 | 325 | 30/30 |
| cross-project | 同内容他工程:全命中,零新块 | 264 ms | 294 | 302 | 30/30 |
| body-modified | 同签名改 body:对象 miss→全重编 | 335 ms | 389 | 391 | 30/30 |
| gc-concurrent-query | GC 真实并发回收时的命中查询(21趟/73块删/69重叠) | 342 ms | 509 | 544 | 30/30 |

## 读数与瓶颈(诚实归因)

1. **对象命中路径由链接支配**:fresh≈298ms vs hot≈272ms,其中 `link_object_staged`
   的 rust-lld 子进程约 150–200ms。R3 轮(原子预算预留+GC 物理沉降测量+
   namespace 门)在同机引入可测开销——fresh/hot 较 R2 轮各 +60~80ms,如实
   记录:staging 预算锁串行化发布、GC settled_space 沉降等待(每次至多
   2×120ms)与批次重扫是主因;生产化需按路径分摊优化。
2. **跨进程重启代价小**:restart-hot P50 217ms(低于同进程 hot 的锁竞争窗口),磁盘热路径成立。
3. **GC 并发尾延迟**:P99 544ms(基线 328ms),来源为 liveness 互锁排队+预算锁
   串行化+删除 I/O 与查询交错;GC 删除真实发生且与计时查询重叠(69/73)。
   生产化需 GC 限速/有界批次让路(Design40 §7.2),探针未实现调度——见 D-783-A。
4. **磁盘占用**:单版本五工件约 90KB;fresh/body-modified 每样本新增约 150KB;
   cross-project 0 新块(内容共享)。
5. **内存**:in-process 场景记录 OS 级 private-bytes 差量(随 JSON,四场景实测,
   restart 跨进程报告 0 并注明);staging 真实并发峰值由锁内观测记录
   (churn 收据 `peak_staging`,R3-01)。逐场景 per-process I/O 计数器不可用
   (已声明局限);布局证据(V-03)的 I/O 为真实页读计数。

## 亚秒距离(不宣称达标)

冷编译 P95 363ms、命中 P95 322ms——P95 意义上仍亚秒,但收益结构(链接支配)
决定更大节省需增量链接/对象直出(D-783-B);R3 轮的协调开销是正确性代价,
生产化时需分摊优化;本计划只测距离。

## 下一阶段建议(不在本计划实施)

- 链接支配→增量链接/对象缓存直出 PE 的增量发布(A10 拆阶段)。
- GC 互锁竞争→低优先级 GC 队列+批次让路(尾延迟归因已有重叠证据)。
- 大编译单元下重测 fresh-vs-hit 收益差与 A04 摘要免体装载。
