# auto-mir

> **Status**: experimental（PLAN-798）；逻辑模块位于 `experimental/ac-av-probe/core/src/{mir,mir_verify}.rs`，实际Rust包是 `auto-av-core`，没有独立已交付的auto-mir crate。

Checked HIR → Typed MIR → verify_mir/CheckedMir → ABC/AV。MIR采用显式块、有类型可变槽与terminator，不要求SSA；当前仅i32/bool、直接调用、控制流和受控测试trace。类型/CFG/初始化/资源检查授信，guest trap不提前提升为前端错误。

- [标量合同](core-i32.md)：支持面、凭证、资源和语义。
- HIR adapter：`experimental/ac-av-probe/probe/src/hir_adapter.rs`。
- [AV缓存与验收](../auto-av/cache-and-conformance.md)：持久单元、实际重算粒度与证据。

798有函数持久产物，不具全链路函数增量：miss仍whole-module lower/verify/split。828负责单函数前端/HIR/MIR/装载替换；该能力未交付前不冒领。
