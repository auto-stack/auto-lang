---
title: "RealWorld 应用示例"
description: "用文章社区的结构展示路由、账号入口、文章列表与详情。多页面与前后端数据链比单个组件更接近完整应用。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# RealWorld 应用示例

用文章社区的结构展示路由、账号入口、文章列表与详情。多页面与前后端数据链比单个组件更接近完整应用。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/realworld.png" alt="RealWorld 应用示例真实运行界面" caption="Vue 画廊运行连接实际后端 · 内置文章与标签。" :width="1022" :height="718" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**Vue** · 2026-10-01

## 一次典型操作

1. 在 Global Feed 浏览文章与标签。
2. 进入一篇文章，查看内容与评论入口。
3. 按需要研究登录、写作等路由及其后端契约。

## 运行条件与当前范围

需要 RealWorld API；账号与内容操作使用示例后端，不连接公开社区。

主图来自连接真实示例后端的 Vue 运行，文章为内置内容。其他运行形态的覆盖需单独验证。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/023-realworld](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/023-realworld)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
