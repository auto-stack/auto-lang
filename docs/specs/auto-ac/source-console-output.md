# Console source profile (PLAN-863)

The independent source profile is `auto.source.host-console.draft`, HIR `auto.hir.host-console.draft` (schema 1, host-console-draft domain). `compile_host_console_source` and `verify_host_console` are its front door. Entry/script rules include the 862 Unit/int behavior: selected entry is `d_main`, explicit main returns int or Unit, scripts can have an int tail or Unit result. Other result types, implicit value returns from an explicit int main, and mixed top-level executable code with main are rejected. A value tail in a Unit main is discarded; it does not become an implicit return. A value-bearing return from Unit main is rejected.

`print(expr)` and `eprint(expr)` each take exactly one positional argument of `int` (i32), `bool`, borrowed `str`, owning `Str`, or `char`. Owning text converts to a readonly borrow without ownership transfer; repeated printing retains the guest owner. Each expression is evaluated once in source order. Calls produce Unit and observable effects; Unit cannot be stored as a local or coerced into an int/condition. Wrong arity/type/named argument uses `source.console-argument`. Dead and unused source is still checked.

Formatting is exact: i32 decimal ASCII without plus/leading zeros, MIN `-2147483648`; booleans `true`/`false`; text preserves UTF-8 bytes, embedded NUL and newlines; char emits its Unicode scalar in UTF-8. Each call appends exactly one byte `0a`, without spaces or normalization. Empty text therefore emits one LF. There is no interpolation/width/locale formatting.

`print` and `eprint` cannot be rebound as functions, locals or parameters in this domain (`source.console-reserved`). Other profiles keep their name-resolution rules and cannot acquire host intrinsics by changing a symbol. HIR declares exact typed intrinsic references and required capabilities; lowering converts them to the independent host table/call, never a guest import or trace instruction. Backend/native/default product routing does not change.

Reference: `../auto-av/host-import-and-console.md`. Implementation: `experimental/ac-core/src/source/console.rs`, host-console descriptor, shared `experimental/auto-mir-lower/src/hir_adapter.rs`.

来源：[PLAN-863](../../plans/archive/863-av-console-host-import-and-print.md)。
