---
title: "博客阅读器"
description: "按分类浏览文章、进入正文，并查看作者等信息。阅读列表与文章页面也可以用于学习内容应用的组织方式。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 博客阅读器

按分类浏览文章、进入正文，并查看作者等信息。阅读列表与文章页面也可以用于学习内容应用的组织方式。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/blog.png" alt="博客阅读器真实运行界面" caption="VM 原生窗口 · 内置文章列表。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 从文章列表选择主题或分类。
2. 进入文章正文，阅读作者与内容。
3. 返回列表，继续查看其他文章。

## 运行条件与当前范围

需要文章后端或本地 VM 后端逻辑。

内容来自内置示例文章，页面不是外部博客聚合服务；写作等入口以具体版本实现为准。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/021-blog-viewer](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/021-blog-viewer)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
