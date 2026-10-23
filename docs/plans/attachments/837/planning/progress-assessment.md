# PLAN-837 后的 AutoGraphics 体系位置（规划推演）

2026-10-23，只读判断，不是验收报告。结论前提：836、837均完成最终审查、冻结完整门、实际合入与收尾。目前836为reviewed9/10、final R2 pass，837 drafting0/10，后续能力不算今天已交付。

## 能力位置

| 层面 | 当前可靠依据 | 837交付后的增量 | 关键剩余面 |
|---|---|---|---|
| 渲染与小游戏 | 829固定wgpu2D/3D；833/834真实Auto Breakout+Squash及原音画/回放/玩法/寿命 | 保留原分母，不新增游戏 | 通用资产/材质/贴图、完整动画/物理、更多样例和平台不由双示例证明 |
| UI/2D/3D组合 | 830独立stock Iced同设备、混合嵌套、坐标/焦点/IME/取消及资源 | 对象层级继续消费原组合设施 | 生产AutoUI/RQHost adapter、迁移后承接；组树不等于任意UI/Portal编辑 |
| 单对象创作 | 835属性/真实Auto预览/创建删除/保存/撤销重开；836拾取/拖动/三轴已复审尚未交付 | 继承836直接操作 | 旋转缩放gizmo、相机工具、多选等未纳入 |
| 场景组织 | 当前平面objects表，无父子继承 | 场景树、同维group、平移/显隐继承、整组动、保世界位置重挂、子树删除及undo/保存重开 | group完整TRS/剪切、Prefab/实例、任意组件/资源关联 |
| Auto与公共接口 | 固定语言SDK/Session与有限profile，真实Auto参与执行 | 显式profile4由Auto计算world/effective，Rust检查后消费 | 稳定通用AutoGraphics库/API、ACE/native全面消费、脚本组件/热改编辑 |
| AutoForge | 独立stock-Iced创作前置实验 | 可组织并直接编辑有限小场景，创作雏形更完整 | 正式Forge产品、生产AutoUI、资源浏览导入、游戏/脚本编辑等 |
| 发行/生态 | 独立source consumer与单机证据 | 原型交接继续完善 | 生产迁移/多平台打包、正式发行/Linux，Shader/CAP按各自前置推进 |

## 可使用的完成口径

837完成后，可以说“独立图形实验的运行、组合、真实Auto和有限场景创作闭环基本成型”：能把一组物体组织成小场景、整体修改、撤销和保存重开。它比展示小游戏或操作单物体更接近AutoForge的可用雏形。

不能据此称“通用引擎已完成”“Godot/Bevy的某个百分比”“生产AutoUI已融合”或“v0.6图形主线全部交付”。双示例分母明确，完整引擎是开放能力池，没有可审计总工作量分母。

对v0.6：独立技术验证与有限创作雏形将较完整；关键仍是迁移完成后接生产AutoUI/RQHost，并据实际消费者打磨公共API、资产/图形能力、Forge与发行。831/832启动锁不变，源码不会因此自动入master。

后续滚动规划先核迁移能否开启I2，再从真实消费者缺口选一块；不预分配838等计划，也不将全套编辑器纳入本月门槛。

## 依据

- [v0.6图形/Forge目标](../../../../roadmap-v0.6.md)
- [当前图形知识](../../../../specs/auto-graphics/overview.md)
- [I1/I2/I3边界](../../../../specs/auto-graphics/integration-contract.md)
- [835创作合同](../../../../specs/auto-graphics/authoring-session.md)
- [836 final R2与merge pending](../../836/review-r2/review.md)
- [837执行草案](../../../837-autographics-scene-hierarchy-and-group-authoring.md)
- [来源观察](source-observation.json.txt)
