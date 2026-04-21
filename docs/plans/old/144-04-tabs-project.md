# 04-Tabs Project

## Design

## Overview

Create `04-Tabs` project to demonstrate bottom tab navigation with 3 tabs, using the existing stdlib Tabs pattern and translating to ArkTS `Tabs` component.

## Project Structure

```
examples/quickstart/04-Tabs/
├── pac.at                    # Project config
├── app.at                    # Main app with Tabs
├── Index.at                  # Entry point
�?├── # Tab 1 - QuickStart
├── QuickStartPage.at         # Simple wrapper
├── Banner.at
├── EnablementView.at
├── EnablementItem.at
├── TutorialView.at
├── TutorialItem.at
�?├── # Tab 2 - CourseLearning
├── CourseLearning.at         # Placeholder
�?├── # Tab 3 - KnowledgeMap
├── KnowledgeMap.at           # Simplified static
└── NavBarItem.at
```

## AutoLang Syntax (App.at)

Using stdlib pattern with ID shorthand:

```auto
use Tabs, TabsList, TabsTrigger, TabsContent
use QuickStartPage, CourseLearning, KnowledgeMap

widget App {
    model {
        activeTab: str = "quickstart"
    }

    view {
        Tabs (activeTab: .activeTab) {
            TabsList {
                TabsTrigger quickstart (label: "快速入�?, active: .activeTab == "quickstart") {}
                TabsTrigger learning (label: "课程学习", active: .activeTab == "learning") {}
                TabsTrigger map (label: "知识地图", active: .activeTab == "map") {}
            }
            TabsContent quickstart (active: .activeTab == "quickstart") {
                QuickStartPage {}
            }
            TabsContent learning (active: .activeTab == "learning") {
                CourseLearning {}
            }
            TabsContent map (active: .activeTab == "map") {
                KnowledgeMap {}
            }
        }
    }
}
```

### ID Shorthand

`TabsTrigger quickstart` and `TabsContent quickstart` imply `id: "quickstart"`.

## Generated ArkTS Output

```typescript
@Component
struct App {
  @State activeTab: string = 'quickstart'
  @State currentIndex: number = 0
  private tabsController: TabsController = new TabsController()

  @Builder
  tabBarBuilder(title: string, targetIndex: number) {
    Column() {
      Text(title)
        .fontFamily('HarmonyHeiTi-Medium')
        .fontSize(10)
        .fontColor(this.currentIndex === targetIndex ? '#0A59F7' : 'rgba(0,0,0,0.60)')
        .fontWeight(500)
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
    .onClick(() => {
      this.currentIndex = targetIndex
      this.tabsController.changeIndex(targetIndex)
    })
  }

  build() {
    Tabs({ barPosition: BarPosition.End, controller: this.tabsController }) {
      TabContent() {
        QuickStartPage()
      }
      .tabBar(this.tabBarBuilder('快速入�?, 0))

      TabContent() {
        CourseLearning()
      }
      .tabBar(this.tabBarBuilder('课程学习', 1))

      TabContent() {
        KnowledgeMap()
      }
      .tabBar(this.tabBarBuilder('知识地图', 2))
    }
    .vertical(false)
    .scrollable(false)
    .backgroundColor('#F1F3F5')
  }
}
```

## Key Transformations

| AutoLang | ArkTS |
|----------|-------|
| `Tabs` | `Tabs({ barPosition: BarPosition.End, controller })` |
| `TabsList` + `TabsTrigger` | `@Builder tabBarBuilder()` function |
| `TabsContent` | `TabContent().tabBar(tabBarBuilder(...))` |
| `activeTab: str` | `currentIndex: number` + `TabsController` |

## Page Components

### QuickStartPage.at

Simple wrapper around 03-ItemList content (no Navigation for now):

```auto
use Banner, EnablementView, TutorialView

widget QuickStartPage {
    view {
        Col {
            Text "快速入�? {
                style: "text-2xl font-bold w-full text-left pl-4"
            }
            Scroll {
                Col {
                    Banner {}
                    EnablementView {}
                    TutorialView {}
                }
                style: "flex-1"
            }
            style: "w-full h-full bg-gray-100"
        }
    }
}
```

### CourseLearning.at (Placeholder)

```auto
widget CourseLearning {
    view {
        Col {
            Text "课程学习" {
                style: "text-xl text-center"
            }
            Text "Coming Soon" {
                style: "text-gray-500"
            }
            style: "w-full h-full justify-center items-center"
        }
    }
}
```

### KnowledgeMap.at (Simplified)

```auto
widget KnowledgeMap {
    model {
        navBarItems: List = [
            { order: "01", title: "准备与学�? },
            { order: "02", title: "构建应用" },
            { order: "03", title: "应用测试" },
            { order: "04", title: "上架" },
            { order: "05", title: "运营增长" },
            { order: "06", title: "商业变现" },
            { order: "07", title: "更多" }
        ]
    }

    view {
        Scroll {
            Col {
                Text "知识地图" {
                    style: "text-2xl font-bold w-full"
                }
                Image "$r('app.media.knowledge_map_banner')" {
                    style: "w-full rounded-2xl mt-4 mb-2"
                }
                Text "通过循序渐进的学习路�?.." {
                    style: "text-sm text-gray-600"
                }
                List {
                    for item in .navBarItems {
                        ListItem {
                            NavBarItem { order: item.order, title: item.title }
                        }
                    }
                    style: "w-full mt-6"
                }
            }
            style: "p-3 bg-gray-100"
        }
    }
}
```

## Generator Changes

**File:** `crates/auto-lang/src/ui_gen/ark/generator.rs`

### Changes

1. **Detect Tabs pattern** - When `Tabs` contains `TabsList` + `TabsContent` children
2. **Extract tab data** - Collect `TabsTrigger` (id, label, icon) from `TabsList`
3. **Generate `@Builder tabBarBuilder`** - Create the tab bar builder function
4. **Transform `TabsContent`** - Generate `TabContent().tabBar(tabBarBuilder(...))`
5. **Add state management** - `currentIndex: number`, `TabsController`

### Component Mappings

```
Tabs        �?Tabs (with controller)
TabsList    �?(absorbed into @Builder)
TabsTrigger �?(absorbed into @Builder)
TabsContent �?TabContent
```

## Implementation Phases

### Phase 1: Tabs Generator Support
1. Add `Tabs`, `TabContent` component handling in generator.rs
2. Implement `TabsList` + `TabsTrigger` �?`@Builder` transformation
3. Map `TabsContent` �?`TabContent().tabBar()`
4. Add `TabsController` and `currentIndex` state generation

### Phase 2: Create 04-Tabs Project
1. Copy 03-ItemList components
2. Create `QuickStartPage.at`
3. Create `CourseLearning.at` placeholder
4. Create `KnowledgeMap.at` + `NavBarItem.at`
5. Create `App.at` with Tabs structure
6. Update `Index.at` to render App

### Phase 3: Testing
1. Add a2ark test case for Tabs pattern
2. Generate and verify ArkTS output

## Deferred Features

- **Navigation/Stack routing** - Deferred to future project (05-Nav)
- **WebView component** - CourseLearning uses placeholder
- **KnowledgeMap nested navigation** - Simplified static content only

## Implementation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Tabs component support to a2ark generator and create 04-Tabs demo project with 3 bottom tabs.

**Architecture:** Transform stdlib `Tabs` pattern (`TabsList` + `TabsTrigger` + `TabsContent`) to ArkTS `Tabs` with `@Builder` for tab bar. Use `TabsController` for state management.

**Tech Stack:** Rust, ArkTS, a2ark generator

---

## Task 1: Add Tabs Component Detection in Generator

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Add is_tabs_pattern helper function**

Add after the `is_grid_element` function (around line 200):

```rust
/// Check if element is a Tabs container with TabsList + TabsContent children
fn is_tabs_pattern(node: &AuraNode) -> bool {
    match node {
        AuraNode::Element { tag, children, .. } => {
            tag == "tabs" && children.iter().any(|c| {
                matches!(c, AuraNode::Element { tag, .. } if tag == "tabslist" || tag == "tabscontent")
            })
        }
        _ => false,
    }
}
```

**Step 2: Run tests to verify compilation**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): add is_tabs_pattern helper"
```

---

## Task 2: Add extract_tabs_data Function

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Add TabItem struct and extract function**

Add after `is_tabs_pattern`:

```rust
/// Extracted tab item data for @Builder generation
#[derive(Debug, Clone)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    pub icon_on: Option<String>,
    pub icon_off: Option<String>,
}

/// Extract tab triggers from TabsList
fn extract_tab_triggers(tabs_list: &AuraNode) -> Vec<TabItem> {
    let mut items = Vec::new();

    if let AuraNode::Element { children, .. } = tabs_list {
        for child in children {
            if let AuraNode::Element { tag, props, .. } = child {
                if tag == "tabstrigger" {
                    let id = props.get("id").cloned().unwrap_or_default();
                    let label = props.get("label").cloned().unwrap_or_default();
                    items.push(TabItem {
                        id,
                        label,
                        icon_on: props.get("iconOn").cloned(),
                        icon_off: props.get("iconOff").cloned(),
                    });
                }
            }
        }
    }

    items
}
```

**Step 2: Run tests**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): add TabItem struct and extract_tab_triggers"
```

---

## Task 3: Add generate_tabs_builder Function

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Add @Builder generation function**

Add after `extract_tab_triggers`:

```rust
/// Generate @Builder function for tab bar
fn generate_tabs_builder(&self, tab_items: &[TabItem]) -> String {
    let mut lines = Vec::new();

    lines.push("  @Builder".to_string());
    lines.push("  tabBarBuilder(title: string, targetIndex: number, selectedIcon: Resource, unselectIcon: Resource) {".to_string());
    lines.push("    Column() {".to_string());
    lines.push("      Image(this.currentIndex === targetIndex ? selectedIcon : unselectIcon)".to_string());
    lines.push("        .width(24)".to_string());
    lines.push("        .height(24)".to_string());
    lines.push("      Text(title)".to_string());
    lines.push("        .fontFamily('HarmonyHeiTi-Medium')".to_string());
    lines.push("        .fontSize(10)".to_string());
    lines.push("        .fontColor(this.currentIndex === targetIndex ? '#0A59F7' : 'rgba(0,0,0,0.60)')".to_string());
    lines.push("        .textAlign(TextAlign.Center)".to_string());
    lines.push("        .lineHeight(14)".to_string());
    lines.push("        .fontWeight(500)".to_string());
    lines.push("    }".to_string());
    lines.push("    .width('100%')".to_string());
    lines.push("    .height('100%')".to_string());
    lines.push("    .justifyContent(FlexAlign.Center)".to_string());
    lines.push("    .alignItems(HorizontalAlign.Center)".to_string());
    lines.push("    .onClick(() => {".to_string());
    lines.push("      this.currentIndex = targetIndex".to_string());
    lines.push("      this.tabsController.changeIndex(targetIndex)".to_string());
    lines.push("    })".to_string());
    lines.push("  }".to_string());

    lines.join("\n")
}
```

**Step 2: Run tests**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): add generate_tabs_builder for @Builder"
```

---

## Task 4: Add generate_tabs_component Function

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Add main Tabs generation function**

Add after `generate_tabs_builder`:

```rust
/// Generate Tabs component with TabContent children
fn generate_tabs_component(&self, node: &AuraNode, tab_items: &[TabItem]) -> String {
    let mut lines = Vec::new();

    // Tabs header
    lines.push("    Tabs({ barPosition: BarPosition.End, controller: this.tabsController }) {".to_string());

    // Generate TabContent for each TabsContent child
    if let AuraNode::Element { children, .. } = node {
        let mut content_index = 0;
        for child in children {
            if let AuraNode::Element { tag, props, children: content_children, .. } = child {
                if tag == "tabscontent" {
                    let tab_id = props.get("id").cloned().unwrap_or_default();

                    // Find matching tab item for icon resources
                    let tab_item = tab_items.iter().find(|t| t.id == tab_id);

                    lines.push(format!("      TabContent() {{"));

                    // Generate child content
                    for content_child in content_children {
                        let child_code = self.generate_node(content_child, 3);
                        lines.push(child_code);
                    }

                    lines.push("      }".to_string());

                    // Add tabBar with builder call
                    if let Some(item) = tab_item {
                        let icon_on = item.icon_on.as_deref().unwrap_or("$r('app.media.ic_default_on')");
                        let icon_off = item.icon_off.as_deref().unwrap_or("$r('app.media.ic_default_off')");
                        lines.push(format!("      .tabBar(this.tabBarBuilder('{}', {}, {}, {}))",
                            item.label, content_index, icon_on, icon_off));
                    }

                    content_index += 1;
                }
            }
        }
    }

    lines.push("    }".to_string());

    // Add Tabs modifiers
    lines.push("    .vertical(false)".to_string());
    lines.push("    .scrollable(false)".to_string());
    lines.push("    .backgroundColor('#F1F3F5')".to_string());

    lines.join("\n")
}
```

**Step 2: Run tests**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): add generate_tabs_component"
```

---

## Task 5: Integrate Tabs into generate_node

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Add Tabs case in generate_node**

Find the `generate_node` function and add this case in the element tag match (around line 500):

```rust
// Check for Tabs pattern before regular element handling
if self.is_tabs_pattern(node) {
    // Extract TabsList children
    let tabs_list = node.children.iter().find(|c| {
        matches!(c, AuraNode::Element { tag, .. } if tag == "tabslist")
    });

    let tab_items = if let Some(list) = tabs_list {
        self.extract_tab_triggers(list)
    } else {
        Vec::new()
    };

    return self.generate_tabs_component(node, &tab_items);
}
```

**Step 2: Run tests**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): integrate Tabs pattern into generate_node"
```

---

## Task 6: Add Tabs State Variables

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Add state generation for Tabs in generate_widget**

Find the state variable generation section and add detection for Tabs:

```rust
// Check if widget contains Tabs - add controller and index state
let has_tabs = self.widget_has_tabs(&widget.view_tree);
if has_tabs {
    lines.push(format!("{}@State currentIndex: number = 0", self.indent()));
    lines.push(format!("{}private tabsController: TabsController = new TabsController()", self.indent()));
}
```

**Step 2: Add widget_has_tabs helper**

```rust
/// Check if widget view tree contains Tabs component
fn widget_has_tabs(&self, node: &AuraNode) -> bool {
    match node {
        AuraNode::Element { tag, children, .. } => {
            if tag == "tabs" {
                return true;
            }
            children.iter().any(|c| self.widget_has_tabs(c))
        }
        _ => false,
    }
}
```

**Step 3: Run tests**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): add TabsController and currentIndex state"
```

---

## Task 7: Add a2ark Test Case for Tabs

**Files:**
- Create: `crates/auto-lang/test/a2ark/015_tabs/input.at`
- Create: `crates/auto-lang/test/a2ark/015_tabs/input.expected.ets`

**Step 1: Create test input file**

Create `crates/auto-lang/test/a2ark/015_tabs/input.at`:

```auto
// Test Tabs pattern
widget TestTabs {
    model {
        activeTab: str = "tab1"
    }

    view {
        Tabs (activeTab: .activeTab) {
            TabsList {
                TabsTrigger tab1 (label: "Tab 1", active: .activeTab == "tab1") {}
                TabsTrigger tab2 (label: "Tab 2", active: .activeTab == "tab2") {}
            }
            TabsContent tab1 (active: .activeTab == "tab1") {
                Text "Content 1"
            }
            TabsContent tab2 (active: .activeTab == "tab2") {
                Text "Content 2"
            }
        }
    }
}
```

**Step 2: Create expected output file**

Create `crates/auto-lang/test/a2ark/015_tabs/input.expected.ets`:

```typescript
import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestTabs {
  @State activeTab: string = 'tab1'
  @State currentIndex: number = 0
  private tabsController: TabsController = new TabsController()

  @Builder
  tabBarBuilder(title: string, targetIndex: number) {
    Column() {
      Text(title)
        .fontFamily('HarmonyHeiTi-Medium')
        .fontSize(10)
        .fontColor(this.currentIndex === targetIndex ? '#0A59F7' : 'rgba(0,0,0,0.60)')
        .textAlign(TextAlign.Center)
        .lineHeight(14)
        .fontWeight(500)
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
    .alignItems(HorizontalAlign.Center)
    .onClick(() => {
      this.currentIndex = targetIndex
      this.tabsController.changeIndex(targetIndex)
    })
  }

  build() {
    Tabs({ barPosition: BarPosition.End, controller: this.tabsController }) {
      TabContent() {
        Text('Content 1')
      }
      .tabBar(this.tabBarBuilder('Tab 1', 0))
      TabContent() {
        Text('Content 2')
      }
      .tabBar(this.tabBarBuilder('Tab 2', 1))
    }
    .vertical(false)
    .scrollable(false)
    .backgroundColor('#F1F3F5')
  }
}
```

**Step 3: Add test function in generator.rs**

Add to the tests module:

```rust
#[test]
fn test_015_tabs() {
    test_a2ark("015_tabs").unwrap();
}
```

**Step 4: Run test**

Run: `cargo test -p auto-lang test_015_tabs`
Expected: Test passes

**Step 5: Commit**

```bash
git add crates/auto-lang/test/a2ark/015_tabs/
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "test(a2ark): add Tabs pattern test case"
```

---

## Task 8: Create 04-Tabs Project Structure

**Files:**
- Create: `examples/quickstart/04-Tabs/pac.at`
- Create: `examples/quickstart/04-Tabs/Index.at`

**Step 1: Create pac.at**

```auto
name: "04-Tabs"
src: ["."]
```

**Step 2: Create Index.at**

```auto
use App

widget Index {
    view {
        App {}
    }
}
```

**Step 3: Commit**

```bash
git add examples/quickstart/04-Tabs/
git commit -m "feat(example): create 04-Tabs project structure"
```

---

## Task 9: Copy 03-ItemList Components

**Files:**
- Copy: `examples/quickstart/03-ItemList/Banner.at` �?`examples/quickstart/04-Tabs/`
- Copy: `examples/quickstart/03-ItemList/EnablementView.at` �?`examples/quickstart/04-Tabs/`
- Copy: `examples/quickstart/03-ItemList/EnablementItem.at` �?`examples/quickstart/04-Tabs/`
- Copy: `examples/quickstart/03-ItemList/TutorialView.at` �?`examples/quickstart/04-Tabs/`
- Copy: `examples/quickstart/03-ItemList/TutorialItem.at` �?`examples/quickstart/04-Tabs/`

**Step 1: Copy files**

```bash
cp examples/quickstart/03-ItemList/Banner.at examples/quickstart/04-Tabs/
cp examples/quickstart/03-ItemList/EnablementView.at examples/quickstart/04-Tabs/
cp examples/quickstart/03-ItemList/EnablementItem.at examples/quickstart/04-Tabs/
cp examples/quickstart/03-ItemList/TutorialView.at examples/quickstart/04-Tabs/
cp examples/quickstart/03-ItemList/TutorialItem.at examples/quickstart/04-Tabs/
```

**Step 2: Commit**

```bash
git add examples/quickstart/04-Tabs/
git commit -m "feat(example): copy 03-ItemList components to 04-Tabs"
```

---

## Task 10: Create QuickStartPage.at

**Files:**
- Create: `examples/quickstart/04-Tabs/QuickStartPage.at`

**Step 1: Create QuickStartPage.at**

```auto
// QuickStartPage.at - Tab 1 content

use Banner
use EnablementView
use TutorialView

/// Quick start page with Banner, Grid, and List
widget QuickStartPage {
    view {
        Col {
            Text "快速入�? {
                style: "text-2xl font-bold w-full text-left pl-4"
            }
            Scroll {
                Col {
                    Banner {}
                    EnablementView {}
                    TutorialView {}
                }
                style: "flex-1"
            }
            style: "w-full h-full bg-gray-100"
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/quickstart/04-Tabs/QuickStartPage.at
git commit -m "feat(example): add QuickStartPage component"
```

---

## Task 11: Create CourseLearning.at Placeholder

**Files:**
- Create: `examples/quickstart/04-Tabs/CourseLearning.at`

**Step 1: Create CourseLearning.at**

```auto
// CourseLearning.at - Tab 2 placeholder

/// Course learning placeholder (WebView to be added later)
widget CourseLearning {
    view {
        Col {
            Text "课程学习" {
                style: "text-xl font-bold"
            }
            Text "Coming Soon" {
                style: "text-gray-500 mt-2"
            }
            style: "w-full h-full justify-center items-center"
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/quickstart/04-Tabs/CourseLearning.at
git commit -m "feat(example): add CourseLearning placeholder"
```

---

## Task 12: Create NavBarItem.at

**Files:**
- Create: `examples/quickstart/04-Tabs/NavBarItem.at`

**Step 1: Create NavBarItem.at**

```auto
// NavBarItem.at - Navigation bar item component

/// Navigation bar item with order number and title
widget NavBarItem {
    model {
        order: str = ""
        title: str = ""
    }

    view {
        Row {
            Text .order {
                style: "text-lg font-bold text-blue-500 w-8"
            }
            Text .title {
                style: "text-base flex-1"
            }
            style: "w-full p-4 bg-white rounded-lg items-center"
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/quickstart/04-Tabs/NavBarItem.at
git commit -m "feat(example): add NavBarItem component"
```

---

## Task 13: Create KnowledgeMap.at

**Files:**
- Create: `examples/quickstart/04-Tabs/KnowledgeMap.at`

**Step 1: Create KnowledgeMap.at**

```auto
// KnowledgeMap.at - Tab 3 simplified version

use NavBarItem

/// Knowledge map page with static navigation items
widget KnowledgeMap {
    model {
        navBarItems: List = [
            { order: "01", title: "准备与学�? },
            { order: "02", title: "构建应用" },
            { order: "03", title: "应用测试" },
            { order: "04", title: "上架" },
            { order: "05", title: "运营增长" },
            { order: "06", title: "商业变现" },
            { order: "07", title: "更多" }
        ]
    }

    view {
        Scroll {
            Col {
                Text "知识地图" {
                    style: "text-2xl font-bold w-full"
                }
                Image "$r('app.media.knowledge_map_banner')" {
                    style: "w-full rounded-2xl mt-4 mb-2"
                }
                Text "通过循序渐进的学习路径，无经验和有经验的开发者都可以轻松掌握ArkTS语言声明式开发范式，体验更简洁、更友好的HarmonyOS应用开发旅程�? {
                    style: "text-sm text-gray-600 w-full"
                }
                List {
                    for item in .navBarItems {
                        ListItem {
                            NavBarItem { order: item.order, title: item.title }
                        }
                    }
                    style: "w-full mt-6"
                }
            }
            style: "p-3 bg-gray-100 h-full"
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/quickstart/04-Tabs/KnowledgeMap.at
git commit -m "feat(example): add KnowledgeMap simplified version"
```

---

## Task 14: Create App.at with Tabs

**Files:**
- Create: `examples/quickstart/04-Tabs/App.at`

**Step 1: Create App.at**

```auto
// App.at - Main app with bottom tabs

use Tabs, TabsList, TabsTrigger, TabsContent
use QuickStartPage, CourseLearning, KnowledgeMap

/// Main application with 3 tabs
widget App {
    model {
        activeTab: str = "quickstart"
    }

    view {
        Tabs (activeTab: .activeTab) {
            TabsList {
                TabsTrigger quickstart (label: "快速入�?, active: .activeTab == "quickstart") {}
                TabsTrigger learning (label: "课程学习", active: .activeTab == "learning") {}
                TabsTrigger map (label: "知识地图", active: .activeTab == "map") {}
            }
            TabsContent quickstart (active: .activeTab == "quickstart") {
                QuickStartPage {}
            }
            TabsContent learning (active: .activeTab == "learning") {
                CourseLearning {}
            }
            TabsContent map (active: .activeTab == "map") {
                KnowledgeMap {}
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/quickstart/04-Tabs/App.at
git commit -m "feat(example): add App with Tabs component"
```

---

## Task 15: Run Full Test Suite

**Step 1: Run all a2ark tests**

Run: `cargo test -p auto-lang --lib -- generator::tests::test_0`
Expected: All tests pass

**Step 2: Generate ArkTS for App.at**

Run: `cargo run -- ark examples/quickstart/04-Tabs/App.at`
Expected: Generates valid ArkTS with Tabs component

**Step 3: Final commit (if any fixes needed)**

```bash
git add -A
git commit -m "feat(a2ark): complete Tabs implementation"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add is_tabs_pattern helper | generator.rs |
| 2 | Add TabItem struct + extract | generator.rs |
| 3 | Add @Builder generator | generator.rs |
| 4 | Add Tabs component generator | generator.rs |
| 5 | Integrate into generate_node | generator.rs |
| 6 | Add state variables | generator.rs |
| 7 | Add test case 015_tabs | test/a2ark/ |
| 8 | Create project structure | 04-Tabs/ |
| 9 | Copy 03-ItemList components | 04-Tabs/ |
| 10 | Create QuickStartPage | QuickStartPage.at |
| 11 | Create CourseLearning | CourseLearning.at |
| 12 | Create NavBarItem | NavBarItem.at |
| 13 | Create KnowledgeMap | KnowledgeMap.at |
| 14 | Create App with Tabs | App.at |
| 15 | Run full test suite | - |
