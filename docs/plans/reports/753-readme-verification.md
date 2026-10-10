# PLAN-753 — README and language overview verification

Date: 2026-10-10. Contract: PLAN-753 revision 1, approved by the user's
“OK，继续执行”. Worktree: `D:/autostack/.wt/lang-753/auto-lang`, `plan-753-dev`.
Implementation base: `f68e1690e` (approval bookkeeping); dependency `auto-down`:
`895f8d0` (read-only sibling). Full reviewed/delivery commit identities and
consolidation checkpoints are recorded in the active Plan's §9.

## Delivered content

| Artifact | Result |
|---|---|
| README.md / README.cn.md | 154 / 147 lines; ecosystem, four main apps, AutoDown foundation, v0.5 scope, runnable entry points, learning and sources |
| docs/language/overview.md / overview.cn.md | 306 / 297 lines; current execution paths, syntax, errors, Actor/async/generators, bridges, configuration/UI, tools and support boundaries |
| website/content/learning-navigation.mjs | One bilingual overview card; existing preparation produces the two real documentation routes |
| docs/specs/website/design/ecosystem-readme.md | SD-01, enduring landing-page/content/source/asset contract |
| docs/specs/website/project.md | SD-02, shared screenshots and generated bilingual overview navigation |

The old root-level language catalogue is replaced by the current overview and
links to the existing Tour, Script to Ship, interop cases, and implementation
Specs. No grammar definition, runtime, compiler, app or sibling-repository
implementation was changed. The older versioned specification retains its
identity rather than receiving a misleading v0.5 version label.

## Facts and bilingual audit

The four ecosystem responsibilities and four product roles were compared with
the current website application/OS/AI topic data and module Specs named in the
Plan's source table. Both locales cover the same sections and product set;
Chinese paragraphs, headings, alt text, captions and learning labels are localized.

The v0.5 summary uses `docs/releases/v0.5.md`'s audited history through 2026-09-30.
It distinguishes a rolling milestone from a frozen release candidate, the hosted
desktop from independent-system roadmap work, main Vue/iced paths from mobile
demo paths, and Rust-assisted self-hosting from the reference toolchain.
No app count is presented as release coverage, and no universal library/backend
compatibility or unattended Agent workflow is claimed.

Public website checks during research found `autolang.dev` to be a placeholder
and the documented `/v05/` deployment to return homepage content. Consequently
the README uses the repository's actual authored website pages. This is an
explicit source link, not a claim that those deployed routes work.

## Links and rendering

Markdown links/images were extracted with MarkdownIt, including the HTML logo.
Source paths were resolved from each document, fragments decoded and matched
against heading slugs, and same-repository GitHub source URLs mapped to the
reviewed checkout. Results: **152 local references, 52 repository-source
references, zero missing paths/anchors**. The new source URLs become available
on GitHub when these changes are published; local existence is not remote
publication evidence.

All nine distinct external repository links returned HTTP 200 and the matching
GitHub repository title: auto-ai, auto-down, auto-edit, auto-lang, auto-musk,
auto-os, auto-shell, books and jade-edit. No homepage fallback counted as a pass.

GFM previews used a plain MarkdownIt renderer with tables and GitHub-style layout,
served locally and inspected through headless Chromium. Four documents × 390px
and 1280px = **eight views**, all without document-level horizontal overflow.
README logo and all three shared captures loaded with localized alt text in
each locale/width; wide tables scroll within their containers on narrow screens.
This checks a GFM approximation, not GitHub's live hosted renderer.

| Document | 390px | 1280px |
|---|---|---|
| English README | [capture](assets/753/readme-en-390.jpg) | [capture](assets/753/readme-en-1280.jpg) |
| Chinese README | [capture](assets/753/readme-zh-390.jpg) | [capture](assets/753/readme-zh-1280.jpg) |
| English language overview | [capture](assets/753/language-en-390.jpg) | [capture](assets/753/language-en-1280.jpg) |
| Chinese language overview | [capture](assets/753/language-zh-390.jpg) | [capture](assets/753/language-zh-1280.jpg) |

## Shared screenshot identity

All README product captures directly reference the following existing originals.
None of these source files changed. The report's JPGs are page-verification
evidence, not another app screenshot collection.

| Original | SHA256 |
|---|---|
| website/public/desktop-showcase/02-desktop-dark.png | 77ef82ea50beaa99a29dc1ca9492ea11cdd47535b0866c10b537f4bdca3dc042 |
| website/public/apps/autoedit/overview-dark.png | fc52cc6cee4818237fae74652f3391c64054061748b818ff68bcfbbc4528fd26 |
| website/public/apps/autoshell/ash-01.png | 5c5512cc8eaaadfa810977f1b732b3db89b236952310a921c1bf2915b36f7d0c |

Desktop and AutoEdit captions identify 2026-10-01 captures; ash identifies the
September capture set used by the website. Musk/Jade remain text introductions
under their existing material-readiness contract.

## Executable examples and commands

Existing executable: `D:/autostack/auto-lang/target/debug/auto.exe`, SHA256
`24239007b3ae711b942f208a25d1addaf109ac31d4d7d45dddfd7de7fcd05283`.
This identifies the tested binary; it is not a claim of rebuilding it from the
review commit. Each standalone `auto` fence was extracted into a temporary
`.at` in the worktree. Deduplication across the four pages leaves eight programs.
Each was run with a 20s deadline and assertions on exit status, stderr and exact
program output after the CLI banner. All eight passed, both locales contain the
same executable snippets.

| Example | Asserted program output |
|---|---|
| Hello | `Hello, World!` |
| Bindings/interpolation | `Hello, Auto!` then `1` |
| Collections/object | `6` then `Auto` |
| Functions/closure | `7` then `15` |
| Pattern match | `zero` then `other` |
| Type/generic | `3` then `42` |
| Error propagation | `propagated error` |
| Lazy generator | `6` |

The error example matches the existing
`crates/auto-lang/test/vm/16_option_result/020_result_propagate` fixture and its
expected output. The generator links to `22_generator/001_sum/sum.at`.
The bilingual `auto,ignore` UI excerpts exactly match the current counter
project after removing comments/whitespace. They require project/UI context and
are not advertised as standalone Playground programs.

CLI commands were compared with `auto --help`, `auto trans --help`,
`auto run --help` and `crates/auto/src/main.rs`. Executed:

```sh
auto docs/tour/ch01-hello/01_hello.at
auto trans --path docs/tour/ch01-hello/01_hello.at rust
```

The emitted Rust compiled with `rustc --crate-name readme_hello` and the resulting
program printed `Hello, World!`. Source-build prerequisites were checked against
the manifests: sibling auto-down resolution and default Python/AutoDown/iced
features are disclosed. No UI runtime regression is claimed from this doc task.

## Website and format checks

In the worktree, dependencies were installed with npm without a lockfile change
or linking sibling folders. `AUTO_BOOK_ROOT=D:/autostack/book` resolves the actual
book source. Final `npm --prefix website run build` completed in **133.04s**.
No hand-written generated site page is included in the diff.

The built site was served on loopback and actual DOM/body checked:

| Route | Check |
|---|---|
| /docs/language/overview | English H1, English introductory section, error example, no page overflow |
| /zh/docs/language/overview | Chinese H1 and introductory section, error example, no English fallback |
| /docs/ | English overview card and correct local overview href |
| /zh/docs/ | Chinese overview card and correct local overview href |

All four returned 200 with their expected localized content. Existing build
messages say the `auto` highlighter falls back to text; snippets still render.
`git diff --check` passed. `python scripts/spec-index.py` completed (26 projects),
and the derived index had no content change. Category A applies: no `cargo t`,
Rust suite or `docs_gen` was run.

## Frozen Spec delta

SD-01 is the added contract, SD-02 is its module-overview reference; neither
introduces a new language grammar or production deployment requirement.

| Reviewed canonical target | SHA256 |
|---|---|
| docs/specs/website/design/ecosystem-readme.md | 895310da8b5aeb39289d4641811b42e8ad017aa6bc30c89e048f61e79ea6cfda |
| docs/specs/website/project.md | 2c34c8e9418c53fa689659ff5806e9073b791b101a39d4f30349237ec5cfacb5 |

## Consolidation prerequisite

P753-D1: the existing runtime `.autoos/specs.json` cannot be loaded by the current
AutoMusk Specs store. A task-owned loopback service using the existing `musk.exe`
returned HTTP 500 for `GET /api/specs` at 2026-10-10T07:55:35Z. It was stopped
after the read-only probe. The store's current required document/item schema is
incompatible with this existing ledger. Original SHA256 before/after the probe:
`3a4dc390529f95045f6bc93013b421645fc1f914cf3642d024dbfab23f80a968`.

No ledger data was rewritten or removed. README delivery is independently
verifiable; ledger publication, archival and worktree cleanup remain pending
until a supported store-mediated recovery succeeds. The active Plan holds the
review and merge checkpoints; this report does not claim those pending steps passed.
