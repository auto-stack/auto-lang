# 148 - 05-Nav Navigation Design and Implementation

Date: 2026-03-24
Status: Completed (2026-03-25)

## Objective

Implement Navigation for the 05-Nav project in examples/quickstart to enable page-level navigation within tabs.

## Reference Sample

`D:\gitee\QuickStart\09_SettingUpComponentNavigation\09_Complete`

## Current State

- Basic Tabs implemented in App.at
- QuickStartPage, CourseLearning, KnowledgeMap widgets exist
- No Navigation wrapper (pages are static)
- No detail pages for navigation

## Architecture

```
App (Tabs)
├── Tab: QuickStart (Navigation + NavPathStack)
│   ├── Banner → BannerDetailPage
│   ├── EnablementView → ArticleDetailPage
│   └── TutorialView → ArticleDetailPage
│
├── Tab: CourseLearning (WebView placeholder)
│
└── Tab: KnowledgeMap (Navigation + NavPathStack)
    ├── NavBarItem list (7 items)
    └── KnowledgeMapContent detail page
```

## AURA Navigation Syntax

### Page with Navigation Wrapper

```auto
widget QuickStartPage {
    model {
        @Provide('pathStack') pathStack NavPathStack = NavPathStack()
    }

    view {
        Navigation(pathStack) {
            // scrollable content
        }
        .navDestination(buildNavDestination)
        .hideTitleBar(true)
        .mode(NavigationMode.Stack)
    }
}
```

### Click Handler with Navigation

```auto
widget TutorialItem {
    model {
        @Consume('pathStack') pathStack NavPathStack
    }

    view {
        Row {
            // content
        }
        .onClick(() => pathStack.pushPathByName('articleDetail', this.item))
    }
}
```

### Detail Page with NavDestination

```auto
widget ArticleDetailPage {
    model {
        @Consume('pathStack') pathStack NavPathStack
        article ArticleClass
    }

    view {
        NavDestination {
            // detail content
        }
        .hideTitleBar(true)
    }
}
```

## Components

### New Files Created

| File | Purpose |
|-----|---------|
| `ArticleDetailPage.at` | Detail page showing article in WebView (placeholder) |
| `BannerDetailPage.at` | Detail page for banner content |
| `KnowledgeMapContent.at` | Detail page showing knowledge section materials |
| `KnowledgeMapItem.at` | NavBarItem component for knowledge map sidebar |
| `MapData.json` | Data file with knowledge map sections |

### Files Modified

| File | Changes |
|-----|---------|
| `QuickStartPage.at` | Add Navigation wrapper, @Provide pathStack |
| `TutorialItem.at` | Add @Consume pathStack, onClick handler |
| `EnablementItem.at` | Add @Consume pathStack, onClick handler |
| `Banner.at` | Add @Consume pathStack, onClick handler |
| `KnowledgeMap.at` | Rewrite with Navigation, NavBarItem list, detail page |
| `CourseLearning.at` | Add WebView placeholder |

## Generator Updates

### nav() Function Support

Add `nav()` function to AURA widget spec for navigation:

```auto
// In widget model
nav routeName targetPage

// In widget view
.onClick(() => pathStack.pushPathByName('articleDetail', this.item))
```

### @Provide/@Consume Support

- `@Provide('pathStack')` in parent page (QuickStartPage, KnowledgeMap)
- `@Consume('pathStack')` in child components (TutorialItem, etc.)

### NavDestination Wrapper

Detail pages wrapped in NavDestination component with `.hideTitleBar(true)`.

## Data Flow

1. User taps TutorialItem
2. onClick calls `pathStack.pushPathByName('articleDetail', item)`
3. Navigation's navDestination builder creates ArticleDetailPage
4. User taps back → `pathStack.pop()` returns to list

## Implementation Status

| Task | Description | Status |
|------|-------------|--------|
| 0 | @Consume/@Provide decorators | ✅ Done |
| 1 | nav() function | ✅ Done (Object wrapper for ArkTS) |
| 2 | Navigation wrapper | ✅ Done |
| 3-6 | Detail pages & navigation | ✅ Done |
| 7 | KnowledgeMap | ✅ Done |
| 8 | CourseLearning placeholder | ✅ Done |
| 9 | Generate and test | ✅ Done - Nav working! |

## Technical Implementation Details

### Task 0: @Consume/@Provide Decorator Support

**Files Modified:**
- `crates/auto-lang/src/parser.rs` - Parse decorators in model fields
- `crates/auto-lang/src/ast.rs` - Add decorator fields to ModelField
- `crates/auto-lang/src/aura/extract.rs` - Extract decorators to AuraStateDef
- `crates/auto-lang/src/aura/types.rs` - Add decorator to AuraStateDef
- `crates/auto-lang/src/ui_gen/ark/generator.rs` - Generate ArkTS decorators
- Test: `crates/auto-lang/test/a2ark/017_decorators/`

**ArkTS Decorator Mapping:**

```typescript
// Input: #[Provide("pathStack")] pathStack NavPathStack = NavPathStack()
// Output: @Provide("pathStack") pathStack: NavPathStack = new NavPathStack()

// Input: #[Consume("pathStack")] pathStack NavPathStack
// Output: @Consume("pathStack") pathStack: NavPathStack
```

### Task 1: nav() Function

**Syntax:** `nav("routeName", param)`

**Generated Code:**

```typescript
// Input: .onClick(() => nav("articleDetail", item))
// Output: .onClick(() => { this.pathStack.pushPathByName('articleDetail', this.item) })
```

### Task 2: Navigation Wrapper

**AURA Syntax:**

```auto
widget QuickStartPage {
    model {
        @Provide("pathStack") pathStack NavPathStack = NavPathStack()
    }

    view {
        Navigation(pathStack) {
            // content
        }
        .navDestination(buildNavDestination)
        .hideTitleBar(true)
        .mode(NavigationMode.Stack)
    }

    routes {
        "articleDetail" => ArticleDetailPage
    }
}
```

### Task 3: ArticleDetailPage Widget

```auto
use ArticleClass

widget ArticleDetailPage {
    model {
        @Consume("pathStack") pathStack NavPathStack
        article ArticleClass
    }

    view {
        NavDestination {
            Col {
                // Header with back button
                Row {
                    Image "$r('app.media.ic_back')" {
                        style: "w-10 h-10"
                        .onClick(() => pathStack.pop())
                    }
                    Text article.title {
                        style: "text-xl font-bold ml-2 flex-1"
                    }
                    style: "w-4/5 h-14"
                }
                // WebView placeholder
                Col {
                    Text "WebView: ${article.webUrl}" {
                        style: "text-gray-500 text-center"
                    }
                    style: "flex-1 justify-center items-center"
                }
                style: "w-full h-full px-4"
            }
        }
        .hideTitleBar(true)
    }
}
```

### Task 4-6: Update Items with Navigation

```auto
widget TutorialItem {
    model {
        @Consume("pathStack") pathStack NavPathStack
        item ArticleClass
    }

    view {
        Row {
            // content
        }
        .onClick(() => nav("articleDetail", item))
    }
}
```

## Success Criteria

- ✅ All tabs render correctly
- ✅ Navigation works within QuickStart and KnowledgeMap tabs
- ✅ Detail pages display when items are clicked
- ✅ Back navigation works correctly
- ✅ Generated code matches reference sample patterns

## Files Merged

This document merges:
- `2026-03-24-05-nav-navigation-design.md` (Design)
- `2026-03-24-05-nav-navigation-implementation.md` (Implementation Plan)
