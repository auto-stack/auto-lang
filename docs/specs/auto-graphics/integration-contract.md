# auto-graphics — integration contract

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
