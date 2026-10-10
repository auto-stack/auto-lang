# Ecosystem README and language introduction

> PLAN-753 · 2026-10-10 · Markdown / website shared assets

## Reader-facing structure

README.md and README.cn.md are the bilingual landing pages for the Auto ecosystem.
Introduce AutoLang, AutoUI, AutoOS, AutoAI, and applications before language detail.
Keep reciprocal locale links and entry points for learning, sources, UI examples, and release notes.
AutoEdit, AutoShell, AutoMusk, and JadeEdit are the four main application introductions;
AutoDown is their related document/editor foundation. System demos have their own coverage.

The current language introduction lives in docs/language/overview.md and overview.cn.md.
The existing site preparation maps them to /docs/language/overview and
/zh/docs/language/overview. DOCS_HUB in website/content/learning-navigation.mjs links the overview.
Keep examples grounded in current CLI/source/fixtures. An older specification draft is not
automatically the current release specification. Overview updates do not redefine grammar.

## Facts and implementation boundaries

Use current module Specs, sources, verified fixtures, and the latest product topics to resolve
differences from older promotional copy. Distinguish the reference Rust toolchain, AutoVM scripts,
the main Rust shipping path, other generators, and experimental self-hosting.
Vue and iced are the main UI paths; mobile demo paths and independent-system directions remain
separate. The hosted desktop does not imply a delivered custom kernel. Agent tools execute in
the application layer; shared model services do not constitute unattended application workflows.
Do not turn a corpus count, app catalog, screenshot, or local parity test into a universal promise.

Current README source-build instructions identify sibling auto-down path resolution and the
default CLI's Python/AutoDown features. Check actual manifests before changing prerequisites.
Release milestone notes and frozen release artifacts are distinct.

## Shared images and links

README images directly reference existing website/public files; no copied image collection.
Use the desktop-showcase dark desktop, the approved AutoEdit workspace, and original ash-01
terminal capture with localized alt/captions, capture provenance, and full-size image links.
Musk/Jade follow the readiness rules in [application introductions](application-introductions.md).
Do not generate mock screenshots or rewrite UI text inside captures. Historical captures are
illustrations of observed versions, not frozen-candidate verification receipts.

Repository landing pages use reliable authored-source links while public deployment is pending.
Language overview links within published docs can remain relative. Source-file, repository,
unpublished Spec, and locale-suffixed author paths use full repository URLs so site preparation
cannot turn them into nonexistent deployed paths. Site navigation supplies local locale switching.
When adopting public URLs, verify the actual page body; an HTTP 200 home-page fallback is not success.

## Verification

Documentation-only changes use Category A: no Rust test suite or docs_gen.
Check local links/anchors and shared image hashes, render GFM at narrow/wide widths in both
languages, run introductory snippets and compare output, and build the website with both new
overview pages. UI project excerpts use auto,ignore fences because they need project/UI context;
only standalone executable examples receive the site's inline Run control.
