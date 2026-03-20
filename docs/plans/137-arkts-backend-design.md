# Plan 137: ArkTS (HarmonyOS) Backend - Design Document

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add ArkTS (HarmonyOS/Harmony Next) backend support for AutoUI, enabling AURA widgets to be transpiled into ArkTS code for HarmonyOS applications.

**Architecture:** AURA Widget → ArkGenerator → ArkTS Code (.ets files) → HarmonyOS Project Structure

**Tech Stack:** Rust, ArkTS, HarmonyOS SDK, hvigor build system

---

## ArkTS Project Structure (Generated)

```
arkts/
├── build-profile.json5          # Root build config
├── oh-package.json5             # Dependencies
├── hvigorfile.ts                # Build script
├── AppScope/
│   └── resources/
│       └── base/
│           └── element/
│               └── string.json  # App name
└── entry/
    ├── build-profile.json5
    ├── src/main/
    │   ├── ets/
    │   │   ├── entryability/
    │   │   │   └── EntryAbility.ets
    │   │   └── pages/
    │   │       └── Index.ets    # Generated from AURA
    │   ├── resources/
    │   │   └── base/
    │   │       └── profile/
    │   │           └── main_pages.json
    │   └── module.json5
    └── index.ets
```

---

## Component Mapping (AURA → ArkTS)

| AURA Tag | ArkTS Component | Notes |
|----------|-----------------|-------|
| `col` | `Column()` | Vertical layout |
| `row` | `Row()` | Horizontal layout |
| `text` | `Text()` | Text display |
| `button` | `Button()` | Clickable button |
| `input` | `TextInput()` | Text input field |
| `checkbox` | `Checkbox()` | Checkbox |
| `if` | `if () { }` | Conditional rendering |
| `for` | `ForEach()` | List rendering |

### Example Transformation

**AURA Input:**
```auto
widget App {
    msg Msg { Click }
    model {
        var count int = 0
    }
    view {
        col {
            text `Count: ${.count}` {}
            button (text: "Inc", onclick: .Click) {}
        }
    }
    on {
        .Click => {
            .count = .count + 1
        }
    }
}
```

**Generated ArkTS:**
```typescript
sealed class Msg {
  object Click : Msg
}

@Entry
@Component
struct App {
  @State count: number = 0

  private dispatch(msg: Msg): void {
    when (Msg.Click) {
      this.count += 1
    }
  }

  build() {
    Column() {
      Text(`Count: ${this.count}`)
      Button("Inc")
        .onClick(() => {
          this.dispatch(Msg.Click)
        })
    }
    .width('100%')
    .height('100%')
  }
}
```

---

## State & Message Handling

### State Management

| AURA | ArkTS | Notes |
|------|-------|-------|
| `var count int = 0` | `@State count: number = 0` | Local state |
| `var name str = ""` | `@State name: string = ""` | String state |
| `var items []Item = []` | `@State items: Item[] = []` | Array state |

### Message Dispatch Pattern

Following the jet backend pattern:

1. **Msg sealed class** - Generated from `msg Msg { ... }` block
2. **dispatch function** - Handles all messages with `when` statement
3. **Event handlers** - Call `this.dispatch(Msg.VariantName)`

```typescript
// Generated from msg block
sealed class Msg {
  object Click : Msg
  object Load : Msg
}

// Generated from handlers
private dispatch(msg: Msg): void {
  when (msg) {
    Msg.Click: {
      this.count += 1
    }
    Msg.Load: {
      this.count = 0
    }
  }
}

// In UI components
Button("Inc")
  .onClick(() => {
    this.dispatch(Msg.Click)
  })
```

---

## Module Structure

```
crates/auto-lang/src/ui_gen/ark/
├── mod.rs           # Module exports, ArkGenerator struct
├── generator.rs     # Main generator (widget → ArkTS code)
├── components.rs    # Component registry (AURA tag → ArkTS component)
├── state.rs          # State management (@State, dispatch pattern)
├── project.rs        # Project scaffolding (build-profile.json5, module.json5, etc.)
└── modifier.rs       # Style modifiers (width, height, fontSize, etc.)
```

---

## Integration with auto-man

### pac.at Configuration

```auto
name: "unified-demo"
scene: "ui"
backend: "arkts"    // or ["vue", "jet", "arkts"] for multi-backend
```

### Build Workflow

```
auto gen
    ↓
Read pac.at → backend = "arkts"
    ↓
Create arkts/ folder
    ↓
Generate project structure (project.rs)
    ↓
Transpile AURA → ArkTS (generator.rs)
    ↓
Write entry/src/main/ets/pages/Index.ets
```

---

## Implementation Phases

### Phase 1: Core Infrastructure
- Module structure (mod.rs, generator.rs)
- Basic ArkGenerator struct
- Project scaffolding (project.rs)

### Phase 2: Basic Components
- Component registry (components.rs)
- Column, Row, Text, Button mapping
- Modifier DSL (modifier.rs)

### Phase 3: State & Events
- State management (state.rs)
- Message sealed class generation
- Dispatch function generation
- Event handler wiring

### Phase 4: Testing & Examples
- Unit tests for generator
- Integration test with unified-demo
- End-to-end build verification

---

## Files Changed Summary

```
crates/auto-lang/src/ui_gen/
└── ark/                        # New module
    ├── mod.rs
    ├── generator.rs
    ├── components.rs
    ├── state.rs
    ├── project.rs
    └── modifier.rs

crates/auto-lang/src/ui_gen/
└── mod.rs                      # Add ark module export

crates/auto-man/src/
└── ark.rs                      # New: ArkTS build integration

examples/unified-demo/
└── pac.at                      # Update: backend: "arkts"
```

---

## Deferred (Future Work)

| Item | Reason |
|------|--------|
| Form components (Slider, Switch) | Phase 2 |
| Navigation/Routing | Phase 2 |
| List components (LazyForEach) | Phase 2 |
| Custom components | Phase 2 |
| Theme system | Phase 3 |
| Animation support | Phase 3 |

---

## Success Criteria

- [ ] `ark/` module compiles without errors
- [ ] `auto gen` generates valid HarmonyOS project structure
- [ ] Basic components (Column, Row, Text, Button) transpile correctly
- [ ] State management with @State decorator works
- [ ] Message dispatch pattern generates valid ArkTS code
- [ ] Generated project can be opened in DevEco Studio
