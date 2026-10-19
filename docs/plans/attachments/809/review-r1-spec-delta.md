### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-man-core/project.md | 规则散在Pac/Target/Dir/Builder→单份管理/图/扫描选择/构建规划、效果边及图错误合同 | 独立核心 | AC-01..04 |
| SD-02 | add | docs/specs/project-model/design/generation-request.md | 无通用生成输入→I-PROJECT字段/来源/覆盖顺序/未知与动态准入/无副作用 | UI工具脱Man前置 | AC-02 |
| SD-03 | modify | docs/specs/project-model/project.md | 静态pilot不变；新增独立请求数据消费，不扩大静态语法或786护栏 | 避免假等价 | AC-02,03 |
| SD-04 | modify | docs/specs/auto-man/project.md | 记录实际core委托/旧公开门面/adapter和仍未迁UI/CLI/发行职责 | 不虚报全Man独立 | AC-01..04 |

