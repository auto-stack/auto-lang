---
title: "照片图库"
description: "直接扫描配置目录，用照片网格浏览图片和子目录，再进入大图查看。搜索、排序、密度与收藏帮助整理浏览过程。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 照片图库

直接扫描配置目录，用照片网格浏览图片和子目录，再进入大图查看。搜索、排序、密度与收藏帮助整理浏览过程。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/photo-gallery.png" alt="照片图库真实运行界面" caption="VM 原生窗口连接实际照片服务 · 扫描三张公开 AutoOS 截图。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 配置照片根目录，让后端扫描真实图片。
2. 选择子目录或搜索关键词，调整网格密度与排序。
3. 打开大图，前后查看或收藏，再返回网格。

## 运行条件与当前范围

需要照片扫描、缩略图和原图服务，以及有访问权限的本地图片目录。

当前目录逐层浏览，收藏可跨目录聚合。截图扫描的是公开 AutoOS 截图文件，不使用私人照片。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/029-photo-gallery](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/029-photo-gallery)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
