# av.data/1 数据布局与纯值 record（PLAN-820）

> **Status**: experimental（PLAN-820 交付；独立 data profile，非默认产品面）

`av.data/1`（profile id 2，AVAD wire v1）在 798 共享核心上交付宽标量与纯值
record 的数据表示与布局。语义版本轴 1/1/1 独立；core-i32 的 1/2/1 与 AVAB
字节语义不变（见 [project](project.md)）。

## 支持面

- **六标量**：i32/bool 保持旧标量运算；新增 i64/u64/f32/f64 仅数据面
  （const/copy/call/return/entry，逐位保真，无算术——`mir.type-op` 拒）。
- **名义 record**：owner+name+字段表结构相等为类型身份；同宽不同 owner
  不可互换；类型图无环（迭代检测）；空 record 不支持；每类型字段 ≤1024。
- **指令**：FIELD_GET / FIELD_SET / AGGREGATE（仅 data profile）；12 opcode
  合同表与覆盖检查见 `core/instr_spec.rs`（漏项即红）。

## 布局与预算

- 布局从受检类型图推导：checked 算术、逻辑描述符（size/align/字段位置）；
  物理存储 = `Value` 枚举，绝不 reinterpret Rust 对象；恢复时重新计算并
  校验，不信磁盘 offset。
- 布局推导计入**共享验证工作预算**（2M 步；跨 record 持久 memo，链式
  O(N)，Budget→Resource(VerifySteps)，PLAN-820 R3）；size/align/offset
  溢出为受控 `mir.layout-overflow`（嵌套二叉展开实测可达）。
- 槽+聚合共享 8MiB 活值预算（deep_size delta 记账、分配前拒、peak 全出口
  上 trace）；路径深 ≤64。

## 验收

V-01..05/11/12：稳定用例 ID、独立反例及复用关系见
[review-r10](../../plans/attachments/820/review-r10/review.md)；
core layout/instr_spec 测试；`scripts/verify-av-data-820.ps1` 8 Scope。
