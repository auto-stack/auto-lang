---
title: "Video community example"
description: "Video cards, categories, and watch pages introduce a portal-style content application, distinct from the local-file video player."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Video community example

Video cards, categories, and watch pages introduce a portal-style content application, distinct from the local-file video player.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/video-app.png" alt="Video community example actual running interface" caption="Native VM window · built-in video content cards." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Browse video cards and categories on the home page.
2. Choose an item to open its watch page.
3. Inspect its information and return to the home page.

## Runtime requirements and current scope

Lists and metadata need the application backend; playback additionally needs valid media and decoding support.

The capture shows built-in titles and cards. It records content browsing, not successful playback of every listed video.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/019-video-app](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/019-video-app)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
