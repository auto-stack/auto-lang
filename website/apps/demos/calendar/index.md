---
title: "Calendar"
description: "A month grid for looking up dates, switching months, and keeping the current selection."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Calendar

A month grid for looking up dates, switching months, and keeping the current selection.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/calendar.png" alt="Calendar actual running interface" caption="Native VM window · initial sample month and selected date." :width="720" :height="717" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Inspect the displayed month.
2. Move between months and select a date.
3. Use Today to return to the application’s current-day location.

## Runtime requirements and current scope

The month view needs no business backend; a desktop widget needs the host and a running app.

This introduces a calendar and date selection. The captured initial month is sample state, not a full collaborative scheduling service.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/016-calendar](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/016-calendar)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
