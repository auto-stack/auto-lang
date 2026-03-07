# Migration Guide: AutoMan BPBE Architecture

This guide explains how to migrate existing AutoMan projects to the new **B.P.B.E.** architecture.

## Overview of Changes

In the previous version of AutoMan, the `builder` property in a `port` block determined both how the internal build happened and what external files were generated.

In the new architecture:
1.  **`backend`** (root level) determines the internal builder (e.g., `c` -> `Ninja`).
2.  **`builder`** (inside `port`) is now **deprecated** (but still supported for backward compatibility) and mapped to the internal builder logic.
3.  **`exports`** (inside `port`) lists supported project generation formats.

## 1. Update `pac.at`

### Before (Old)
```auto
// pac.at
project "my-project"

port "win32" {
    builder: "cmake"
    at: "build/win32"
}
```

### After (New)
```auto
// pac.at
project "my-project"
backend: "c" // Explicitly define the language backend

port "win32" {
    // os/arch/toolchain are recommended
    os: "windows"
    arch: "x86_64"
    toolchain: "msvc"
    
    exports: ["cmake"] // Use export system for CMakeLists.txt
    at: "build/win32"
}
```

## 2. CLI Command Changes

### Building
- **Old**: `am build` (automatically picked the port)
- **New**: `auto build --port win32` (explicit selection is now preferred, though automatic scanning still works).

### Project Generation
- **Old**: `am build` automatically generated `CMakeLists.txt` or IAR projects if the builder was set.
- **New**: Use the dedicated export command:
  ```bash
  auto export --port win32 --format cmake
  ```

## 3. Toolchain Configuration
If you were using custom toolchain properties in the `port` block, ensure they are compatible with the new `os`/`arch` schema. The `Port` struct now explicitly prioritizes these fields for environment detection.
