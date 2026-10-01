---
title: "Application launcher"
description: "Find and launch applications in list or icon form. The desktop version uses the host registry; the standalone example introduces launcher interaction."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Application launcher

Find and launch applications in list or icon form. The desktop version uses the host registry; the standalone example introduces launcher interaction.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/launcher.png" alt="Application launcher actual running interface" caption="AutoOS VM virtual desktop · actual launcher in icon view." :width="2560" :height="1600" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · retained / 留存素材

## A typical workflow

1. Open the launcher.
2. Search by keyword or browse applications in icon view.
3. Choose an application to launch and return to existing desktop windows.

## Runtime requirements and current scope

Actual launching requires the AutoOS desktop host, application registration, and the relevant runtime capabilities.

The main image is an existing desktop launcher capture. The standalone 028 example’s data is not identical to the current desktop registry.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-os/apps/028-launcher](https://github.com/auto-stack/auto-os/tree/master/apps/028-launcher)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
