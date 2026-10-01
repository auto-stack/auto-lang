---
title: "视频社区示例"
description: "用视频卡片、分类和观看页面组织内容。它展示类似视频门户的页面结构，与直接打开本地文件的视频播放器分开介绍。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 视频社区示例

用视频卡片、分类和观看页面组织内容。它展示类似视频门户的页面结构，与直接打开本地文件的视频播放器分开介绍。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/video-app.png" alt="视频社区示例真实运行界面" caption="VM 原生窗口 · 内置视频内容卡片。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 在首页浏览视频卡片与分类。
2. 选择一条内容进入观看页面。
3. 查看内容信息，再返回首页。

## 运行条件与当前范围

列表和内容信息需要应用后端；真正播放还需要有效媒体资源与解码能力。

截图中的标题与卡片来自内置数据，只记录内容浏览界面，不证明每条视频都能播放。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/019-video-app](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/019-video-app)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
