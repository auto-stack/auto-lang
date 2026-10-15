# PLAN-780 r1 / R1 规范增量冻结（needs_fix）

| delta | 操作 | 目标 | 已核方向 |
|---|---|---|---|
| SD01 | add | docs/specs/project-model/project.md | 数据DTO/独立依赖、静态入口、source/path归属、raw与unsupported策略、明确pilot与未迁范围 |
| SD02 | modify | docs/specs/auto-man/project.md | 显式旧新探针、动态配置仍保留VM adapter、生产读取未切换、字段owner/后续接线职责 |

当前auto-man/project.md源SHA256：bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3；新project-model规范尚不存在。

方向与Design37及既有PLAN635/647声明门一致；metadata new_spec_components=[docs/specs/project-model/project.md]、supersedes_spec_components=[docs/specs/auto-man/project.md]、touched_goals=[]（有限解耦探针，无新增产品目标）。

status=partial：独立消费与零生产切换已核实；静态等价/unknown信息保留/读取错误传播尚有合同内缺口，不能发布为已成立的持久保障。修复复审后再冻结具体支持/拒绝面与规范文本；本轮不发布canonical Spec或ledger。
