---
title: "Music player"
description: "Local music scanning, a library, queue, and record stage present tracks and playback controls together."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Music player

Local music scanning, a library, queue, and record stage present tracks and playback controls together.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/music-player.png" alt="Music player actual running interface" caption="Native VM window connected to the real media scan service · local WAV test track." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Configure an accessible media directory and scan the library.
2. Select a track to inspect its title, format, and position.
3. Use playback controls, the queue, and volume to organize listening.

## Runtime requirements and current scope

Needs the real media scanning/streaming service, audio files, and playback support in the chosen runtime.

The capture uses a local WAV test tone. The Hi-Res interface label is not evidence of the file’s sample rate or audio quality.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/020-music-player](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/020-music-player)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
