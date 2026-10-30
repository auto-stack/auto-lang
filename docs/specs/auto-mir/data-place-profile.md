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
  `experimental/ace/crates/av-core/src/instr_spec.rs`（12 opcode 表 +
  coverage_check 漏项即红）。
- 类型等价=结构相等，在**全部消费点**一致（Const/Copy/Bin/Un/Call/
  Return/Aggregate/FieldGet/FieldSet/entry）；重复 primitive 条目按形状
  归一；宽标量运算 `mir.type-op` 拒。
- entry 参数/返回限六标量（record 不出 entry）；unit 模式沿 798 r6 合同
  （每 unit 单函数+`fn:<name>` 导入）。

## 验收

V-01/02/04/11；core MIR/verifier 测试与 semantics 族（13/13）。


## native 消费者覆盖（PLAN-870）

- mn（`experimental/ace/crates/native`）按本合同消费 av.data/1 的**纯值 record**
  子集：Copy/Aggregate/FieldGet/FieldSet 的机器语义=本合同的纯值语义
  （字段声明序布局、结构重建写回、RHS 先物化）；三路差分与 ACE-M1
  R01/R02/R03/R06 实测绑定见
  [native-data-owned-lowering](native-data-owned-lowering.md)。
- 宽标量**不在** mn 面（触达即具名拒绝，R2-N 保持关闭）；record 不出
  entry（沿既有约束）。
- mn 单元身份/cache：A09' 键轴不变；fp 渲染对 record 以名义身份
  （owner|name:字段形状）入指纹（PLAN-870 修复了旧判别式标签碰撞）。

## 方法与 mut 写回（PLAN-881）

- `DefKind::Method`（static/view）经共享 lowering 降为普通 MIR 函数
  （`m.<Owner>.<name>` 单元身份）；未调用方法不再拒整模块。细节合同见
  [record-method-lowering](record-method-lowering.md)。
- **mut-record 写回 ABI（新增）**：mut record 参数包装结果为
  `t_MutRet_<fn>`（f0=源结果，wb*=终值），借用临时与写回均为 Copy（纯值
  语义；排他为源级 verify.mut-conflict）。此前 av.data/1 拒 BorrowMut 的
  面由本 ABI 取代（820 冻结面扩容，R07=42 实测关闭）。其余排除（collections
  mut 实例方法等）保留。
