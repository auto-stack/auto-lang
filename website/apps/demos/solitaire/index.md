---
title: "Solitaire"
description: "Move cards between stock, tableau, and foundations using Klondike rules, with undo, restart, and appearance controls."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Solitaire

Move cards between stock, tableau, and foundations using Klondike rules, with undo, restart, and appearance controls.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/solitaire.png" alt="Solitaire actual running interface" caption="Native VM window · actual dealt Klondike board." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Start a game and inspect the tableau and stock.
2. Draw cards or move them according to the rules.
3. Undo a move, deal again, or inspect game records.

## Runtime requirements and current scope

Basic play needs an AutoUI runtime; persistent records depend on its backend or storage.

The main image shows an actual dealt board. It does not establish a completed game or full rule verification.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-os/apps/037-klondike](https://github.com/auto-stack/auto-os/tree/master/apps/037-klondike)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
