---
title: "Database interface example"
description: "A database tree, records, editing controls, and query panel introduce the structure of a data workspace."
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# Database interface example

A database tree, records, editing controls, and query panel introduce the structure of a data workspace.

[All applications](/apps) · [28 system apps and examples](/apps/demos/)

## The interface

<EvidenceImage src="/apps/demos/database.png" alt="Database interface example actual running interface" caption="Native VM window · built-in table structure and sample records." :width="1920" :height="1200" :framed="false" loading="eager" zoom-label="Enlarge image" close-label="Close" original-label="View original" />

Capture runtime：**VM** · 2026-10-01

## A typical workflow

1. Choose a table from the database tree.
2. Inspect columns, records, and editing controls.
3. Switch to the query panel to inspect input and result layout.

## Runtime requirements and current scope

The application can display sample data; real connections, execution, and persistence need their corresponding backend.

The capture uses built-in Northwind-style data. SQLite labels and query/transaction controls do not prove a real database connection; transaction behavior includes simulation.

This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.

## Source and related introductions

Source project：[auto-lang/examples/ui/026-database](https://github.com/auto-stack/auto-lang/tree/master/examples/ui/026-database)

[AutoOS virtual desktop](/autoos/) · [AutoUI](/ui) · [Back to the application catalog](/apps/demos/)

Descriptions as of：**2026-10-01**。New and retained images have their capture dates and runtimes recorded separately.
