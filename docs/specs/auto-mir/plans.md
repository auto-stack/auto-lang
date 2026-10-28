# auto-mir plans

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 824 | AV动态Variant与可移植冻结 | ✅ delivered（R3 最终 PASS） | [计划](../../plans/archive/824-av-dynamic-values-freeze.md) | dynamic MIR 递归白名单、类型身份与 checked 恢复；[dynamic-cast-profile.md](dynamic-cast-profile.md) |
| 821 | AV Place拥有型读写与全出口清理 | ✅（delivered，R6 最终 PASS） | [计划](../../plans/archive/821-av-place-owned-access.md) | av.owned/1 owned 访问 MIR：RecordView/FIELDMOVE 逐字段编码（前缀重叠 once、moved_fields 稀疏数据流+定义点重置）+t_MutRet 写回 ABI；[owned-access-profile.md](owned-access-profile.md) |
| 798 | Typed MIR与ABC/AV首个全路径探针 | archived | [计划](../../plans/archive/798-typed-mir-abc-av-probe.md) | 有限标量执行/信任/函数持久产物，准确记录整模块重算及828未交付边界 |
| 828 | av-function-incremental-session | ✅（delivered，T-10 最终 PASS） | [计划](../../plans/archive/828-av-function-incremental-session.md) | 函数级 lower/verify（不经 whole lower/split）+单元局部性+稳定 def id 跨单元引用+内容命中复用；[function-incremental.md](function-incremental.md) |
| 823 | AV类型化闭包环境与捕获生命周期 | ✅（delivered，R3 最终 PASS） | [计划](../../plans/archive/823-av-owned-closure-captures.md) | closures/1 MIR：Closure 槽+CREATE_CLOSURE(CalleeRef)/CALL_CLOSURE/ENV_GET(0x32-34)、闭包体自带 env 声明+link 等价检查、捕获三态(Move/Clone/Borrow)与逃逸 admissible 集、帧代际/句柄不复用；[closure-capture-profile.md](closure-capture-profile.md) |
| 822 | AV类型化集合与Variant载荷访问 | ✅（delivered，R19 最终 PASS） | [计划](../../plans/archive/822-av-typed-collections-variants.md) | collections/1 MIR：List/Variant/Place(tag13/14/15) 类型域+无环+guard must-intersect 格点+payload-moved may-union+逃逸门；[collection-place-profile.md](collection-place-profile.md) |
| 868 | HIR→MIR lowering 出库为共享 crate | ✅（delivered，R2 最终 PASS） | [计划](../../plans/archive/868-hir-mir-lowering-shared-crate.md) | lowering 唯一实现=auto-mir-lower（纯搬移+三消费者改指+mir-lowering 版本轴必报，收据键集不动）；[hir-lowering-boundary.md](hir-lowering-boundary.md) |
