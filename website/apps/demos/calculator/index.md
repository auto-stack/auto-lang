---
title: "Calculator"
description: "A compact tool for everyday arithmetic, with scientific and programmer modes available from the same window."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Calculator

A compact tool for everyday arithmetic, with scientific and programmer modes available from the same window.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/calculator.png" alt="Calculator actual running interface" caption="Native VM window · basic calculation mode." :width="576" :height="678" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Enter numbers and operators to inspect the expression and result.
2. Switch between Basic, Scientific, and Programmer modes.
3. Clear the current input and start a new calculation.

## Runtime requirements and current scope

Basic arithmetic needs an AutoUI runtime and no application backend.

This introduces calculator input and modes; operator coverage depends on the implementation version.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/011-calculator](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/011-calculator)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
