# AutoVM Interpreter

AutoVM is Auto's dedicated virtual machine and interpreter, designed to run Auto code efficiently across all supported platforms.

## Architecture

AutoVM uses a register-based bytecode virtual machine with:
- Ahead-of-Time (AOT) compilation for production builds
- Just-in-Time (JIT) compilation for hot paths
- Incremental compilation for fast development cycles

## Hot Reloading

During development, AutoVM supports hot reloading of code changes without restarting the application. This works for:
- Function body updates
- Type definition changes
- Actor message handlers

## Cross-Platform

AutoVM runs on:
- Desktop: Windows, macOS, Linux
- Mobile: iOS, Android
- Embedded: bare-metal with no OS
- Web: via WebAssembly transpilation

## Debugging

AutoVM includes a built-in debugger with:
- Breakpoints and single-stepping
- Variable inspection
- Memory profiling
- Actor message tracing
