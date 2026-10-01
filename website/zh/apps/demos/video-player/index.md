---
title: "视频播放器"
description: "以本地视频为中心，用大视口、进度条和播放队列控制观看。它与视频社区示例的内容门户用途不同。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 视频播放器

以本地视频为中心，用大视口、进度条和播放队列控制观看。它与视频社区示例的内容门户用途不同。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/video-player.png" alt="视频播放器真实运行界面" caption="Vue 运行界面 · 实际读取并定位本地 WebM 测试视频。" :width="1440" :height="900" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**Vue** · 2026-10-01

## 一次典型操作

1. 扫描本地媒体目录，或选择可播放的视频文件。
2. 打开视频，使用进度条定位到需要的位置。
3. 调整音量、倍速或队列，并在需要时收起侧栏。

## 运行条件与当前范围

需要真实媒体文件与扫描/流服务；Vue 使用浏览器解码，原生视频能力还取决于 libmpv 等运行库。

图中真实解码的是本地 WebM 测试片段。Vue 画面不等于已经验证所有原生格式与播放能力。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/030-video-player](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/030-video-player)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
