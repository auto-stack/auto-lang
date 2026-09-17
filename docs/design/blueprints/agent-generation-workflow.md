# Agent-driven Blueprint Generation Workflow (Plan 343, Design 17)

> The `auto` binary **never calls an LLM**. AI generation of a blueprint happens in
> an **agent** (a Claude Code session, `auto-musk`, etc.). `auto` only *supplies*
> the spec and *validates* the output. The blueprint spec is a curated **skill
> file**; the agent is its executor.

## Why not generate inside `auto bp add --from …`?

The `auto` compiler binary has no LLM/HTTP infrastructure and shouldn't grow
any — generation is an *agent-layer* responsibility, not a compiler one. A CLI
flag that shells out to an LLM would put API keys, provider clients, and retry
logic in the wrong binary. So Plan 343's original `--from` mode is replaced by
this agent workflow.

## The loop

```
auto blueprint show <kind>/<name>          # 1. agent reads the spec (skill input)
        │  (spec frontmatter + NL body + variants + gotchas)
        ▼
agent writes <name>.at                 # 2. per consumer intent + project context
        │   (available widgets, existing #[api] signatures, theme tokens)
        ▼
auto blueprint check <name>.at --spec K/N  # 3. static gate (loading/error/palette)
auto build                             #    + full compile gate
        │  fail? → feed errors/unmet items back to agent → rewrite (≤ N rounds)
        ▼
drop into src/front/bps/<name>.at   # 4. owned, editable, ejectable
```

## Step detail

1. **Read the spec** — `auto bp show form/login` prints the whole package:
   frontmatter (kind, palette, extension_points, variants, `dataSource`),
   the NL intent/assembly-guidance, and the gotchas. This *is* the skill.

2. **Generate** — the agent composes widgets from `palette`, implements the
   `extension_points` (each marked `// EDIT: <point>`), wires the declared
   `dataSource` slots, and honors the behavior contract (loading/error states).

3. **Validate** — two gates:
   - `auto bp check <file> --spec <kind/name>`: fast, deterministic static
     checks (loading + error present, used widgets within palette; EDIT-marker
     coverage reported as info).
   - `auto build`: full compile. Failures + unmet `auto bp check` items go
     back to the agent as repair instructions, up to N rounds (the Design 16
     metric).

4. **Land** — the result is copied into the consumer's
   `src/front/bps/<name>.at`. It is **owned source**: the consumer may edit
   it freely; if customization exceeds the blueprint's extension-point vocabulary,
   they simply edit / eject (Design 17 §2.2).

## When to copy vs. generate

- **`auto bp add --reference <variant>`** — when the consumer wants the
  standard form with light edits. Fast, deterministic, no generation.
- **Agent generation** — when the consumer needs adaptation beyond a variant
  (custom fields, SSO + 2FA, brand) described in natural language.

## Channel discipline (PLAN-639, three-tier consumption)

Generation (this document) is **L3** — the fallback tier, not the default.
Prefer, in order:

1. **L1 import/bind** — app declares the bp package in `pac.at` (`dep`), imports
   the reference implementation via `use`, and binds declaratively
   (`src/front/bps/<name>.bind.at`, GENERATED header). Zero copies; upgrades to
   the shared bp propagate by rebuild.
2. **L2 copy reference** — `auto bp add --reference <variant>`; owned source with
   a provenance header (source bp + variant + date).
3. **L3 agent generation** — the loop above; landed files carry the same
   provenance header.

Rule of thumb: the number of copies is a linear function of drift risk — choose
the lowest channel that fits.

## Variant promotion review (L3 aftermath)

When a generated/copied blueprint exhibits a **structural difference the spec
does not declare**:

1. Open a promotion: add a `promotions:` subsection to the package's
   `spec.md` — difference description → slot / action-point proposal.
2. Review passes → the difference enters the spec formally (new slot, action
   point, or variant); the next consumer gets it for free via L1.
3. Review rejects → the difference stays app-side and is recorded in
   `docs/plans/KNOWN-DEBT-AND-RISKS.md` (does not block the consuming app).

This is the structural counterpart of the gotchas feedback loop: gotchas
capture *how assemblies go wrong*, promotions capture *what new structure
grows out of them*.
