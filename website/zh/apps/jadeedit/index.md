---
title: JadeEdit — 文档与本地知识工作台
description: AutoDown 文档编辑、页面链接、检索、组织和草稿恢复，以及当前开发边界。
outline: [2, 3]
editLink: false
---

# JadeEdit：文档与本地知识工作台

JadeEdit 是面向 AutoDown（`.ad`）文档和本地知识库的编辑器。它从写好一篇文档开始，把页面之间的链接、搜索、标签和目录组织逐步连接起来，服务于日常记录与个人知识整理。

它是独立应用，与 AutoEdit 长期并存。AutoEdit 侧重代码与文本审阅；JadeEdit 侧重写作和知识关系。AutoDown 提供文档与编辑器基础，旧 Jade Garden 则提供已有功能和设计经验；它们的名称不代表同一个产品。

[全部应用](/zh/apps) · [AutoDown 与 Jade Garden 资料](/zh/apps/autodown/)

<!-- Screenshot slot: JadeEdit main interface after its Milestone, using one coherent knowledge-base fixture. -->

## 从一篇文档开始

在工作区文件树中找到页面，打开、编辑、保存，并在多个标签之间浏览。近期页面和快速打开帮助回到正在处理的内容；文档可以承载 Markdown 写作结构与 YAML 元数据。

短期工作重点是编辑体验与文档操作的可靠性。在此基础上，一个典型知识整理过程是：写下内容 → 链接相关页面 → 查看引用关系 → 用标签和搜索重新找到资料。

## 连接页面，而不只保存文件

| 能力 | 用于什么 |
|---|---|
| 页面链接与反链 | 顺着相关页面阅读，查看哪些文档引用了当前页面 |
| 悬空链接与建页 | 发现尚不存在的链接目标，再创建对应页面 |
| 快速打开与全文检索 | 按页面名称、别名或内容查找资料 |
| 标签与页面属性 | 组织标题、标签、别名等信息，并按标签浏览 |
| 每日笔记 | 将日常记录放进可连续浏览的知识库 |

链接与显示名有各自的规则，移动页面、修改标题和重命名文件也不是同一种操作。页面改名会涉及引用更新，文件删除会留下待处理的悬空关系；应用逐步完善这些操作的预览、确认与保护。

<!-- Screenshot slot: links/backlinks and search, continuing with the same knowledge-base fixture. -->

## 组织工作区与保护未保存内容

文件和目录操作用于整理本地工作区，包括创建、重命名、移动、回收站以及部分目录合并处理。涉及多个页面或引用改写时，需要检查实际影响范围。

目前已有单文档重载保护、本地草稿检查点与重启后的恢复副本。恢复草稿时先打开副本，不直接覆盖原文件；草稿受到保护也不等于正文已经保存。多文档文件操作的保护仍在实施与收口中，本页不将其写成已经完全提供的数据安全保证。

<!-- Screenshot slot: draft recovery or a document-operation confirmation, after the corresponding behavior is accepted. -->

## 同一工程的 Web 与桌面路径

JadeEdit 用同一份 Auto 界面与应用源码组织 Web 和桌面形态。Vue 生成路径需要应用后端，原生路径由 AutoVM/AutoUI 渲染；编辑组件来自 AutoDown，部分公共组件与 AutoEdit 家族共用。

同源开发为两条路径提供共同基础，但不自动保证每个功能、版本和环境都有相同体验。实际支持情况仍以该版本的运行验证为准。

准备匹配的 Auto 工具链和 `auto-lang`、`auto-edit`、`auto-down` 等工程依赖后，仓库根目录的开发入口包括：

```sh
# 原生 VM 窗口
auto run -r vm

# 生成并构建 Vue 界面（需准备对应依赖）
pnpm build
```

Web 的文件与知识功能需要配套后端。只把生成的界面放在静态网站上，不能据此宣称读写、搜索和保存已经可用。

## 当前进展与后续方向

文档编辑、链接、检索、组织与草稿恢复已经有实现资料；Milestone 仍在收口，截图将在相应计划完成后补齐。大文档体验、跨页面操作保护和交付验证需要持续完善。

长期方向是更完整的知识工作台。图谱、块级协作、多人协作及与 AI 更深入的知识工作流，应按独立交付说明介绍，不把旧功能池和长期目标全部当作当前 JadeEdit 的完成项。

资料时点为 **2026-10-01**。

[AutoEdit 的代码与文本工作台](/zh/apps/autoedit/) · [AutoOS 的长期方向](/zh/articles/autoos-history) · [返回应用总览](/zh/apps)
