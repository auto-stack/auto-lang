# auto-semantics — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 796 | ac-semantic-metadata-inference-probe | ✅（delivered，r16；R1..R15 反例修复清偿后 final pass） | [archive/796-ac-semantic-metadata-inference-probe.md](../../plans/archive/796-ac-semantic-metadata-inference-probe.md) | AC 语义元信息与类型推导合同及探针：名义身份（方法含 owner）/完整覆盖矩阵/有限 i32·bool 推导/CT·反射模型接缝；783 store 消费（键覆盖一切实际依赖、嵌入 ref 全量当前准入、manifest 预算、SCC/有效版本/GC 保活）；AutoDown probe/run 受限 adapter（semaworker/1）；完整编辑请求窗口周转硬门 P95<1s@1MiB；D1–D24 冻结决策；121 定向测试+16 轮独立复审 |
| 817 | AV驱动的CTE查询与冻结缓存闭环 | ✅ delivered | [计划](../../plans/archive/817-av-cte-query-frozen-cache.md) | 真实执行域指针（模型合同零修改）；consumer 回灌消费 infer/SemaInput 事实源 |
