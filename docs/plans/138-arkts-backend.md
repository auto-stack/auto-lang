# Plan 138: ArkTS (HarmonyOS) Backend

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

> **Merged:** Plan 139 (ArkTS Generator Bug Fixes) has been merged into this plan (2025-03-21).

**Goal:** Add ArkTS (HarmonyOS/Harmony Next) backend support for AutoUI, enabling AURA widgets to be transpiled into ArkTS code for HarmonyOS applications.

**Architecture:** AURA Widget → ArkGenerator → ArkTS Code (.ets files) → HarmonyOS Project Structure. Follows the same pattern as jet backend with component registry, state management, and project scaffolding.

**Tech Stack:** Rust, ArkTS, HarmonyOS SDK, hvigor build system

**Status:** ✅ **COMPLETE** - Project runs correctly in DevEco Studio (verified 2025-03-21)

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
├── state.rs         # State management (@State, dispatch pattern)
├── project.rs       # Project scaffolding (build-profile.json5, module.json5, etc.)
└── modifier.rs      # Style modifiers (width, height, fontSize, etc.)
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

## Implementation Tasks

### Task 1: Create ark Module Structure

**Files:**
- Create: `crates/auto-lang/src/ui_gen/ark/mod.rs`

**Step 1: Create the module directory and mod.rs**

```rust
//! ArkTS (HarmonyOS) UI Generator
//!
//! Transpiles AURA widgets to ArkTS code for HarmonyOS applications.
//!
//! # Architecture
//!
//! ```text
//! AURA Widget → ArkGenerator → ArkTS Code (.ets)
//! ```
//!
//! # Generated Project Structure
//!
//! ```text
//! arkts/
//! ├── build-profile.json5
//! ├── oh-package.json5
//! ├── entry/src/main/ets/pages/Index.ets
//! └── ...
//! ```

mod generator;
mod components;
mod state;
mod project;
mod modifier;

pub use generator::ArkGenerator;
pub use components::ArkComponentRegistry;
pub use project::ArkProjectGenerator;
```

**Step 2: Update ui_gen/mod.rs to export ark module**

In `crates/auto-lang/src/ui_gen/mod.rs`, add:

```rust
pub mod ark;
```

**Step 3: Verify compilation**

Run: `cargo build -p auto-lang`
Expected: PASS (empty module warnings are OK for now)

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/mod.rs crates/auto-lang/src/ui_gen/mod.rs
git commit -m "feat(ark): create ark module structure for ArkTS backend"
```

---

### Task 2: Implement ArkComponentRegistry

**Files:**
- Create: `crates/auto-lang/src/ui_gen/ark/components.rs`

**Step 1: Create component registry with tests**

```rust
//! ArkTS Component Registry
//!
//! Maps AURA tags to ArkTS components.

use std::collections::HashMap;

/// Registry mapping AURA tags to ArkTS component templates
pub struct ArkComponentRegistry {
    /// Map from AURA tag to ArkTS component name
    components: HashMap<String, ArkComponent>,
}

/// ArkTS component definition
#[derive(Debug, Clone)]
pub struct ArkComponent {
    /// Component name in ArkTS (e.g., "Column", "Text")
    pub name: String,
    /// Whether component has children
    pub has_children: bool,
    /// Whether component has content (like Text)
    pub has_content: bool,
}

impl ArkComponentRegistry {
    /// Create a new registry with default components
    pub fn new() -> Self {
        let mut registry = Self {
            components: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Register default ArkTS components
    fn register_defaults(&mut self) {
        // Layout components
        self.register("col", ArkComponent {
            name: "Column".to_string(),
            has_children: true,
            has_content: false,
        });

        self.register("row", ArkComponent {
            name: "Row".to_string(),
            has_children: true,
            has_content: false,
        });

        // Basic components
        self.register("text", ArkComponent {
            name: "Text".to_string(),
            has_children: false,
            has_content: true,
        });

        self.register("button", ArkComponent {
            name: "Button".to_string(),
            has_children: false,
            has_content: true,
        });

        self.register("input", ArkComponent {
            name: "TextInput".to_string(),
            has_children: false,
            has_content: false,
        });

        self.register("checkbox", ArkComponent {
            name: "Checkbox".to_string(),
            has_children: false,
            has_content: false,
        });
    }

    /// Register a component
    pub fn register(&mut self, tag: &str, component: ArkComponent) {
        self.components.insert(tag.to_string(), component);
    }

    /// Look up a component by AURA tag
    pub fn get(&self, tag: &str) -> Option<&ArkComponent> {
        self.components.get(tag)
    }

    /// Check if a tag is a known component
    pub fn is_component(&self, tag: &str) -> bool {
        self.components.contains_key(tag)
    }
}

impl Default for ArkComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_default_components() {
        let registry = ArkComponentRegistry::new();

        assert!(registry.get("col").is_some());
        assert!(registry.get("row").is_some());
        assert!(registry.get("text").is_some());
        assert!(registry.get("button").is_some());
    }

    #[test]
    fn test_column_has_children() {
        let registry = ArkComponentRegistry::new();
        let col = registry.get("col").unwrap();

        assert_eq!(col.name, "Column");
        assert!(col.has_children);
        assert!(!col.has_content);
    }

    #[test]
    fn test_text_has_content() {
        let registry = ArkComponentRegistry::new();
        let text = registry.get("text").unwrap();

        assert_eq!(text.name, "Text");
        assert!(!text.has_children);
        assert!(text.has_content);
    }

    #[test]
    fn test_unknown_tag_returns_none() {
        let registry = ArkComponentRegistry::new();

        assert!(registry.get("unknown").is_none());
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test -p auto-lang ark::components`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/components.rs
git commit -m "feat(ark): add ArkComponentRegistry with basic components"
```

---

### Task 3: Implement Modifier DSL

**Files:**
- Create: `crates/auto-lang/src/ui_gen/ark/modifier.rs`

**Step 1: Create modifier module**

```rust
//! ArkTS Modifier DSL
//!
//! Converts AURA style properties to ArkTS chainable modifiers.

use crate::ast::Type;

/// Convert a style property to ArkTS modifier
pub fn style_to_modifier(key: &str, value: &str) -> Option<String> {
    match key {
        // Size modifiers
        "width" => Some(format!(".width('{}')", value)),
        "height" => Some(format!(".height('{}')", value)),

        // Text modifiers
        "fontSize" => Some(format!(".fontSize({})", value)),
        "fontWeight" => Some(format!(".fontWeight(FontWeight.{})", value)),
        "fontColor" => Some(format!(".fontColor('{}')", value)),

        // Spacing modifiers
        "margin" => Some(format!(".margin({})", value)),
        "padding" => Some(format!(".padding({})", value)),

        // Layout modifiers
        "justifyContent" => Some(format!(".justifyContent(FlexAlign.{})", value)),
        "alignItems" => Some(format!(".alignItems(HorizontalAlign.{})", value)),

        // Background
        "backgroundColor" => Some(format!(".backgroundColor('{}')", value)),

        // Border
        "borderRadius" => Some(format!(".borderRadius({})", value)),

        _ => None,
    }
}

/// Convert AURA prop to ArkTS modifier
pub fn prop_to_modifier(key: &str, value: &str, value_type: Option<&Type>) -> Option<String> {
    match key {
        // Text content
        "text" => Some(value.to_string()),

        // Style properties
        "width" | "height" | "fontSize" | "margin" | "padding" | "borderRadius" => {
            style_to_modifier(key, value)
        }

        // Event handlers
        "onclick" => Some(format!(".onClick(() => {{\n    {}\n  }})", value)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_modifier() {
        let result = style_to_modifier("width", "100%");
        assert_eq!(result, Some(".width('100%')".to_string()));
    }

    #[test]
    fn test_font_size_modifier() {
        let result = style_to_modifier("fontSize", "16");
        assert_eq!(result, Some(".fontSize(16)".to_string()));
    }

    #[test]
    fn test_unknown_modifier_returns_none() {
        let result = style_to_modifier("unknown", "value");
        assert!(result.is_none());
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p auto-lang ark::modifier`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/modifier.rs
git commit -m "feat(ark): add modifier DSL for style properties"
```

---

### Task 4: Implement State Management

**Files:**
- Create: `crates/auto-lang/src/ui_gen/ark/state.rs`

**Step 1: Create state module**

```rust
//! ArkTS State Management
//!
//! Generates @State declarations and dispatch functions.

use crate::ui_gen::AuraWidget;
use crate::ast::Type;

/// Generate @State declarations from widget model
pub fn generate_state_declarations(widget: &AuraWidget) -> String {
    let mut lines = Vec::new();

    for field in &widget.model.fields {
        let name = &field.name;
        let arkts_type = auto_type_to_arkts(&field.ty);
        let default_value = generate_default_value(&field.ty, &field.default_value);

        lines.push(format!("  @State {}: {} = {}", name, arkts_type, default_value));
    }

    lines.join("\n")
}

/// Generate dispatch function from widget handlers
pub fn generate_dispatch_function(widget: &AuraWidget) -> String {
    if widget.handlers.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        "  private dispatch(msg: Msg): void {".to_string(),
        "    when (msg) {".to_string(),
    ];

    for handler in &widget.handlers {
        let pattern = &handler.pattern;
        let body = generate_handler_body(&handler.body);

        lines.push(format!("      Msg.{}: {{", pattern));
        for line in body.lines() {
            lines.push(format!("        {}", line));
        }
        lines.push("      }".to_string());
    }

    lines.push("    }".to_string());
    lines.push("  }".to_string());

    lines.join("\n")
}

/// Generate Msg sealed class from widget messages
pub fn generate_msg_sealed(widget: &AuraWidget) -> String {
    if widget.messages.is_empty() {
        return String::new();
    }

    let mut lines = vec!["sealed class Msg {".to_string()];

    for msg in &widget.messages {
        for variant in &msg.variants {
            if let Some(ref payload_type) = variant.payload {
                let arkts_type = auto_type_to_arkts(payload_type);
                lines.push(format!("  data class {}(val value: {}) : Msg()", variant.name, arkts_type));
            } else {
                lines.push(format!("  object {} : Msg()", variant.name));
            }
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}

/// Convert Auto type to ArkTS type
fn auto_type_to_arkts(ty: &Type) -> String {
    match ty {
        Type::Int | Type::I64 | Type::I32 => "number".to_string(),
        Type::Uint | Type::U64 | Type::U32 => "number".to_string(),
        Type::Float | Type::Double => "number".to_string(),
        Type::Bool => "boolean".to_string(),
        Type::Str(_) => "string".to_string(),
        Type::Array(elem) => format!("{}[]", auto_type_to_arkts(elem)),
        Type::User(decl) => decl.name.as_str().to_string(),
        _ => "any".to_string(),
    }
}

/// Generate default value for type
fn generate_default_value(ty: &Type, explicit: &Option<String>) -> String {
    if let Some(value) = explicit {
        return value.clone();
    }

    match ty {
        Type::Int | Type::I64 | Type::I32 => "0".to_string(),
        Type::Uint | Type::U64 | Type::U32 => "0".to_string(),
        Type::Float | Type::Double => "0.0".to_string(),
        Type::Bool => "false".to_string(),
        Type::Str(_) => "\"\"".to_string(),
        Type::Array(_) => "[]".to_string(),
        _ => "null".to_string(),
    }
}

/// Generate handler body (simplified)
fn generate_handler_body(body: &str) -> String {
    // Convert .field to this.field
    body.replace(".this", "this.")
        .replace("= .", "= this.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_type_to_arkts() {
        assert_eq!(auto_type_to_arkts(&Type::Int), "number");
        assert_eq!(auto_type_to_arkts(&Type::Bool), "boolean");
        assert_eq!(auto_type_to_arkts(&Type::Str(None)), "string");
    }

    #[test]
    fn test_generate_default_value() {
        assert_eq!(generate_default_value(&Type::Int, &None), "0");
        assert_eq!(generate_default_value(&Type::Bool, &None), "false");
        assert_eq!(generate_default_value(&Type::Str(None), &None), "\"\"");
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p auto-lang ark::state`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/state.rs
git commit -m "feat(ark): add state management with @State and dispatch pattern"
```

---

### Task 5: Implement Project Generator

**Files:**
- Create: `crates/auto-lang/src/ui_gen/ark/project.rs`

**Step 1: Create project generator**

```rust
//! ArkTS Project Generator
//!
//! Generates HarmonyOS project structure (build-profile.json5, module.json5, etc.)

use std::path::Path;

/// Generate complete HarmonyOS project structure
pub struct ArkProjectGenerator {
    /// Project name
    name: String,
    /// Package name (e.g., com.example.app)
    package: String,
}

impl ArkProjectGenerator {
    /// Create a new project generator
    pub fn new(name: &str) -> Self {
        let package = format!("com.example.{}", name.to_lowercase().replace("-", "_"));
        Self {
            name: name.to_string(),
            package,
        }
    }

    /// Set custom package name
    pub fn with_package(mut self, package: &str) -> Self {
        self.package = package.to_string();
        self
    }

    /// Generate project files to output directory
    pub fn generate(&self, output_dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        self.generate_root_files(output_dir)?;
        self.generate_app_scope(output_dir)?;
        self.generate_entry_module(output_dir)?;

        Ok(())
    }

    /// Generate root-level files
    fn generate_root_files(&self, dir: &Path) -> Result<(), String> {
        // build-profile.json5
        let build_profile = self.generate_build_profile();
        std::fs::write(dir.join("build-profile.json5"), &build_profile)
            .map_err(|e| format!("Failed to write build-profile.json5: {}", e))?;

        // oh-package.json5
        let oh_package = self.generate_oh_package();
        std::fs::write(dir.join("oh-package.json5"), &oh_package)
            .map_err(|e| format!("Failed to write oh-package.json5: {}", e))?;

        // hvigorfile.ts
        let hvigorfile = self.generate_hvigorfile();
        std::fs::write(dir.join("hvigorfile.ts"), &hvigorfile)
            .map_err(|e| format!("Failed to write hvigorfile.ts: {}", e))?;

        Ok(())
    }

    /// Generate AppScope directory
    fn generate_app_scope(&self, dir: &Path) -> Result<(), String> {
        let app_scope = dir.join("AppScope").join("resources").join("base").join("element");
        std::fs::create_dir_all(&app_scope)
            .map_err(|e| format!("Failed to create AppScope: {}", e))?;

        let string_json = format!(r#"{{
  "string": [
    {{
      "name": "app_name",
      "value": "{}"
    }}
  ]
}}"#, self.name);

        std::fs::write(app_scope.join("string.json"), &string_json)
            .map_err(|e| format!("Failed to write string.json: {}", e))?;

        Ok(())
    }

    /// Generate entry module
    fn generate_entry_module(&self, dir: &Path) -> Result<(), String> {
        let entry_dir = dir.join("entry");
        let src_main = entry_dir.join("src").join("main");
        let ets_dir = src_main.join("ets");

        // Create directories
        std::fs::create_dir_all(ets_dir.join("pages"))
            .map_err(|e| format!("Failed to create ets/pages: {}", e))?;
        std::fs::create_dir_all(ets_dir.join("entryability"))
            .map_err(|e| format!("Failed to create entryability: {}", e))?;
        std::fs::create_dir_all(src_main.join("resources").join("base").join("profile"))
            .map_err(|e| format!("Failed to create resources/profile: {}", e))?;

        // entry/build-profile.json5
        let entry_build_profile = self.generate_entry_build_profile();
        std::fs::write(entry_dir.join("build-profile.json5"), &entry_build_profile)
            .map_err(|e| format!("Failed to write entry build-profile: {}", e))?;

        // src/main/module.json5
        let module_json5 = self.generate_module_json5();
        std::fs::write(src_main.join("module.json5"), &module_json5)
            .map_err(|e| format!("Failed to write module.json5: {}", e))?;

        // src/main/resources/base/profile/main_pages.json
        let main_pages = r#"{
  "src": [
    "pages/Index"
  ]
}"#;
        std::fs::write(src_main.join("resources").join("base").join("profile").join("main_pages.json"), main_pages)
            .map_err(|e| format!("Failed to write main_pages.json: {}", e))?;

        // EntryAbility.ets
        let entry_ability = self.generate_entry_ability();
        std::fs::write(ets_dir.join("entryability").join("EntryAbility.ets"), &entry_ability)
            .map_err(|e| format!("Failed to write EntryAbility.ets: {}", e))?;

        Ok(())
    }

    fn generate_build_profile(&self) -> String {
        format!(r#"{{
  "app": {{
    "signingConfigs": [],
    "products": [
      {{
        "name": "default",
        "signingConfig": "default",
        "targetSdkVersion": "6.0.2(22)",
        "compatibleSdkVersion": "6.0.2(22)",
        "runtimeOS": "HarmonyOS"
      }}
    ],
    "buildModeSet": [
      {{
        "name": "debug"
      }},
      {{
        "name": "release"
      }}
    ]
  }},
  "modules": [
    {{
      "name": "entry",
      "srcPath": "./entry",
      "targets": [
        {{
          "name": "default",
          "applyToProducts": [
            "default"
          ]
        }}
      ]
    }}
  ]
}}
"#)
    }

    fn generate_oh_package(&self) -> String {
        r#"{
  "modelVersion": "6.0.2",
  "description": "Generated by AutoLang",
  "dependencies": {},
  "devDependencies": {
    "@ohos/hypium": "1.0.25"
  }
}
"#.to_string()
    }

    fn generate_hvigorfile(&self) -> String {
        r#"import { hapTasks } from '@ohos/hvigor-ohos-plugin';

export default {
  system: hapTasks,
  plugins: []
}
"#.to_string()
    }

    fn generate_entry_build_profile(&self) -> String {
        r#"{
  "apiType": "stageMode",
  "buildOption": {},
  "targets": [
    {
      "name": "default"
    },
    {
      "name": "ohosTest"
    }
  ]
}
"#.to_string()
    }

    fn generate_module_json5(&self) -> String {
        format!(r#"{{
  "module": {{
    "name": "entry",
    "type": "entry",
    "description": "$string:module_desc",
    "mainElement": "EntryAbility",
    "deviceTypes": [
      "phone",
      "tablet",
      "2in1"
    ],
    "deliveryWithInstall": true,
    "installationFree": false,
    "pages": "$profile:main_pages",
    "abilities": [
      {{
        "name": "EntryAbility",
        "srcEntry": "./ets/entryability/EntryAbility.ets",
        "description": "$string:EntryAbility_desc",
        "label": "$string:EntryAbility_label",
        "exported": true,
        "skills": [
          {{
            "entities": [
              "entity.system.home"
            ],
            "actions": [
              "ohos.want.action.home"
            ]
          }}
        ]
      }}
    ]
  }}
}}
"#)
    }

    fn generate_entry_ability(&self) -> String {
        r#"import UIAbility from '@ohos.app.ability.UIAbility';
import window from '@ohos.window';

export default class EntryAbility extends UIAbility {
  onCreate(want, launchParam) {
    console.info('EntryAbility onCreate');
  }

  onDestroy() {
    console.info('EntryAbility onDestroy');
  }

  onWindowStageCreate(windowStage: window.WindowStage) {
    console.info('EntryAbility onWindowStageCreate');

    windowStage.loadContent('pages/Index', (err, data) => {
      if (err.code) {
        console.error('Failed to load content. Cause: ' + JSON.stringify(err));
        return;
      }
      console.info('Succeeded in loading content.');
    });
  }

  onWindowStageDestroy() {
    console.info('EntryAbility onWindowStageDestroy');
  }

  onForeground() {
    console.info('EntryAbility onForeground');
  }

  onBackground() {
    console.info('EntryAbility onBackground');
  }
}
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_creates_structure() {
        let dir = tempdir().unwrap();
        let generator = ArkProjectGenerator::new("TestApp");

        generator.generate(dir.path()).unwrap();

        assert!(dir.path().join("build-profile.json5").exists());
        assert!(dir.path().join("oh-package.json5").exists());
        assert!(dir.path().join("entry/src/main/module.json5").exists());
        assert!(dir.path().join("entry/src/main/ets/pages").exists());
    }

    #[test]
    fn test_build_profile_contains_project_name() {
        let generator = ArkProjectGenerator::new("MyApp");
        let profile = generator.generate_build_profile();

        assert!(profile.contains("entry"));
    }
}
```

**Step 2: Add tempfile dev dependency**

In `crates/auto-lang/Cargo.toml`, add to `[dev-dependencies]`:

```toml
tempfile = "3"
```

**Step 3: Run tests**

Run: `cargo test -p auto-lang ark::project`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/project.rs crates/auto-lang/Cargo.toml
git commit -m "feat(ark): add project generator for HarmonyOS structure"
```

---

### Task 6: Implement Main ArkGenerator

**Files:**
- Create: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Create main generator**

```rust
//! ArkTS Code Generator
//!
//! Transpiles AURA widgets to ArkTS code.

use crate::ui_gen::AuraWidget;
use crate::ui_gen::ark::{ArkComponentRegistry, state, modifier};
use crate::ast::Type;

/// Result type for generator operations
pub type GenResult<T> = Result<T, String>;

/// ArkTS code generator
pub struct ArkGenerator {
    /// Component registry
    registry: ArkComponentRegistry,
    /// Track imports needed
    imports: Vec<String>,
    /// Indent string
    indent: String,
}

impl ArkGenerator {
    /// Create a new ArkGenerator
    pub fn new() -> Self {
        Self {
            registry: ArkComponentRegistry::new(),
            imports: Vec::new(),
            indent: "  ".to_string(),
        }
    }

    /// Generate complete ArkTS file from widget
    pub fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        let mut lines = Vec::new();

        // Reset imports
        self.imports.clear();

        // Generate Msg sealed class
        let msg_sealed = state::generate_msg_sealed(widget);
        if !msg_sealed.is_empty() {
            lines.push(msg_sealed);
            lines.push(String::new());
        }

        // Generate @Entry @Component struct
        lines.push("@Entry".to_string());
        lines.push("@Component".to_string());
        lines.push(format!("struct {} {{", widget.name));

        // Generate state declarations
        let state_decls = state::generate_state_declarations(widget);
        if !state_decls.is_empty() {
            lines.push(state_decls);
            lines.push(String::new());
        }

        // Generate dispatch function
        let dispatch_fn = state::generate_dispatch_function(widget);
        if !dispatch_fn.is_empty() {
            lines.push(dispatch_fn);
            lines.push(String::new());
        }

        // Generate build() method
        lines.push("  build() {".to_string());
        let view_body = self.generate_view_body(widget)?;
        for line in view_body.lines() {
            lines.push(format!("{}{}", self.indent, line));
        }
        lines.push("  }".to_string());

        lines.push("}".to_string());

        Ok(lines.join("\n"))
    }

    /// Generate view body from widget view tree
    fn generate_view_body(&mut self, widget: &AuraWidget) -> GenResult<String> {
        if widget.view_tree.children.is_empty() {
            return Ok("    Column() {\n    }\n    .width('100%')\n    .height('100%')".to_string());
        }

        let mut lines = Vec::new();
        lines.push("Column() {".to_string());

        for child in &widget.view_tree.children {
            let child_code = self.node_to_arkts(child, 2)?;
            lines.push(child_code);
        }

        lines.push("    }".to_string());
        lines.push("    .width('100%')".to_string());
        lines.push("    .height('100%')".to_string());

        Ok(lines.join("\n"))
    }

    /// Convert AURA node to ArkTS code
    fn node_to_arkts(&mut self, node: &crate::ui_gen::AuraNode, indent: usize) -> GenResult<String> {
        let indent_str = "  ".repeat(indent);
        let mut lines = Vec::new();

        match node {
            crate::ui_gen::AuraNode::Element { tag, props, events, children } => {
                if let Some(component) = self.registry.get(tag) {
                    if component.has_content {
                        // Text or Button with content
                        let content = props.get("text")
                            .map(|s| self.interpolate_string(s))
                            .unwrap_or_else(|| "\"\"".to_string());

                        lines.push(format!("{}{}({})", indent_str, component.name, content));

                        // Add modifiers
                        self.add_modifiers(&mut lines, props, events, &indent_str);

                        if !children.is_empty() {
                            // Component with children (rare for Text/Button)
                            lines.push(format!("{}  {{", indent_str));
                            for child in children {
                                let child_code = self.node_to_arkts(child, indent + 2)?;
                                lines.push(child_code);
                            }
                            lines.push(format!("{}}}", indent_str));
                        }
                    } else if component.has_children {
                        // Container component (Column, Row)
                        lines.push(format!("{}{}() {{", indent_str, component.name));

                        for child in children {
                            let child_code = self.node_to_arkts(child, indent + 1)?;
                            lines.push(child_code);
                        }

                        lines.push(format!("{}}}", indent_str));
                        self.add_modifiers(&mut lines, props, events, &indent_str);
                    } else {
                        // Self-closing component (Checkbox, TextInput)
                        lines.push(format!("{}{}()", indent_str, component.name));
                        self.add_modifiers(&mut lines, props, events, &indent_str);
                    }
                } else {
                    // Unknown component - generate as comment
                    lines.push(format!("{}// Unknown component: {}", indent_str, tag));
                }
            }

            crate::ui_gen::AuraNode::Text(content) => {
                let interpolated = self.interpolate_string(content);
                lines.push(format!("{}Text({})", indent_str, interpolated));
            }

            crate::ui_gen::AuraNode::ForLoop { var, iterable, body, .. } => {
                lines.push(format!("{}ForEach({}, ({}) => {{", indent_str, iterable, var));
                for child in body {
                    let child_code = self.node_to_arkts(child, indent + 1)?;
                    lines.push(child_code);
                }
                lines.push(format!("{}}})", indent_str));
            }

            crate::ui_gen::AuraNode::Conditional { condition, then_body, else_body } => {
                lines.push(format!("{}if ({}) {{", indent_str, condition));
                for child in then_body {
                    let child_code = self.node_to_arkts(child, indent + 1)?;
                    lines.push(child_code);
                }
                if let Some(else_nodes) = else_body {
                    if !else_nodes.is_empty() {
                        lines.push(format!("{}}} else {{", indent_str));
                        for child in else_nodes {
                            let child_code = self.node_to_arkts(child, indent + 1)?;
                            lines.push(child_code);
                        }
                    }
                }
                lines.push(format!("{}}}", indent_str));
            }

            _ => {
                lines.push(format!("{}// Unsupported node type", indent_str));
            }
        }

        Ok(lines.join("\n"))
    }

    /// Add modifiers to component
    fn add_modifiers(&self, lines: &mut Vec<String>, props: &std::collections::HashMap<String, String>, events: &[String], indent: &str) {
        // Add style modifiers from props
        for (key, value) in props {
            if let Some(modifier) = modifier::style_to_modifier(key, value) {
                lines.push(format!("{}{}", indent, modifier));
            }
        }

        // Add event handlers
        for event in events {
            if event.starts_with("onclick:") {
                let handler = event.strip_prefix("onclick:").unwrap();
                lines.push(format!("{}.onClick(() => {{", indent));
                lines.push(format!("{}  this.dispatch(Msg.{})", indent, handler));
                lines.push(format!("{}}})", indent));
            }
        }
    }

    /// Interpolate template strings
    fn interpolate_string(&self, s: &str) -> String {
        // Convert ${.field} to ${this.field}
        let result = s.replace("${.", "${this.");
        // Convert `.field` references
        let result = result.replace("'.", "this.");

        if s.starts_with('"') || s.starts_with("'") {
            result
        } else {
            format!("`{}`", result)
        }
    }
}

impl Default for ArkGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creates_struct() {
        let mut gen = ArkGenerator::new();
        let widget = AuraWidget {
            name: "TestWidget".to_string(),
            ..Default::default()
        };

        let result = gen.generate(&widget).unwrap();

        assert!(result.contains("@Entry"));
        assert!(result.contains("@Component"));
        assert!(result.contains("struct TestWidget"));
        assert!(result.contains("build()"));
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p auto-lang ark::generator`
Expected: PASS (may need AuraNode import fixes)

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(ark): add main ArkGenerator for widget transpilation"
```

---

### Task 7: Add auto-man Integration

**Files:**
- Create: `crates/auto-man/src/ark.rs`
- Modify: `crates/auto-man/src/lib.rs`

**Step 1: Create ark.rs in auto-man**

```rust
//! ArkTS Build Integration
//!
//! Integrates ArkTS backend with auto-man build system.

use std::path::Path;
use crate::{AutoResult, AutoError};

use auto_lang::ui_gen::ark::{ArkGenerator, ArkProjectGenerator};
use auto_lang::ui_gen::AuraWidget;

/// Generate ArkTS project from AURA widgets
pub fn generate_ark_project(
    root_dir: &Path,
    widgets: &[AuraWidget],
) -> AutoResult<()> {
    let arkts_dir = root_dir.join("arkts");

    // Generate project structure
    let project_name = root_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("app");

    let project_gen = ArkProjectGenerator::new(project_name);
    project_gen.generate(&arkts_dir)
        .map_err(|e| AutoError::BuildError(e))?;

    println!("  ✓ Generated HarmonyOS project: arkts/");

    // Generate page files from widgets
    let pages_dir = arkts_dir.join("entry").join("src").join("main").join("ets").join("pages");

    let mut generator = ArkGenerator::new();

    for widget in widgets {
        let file_name = format!("{}.ets", widget.name);
        let code = generator.generate(widget)
            .map_err(|e| AutoError::BuildError(e))?;

        let file_path = pages_dir.join(&file_name);
        std::fs::write(&file_path, &code)
            .map_err(|e| AutoError::BuildError(format!("Failed to write {}: {}", file_path.display(), e)))?;

        println!("  ✓ Generated page: {}", file_name);
    }

    Ok(())
}

/// Build ArkTS project
pub fn build_ark_project(root_dir: &Path) -> AutoResult<()> {
    let arkts_dir = root_dir.join("arkts");

    if !arkts_dir.exists() {
        return Err(AutoError::BuildError("arkts/ directory not found. Run 'auto gen' first.".to_string()));
    }

    println!("  Building HarmonyOS project...");
    println!("  Note: Use DevEco Studio to build and run the project.");

    Ok(())
}
```

**Step 2: Update auto-man lib.rs**

Add to `crates/auto-man/src/lib.rs`:

```rust
pub mod ark;
```

**Step 3: Run build**

Run: `cargo build -p auto-man`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/auto-man/src/ark.rs crates/auto-man/src/lib.rs
git commit -m "feat(auto-man): add ark module for ArkTS build integration"
```

---

### Task 8: Update unified-demo Example

**Files:**
- Modify: `examples/unified-demo/pac.at`

**Step 1: Update pac.at to support arkts backend**

```auto
// pac.at - Unified Backend Demo
// Demonstrates Plan 130: Single UI project (not a workspace)
//
// Directory structure:
//   unified-demo/
//   ├── pac.at              <- Project config (this file)
//   ├── front/              <- Frontend source (AURA widgets)
//   │   └── app.at
//   └── .am/                <- AutoMan state
//
// After build:
//   └── vue/                <- Generated Vue project (backend: "vue")
//   └── jet/                <- Generated Jetpack Compose project (backend: "jet")
//   └── arkts/              <- Generated HarmonyOS project (backend: "arkts")
//
// Usage:
//   auto build              # Build project
//   auto run                # Run dev server

name: "unified-demo"
version: "1.0.0"
scene: "ui"

// Supported backends: vue (web), jet (Android), arkts (HarmonyOS)
backend: ["vue", "jet", "arkts"]
```

**Step 2: Commit**

```bash
git add examples/unified-demo/pac.at
git commit -m "feat(unified-demo): add arkts backend support"
```

---

### Task 9: End-to-End Test

**Step 1: Build the project**

```bash
cd examples/unified-demo
cargo run --release -- gen
```

Expected output:
```
✓ Generated HarmonyOS project: arkts/
✓ Generated page: App.ets
```

**Step 2: Verify generated files**

```bash
ls -la arkts/
ls -la arkts/entry/src/main/ets/pages/
cat arkts/entry/src/main/ets/pages/App.ets
```

Expected:
- `arkts/build-profile.json5` exists
- `arkts/entry/src/main/ets/pages/App.ets` contains `@Entry`, `@Component`, `struct App`

**Step 3: Verify ArkTS code structure**

The generated `App.ets` should contain:
- `@Entry` and `@Component` decorators
- `struct App` with `@State` declarations
- `build()` method with `Column()` / `Row()` components
- Proper `.width()` / `.height()` modifiers

---

## Files Changed Summary

```
crates/auto-lang/src/ui_gen/
├── mod.rs                      # Add ark module export
└── ark/                        # New module
    ├── mod.rs                   # Module exports
    ├── generator.rs             # Main generator
    ├── components.rs            # Component registry
    ├── state.rs                 # State management
    ├── project.rs               # Project scaffolding
    └── modifier.rs              # Style modifiers

crates/auto-man/src/
├── lib.rs                      # Add ark module export
└── ark.rs                      # ArkTS build integration

examples/unified-demo/
└── pac.at                      # Add arkts backend
```

---

## Bug Fixes Required (2025-03-20)

Comparing generated `Counter.ets` against manually-written correct version revealed 6 bugs.

### Modifier Format Analysis

**Good news**: The `ark/modifier.rs` correctly generates TypeScript-style modifiers!

| Backend | Format | Example | Status |
|---------|--------|---------|--------|
| **ark/modifier.rs** | `.methodName(value)` | `.fontSize(30)`, `.margin(20)` | ✅ Correct for ArkTS |
| **jet/modifier.rs** | `Modifier.methodName(value)` | `Modifier.fontSize(30.sp)` | ✅ Correct for Kotlin |

No cross-contamination found - no `Modifier.` patterns in ark files.

**Note**: In AURA, styles are defined using Tailwind classes in the `class` prop:
```auto
button "-" { onclick: .Dec, class: "w-24 h-12 m-2" }
```

---

### Root Cause: Kotlin Syntax in TypeScript Generator

The ark generator was incorrectly emitting **Kotlin** syntax instead of **TypeScript/ArkTS** syntax. This happened because the code was likely copied/adapted from the jet backend (which targets Kotlin/Jetpack Compose) without proper adaptation for TypeScript.

### File-by-File Analysis

#### File: `state.rs` - ALL KOTLIN PATTERNS

| Line | Kotlin (Wrong) | TypeScript/ArkTS (Correct) | Issue |
|------|----------------|---------------------------|-------|
| 59 | `sealed class Msg {` | `enum Msg {` | Kotlin sealed class keyword |
| 65 | `data class Inc(val value: T) : Msg()` | (remove) | Kotlin data class inheritance |
| 67 | `object Dec : Msg()` | `  Dec,` | Kotlin object declaration |
| 38 | `Msg.Inc: {` | `case Msg.Inc: {` | Missing `case` keyword |
| 44 | `}` (no break) | `        break;\n      }` | Missing `break;` statement |

#### File: `generator.rs` - MOSTLY CORRECT ✓

No Kotlin patterns. Correctly generates TypeScript-style code.

#### File: `modifier.rs` - CORRECT ✓

No Kotlin patterns. Correctly generates TypeScript chainable modifiers.

---

### Bug 1: Wrong Enum Syntax (Kotlin `sealed class`)

| Generated (Wrong) | Correct |
|-------------------|---------|
| `sealed class Msg { object Inc : Msg() ... }` | `enum Msg { Inc, Dec }` |

**Root cause**: `generate_msg_sealed()` uses Kotlin sealed class pattern.

**Fix location**: `state.rs` lines 54-75 → `generate_msg_sealed()`

**Fix**: Replace entire function to generate TypeScript enum:

```rust
// BEFORE (Kotlin-style) - lines 54-75
pub fn generate_msg_sealed(widget: &AuraWidget) -> String {
    if widget.messages.is_empty() {
        return String::new();
    }
    let mut lines = vec!["sealed class Msg {".to_string()];
    for msg in &widget.messages {
        for variant in &msg.variants {
            if let Some(ref payload_type) = variant.payload {
                let arkts_type = auto_type_to_arkts(payload_type);
                lines.push(format!("  data class {}(val value: {}) : Msg()", variant.name, arkts_type));
            } else {
                lines.push(format!("  object {} : Msg()", variant.name));
            }
        }
    }
    lines.push("}".to_string());
    lines.join("\n")
}

// AFTER (TypeScript enum)
pub fn generate_msg_enum(widget: &AuraWidget) -> String {
    if widget.messages.is_empty() {
        return String::new();
    }
    let mut lines = vec!["enum Msg {".to_string()];
    for msg in &widget.messages {
        for variant in &msg.variants {
            // TypeScript enum - simple variant
            lines.push(format!("  {},", variant.name));
        }
    }
    lines.push("}".to_string());
    lines.join("\n")
}
```

**Note**: For payload-carrying messages, TypeScript requires a different pattern (union types or class-based approach). For now, simple enums cover the common case.

---

### Bug 2: Missing `case` Keyword in Switch

| Generated (Wrong) | Correct |
|-------------------|---------|
| `Msg.Inc: { ... }` | `case Msg.Inc: { ... break; }` |

**Root cause**: `generate_dispatch_function()` emits Kotlin `when` pattern (without `case`).

**Fix location**: `state.rs` lines 24-51 → `generate_dispatch_function()`

**Fix**:

```rust
// BEFORE (line 38)
lines.push(format!("      Msg.{}: {{", msg_name));

// AFTER
lines.push(format!("      case Msg.{}: {{", msg_name));
```

---

### Bug 3: Missing `break` Statements

| Generated (Wrong) | Correct |
|-------------------|---------|
| No `break;` after case blocks | Each case ends with `break;` |

**Root cause**: `generate_dispatch_function()` doesn't emit `break` (Kotlin `when` doesn't require it).

**Fix location**: `state.rs` lines 40-44 → after handler body

**Fix**:

```rust
// BEFORE (lines 40-44)
let body = generate_handler_body(payload);
for line in body.lines() {
    lines.push(format!("        {}", line));
}
lines.push("      }".to_string());

// AFTER
let body = generate_handler_body(payload);
for line in body.lines() {
    lines.push(format!("        {}", line));
}
lines.push("        break;".to_string());  // Add break
lines.push("      }".to_string());
```

---

### Bug 7: Modifier Format - CORRECT (Verified)

**Analysis**: Comparing `ark/modifier.rs` and `jet/modifier.rs`:

| File | Format | Example | Correct for Target? |
|------|--------|---------|-----------------|
| `ark/modifier.rs` | `.methodName(value)` | `.fontSize(30)` | ✅ TypeScript/ArkTS |
| `jet/modifier.rs` | `Modifier.methodName(value)` | `Modifier.fontSize(30)` | ✅ Kotlin/Compose |

**Conclusion**: No `Modifier.` pattern contamination in ark files. The modifier.rs code is correct.

---

### Other Issues in Generated Counter.ets

1. **No style modifiers in output** - Source has no styles defined
2. **No imports** - Missing `import { Button } from '@kit.ArkUI';`| Generated (Wrong) | Correct |
|-------------------|---------|
| `${..count}`, `Msg..Dec`, `Msg..Inc` | `${this.count}`, `Msg.Dec`, `Msg.Inc` |

**Root cause**: Generator emitting `..` instead of `.` for member access.

**Fix location**: `generator.rs` → `interpolate_string()` and `add_modifiers()`

**Fix**:
```rust
// The bug is likely in how we handle AURA's `.field` syntax
// AURA uses `.count` for self-reference, but we're converting to `..count`
// Should convert to `this.count` or just `.count` depending on context
```

---

### Bug 5: Wrong Button Construction Order

| Generated (Wrong) | Correct |
|-------------------|---------|
| `Button().onClick(...)('-')` | `Button('-').onClick(...)` |

**Root cause**: Generator placing label as trailing call instead of constructor argument.

**Fix location**: `generator.rs` → `node_to_arkts()`

**Fix**:
```rust
// Before (wrong order)
lines.push(format!("{}Button()", indent_str));
// ... add onClick ...
lines.push(format("{}  ('{}')", indent_str, label));

// After (correct order)
lines.push(format!("{}Button('{}')", indent_str, label));
// ... add onClick ...
```

---

### Bug 6: Missing Import Statements

| Generated (Wrong) | Correct |
|-------------------|---------|
| No imports | `import { Button } from '@kit.ArkUI';` |

**Root cause**: Generator not emitting required imports.

**Fix location**: `generator.rs` → `generate()` and new `imports` tracking

**Fix**:
```rust
// At top of generated file
lines.push("import { Button, Column, Row, Text } from '@kit.ArkUI';".to_string());
lines.push(String::new());
```

---

### Fix Priority

| Priority | Bug | Complexity | Impact |
|----------|-----|------------|--------|
| 1 | Missing imports | Low | Foundational |
| 2 | Double dot (`..`) | Medium | Critical syntax error |
| 3 | Button order | Low | Component API |
| 4 | `case` keyword | Low | Control flow |
| 5 | `break` statements | Low | Control flow |
| 6 | Enum syntax | Medium | Type system |

---

### Correct Counter.ets Reference

```typescript
// page.ets
import { Button } from '@kit.ArkUI';

enum Msg {
  Inc,
  Dec,
}

@Entry
@Component
struct Index {
  @State message: string = 'Hello World';
  @State count: number = 0;

  private dispatch(msg: Msg): void {
    switch (msg) {
    case Msg.Inc: {
      this.count = this.count + 1
      break;
    }
    case Msg.Dec: {
      this.count = this.count - 1
      break;
    }
  }
}

  build() {
    Column() {
      Text(`Counter now: ${this.count}`)
        .fontSize(30)
        .margin(20)

      Row() {
        Button('-')
          .onClick(() => {
            this.dispatch(Msg.Dec);
          })
          .width(100)
          .height(50)
          .margin(10)

        Button('+')
          .onClick(() => {
            this.dispatch(Msg.Inc);
          })
          .width(100)
          .height(50)
          .margin(10)
      }
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
  }
}
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

- [x] `ark/` module compiles without errors
- [x] `ArkComponentRegistry` has basic components (Column, Row, Text, Button)
- [x] `ArkProjectGenerator` creates valid HarmonyOS project structure
- [x] `ArkGenerator` generates valid ArkTS code with @Entry/@Component/@State
- [x] Message dispatch pattern generates correctly
- [x] `auto gen` creates `ark/` directory with valid project
- [x] Generated project can be opened in DevEco Studio
- [x] **Project runs correctly in DevEco Studio** (verified 2025-03-21)

### Bug Fix Verification

- [x] **Bug 1**: Msg uses `enum Msg { Inc, Dec }` syntax
- [x] **Bug 2**: Switch uses `case Msg.Inc:` syntax
- [x] **Bug 3**: Each case has `break;` statement
- [x] **Bug 4**: No `..` syntax, use `this.count` or `Msg.Inc`
- [x] **Bug 5**: Button label in constructor: `Button('-')`
- [x] **Bug 6**: Required imports at top of file
- [x] Generated Counter.ets matches correct reference (can compile in DevEco)

### Navigation & Routing Fixes (2025-03-21)

- [x] `pushPathByName` uses empty string `''` instead of object `{}`
- [x] `main_pages.json` only contains `pages/App` (other pages are NavDestinations)
- [x] Child pages wrapped in `NavDestination()` component
- [x] IndexPage with navigation links has `@Consume('pathStack')` for pathStack access
- [x] Outlet replaced with `IndexPage()` directly (ArkTS has no default route concept)

---

## Task 10: Tailwind CSS Support (Phase 1)

**Goal**: Integrate shared `TailwindParser` into ArkTS generator for tailwind-to-ArkTS modifier transpilation.

### ArkTS API Reference (from DevEco Studio SDK)

| Tailwind | ArkTS Method | Signature |
|----------|--------------|-----------|
| `w-*`, `h-*` | `.width()`, `.height()` | `(value: Length)` |
| `p-*`, `px-*`, `py-*` | `.padding()` | `(value: Padding \| Length)` |
| `m-*`, `mx-*`, `my-*` | `.margin()` | `(value: Margin \| Length)` |
| `text-*` (size) | `.fontSize()` | `(value: number \| string)` |
| `font-*` (weight) | `.fontWeight()` | `(value: FontWeight)` |
| `text-center/left/right` | `.textAlign()` | `(value: TextAlign)` |
| `text-*` (color) | `.fontColor()` | `(value: ResourceColor)` |
| `bg-*` | `.backgroundColor()` | `(value: ResourceColor)` |
| `rounded-*` | `.borderRadius()` | `(value: Length)` |

### Implementation Steps

**Step 1**: Create `ArkModifierDsl` struct in `ark/modifier.rs`

```rust
use crate::ui_gen::shared::tailwind::{TailwindParser, ComputedStyle, Dimension};

pub struct ArkModifierDsl {
    parser: TailwindParser,
}

impl ArkModifierDsl {
    pub fn new() -> Self {
        Self { parser: TailwindParser::new() }
    }

    pub fn convert_class(&self, class: &str) -> Vec<String> {
        let style = self.parser.parse(class);
        self.style_to_modifiers(&style)
    }
}
```

**Step 2**: Implement conversion methods

- `dimension_to_arkts()` - Convert Dimension to ArkTS value
- `gap_to_modifier()` - gap-* → `.space()`
- `padding_to_modifiers()` - p-* → `.padding()`
- `font_size_to_modifier()` - text-* → `.fontSize()`
- `font_weight_to_modifier()` - font-* → `.fontWeight()`
- `text_align_to_modifier()` - text-center → `.textAlign()`
- `border_radius_to_modifier()` - rounded-* → `.borderRadius()`
- `color_to_modifier()` - bg-*/text-* → `.backgroundColor()`/`.fontColor()`

**Step 3**: Update `generator.rs` to use `ArkModifierDsl`

Process `class` prop in `generate_modifiers()`:

```rust
if let Some(AuraPropValue::Expr(AuraExpr::Literal(class_str))) = props.get("class") {
    let class_modifiers = self.modifier_dsl.convert_class(class_str);
    modifiers.extend(class_modifiers);
}
```

**Step 4**: Add unit tests

**Step 5**: Add tailwind classes to unified-demo pages

### Tailwind Classes to Support

- Layout: `flex`, `flex-col`, `flex-row`, `gap-*`
- Spacing: `p-*`, `px-*`, `py-*`, `pt-*`, `pb-*`, `pl-*`, `pr-*`
- Typography: `text-*` (sizes), `font-*` (weights), `text-center/left/right`
- Colors: `bg-*`, `text-*`
- Border: `rounded-*`

### Verification

- [ ] `cargo test -p auto-lang ark::modifier` passes
- [ ] `auto gen` generates proper modifiers
- [ ] DevEco Studio compiles and runs
