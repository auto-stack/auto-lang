---
title: "AutoOS 配置"
description: "用图形界面组织 AutoOS 与 AI 工具的配置，例如模型、角色、Agent、技能与主题。它让配置项可以按用途查找和管理。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# AutoOS 配置

用图形界面组织 AutoOS 与 AI 工具的配置，例如模型、角色、Agent、技能与主题。它让配置项可以按用途查找和管理。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/settings.png" alt="AutoOS 配置真实运行界面" caption="已有原生实拍资料 · 技能配置列表。" :width="1280" :height="720" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**Native configuration UI (existing website capture)** · retained / 留存素材

## 一次典型操作

1. 选择要配置的模块。
2. 查看已有条目与对应配置字段。
3. 修改后按应用提供的保存方式写回，再让使用方读取配置。

## 运行条件与当前范围

读写需要配置文件权限；模型、Agent 和守护进程的运行需要各自服务。

复用网站已有技能管理实拍图。配置界面里存在条目不等于对应服务已运行或模型已连通。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-os-config/.](https://github.com/auto-stack/auto-os-config/tree/master/)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
