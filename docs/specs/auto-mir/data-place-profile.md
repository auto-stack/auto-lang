# auto.mir.av-data/1 数据 MIR 合同（PLAN-820）

> **Status**: experimental（PLAN-820 交付；实际位于 auto-av-core 的
> mir/mir_verify，无独立 crate）

## 类型与常量

- `MirType` 增 I64/U64/F32/F64/Record（名义身份=owner+name+fields 结构）。
- `MirConst` 增宽标量位型（to_bits 逐位保真；冻结样本含 u64::MAX、
  i64::MIN、f64 负 NaN payload 0xFFF8_0000_DEAD_BEEF、-0.0、f32 NaN
  payload 0x7F80_AD00）。

## 指令合同与等价

- 新指令的 operand/read-def/trap/fuel/profile 合同见
  `experimental/ac-av-probe/core/src/instr_spec.rs`（12 opcode 表 +
  coverage_check 漏项即红）。
- 类型等价=结构相等，在**全部消费点**一致（Const/Copy/Bin/Un/Call/
  Return/Aggregate/FieldGet/FieldSet/entry）；重复 primitive 条目按形状
  归一；宽标量运算 `mir.type-op` 拒。
- entry 参数/返回限六标量（record 不出 entry）；unit 模式沿 798 r6 合同
  （每 unit 单函数+`fn:<name>` 导入）。

## 验收

V-01/02/04/11；core MIR/verifier 测试与 semantics 族（13/13）。
