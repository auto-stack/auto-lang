# Auto v0.6 Roadmap · 快速发布

v0.6 按 **1–2 个月**的快速发布周期开列范围。本版集中交付四条主线，并于2026-11-12追加面向实际工作的Auto ROS2 SDK专项；具体发布日期随交付与联合验收确定，不据此推导已核定工期。

<a id="5-linux-发行版与-rqhost"></a>

## 从版本目标进入路线

| 路线 | 本版目标 | 继续阅读 |
|---|---|---|
| 1. 项目拆分与重构 | 完成 M1 的仓库与构建边界收口，验收真实产品的构建和交付。 | [收口战略](design/strategy/modularization-and-refactoring-strategy.md) · [路线进度与下一批](plans/reports/v06-extraction-execution-roadmap.md) · [试读：从 1001 的调试策略研究到 1003 的生产切换](plans/reports/v06-extraction-execution-roadmap.md#生成工程的调试策略) |
| 2. ACE（M1、M2） | M1 交付含 A2C 的引擎基础；M2 补全语言与标准库，完成既有产品和目标的接管、安装发行切换及旧引擎退役。 | [M1 实施路线](design/strategy/ace-implementation-handoff.md) · [M2 设计](design/55-ace-m2-complete-language-and-product-replacement.md) · [M2 路线与计划](design/strategy/ace-m2-delivery-roadmap.md) · [试读：961 的数值语义如何交给 962](design/strategy/ace-m2-delivery-roadmap.md#数值与组合类型) |
| 3. AGE（M1、M2） | 分两阶段交付图形引擎，M2 覆盖完整 AGE 语法、ACE 接入、2D/3D 增强，以及与实际应用和桌面 UI 的融合。 | [AGE 两阶段路线及与 ACE 的交接](../../auto-ui/docs/design/autographics/age-three-milestones-and-ace-m2.md) |
| 4. Linux 发行版与 App 强化 | 交付 Linux 发行版，完善 AutoOS 应用的实际使用能力。版本所含发行范围、应用清单与验收场景由本路线核定。 | [发行版仓库路线](https://github.com/auto-stack/auto-os-distro/blob/main/ROADMAP.md) · [App 基础能力盘点](../../auto-os/docs/research/apps/basic-capability-audit-20261029.md) · [应用强化与 Linux 移植分期](../../auto-os/docs/research/apps/linux-mvp-priorities.md) |

追加专项：**Auto ROS2 SDK** 以实际工作可用为v0.6目标，交付类型生成、Topic/Service/Action、参数/QoS/时间与资源、AV开发/A2R发布、ROS工程工具、真实任务＋AutoUI控制台、Autonix安装运行和质量验收。见[专项设计与12项必做矩阵](design/57-auto-ros2-sdk.md)、[R0–R5与官方示例首组路线](design/strategy/ros2-sdk-delivery-roadmap.md)。本次新增[1018官方基础示例](plans/1018-ros2-sdk-official-examples-conformance.md)→[1019 turtlesim综合教程](plans/1019-ros2-sdk-turtlesim-tutorial-and-interoperability.md)→1011传感器工作流的规划顺序；复用1007原生预验证，正式示例只走公共SDK并验AV/A2R。官方示例输入可由1007具名切片正式交接，设备/最终预算与发行合同仍保留；本轮只规划，不运行新例子/GUI，不改M1合同或当前不合并限制。核心SDK按ACE接口切片推进，AGE增强不作为核心前置，Linux集成按NixOS＋COSMIC实际合同验收。

## 联合里程碑与正式发布

- **v0.6-M1** 汇合拆分与重构M1、ACE M1与AGE M1。拆分路线1006/989/1015均已正式归档，按用户确认完成本路线[M1收口文档整理](plans/reports/split-refactor-m1-closeout-2026-11-12.md)；013静态生成AddTodo未交付，已列M2 AutoUI有限修复，不阻本M1。联合标签仍须1013等当前ACE/AGE正式签章、覆盖1015的到期批门及最终跨仓清单；当前ACE测试运行，批门待独占窗口，不能用旧ready或旧覆盖代替。实际打标/推送另待用户指令。
- **正式 v0.6** 在拆分与重构 M1 的基础上，汇合 ACE M2、AGE M2、Linux 发行版与 App 强化的交付结果，通过最终版本联合验收。各路线按实际接口依赖并行推进。Linux 两应用 RQHost 准入的验收范围见 [OS062](../../auto-os/docs/plans/062-linux-native-rqhost-admission.md)。

<a id="7-v06路线阶段与拆分边界矩阵2026-10-16"></a>

ROS2专项新增发布义务按Design57 R06-01..12独立验收，并纳入正式v0.6联合范围；未通过不得由bridge demo冒称SDK完成。ACE M2、AGE M2和发行版原门保持，ROS2签章不替代任何主线签章。

## 旧版规划

旧文档中引用的十方向矩阵、早期日历和历次调整保存在[快速发布前的规划稿](roadmap-v0.6-2026-11-10.md#7-v06路线阶段与拆分边界矩阵2026-10-16)。当前版本的交付范围以本页四条主线及新增ROS2专项为准。
