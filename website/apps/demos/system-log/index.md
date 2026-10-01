---
title: "System log"
description: "A filterable list combines time, source, level, and message. The desktop host can supply system and application events."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# System log

A filterable list combines time, source, level, and message. The desktop host can supply system and application events.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/system-log.png" alt="System log actual running interface" caption="Standalone VM window · three built-in demonstration log records." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Inspect log times, sources, and levels.
2. Filter messages by level or keyword.
3. Return to the complete list and observe running events.

## Runtime requirements and current scope

Real host logs need a desktop-injected log source; standalone mode can display built-in records.

This standalone capture contains three demonstration records labeled mock. They are not actual operating-system failures or live host events.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-os/apps/039-syslog](https://github.com/auto-stack/auto-os/tree/master/apps/039-syslog)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
