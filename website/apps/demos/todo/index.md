---
title: "Tasks"
description: "A task list for adding, completing, filtering, and clearing everyday items. It also introduces application state connected to a backend."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Tasks

A task list for adding, completing, filtering, and clearing everyday items. It also introduces application state connected to a backend.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/todo.png" alt="Tasks actual running interface" caption="Native VM window · four built-in example tasks." :width="960" :height="720" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Add a task using the input field.
2. Toggle completion and switch between all, active, and completed items.
3. Inspect the remaining count and clear completed items when needed.

## Runtime requirements and current scope

Requires the task backend or a local VM runtime that executes its backend logic.

The capture uses application seed tasks. This introduction page does not provide cross-device task synchronization.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/013-todo](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/013-todo)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
