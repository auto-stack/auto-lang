# PLAN-825 proposed Spec deltas (T-06)

Status: proposed for independent review and merge-owned deposit; no canonical
Spec or ledger has been edited by work.

- SD-01 -> `docs/specs/auto-av/loaded-program-runtime.md`: see same-directory
  `loaded-program-runtime.md` for LoadedProgram admission, execution, identity,
  profile, accounting, observability, and recipe rules.
- SD-02 -> `docs/specs/auto-av/cache-and-conformance.md`: restored portable
  artifacts re-enter existing envelope/decode/verifier gates before indexes
  are rebuilt. Cache hits never restore serialized execution indexes or PASS.
- SD-03 -> `docs/specs/auto-av/project.md`: the scalar optimized and reference
  entries coexist; data/owned/collections remain checked reference paths and
  no JIT/native/source expansion is claimed.
- SD-04 -> `docs/specs/auto-test-case/architecture.md`: append-only
  `ac-av/loaded-run-v1` registration, `av-abc-825` corpus, parent-side runtime
  counter judgement, and missing/fake/duplicate counter-controls.
- SD-05 -> `docs/specs/auto-av/runtime-benchmark-method.md`: see same-directory
  `runtime-benchmark-method.md` for required engine/tier identities, workloads,
  phases, raw samples, distributions, memory, and failure policy.
- SD-06 -> `docs/specs/auto-av/resident-function-install.md`: unchanged code
  Arcs retain resident index identity and generation; precise code dependency
  changes rebuild only affected tables while old snapshots remain valid.
- SD-07 -> `docs/specs/auto-test-case/ace-m1-conformance.md`: append the 825
  binding from `852-validation/binding.md`, preserving source/native/M1 gaps
  and prohibiting typed-fixture substitution for source four-path claims.
