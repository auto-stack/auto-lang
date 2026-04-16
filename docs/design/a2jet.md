# a2jet - Jetpack Compose Code Generator

> Extracted from CLAUDE.md for reference. See CLAUDE.md for rules and quick reference.

### Overview

a2jet generates Jetpack Compose Kotlin code from AURA widgets, producing modern Android apps with Material3 design.

**Location**: `crates/auto-lang/src/ui_gen/jet/`

### Architecture

```text
AuraWidget → JetGenerator → Kotlin/Compose Code
                │
                ├── Material3Registry (component mappings)
                ├── FormGenerator (inputs, buttons)
                ├── LayoutGenerator (Column, Row, Box)
                ├── ListGenerator (LazyColumn, Grid)
                ├── NavigationGenerator (NavHost)
                ├── ModifierDsl (Tailwind → Compose)
                ├── StateConverter (model → mutableStateOf)
                └── ProjectGenerator (full Android project)
```

### Modules

| Module | Purpose | Lines |
|--------|---------|-------|
| `mod.rs` | Module exports & documentation | 80+ |
| `generator.rs` | Main JetGenerator | 800+ |
| `components.rs` | Material3 component registry | 200+ |
| `form.rs` | Form components (Input, Checkbox, Switch, Slider) | 400+ |
| `layout.rs` | Layout components (Column, Row, Box, Card, Scroll) | 350+ |
| `list.rs` | List components (LazyColumn, LazyRow, Grid) | 300+ |
| `modifier.rs` | Tailwind → Compose Modifier DSL | 250+ |
| `navigation.rs` | Navigation (NavHost, routes) | 300+ |
| `state.rs` | State management (mutableStateOf) | 150+ |
| `project.rs` | Android project generation | 850+ |

### Usage

#### Generate a Widget

```rust
use auto_lang::ui_gen::jet::JetGenerator;
use auto_lang::ui_gen::BackendGenerator;

let mut gen = JetGenerator::new();
let kotlin_code = gen.generate(&aura_widget)?;
```

#### Generate a Full Android Project

```rust
use auto_lang::ui_gen::jet::JetGenerator;

let gen = JetGenerator::new();

// With defaults
let files = gen.generate_project_default("MyApp");

// With custom package
let files = gen.generate_project_with_package("MyApp", "com.company.myapp");

// With custom theme
let files = gen.generate_project_with_theme("MyApp", "#6750A4", "#625B71");
```

### Project Generation

Full project generation creates:

```text
myapp/
├── app/
│   ├── src/main/
│   │   ├── java/com/example/myapp/
│   │   │   ├── MainActivity.kt
│   │   │   └── ui/
│   │   │       ├── theme/
│   │   │       │   ├── Theme.kt
│   │   │       │   ├── Color.kt
│   │   │       │   └── Type.kt
│   │   │       └── widgets/
│   │   └── AndroidManifest.xml
│   └── build.gradle.kts
├── build.gradle.kts
├── settings.gradle.kts
└── gradle/libs.versions.toml
```

### Component Mappings

| AURA Tag | Compose Component |
|----------|-------------------|
| `col` | `Column` |
| `row` | `Row` |
| `box` | `Box` |
| `card` | `Card` |
| `button` | `Button` |
| `input` | `OutlinedTextField` |
| `checkbox` | `Checkbox` |
| `switch`/`toggle` | `Switch` |
| `slider` | `Slider` |
| `list` | `LazyColumn` |
| `list-row` | `LazyRow` |
| `grid` | `LazyVerticalGrid` |

### Testing

```bash
# Run all a2jet tests
cargo test -p auto-lang jet

# Run specific module tests
cargo test -p auto-lang jet::generator
cargo test -p auto-lang jet::project

# Count tests
cargo test -p auto-lang jet 2>&1 | grep "test result:"
```

### Implementation Phases

| Phase | Content | Status |
|-------|---------|--------|
| Phase 1 | 基础结构 + 简单组件 | ✅ Complete |
| Phase 2 | 表单组件 | ✅ Complete |
| Phase 3 | Modifier DSL | ✅ Complete |
| Phase 4 | 布局与导航 | ✅ Complete |
| Phase 5 | 列表与数据 | ✅ Complete |
| Phase 6 | 项目生成 (auto build) | ✅ Complete |
| Phase 7 | 测试与文档 | ✅ Complete |

### Plan Files

- [Plan 113](../plans/113-a2jet-design.md) - Main design
- [Plan 114](../plans/114-a2jet-phase2-forms.md) - Form components
- [Plan 115](../plans/115-a2jet-phase4-layout.md) - Layout & Navigation
- [Plan 116](../plans/116-a2jet-phase5-lists.md) - Lists & Data
- [Plan 117](../plans/117-a2jet-phase6-project-gen.md) - Project generation
- [Plan 118](../plans/118-a2jet-phase7-docs-tests.md) - Documentation & Tests
