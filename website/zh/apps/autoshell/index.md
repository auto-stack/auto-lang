---
title: AutoShell — 命令、数据管道与 AutoScript
description: 在同一个 Shell 中处理日常命令、结构化数据和多行 Auto 脚本。
outline: [2, 3]
editLink: false
---

# AutoShell：命令、数据管道与 AutoScript

AutoShell（命令名 `ash`）将日常命令、结构化数据管道和 AutoScript 放在同一套执行引擎中。可以保持一个交互会话，也可以通过 CLI 执行单条命令或脚本文件。

它的使用重点是把探索与重复工作连起来：先查看文件或数据，按字段筛选和转换，再将稳定的操作写成脚本。

[全部应用](/zh/apps) · [使用指南与实跑示例](/zh/apps/autoshell/guide/)

<!-- Screenshot slot: ash-01 native colored ls table is the approved main capture. Main introduction remains image-free for this writing phase; existing captures are retained in guide. -->

## 在一个会话里连续工作

目录、变量和别名可以在命令之间延续。补全、历史搜索和行内建议减少重复输入；管道、重定向、命令链和后台作业用于组合日常工作。

例如，先用 `ls` 浏览目录，再筛选需要的文件，最后将结果输出给其他程序。交互式使用与 `ash -c` 调用共用引擎，便于将试过的命令放进自动化。

## 让命令传递结构化记录

一些内置命令返回带字段的记录，而不是只输出一段文本。`ls` 的文件记录、JSON 解析结果，都可以继续筛选、排序、投影或转换。

```sh
cat users.json | from_json | .age > 30 | select .name .age
```

这个例子读取 JSON，按年龄过滤，再保留姓名和年龄。数据文件、可复制命令与实际结果见 [结构化管道示例](/zh/apps/autoshell/guide/#data-pipelines)。外部命令的输出与结构化记录并不总是同一种数据，需要按具体命令处理。

## 从一条命令扩展成脚本

F2 打开多行 AutoScript 编辑器，可编写函数、条件和循环。Enter 换行，F5 执行，运行结束后回到提示符；Ctrl+Enter 的识别取决于终端是否传递修饰键。

脚本可以捕获查询结果，再计算、循环或汇总。保存为 `.ash` 文件后，可以用 CLI 重复运行。指南保留了函数分类、金额汇总和 JSON 查询三个完整示例。

<!-- Screenshot slot: F2 script input followed by its real output; existing recorded examples stay in guide. -->

## 命令、脚本与 AI 入口

| 入口 | 用途 |
|---|---|
| F1 | 锁定 Shell 命令模式 |
| F2 | 编辑并运行多行 AutoScript |
| F3 | 进入 AI 对话输入区，需要可用的模型服务配置 |

F3 提供 AI 会话入口；示例中的输入区截图并不代表已经完成外部模型调用。更复杂的开发任务组织可以了解 [AutoMusk](/zh/apps/automusk/)，共享模型服务关系见 [AutoAI](/zh/ai)。

## 自动化与执行范围

CLI 支持 JSON 输出、只读、执行预览、路径范围和命令限制，以及审计记录。这些选项帮助程序调用与人工检查，但不同限制有各自覆盖范围：ash 管理的文件操作受其策略约束，外部程序自身的文件访问需要操作系统隔离配合。

当前 Agent 集成可以使用命令调用、JSON 结果和执行策略；旧资料中的部分专用 Agent 子命令尚未实现，不作为当前使用入口。

## 开始使用与当前边界

构建需要 Rust 及对应版本的相关仓库依赖。将构建出的 `ash` 加入 PATH 后：

```sh
ash
ash --json -c 'ls'
ash user-report.ash
```

目前专题中的实跑资料覆盖 Windows CLI 与交互式 Shell，捕获日期为 2026-09-29/30，程序版本为 ash v0.1.0。其他平台和独立 GUI/TUI 的功能覆盖需分别确认。网站的 AutoLang v0.5 发布号与 ash 程序版本分别标注。

完整构建步骤、当前查询语法、脚本下载，以及已观察到的 CSV 等限制，见 [使用指南](/zh/apps/autoshell/guide/#quick-start)。本页介绍整理于 **2026-10-01**。

[查看真实终端与操作示例](/zh/apps/autoshell/guide/#interface-overview) · [AutoOS](/zh/os) · [返回应用总览](/zh/apps)
