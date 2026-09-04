# AutoUI 域需求级设计

> AutoUI/App 生成域的**需求级设计文档**子目录（Plan 468 归位，2026-08-28）。
> 域级章见根级 [Design 16（App 生成战略）](../16-app-generation-and-ai-authoring.md) 与
> [Design 20（AutoUI 分离架构）](../20-autoui-separation-architecture.md)；本目录存放各需求/专题的深入设计。

| 文档 | 原号 | 主题 | 关联计划 |
|---|---|---|---|
| [shared-store](shared-store.md) | 18 | Rung 4 跨 widget/跨路由共享状态 | 351/370 |
| [theming-and-dark-mode](theming-and-dark-mode.md) | 19 | 深浅色模式与主题色配置 | 458 |
| [examples-app-track](examples-app-track.md) | 21 | examples/ui 应用轨道与 AutoOS 默认应用矩阵 | 401–441 |
| [base-styles-and-visual-parity](base-styles-and-visual-parity.md) | 22 | 跨后端基础样式与视觉一致性规范 | 411/455 |
| [virtual-desktop](virtual-desktop.md) | 23 | 虚拟桌面架构（WM/多窗口/桌面会话） | 452/459/462 |
| [desktop-shell-and-launcher](desktop-shell-and-launcher.md) | 24 | 桌面 Shell 与 Launcher（M2–M4） | 463/464/465 |
| [a2ui-composer-analysis](a2ui-composer-analysis.md) | 25* | Google A2UI 技术分析与实现映射（研究输入） | — |
| [desktop-protocol-v1](desktop-protocol-v1.md) | — | 桌面协议 v1：进程外 App 五通道（孵化/帧/输入/控制/观测）规范 | 386（Stage 2 落地） |
| [diagram-components](diagram-components.md) | — | Diagram 组件家族与 DSL 设计（Mermaid/D2 对标；统一 498/499 交互与 canvas 模型） | 设计先行（建议拆 plan，§8） |
| [canvas-pointer-events](canvas-pointer-events.md) | — | Canvas 交互 v2：通用指针事件原语（mousemove 限频流/坐标语义/P-list 协议草案/扇区-边命中同源/axisPointer/动画双轨） | 499 |
| [image-viewer-pipeline](image-viewer-pipeline.md) | — | 全栈 Auto Image Viewer：后端媒体管线、三运行形态与高性能原生显示契约 | 547（draft） |
| [sidebar-family-and-nav-retirement](sidebar-family-and-nav-retirement.md) | — | sidebar_* 族按 shadcn 1:1 做实（桌面模式）→ VM 契约子集 → 迁移退役 nav-group/nav-item | 设计完成（建议拆 3 plan，§4） |
| [025-gap-enumeration](025-gap-enumeration.md) | 16a | 025 示例差距枚举（历史记录） | 345 |

> \* 原 `25-a2ui-composer-analysis.md`。注：该文档已归位本目录为 `desktop-shell.md`（Design 25，曾用名 AutoShell），
> 届时 25 号一并封存。
