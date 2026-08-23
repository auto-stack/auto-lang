# 040 — try/catch/finally in handler bodies

Real error handling in `on` handlers — the DSL form of JS
`try/catch/finally` (gap 4). Plan 010 gave the language `try { } catch (e)
{ }` (parser + VM); this batch adds the optional `finally { }` clause and,
critically, the **Vue/TS emission** — previously a `try` inside a widget or
store handler was **silently dropped** from the generated code (the handler
emitted empty), which forced jade-garden stores to route every fallible call
through hand-written "safe wrapper" ext functions.

## Syntax

```auto
on {
    .Save -> {
        try {
            .error = ""
            saveDoc(.text)          // ext fn that may throw
        } catch (e) {               // binding optional: catch { ... } works too
            .error = "save failed"
        } finally {                 // optional; runs on BOTH paths
            .busy = false
        }
    }
}
```

emits (state refs are `.value`-rewritten inside all three clauses):

```ts
async function Save(args: any): Promise<void> {
  try {
    error.value = '';
    saveDoc(text.value);
  } catch (e) {
    error.value = 'save failed';
  } finally {
    busy.value = false;
  }
}
```

## Semantics & limits

- Bodies stay AURA-aware: `.x` state writes, api calls, etc. transpile
  exactly as outside the try. `use back.api:` functions are still awaited,
  so a rejection lands in `catch` — this closes jade's last behavioral
  deviation (save-failure rejection handling).
- `catch` is required (parser-enforced); `finally` is optional. Bare
  `try/finally` without `catch` is not yet supported.
- VM runtime: finally runs on the normal path and after the catch handler;
  an error thrown *inside* the catch body propagates without running
  finally (documented deviation from JS).
- Vue path only: kotlin/ark adapters do not emit Stmt::Try yet.
