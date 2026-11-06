# Source CTE Integration (ac.source.cte.scalar/v1)

Status: active (PLAN-936; supersedes deck-only wiring of the 817 probe)
Producers: `ace-source-cte` (`plan936/source-cte`), the CT lanes of
`compile_scalar_source_product`
Consumers: AV driver/session, native driver, A2X (via `ProductCompile`);
the 817 probe delegates its closure-lane real-AV tail to the shared
service.

## 1. Surface

* The explicit capability `ac.source.cte.scalar/v1` admits the single
  expression demand `#{ expr }` on the `scalar-ct` grammar face (the
  scalar-instances full surface plus the hash atom; front-door key
  `ac-source-front/3`). Without the capability `#` is not lexed and the
  rejection face is unchanged.
* The meta query capability `cte.meta.fields/v1` (default CLOSED) admits
  the protected selectors `ct_field_count(Record)` → i32 and
  `ct_has_field(Record, "literal")` → bool. Selectors are DATA: a bare
  declared record name and a string literal; they are validated at bind,
  evaluated locally against the CURRENT nominal declarations, and become
  REAL AV arguments by structural substitution. Field existence is a
  query result (absent = false), never a bind error. Same-name user
  declarations under the capability are a named conflict; without it the
  names are ordinary.
* Unserved `#{ ... }` binds fully (inner in a CT context; the hash type
  is the inner type) and is rejected by the typecheck walk
  (`source.cte.unserved`) — a hash is never a runtime expression.

## 2. Service (acyclic extraction)

`ace-source-cte` depends on core/lowering/av-core/artifacts/
semantics-contract/source-query ONLY — never on the AV-frontend session,
native, or the comptime probe packages (its manifest asserts this). The
817 closure lane's real-AV tail (lower → MIR/ABC admission → execute)
delegates to the shared service (`CtExecutor`'s cold implementation);
the snapshot lane keeps its 828 frontend probe-side per the plan.

## 3. Execution

* A demand compiles on a CLOSURE-FILTERED program (the owner's
  transitive callee closure + type declarations; every served site
  replaced by its frozen literal and the current site by its substituted
  inner) under NO granted capabilities. The minimal entry is an
  AST-level wrapper (never text synthesis, never a re-parse).
* i32 demands return only I32 AV values; bool demands return Bool or the
  lane-versioned 0/1 projection wrapper's output (any other i32 is
  rejected). Expected class ≠ expression class = typed rejection.
* Counters are trust anchors: `ct_execute` counts only attempts that
  reached the AV (trap/resource/cancel included); hits never execute;
  meta queries are `meta_query`, never AV. The execute-boundary log
  correlates every attempt with its execution key.

## 4. Keys, freeze and reuse

* The execution key = callee logical + signature/body versions + the
  TRANSITIVE closure versions + canonical args (or the substituted
  expression's canonical hash) + engine/lane/profile. Whole-module
  source hashes, callsite spans and provenance revisions never key.
* Only successful executions mint envelopes (`ac936/ct-exec/v1`, layer
  S936, producer `plan936/source-cte`, no source-level fingerprint);
  the sink refuses pages its own codec would reject (write-path
  admission). Use-binding data (owner/site/expectation) never enters
  the cached page.
* Checked-child transitivity: an inner CT value (or a template body
  edit) changes the substituted instance body's structural version,
  which re-keys the outer demand — a source-hash-only cache mis-hits.

## 5. Root discipline

One root operation shares one cumulative budget (10s total; each AV
call min(2s, remaining); no demand resets it; exhaustion = resource
state, nothing published). Query cycles close over (owner, span) node
identity and are rejected with the named chain; plain recursion is not
a cycle. Explicit hashes are demands wherever they appear (nested in a
hash expression or in any CT-closure callee body) — an un-taken runtime
trap inside a single hash stays latent (the AV's control flow decides).

## 6. Actual A2X worker entry (PLAN-958)

The `a2x-worker` scalar `daily-emit`, `daily-emit-c`, `certify`, and
`certify-c` operations dispatch to `compile_scalar_ct` when the request
grants `ac.source.cte.scalar/v1`. Their common scalar helper owns one
cold `CtService<MemoryEnvelopeSink>` per operation, then passes that
product's Checked HIR, input hash and atom text to the original emitter.
Ordinary scalar requests use the source-query fact entry. An ungranted
`#` remains a front-door diagnostic; it cannot reach target compilation.

The actual CLI regression covers sum42/body-edited sum43, boollit's
program result 7, field-count 2/3 producing 42/43, choose42 and pick40.
Rust/C daily emission is compiler-spawn-free. Explicit certification
compiles actual and independent references separately; the regression
also executes both newly materialized executables through the owned
process facility and compares raw bytes with each hand oracle. This cold
worker contract does not establish warm CT reuse or public restore pages.
