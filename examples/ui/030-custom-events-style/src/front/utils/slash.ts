// Simulates the hand-written editor extension that dispatches SlashMenu
// lifecycle CustomEvents on document. Consumed by the widget via
// `use { fn: ... }` and copied into the generated Vue project as
// src/ext/src/front/utils/slash.ts.

export interface SlashOpenDetail {
  query: string
  top: number
  left: number
}

export function fireSlashOpen(query: string, top: number, left: number): void {
  document.dispatchEvent(
    new CustomEvent<SlashOpenDetail>('autodown:slash-open', {
      detail: { query, top, left },
    }),
  )
}

export function fireSlashClose(): void {
  document.dispatchEvent(new CustomEvent('autodown:slash-close'))
}

// Element-level custom event: dispatched directly on the target element
// (component-emit equivalent for native elements).
export function firePoke(el: HTMLElement | null, note: string): void {
  el?.dispatchEvent(new CustomEvent('demo:poke', { detail: { note } }))
}
