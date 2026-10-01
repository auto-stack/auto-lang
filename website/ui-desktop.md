---
title: AutoUI on the desktop
description: Browser, native VM, and compiled desktop paths in AutoUI.
---

<script setup>
import IntroductionFrame from './.vitepress/theme/components/IntroductionFrame.vue'
</script>

<IntroductionFrame kind="uiDesktop" />

## Browser and native preview

AutoUI has working desktop paths. `auto run` uses the Vue/browser path; `auto run -r vm` executes Auto UI sources in the VM with an iced native interface. These paths share UI definitions, while renderers provide their own platform facilities.

Native development supports hot reload. The path and the kind of change determine which running state can be preserved. Shared definitions aim for consistent layout, interaction, and themes; they do not promise identical text rasterization across platforms.

## Compiled applications and composition

Code generation and a2r provide compiled paths. Desktop shell, application rendering sources, and the composition host are connected through window and rendering contracts. RenderQueue and RQHost work belong to this boundary; they do not imply that every existing VM application has migrated to native execution.

`auto-os` assembles the desktop and applications, while `auto-lang` implements the language and framework. The current launch manifest primarily declares VM applications. Host integration and platform coverage must be checked for each path and feature.

## From application UI to AutoOS

A virtual desktop adds shared window management, workspaces, a launcher, and notifications around application UI. It reuses the existing host system. Early Linux composition-host work exists, with independent systems remaining a longer-term direction.

[View the actual desktop](/autoos/) · [Read the OS overview](/os) · [Desktop architecture](/docs/design/autoui/virtual-desktop) · [Compiled shell design](/docs/design/autoui/desktop-shell-a2r) · [AutoUI overview](/ui)
