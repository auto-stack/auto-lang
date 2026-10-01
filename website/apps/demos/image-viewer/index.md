---
title: "Image viewer"
description: "Open an image or directory and inspect it using fit, zoom, rotation, and navigation in a large viewport."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Image viewer

Open an image or directory and inspect it using fit, zoom, rotation, and navigation in a large viewport.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/image-viewer.png" alt="Image viewer actual running interface" caption="Native VM window · a real 320×240 test image opened." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Open an image or directory and select an image.
2. Use fit, 1:1, or zoom controls to inspect details.
3. Rotate or pan the image, then move to a neighboring file.

## Runtime requirements and current scope

Requires accessible image files; supported formats and decoding depend on the runtime libraries.

This is a viewer rather than an image editor. The capture opens scenic-320x240.png from the source test fixtures.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/031-image-viewer](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/031-image-viewer)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
