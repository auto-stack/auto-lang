# B.P.B.E. Architecture

The **B.P.B.E.** (Backend, Port, Builder, Export) architecture is the core design principle of AutoMan, designed to decouple project configuration, target hardware/environment, build orchestration, and third-party IDE integration.

## Components

### 1. Backend (B)
The **Backend** defines the compilation strategy for the project. It determines how source files are processed and what internal builder will be used.
- **Examples**: `c`, `rust`, `typescript`.
- **Location**: Defined at the root of `pac.at` via the `backend` property.

### 2. Port (P)
The **Port** represents the target environment or hardware where the code will run.
- **Attributes**: `os`, `arch`, `toolchain`.
- **Dependencies**: Ports can have their own `sdk` dependencies.
- **Exports**: Lists which project formats can be generated for this target.

### 3. Builder (B)
The **Builder** is the internal engine that orchestrates the actual compilation process.
- **Examples**: `Ninja`, `Cargo`, `Vite`.
- **Logic**: Strictly determined by the **Backend**. Users don't usually select the builder; they select the backend, and AutoMan picks the most efficient builder.

### 4. Export (E)
The **Export** system generates project files for third-party IDEs or external build systems.
- **Examples**: `CMakeLists.txt`, `IAR (.ewp)`, `GHS (.gpj)`.
- **Command**: `auto export --port <port> --format <format>`.

## Workflow

1. **Scan**: AutoMan reads `pac.at` and resolves all `port` configurations.
2. **Select Port**: The user selects a target port (e.g., `auto build --port win32`).
3. **Internal Build**: AutoMan uses the **Builder** to compile source code (e.g., using Ninja for C projects).
4. **External Export (Optional)**: If the user needs to use an external IDE, they can **Export** the project (e.g., `auto export --format iar`).

## Configuration Example

```auto
backend: "c"

port "stm32f4" {
    os: "none"
    arch: "armv7e-m"
    toolchain: "arm-none-eabi-gcc"
    
    sdk "cmsis" {
        at: "deps/cmsis"
    }
    
    exports: ["iar", "ghs"]
}
```
