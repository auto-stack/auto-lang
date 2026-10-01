---
title: "Clock"
description: "Clock, world time, alarms, stopwatch, and timer share one application. A desktop widget keeps time visible outside its main window."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Clock

Clock, world time, alarms, stopwatch, and timer share one application. A desktop widget keeps time visible outside its main window.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/clock.png" alt="Clock actual running interface" caption="Native VM window · local time at capture." :width="720" :height="868" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Inspect the local time and date.
2. Select world time, alarms, stopwatch, or timer.
3. In the desktop runtime, keep the app running and minimize it to see its widget.

## Runtime requirements and current scope

Timers require a running application; desktop widgets require the AutoOS host.

The standalone window and desktop widget are separate presentations. The widget currently appears after launching the application.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/012-clock](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/012-clock)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
