# auto-mir plans

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 821 | AV Place拥有型读写与全出口清理 | ✅（delivered，R6 最终 PASS） | [计划](../../plans/archive/821-av-place-owned-access.md) | av.owned/1 owned 访问 MIR：RecordView/FIELDMOVE 逐字段编码（前缀重叠 once、moved_fields 稀疏数据流+定义点重置）+t_MutRet 写回 ABI；[owned-access-profile.md](owned-access-profile.md) |
| 798 | Typed MIR与ABC/AV首个全路径探针 | archived | [计划](../../plans/archive/798-typed-mir-abc-av-probe.md) | 有限标量执行/信任/函数持久产物，准确记录整模块重算及828未交付边界 |
| 828 | av-function-incremental-session | ✅（delivered，T-10 最终 PASS） | [计划](../../plans/archive/828-av-function-incremental-session.md) | 函数级 lower/verify（不经 whole lower/split）+单元局部性+稳定 def id 跨单元引用+内容命中复用；[function-incremental.md](function-incremental.md) |
| 822 | AV类型化集合与Variant载荷访问 | ✅（delivered，R19 最终 PASS） | [计划](../../plans/archive/822-av-typed-collections-variants.md) | collections/1 MIR：List/Variant/Place(tag13/14/15) 类型域+无环+guard must-intersect 格点+payload-moved may-union+逃逸门；[collection-place-profile.md](collection-place-profile.md) |
