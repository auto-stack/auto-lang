---
title: "看板示例"
description: "用工作区、列和卡片整理任务状态，理解任务从一个阶段进入下一阶段的过程。它也是较完整的应用界面结构示例。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 看板示例

用工作区、列和卡片整理任务状态，理解任务从一个阶段进入下一阶段的过程。它也是较完整的应用界面结构示例。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/kanban.png" alt="看板示例真实运行界面" caption="VM 原生窗口 · 内置工作区与任务卡片。" :width="1680" :height="1110" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 选择工作区或看板。
2. 查看不同列中的卡片，进入任务详情。
3. 调整任务内容或状态，观察看板变化。

## 运行条件与当前范围

需要示例后端或本地 VM 后端逻辑。

这是 022-kanban 示例，与独立的 auto-kanban 产品工程分开介绍；截图使用内置工作区数据。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/022-kanban](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/022-kanban)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
