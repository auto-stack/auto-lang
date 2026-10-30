# auto-mir

> PLAN-896当前组织：上述ACE相关实现统一属于`experimental/ace/Cargo.toml`独立workspace（单Cargo.lock/同树target），各子目录为成员包，不再是独立workspace。ace-lowering为唯一共享lowering，ace-av-core持有共享MIR/AV模型；ace-native等真实消费者保持原依赖方向。普通语言cargo t不构建ACE；cargo t ace选择常规集合，完整包/all及ace-performance保原专项和适用条件。

> **Status**: experimental

PLAN-798：逻辑模块位于 `experimental/ace/crates/av-core/src/{mir,mir_verify}.rs`，实际Rust包是 `auto-av-core`，没有独立已交付的auto-mir crate。

Checked HIR → Typed MIR → verify_mir/CheckedMir → ABC/AV。MIR采用显式块、有类型可变槽与terminator，不要求SSA。core-i32 profile 为 i32/bool、直接调用、控制流和受控测试trace（历史边界）；PLAN-820 起 av.data/1 profile 支持宽标量数据与纯值 record/字段 Place（见 [数据 profile](data-place-profile.md)）。类型/CFG/初始化/资源检查授信，guest trap不提前提升为前端错误。

- [标量合同](core-i32.md)：支持面、凭证、资源和语义（core-i32 profile）。
- [数据 profile](data-place-profile.md)：av.data/1 宽标量/名义 record（PLAN-820）。
- HIR adapter：`experimental/ace/crates/av/src/hir_adapter.rs`。
- [AV缓存与验收](../auto-av/cache-and-conformance.md)：持久单元、实际重算粒度与证据。

798有函数持久产物，不具全链路函数增量：miss仍whole-module lower/verify/split。单函数前端/HIR/MIR/装载替换已由 828 交付（见
[auto-av/resident-function-install](../auto-av/resident-function-install.md)）；PLAN-820 在数据域消费该接口（record 域单函数 AVAD unit + 全链增量），不扩大 native/CTE 默认 profile。

PLAN-824 在实验 typed MIR 增加 [dynamic-cast-profile](dynamic-cast-profile.md)，以验证过的类型擦除及 checked 恢复接入 ABC/AV，不扩大源码或 native 支持面。

- [native集合/变体](native-collections-variants.md)：显式864源码/typed Clone→shared MIR→COFF/PE、资源/真函数增量/深验恢复（PLAN-889）；不包含宽typed入口或native在线帧。
