# 05-Nav Navigation Design

Date: 2026-03-24
Status: Approved

## Objective
Implement Navigation for the 05-Nav project in examples/quickstart to enabling page-level navigation within tabs.

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

### New Files to Create
| File | Purpose |
|-----|---------|
| `ArticleDetailPage.at` | Detail page showing article in WebView (placeholder) |
| `BannerDetailPage.at` | Detail page for banner content |
| `KnowledgeMapContent.at` | Detail page showing knowledge section materials |
| `KnowledgeMapItem.at` | NavBarItem component for knowledge map sidebar |
| `MapData.json` | Data file with knowledge map sections |

### Files to Modify
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

## Implementation Phases

### Phase 1: Generator Updates
- Add `nav()` function support to AURA widgets
- Generate `pathStack.pushPathByName()` calls in onClick handlers
- Support `NavDestination` wrapper for detail pages

### Phase 2: QuickStartPage Navigation
- Add Navigation wrapper to QuickStartPage
- Add @Provide pathStack
- Add navDestination builder
- Update TutorialItem, EnablementItem, Banner with onClick handlers

### Phase 3: Detail Pages
- Create ArticleDetailPage.at
- Create BannerDetailPage.at

### Phase 4: KnowledgeMap Rewrite
- Rewrite KnowledgeMap with Navigation
- Create NavBarItem list
- Create KnowledgeMapContent detail page
- Add MapData.json

### Phase 5: CourseLearning Placeholder
- Update CourseLearning with WebView placeholder

### Phase 6: Testing
- Generate ArkTS from all widgets
- Compare with reference sample
- Verify navigation flow

## Success Criteria
- All tabs render correctly
- Navigation works within QuickStart and KnowledgeMap tabs
- Detail pages display when items are clicked
- Back navigation works correctly
- Generated code matches reference sample patterns
