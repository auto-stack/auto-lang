---
title: "聊天"
description: "联系人、消息记录与输入区组成一个即时通信界面示例。它用应用自己的后端解释会话选择、发消息与事件更新。"
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# 聊天

联系人、消息记录与输入区组成一个即时通信界面示例。它用应用自己的后端解释会话选择、发消息与事件更新。

[全部应用](/zh/apps) · [28 个系统应用与示例](/zh/apps/demos/)

## 先看界面

<EvidenceImage src="/apps/demos/chat.png" alt="聊天真实运行界面" caption="VM 原生窗口连接实际聊天后端 · 内置联系人与消息。" :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

截图形态：**VM** · 2026-10-01

## 一次典型操作

1. 选择联系人，阅读对应会话。
2. 输入消息或选择快捷回复，再发送。
3. 观察消息列表与联系人摘要的变化。

## 运行条件与当前范围

需要聊天 API；事件通知还依赖对应的流服务与运行形态。

联系人与对话是内置演示内容；AutoBot 标签不代表已接入真实模型服务，也没有连接微信网络。

这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。

## 源码与相关介绍

源码工程：[auto-lang/examples/ui/017-chat](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/017-chat)

[AutoOS 虚拟桌面](/zh/autoos/) · [AutoUI](/zh/ui) · [返回应用目录](/zh/apps/demos/)

介绍资料时点：**2026-10-01**。新拍图与留存图的日期、运行形态分别记录。
