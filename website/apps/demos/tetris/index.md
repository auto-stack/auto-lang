---
title: "Tetris"
description: "Falling blocks, rotation, movement, and dropping introduce a real-time game, with score, level, and next-piece information around the board."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Tetris

Falling blocks, rotation, movement, and dropping introduce a real-time game, with score, level, and next-piece information around the board.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/tetris.png" alt="Tetris actual running interface" caption="Native VM window · actual board after starting a game." :width="816" :height="1101" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Start the game and inspect the current and next pieces.
2. Move and rotate with the keys shown by the interface, then drop a piece.
3. Pause or restart and inspect scores and records.

## Runtime requirements and current scope

Requires continuous timing and keyboard input; records use the corresponding storage or backend.

The capture shows an actual started game. Piece positions change as the game runs.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-os/apps/036-tetris](https://github.com/auto-stack/auto-os/tree/master/apps/036-tetris)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
