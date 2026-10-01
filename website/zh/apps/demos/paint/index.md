---
title: "像素画板"
description: "在 16×16 网格上逐格绘制，用画笔、橡皮、填充和取色器组织一幅小型像素画。颜色与历史操作都在应用状态中维护。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 像素画板

在 16×16 网格上逐格绘制，用画笔、橡皮、填充和取色器组织一幅小型像素画。颜色与历史操作都在应用状态中维护。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/paint.png" alt="像素画板真实运行界面" caption="VM 原生窗口 · 通过真实点击绘制的网格图案。" :width="669" :height="772" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 选择颜色，用画笔点击网格绘制。
2. 切换橡皮、填充或取色器调整图案。
3. 撤销、重做，或保存并重新载入画板。

## 运行条件与当前范围

本地交互不需要业务后台；保存依赖运行形态提供的 Storage。

当前以点击逐格绘制为主，不将它介绍为已有连续拖画和通用图像导出的绘图软件。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/031-paint](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/031-paint)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
