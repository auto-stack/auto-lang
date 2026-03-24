# 05-Nav Navigation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement page-level navigation within tabs for the 05-Nav project, enabling navigation from list items to detail pages.

**Architecture:** Per-tab Navigation with NavPathStack. Each tab that needs navigation (QuickStart, KnowledgeMap) wraps its content in a Navigation component with its own NavPathStack. Child widgets use `@Consume` to access the pathStack and call `pushPathByName()` on click.

**Tech Stack:** AutoLang AURA widgets, ArkTS code generation, a2ark transpiler

**Current Status:**
| Task | Description | Status |
|------|-------------|--------|
| 0 | @Consume/@Provide decorators | ✅ Done |
| 1 | nav() function | ✅ Done (Object wrapper for ArkTS) |
| 2 | Navigation wrapper | ✅ Done |
| 3-6 | Detail pages & navigation | ✅ Done |
| 7 | KnowledgeMap | ✅ Done |
| 8 | CourseLearning placeholder | ✅ Done |
| 9 | Generate and test | ✅ Done - Nav working! |

---

## Task 0: Add @Consume and @Provide Decorator Support

**Status:** ✅ Done (2026-03-25)

**Files:**
- Modify: `crates/auto-lang/src/parser.rs` (parse decorators in model fields)
- Modify: `crates/auto-lang/src/ast.rs` (add decorator fields to ModelField)
- Modify: `crates/auto-lang/src/aura/extract.rs` (extract decorators to AuraStateDef)
- Modify: `crates/auto-lang/src/aura/types.rs` (add decorator to AuraStateDef)
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs` (generate ArkTS decorators)
- Test: `crates/auto-lang/test/a2ark/017_decorators/` (new test case)

**Background:**

HarmonyOS ArkTS uses `@Provide` and `@Consume` decorators for hierarchical state sharing:
- `@Provide("key")` in parent component provides a value to descendants
- `@Consume("key")` in child component consumes the value from nearest ancestor

**Step 1: Update AST to support decorators on model fields**

Add decorator field to ModelField struct:

```rust
// In ast.rs
pub struct ModelField {
    pub name: AutoStr,
    pub ty: Type,
    pub init: Option<Expr>,
    pub decorators: Vec<Decorator>,  // NEW
}

pub struct Decorator {
    pub name: AutoStr,           // e.g., "Consume", "Provide"
    pub args: Vec<Expr>,         // e.g., ["pathStack"]
}
```

**Step 2: Update parser to parse decorators**

Syntax: `#[DecoratorName("arg")] fieldName Type = value`

```auto
model {
    #[Provide("pathStack")] pathStack NavPathStack = NavPathStack()
    #[Consume("pathStack")] pathStack NavPathStack
}
```

Parser changes needed in `parse_model_field`:
- Check for `#[...]` before field name
- Parse decorator name and arguments
- Support multiple decorators: `#[Consume, Optional]`

**Step 3: Update AURA extraction**

Add decorator info to AuraStateDef:

```rust
// In aura/types.rs
pub struct AuraStateDef {
    pub name: String,
    pub type_info: Type,
    pub initial: AuraExpr,
    pub decorators: Vec<AuraDecorator>,  // NEW
}

pub struct AuraDecorator {
    pub name: String,      // "Consume" or "Provide"
    pub args: Vec<String>, // ["pathStack"]
}
```

**Step 4: Update ArkTS generator**

Generate proper ArkTS decorators:

```typescript
// Input:
// #[Provide("pathStack")] pathStack NavPathStack = NavPathStack()

// Output:
@Provide("pathStack") pathStack: NavPathStack = new NavPathStack()
```

```typescript
// Input:
// #[Consume("pathStack")] pathStack NavPathStack

// Output:
@Consume("pathStack") pathStack: NavPathStack
```

**Step 5: Create test case 017_decorators**

Create test files:
- `input.at` - Widget with @Provide and @Consume decorators
- `input.expected.ets` - Expected ArkTS output with decorators

**Step 6: Run tests to verify**

Run: `cargo test -p auto-lang --lib -- generator::tests::test_017_decorators`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/auto-lang/src/parser.rs crates/auto-lang/src/ast.rs \
        crates/auto-lang/src/aura/extract.rs crates/auto-lang/src/aura/types.rs \
        crates/auto-lang/src/ui_gen/ark/generator.rs \
        crates/auto-lang/test/a2ark/017_decorators/
git commit -m "feat(a2ark): add @Consume and @Provide decorator support"
```

**Notes:**
- AutoLang uses `#[...]` syntax for annotations (Rust-style)
- Only `@Consume` and `@Provide` are needed for navigation
- Other decorators (`@State`, `@Prop`, `@Link`) can be added later

---

## Task 1: Add nav() Function to AURA Generator

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs:499-530` (onClick generation)
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs` (nav function support)
- Test: `crates/auto-lang/test/a2ark/016_nav/` (new test case)

**Step 1: Add nav() function parsing in AURA parser**

Add support for `nav("routeName", param)` function in onClick handlers:

```auto
// In TutorialItem.at
Row {
    // content
}
.onClick(() => nav("articleDetail", this.item))
```

**Step 2: Update generator to generate pathStack.pushPathByName()**

When `nav()` is detected in onClick, generate:
```typescript
.onClick(() => {
    this.pathStack.pushPathByName('articleDetail', this.item)
})
```

**Step 3: Create test case 016_nav**

Create test files:
- `input.at` - Widget with nav() call
- `input.expected.ets` - Expected ArkTS output with pushPathByName

**Step 4: Run test to verify**

Run: `cargo test -p auto-lang --lib -- generator::tests::test_016_nav`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs crates/auto-lang/test/a2ark/016_nav/
git commit -m "feat(a2ark): add nav() function for navigation"
```

---

## Task 2: Update QuickStartPage with Navigation Wrapper

**Files:**
- Modify: `examples/quickstart/05-Nav/QuickStartPage.at`
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs` (Navigation generation)

**Step 1: Update QuickStartPage.at**

Add Navigation wrapper with @Provide pathStack:

```auto
use Banner
use EnablementView
use TutorialView
use ArticleDetailPage

widget QuickStartPage {
    model {
        @Provide("pathStack") pathStack NavPathStack = NavPathStack()
    }

    view {
        Navigation(pathStack) {
            Col {
                Text "快速入门" {
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
        .navDestination(buildNavDestination)
        .hideTitleBar(true)
        .mode(NavigationMode.Stack)
    }

    routes {
        "articleDetail" => ArticleDetailPage
    }
}
```

**Step 2: Update generator for routes block**

Add support for `routes` block in AURA widgets that generates `@Builder navDestinationBuilder` function.

**Step 3: Run existing tests**

Run: `cargo test -p auto-lang -- trans`
Expected: All existing tests pass

**Step 4: Commit**

```bash
git add examples/quickstart/05-Nav/QuickStartPage.at
git commit -m "feat(05-Nav): add Navigation wrapper to QuickStartPage"
```

---

## Task 3: Create ArticleDetailPage Widget

**Files:**
- Create: `examples/quickstart/05-Nav/ArticleDetailPage.at`
- Create: `examples/quickstart/05-Nav/ArticleClass.at` (model)

**Step 1: Create ArticleClass.at model**

```auto
// ArticleClass.at - Article data model

type ArticleClass {
    id str
    title str
    brief str
    imageSrc str
    webUrl str
}
```

**Step 2: Create ArticleDetailPage.at**

```auto
// ArticleDetailPage.at - Article detail page with WebView placeholder

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
                // WebView placeholder (not yet supported)
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

**Step 3: Verify syntax**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: No errors

**Step 4: Commit**

```bash
git add examples/quickstart/05-Nav/ArticleDetailPage.at examples/quickstart/05-Nav/ArticleClass.at
git commit -m "feat(05-Nav): add ArticleDetailPage widget"
```

---

## Task 4: Update TutorialItem with Navigation

**Files:**
- Modify: `examples/quickstart/05-Nav/TutorialItem.at`

**Step 1: Update TutorialItem.at**

Add @Consume pathStack and onClick handler:

```auto
use TutorialItem
use ArticleClass

widget TutorialItem {
    model {
        @Consume("pathStack") pathStack NavPathStack
        item ArticleClass
    }

    view {
        Row {
            Col {
                Text item.title {
                    style: "text-sm font-normal w-full text-left mt-1"
                }
                Text item.brief {
                    style: "text-xs text-gray-600 w-full text-left mt-1 line-clamp-2"
                }
                style: "h-full flex-1 items-start mr-3"
            }
            Image item.imageSrc {
                style: "w-28 h-16 rounded-2xl object-cover"
            }
            style: "w-full h-22 bg-white rounded-2xl p-3"
        }
        .onClick(() => nav("articleDetail", item))
    }
}
```

**Step 2: Update TutorialView to pass data**

Update TutorialView.at to load data from JSON and pass to TutorialItem.

**Step 3: Commit**

```bash
git add examples/quickstart/05-Nav/TutorialItem.at examples/quickstart/05-Nav/TutorialView.at
git commit -m "feat(05-Nav): add navigation to TutorialItem"
```

---

## Task 5: Update EnablementItem with Navigation

**Files:**
- Modify: `examples/quickstart/05-Nav/EnablementItem.at`

**Step 1: Update EnablementItem.at**

Add onClick handler with nav():

```auto
widget EnablementItem {
    model {
        @Consume("pathStack") pathStack NavPathStack
        item EnablementClass
    }

    view {
        Col {
            Image item.imageSrc {
                style: "w-full h-20 rounded-xl object-cover"
            }
            Text item.title {
                style: "text-sm font-medium mt-2"
            }
            Text item.brief {
                style: "text-xs text-gray-600 mt-1"
            }
            style: "w-36 bg-white rounded-xl p-2"
        }
        .onClick(() => nav("articleDetail", item))
    }
}
```

**Step 2: Commit**

```bash
git add examples/quickstart/05-Nav/EnablementItem.at
git commit -m "feat(05-Nav): add navigation to EnablementItem"
```

---

## Task 6: Create BannerDetailPage Widget

**Files:**
- Create: `examples/quickstart/05-Nav/BannerDetailPage.at`
- Modify: `examples/quickstart/05-Nav/Banner.at`

**Step 1: Create BannerDetailPage.at**

```auto
// BannerDetailPage.at - Banner detail page

use BannerClass

widget BannerDetailPage {
    model {
        @Consume("pathStack") pathStack NavPathStack
        banner BannerClass
    }

    view {
        NavDestination {
            Col {
                Row {
                    Image "$r('app.media.ic_back')" {
                        style: "w-10 h-10"
                        .onClick(() => pathStack.pop())
                    }
                    Text banner.title {
                        style: "text-xl font-bold ml-2"
                    }
                    style: "w-full h-14"
                }
                Image banner.imageSrc {
                    style: "w-full h-48 object-cover mt-4"
                }
                Text banner.description {
                    style: "text-sm text-gray-600 mt-4"
                }
                style: "w-full h-full p-4"
            }
        }
        .hideTitleBar(true)
    }
}
```

**Step 2: Update Banner.at with onClick**

Add nav() call to Banner widget.

**Step 3: Commit**

```bash
git add examples/quickstart/05-Nav/BannerDetailPage.at examples/quickstart/05-Nav/Banner.at
git commit -m "feat(05-Nav): add BannerDetailPage and navigation to Banner"
```

---

## Task 7: Rewrite KnowledgeMap with Navigation

**Files:**
- Modify: `examples/quickstart/05-Nav/KnowledgeMap.at`
- Create: `examples/quickstart/05-Nav/KnowledgeMapItem.at`
- Create: `examples/quickstart/05-Nav/KnowledgeMapContent.at`
- Create: `examples/quickstart/05-Nav/data/MapData.json`

**Step 1: Create MapData.json**

Create data file with knowledge map sections based on reference sample.

**Step 2: Create KnowledgeMapItem.at**

NavBarItem component that displays order + title and navigates to detail page.

**Step 3: Create KnowledgeMapContent.at**

Detail page that shows section materials.

**Step 4: Rewrite KnowledgeMap.at**

Add Navigation wrapper, NavBarItem list, navDestination builder.

**Step 5: Commit**

```bash
git add examples/quickstart/05-Nav/KnowledgeMap.at examples/quickstart/05-Nav/KnowledgeMapItem.at examples/quickstart/05-Nav/KnowledgeMapContent.at examples/quickstart/05-Nav/data/
git commit -m "feat(05-Nav): rewrite KnowledgeMap with Navigation"
```

---

## Task 8: Update CourseLearning with WebView Placeholder

**Files:**
- Modify: `examples/quickstart/05-Nav/CourseLearning.at`

**Step 1: Update CourseLearning.at**

Add WebView placeholder with comment:

```auto
// CourseLearning.at - Course Learning tab content
// Note: WebView not yet supported, using placeholder

widget CourseLearning {
    view {
        Col {
            Text "课程学习" {
                style: "text-2xl font-bold mb-4"
            }
            Col {
                Text "WebView Placeholder" {
                    style: "text-gray-500"
                }
                Text "Course content will be displayed here" {
                    style: "text-sm text-gray-400"
                }
                style: "flex-1 justify-center items-center bg-gray-100"
            }
            style: "w-full h-full p-4 bg-white"
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/quickstart/05-Nav/CourseLearning.at
git commit -m "feat(05-Nav): add WebView placeholder to CourseLearning"
```

---

## Task 9: Generate and Test

**Files:**
- All generated .ets files

**Step 1: Generate ArkTS from all widgets**

Run: `cargo run --release -- build examples/quickstart/05-Nav`
Expected: All .ets files generated

**Step 2: Compare with reference sample**

Manually compare generated code with reference sample patterns.

**Step 3: Fix any issues**

If generated code doesn't match expected patterns, update generator.

**Step 4: Final commit**

```bash
git add examples/quickstart/05-Nav/
git commit -m "feat(05-Nav): complete navigation implementation"
```

---

## Testing Strategy

1. **Unit Tests**: Add a2ark test case for nav() function (Task 1)
2. **Integration Tests**: Generate ArkTS and compare with reference sample
3. **Manual Testing**: Run generated app in DevEco Studio

## Dependencies
- Task 1 (nav() function) - ✅ DONE (partially - object params work)
- Task 0 (@Consume/@Provide) - 🔴 BLOCKER for Tasks 2-6
- Task 2 depends on Task 0 (@Provide needed for Navigation wrapper)
- Task 3-6 depend on Task 0 (@Consume needed in child widgets)
- Task 7 depends on Task 0 and Task 1
- Task 9 depends on all previous tasks

## Risk Areas
1. **Generator complexity**: Navigation generation adds significant complexity
2. **Data loading**: JSON data loading may need runtime support
3. **WebView**: Not supported, using placeholders
