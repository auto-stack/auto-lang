// inject_styles.ts — Plan 442 A3 corpus: TS source referenced by the web
// adapter's nested use.web. No VM-target implementation exists; the loader
// synthesizes a no-op platform stub for `injectStyles`.
export function injectStyles() {
  const style = document.createElement('style')
  style.textContent = '/* web-only */'
  document.head.appendChild(style)
}
