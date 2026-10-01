---
title: "图片查看器"
description: "打开图片或目录，以适应窗口、缩放、旋转与前后导航检查图像。侧栏呈现文件选择，大视口负责查看。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 图片查看器

打开图片或目录，以适应窗口、缩放、旋转与前后导航检查图像。侧栏呈现文件选择，大视口负责查看。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/image-viewer.png" alt="图片查看器真实运行界面" caption="VM 原生窗口 · 实际打开 320×240 测试图像。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 打开图片或目录，选中一张图像。
2. 使用适应窗口、1:1 或缩放控制查看细节。
3. 旋转或平移图像，再切换到相邻文件。

## 运行条件与当前范围

需要可访问的图片文件；支持格式与解码能力以对应运行库为准。

它用于查看而非修图；截图打开源码测试目录中的 scenic-320x240.png。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/031-image-viewer](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/031-image-viewer)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
