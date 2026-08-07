# 037 — Multi-Store Single Build (Plan 012 Batch B)

Store-side compile correctness. Three Batch B items in one workspace:

| Gap | Was | Now |
| --- | --- | --- |
| 9a | incremental build drained the `STORE_EXTRA_FILES` thread-local, which `generate_component_from_file` clears per call — with 2+ store files changed in one build, only the LAST store's composable was written | store composables are collected explicitly per compiled file; one build emits all of them |
| 9b | incremental path (`auto build`) swallowed parse errors (`if let Ok(...)`), so a broken `.at` failed silently and left a stale composable/SFC behind | parse failures print `Warning: Failed to compile <file>: <err>` (same line as the fresh path) and continue; `auto build --strict` escalates to a hard build failure |
| 2 | store codegen injected a 015-notes-specific `get all_tags()` (referencing `notes.value`) into any store with a `notes` state var | hack removed — a store gets `all_tags` only when it declares one in `computed {}` |

## Layout

- `src/front/alpha_store.at` — store with a `notes` state var and NO
  `all_tags` declaration. Its composable must contain no `all_tags`
  (gap 2).
- `src/front/beta_store.at` — second store; both composables must appear
  after a single build (gap 9a). Declares a `double_count` computed.
- `src/front/panel.at` — sub-widget consuming BetaStore.
- `src/front/app.at` — consumes AlphaStore, renders BetaPanel.

## Build

```sh
auto build            # one build writes BOTH stores/useAlphaStoreStore.ts
                      # and stores/useBetaStoreStore.ts
auto build --strict   # same, but any warning/parse failure fails the build
```

## Parse-failure demo (gap 9b)

Break a store file, e.g. truncate `src/front/beta_store.at`:

```sh
printf 'store BetaStore {\n    model {\n        var count int = \n' > src/front/beta_store.at

auto build          # prints "Warning: Failed to compile .../beta_store.at: ..."
                    # and CONTINUES (exit 0) — same semantics as the fresh path
auto build --strict # FAILS the build (non-zero exit)
```

Restore the file afterwards (`git checkout -- src/front/beta_store.at`).

## Verify

- `gen/front/vue/src/stores/useAlphaStoreStore.ts` — exists, contains
  `export function useAlphaStoreStore()`, and contains NO `all_tags`.
- `gen/front/vue/src/stores/useBetaStoreStore.ts` — exists, contains
  `get double_count()`.
- `gen/front/vue/src/App.vue` imports `useAlphaStoreStore`;
  `gen/front/vue/src/components/BetaPanel.vue` imports `useBetaStoreStore`.
- Touch both store files and rebuild: both composables are re-emitted in
  the same run (pre-Batch B only the last one survived).

## Note on the `all_tags => []` placeholder

Older stores (015-notes, jade) carry a `computed { all_tags => [] }`
declaration that used to suppress the bogus injection. After Batch B the
placeholder is unnecessary but still compiles — it now simply emits the
declared getter once. It can be deleted at leisure.
