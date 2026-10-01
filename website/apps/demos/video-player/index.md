---
title: "Video player"
description: "A large viewport, timeline, and queue focus on local video playback, separate from the video community example."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Video player

A large viewport, timeline, and queue focus on local video playback, separate from the video community example.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/video-player.png" alt="Video player actual running interface" caption="Vue runtime · a real local WebM test video opened and positioned." :width="1440" :height="900" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**Vue** · 2026-10-01

## A typical workflow

1. Scan a local media directory or choose a playable video file.
2. Open the video and seek to the desired position.
3. Adjust volume, speed, or queue options and collapse the sidebar as needed.

## Runtime requirements and current scope

Needs real media and scanning/streaming services. Vue uses browser decoding; native playback additionally depends on libraries such as libmpv.

The capture decodes a local WebM test clip. A Vue capture does not verify every native format or playback capability.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/030-video-player](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/030-video-player)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
