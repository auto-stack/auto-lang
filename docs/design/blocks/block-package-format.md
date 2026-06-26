# Block Package Format (Plan 342, Design 17)

A **block package** is the unit of the block tier. Each lives at
`blocks/<kind>/<name>/` and has three parts: a **spec**, one or more
**reference implementations**, and a **gotchas** file.

## Directory layout

```
blocks/<kind>/<name>/
  spec.md                  # TOML frontmatter + natural-language body
  reference/
    <variant>.at           # one reference Auto UI widget per variant
    <variant>.at
  gotchas.md               # anti-examples: { wrong, why, right }
```

## spec.md

TOML frontmatter delimited by `+++` … `+++`, then a markdown body.

```markdown
+++
kind = "form"
name = "login"
palette = ["Input", "Button", "Label", "Checkbox", "Separator"]
extension_points = ["fields", "submit", "third_party", "success", "error_display"]
variants = ["minimal", "with_sso"]

[dataSource]
attempt = "(creds) -> Session"
providers = "[]Provider"
+++

# Intent
A credential-capture form that authenticates against a session endpoint.

# What this block absorbs (per-app variation)
fields / SSO / 2FA / captcha / success-redirect-vs-inline / validation / ...

# Assembly guidance
Compose Label+Input per field; submit -> dataSource.attempt; show loading on
Button; surface errors via the error slot; mark EDIT regions in the output.

# References
- minimal — email + password only (reference/minimal.at)
- with_sso — adds social-provider buttons (reference/with_sso.at)

# Gotchas
See gotchas.md.
```

### Frontmatter schema

| field | type | meaning |
|---|---|---|
| `kind` | string | block category: `form` / `data-display` / `feedback` / `layout` / `composite` |
| `name` | string | block name (kebab) |
| `palette` | []string | widgets this block composes — each must exist in `WidgetRegistry` |
| `extension_points` | []string | the bounded vocabulary a consumer may vary (EDIT regions) |
| `variants` | []string | named presets; each must have a `reference/<variant>.at` |
| `dataSource` | table | typed fetcher signatures the block expects (consumer wires real `#[api]` fns) |

> TOML is used (not YAML) because it is already a workspace dependency and
> authoring quality is comparable for this size of document.

## reference/&lt;variant&gt;.at

A valid Auto UI `widget` source. It must:

- Be self-contained and compile (`auto build` green) — pass data/callbacks in as
  props or model rather than importing a backend, so it runs without one (mirrors
  `EditorPanel(note: str)` in `examples/ui/015-notes`).
- Materialize the block's extension points as `// EDIT: <point>` marked regions.
- Demonstrate the loading / error / empty states the spec declares as contract.

## gotchas.md

Each entry:

```markdown
### <short title>

**Wrong**
<description or ```auto code block of the anti-example>

**Why**
<why it's wrong>

**Right**
<the fix>
```

Gotchas may start sparse and grow from real AI failure modes (each time the
generator errs on this block, a gotcha is added).
