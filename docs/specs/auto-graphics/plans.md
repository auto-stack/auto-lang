# auto-graphics 相关 Plan 索引

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 829 | AutoGraphics 独立调研与 2D/3D 游戏原型 | archived | [archive/829](../../plans/archive/829-autographics-independent-research-and-game-prototypes.md) | AGP 独立 wgpu workspace：Breakout/Squash 双游戏 bit-exact 原型 + Iced 同设备探针 + 十探针验证矩阵；代码落 `autographics-dev` 集成分支（不进 master），I1–I3 生产集成 pending |
| 830 | AutoGraphics 可组合 Surface/Portal 与嵌套输入原型 | archived | [archive/830](../../plans/archive/830-autographics-surface-portal-composition-and-input.md) | 在 829 单窗 probe 上验证可组合 Surface/View/Portal：UI→3D→UI / 3D→UI→2D 双向嵌套、离屏 iced 采样合成（预乘 alpha/clip/平面相机）、共享内容多挂载单 tick、捕获/唯一焦点/三态取消、真实 caret OS IME、全 adapter 同设备归属、真实上传/owned 内存计量与省略计数负控；代码落 `autographics-dev`，I1–I3/AutoShader 仍 pending |

| 833 | 真实 Auto 源码驱动 AutoGraphics 最小闭环 | archived | [archive/833](../../plans/archive/833-autographics-auto-source-minimal-loop.md) | 纯 SDK 公开 CPU slice 驱动真实 Auto 场景/Breakout，输出准入、共享单 tick、全资源归属与文字真实上传计量；源码在 autographics-dev，I1 首片，Auto Squash/I2/I3/Shader/CAP 仍 pending |

| 834 | 真实 Auto 驱动 Squash 与双游戏 SDK 闭环 | archived | [archive/834](../../plans/archive/834-autographics-auto-squash-and-dual-game-sdk.md) | 同一公开 SDK/Session 驱动真实 Auto Breakout+Squash，原轨迹/音画/输入/寿命/精确计量与正反控通过；I1实验双游戏分母交付，源码仅autographics-dev，I2/I3/Shader/CAP/生产迁移仍pending |

| 835 | 场景创作保存与撤销原型 | archived | [archive/835](../../plans/archive/835-autographics-scene-authoring-save-and-undo.md) | 独立stock Iced+真实Auto profile3的有限2D/3D文档、编辑事务、保存/撤销/重开与失败恢复；源码仅autographics-dev，I2/I3/生产Forge仍pending |

| 836 | 视口选择与平移交互原型 | archived | [archive/836](../../plans/archive/836-autographics-viewport-picking-and-translation.md) | 有限2D几何/3D OBB与世界轴平移，真实Auto临时预览、一次历史、失败恢复；源码autographics-dev，生产I2/I3 pending |

| 837 | 场景层级与组创作原型 | archived | [archive/837](../../plans/archive/837-autographics-scene-hierarchy-and-group-authoring.md) | 有限同维平移/显隐组、真实Auto profile4、v2保存与保持世界位置挂接；源码autographics-dev，生产I2/I3 pending |

| 838 | 静态GLB导入与可携带场景资源 | archived | [archive/838](../../plans/archive/838-autographics-glb-import-and-portable-scene-assets.md) | 严格静态GLB/真实Auto profile5、内容寻址资源/原字节身份、schema3闭包保存/搬迁重开；源码autographics-dev，生产I2/I3 pending |
