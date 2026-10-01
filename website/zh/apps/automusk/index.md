---
title: AutoMusk — 围绕 Plan 与 Spec 的 Coding Agent
description: 对话、工具调用、开发计划、复审和项目知识，以及当前自动化边界。
outline: [2, 3]
editLink: false
---

# AutoMusk：围绕 Plan 与 Spec 的 Coding Agent

AutoMusk 是 Auto 生态中的 Coding Agent 应用。它把项目对话、工具调用和开发过程放在一个工作台中，用 Plan 组织一次变更，用 Spec 保留项目已有的知识与约束。

它希望解决的具体问题是：任务经过多轮分析和执行后，需求、决策、改动和验证结果仍能被找到，下一次工作也有可靠的起点。

[全部应用](/zh/apps) · [AutoAI 架构](/zh/ai)

<!-- Screenshot slot: overview from the agreed sample-project walkthrough, after AutoMusk preparation is complete. -->

## 从对话进入项目工作

选择工作目录后，可以先让 Agent 阅读项目并解释结构，再讨论一个具体变更。应用呈现消息、工具事件、计划进度、确认门和报告；用户据此检查发生了什么，并在需要决策的位置继续参与。

一个有代表性的示例应该贯穿同一个小工程：理解项目 → 提出需求 → 检查计划 → 执行改动 → 查看复审与结果。本页先说明这个过程，完整截图将在应用准备好后按该流程补齐。

## Plan 与 Spec 各自记录什么

| 资料 | 用途 |
|---|---|
| Plan | 一次变更的目标、设计、任务、验收条件、进展与复审结果 |
| 模块 Specs | 项目当前约定的行为、接口和设计知识 |
| Spec Ledger | 对知识、关系和历史的派生索引，不替代模块规范 |
| 对话与报告 | 解释执行过程、工具活动与需要人工处理的问题 |

Plan 的完成意味着这次工作满足了验收条件；项目知识的沉淀则要把交付结果反映到规范中。这两件事不应仅凭“运行结束”就视为完成。

## 规划、执行、复审与沉淀

AutoMusk 使用四阶段方法：先形成可检查的计划，再实施变更，复查实际代码与测试结果，最后沉淀知识。应用内置 Relay 的 `plan` 流程当前串行使用 `plan-dev` 角色，在执行前设置人工确认门。

仓库的四个开发技能与应用内 Relay 是两个入口。技能侧的 worktree、复审、合入和沉淀规约，不应被描述为所有应用运行都已自动强制执行。当前复审失败后还需要跟进修复和再次复审；服务重启也不会恢复内存中的活动 Relay 运行。

<!-- Screenshot slot: the same sample project's Plan, confirmation gate, tool activity, and reviewed result. -->

## 模型、工具与应用的分工

AutoMusk 负责开发任务、项目资料、本地工具、API 与界面。公共 Agent 循环、角色、技能和通用编排能力来自 AutoAI；模型通信通过客户端连接到 aaid，由它管理模型服务、并发和用量。应用需要文件或命令操作时，执行仍属于工具所在的应用或 Agent 层。

模型、角色与技能可以配置；实际可用模型取决于已配置的提供商、账户权限与运行环境。生态提供共享配置入口，具体模型服务关系见 [AI 介绍](/zh/ai)。

## 当前实现与使用条件

主 Web 界面由 Auto 源码生成 Vue；默认 HTTP 后端由 Rust 基础设施与 Auto 生成的 Rust 组成。仓库也保留 AutoVM 后端和原生 UI 路径，不能据此宣称整个应用已经是纯 VM 或全部由 Auto 单独实现。

使用需要匹配的 Auto 工具链、Rust、Node.js/pnpm、相关仓库依赖，以及模型任务所需的 aaid 配置。CLI 的常见入口包括：

```sh
musk chat
musk run "阅读当前项目并总结结构"
musk serve
```

在目标项目目录里发起 CLI 任务；启动服务前按该版本的构建说明准备应用与前端，并设置合适的工作目录。网站的 Playground 是语言体验入口，不是已经部署的 AutoMusk 服务。

应用当前仍在开发与演示流程准备阶段。持久运行、重启恢复、自动修复循环和更完整的运行时规约收口属于后续工作，不在本页冒领。

介绍依据当前 README 与架构资料整理，资料时点为 **2026-10-01**。

[AI 的历史与展望](/zh/articles/auto-ai-history) · [AutoEdit](/zh/apps/autoedit/) · [返回应用总览](/zh/apps)
