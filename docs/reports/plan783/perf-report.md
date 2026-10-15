# PLAN-783 性能与资源基线(V-11,T-06;R1 修复后再生)

- 机器:见 `perf.json` `machine`(Windows,核心数/物理内存随文件);参考机数据,非产品承诺。
- fixture:`p04-while-retest.at`(oracle exit 3);每场景 **30 样本**(plan §5.5);构建 profile=release,`source_commit` 随报告(本文件对应未提交工作区,提交后由 closeout 再生为提交号)。
- 语义正确性:每样本断言 exit oracle(`semantic_failures=0`),不减验证换时延。
- GC 场景证据(R1 F-07 修复):22 次 GC 运行、**77 个 chunk 真实物理删除**、0 worker 失败;触发为稳态基准之上的 hair-pin 高水位 + 无地板(每趟回收全部死候选)。真实水位边界(64/48MiB)由 512 churn 载荷证明。

| 场景 | 含义 | P50 | P95 | P99 | 全对 |
|---|---|---|---|---|---|
| fresh | 冷仓:parse→HIR→verify→lower→link→run(每样本不同源) | 559 ms | 615 | 648 | 30/30 |
| same-process-hot | 同进程对象命中:仅真 link+run | 391 ms | 470 | 509 | 30/30 |
| restart-disk-hot | 新进程对象命中(进程生发+开仓+link+run) | 308 ms | 364 | 376 | 30/30 |
| cross-project | 同内容他工程:全命中,零新块 | 402 ms | 465 | 501 | 30/30 |
| body-modified | 同签名改 body:对象 miss→全重编 | 569 ms | 700 | 722 | 30/30 |
| gc-concurrent-query | GC 真实并发回收时的命中查询(77 块删除/22 趟) | 528 ms | 633 | 667 | 30/30 |

## 读数与瓶颈(诚实归因)

1. **对象命中路径由链接支配**:fresh≈559ms vs hot≈391ms,其中 `link_object_staged`
   的 rust-lld 子进程约 200–230ms;前端+verify+lower 合计约 150ms。R1 修复后
   (liveness 互锁、CAS 准入校验)命中路径多了协调与校验开销,同机较修复前
   慢约 20–30%——这是正确性修复的真实代价,如实记录。
2. **跨进程重启代价小**:restart-hot P50 308ms 低于同进程 hot(进程生发比
   同进程的锁竞争窗口更轻),磁盘热路径成立。
3. **GC 并发尾延迟**:P99 667ms(基线 509ms),来源为 liveness 互锁排队与
   删除 I/O 与查询交错;GC 删除真实发生(77 块)。生产化需 GC 限速/有界批次
   让路(Design40 §7.2),探针未实现调度——见 D-783-A。
4. **磁盘占用**:单版本五工件约 90KB;fresh/body-modified 每样本新增约 150KB;
   cross-project 0 新块(内容共享)。
5. **内存**:in-process 场景记录 OS 级 private-bytes 差量(随 JSON);
   restart 场景跨进程,报告 0 并注明(不编造数字)。逐场景 per-process I/O
   计数器不可用(已声明局限);布局证据(V-03)的 I/O 为真实页读计数。

## 亚秒距离(不宣称达标)

冷编译 P95 615ms、命中 P95 470ms——**亚秒在 P95 意义上已达**,但收益结构
(链接支配)决定更大的节省需增量链接/对象直出(D-783-B);本计划只测距离。

## 下一阶段建议(不在本计划实施)

- 链接支配→增量链接/对象缓存直出 PE 的增量发布(A10 拆阶段)。
- GC 写锁/互锁竞争→低优先级 GC 队列 + 批次让路(尾延迟归因明确后)。
- 大编译单元下重测 fresh-vs-hit 收益差与 A04 摘要免体装载。
