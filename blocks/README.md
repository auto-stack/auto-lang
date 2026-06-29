# AutoUI Blocks

AutoUI's **Skill-tier** UI unit (Design 17): a natural-language **spec** + a
**reference implementation** set + **gotchas**, that an agent assembles from
widgets. Each block is a *package*:

```
blocks/<kind>/<name>/
  spec.md              # TOML frontmatter + NL body
  reference/<v>.at     # one reference widget per variant
  gotchas.md           # anti-examples ({wrong, why, right})
```

Kinds: `form`, `data-display`, `feedback`, `layout`, `composite`.

## Browse

```bash
auto block list                       # catalog, grouped by kind
auto block show form/login            # full spec + variants + gotchas
```

Or visually: `examples/blocks-gallery` (renders spec + reference source +
gotchas).

## Consume (two paths)

**1. Copy a reference** — when you want the standard form with light edits:

```bash
auto block add form/login --reference minimal --out src/front/blocks
# -> src/front/blocks/login.at  (owned; edit freely)
```

Reports palette deps, `dataSource` slots to wire, and gotcha titles.

**2. Agent generation** — when you need adaptation beyond a variant (custom
fields, SSO+2FA, brand). The agent reads the spec via `auto block show`,
writes a `.at`, and loops on `auto block check` + `auto build`. See
[agent-generation-workflow.md](../docs/design/blocks/agent-generation-workflow.md).

After landing, the file is **yours** — edit it, or if a customization exceeds
the block's extension-point vocabulary, eject and own it fully.

## Author a new block

Follow the [package format](../docs/design/blocks/block-package-format.md):
create `blocks/<kind>/<name>/{spec.md, reference/<variant>.at, gotchas.md}`.
The `BlockRegistry` scans it on the next `auto block list`; the
palette-drift guard checks every `palette` entry exists in the AURA
`WidgetRegistry`.

## Relation to the other tiers

- **widgets** (Plan 331/336/337): the palette blocks compose.
- **apps** (Design 16): `app = shell + route→block selection + block data wiring`.

See [Design 17](../docs/design/17-blocks-first-class.md) and
[Plan 342](../docs/plans/342-block-tier-phase-a-package-foundation.md) /
[Plan 343](../docs/plans/343-block-tier-phase-b-generator-and-cli.md).
