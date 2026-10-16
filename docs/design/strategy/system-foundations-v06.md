# 应用通信、AutoAI 与 DevTools：v0.6 阶段与拆分接缝

> 2026-10-16；本仓的跨路线规划补充，未实施。不复制或取代 auto-os 所有的协议 RFC；后续实施由所属仓维护。
> 权威设计输入：[通信 RFC](../../../../auto-os/docs/design/strategy/auto-app-communication-v0.6.md)、[DevTools RFC](../../../../auto-os/docs/design/strategy/system-devtools-v0.6.md)、[F0–F5 分期](../../../../auto-os/docs/design/strategy/v0.6-foundations-roadmap.md)；本轮只读这些外仓文件。

## 1. 方向、现状与最新约束

AutoOS 自己拥有服务/能力与通信语义，通过边界 adapter 兼容 ROS2；不把 ROS2 设为所有 App 的必经底层。transport、编码和语言 Task/Msg 是不同层，Zenoh/本地 IPC 等选型仍需证据，本文不冻结具体中间件。

现有 Task/Msg 是进程内模型，桌面 RenderQueue、HTTP AI daemon 和现有 MCP 也不等于已实现通用跨进程框架。auto-ai daemon/client 可复用；auto-ai-agent 对 Atom/Value 的依赖需要随具体变更重核，不能把整个 AI 仓都称完全独立。

用户 2026-10-16 明确顺序：先建立 AutoOS 跨进程通信框架，再开展具体机器人算法。本轮不规划 BEV/SLAM、训练、板级算法部署或机器人算法 demo；[ROS2 战略](ros2-ecosystem-strategy.md)保留生态兼容方向和待定支持矩阵。

## 2. v0.6 阶段与验收

通信/AutoAI、系统 DevTools 是已确认版本重点；以下是分阶段范围建议，协议字段/平台预算与发布范围由正式 Plan 冻结。早期独立原型不替代后面的真实集成验收。

| 阶段 | 结果与验收 | 必要前置 |
|---|---|---|
| F0 | schema/能力/错误/取消/回执及 golden 样例；未知版本、不可序列化值和无权限请求明确拒绝 | 复用 RFC，独立于旧 VM，不等待 Batom v2 |
| F1 | 两个真实进程注册/发现/typed call；有界消息与截止；服务崩溃/重启后旧 instance 引用失效；错误不变成成功空值 | F0；选定单平台、IPC 与保真 codec |
| F2 | 长任务进度/状态/取消、事件/流背压；超时后查询收据，幂等请求不重复产生领域效果；断线/取消/结果未知分别表达 | F1；独立 client/server 先验证，再接应用 |
| AI profile | 同一能力被普通 client 与 Agent/MCP bridge 调用，得到同义结果/错误；模型流取消后释放资源；记录调用轮次、字节/token、耗时与任务成功率 | F1/F2 对应能力；复用 AI daemon/client，不把它们改名为已完成的系统框架 |
| F3 | 公共快照、能力/缺失字段、身份与坐标合同；离线语义/布局/像素 diff 三种结果分别报告；stale revision、缺字段和不匹配帧可识别 | 离线样例可独立做；协议不复制现有 VTree 实现 |
| F4 | 独立 Inspector/Agent client 消费真实 adapter；VM 与生成 Rust 两 backend 的共同能力逐项证明；一个第三方样板声明可用和不可用能力 | F1 与真实 SDK/UI adapter；Qt/accessibility 接入不冒称完整源码调试 |
| F5 | Launcher→Notes 捕获任务和 receipt 实际贯通，退出重启后内容可核对；有失败/重试反例和 DevTools 取证 | F2、应用 adapter、真实持久化与 UI 集成 |

AutoLens 的独立候选是“输入一个固定内容/上下文快照→保存来源→进入知识整理”；活体选区/VTree 采集依赖 F3/F4 和平台权限。离线导入只算知识原型，不冒称系统级采集完成。

## 3. 依赖与拆分边界

F0/F1、独立 client/server、F3 离线 schema/diff 可不等搬仓；消息使用独立 DTO/显式 codec，不序列化现有 Value、闭包、指针或 UI 句柄。实验可先用可验证的兼容编码，须保证宽整数/bytes/variant 保真或明确拒绝。

现有 Task/Msg、VM native 注册、自动生成语言 binding、VTree/输入采样和应用装配归语言/UI SDK adapter；调整这些入口须与 Design37 及在途拆分 Plan 协调。auto-man 只提供包/安装身份，broker 的临时进程实例不等于包版本。
AutoWeb、AutoForge、AutoLens 与服务复用共同能力/trace/错误语义，不各造发现与授权体系。独立 GPU/知识/输入引擎原型可以先完成，成为真实 App 后再接通信。

## 4. 后续与待决事项

完整分布式 Actor 集群、透明迁移、完整 ROS2 QoS 映射、全应用迁移、全面 debugger/时间旅行不自动进入首个原型。ROS2 兼容先定义 Topic/Service/Action 的语义矩阵，再决定 bridge 探针；不在本轮建立算法验收。
待选工作后冻结第一平台、transport/codec、授权与服务执行并发规则、首个领域服务及性能/资源预算；协议预期与实测分别记录。本次路线整理不启动 F0–F5 的代码实施。
