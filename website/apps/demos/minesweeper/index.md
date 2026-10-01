---
title: "Minesweeper"
description: "Open safe cells and mark mines using numerical clues, with difficulty, timer, and remaining-marker information."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Minesweeper

Open safe cells and mark mines using numerical clues, with difficulty, timer, and remaining-marker information.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/minesweeper.png" alt="Minesweeper actual running interface" caption="Native VM window · actual beginner board with cells opened." :width="498" :height="637" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Choose an easy, medium, or hard board.
2. Open a cell and continue using its clues.
3. Mark suspect cells or start another game when needed.

## Runtime requirements and current scope

Basic play needs no external service; pointer and marking gestures follow the current interface.

The capture shows a real beginner board after opening a safe area. Difficulty and first-move behavior come from the application.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-os/apps/038-minesweeper](https://github.com/auto-stack/auto-os/tree/master/apps/038-minesweeper)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
