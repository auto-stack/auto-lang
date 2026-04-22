# Multi-Target Transpiler

Auto is designed as a multi-target transpiler that compiles to multiple mainstream languages, enabling true "write once, run anywhere" capabilities.

## Supported Targets

- **C** — For embedded systems and bare-metal programming
- **Rust** — For systems programming with memory safety
- **TypeScript** — For web and Node.js applications
- **Python** — For data science and rapid prototyping

## Zero-Cost Abstractions

Auto's transpiler preserves the semantics of high-level abstractions without runtime overhead. Concepts like generics, enums, and pattern matching are compiled down to efficient target-language code.

## Use Cases

- Cross-platform libraries that need to work across ecosystems
- Performance-critical code that needs C-level speed
- Web applications that compile to TypeScript
- Prototyping in Python, then compiling to C for production
