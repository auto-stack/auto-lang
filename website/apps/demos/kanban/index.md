---
title: "Kanban example"
description: "Workspaces, columns, and cards organize task states and introduce how a task moves between stages."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Kanban example

Workspaces, columns, and cards organize task states and introduce how a task moves between stages.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/kanban.png" alt="Kanban example actual running interface" caption="Native VM window · built-in workspace and task cards." :width="1680" :height="1110" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Select a workspace or board.
2. Inspect cards in different columns and open task details.
3. Adjust task content or state and observe the board.

## Runtime requirements and current scope

Requires the example backend or local VM backend logic.

This is the 022-kanban example, distinct from the separate auto-kanban product project. The capture uses built-in workspace data.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/022-kanban](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/022-kanban)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
