---
title: "Book reader"
description: "A library leads into books, chapters, and reading settings, introducing navigation and reading progress."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Book reader

A library leads into books, chapters, and reading settings, introducing navigation and reading progress.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/book-reader.png" alt="Book reader actual running interface" caption="Native VM window connected to the actual reading backend · three sample books." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Select a book from the library.
2. Open a chapter, read its text, and move between chapters.
3. Inspect progress and settings, then return to the library.

## Runtime requirements and current scope

Requires book and chapter APIs; the local example backend supplies built-in books and text.

The capture shows the built-in library. It does not establish an online bookstore or general ebook import.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/018-book-reader](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/018-book-reader)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
