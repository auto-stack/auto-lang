---
title: "AutoTerm terminal"
description: "A general desktop terminal runs host shells and interactive programs in tabs and split panes. AutoTerm hosts terminal sessions; AutoShell supplies a command language and engine."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# AutoTerm terminal

A general desktop terminal runs host shells and interactive programs in tabs and split panes. AutoTerm hosts terminal sessions; AutoShell supplies a command language and engine.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/terminal.png" alt="AutoTerm terminal actual running interface" caption="Native verification material · Windows shell session (retained 2026-09-22 image)." :width="1100" :height="800" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**Rust native (retained verification capture)** · 2026-09-22

## A typical workflow

1. Open the terminal into its configured shell.
2. Create another tab and split panes horizontally or vertically as needed.
3. Enter commands in the active pane and inspect output and scrollback.

## Runtime requirements and current scope

Native interaction needs autoterm-core, platform PTY support, and a configured shell. The Vue viewport is currently read-only.

An existing native verification image records a Windows command session. Its capture version is tracked separately from current sources and is not a new full feature acceptance.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-term/app](https://github.com/auto-stack/auto-term/tree/master/app)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
