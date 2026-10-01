---
title: "系统日志"
description: "把时间、来源、等级和消息组成可筛选的日志列表。桌面宿主可以将系统与应用事件接入同一查看器。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 系统日志

把时间、来源、等级和消息组成可筛选的日志列表。桌面宿主可以将系统与应用事件接入同一查看器。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/system-log.png" alt="系统日志真实运行界面" caption="VM 独立窗口 · 应用内置的三条演示日志。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 查看日志时间、来源与等级。
2. 按等级或关键词筛选消息。
3. 回到完整列表，持续观察运行中的事件。

## 运行条件与当前范围

真实宿主日志需要桌面注入的日志源；独立示例可展示内置记录。

本次截图是独立运行的三条演示日志，图中也标为 mock；它们不是操作系统实际故障或实时宿主事件。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-os/apps/039-syslog](https://github.com/auto-stack/auto-os/tree/master/apps/039-syslog)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
