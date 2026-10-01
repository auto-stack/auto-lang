---
title: "待办"
description: "用一张任务列表管理日常事项：新增、完成、筛选，再清理已完成项目。它也展示前端状态如何与应用后端联动。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 待办

用一张任务列表管理日常事项：新增、完成、筛选，再清理已完成项目。它也展示前端状态如何与应用后端联动。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/todo.png" alt="待办真实运行界面" caption="VM 原生窗口 · 应用内置的四条示例任务。" :width="960" :height="720" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 在输入框中添加一项任务。
2. 勾选完成状态，切换全部、未完成和已完成列表。
3. 检查剩余数量，按需清理已完成任务。

## 运行条件与当前范围

需要任务后端或支持应用后端逻辑的本地 VM 运行环境。

截图使用应用内置任务数据；不会向访问这个介绍页的人提供跨设备任务同步。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/013-todo](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/013-todo)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
