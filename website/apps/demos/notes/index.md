---
title: "Notes"
description: "Short notes combine a list and a text editor, with folders, tags, and pinning for organization."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Notes

Short notes combine a list and a text editor, with folders, tags, and pinning for organization.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/notes.png" alt="Notes actual running interface" caption="Native VM window · built-in notes and editing area." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Choose a note from the left-hand list.
2. Edit its title and body or create another note.
3. Organize notes using folders, tags, and pinning.

## Runtime requirements and current scope

Requires the notes backend or local VM backend logic; persistence depends on the runtime.

This is a short-note application. Page relationships and knowledge-base work are described separately for JadeEdit.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/015-notes](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/015-notes)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
