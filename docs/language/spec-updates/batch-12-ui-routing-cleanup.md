# Spec Update: Batch 12 — UI, Routing, Final Cleanup

**Date**: 2026-04-16
**Plans Referenced**: Plan 096 (UI widgets), Plan 105 (routing), Plan 106 (navigation)
**Source Files**: `ast.rs` (WidgetDecl, MsgDecl, ModelBlock, ViewBlock, NavCall stmt variants), `token.rs` (routes, outlet, link, route, nav keywords)
**Sections Updated**: UI Widgets and Routing (Section 21 — NEW), Appendices (updated)

## Old Content

No UI or routing documentation existed.

## New Content

### UI Widgets and Routing (Section 21)
- Widget declaration: `widget Name { model {}, msg {}, view {} }`
- Model block: state declarations with defaults
- Msg block: message type declarations
- View block: widget tree using AURA components
- `emit` for sending messages
- Routing: `routes { route(path, view) }`
- Navigation: `nav(path)`, `link(text, to)`, `outlet()`

### Appendices Updated
- Appendix A: Precedence table expanded from 9 to 11 levels
- Appendix B: Keywords expanded from 24 to 56
- Appendix C: NEW — Expression types list (55 variants from ast.rs)
- Removed old ASCII art grammar (outdated)
- Removed C/Rust implementation code examples (moved to Implementation Comparison, simplified)

### Cleanup Applied
- All `println()` → `print()` throughout
- Pattern matching arrows: `->` → `=>`
- F-string syntax: `{name}` → `$name` and `${expr}`
- Removed `while` keyword references
- Fixed lambda syntax: `|a, b|` → `(a, b) =>`
- Fixed old onclick syntax: `|| println(...)` → `=> print(...)`
- Version bumped to 0.2
- TOC restructured from 15 to 24 sections

## Notes

- WidgetDecl, MsgDecl, ModelBlock, ViewBlock are AST stmt variants for UI
- NavCall is an expression variant for router navigation
- AURA components (col, row, text, button) are documented in CLAUDE.md, not duplicated in spec
- The Implementation Comparison section was simplified (removed code examples, kept feature table)
