# 027 — Native CSS Pass-Through

Two levels of native CSS support in the Auto → Vue generator:

- **Widget-level** — a `style { ... }` block inside `widget` is captured
  verbatim by the lexer (never tokenized) and emitted into the component's
  `<style scoped>` block. Nested `{}`, `/* */` comments, media queries and
  pseudo-classes (`:hover`, `::before`) pass through unchanged.
- **Project-level** — `styles: [...]` in `pac.at` declares local `.css`
  files that are copied byte-for-byte into `gen/front/vue/src/styles/` and
  imported from `src/main.ts`.

## Build

```sh
auto build
```

## Verify

- `gen/front/vue/src/App.vue` ends with a `<style scoped>` block whose
  content matches the `style { ... }` block in `src/front/app.at` exactly.
- `gen/front/vue/src/styles/autodown-theme.css` is byte-for-byte identical
  to `src/front/autodown-theme.css` (a snippet of the real
  `@autodown/editor` stylesheet).
- `gen/front/vue/src/main.ts` contains
  `import './styles/autodown-theme.css'`.
