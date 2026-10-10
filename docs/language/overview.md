# AutoLang: language overview

[中文](https://github.com/auto-stack/auto-lang/blob/master/docs/language/overview.cn.md) · [Auto ecosystem](https://github.com/auto-stack/auto-lang/blob/master/README.md) · [Hands-on Tour](../tour/ch01-hello.md)

This guide introduces the current language and its main execution paths. It was checked against
repository sources on **2026-10-10**, including the [v0.5 milestone notes](../releases/v0.5.md).
It is an introduction, rather than a claim that every feature has equal coverage in every backend.

## One language, several uses

Auto combines scripting with explicit types and a static shipping path. Its syntax also supplies
structured configuration and declarative interfaces. The same ecosystem includes the AutoVM,
transpilers, AutoMan build tools, AutoUI, and application-facing AI services.

| Use | Main path |
|---|---|
| Scripts and application logic | Run `.at` sources on AutoVM |
| Dynamic script conveniences | `.as` files or `#[script]`, lowered through the scripting pipeline |
| Native shipping | Transpile to Rust with a2r, then build with Rust tooling |
| Rust libraries | Built-in natives and build-time `dep` / `use.rs` shims |
| Python libraries | Embedded CPython through `use.py`; a2py emits Python |
| Interfaces | AutoUI state/events/views, rendered through Vue or native iced |
| Configuration and structured content | Objects, named values, and Node/Atom forms |

Rust is the main static shipping path. C, TypeScript, JavaScript, Python, GDScript, and Godot scene
outputs have separate generators and test coverage. The language does not promise automatic
compatibility with every library or identical behavior across all those targets.

## Running and building

With an `auto` binary on your PATH, run a source directly from the repository root:

```sh
auto docs/tour/ch01-hello/01_hello.at
```

The program is:

```auto
fn main() {
    print("Hello, World!")
}
```

`print` writes a line. Script files can also contain top-level statements; a `main` function is a
convenient way to organize a program, rather than a requirement for every script.

```sh
# Inspect the CLI and the available transpilation targets.
auto --help
auto trans --help

# Generate Rust from a single source.
auto trans --path docs/tour/ch01-hello/01_hello.at rust
```

The last command emits source; compiling and packaging the generated program is a subsequent step.
For project workflows, use `auto new`, `auto build`, and `auto run` with the project's `pac.at`.
[From Script to Ship](../script-to-ship/README.md) explains the development-to-release workflow.

For a source build, clone `auto-lang` and `auto-down` as adjacent directories. The current workspace
has a relative `autodown-core` dependency in the sibling repository. From `auto-lang`:

```sh
cargo build -p auto --release
```

Prepare a Rust/native build environment and matching CPython development environment for this
default CLI, whose default features include `ui-iced`, `python`, and `autodown`. The executable is
`target/release/auto` (`auto.exe` on Windows). For a CLI without the optional Python/AutoDown
integration, the package accepts `--no-default-features`; it still has the language crate's default
iced dependencies, and Cargo still needs to resolve the sibling path dependency. Consult the
[current manifests](https://github.com/auto-stack/auto-lang/blob/master/crates/auto/Cargo.toml), rather than assuming a standalone Rust-only clone.

## Bindings, values, and collections

Use `let` for an immutable binding, `var` for a mutable one, and `const` for constants. Types can be
inferred, or written after a name without a colon:

```auto
fn main() {
    let name = "Auto"
    var count int = 0
    count = count + 1
    print(f"Hello, $name!")
    print(count)
}
```

Basic values include integers, floating-point numbers, booleans, strings, arrays, and objects.
Ordinary typed bindings and dynamic scripting conveniences have different type rules; mutability
alone is not a blanket permission to change a binding's type. `None`/`Some` represent optional
values, while `nil` is a distinct value; they should not be treated as interchangeable spellings.

```auto
fn main() {
    let numbers = [1, 2, 3]
    var total = 0
    for n in numbers {
        total += n
    }
    let person = {name: "Auto", age: 5}
    print(total)
    print(person.name)
}
```

Strings support interpolation, and collections provide indexing and built-in operations.
See [collections](../tour/ch07-collections.md) for arrays, lists, maps, and iteration.

## Functions and control flow

Function parameters use `name Type`; a return type follows the parameter list. A function can use
`return` or leave its result as the last expression. Closures use `(parameters) => expression`.

```auto
fn add(a int, b int) int {
    a + b
}

fn main() {
    let base = 10
    let add_base = (x int) => x + base
    print(add(3, 4))
    print(add_base(5))
}
```

`if`/`else`, `for`, `while`, `loop`, `break`, and `continue` supply control flow. `is` performs
pattern matching with `->` arms:

```auto
fn describe(n int) str {
    is n {
        0 -> return "zero"
        1 -> return "one"
    }
    return "other"
}

fn main() {
    print(describe(0))
    print(describe(42))
}
```

[Functions](../tour/ch03-functions.md), [control flow](../tour/ch04-control.md), and
[patterns](../tour/ch05-patterns.md) have longer examples.

## Types, methods, specs, and generics

`type` defines fields and methods; `enum` defines alternatives, including variants carrying data.
`ext` adds methods. `spec` describes an interface contract, and type parameters express generic code.

```auto
type Point {
    x int
    y int
}

fn identity<T>(x T) T {
    return x
}

fn main() {
    let p = Point { x: 3, y: 4 }
    print(p.x)
    print(identity(42))
}
```

Ownership and borrowing are part of the language's static path. The `view`, `mut`, and `take`
vocabulary makes shared access, mutable access, and movement explicit. The VM's value handling and
Rust's generated ownership model are different implementations; use the relevant examples and
parity checks to establish behavior for your code.

Continue with [types](../tour/ch02-types.md), [methods and specs](../tour/ch08-methods.md),
[generics](../tour/ch09-generics.md), and [types and ownership](../script-to-ship/ch03-types-ownership.md).

## Errors and optional values

A return type such as `!int` marks an error-capable function. `Err(...)` returns a failure; `.?`
unwraps a success or propagates the error. `?T`, `Some`, `None`, and `??` handle optional values.

```auto
fn try_parse(s str) !int {
    if s == "" {
        return Err("empty string")
    }
    Ok(42)
}

fn chain() !int {
    let a = try_parse("hello").?
    let b = try_parse("").?
    Ok(a + b)
}

let result = chain()
is result {
    Ok(n) -> print(n)
    Err(e) -> print("propagated error")
}
```

This [error propagation fixture](https://github.com/auto-stack/auto-lang/blob/master/crates/auto-lang/test/vm/16_option_result/020_result_propagate/result_propagate.at) prints `propagated error`.

See [error handling](../tour/ch06-errors.md) and the [shipping examples](../script-to-ship/ch04-errors.md).
Python-facing script mode also has exception and context-manager handling; it is a distinct bridge
with its own lowering and interoperability tests.

## Async, Actors, and generators

Asynchronous values use `~T`, with `.await` to obtain a result. Actor programs use `task` state,
messages, and handlers; their VM scheduling and Rust shipping paths have separate tests.
[Actor fixtures](https://github.com/auto-stack/auto-lang/tree/master/crates/auto-lang/test/vm/23_actor) show actual message-based examples.

Generators use `yield`, producing values lazily as an iteration consumes them:

```auto
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}

fn main() {
    var sum = 0
    for n in counter() {
        sum = sum + n
    }
    print(sum)
}
```

This is the existing [VM generator example](https://github.com/auto-stack/auto-lang/blob/master/crates/auto-lang/test/vm/22_generator/001_sum/sum.at).
[Async examples](../tour/ch11-async.md) and [generator fixtures](https://github.com/auto-stack/auto-lang/tree/master/crates/auto-lang/test/vm/22_generator)
provide more detail. These capabilities are implemented; execution and transpilation coverage
should still be checked for the particular operation.

## Modules and existing ecosystems

Use `use` for Auto modules. AutoVM loads and links multi-module programs; project dependencies and
build configuration live in `pac.at`. [Module examples](../tour/ch10-modules.md) introduce visibility,
file organization, and imports.

| Bridge | What it supplies | Evidence and further reading |
|---|---|---|
| Rust | Natives and `dep`/`use.rs` shims, with signature metadata, layout probes, and concrete generic instances | [Rust ecosystem](https://github.com/auto-stack/auto-lang/blob/master/website/rust.md) · [Rust library cases](https://github.com/auto-stack/auto-lang/tree/master/parity/libs/rust) |
| Python | CPython objects through `use.py`; `.as` lowering, kwargs, exceptions, context managers, and a2py | [Python ecosystem](https://github.com/auto-stack/auto-lang/blob/master/website/python.md) · [Python/PyTorch cases](https://github.com/auto-stack/auto-lang/tree/master/parity/libs/python) |
| C | C bindings and C ABI interoperability; C transpilation is another path | [FFI guide](../guides/ffi-usage-guide.md) · [C binding examples](../tour/ch12-interop.md) |

Compatibility depends on the specific signature, ownership/layout requirements, library, runtime,
and backend. Rust lifetime-sensitive/unsafe interfaces and unrestricted Python language/library
compatibility are not implied by the existence of a bridge. See [parity](https://github.com/auto-stack/auto-lang/blob/master/parity/README.md)
for concrete AutoVM/transpiled/native comparisons.

## Configuration, Nodes, and AutoUI

Objects, named values, and Node forms make Auto useful for configuration and structured content.
[`pac.at`](https://github.com/auto-stack/auto-lang/blob/master/examples/ui/002-counter/pac.at) configures a project; [`auto-atom`](https://github.com/auto-stack/auto-lang/tree/master/crates/auto-atom)
provides the structured Atom data representation. Compile-time evaluation and annotations support
metaprogramming; the [comptime module](https://github.com/auto-stack/auto-lang/blob/master/docs/specs/auto-lang/comptime/overview.md) describes that path.

AutoUI uses `widget` blocks to connect model state, a view, and events. This is an excerpt from the
[current counter](https://github.com/auto-stack/auto-lang/blob/master/examples/ui/002-counter/src/front/app.at):

```auto,ignore
widget App {
    model {
        var count int = 0
    }
    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: () => {.count -= 1} }
                button "Reset" { onclick: () => {.count = 0} }
                button "+" { onclick: () => {.count += 1} }
            }
            style: "items-center gap-4 p-6"
        }
    }
}
```

Run the project with `auto run examples/ui/002-counter -r vue` or `-r vm`. Vue emits web components;
VM mode renders through iced, while the Rust UI path generates native code. State and events share
an authoring model, with backend-specific coverage. [AutoUI](https://github.com/auto-stack/auto-lang/blob/master/website/ui/index.md),
[UI projects](https://github.com/auto-stack/auto-lang/blob/master/examples/ui/README.md), and [Blueprints](https://github.com/auto-stack/auto-lang/blob/master/blueprints/README.md) introduce
components, styles, charts, and reusable designs. Android/ArkTS paths are currently demo/feasibility work.

## Tools, reference, and next steps

- **AutoMan / AutoCache:** project builds, dependency preparation, generation, and cached artifacts.
- **LSP / debugger:** editor diagnostics, navigation, and execution inspection.
- **DevTools / MCP:** live UI trees, layout, events, and screenshots; see the [verification guide](../guides/autoui-verification-and-mcp-guide.md).
- **Playground / books:** [browser-example page](https://github.com/auto-stack/auto-lang/blob/master/website/playground.md), [website development](https://github.com/auto-stack/auto-lang/blob/master/website/README.md), and [Auto books](https://github.com/auto-stack/books).
- **Self-hosting:** [`auto/`](https://github.com/auto-stack/auto-lang/tree/master/auto) contains the experimental Auto-written VM/transpiler;
  [current scope](https://github.com/auto-stack/auto-lang/blob/master/docs/specs/aavm/project.md) distinguishes this Rust-assisted loop from the reference implementation.

Use the [Tour](../tour/README.md) to learn by example, [Script to Ship](../script-to-ship/README.md)
for release workflows, and [v0.5 notes](../releases/v0.5.md) for progress and known boundaries.
The [full specification](specification.md) is an older versioned draft, not a frozen v0.5 specification.
Current module [Specs](https://github.com/auto-stack/auto-lang/blob/master/docs/specs/auto-lang/project.md), actual sources, and validation cases provide
more precise implementation details when that draft and current behavior differ.
