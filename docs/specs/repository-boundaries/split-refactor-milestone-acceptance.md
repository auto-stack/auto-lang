# Split-refactor milestone acceptance (Windows M1)

> PLAN-989 r8 final verification (2026-11-12). This spec records the verified
> support surface of the split-and-refactor route at the frozen candidate N,
> the consumption boundaries of upstream deliveries, and the two readiness
> verdicts. Sources: PLAN-1003 (production C policy), auto-os/PLAN-068
> (Windows RQ, amended acceptance), PLAN-1006 (Web member delivery), and the
> PLAN-989 verification runs recorded in the archived plan §9.

## 1. Frozen candidate N (2026-11-12)

| Repo | SHA | Note |
|---|---|---|
| auto-lang | 075f65cc… (= master d0dc40031 + audit-baseline re-anchor; code delivery point 7296df7d from PLAN-1006) | contains 1006 neutral commit/closure primitives |
| auto-web | 8b1742a0… | 1006 delivery + SDK-installer closure repair (fix-989-sdk-staging) |
| auto-man | 54fe4070… | 1006 service-export + installer mapping repair + core lock refresh |
| auto-ui | 69104765… | = OS068 delivery tip (unchanged since) |
| auto-os (main) | 868c80d3… | 068 delivery 3ea7ead + receipts |
| auto-down | 58ea33f… | unchanged |
| Products (read-only pins) | auto-chat 5cdca0f9…, auto-database dee81e00…; auto-video 0922b44… (B-06 handoff boundary) |

Toolchain rustc/cargo 1.98.0; Windows x64; device boundary per OS068 (60Hz
applicability, no photon-endpoint claim).

## 2. Verified support surface

- **Independent boundaries (C-01/02/04)**: strict modularity audit PASS at N
  (baseline re-anchored; self-test 16/16); L1 language-sdk-consumer 5/5; L2
  workspace `metadata --locked` + `build --workspace --locked` PASS; fixed
  groups `auto-generation-core` / `web-delivery-controls` /
  `generated-delivery-controls` PASS at N; UI real consumer driver 43/43
  (windowed run, five-generation matrix, install-interrupt-retry, VM session
  protocol); Man CLI real (`--version`, `service-export --help`); Web member
  delivery per SD (auto-web/docs/specs/generated-service-member-delivery.md)
  with 6/6 library tests and 30/30 external driver receipt 3f72415b….
- **Production C → real RQ (C-03)**: N release trio rebuilt
  (4D58639F…/C7DCD822…/FC1CABD9…); generated workspace root manifests carry
  the strategy-C block; real Todo backend task through the N CLI with
  isolated `AUTO_TODO_DATA` and independent SQLite read-back (6/6).
- **Same-source space accounting**: PLAN-1003's bound working-set measurement
  (11.64–11.70 GiB → ~5.08 GiB under C) remains the only physical claim; no
  new extrapolation is made at N (no RQ-path structural change since).
- **RQ functional/performance surface**: OS068's amended matrix (B-01..08)
  is consumed as written; N differs from the 068 trio only by
  language-delivery-additive changes (recorded equivalence), so 068's
  identity-bound runtime evidence (35/35 demo matrix, release measurements,
  N8 adopt, fault isolation) is reused; the RC-RQ drill adds the
  package-group/data/bystander/recovery plane at N (10/10).
- **Limited recovery objects (C-05)**: RC-SDK-01 (14/14 + full-profile
  install), RC-UI-01 (old/new pair generation→verify→locked build→relocate→
  managed upgrade→fault preservation→real run), RC-WEB-01 (PLAN-1006 30/30
  reuse), RC-RQ-01 (10/10) — see the archived plan §9 for receipts.
- **SDK installers**: fresh Core/Full/Web SDK installs at N are verified end
  to end after the fix-989-sdk-staging repairs (PLAN-944 closure mapping,
  PLAN-007 example relocation, PLAN-1003 compile inputs, verify-guard
  precision) — a real pre-existing gap chain surfaced by this verification.

## 3. Boundaries and named non-claims

- Window-plane RQ drills reuse OS068 evidence (identical OS/UI sources);
  the new drills cover service/data planes. No new >60Hz, photon-endpoint,
  byte-level transfer, replay-p95, or native-input-latency claim is made.
- auto-video backend remains the B-06 named handoff (owner + closure
  condition; not delivered by this route).
- RC pairs are first-version controlled pairs; no historical-release
  cross-schema upgrade, dual-machine zero-downtime, Linux, or offline-build
  claim. UI pairs use the rust-backend delivery form (002-counter's own
  pac.at is vue-render; the canonical rust-render fixture project carries
  the pair — recorded adjustment).
- Known environment attributions: plan988 slow integration test flaps under
  20-way parallel group runs (isolated replay green — governance owner);
  crates-io mirror gaps worked around offline-first; jet/ark UI legs remain
  user-ruled environment gates.

## 4. Readiness verdicts (PLAN-989 T-06)

- `split_refactor_ready = true` — AC-01..06 verified at N per this spec and
  the archived receipts; debts D-942-P1/P2 closed; no unrenamed in-scope
  required gap.
- `joint_m1_ready = true` — ACE M1 (PLAN-1000 archived, final 1006P/0F/0U)
  and AGE M1 (AG026 archived) receipts exist; the cross-repo consumable
  list is the N table above; the shared combination evidence is this
  route's N-verification plus the per-route receipts. Absent items would be
  named here; none remain.
- These are milestone readiness reports only. Formal `v0.6` additionally
  requires ACE M2, AGE M2, Linux distribution admission and App hardening
  per the version roadmap; no tag, deployment or release is created by
  PLAN-989 (template only, in the archived plan).

## 5. Re-verification entries

- Language: `cargo t auto-generation-core web-delivery-controls
  generated-delivery-controls`; `python scripts/modularity-audit.py --mode
  strict --output <scope>/boundaries.json` (+ `--self-test`); L1/L2 per
  docs/specs/repository-boundaries/standalone-language-build.md.
- UI delivery: `python test/fixtures/modularity/generated-ui-consumer/driver.py
  --scratch <scope>`; Web member delivery:
  `python scripts/generated-member-delivery.py …` (web repo).
- SDK installs: `man-install.py install|verify --profile core|full`,
  `web-install.py source|verify-source|install|verify` (group-pinned
  sources). RQ trio build commands per OS068 SD-03 §4; RC drills per the
  archived PLAN-989 §9 receipts (t05-* scopes).
