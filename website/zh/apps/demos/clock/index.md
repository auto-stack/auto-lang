---
title: "时钟"
description: "把时钟、世界时钟、闹钟、秒表与计时器放在一个应用中。桌面小组件让时间信息可以在窗口之外持续查看。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 时钟

把时钟、世界时钟、闹钟、秒表与计时器放在一个应用中。桌面小组件让时间信息可以在窗口之外持续查看。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/clock.png" alt="时钟真实运行界面" caption="VM 原生窗口 · 运行时的本地时间。" :width="720" :height="868" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 查看本地时间与日期。
2. 切换到世界时钟、闹钟、秒表或计时器。
3. 在桌面形态下保持应用运行，并最小化窗口查看小组件。

## 运行条件与当前范围

计时需要应用保持运行；小组件需要 AutoOS 桌面宿主。

独立时钟窗口与桌面小组件是两种呈现方式，小组件当前需打开应用后才显示。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/012-clock](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/012-clock)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
