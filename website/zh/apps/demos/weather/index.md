---
title: "天气"
description: "按城市浏览当前天气、小时变化与天气指标。图标、信息层级和桌面小组件用于解释一组天气数据。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 天气

按城市浏览当前天气、小时变化与天气指标。图标、信息层级和桌面小组件用于解释一组天气数据。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/weather.png" alt="天气真实运行界面" caption="VM 原生窗口 · 北京的内置演示天气。" :width="1440" :height="1020" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 选择城市，查看温度与天气概况。
2. 阅读小时预报、湿度、风速等指标。
3. 在桌面宿主中打开应用后查看天气小组件。

## 运行条件与当前范围

当前内置天气数据可离线展示；小组件依赖 AutoOS 宿主。

当前数据是演示天气，不能用于判断实时天气或出行条件。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/014-weather](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/014-weather)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
