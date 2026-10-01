# Website application introductions

> PLAN-722 / PLAN-723 · 2026-10-01 · VitePress / Vue 3

## Product structure

/apps (EN/ZH) distinguishes four main products—AutoEdit, AutoShell, AutoMusk, JadeEdit—from system apps and demos. AutoDown and AutoUI retain related resource pages; neither replaces one of the four product entries. AutoEdit and JadeEdit remain separate products with shared foundations.

AppsOverview.vue and data/applications.ts provide SSR cards, a two-column desktop grid, a single-column narrow layout, system categories, and ecosystem/resource links. No business API or runtime gallery is loaded by the introduction. Global navigation is a separate change.

## Reading topics and truthful progress

Four EN/ZH product topics use the default reading layout with a single H1, an outline, locale-correct internal links, and a material date. Explain purpose, a typical operation, current capabilities, runtime prerequisites, related projects, and work still in progress. Current claims come from the sibling source/spec snapshot recorded in docs/reports/p722-source-baseline.json.

Do not claim all-Auto, full runtime parity, unattended coding, completed Milestones, universal performance budgets, or collaboration from project direction alone. AutoMusk distinguishes skill contracts from runtime Relay, and provider/daemon communication from application tool execution. JadeEdit distinguishes single-document draft/reload protection from in-progress multi-document operations.

## Approved product screenshot contract

The initial writing phase was image-free. PLAN-723 introduces the approved AutoEdit PixPin_2026-10-01_15-14-27.png capture and AutoShell ash-01 native colored ls capture in the four-product overview and their EN/ZH details. Use EvidenceImage for keyboard enlargement and return focus. Never render empty frames, broken images, visible TODOs, or fake screenshots. Musk remains text-only until its sample-project walkthrough is ready; Jade remains text-only until its Milestone. Further operation images require their own real evidence.

AutoShell's existing evidence and interactive examples remain at /apps/autoshell/guide/ and the ZH mirror, using unchanged AutoShellLanding.vue. The introduction links the guide; the guide links back. Retain the native ls overview, mode records and their provenance, script tabs, keyboard navigation, copy, downloads, and version/platform boundaries. Release AutoShellPreview remains unchanged.

## Verification

Build and verify SSR, routes, local links, outline/locale behavior, approved image hashes and loading, absence of unready-product images/iframes/backend requests, original guide interactions, and EN/ZH at five widths in both themes. Demo images follow [the capture catalog contract](demo-capture-catalog.md). Prepared Spec changes belong to the plan worktree until landing. Category A does not trigger Cargo tests or docs_gen.
