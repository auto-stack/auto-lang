# PLAN-761 r2 独立复审与搬迁验证

## Baseline

- Review stage rebuilt conclusions from the committed diff, Git objects, current Specs and command evidence. Same implementation session; no separate reviewer agent/session was used, so this is artifact-based independent verification with that limitation.
- Reviewed commit: `62eb5435ecf3e2c5675cb9dd47557a419b621232`; base: `ef778ae31`.
- Prior implementation `28ddc73b1 → 27791bb71`, debt record `a48cd5b53 → 62eb5435e`; `git range-diff b25729f5a..a48cd5b53 master..plan-761-dev` showed both patches `=` after rebase. Main's intervening change was the desktop golden/AGENTS note, outside this plan's runtime logic.
- Read-only sibling auto-down: `895f8d0f9355c9f5ec3ce8fca268bdb768395846`.
- Frozen canonical Spec delta SHA256: `55B3531451AC18FF7655A53B8FB4639840A7707812C9F50F43F18F28F2D1FC4A` (Git diff from the review base over docs/specs, UTF-8 lines joined with LF).

## Acceptance reconstruction

| ID | Result | Evidence |
|---|---|---|
| AC-01 | pass | Committed root tree exactly 30 entries, 18 directories; original 19 scattered/local-config roots and 4 consolidated directories gone. README/README.cn/LICENSE and Node entry files retained |
| AC-02 | pass for committed delivery; main restoration is merge checkpoint | Independent old/new Git-object IDs confirm 44 of 64 moved assets identical, including Plan600 PNG and DEBTS; 20 explained path/safety edits. docs/reports/root-layout-761.json retains per-file mapping/pre-edit SHA256. Main config backups and unrelated WIP were inventoried before edits |
| AC-03 | pass for migration | Tracked references scanned independently; Vue fixture points to root blueprints, proxy helper finds deployment project/local nginx, all five MCP --check commands resolve corpus, native-fixture remains standalone workspace, both group guards clean. Legacy runtime limits below remain separate |
| AC-04 | canonical delta pass; publication postconditions checked at merge | Current Spec references/overview/design/index reflect delivered paths; canonical content avoids Plan chapter copies. Ledger compatible projection, archive, local-settings restoration and cleanup receipts are post-review merge operations |

## Verification

- `cargo check -p auto-lang`: pass (1m06s), 406 existing warnings. New Rust changes are path literals/comments, not declarations or behavior. Existing extern/unused warning families remain documented; no new warning-producing construct.
- `cargo --config build.target-dir=... t --no-fail-fast`: 5243 run, 5242 passed, one known Plan484 flake, 1541 skipped, 107.061s. `plan484_024_charts_streaming_recompute` passed two isolated reruns (0.305/0.317s), and is already identified in the batch receipt.
- `cargo tv --no-fail-fast`: 162/162 pass (2.135s). No generator logic, Schema/grammar or aavm trigger source changed, so tu/taa/standalone docs_gen are not added gates.
- Initial CARGO_TARGET_DIR environment sharing polluted nested oracle build destinations and caused five binary-not-found failures; discarded as a verification-environment error. Final outer-only Cargo configuration removed that inheritance and all five passed.
- Debug Auto CLI actually executed `test/auto/field_access.at`: 7/7 pass (0.051s). Rust and Auto golden asset paths inside the four moved runners remain unchanged; only their own locator comments changed.
- Python/Node/Bash/PowerShell syntax: pass; five p446 probes' `--check`: pass without launching apps. Proxy `--help`, fstr dry-run and screenshot-diff smoke: pass. Tailwind regeneration: exactly 8861 classes/15 families, byte-equal to the adjusted manifest.
- Native fixture `cargo metadata` identifies `scripts/tools/native-fixture` as its workspace; cargo check passes (6.82s), one pre-existing unnecessary-unsafe warning. No UI process termination or live deployment was used as verification.
- `git diff --check`: pass. Root ignore correctly excludes temporary probes and local configs, and does not exclude `crates/auto-lang/src/ui/debug/inspector.rs`.

## Explicit legacy limits and debt scan

1. Plan446 J1 was explicitly executed against the new fixture, with no SKIP. Current bridge initialization rejects ambiguous `Init` across Theme/Modules. The original `b25729f5a` export of all 22 b3check files, replayed through the same unchanged runtime with the repository's 16MB test stack, produces exactly the same error. Historical source is preserved rather than silently repaired. Candidate P761-D2 is in KNOWN-DEBT.
2. An extra runner probe using an unmatched CLI filter still executed runner bodies: a2c/a2r output real golden results, a2ts emitted nested failures while the filtered summary reported 0/0, VM ran until the 30s probe timeout. These exits are **not passing suite evidence**. Source review shows `test_file` executes before the display filter; candidate P761-D1 is in KNOWN-DEBT. Root-directory migration does not change this existing API behavior.
3. No dropped root-cleanup task, newly discarded coverage, unapproved workaround or deferred migration was found. Conditional Node-manager consolidation was explicitly outside this approved implementation. Both legacy debt candidates are outside the cleanup contract; none substitutes for a missing migration acceptance item.
4. The repository's tracked ledger has 825 items in a legacy six-section schema. An isolated current Musk store could not load it (GET /api/specs HTTP500); no write request was sent and that owned process was stopped. User-provided AGENTS knowledge-deposit instructions plus the local specs README's compatibility/manual-fallback convention apply here; the merge projection must preserve every existing item/field and append only this delivery's references. No schema conversion or unrelated entry rewrite is authorized.

Review outcome: **pass for implementation and SD-01..04**. Merge performs and verifies the publication/restoration/archival postconditions; it must not treat this review as a claim they already happened.
