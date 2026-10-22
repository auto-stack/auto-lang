# auto-mir 函数级增量（PLAN-828）

> Status: active（实现=`experimental/ac-av-probe/probe/src/hir_adapter.rs` +
> `core/src/{mir,mir_verify}.rs`）。

- 单函数 lower：`lower_function_unit` 从 CheckedFunction（经单函数 bundle）直接
  lower/verify 该函数；**不调用 whole lower 再 split**；未改函数零 lower/emit/
  decode/verify/load（结构门 0 断言）。
- 单元局部性：函数局部常量/类型/槽/PC 独立于其他函数长度/顺序；跨函数引用=稳定
  def id 文本（`fn:<name>` import + 链接期绑定表），不是模块数组序号/物理地址 ABI。
- 首版函数为唯一根区域（Design43 §8.1）：身份/依赖/代码引用保持可分解接缝；
  block 级增量/活帧迁移/无语义占位指令不在本合同。
- 内容命中：同内容版本复用现存不可变 MIR/ABC/代码对象与各自 mint 来源/代际；
  仅变更单元产生新产物（计数只计新产物）。
