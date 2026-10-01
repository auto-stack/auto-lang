---
title: "System monitor"
description: "Inspect CPU, memory, and process information in a full window, with a compact desktop widget for ongoing status."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# System monitor

Inspect CPU, memory, and process information in a full window, with a compact desktop widget for ongoing status.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/system-monitor.png" alt="System monitor actual running interface" caption="Native VM window · actual capture-machine resource samples." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Read the resource usage overview.
2. Inspect process or resource panels for details.
3. Keep the app running and minimize it in the desktop to display its monitoring widget.

## Runtime requirements and current scope

Requires host system sampling; available measurements vary across platforms.

The capture records the capture machine at that moment, not the website visitor’s device.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-os/apps/025-sys-monitor](https://github.com/auto-stack/auto-os/tree/master/apps/025-sys-monitor)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
