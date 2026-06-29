# Agent-driven Block Generation Workflow (Plan 343, Design 17)

> The `auto` binary **never calls an LLM**. AI generation of a block happens in
> an **agent** (a Claude Code session, `auto-musk`, etc.). `auto` only *supplies*
> the spec and *validates* the output. The block spec is a curated **skill
> file**; the agent is its executor.

## Why not generate inside `auto block add --from …`?

The `auto` compiler binary has no LLM/HTTP infrastructure and shouldn't grow
any — generation is an *agent-layer* responsibility, not a compiler one. A CLI
flag that shells out to an LLM would put API keys, provider clients, and retry
logic in the wrong binary. So Plan 343's original `--from` mode is replaced by
this agent workflow.

## The loop

```
auto block show <kind>/<name>          # 1. agent reads the spec (skill input)
        │  (spec frontmatter + NL body + variants + gotchas)
        ▼
agent writes <name>.at                 # 2. per consumer intent + project context
        │   (available widgets, existing #[api] signatures, theme tokens)
        ▼
auto block check <name>.at --spec K/N  # 3. static gate (loading/error/palette)
auto build                             #    + full compile gate
        │  fail? → feed errors/unmet items back to agent → rewrite (≤ N rounds)
        ▼
drop into src/front/blocks/<name>.at   # 4. owned, editable, ejectable
```

## Step detail

1. **Read the spec** — `auto block show form/login` prints the whole package:
   frontmatter (kind, palette, extension_points, variants, `dataSource`),
   the NL intent/assembly-guidance, and the gotchas. This *is* the skill.

2. **Generate** — the agent composes widgets from `palette`, implements the
   `extension_points` (each marked `// EDIT: <point>`), wires the declared
   `dataSource` slots, and honors the behavior contract (loading/error states).

3. **Validate** — two gates:
   - `auto block check <file> --spec <kind/name>`: fast, deterministic static
     checks (loading + error present, used widgets within palette; EDIT-marker
     coverage reported as info).
   - `auto build`: full compile. Failures + unmet `auto block check` items go
     back to the agent as repair instructions, up to N rounds (the Design 16
     metric).

4. **Land** — the result is copied into the consumer's
   `src/front/blocks/<name>.at`. It is **owned source**: the consumer may edit
   it freely; if customization exceeds the block's extension-point vocabulary,
   they simply edit / eject (Design 17 §2.2).

## When to copy vs. generate

- **`auto block add --reference <variant>`** — when the consumer wants the
  standard form with light edits. Fast, deterministic, no generation.
- **Agent generation** — when the consumer needs adaptation beyond a variant
  (custom fields, SSO + 2FA, brand) described in natural language.
