---
title: "计算器"
description: "从日常四则运算开始，也可以切换科学计算与程序员模式。键盘、数字按钮和结果区组成一个紧凑的工具窗口。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 计算器

从日常四则运算开始，也可以切换科学计算与程序员模式。键盘、数字按钮和结果区组成一个紧凑的工具窗口。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/calculator.png" alt="计算器真实运行界面" caption="VM 原生窗口 · 基础计算模式。" :width="576" :height="678" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 输入数字和运算符，查看表达式与结果。
2. 用 Basic / Scientific / Programmer 切换计算模式。
3. 清空当前输入，开始下一次计算。

## 运行条件与当前范围

基础运算不需要业务后台；需要对应的 AutoUI 运行环境。

它是计算工具与输入交互示例；不同模式的运算覆盖以对应版本实现为准。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/011-calculator](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/011-calculator)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
