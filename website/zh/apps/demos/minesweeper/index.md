---
title: "扫雷"
description: "通过数字线索打开安全格、标记地雷，按难度调整棋盘。计时和剩余标记数帮助查看当前局面。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 扫雷

通过数字线索打开安全格、标记地雷，按难度调整棋盘。计时和剩余标记数帮助查看当前局面。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/minesweeper.png" alt="扫雷真实运行界面" caption="VM 原生窗口 · 实际打开格子后的初级棋盘。" :width="498" :height="637" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 选择初级、中级或高级棋盘。
2. 打开一个格子，按线索继续探索。
3. 用标记动作标出可疑格，必要时重新开始。

## 运行条件与当前范围

基本游戏不需要外部服务；鼠标操作与标记方式以当前界面提示为准。

主图显示一局打开安全区域后的真实棋盘。难度与首步安全等行为由应用实现提供。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-os/apps/038-minesweeper](https://github.com/auto-stack/auto-os/tree/master/apps/038-minesweeper)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
