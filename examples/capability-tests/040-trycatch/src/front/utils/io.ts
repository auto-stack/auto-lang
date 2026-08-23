// Hand-written TS extension for 040 — a fallible operation the DSL handler
// wraps in try/catch/finally. Synchronous throw keeps the example exact
// (no await plumbing); async api fns via `use back.api:` are awaited by the
// compiler and hit the same catch path (see README).
export function saveDoc(text: string): void {
  if (text.trim() === '') {
    throw new Error('empty document')
  }
  // pretend to persist
}
