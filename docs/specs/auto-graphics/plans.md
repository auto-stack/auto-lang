# auto-graphics 相关 Plan 索引

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 829 | AutoGraphics 独立调研与 2D/3D 游戏原型 | archived | [archive/829](../../plans/archive/829-autographics-independent-research-and-game-prototypes.md) | AGP 独立 wgpu workspace：Breakout/Squash 双游戏 bit-exact 原型 + Iced 同设备探针 + 十探针验证矩阵；代码落 `autographics-dev` 集成分支（不进 master），I1–I3 生产集成 pending |
| 830 | AutoGraphics 可组合 Surface/Portal 与嵌套输入原型 | archived | [archive/830](../../plans/archive/830-autographics-surface-portal-composition-and-input.md) | 在 829 单窗 probe 上验证可组合 Surface/View/Portal：UI→3D→UI / 3D→UI→2D 双向嵌套、离屏 iced 采样合成（预乘 alpha/clip/平面相机）、共享内容多挂载单 tick、捕获/唯一焦点/三态取消、真实 caret OS IME、全 adapter 同设备归属、真实上传/owned 内存计量与省略计数负控；代码落 `autographics-dev`，I1–I3/AutoShader 仍 pending |

| 833 | 真实 Auto 源码驱动 AutoGraphics 最小闭环 | archived | [archive/833](../../plans/archive/833-autographics-auto-source-minimal-loop.md) | 纯 SDK 公开 CPU slice 驱动真实 Auto 场景/Breakout，输出准入、共享单 tick、全资源归属与文字真实上传计量；源码在 autographics-dev，I1 首片，Auto Squash/I2/I3/Shader/CAP 仍 pending |
