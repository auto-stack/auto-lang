# 029 — External TS/Vue Imports

Widget-level `use { ... }` block: import hand-written TypeScript, Vue
components, and composables into a generated SFC — the generic escape
hatch that the hardcoded `autodown_editor` registry entry is a special
case of.

## Syntax

```auto
widget App {
    use {
        // TS functions/constants, callable from computed + on blocks.
        // Local file (project-root-relative) → copied to src/ext/, imported
        // as `@/ext/...`. npm specifier → used as-is (pair with npm_deps).
        fn: greet, farewell from "src/front/utils/greet.ts"

        // External Vue components, instantiable in the view with generic
        // `:prop` bindings and `@event` listeners. `.vue` paths become
        // default imports; anything else becomes a named import.
        component: FancyBadge from "src/front/components/FancyBadge.vue"
        component: Smile from "lucide-vue-next"

        // Vue composables: imported, then called once at <script setup>
        // top level — `useClock` → `const clock = useClock()` — reachable
        // from `on` handlers as `clock.stamp()`.
        composable: useClock from "src/front/composables/useClock.ts"
    }
}
```

View tags accept the component name in PascalCase, snake_case, or
kebab-case (`FancyBadge` / `fancy_badge` / `fancy-badge`). Declared
components shadow the built-in widget registry; the registry stays the
fallback for undeclared tags (so `autodown_editor` keeps working).

## Path scheme

Paths starting with `.`/`/` or ending in `.ts`/`.tsx`/`.js`/`.mjs`/`.vue`
are project-local, interpreted relative to the pac.at directory
(like `styles:`), copied into `gen/front/vue/src/ext/<same relative
path>` (layout preserved so sibling relative imports keep resolving),
and imported through the `@` alias. Everything else is an npm specifier
— add it to pac.at `npm_deps:` if it isn't already a base dependency
(`lucide-vue-next` is).

## Scenarios

1. **fn** — `greet(name)` in a `computed`, `farewell("badge")` in an
   `on` handler.
2. **component** — `FancyBadge` (local `.vue`) with `:label` prop and
   `@selected` listener; `Smile` (npm `lucide-vue-next`) as an icon.
3. **composable** — `clock.stamp()` in the `Greet` handler.

## Build

```
auto build   # → gen/front/vue, runs vue-tsc + vite build
```
