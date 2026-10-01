---
title: "日历"
description: "用月历查看日期、前后切换月份，并保留当前选择。简洁的日期网格适合日常查阅，也可用于理解日期选择组件。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 日历

用月历查看日期、前后切换月份，并保留当前选择。简洁的日期网格适合日常查阅，也可用于理解日期选择组件。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/calendar.png" alt="日历真实运行界面" caption="VM 原生窗口 · 示例初始月份与选定日期。" :width="720" :height="717" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 查看当前展示的月份。
2. 用左右箭头切换月份，选择一个日期。
3. 通过 Today 返回应用的今日定位。

## 运行条件与当前范围

基础月历不需要业务后台；桌面小组件依赖宿主与运行中的应用。

这是月历与日期选择示例，截图中的选定日期来自应用示例初始状态；未将它介绍为完整日程协作服务。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/016-calendar](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/016-calendar)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
