---
title: "图表工坊"
description: "在同一工作台中切换图表类型、数据系列和展示形态，观察数据与图形之间的关系。适合研究图表组件的配置与状态。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 图表工坊

在同一工作台中切换图表类型、数据系列和展示形态，观察数据与图形之间的关系。适合研究图表组件的配置与状态。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/charts.png" alt="图表工坊真实运行界面" caption="VM 原生窗口 · 本地示例数据绘制的折线图。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 选择折线、柱形等图表类型。
2. 切换数据系列与桌面/移动/平板展示设置。
3. 播放或重置示例数据，观察图表更新。

## 运行条件与当前范围

当前工作台使用本地示例数据；实时业务数据需要应用另行接入。

主图展示的是内置销售等示例系列，不是项目真实经营指标。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/024-charts](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/024-charts)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
