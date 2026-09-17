# AutoUI Blueprints

AutoUI's **Skill-tier** UI unit (Design 17): a natural-language **spec** + a
**reference implementation** set + **gotchas**, that an agent assembles from
widgets. Each blueprint is a *package*:

```
blueprints/<kind>/<name>/
  spec.md              # TOML frontmatter + NL body
  reference/<v>.at     # one reference widget per variant
  gotchas.md           # anti-examples ({wrong, why, right})
```

Kinds: `form`, `data-display`, `feedback`, `layout`, `composite`.

## Browse

```bash
auto blueprint list                       # catalog, grouped by kind
auto bp show form/login            # full spec + variants + gotchas
```

Or visually: `examples/blueprints-gallery` (renders spec + reference source +
gotchas).

## Consume (two paths)

**1. Copy a reference** — when you want the standard form with light edits:

```bash
auto bp add form/login --reference minimal --out src/front/blueprints
# -> src/front/blueprints/login.at  (owned; edit freely)
```

Reports palette deps, `dataSource` slots to wire, and gotcha titles.

**2. Agent generation** — when you need adaptation beyond a variant (custom
fields, SSO+2FA, brand). The agent reads the spec via `auto bp show`,
writes a `.at`, and loops on `auto bp check` + `auto build`. See
[agent-generation-workflow.md](../docs/design/blueprints/agent-generation-workflow.md).

After landing, the file is **yours** — edit it, or if a customization exceeds
the blueprint's extension-point vocabulary, eject and own it fully.

## Author a new blueprint

Follow the [package format](../docs/design/blueprints/blueprint-package-format.md):
create `blueprints/<kind>/<name>/{spec.md, reference/<variant>.at, gotchas.md}`.
The `BlockRegistry` scans it on the next `auto bp list`; the
palette-drift guard checks every `palette` entry exists in the AURA
`WidgetRegistry`.

## Relation to the other tiers

- **widgets** (Plan 331/336/337): the palette blueprints compose.
- **apps** (Design 16): `app = shell + route→blueprint selection + blueprint data wiring`.

See [Design 17](../docs/design/blueprints/blueprints-first-class.md) and
[Plan 342](../docs/plans/archive/342-blueprint-tier-phase-a-package-foundation.md) /
[Plan 343](../docs/plans/archive/343-blueprint-tier-phase-b-generator-and-cli.md).
