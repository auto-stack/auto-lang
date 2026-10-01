---
title: "音乐播放器"
description: "扫描本地音乐，结合曲库、播放队列与唱片舞台查看当前曲目。媒体数据和播放控制在同一界面中呈现。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 音乐播放器

扫描本地音乐，结合曲库、播放队列与唱片舞台查看当前曲目。媒体数据和播放控制在同一界面中呈现。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/music-player.png" alt="音乐播放器真实运行界面" caption="VM 原生窗口连接真实媒体扫描服务 · 本地 WAV 测试曲目。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 配置可访问的媒体目录并扫描曲库。
2. 选择一首曲目，查看标题、格式与播放位置。
3. 用播放控制、队列和音量入口安排收听。

## 运行条件与当前范围

需要实际媒体扫描/流服务、音频文件和运行形态支持的播放能力。

截图使用本地 WAV 测试音频；界面上的 Hi-Res 文案不能作为文件实际采样率或音质的证明。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/020-music-player](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/020-music-player)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
