---
title: "俄罗斯方块"
description: "用下落方块、旋转、移动和落下组织一个实时小游戏。分数、等级与下一块信息围绕棋盘呈现。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 俄罗斯方块

用下落方块、旋转、移动和落下组织一个实时小游戏。分数、等级与下一块信息围绕棋盘呈现。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/tetris.png" alt="俄罗斯方块真实运行界面" caption="VM 原生窗口 · 开始游戏后的实际棋盘。" :width="816" :height="1101" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 开始游戏，观察当前方块和下一块。
2. 用方向键移动和旋转，按界面提示快速落下。
3. 暂停或重新开始，查看得分与记录。

## 运行条件与当前范围

游戏需要持续计时与键盘输入；记录功能需要对应存储或后端。

截图是真实启动后的棋盘状态，运行中的方块位置会随时间变化。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-os/apps/036-tetris](https://github.com/auto-stack/auto-os/tree/master/apps/036-tetris)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
