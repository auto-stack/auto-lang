---
title: "纸牌接龙"
description: "在牌堆、桌面列与收集区之间移动纸牌，按接龙规则整理一副牌。窗口中也有撤销、重开与外观设置入口。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 纸牌接龙

在牌堆、桌面列与收集区之间移动纸牌，按接龙规则整理一副牌。窗口中也有撤销、重开与外观设置入口。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/solitaire.png" alt="纸牌接龙真实运行界面" caption="VM 原生窗口 · 实际发牌后的接龙牌桌。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 开始一局，观察桌面列与发牌堆。
2. 点击发牌或按规则移动纸牌。
3. 撤销一步、重新发牌，或查看游戏记录。

## 运行条件与当前范围

基本游戏需要 AutoUI 运行环境；记录的持久化取决于对应后端或存储。

主图是真实发牌后的窗口，不将一张截图当作整局胜利或全部规则验证。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-os/apps/037-klondike](https://github.com/auto-stack/auto-os/tree/master/apps/037-klondike)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
