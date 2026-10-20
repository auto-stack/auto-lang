# auto-av

> **Status**: experimental（PLAN-798）。实际Rust包：`experimental/ac-av-probe/core` 的auto-av-core与 `probe` 的auto-av-probe，独立workspace/锁；不替代产品旧AutoVM，不改变默认CLI/UI/CTE。

Checked HIR → Typed MIR → ABC decode/verify → 私有LoadedModule → Rust AV。解释器使用显式frame/slot栈和受限dispatch，支持i32/bool、控制流/直接调用和精确trap。

- [ABC与执行](abc-core-i32.md)
- [缓存、资源与验收](cache-and-conformance.md)
- [MIR](../auto-mir/core-i32.md)

798不是在线函数替换/完整AJ/发布器。用户指定798完成后优先828，再进行ACE扩展；实际增量边界以缓存合同为准。825执行优化、826紧凑wire和820–824数据扩展不得冒领未交付能力。
