---
title: "Chat"
description: "Contacts, message history, and an input area introduce a messaging interface connected to its own application backend."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Chat

Contacts, message history, and an input area introduce a messaging interface connected to its own application backend.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/chat.png" alt="Chat actual running interface" caption="Native VM window connected to the actual chat backend · built-in contacts and messages." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Select a contact and read the conversation.
2. Enter a message or choose a quick reply, then send it.
3. Observe changes to the conversation and contact summary.

## Runtime requirements and current scope

Requires the chat API; event notifications also need the corresponding streaming service and runtime support.

Contacts and messages are built-in examples. The AutoBot label does not establish a live model connection, and this app is not connected to WeChat.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/017-chat](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/017-chat)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
