# auto-graphics — integration contract

> Current status (PLAN-834): I1's original real Auto Breakout + Squash denominator is delivered experimentally on autographics-dev. Earlier PLAN-829/830/833 pending statements below record their historical stage. I2 production AutoUI/RQHost migration, I3 Forge/distribution and Shader/CAP remain pending. Master carries knowledge only; graphics source does not enter master.

> Status: experimental（PLAN-829 交接合同；代码在 `autographics-dev` 分支，
> 见 [project.md](project.md)）。**两道合成检测门必须并列保留，不可互替**
> （R5-03/R18 裁定，见 IH-01）。

## Verified（PLAN-829，probe 证据）

**IH-01（独立探针级）**：iced 0.14 应用可在**同一** wgpu device/queue 上承载
真实控件 + 2D/3D 图形视图（iced_wgpu 交给 custom primitive prepare 的实例）。
合成=UI 之下不透明图形、按控件裁剪；resize/DPI/退出已验证。同设备证明是
功能+架构双重的：wgpu 27 在 API 层不拒绝跨 device 绑定（全局资源 id）——
视图路径**永不自建设备**（永久审计测试）。

合成验证有**两道独立门**（[counter 门 ≠ 内容门]）：

1. **Counter 门**（带内合成计数，BLIT_DRAWS 仅紧邻真实 composite draw
   递增）：捕获**漏画/跳过最终绘制**变异（blank-blit 永久反控）。
   早期 GDI 窗口截取环境受阻的记录即此门的来源背景。
2. **Content 门**（iced 0.14 `window::screenshot` 三时刻像素断言 +
   `transparent_blit` 永久反控）：捕获**最终合成全透明**变异——R4-05 实测
   该变异下 counter 门保持绿（2→104→111）而内容门三时刻全红；即
   **counter 不能检测透明片元**，内容门不可弱化回 counter-only。

当前正控（三时刻全绿）与完整透明负控（三时刻全拒识）在冻结提交
`ddbde3e62857` 上由 R18 复审与 T-11 merge 门禁双重复核。Iced 不持游戏
业务状态。Stock iced_widget 0.14.2 满足本 spike；生产 PLAN-043 patch
在此不需要（r1 F09，已记录）。

**IH-05（仓界）**：独立 workspace、crates.io stock iced_widget 0.14.2
（r1 F09 去 vendoring，无 provenance-tracked patch 残留）、sha256 冻结
资产、根 workspace 零附带改动（V-09 门）。

## Pending（不得读作已交付）

- IH-02 Auto SDK：G-base 运行时合同已有（身份/时间/动作/事件/回放），
  但无已发布 SDK/profile；`.at` 入口是 I1（pending）。
- IH-03 RQHost：无 GPU 协议；现行 rqhost 仍为 CPU/软件路径。
- IH-04 嵌套/Forge：仅单窗口合成；无 UI→3D→UI 矩阵、pick-through、
  IME、Forge 编辑闭环。

## I1–I3 gates

- I1 真实 Auto 源码游戏入口：需已发布 SDK 符号+收据；验收=双冻结游戏经
  公开 SDK 由 `.at` 驱动。
- I2 生产 AutoUI/嵌套/RQHost：需迁移完成收据（M814-B 及后继）；
  iced 探针不抵扣。**I2 的迁移完成/写面交接同时是 `autographics-dev`
  合回 master 的裁定前提（2026-10-22 用户裁定）。**
- I3 Forge 与发行：需 I2 + 编辑闭环设计收据。

后继计划必须引用本文件，且只能以绑定收据的证据替换 "pending"。

## PLAN-830 实验增量（组合 Surface/Portal 与嵌套输入）

> 代码落 `autographics-dev`（交付提交 `764e87559`）；证据
> `docs/plans/attachments/830/work-r6/merge/`。以下**只**扩充 IH-01/IH-04 的
> 实验事实，不改变 IH-02/03/05 与 I1–I3 的 pending 状态，也不表示生产接线完成。

**IH-01 增量（同设备/离屏内容采样）**

- 真实 iced 控件内容以离屏采样目标被 3D 平面与 2D 四边形消费；预乘 alpha
  over、线性/sRGB 一致；2D 用 scissor 裁剪、3D 平面用挂载 host Surface 相机；
  DAG 顺序（子先于父）由运行时检查（`DependencyOrder`）。
- 同设备归属对 Iced/2D/3D adapter 逐一记录并校验；外设备内容具名拒绝
  （`DeviceMismatch`），无法验证归属的 raw-device 内容 `DeviceUnverifiable`。
- 失败事务：目标超限/尺寸非法先分类后提交，保持 last-good；resize 替换/丢弃
  准备/slot 删除的真实目标生命周期计数守恒。
- 计量：两条组合管线与内容 adapter 的真实 `write_buffer` 计数（portal 160B/
  次、平面 128B/次）；验收 oracle 要求分管线精确、总量=分项之和；省略计数的
  变异必须令门禁红（V-08 钩子）。

**IH-04 增量（嵌套输入）**

- 双向嵌套均实测：UI→3D→UI 全链路点击/焦点/DPI2/resize；3D→UI→2D（冻结
  Breakout 暂停/reset/键盘焦点）；共享 Squash 内容两 Portal 同像素、单 tick、
  每挂载一次输入。
- 命中：2D clip/仿射反算、3D 射线+UV（按 host 相机）、递归非交互遮挡吞掉、
  捕获绑挂载路径、销毁/失焦/离窗取消。
- 焦点：唯一真实编辑目标；宿主合批与“取消先于 pending 渲染”均由 router 三态
  （`Unset/Target/Cancelled`）贯通到真实 widget 缓存；新合法点击/公开 focus
  可恢复。非法 Surface（foreign/stale/非 iced）的 focus 请求拒绝且无副作用。
- IME：真实 `text_input` 的 `InputMethod::Enabled.cursor` 作为 caret，沿活跃
  挂载链投影为 OS 候选锚点并随输入/选择/滚动更新；真实 OS 中文 IME 提交已取证。
  候选窗逐像素模板比对不在本版证据范围。

**仍 pending（不得读作已交付）**

- IH-02 Auto SDK 入口、IH-03 RQHost GPU 协议、IH-05 生产发行边界保持原状；
  I1 真实 Auto 源码入口、I2 生产 AutoUI/嵌套/RQHost（迁移完成收据）、I3 Forge
  发行仍 pending；AutoShader/CAP 未实现。
- 云/多窗口/跨平台、曲面或任意网格 Portal、逐平面深度遮挡、iced 内部上传与
  驱动驻留内存计量、组合管线 error scope：均未支持/未测，见 prototype-profile。
## PLAN-833 delivered experimental slice

Adds a finite real Auto SDK/AutoVM driver for 2D/3D object control and Auto-owned Breakout, with checked output/event admission and original Surface/Portal consumption. See [auto-source-driver.md](auto-source-driver.md). This is only I1's first slice: Auto Squash and the original two-game denominator remain pending. IH-03, production I2/RQHost, I3/Forge/distribution, AutoShader/CAP and the migration gate for merging graphics code into master remain pending. PLAN-830 composition/input facts above are retained.

Delivery provenance: [PLAN-833](../../plans/archive/833-autographics-auto-source-minimal-loop.md), reviewed code 77302d855; production boundaries above remain unchanged.


## PLAN-834 delivered I1 dual-game evidence

The delivered experimental slice supplies the original second denominator member: real Auto Squash through the same pinned SDK/Session consumer as Auto Breakout. The exact frozen games, numerical/event tolerances and image/audio resources remain the denominator. Work evidence includes unchanged Breakout 10800 ticks/210 contacts, Squash 3600 ticks/45 spawns/91 events/135 random draws, actual GLB/HUD/Retry images, actual Kira playback and native input/focus/retry controls. Source hashes and the independent source consumer bind the delivered slice; tests that mutate Auto physics/animation and real rendering/input/accounting paths must fail their corresponding formal oracles.

I1 is delivered for its original experimental denominator: both frozen games run from real Auto through the same published SDK symbols. Independent T-09 R1 and frozen T-10 full driver/AGP/native/source-consumer gates passed. This completes I1 only, not a production migration. IH-03/RQHost, production I2, I3/Forge/distribution, AutoShader/CAP and the requirement for a separate authorized production migration before merging graphics source into master remain unchanged. The delivered source targets autographics-dev only. The two original composition counter/content gates remain distinct.

Delivery provenance: [PLAN-834](../../plans/archive/834-autographics-auto-squash-and-dual-game-sdk.md), reviewed ebeab12d2888b9c9fb7e00e3c24479fb3c028011; [full merge receipt](../../plans/attachments/834/merge-r1/receipt.md). Frozen limits and original tolerances remain unchanged.

## PLAN-835 delivered scene authoring extension

Delivered experimental authoring entry `autographics-authoring` in the existing driver workspace. See [scene-document](scene-document.md) for finite schema/resources/local save/reopen and [authoring-session](authoring-session.md) for real stock-Iced commands, explicit Auto profile3, transactional preview/history/dirty, failures and tests. Profile1/2, original game goldens/GLB/audio and shared composition/input contracts retain their meaning. One same-device window, two fixed2D/3D views, bounded document/history, local same-volume single writer; no production UI/compiler/SDK expansion. I1 dual-game delivery remains; I2 AutoUI/RQHost and I3 Forge/production distribution remain pending. Final R2 and frozen driver/AGP/native/source gates passed; source lands on autographics-dev only.

Delivery: [PLAN-835](../../plans/archive/835-autographics-scene-authoring-save-and-undo.md), [review R2](../../plans/attachments/835/review-r2/review.md), [merge receipt](../../plans/attachments/835/merge-r1/receipt.md). Production I2/I3 remain pending.
