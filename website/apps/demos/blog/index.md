---
title: "Blog reader"
description: "Browse articles by category and open their text and author information. The list and article views introduce a content application."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Blog reader

Browse articles by category and open their text and author information. The list and article views introduce a content application.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/blog.png" alt="Blog reader actual running interface" caption="Native VM window · built-in article list." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Choose a topic or category from the article list.
2. Open an article and read its author information and text.
3. Return to the list to explore another article.

## Runtime requirements and current scope

Requires the article backend or local VM backend logic.

Content comes from built-in articles rather than an external blog aggregation service; writing features depend on the version.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/021-blog-viewer](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/021-blog-viewer)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
