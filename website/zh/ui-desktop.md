---
title: AutoUI 桌面运行
description: AutoUI 的浏览器、原生 VM 与编译桌面路径。
---

<script setup>
import IntroductionFrame from '../.vitepress/theme/components/IntroductionFrame.vue'
</script>

<IntroductionFrame kind="uiDesktop" />

## 浏览器与原生预览

AutoUI 已有桌面运行路径。`auto run` 使用 Vue/浏览器路径；`auto run -r vm` 在 VM 中执行 Auto UI 源码，以 iced 提供原生界面。两条路径共享 UI 定义，渲染器分别提供平台设施。

原生开发支持热更新。具体路径和修改类型决定哪些运行状态能够保留。共享定义以布局、交互和主题一致为目标，不保证各平台的文本栅格化完全相同。

## 编译应用与合成

代码生成和 a2r 提供编译路径。桌面壳、应用渲染源与合成宿主通过窗口和渲染契约连接。RenderQueue 与 RQHost 的工作属于这一边界，并不意味着所有已有 VM 应用都已迁移为原生执行。

`auto-os` 组装桌面与应用，`auto-lang` 实现语言和框架。当前启动清单主要声明 VM 应用；宿主接入和平台覆盖需要按路径、功能分别核实。

## 从应用 UI 到 AutoOS

虚拟桌面在应用 UI 之外组织共同的窗口管理、工作区、启动器与通知，并复用已有宿主系统。Linux 合成宿主已有早期工作，独立系统仍是长期方向。

[查看真实桌面](/zh/autoos/) · [阅读 OS 总览](/zh/os) · [桌面架构](/zh/docs/design/autoui/virtual-desktop) · [编译壳设计](/zh/docs/design/autoui/desktop-shell-a2r) · [AutoUI 总览](/zh/ui)
