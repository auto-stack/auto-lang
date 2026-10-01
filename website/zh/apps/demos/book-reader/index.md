---
title: "图书阅读器"
description: "从书架进入图书、章节和阅读设置。书目、进度与阅读页面组成一个可导航的阅读应用结构。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 图书阅读器

从书架进入图书、章节和阅读设置。书目、进度与阅读页面组成一个可导航的阅读应用结构。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/book-reader.png" alt="图书阅读器真实运行界面" caption="VM 原生窗口连接实际阅读后端 · 三本示例图书。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 在书架中选择一本书。
2. 进入章节，阅读正文并切换章节。
3. 查看阅读进度与设置，再返回书架。

## 运行条件与当前范围

需要图书和章节 API；本地示例后端提供内置书目与正文。

截图是应用内置书架，不代表已经提供在线书城或通用电子书导入。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/018-book-reader](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/018-book-reader)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
