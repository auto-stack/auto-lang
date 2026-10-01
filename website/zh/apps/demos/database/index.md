---
title: "数据库界面示例"
description: "用树状结构浏览表与字段，再查看记录、编辑入口和查询面板。它展示数据工具的工作台结构。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 数据库界面示例

用树状结构浏览表与字段，再查看记录、编辑入口和查询面板。它展示数据工具的工作台结构。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/database.png" alt="数据库界面示例真实运行界面" caption="VM 原生窗口 · 内置表结构与示例记录。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 从数据库树选择一张表。
2. 查看列、记录和数据编辑入口。
3. 切换到查询面板，观察查询输入与结果的组织方式。

## 运行条件与当前范围

示例数据可在应用中展示；实际数据库连接、执行与持久化需要相应后端实现。

截图使用内置 Northwind 风格数据；SQLite 标题、查询与事务入口不证明已连到真实 SQLite 数据库，事务部分含模拟行为。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/026-database](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/026-database)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
