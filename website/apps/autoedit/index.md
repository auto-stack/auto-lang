---
title: AutoEdit — A code and text workspace
description: Project browsing, tabbed editing, search, and comparisons in AutoEdit, with its current runtime boundaries.
outline: [2, 3]
editLink: false
---

<script setup>
import ScreenshotSlot from '../../.vitepress/theme/components/ScreenshotSlot.vue'
import EvidenceImage from '../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# AutoEdit: a code and text workspace

AutoEdit is the Auto ecosystem's lightweight text editor. Browsing, comparing, and reviewing code are priorities alongside everyday editing and saving. Native desktop use guides its current development. Auto sources organize application logic and interfaces, with editing and rendering foundations supplied by AutoLang and AutoUI.

[All applications](/apps) · [System apps and examples](/apps#system-apps)

<EvidenceImage src="/apps/autoedit/overview-dark.png" alt="autoedit native main interface" caption="AutoEdit dark native workspace · user-provided 2026-10-01 main capture" :width="1924" :height="1247" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

## Start with a file in a project

The project tree helps you browse directories and open files in separate tabs. The editor displays line numbers, syntax highlighting, and Chinese text. Menus, the toolbar, and keyboard shortcuts provide common operations; recent files help you return to previous work.

A typical sequence is to find a file, open related tabs, read and make focused edits, save, and compare the changes. The console provides operation feedback, while the status bar reports cursor position, encoding, and line endings.

## Editing, search, and file saving

- **Multiple tabs:** create, open, save, undo, redo, and confirm closing unsaved changes.
- **Find and replace:** case sensitivity, whole words, regular expressions, escapes, replace all, and search across files.
- **UTF-8 handling:** recognize BOM and line endings, with line-ending conversion. An invalid-UTF-8 fallback avoids silently transcoding and saving unreadable input.
- **Sessions and recent files:** remember open files and the active tab, loading files on demand after restart. Restoring open files and restoring unsaved text are separate capabilities.

Large-file handling reduces unnecessary highlighting and full-document operations. Practical limits and performance depend on the runtime and operation; development measurements are not presented here as universal performance guarantees.

<ScreenshotSlot capture-id="SHOT-02" />

## Compare files, directories, and editing buffers

Comparisons support reading and checking changes after editing.

| Object | How it can be used |
|---|---|
| Two files | Read side-by-side or unified differences and navigate between change blocks |
| Two directories | Inspect unchanged, modified, added, and removed entries, then compare individual files |
| Open buffers | Compare editing content that has not all been saved, separately from disk-file comparisons |

You can jump from a comparison to the corresponding file, edit, save, and compare again. This differs from editing directly inside a diff row, which still needs further development. Directory synchronization includes confirmation; check the destination and affected entries first.

<ScreenshotSlot capture-id="SHOT-03" />

## Its role in AI-assisted work

AutoEdit provides a place for people to read, review, and refine text. Longer-term integration includes agent-driven operations and embedding shared editing components in other applications. [AutoMusk](/apps/automusk/) organizes coding-agent tasks; those integration directions do not make AutoEdit a completed agent system.

AutoEdit and [JadeEdit](/apps/jadeedit/) remain separate products. AutoEdit emphasizes code, text, and change review; JadeEdit emphasizes documents and their knowledge relationships. They aim to share components while describing their product capabilities independently.

## Current progress and runtime

Editing, search, and file and directory comparisons have implementations and verification material. Native delivery, performance work, and experience details continue; the Milestone is not described as completed.

The source project lives in `specs/auto-edit` in the `auto-edit` repository. With the matching Auto toolchain and project dependencies prepared, the development entry for a native VM window is:

```sh
cd specs/auto-edit
auto run -r vm
```

A Vue generation path also exists. Desktop and web prerequisites and feature coverage need separate confirmation. Consult the relevant version's delivery notes for compiled releases, installation, and performance measurements.

This introduction reflects project specifications and source as of **2026-10-01**. Future captures will identify their runtime and operation.

[AutoUI desktop runtime](/ui-desktop) · [AutoOS virtual desktop](/autoos/)
