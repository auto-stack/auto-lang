---
title: "便笺"
description: "用列表和正文编辑区记录短笔记，按文件夹、标签或置顶状态组织内容。适合轻量记录，也展示列表选择与编辑状态。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 便笺

用列表和正文编辑区记录短笔记，按文件夹、标签或置顶状态组织内容。适合轻量记录，也展示列表选择与编辑状态。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/notes.png" alt="便笺真实运行界面" caption="VM 原生窗口 · 内置便笺与编辑区。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 从左侧列表选择一条便笺。
2. 修改标题与正文，或新建记录。
3. 用文件夹、标签和置顶入口整理已有便笺。

## 运行条件与当前范围

需要便笺后端或本地 VM 后端逻辑；持久化行为以运行形态为准。

它是短笔记应用；页面关系与知识库工作由 JadeEdit 的独立介绍说明。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/015-notes](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/015-notes)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
