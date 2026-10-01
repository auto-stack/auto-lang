---
title: "应用启动器"
description: "集中查找和启动应用，用列表或图标视图组织入口。桌面版本从宿主注册表读取应用，独立示例用于展示启动器交互。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 应用启动器

集中查找和启动应用，用列表或图标视图组织入口。桌面版本从宿主注册表读取应用，独立示例用于展示启动器交互。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/launcher.png" alt="应用启动器真实运行界面" caption="AutoOS VM 虚拟桌面 · 图标形式的真实启动器。" :width="2560" :height="1600" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · retained / 留存素材

## 一次典型操作

1. 打开启动器。
2. 输入关键词，或切换到图标视图浏览应用。
3. 选择应用启动，再返回已有桌面窗口。

## 运行条件与当前范围

真正启动应用需要 AutoOS 桌面宿主、应用注册与对应运行能力。

本页主图来自已拍摄的桌面启动器。独立 028 示例的数据与当前桌面应用注册表并不完全相同。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-os/apps/028-launcher](https://github.com/auto-stack/auto-os/tree/master/apps/028-launcher)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
