---
title: "系统监视器"
description: "查看 CPU、内存与进程等系统信息。独立窗口与桌面小组件把较完整的监控视图和简短状态显示分开。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 系统监视器

查看 CPU、内存与进程等系统信息。独立窗口与桌面小组件把较完整的监控视图和简短状态显示分开。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/system-monitor.png" alt="系统监视器真实运行界面" caption="VM 原生窗口 · 拍摄机器的实际资源采样。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 查看资源使用概况。
2. 进入进程或资源面板，检查具体条目。
3. 在桌面中保持应用运行并最小化，以查看监控小组件。

## 运行条件与当前范围

需要宿主系统采样能力；不同平台的可用指标可能不同。

截图采样的是拍摄机器当时的状态，不是网站访客设备的信息。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-os/apps/025-sys-monitor](https://github.com/auto-stack/auto-os/tree/master/apps/025-sys-monitor)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
