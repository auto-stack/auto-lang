# KnowledgeMap Data Loading Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace static placeholder content in KnowledgeMap with real data loaded from JSON at runtime.

**Architecture:** Define data types and data in AutoLang .at files. Transpile MapData.at to MapData.json in rawfile. Add Json.load() function and lifecycle block support in generator. Use for-index syntax for ForEach with index.

**Tech Stack:** AutoLang AURA widgets, ArkTS code generation, a2ark transpiler

---

## Task 0: Create Types.at with Data Type Definitions

**Files:**
- Create: `examples/quickstart/05-Nav/Types.at`

**Step 1: Create Types.at**

```auto
// Types.at - Data types for Knowledge Map

/// Knowledge base item (e.g., "指南 - DevEco Studio")
type KnowledgeBaseItem {
    type str
    title str
}

/// Material section within a learning phase
type Material {
    subtitle str
    knowledgeBase KnowledgeBaseItem[]
}

/// Learning section/phase in the knowledge map
type Section {
    title str
    brief str
    materials Material[]
}

/// Nav bar item for navigation list
type NavBarItemType {
    order str
    title str
}
```

**Step 2: Verify syntax**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: No errors

**Step 3: Commit**

```bash
git add examples/quickstart/05-Nav/Types.at
git commit -m "feat(05-Nav): add data type definitions for KnowledgeMap"
```

---

## Task 1: Create MapData.at with Full Data

**Files:**
- Create: `examples/quickstart/05-Nav/MapData.at`

**Step 1: Create MapData.at with sections data**

Copy full data from reference `MapData.json` (7 sections with all materials):

```auto
// MapData.at - Knowledge map data

use Types

let sections Section[] = [
    {
        title: "准备与学习",
        brief: "加入HarmonyOS生态，注册成为开发者，通过HarmonyOS课程了解基本概念和基础知识，轻松开启HarmonyOS的开发旅程。",
        materials: [
            {
                subtitle: "HarmonyOS简介",
                knowledgeBase: [
                    { type: "准备", title: "注册账号" },
                    { type: "准备", title: "实名认证" },
                    { type: "学习与获取证书", title: "HarmonyOS第一课" },
                    { type: "学习与获取证书", title: "HarmonyOS应用开发认证" }
                ]
            },
            {
                subtitle: "赋能套件介绍",
                knowledgeBase: [
                    { type: "指南", title: "开发" },
                    { type: "指南", title: "最佳实践" },
                    { type: "指南", title: "API参考" },
                    { type: "指南", title: "视频教程" },
                    { type: "指南", title: "Codelabs" },
                    { type: "指南", title: "FAQ" }
                ]
            }
        ]
    },
    {
        title: "构建应用",
        brief: "为了帮助开发者更好的理解HarmonyOS提供的能力，我们对重点功能提供了开发指导，辅助开发者完成应用的开发。",
        materials: [
            {
                subtitle: "开发工具",
                knowledgeBase: [
                    { type: "指南", title: "DevEco Studio" },
                    { type: "指南", title: "低代码开发" },
                    { type: "视频教程", title: "使用DevEco Studio高效开发" }
                ]
            },
            {
                subtitle: "开发语言",
                knowledgeBase: [
                    { type: "指南", title: "ArkTS" },
                    { type: "视频教程", title: "ArkTS基础知识" },
                    { type: "视频教程", title: "ArkTS开发实践" }
                ]
            },
            {
                subtitle: "开发框架",
                knowledgeBase: [
                    { type: "指南", title: "ArkUI" },
                    { type: "视频教程", title: "ArkUI之属性动画" }
                ]
            },
            {
                subtitle: "HarmonyOS云开发",
                knowledgeBase: [
                    { type: "指南", title: "体验HarmonyOS云开发" },
                    { type: "指南", title: "云开发" },
                    { type: "视频教程", title: "HarmonyOS云开发" }
                ]
            },
            {
                subtitle: "集成开放能力",
                knowledgeBase: [
                    { type: "指南", title: "推送服务" },
                    { type: "指南", title: "广告服务" },
                    { type: "指南", title: "帐号服务" },
                    { type: "指南", title: "分析服务" },
                    { type: "指南", title: "应用内支付服务" },
                    { type: "指南", title: "云函数" },
                    { type: "指南", title: "云存储" },
                    { type: "指南", title: "云数据库" }
                ]
            },
            {
                subtitle: "编译调试",
                knowledgeBase: [
                    { type: "指南", title: "编译构建" },
                    { type: "指南", title: "应用签名" },
                    { type: "指南", title: "云调试" },
                    { type: "视频教程", title: "HarmonyOS应用调试前准备" },
                    { type: "视频教程", title: "HarmonyOS应用调试" },
                    { type: "视频教程", title: "HarmonyOS调试工具介绍" }
                ]
            }
        ]
    },
    {
        title: "应用测试",
        brief: "HarmonyOS应用/服务开发完成后，在发布到应用/服务市场前，还需要对应用进行：漏洞、隐私、兼容性、稳定性、性能等测试，确保HarmonyOS应用/服务纯净、安全，给用户带来更好的使用体验。",
        materials: [
            {
                subtitle: "",
                knowledgeBase: [
                    { type: "指南", title: "云测试" },
                    { type: "指南", title: "开放式测试" }
                ]
            }
        ]
    },
    {
        title: "上架",
        brief: "HarmonyOS应用/服务开发、测试完成后，将应用/服务发布至应用市场，用户可以通过应用市场、负一屏等渠道获取到对应的HarmonyOS应用/服务。",
        materials: [
            {
                subtitle: "应用发布",
                knowledgeBase: [
                    { type: "指南", title: "发布HarmonyOS应用" },
                    { type: "指南", title: "发布元服务" },
                    { type: "指南", title: "分阶段发布" },
                    { type: "视频教程", title: "发布HarmonyOS应用" },
                    { type: "视频教程", title: "发布元服务" }
                ]
            }
        ]
    },
    {
        title: "运营增长",
        brief: "HarmonyOS应用/服务发布以后，通过数据及时了解运营情况、质量表现，制定增长策略，借助App Linking、崩溃服务等能力，实现应用及服务的用户增长以及质量提升。",
        materials: [
            {
                subtitle: "应用发布",
                knowledgeBase: [
                    { type: "指南", title: "应用分析" },
                    { type: "指南", title: "App Linking" },
                    { type: "指南", title: "崩溃服务" },
                    { type: "视频教程", title: "远程配置" },
                    { type: "视频教程", title: "发布元服务" }
                ]
            }
        ]
    },
    {
        title: "商业变现",
        brief: "HarmonyOS应用/服务发布以后，通过数据及时了解运营情况、质量表现，制定增长策略，借助App Linking、崩溃服务等能力，实现应用及服务的用户增长以及质量提升。",
        materials: [
            {
                subtitle: "",
                knowledgeBase: [
                    { type: "指南", title: "流量变现" },
                    { type: "指南", title: "联运服务" },
                    { type: "指南", title: "付费服务" },
                    { type: "指南", title: "结算指南" }
                ]
            }
        ]
    },
    {
        title: "更多",
        brief: "",
        materials: [
            {
                subtitle: "",
                knowledgeBase: [
                    { type: "指南", title: "常见问题" },
                    { type: "指南", title: "版本说明" }
                ]
            }
        ]
    }
]
```

**Step 2: Verify syntax**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: No errors

**Step 3: Commit**

```bash
git add examples/quickstart/05-Nav/MapData.at
git commit -m "feat(05-Nav): add MapData with 7 sections"
```

---

## Task 2: Add lifecycle Block Parsing in Parser

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`
- Modify: `crates/auto-lang/src/aura/types.rs`

**Step 1: Add AuraLifecycle to types.rs**

```rust
// In aura/types.rs

/// Lifecycle method definition
#[derive(Debug, Clone, PartialEq)]
pub struct AuraLifecycle {
    pub name: String,        // "aboutToAppear"
    pub body: Vec<Stmt>,     // Method body statements
}

// Add to AuraWidget struct
pub struct AuraWidget {
    // ... existing fields
    pub lifecycle: Vec<AuraLifecycle>,
}
```

**Step 2: Add lifecycle parsing in parser.rs**

Parse `lifecycle { aboutToAppear() { ... } }` block.

**Step 3: Run existing tests**

Run: `cargo test -p auto-lang`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/auto-lang/src/parser.rs crates/auto-lang/src/aura/types.rs
git commit -m "feat(parser): add lifecycle block parsing"
```

---

## Task 3: Add for-index Loop Parsing

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`
- Modify: `crates/auto-lang/src/ast.rs`

**Step 1: Update ForIn AST to support index variable**

```rust
// In ast.rs
pub struct ForIn {
    pub var: AutoStr,
    pub index: Option<AutoStr>,  // NEW: optional index variable
    pub iterable: Expr,
    pub body: Vec<Stmt>,
}
```

**Step 2: Update parser to parse `for i, item in list`**

Parse comma-separated variables in for-in statement.

**Step 3: Run tests**

Run: `cargo test -p auto-lang`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/auto-lang/src/parser.rs crates/auto-lang/src/ast.rs
git commit -m "feat(parser): add for-index loop syntax support"
```

---

## Task 4: Generate for-index to ForEach with Index

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Update generate_for_in to handle index variable**

When index is present, generate:

```typescript
ForEach(this.list, (item: Type, i: number) => {
    // body
}, (item: Type, i: number) => i.toString())
```

**Step 2: Run a2ark tests**

Run: `cargo test -p auto-lang --lib -- generator::tests`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): generate ForEach with index from for-index syntax"
```

---

## Task 5: Add Json.load() Function Support

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Detect Json.load() in expressions**

When `Json.load("filename.json")` is detected, generate:

```typescript
((context) => {
    context.getHostContext()?.resourceManager.getRawFileContent("filename.json",
        (error: BusinessError, value: Uint8Array) => {
            const textDecoder = util.TextDecoder.create("utf-8");
            const res = textDecoder.decodeToString(value, { stream: false });
            this.sections = JSON.parse(res);
        });
})(this.getUIContext())
```

**Step 2: Add required imports**

Add `import { BusinessError } from '@kit.BasicServicesKit';`
Add `import { util } from '@kit.ArkTS';`

**Step 3: Run tests**

Run: `cargo test -p auto-lang -- trans`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): add Json.load() function for rawfile loading"
```

---

## Task 6: Generate aboutToAppear Lifecycle Method

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Generate lifecycle methods in component**

For `lifecycle { aboutToAppear() { ... } }`, generate:

```typescript
aboutToAppear(): void {
    // body
}
```

**Step 2: Run tests**

Run: `cargo test -p auto-lang -- trans`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "feat(a2ark): generate aboutToAppear lifecycle method"
```

---

## Task 7: Rewrite KnowledgeMap with JSON Loading

**Files:**
- Modify: `examples/quickstart/05-Nav/KnowledgeMap.at`

**Step 1: Rewrite KnowledgeMap.at**

```auto
// KnowledgeMap.at - Knowledge Map tab content

use NavBarItem
use KnowledgeMapContent
use Types

widget KnowledgeMap {
    model {
        #[Provide("pathStack")] pathStack NavPathStack = NavPathStack()
        currentIndex int = -1
        sections Section[] = []
    }

    lifecycle {
        aboutToAppear() {
            this.sections = Json.load("MapData.json")
        }
    }

    view {
        Navigation(pathStack) {
            Scroll {
                Col {
                    Text "知识地图" {
                        style: "text-2xl font-bold w-full"
                    }
                    Image "$r('app.media.knowledge_map_banner')" {
                        style: "w-full rounded-2xl mt-5 mb-2"
                    }
                    Text "通过循序渐进的学习路径，无经验和有经验的开发者都可以轻松掌握ArkTS语言声明式开发范式，体验更简洁、更友好的HarmonyOS应用开发旅程。" {
                        style: "text-sm text-gray-600 text-justify"
                    }
                    List {
                        for i, section in sections {
                            ListItem {
                                NavBarItem(
                                    order: format("{:02d}", i + 1),
                                    title: section.title,
                                    currentIndex: currentIndex,
                                    index: i
                                )
                            }
                        }
                        style: "w-full mt-6"
                    }
                    style: "w-full p-4"
                }
            }
            style: "flex-1 bg-gray-100"
        }
        .hideTitleBar(true)
        .mode(NavigationMode.Stack)
    }

    routes {
        "KnowledgeMapContent" => use KnowledgeMapContent
    }
}
```

**Step 2: Verify syntax**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: No errors

**Step 3: Commit**

```bash
git add examples/quickstart/05-Nav/KnowledgeMap.at
git commit -m "feat(05-Nav): rewrite KnowledgeMap with JSON loading"
```

---

## Task 8: Create NavBarItem Widget

**Files:**
- Modify: `examples/quickstart/05-Nav/NavBarItem.at`

**Step 1: Rewrite NavBarItem.at**

```auto
// NavBarItem.at - Navigation bar item component

widget NavBarItem {
    model {
        #[Consume("pathStack")] pathStack NavPathStack
        order str
        title str
        currentIndex int
        index int
    }

    view {
        Row {
            Text order {
                style: "text-xl font-bold text-gray-800 mr-1.5"
            }
            Text title {
                style: "text-base font-medium text-gray-800"
            }
            Image "$r('app.media.ic_arrow')" {
                style: "w-3 h-6 ml-auto"
            }
            style: "w-full h-12 rounded-2xl p-3 items-center"
            class: currentIndex == index ? "bg-blue-100" : ""
            onclick: nav("KnowledgeMapContent", Object({ sectionIndex: index }))
        }
    }
}
```

**Step 2: Verify syntax**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: No errors

**Step 3: Commit**

```bash
git add examples/quickstart/05-Nav/NavBarItem.at
git commit -m "feat(05-Nav): add NavBarItem widget with navigation"
```

---

## Task 9: Rewrite KnowledgeMapContent with Materials

**Files:**
- Modify: `examples/quickstart/05-Nav/KnowledgeMapContent.at`

**Step 1: Rewrite KnowledgeMapContent.at**

```auto
// KnowledgeMapContent.at - Knowledge map content detail page

use Types
use MapData

widget KnowledgeMapContent {
    model {
        #[Consume("pathStack")] pathStack NavPathStack
        #[NavParam("KnowledgeMapContent")] sectionIndex int = 0
        section Section = sections[sectionIndex]
    }

    view {
        NavDestination {
            Scroll {
                Col {
                    Text section.title {
                        style: "text-xl font-bold text-black"
                    }
                    Text section.brief {
                        style: "text-xs text-gray-600 text-justify mt-3"
                    }
                    for material in section.materials {
                        Col {
                            if material.subtitle != "" {
                                Text material.subtitle {
                                    style: "text-sm font-medium mt-7 mb-2"
                                }
                            }
                            Col {
                                for item in material.knowledgeBase {
                                    Row {
                                        Image "$r('app.media.ic_guide')" {
                                            style: "w-5 h-5"
                                        }
                                        Col {
                                            Text item.title {
                                                style: "text-base font-medium"
                                            }
                                            Text item.type {
                                                style: "text-sm text-gray-600"
                                            }
                                            style: "items-start ml-4"
                                        }
                                        Image "$r('app.media.ic_arrow')" {
                                            style: "w-3 h-6 ml-auto"
                                        }
                                        style: "w-full h-16 items-center"
                                    }
                                }
                                style: "w-full bg-white rounded-2xl p-3"
                            }
                        }
                    }
                    style: "w-full px-6 py-3"
                }
            }
            style: "flex-1 bg-gray-100"
        }
        .hideTitleBar(true)
    }
}
```

**Step 2: Verify syntax**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: No errors

**Step 3: Commit**

```bash
git add examples/quickstart/05-Nav/KnowledgeMapContent.at
git commit -m "feat(05-Nav): rewrite KnowledgeMapContent with materials display"
```

---

## Task 10: Generate MapData.json to rawfile

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/project.rs`

**Step 1: Detect data definitions in .at files**

When `let varName Type[] = [...]` is found, generate JSON file.

**Step 2: Copy JSON to rawfile directory**

Output: `entry/src/main/resources/rawfile/MapData.json`

**Step 3: Run build and verify**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: MapData.json created in rawfile

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/project.rs
git commit -m "feat(a2ark): generate JSON from data definitions to rawfile"
```

---

## Task 11: Generate and Test

**Files:**
- All generated .ets files

**Step 1: Generate full project**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: All .ets files generated, MapData.json in rawfile

**Step 2: Open in DevEco Studio and build**

Verify no ArkTS compiler errors.

**Step 3: Test navigation**

- Click on each nav bar item
- Verify correct section content displays
- Verify back navigation works

**Step 4: Final commit**

```bash
git add examples/quickstart/05-Nav/
git commit -m "feat(05-Nav): complete KnowledgeMap with real data loading"
```

---

## Testing Strategy

1. **Unit Tests**: Add a2ark test for for-index and Json.load
2. **Integration Tests**: Generate ArkTS and compare with reference patterns
3. **Manual Testing**: Run in DevEco Studio, verify navigation and data display

## Dependencies

- Task 2-3 (parser) can run in parallel
- Task 4-6 (generator) depend on Task 2-3
- Task 7-9 (widgets) depend on Task 4-6
- Task 10-11 (integration) depend on all previous

## Risk Areas

1. **Json.load async**: ArkTS resourceManager is async, need callback handling
2. **Type mapping**: Complex nested types need correct interface generation
3. **Index binding**: ForEach key function needs proper index handling