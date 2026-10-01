---
title: "Photo gallery"
description: "Scan a configured directory, browse images and subdirectories in a grid, then open a larger view with search, sorting, density, and favorites."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Photo gallery

Scan a configured directory, browse images and subdirectories in a grid, then open a larger view with search, sorting, density, and favorites.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/photo-gallery.png" alt="Photo gallery actual running interface" caption="Native VM window connected to the actual photo service · three public AutoOS screenshot files." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Configure a photo root and let the backend scan real images.
2. Choose a subdirectory or search term and adjust density or sorting.
3. Open an image, move between images or favorite it, then return to the grid.

## Runtime requirements and current scope

Requires scan, thumbnail, and full-image services plus an accessible local image directory.

Browsing proceeds directory by directory; favorites can span directories. The capture scans public AutoOS screenshots rather than personal photos.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/029-photo-gallery](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/029-photo-gallery)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
