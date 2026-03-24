# QuickStart Sprint A

## Design

## Objective

Improve the ArkTS generator by reimplementing the 12 QuickStart tutorial projects from `D:\gitee\QuickStart\` as Auto projects. This will drive feature development and ensure the generator supports real-world HarmonyOS development patterns.

## Source Material

- **Tutorials**: `D:\gitee\QuickStart\` - 12 tutorial projects
- **API Reference**: `D:\Huawei\DevEco Studio\sdk\default\openharmony\ets\component` - Full ArkTS component API

## Sprint Organization

### Sprint A: Basic UI (Tutorials 01-03)
**Goal**: Foundation components and basic rendering

| Tutorial | Topics | Key Features |
|----------|--------|--------------|
| 01-HelloWorld | Project structure, basic Text | Column, Text, styling |
| 02-Component | Custom components, @Component | Component definition, props |
| 03-Swiper | Swiper, Image, animation | Swiper component, Image loading |

**Success Criteria**:
- Generate compilable ArkTS from AURA
- Support Column, Text, Image, Swiper
- Custom component generation

### Sprint B: Data & Architecture (Tutorials 04-06)
**Goal**: Data handling and MVVM pattern

| Tutorial | Topics | Key Features |
|----------|--------|--------------|
| 04-Grid | Grid layout, data binding | Grid component, ForEach |
| 05-List | List rendering, lazy loading | List, ListItem, LazyForEach |
| 06-MVVM | State management, @State, @Observed | MVVM architecture, reactive updates |

**Success Criteria**:
- Grid and List components
- Data binding syntax
- State management primitives

### Sprint C: Navigation & State (Tutorials 07-09)
**Goal**: Navigation and dynamic UI

| Tutorial | Topics | Key Features |
|----------|--------|--------------|
| 07-WebView | WebView, JavaScript bridge | WebView integration |
| 08-DataDriven | Dynamic UI generation | Conditional rendering, data-driven |
| 09-Navigation | Navigation stack, routes | NavHost, NavDestination, pathStack |

**Success Criteria**:
- Navigation system
- WebView support
- Conditional rendering

### Sprint D: Advanced APIs (Tutorials 10-12)
**Goal**: Platform integration

| Tutorial | Topics | Key Features |
|----------|--------|--------------|
| 10-TTS | Text-to-speech, permissions | TTS API, permission handling |
| 11-MultiDevice | Responsive design, breakpoints | Multi-device layout |
| 12-Distributed | Distributed data, sync | Distributed capabilities |

**Success Criteria**:
- Platform API access
- Responsive design utilities
- Cross-device features

## Project Structure

```
examples/quickstart/
├── 01-HelloWorld/
�?  ├── aura/
�?  �?  ├── pac.at
�?  �?  └── pages/
�?  �?      └── Index.at
�?  └── ark/              # Generated ArkTS project
├── 02-Component/
�?  └── ...
├── ... (03-12)
└── README.md
```

## Implementation Approach

For each tutorial:

1. **Study the original**: Read tutorial ArkTS code, understand patterns
2. **Design AURA equivalent**: Map ArkTS patterns to AURA syntax
3. **Implement in AURA**: Write AURA widget definitions
4. **Generate and verify**: Run generator, check output compiles
5. **Fix generator**: Add missing features as needed
6. **Document**: Update component mappings

## Component Mapping Strategy

As we implement each tutorial, we'll extend the generator:

| ArkTS Pattern | AURA Equivalent | Generator Support |
|---------------|-----------------|-------------------|
| `Column() { }` | `col { }` | �?Done |
| `Text("text")` | `text (text: "...") {}` | �?Done |
| `@State var` | `state { }` block | Sprint B |
| `@Builder func` | `widget Name {}` | Sprint A |
| `NavHost` | `navigation { }` | Sprint C |
| `List { ForEach }` | `list { for-in }` | Sprint B |

## Timeline

- **Sprint A**: 3 tutorials, foundation work
- **Sprint B**: 3 tutorials, data layer
- **Sprint C**: 3 tutorials, navigation
- **Sprint D**: 3 tutorials, advanced APIs

Each sprint produces working demos and generator improvements.

## Success Metrics

1. All 12 tutorials compile and run
2. Generator supports all patterns used in tutorials
3. AURA syntax is clean and intuitive
4. Documentation updated with examples

## Implementation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement tutorials 01-03 from QuickStart as Auto projects to drive ArkTS generator improvements.

**Architecture:** Create AURA source files in `examples/quickstart/01-HelloWorld/`, `02-Banner/`, `03-Components/`. Extend generator to support Image, Swiper, custom components, and additional modifiers.

**Tech Stack:** Rust (ArkTS generator), AURA syntax, ArkTS/HarmonyOS

---

## Task 1: Add Swiper and Image Components to Registry

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/components.rs:75-80`

**Step 1: Add Swiper component to registry**

Add after the `image` registration:

```rust
self.register("swiper", ArkComponent {
    name: "Swiper".to_string(),
    has_children: true,
    has_content: false,
});
```

**Step 2: Run tests to verify**

Run: `cargo test -p auto-lang ark::components`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/components.rs
git commit -m "feat(ark): add Swiper component to registry"
```

---

## Task 2: Add Additional Modifiers to ArkModifierDsl

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/modifier.rs:45-113`

**Step 1: Add fontFamily modifier**

In `style_to_modifiers`, add after textAlign:

```rust
// Font family
if let Some(family) = &style.font_family {
    modifiers.push(format!(".fontFamily('{}')", family));
}
```

**Step 2: Add lineHeight modifier**

In `style_to_modifiers`, add after font_family:

```rust
// Line height
if let Some(height) = &style.line_height {
    modifiers.push(self.dimension_to_line_height(height));
}
```

Add helper method:

```rust
fn dimension_to_line_height(&self, dim: &Dimension) -> String {
    format!(".lineHeight({})", self.dimension_to_value(dim))
}
```

**Step 3: Add objectFit modifier**

For Image component, add objectFit support. Add to `style_to_modifiers`:

```rust
// Object fit (for images)
if let Some(fit) = &style.object_fit {
    modifiers.push(self.object_fit_to_modifier(fit));
}
```

Add helper method and enum:

```rust
fn object_fit_to_modifier(&self, fit: &ObjectFit) -> String {
    let ark_fit = match fit {
        ObjectFit::Contain => "ImageFit.Contain",
        ObjectFit::Cover => "ImageFit.Cover",
        ObjectFit::Fill => "ImageFit.Fill",
        ObjectFit::ScaleDown => "ImageFit.ScaleDown",
        ObjectFit::None => "ImageFit.None",
    };
    format!(".objectFit({})", ark_fit)
}
```

**Step 4: Add layoutWeight modifier**

Add to `style_to_modifiers`:

```rust
// Layout weight (for flex children)
if let Some(weight) = &style.layout_weight {
    modifiers.push(format!(".layoutWeight({})", weight));
}
```

**Step 5: Run tests**

Run: `cargo test -p auto-lang ark::modifier`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/modifier.rs
git commit -m "feat(ark): add fontFamily, lineHeight, objectFit, layoutWeight modifiers"
```

---

## Task 3: Extend TailwindParser for New Properties

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/shared/tailwind.rs`

**Step 1: Add new fields to ComputedStyle**

Add to ComputedStyle struct:

```rust
pub font_family: Option<String>,
pub line_height: Option<Dimension>,
pub object_fit: Option<ObjectFit>,
pub layout_weight: Option<u32>,
pub max_lines: Option<u32>,
pub text_overflow: Option<TextOverflow>,
```

**Step 2: Add ObjectFit enum**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFit {
    Contain,
    Cover,
    Fill,
    ScaleDown,
    None,
}
```

**Step 3: Add TextOverflow enum**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TextOverflow {
    Ellipsis,
    Clip,
}
```

**Step 4: Parse object-fit classes**

In `parse_single_class`, add:

```rust
"object-contain" => style.object_fit = Some(ObjectFit::Contain),
"object-cover" => style.object_fit = Some(ObjectFit::Cover),
"object-fill" => style.object_fit = Some(ObjectFit::Fill),
"object-scale-down" => style.object_fit = Some(ObjectFit::ScaleDown),
"object-none" => style.object_fit = Some(ObjectFit::None),
```

**Step 5: Parse line-height classes**

```rust
// leading-{n} for line-height
if let Some(val) = class.strip_prefix("leading-") {
    if let Ok(n) = val.parse::<f32>() {
        style.line_height = Some(Dimension::Dp(n));
    }
}
```

**Step 6: Run tests**

Run: `cargo test -p auto-lang shared::tailwind`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/auto-lang/src/ui_gen/shared/tailwind.rs
git commit -m "feat(tailwind): add objectFit, lineHeight, maxLines, textOverflow parsing"
```

---

## Task 4: Add Generator Support for Image Source

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs:450-520`

**Step 1: Handle Image src prop specially**

In `generate_element`, add special handling for Image src:

```rust
// Special handling for Image src prop
let content_arg = if component.has_content {
    if let Some(AuraPropValue::Expr(AuraExpr::Literal(text))) = props.get("text") {
        format!("'{}'", text)
    } else {
        String::new()
    }
} else if tag == "image" {
    // Image component takes src as argument
    if let Some(AuraPropValue::Expr(AuraExpr::Literal(src))) = props.get("src") {
        // Check if it's a resource reference like $r('app.media.xxx')
        if src.starts_with("$r(") {
            src.clone()
        } else {
            format!("'{}'", src)
        }
    } else {
        String::new()
    }
} else {
    String::new()
};
```

**Step 2: Update component call for Image**

```rust
let component_call = if content_arg.is_empty() {
    format!("{}()", component.name)
} else {
    format!("{}({})", component.name, content_arg)
};
```

**Step 3: Run tests**

Run: `cargo test -p auto-lang ark::generator`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(ark): handle Image src prop as component argument"
```

---

## Task 5: Create 01-HelloWorld Tutorial Project

**Files:**
- Create: `examples/quickstart/01-HelloWorld/aura/pac.at`
- Create: `examples/quickstart/01-HelloWorld/aura/pages/Index.at`

**Step 1: Create project directory**

Run: `mkdir -p examples/quickstart/01-HelloWorld/aura/pages`

**Step 2: Create pac.at**

```auto
// 01-HelloWorld - Basic HarmonyOS app
name: "HelloWorld"
platform: "ark"
src: ["aura"]
```

**Step 3: Create Index.at**

```auto
// pages/Index.at - Main page with Banner component

/// Main entry page
widget Index {
    state {
        message: "快速入�?
    }

    view {
        col (class: "w-full h-full bg-gray-100") {
            text (class: "text-2xl font-bold w-full text-left pl-4", text: message) {}
            Banner {}
        }
    }
}

/// Banner component displaying an image
widget Banner {
    view {
        image (
            class: "w-full pt-3 px-4 rounded-2xl",
            src: "$r('app.media.banner_pic1')",
            fit: "contain"
        ) {}
    }
}
```

**Step 4: Generate ArkTS code**

Run: `cargo run -- ark examples/quickstart/01-HelloWorld/aura`

**Step 5: Verify output**

Check that generated files in `examples/quickstart/01-HelloWorld/ark/` compile.

**Step 6: Commit**

```bash
git add examples/quickstart/01-HelloWorld/
git commit -m "feat(quickstart): add 01-HelloWorld tutorial"
```

---

## Task 6: Add ForEach Support for Swiper

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs:667-699`

**Step 1: Update ForEach generation to support key function**

Current ForEach in ArkTS requires a key generator function. Update `generate_for_loop`:

```rust
fn generate_for_loop(
    &mut self,
    var: &str,
    index: Option<&str>,
    iterable: &str,
    body: &[AuraNode],
) -> GenResult<String> {
    let mut lines = Vec::new();

    let index_param = index.map(|i| format!(", {}: number", i)).unwrap_or_default();
    // Generate ForEach with key function (item, index) => item.id
    lines.push(format!(
        "{}ForEach(this.{}, ({}: any{}) => {{",
        self.indent(),
        iterable,
        var,
        index_param
    ));
    self.indent_level += 1;

    for child in body {
        let child_code = self.generate_node(child)?;
        for line in child_code.lines() {
            lines.push(format!("{}{}", self.indent(), line));
        }
    }

    self.indent_level -= 1;
    // Add key function - default to index-based key
    lines.push(format!("{}{}}, ({}: any, {}: number) => {}", self.indent(), "}", var, index.unwrap_or("index"), var));

    Ok(lines.join("\n"))
}
```

**Step 2: Run tests**

Run: `cargo test -p auto-lang ark::generator::tests`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "fix(ark): add key function to ForEach generation"
```

---

## Task 7: Create 02-Banner Tutorial Project

**Files:**
- Create: `examples/quickstart/02-Banner/aura/pac.at`
- Create: `examples/quickstart/02-Banner/aura/pages/Index.at`

**Step 1: Create project directory**

Run: `mkdir -p examples/quickstart/02-Banner/aura/pages`

**Step 2: Create pac.at**

```auto
// 02-Banner - Swiper banner with auto-play
name: "Banner"
platform: "ark"
src: ["aura"]
```

**Step 3: Create Index.at with Swiper**

```auto
// pages/Index.at - Swiper banner demo

/// Main page with auto-playing banner
widget Index {
    state {
        message: "快速入�?
        bannerList: [
            { id: "pic0", src: "$r('app.media.banner_pic0')" },
            { id: "pic1", src: "$r('app.media.banner_pic1')" },
            { id: "pic2", src: "$r('app.media.banner_pic2')" },
        ]
    }

    view {
        col (class: "w-full h-full bg-gray-100") {
            text (class: "text-2xl font-bold w-full text-left pl-4", text: message) {}
            Banner { items: bannerList }
        }
    }
}

/// Swiper banner component
widget Banner {
    props {
        items: Array
    }

    view {
        swiper (class: "auto-play loop") {
            for item in items {
                image (
                    class: "w-full pt-3 px-4 rounded-2xl object-contain",
                    src: item.src
                ) {}
            }
        }
    }
}
```

**Step 4: Generate and verify**

Run: `cargo run -- ark examples/quickstart/02-Banner/aura`

**Step 5: Commit**

```bash
git add examples/quickstart/02-Banner/
git commit -m "feat(quickstart): add 02-Banner tutorial with Swiper"
```

---

## Task 8: Add Swiper Modifiers (autoPlay, loop, indicator)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/modifier.rs`

**Step 1: Add Swiper-specific props**

In `prop_to_modifier`, add:

```rust
// Swiper modifiers
"autoPlay" | "autoplay" => Some(".autoPlay(true)".to_string()),
"loop" => Some(".loop(true)".to_string()),
"indicator" => Some(".indicator(DotIndicator())".to_string()),
```

**Step 2: Handle class-based swiper modifiers**

In `style_to_modifiers`, add:

```rust
// Swiper modifiers from class
if class.contains("auto-play") || class.contains("autoplay") {
    modifiers.push(".autoPlay(true)".to_string());
}
if class.contains("loop") {
    modifiers.push(".loop(true)".to_string());
}
```

**Step 3: Run tests**

Run: `cargo test -p auto-lang ark::modifier`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/modifier.rs
git commit -m "feat(ark): add Swiper modifiers (autoPlay, loop, indicator)"
```

---

## Task 9: Create 03-Components Tutorial Project

**Files:**
- Create: `examples/quickstart/03-Components/aura/pac.at`
- Create: `examples/quickstart/03-Components/aura/pages/Index.at`
- Create: `examples/quickstart/03-Components/aura/widgets/TutorialItem.at`
- Create: `examples/quickstart/03-Components/aura/widgets/EnablementItem.at`

**Step 1: Create project structure**

Run: `mkdir -p examples/quickstart/03-Components/aura/{pages,widgets}`

**Step 2: Create pac.at**

```auto
// 03-Components - Custom components demo
name: "Components"
platform: "ark"
src: ["aura"]
```

**Step 3: Create TutorialItem widget**

```auto
// widgets/TutorialItem.at - Tutorial list item

/// Tutorial item component
widget TutorialItem {
    props {
        title: String
        brief: String
        imageSrc: String
    }

    view {
        row (class: "w-full h-22 rounded-2xl bg-white p-3 items-start") {
            col (class: "h-full flex-1 items-start mr-3") {
                text (
                    class: "h-5 w-full text-sm text-left truncate",
                    text: title
                ) {}
                text (
                    class: "h-8 w-full text-xs text-left text-gray-600 line-clamp-2",
                    text: brief
                ) {}
            }
            image (
                class: "h-16 w-28 rounded-2xl object-cover",
                src: imageSrc
            ) {}
        }
    }
}
```

**Step 4: Create EnablementItem widget**

```auto
// widgets/EnablementItem.at - Card-style item

/// Enablement card component
widget EnablementItem {
    props {
        title: String
        brief: String
        imageSrc: String
    }

    view {
        col (class: "w-40 h-42 rounded-2xl bg-white") {
            image (
                class: "w-full h-24 object-cover rounded-t-2xl",
                src: imageSrc
            ) {}
            text (
                class: "h-5 w-full text-sm text-left px-3 mt-2",
                text: title
            ) {}
            text (
                class: "h-8 w-full text-xs text-left text-gray-600 px-3 mt-1",
                text: brief
            ) {}
        }
    }
}
```

**Step 5: Create Index page**

```auto
// pages/Index.at - Components gallery

use TutorialItem
use EnablementItem

/// Main page showcasing custom components
widget Index {
    view {
        col (class: "w-full h-full bg-gray-100 p-4 gap-4") {
            TutorialItem {
                title: "Step1 快速入门介�?,
                brief: "本篇教程实现了快速入门——一个用于了解和学习HarmonyOS的应用程序�?,
                imageSrc: "$r('app.media.enablement_pic1')"
            }
            EnablementItem {
                title: "HarmonyOS第一�?,
                brief: "基于真实的开发场景，提供向导式学习�?,
                imageSrc: "$r('app.media.enablement_pic1')"
            }
        }
    }
}
```

**Step 6: Generate and verify**

Run: `cargo run -- ark examples/quickstart/03-Components/aura`

**Step 7: Commit**

```bash
git add examples/quickstart/03-Components/
git commit -m "feat(quickstart): add 03-Components tutorial with custom widgets"
```

---

## Task 10: Add @Preview Decorator Support

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs:140-195`

**Step 1: Add @Preview decorator to non-entry components**

In `generate_entry_component`, add @Preview for child components:

```rust
// Add @Preview for non-entry components (helpful for DevEco Studio preview)
if !has_routes {
    lines.push("@Preview".to_string());
}
lines.push("@Component".to_string());
```

**Step 2: Run tests**

Run: `cargo test -p auto-lang ark::generator`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(ark): add @Preview decorator for child components"
```

---

## Task 11: Add borderRadius Corner-Specific Support

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/modifier.rs:219-225`

**Step 1: Add corner-specific borderRadius**

In `dimension_to_border_radius`, handle object with corners:

```rust
fn dimension_to_border_radius(&self, dim: &Dimension) -> String {
    match dim {
        Dimension::Dp(v) if *v >= 9999.0 => ".borderRadius('50%')".to_string(),
        _ => format!(".borderRadius({})", self.dimension_to_value(dim)),
    }
}

/// Generate corner-specific border radius
fn border_radius_to_modifier(&self, radius: &BorderRadius) -> String {
    match radius {
        BorderRadius::All(v) => format!(".borderRadius({})", self.dimension_to_value(v)),
        BorderRadius::Corners { top_left, top_right, bottom_left, bottom_right } => {
            format!(
                ".borderRadius({{ topLeft: {}, topRight: {}, bottomLeft: {}, bottomRight: {} }})",
                self.dimension_to_value(top_left),
                self.dimension_to_value(top_right),
                self.dimension_to_value(bottom_left),
                self.dimension_to_value(bottom_right)
            )
        }
    }
}
```

**Step 2: Add BorderRadius enum to tailwind.rs**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BorderRadius {
    All(Dimension),
    Corners {
        top_left: Dimension,
        top_right: Dimension,
        bottom_left: Dimension,
        bottom_right: Dimension,
    },
}
```

**Step 3: Run tests**

Run: `cargo test -p auto-lang ark::modifier`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/modifier.rs crates/auto-lang/src/ui_gen/shared/tailwind.rs
git commit -m "feat(ark): add corner-specific borderRadius support"
```

---

## Task 12: Final Verification and Documentation

**Files:**
- Create: `examples/quickstart/README.md`

**Step 1: Run all generator tests**

Run: `cargo test -p auto-lang ark`
Expected: All tests pass

**Step 2: Generate all tutorial projects**

```bash
cargo run -- ark examples/quickstart/01-HelloWorld/aura
cargo run -- ark examples/quickstart/02-Banner/aura
cargo run -- ark examples/quickstart/03-Components/aura
```

**Step 3: Create README**

```markdown
# QuickStart Tutorials

HarmonyOS QuickStart tutorials reimplemented in Auto (AURA syntax).

## Tutorials

| # | Name | Topics |
|---|------|--------|
| 01 | HelloWorld | Column, Text, Image, custom components |
| 02 | Banner | Swiper, ForEach, auto-play |
| 03 | Components | Custom widgets, props, Row, layoutWeight |

## Running

```bash
cargo run -- ark examples/quickstart/01-HelloWorld/aura
```

Output appears in `examples/quickstart/01-HelloWorld/ark/`.
```

**Step 4: Commit**

```bash
git add examples/quickstart/README.md
git commit -m "docs(quickstart): add README for Sprint A tutorials"
```

---

## Summary

| Task | Description | Status |
|------|-------------|--------|
| 1 | Add Swiper to component registry | Pending |
| 2 | Add new modifiers (fontFamily, lineHeight, etc.) | Pending |
| 3 | Extend TailwindParser | Pending |
| 4 | Handle Image src prop | Pending |
| 5 | Create 01-HelloWorld | Pending |
| 6 | Fix ForEach key function | Pending |
| 7 | Create 02-Banner | Pending |
| 8 | Add Swiper modifiers | Pending |
| 9 | Create 03-Components | Pending |
| 10 | Add @Preview decorator | Pending |
| 11 | Add corner-specific borderRadius | Pending |
| 12 | Final verification | Pending |
