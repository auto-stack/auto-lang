# Spec Update: Batch 08 — Specs (Traits)

**Date**: 2026-04-16
**Plans Referenced**: Plan 019 (specs), Plan 057 (generic specs), Plan 059 (spec impl)
**Source Files**: `ast/spec.rs` (SpecDecl, SpecImpl, SpecMethod)
**Sections Updated**: Specs (Traits) (Section 13 — NEW)

## Old Content

No dedicated spec/trait section existed. The `has` keyword was listed but not documented.

## New Content

### Spec Declaration
- `spec Printable { fn print() void }`
- Generic specs: `spec Iterable<T> { ... }`
- Default methods: `fn log(msg str) void { ... }`

### Spec Implementation
- `type User has Printable { ... }`
- Multiple spec impls: `type File has Readable, Writable { ... }`

### SpecDecl Fields
- name, generic_params, methods, is_pub

### Transpilation Behavior
- C: vtables (struct of function pointers)
- Rust: native traits
- VM: dynamic dispatch via method registry

## Notes

- `has` keyword used for spec implementation declaration on types
- Spec methods with body provide default implementations (Plan 019 Stage 8.5)
- `SpecImpl` struct tracks spec_name and type_args for generic spec instances
