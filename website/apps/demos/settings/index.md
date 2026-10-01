---
title: "AutoOS settings"
description: "A graphical workspace organizes AutoOS and AI-tool settings such as models, roles, agents, skills, and themes."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# AutoOS settings

A graphical workspace organizes AutoOS and AI-tool settings such as models, roles, agents, skills, and themes.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/settings.png" alt="AutoOS settings actual running interface" caption="Existing native capture material · skills configuration list." :width="1280" :height="720" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**Native configuration UI (existing website capture)** · retained / 留存素材

## A typical workflow

1. Select the configuration module to manage.
2. Inspect existing entries and their fields.
3. Save changes using the application’s controls, then let the consuming tool read them.

## Runtime requirements and current scope

Read/write operations require configuration-file permissions; models, agents, and daemons need their own services.

This reuses the website’s existing skills-management capture. An entry in settings does not establish a running service or model connection.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-os-config/.](https://github.com/auto-stack/auto-os-config/tree/master/)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
