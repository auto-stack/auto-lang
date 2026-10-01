---
title: "File manager"
description: "Navigate directories in list or grid form and find files by name, size, or date, with operations organized around a real filesystem."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# File manager

Navigate directories in list or grid form and find files by name, size, or date, with operations organized around a real filesystem.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/file-manager.png" alt="File manager actual running interface" caption="Native VM window · real files in a dedicated media fixture directory." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Open an accessible directory using the address bar.
2. Switch list/grid views and use sorting, search, or hidden-file controls.
3. Inspect selected files and check target paths before rename, copy, or delete operations.

## Runtime requirements and current scope

Real reads and writes require filesystem capabilities and permissions; Vue needs the corresponding file API.

The capture browses real files in a dedicated fixture directory. Local filesystem use is distinct from static-web fallback demonstration state.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/027-file-manager](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/027-file-manager)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
