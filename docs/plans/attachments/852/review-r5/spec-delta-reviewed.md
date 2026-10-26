# PLAN-852 r2 — final reviewed knowledge delta (R5)

This freezes the proposed deposit, not a publication to canonical Specs. No semantic contract change or plan_revision bump. It supersedes older Plan delta shorthand saying lossless process counts; actual observer is periodic CIM ancestry sampling.

## SD-01 add docs/specs/auto-test-case/ace-m1-conformance.md

Deposit the frozen draft beside this file, SHA256 `9f668f89f179ee63979d40b11eea0c106f124302d0a1ea5808248f6bc2f405b6`. Scope:58 primary sources,43 handwritten references,5 typed source entries and40 supplementary scenarios; denominator/capability gaps separate from base suite. Runtime baseline/source/native-loader boundaries retained. AC-01/02/05.

## SD-02 add docs/specs/auto-test-case/a2r-golden-certification.md

Deposit the frozen draft beside this file, SHA256 `9dbc7d704ff12e3e99a2eda1d46b305acc711b9122fa53e9d53eaadf23772af4`. Actual compiler identity is measured under containment in explicit certification; harness build default is a separate daily staleness axis. Exact typed receipts, independent actual/reference execution, frozen wrapper/flags/std-only profile and daily snapshot without implicit certification. Add navigation to `receipts`: direct diagnostic entry to the SAME Invoke-ReceiptValidation used by snapshot; supplied BuildIdentity is diagnostic input, not a replacement for the real report-bound snapshot. AC-03/04/06/07.

## SD-03 modify docs/specs/auto-test-case/architecture.md

Append “ace-m1 认证拓扑”:

`ace-certify` (`auto-lang-core::test_cases::ace_certify`) is an explicit independent entry, outside suite dispatch. It freshly emits actual Rust, checks frozen golden equality, then independently compiles/runs actual and handwritten reference against a fixed oracle. Receipts bind source/golden/actual/wrapper/reference/oracle, selected compiler identity, frozen flags/dependency profile and observed outputs. Day-to-day snapshot emits/compares bytes and strictly validates receipts, using `build-rustc-identity` from its real run as the default-profile comparison axis. The helper `receipts` scope calls the same validator with explicitly supplied diagnostic identity and does not establish suite execution or complete M1 acceptance. No automatic recertification. Process watch is periodic CIM ancestry sampling with worker-maintenance observations separately classified; zero samples are not a lossless process-creation count. Current no-certification behavior is also checked through the actual dispatch/code boundary.

Before: no ace-m1 certificate topology. After: distinct explicit certification and daily validity paths, no second language runner. AC-03/04/07.

## SD-04 modify docs/specs/auto-test-case/project.md

Add navigation for `crates/auto-lang/test/cases/ace-m1/` (58 primary sources,43 references,5 typed entries,binding.tsv denominator,base suite,versioned receipt registry), six main verifier scopes plus direct `receipts` diagnostic scope, and explicit `ace-certify`. Base-layer PASS never means full M1 completeness. AC-05/08.

## SD-05 modify docs/specs/auto-test-case/profile-v1.md and profile-v2-native.md

Append navigation only: external receipt/wrapper/binding identity contracts are versioned separately as ace.m1.cert/1 and ace.m1.binding/1. Case/suite decoder acceptance/rejection sets and unknown-field rejection are unchanged. AC-04/06.

Existing target hashes: canonical-input-hashes.txt. New frozen draft/decision hashes: spec-hashes.txt. Reconcile changed target revisions at merge; do not overwrite concurrent canonical knowledge blindly. supersedes/new paths remain exactly the Plan's frontmatter paths. touched_goals=[] is justified by testing infrastructure scope and does not close downstream language/native/HMR goals. Canonical writes, ledger/index, archiving and cleanup remain merge-owned after gates.
