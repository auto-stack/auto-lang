---
title: AutoJade — Documents and local knowledge
description: AutoDown document editing, page links, search, organization, draft recovery, and current development boundaries.
outline: [2, 3]
editLink: false
---

<script setup>
import ScreenshotSlot from '../../.vitepress/theme/components/ScreenshotSlot.vue'
import ScreenshotGallery from '../../.vitepress/theme/components/ScreenshotGallery.vue'
import { autoJadeKnowledgeShots } from '../../.vitepress/theme/data/autojade-shots'
const knowledgeShots = autoJadeKnowledgeShots(false)
</script>

# AutoJade: documents and local knowledge

AutoJade is an editor for AutoDown (`.ad`) documents and local knowledge bases. Starting with writing a document, it connects page links, search, tags, and directory organization to support everyday notes and personal knowledge.

It is a separate application alongside AutoEdit. AutoEdit focuses on code and text review; AutoJade focuses on writing and knowledge relationships. AutoDown provides document and editor foundations, while the older Jade Garden supplies existing features and design experience. These names describe different scopes.

[All applications](/apps) · [AutoDown and Jade Garden resources](/apps/autodown/)

<ScreenshotSlot capture-id="SHOT-07" loading="eager" />

The product and source project are now named **AutoJade / auto-jade**. The source repository still uses its existing name while the repository rename is pending. These dark screenshots use the first-party product manual sample workspace and were provided on 2026-10-10 for release preparation.

## Start with a document

Find a page in the workspace tree, open, edit, and save it, then move between multiple tabs. Recent pages and quick open help you return to ongoing work. Documents can contain Markdown writing structures and YAML metadata.

Editing experience and reliable document operations are near-term priorities. A typical knowledge workflow is to write content, link related pages, inspect references, and find information again through tags and search.

## Connect pages as well as files

| Capability | Purpose |
|---|---|
| Page links and backlinks | Read related pages and find documents referencing the current page |
| Missing links and page creation | Discover missing destinations and create the corresponding pages |
| Quick open and full-text search | Find pages through names, aliases, and content |
| Tags and page properties | Organize titles, tags, and aliases, then browse by tag |
| Daily notes | Keep everyday records in a browsable knowledge base |

Links and display names follow distinct rules. Moving a page, changing its title, and renaming a file are different operations. Renaming can update references; deletion leaves missing-link relationships to resolve. Previews, confirmation, and protection for these operations continue to develop.

<div data-capture-id="SHOT-08"><ScreenshotGallery :shots="knowledgeShots" group-label="AutoJade knowledge workflow" zoom-label="Enlarge image" close-label="Close" original-label="View original" /></div>

Quick open finds pages by title, alias, or path.

## Organize the workspace and protect unsaved content

Local workspace operations include creation, renaming, moving, the trash, and some directory-merge handling. Actions involving several pages or rewritten references need their actual impact checked.

Single-document reload protection, local draft checkpoints, and recovery copies after restart have implementations. Recovering a draft first opens a copy without overwriting the original file. A protected draft is still unsaved content. Protection across multi-document file operations is being implemented and consolidated; this introduction does not claim a complete data-safety guarantee.

<ScreenshotSlot capture-id="SHOT-09" />

## Web and desktop paths from one project

Shared Auto interface and application sources organize web and desktop forms. The generated Vue path needs the application backend; AutoVM/AutoUI renders the native path. AutoDown supplies the editing component, and some common components are shared with the AutoEdit family.

Shared sources provide a foundation for both paths, but do not automatically establish equal coverage in every version and environment. Consult the relevant runtime verification.

With a matching Auto toolchain and dependencies including `auto-lang`, `auto-edit`, and `auto-down`, development entry points from the repository root include:

```sh
# Native VM window
auto run -r vm --theme dark

# Generate and build the Vue interface with dependencies prepared
pnpm build
```

Web file and knowledge operations require the accompanying backend. Hosting a generated interface on a static website does not establish that reading, writing, search, and saving work there.

## Current progress and direction

Document editing, links, search, organization, and draft recovery have implementation material. The v0.1.0 release materials currently describe source-trial preparation; final joint-release checks depend on the matching AutoLang candidate. The screenshots here describe their actual scenes and do not replace release acceptance. Draft recovery retains a capture slot.

The longer-term direction is a fuller knowledge workspace. Graph views, block-level and multi-user collaboration, and deeper AI knowledge workflows require their own delivery descriptions. Neither the old feature pool nor long-term goals should be treated as a list of completed AutoJade capabilities.

Naming and release-preparation material were checked on **2026-10-10**.

[AutoEdit's code and text workspace](/apps/autoedit/) · [AutoOS history and outlook](/articles/autoos-history) · [Application overview](/apps)
