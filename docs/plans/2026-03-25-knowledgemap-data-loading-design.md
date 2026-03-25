# KnowledgeMap Data Loading Design

**Goal:** Replace static placeholder content in KnowledgeMap with real data loaded from JSON, matching the reference implementation.

**Approach:** Define data in AutoLang .at files, transpile to JSON, load at runtime.

## Data Types

```auto
// Types.at - Data types for Knowledge Map

type KnowledgeBaseItem {
    type str      // "指南", "准备", "视频教程", "学习与获取证书"
    title str     // "注册账号", "DevEco Studio", etc.
}

type Material {
    subtitle str
    knowledgeBase KnowledgeBaseItem[]
}

type Section {
    title str
    brief str
    materials Material[]
}

type NavBarItemType {
    order str     // "01", "02", etc.
    title str
}
```

## Data Definition

```auto
// MapData.at - Knowledge map data

use Types

let navBarList NavBarItemType[] = [
    { order: "01", title: "准备与学习" },
    { order: "02", title: "构建应用" },
    { order: "03", title: "应用测试" },
    { order: "04", title: "上架" },
    { order: "05", title: "运营增长" },
    { order: "06", title: "商业变现" },
    { order: "07", title: "更多" }
]

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
    // ... 6 more sections (see MapData.json reference)
]
```

## NavBarItem Widget

```auto
// NavBarItem.at - Navigation bar item component

use Types

widget NavBarItem {
    model {
        #[Consume("pathStack")] pathStack NavPathStack
        item NavBarItemType
        currentIndex int
        index int
    }

    view {
        Row {
            Text item.order {
                style: "text-xl font-bold text-gray-800 mr-1.5"
            }
            Text item.title {
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

## KnowledgeMap Widget

```auto
// KnowledgeMap.at - Knowledge Map tab content

use NavBarItem
use KnowledgeMapContent
use Types

widget KnowledgeMap {
    model {
        #[Provide("pathStack")] pathStack NavPathStack = NavPathStack()
        currentIndex int = -1
        navBarList NavBarItemType[] = []
        sections Section[] = []
    }

    lifecycle {
        aboutToAppear() {
            let data = Json.load("MapData.json")
            this.sections = data
            for i, section in this.sections {
                this.navBarList.push({ order: format("{:02d}", i + 1), title: section.title })
            }
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
                        for i, item in navBarList {
                            ListItem {
                                NavBarItem(item: item, currentIndex: currentIndex, index: i)
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

## KnowledgeMapContent Widget

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
                            Text material.subtitle {
                                style: "text-sm font-medium mt-7 mb-2"
                            }
                            Col {
                                for item in material.knowledgeBase {
                                    Row {
                                        Image "$r(TypeMapIcon[item.type])" {
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

## Generator Changes Required

### 1. Json.load() Function

Generate ArkTS code to load JSON from rawfile:

```typescript
// Auto: Json.load("MapData.json")
// Generated:
getUIContext().getHostContext()?.resourceManager.getRawFileContent("MapData.json",
    (error: BusinessError, value: Uint8Array) => {
        const textDecoder = util.TextDecoder.create("utf-8");
        const res = textDecoder.decodeToString(value, { stream: false });
        this.sections = JSON.parse(res);
    });
```

### 2. lifecycle Block Support

Parse and generate `aboutToAppear()` lifecycle method:

```auto
lifecycle {
    aboutToAppear() {
        // code
    }
}
```

Generates:

```typescript
aboutToAppear(): void {
    // code
}
```

### 3. Data File to JSON

Transpile `MapData.at` data definitions to `MapData.json`:

- Copy generated JSON to `entry/src/main/resources/rawfile/MapData.json`

### 4. for i, item in list Syntax

Generate ForEach with index:

```auto
for i, item in navBarList { ... }
```

Generates:

```typescript
ForEach(this.navBarList, (item: NavBarItemType, i: number) => {
    // ...
}, (item: NavBarItemType, i: number) => i.toString())
```

## Files to Create/Modify

| File | Action |
|------|--------|
| `examples/quickstart/05-Nav/Types.at` | Create - type definitions |
| `examples/quickstart/05-Nav/MapData.at` | Create - data definitions |
| `examples/quickstart/05-Nav/NavBarItem.at` | Modify - add nav support |
| `examples/quickstart/05-Nav/KnowledgeMap.at` | Rewrite - with JSON loading |
| `examples/quickstart/05-Nav/KnowledgeMapContent.at` | Rewrite - with materials display |
| `crates/auto-lang/src/ui_gen/ark/generator.rs` | Modify - Json.load, lifecycle, for-index |

## Reference

- Reference implementation: `D:\gitee\QuickStart\09_SettingUpComponentNavigation\09_Complete\features\map\`
- MapData.json: `features\map\src\main\resources\rawfile\MapData.json`