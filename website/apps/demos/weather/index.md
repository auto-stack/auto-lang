---
title: "Weather"
description: "City weather, hourly changes, and related measurements introduce information layout and desktop widgets."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Weather

City weather, hourly changes, and related measurements introduce information layout and desktop widgets.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/weather.png" alt="Weather actual running interface" caption="Native VM window · built-in demonstration weather for Beijing." :width="1440" :height="1020" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Select a city and inspect its temperature and overview.
2. Read hourly conditions, humidity, wind, and other measurements.
3. Launch the app in the desktop host to display its weather widget.

## Runtime requirements and current scope

Built-in weather data can be shown offline; the widget requires the AutoOS host.

The current data is demonstration weather, not a live weather feed or travel forecast.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/014-weather](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/014-weather)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
