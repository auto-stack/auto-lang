---
title: "AutoTerm 终端"
description: "为桌面应用提供通用终端窗口，在标签页与分屏中运行宿主 Shell 或其他交互程序。它承载终端会话，AutoShell 则提供命令语言与执行引擎。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# AutoTerm 终端

为桌面应用提供通用终端窗口，在标签页与分屏中运行宿主 Shell 或其他交互程序。它承载终端会话，AutoShell 则提供命令语言与执行引擎。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/terminal.png" alt="AutoTerm 终端真实运行界面" caption="原生验证资料 · Windows Shell 会话（2026-09-22 留存图）。" :width="1100" :height="800" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**Rust native (retained verification capture)** · 2026-09-22

## 一次典型操作

1. 打开终端，进入配置的 Shell 会话。
2. 新建标签页，按需要横向或纵向分屏。
3. 在活动窗格中输入命令，查看输出与滚动记录。

## 运行条件与当前范围

原生交互需要 autoterm-core、平台 PTY 与配置的 Shell；Vue 视口当前为只读形态。

复用已有原生验证图，记录当时的 Windows 命令会话；截图版本与当前源码分开记录，不把它当作新的全部功能验收。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-term/app](https://github.com/auto-stack/auto-term/tree/master/app)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
