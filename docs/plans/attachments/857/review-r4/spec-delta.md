# Plan 857 规范增量冻结副本（复审 R4）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/editor-viewports.md | 在856 2D上增加有限View3d/投影/导航/深度grid，不加游戏相机 | 明确编辑支持集 | AC-02/03/05 |
| SD-02 | modify | docs/specs/auto-graphics/viewport-interaction.md | 固定正交默认兼容＋透视/正交实际相机拾取/轴拖动/吸附/退化 | 防止旧近似漂移 | AC-04/05 |
| SD-03 | modify | docs/specs/auto-graphics/surface-composition.md | 有限编辑辅助深度绘制/同device所有权与计量，默认旧内容不变 | 可追踪新增资源 | AC-03/06 |
| SD-04 | modify | docs/specs/auto-graphics/integration-contract.md | 857当前source/consumer/AGP触面和OS-GPU数值/内容/深度负控 | 保留原分母 | AC-01/06/08 |

与 R3 冻结副本逐字相同；本轮无合同变更（R3 修复仅改验证工具与来源清单，delta 不变）。
