---
title: "Pixel paint"
description: "Draw cell by cell on a 16×16 grid with pencil, eraser, fill, and picker tools, with colors and history kept in application state."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Pixel paint

Draw cell by cell on a 16×16 grid with pencil, eraser, fill, and picker tools, with colors and history kept in application state.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/paint.png" alt="Pixel paint actual running interface" caption="Native VM window · a grid pattern drawn through real clicks." :width="669" :height="772" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Choose a color and click grid cells to draw.
2. Switch to eraser, fill, or picker to adjust the pattern.
3. Undo, redo, or save and reload the canvas.

## Runtime requirements and current scope

Local interaction needs no business backend; saving uses the runtime’s Storage support.

The current interaction is cell clicking. This is not presented as continuous drag painting or a general image-export editor.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/031-paint](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/031-paint)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
