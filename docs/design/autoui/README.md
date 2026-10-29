# AutoUI 域需求级设计

> AutoUI/App 生成域的**需求级设计文档**子目录（Plan 468 归位，2026-08-28）。
> 域级章见根级 [Design 16（App 生成战略）](../16-app-generation-and-ai-authoring.md) 与
> [Design 20（AutoUI 分离架构）](../20-autoui-separation-architecture.md)；本目录存放各需求/专题的深入设计。

| 文档 | 原号 | 主题 | 关联计划 |
|---|---|---|---|
| [app-presentation-host](app-presentation-host.md) | — | App 声明 Region/Section/Scene，Feature 多出口按目标装配；灵活布局、面板/整页栏目、overlay/通知、状态恢复与双端合同 | 设计草案（2026-10-14；Design 38 前置，未实施） |
| [editing-feature-syntax](editing-feature-syntax.md) | — | auto-edit 嵌入语法草案：open/bind/split 常见组合简写、bound surfaces/actions、双实例隔离、文档/视图共享、菜单与快捷键路由；保留显式层 | Draft v0.2（2026-10-28；Design 47 细化，仅设计） |
| [design-system-abstractions](../../../../auto-ui/docs/design/autoui/design-system-abstractions.md) | — | 样式高阶抽象：typed 值/组合、组件设计合同与多部件 recipe、Theme/DesignContext/Design Pack、Figma token 与组件映射双链；含官方机制比较 | 专题讨论稿 v0.1（2026-10-29；Design 47 §7 深化，未实施） |
| [button-design-syntax](../../../../auto-ui/docs/design/autoui/button-design-syntax.md) | — | Button 端到端语法：视觉接口、typed recipe、Theme/schema、Design Pack；简洁默认、运行期切包、局部角色/配方覆盖、多状态组合与旧 profile 边界 | Draft v0.1（2026-10-29；仅设计，未实施） |
| [design-data-model](../../../../auto-ui/docs/design/autoui/design-data-model.md) | — | 树状主题/语义样式/组件配置；数据优先、组件 Schema 校验、标准状态规则、普通 Auto 模块与引用；比较并收敛公开样式语言的复杂度 | 讨论稿 v0.3（2026-10-29；方向已确认，具体协议未冻结/实施） |
| [button-theme-atom](../../../../auto-ui/docs/design/autoui/button-theme-atom.md) | — | Button 的 Auto 树状主题：use 导入、token 点引用、主题族/模式、widget 局部 .Design 事件、父子继承与切换事务 | 设计草案 v0.2（2026-10-29；仅设计，未实施） |
| [theme-conventions-and-pilot](../../../../auto-ui/docs/design/autoui/theme-conventions-and-pilot.md) | — | 公共/Feature/app token、extend、模式差异；官方主题系列、首批真实 demo 与 examples 全量逐项迁移、AutoOS 承接 | 迁移路线 v0.2；auto-ui/PLAN-007 r3 drafting（2026-10-29；未实施） |
| [theme-migration-inventory](../../../../auto-ui/docs/design/autoui/theme-migration-inventory.md) | — | examples/ui 37目录的旧语义/固定色候选初步盘点与逐项状态；AutoOS 独立来源边界 | 只读快照（2026-10-29；不等于运行验收） |
| [shared-store](shared-store.md) | 18 | Rung 4 跨 widget/跨路由共享状态 | 351/370 |
| [theming-and-dark-mode](theming-and-dark-mode.md) | 19 | 深浅色模式与主题色配置——**值层已被 [Design 29](../29-autoui-style-theme-system.md) 取代**（词汇表/正交维度仍有效，头部有取代注记） | 458 |
| [examples-app-track](examples-app-track.md) | 21 | examples/ui 应用轨道与 AutoOS 默认应用矩阵 | 401–441 |
| [base-styles-and-visual-parity](base-styles-and-visual-parity.md) | 22 | 跨后端基础样式与视觉一致性规范 | 411/455 |
| [virtual-desktop](virtual-desktop.md) | 23 | 虚拟桌面架构（WM/多窗口/桌面会话） | 452/459/462 |
| [desktop-shell-and-launcher](desktop-shell-and-launcher.md) | 24 | 桌面 Shell 与 Launcher（M2–M4） | 463/464/465 |
| [a2ui-composer-analysis](a2ui-composer-analysis.md) | 25* | Google A2UI 技术分析与实现映射（研究输入） | — |
| [desktop-protocol-v1](desktop-protocol-v1.md) | — | 桌面协议 v1：进程外 App 五通道（孵化/帧/输入/控制/观测）规范 | 386（Stage 2 落地） |
| [rq-remote-renderer](rq-remote-renderer.md) | — | **RQ 远程 Renderer 架构（方案 2 已裁定）**：iced 组件照常构建/布局/命中，渲染原语经 RecordRenderer 序列化过 RenderQueue，RQHost 重放；含四方案对比裁定记录、方案 3（自有渲染）未来选项与重估触发条件 | 680（立项）/679（对拍驱动） |
| [diagram-components](diagram-components.md) | — | Diagram 组件家族与 DSL 设计（Mermaid/D2 对标；统一 498/499 交互与 canvas 模型） | 设计先行（建议拆 plan，§8） |
| [canvas-pointer-events](canvas-pointer-events.md) | — | Canvas 交互 v2：通用指针事件原语（mousemove 限频流/坐标语义/P-list 协议草案/扇区-边命中同源/axisPointer/动画双轨） | 499 |
| [layout-interaction](layout-interaction.md) | — | 布局件交互原语（row/col/div 的 `hover:` 类消费 + `oncontextmenu` 挂点；HoverArea 共享标志零重建） | 002（origin 535） |
| [image-viewer-pipeline](image-viewer-pipeline.md) | — | 全栈 Auto Image Viewer：后端媒体管线、三运行形态与高性能原生显示契约 | 547（implemented，待独立复审） |
| [sidebar-family-and-nav-retirement](sidebar-family-and-nav-retirement.md) | — | sidebar_* 族按 shadcn 1:1 做实（桌面模式）→ VM 契约子集 → 迁移退役 nav-group/nav-item | 设计完成（建议拆 3 plan，§4） |
| [icon-state-and-library-policy](icon-state-and-library-policy.md) | — | 图标 state 契约（on/off 颜色+描边模拟满/空）与图库策略裁定（lucide 主力不迁移 + IconifyJSON 补位 playbook） | 621 |
| [icon-data-source-and-parity](icon-data-source-and-parity.md) | — | 图标数据源与生成器（lucide 全量表/离线重跑/版本漂移门禁）与双端一致性口径（几何/尺寸/颜色/描边） | 620 |
| [025-gap-enumeration](025-gap-enumeration.md) | 16a | 025 示例差距枚举（历史记录） | 345 |
| [autoui-mcp-test-fixture](autoui-mcp-test-fixture.md) | — | AutoUI MCP 测试夹具协议（VM 状态注入与 request-id ack） | 623 |
| [desktop-shell-a2r](desktop-shell-a2r.md) | — | 桌面 Shell a2r 编译化形态（缺口普查 + A 链入宿主/B outproc 形态裁定输入 + 迁移路径草案 S1–S3） | 设计先行（待裁定拆 plan，§10） |
| [vm-frame-budget](vm-frame-budget.md) | — | VM 帧预算与空转渲染：渲染模型分层（条件/for/可见性）、消息泵×Element 缓存双因、easy wins 边界、D-1..D-5 复杂机制设计选项（帧间缓存/挂载退订/列表虚拟化/desktop 层跳帧） | 650（easy wins）；D-1 起建议另拆 plan |
| [vm-loading-responsiveness](vm-loading-responsiveness.md) | — | VM 大程序加载响应性：可续跑 CPU 预算、Init 代际/帧通知、缓存与 computed 热点、独占 worker 后续边界（proposed，未实施） | 708 r2 |
| [component-time-and-events](component-time-and-events.md) | — | 嵌套组件时间源与交互契约：TimeSource 收集/挂载过滤订阅/多层 props·msg·store 路径；D-2 类型级退订；gallery 内嵌 Tick 缺口 | 652（阶段 1 ✅）；654（阶段 2 path + 阶段 3 Clock） |
| [universal-scroll-architecture](universal-scroll-architecture.md) | — | 通用滚动架构（替代原 universal-scrollbar 设想）：scroll-pane primitive + ScrollContent/ScrollState/Intent/Controller/Anchor 协议；普通布局/虚拟内容/自管 buffer 三类宿主同语义；Phase A–G 落地路径与验收 | 设计先行（draft）；Phase A+B+C 立项 PLAN-656；Phase E（terminal 迁移）为后续消费端 |
| [reader-runtime-prerequisites](reader-runtime-prerequisites.md) | — | Reader 框架前置：原生验收通道→子内容锚点→10MiB导入；单一计划、三阶段，保留 READER-001 原验收目标 | 781/r2（drafting，未执行） |

> \* 原 `25-a2ui-composer-analysis.md`。注：该文档已归位本目录为 `desktop-shell.md`（Design 25，曾用名 AutoShell），
> 届时 25 号一并封存。

