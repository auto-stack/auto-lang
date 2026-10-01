---
title: "文件管理器"
description: "从目录导航进入列表或网格，按名称、大小和时间查找文件。文件操作与路径状态围绕真实文件系统组织。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 文件管理器

从目录导航进入列表或网格，按名称、大小和时间查找文件。文件操作与路径状态围绕真实文件系统组织。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/file-manager.png" alt="文件管理器真实运行界面" caption="VM 原生窗口 · 专用媒体测试目录的真实文件。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 通过地址栏打开一个可访问的目录。
2. 切换列表或网格，按需排序、搜索和显示隐藏项。
3. 选择文件查看信息；执行改名、复制或删除前检查目标路径。

## 运行条件与当前范围

真实读写需要文件系统能力和目录权限；Vue 需要对应文件 API。

截图浏览专用测试目录的真实文件。带有文件接口的本地运行与静态网页里的演示后备状态分开说明。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/027-file-manager](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/027-file-manager)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
