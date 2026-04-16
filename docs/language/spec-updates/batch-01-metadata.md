# Spec Update: Batch 01 — Metadata & TOC Restructure

**Date**: 2026-04-16
**Plans Referenced**: N/A (organizational)
**Source Files**: N/A
**Sections Updated**: Header (version/date), Table of Contents (restructured)

## Old Content

```
**Version**: 0.1
**Status**: Draft
**Last Updated**: 2025
```

Old TOC had 15 sections covering only core language features.

## New Content

- Version bumped 0.1 → 0.2, date updated to 2026-04
- TOC expanded from 15 to 24 sections
- New sections added:
  - Section 12: Enums (extracted from "Unions and Tags")
  - Section 13: Specs (Traits) — NEW
  - Section 14: Generics — NEW
  - Section 15: Closures — NEW
  - Section 16: Option and Result — NEW
  - Section 17: Concurrency: Tasks and Async — NEW
  - Section 18: Compile-Time Metaprogramming — NEW
  - Section 19: Ownership and Borrowing — NEW
  - Section 20: Modules and Imports — NEW
  - Section 21: UI Widgets and Routing — NEW
- Section numbers for existing sections adjusted (Nodes → 22, Memory → 23, Implementation → 24)

## Notes

- The old "Unions and Tags" section is split into separate coverage: unions stay in Type Definitions, tags/enums get their own Enum section
- Section content will be filled in by subsequent batches
- Original Nodes, Memory, and Implementation sections kept but renumbered
