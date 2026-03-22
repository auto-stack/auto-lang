# AURA → ArkTS Transpilation Design

## Objective

Add transpilation support for all 54 AURA widgets to ArkTS components for HarmonyOS, enabling AutoLang to generate native HarmonyOS applications.

## Current State

- **ArkGenerator** (`crates/auto-lang/src/ui_gen/ark/generator.rs`): Already uses `WidgetRegistry` for component lookups
- **AURA Widgets** (`stdlib/aura/widgets/`): 54 widgets across 6 categories
- **ArkTS SDK**: 127+ components with declarative modifier-based API

### Gaps

1. Most AURA widgets have `#[backend(vue, ...)]` but few have `#[backend(ark, ...)]`
2. No prop/event mapping for ArkTS-specific API differences
3. No test coverage for ArkTS output

## Proposed Solution

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AURA Widget (.at file)                    │
│                                                          │
│  #[backend(ark, component = "TextInput")]                │
│  #[backend(ark, prop:value = "text")]                    │
│  #[backend(ark, event:onchange = "onChange")]            │
│  widget Input { ... }                                    │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                  WidgetRegistry (Rust)                       │
│  - Parses #[backend(ark, ...)] annotations                 │
│  - Stores BackendMapping with component, props, events      │
│  - get_backend_component(tag, "ark") → "TextInput"          │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                    ArkGenerator (Rust)                       │
│  - Looks up widget in registry                             │
│  - Generates ArkTS code: TextInput({ ... })                │
│  - Applies modifiers: .onChange(...).width(...)            │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                    Generated .ets file                      │
│                                                          │
│  TextInput({ placeholder: 'Enter text' })                 │
│    .onChange((value) => { ... })                          │
│    .width('100%')                                          │
└─────────────────────────────────────────────────────────────┘
```

### Component Mappings

#### Layout Widgets (7)
| AURA | ArkTS | Notes |
|------|-------|-------|
| `col` | `Column` | Built-in |
| `row` | `Row` | Built-in |
| `box` | `Stack` | Built-in |
| `center` | `Column` + `.justifyContent(FlexAlign.Center)` | Composite |
| `card` | `Column` + styling | Styled container |
| `scrollArea` | `Scroll` | Built-in |
| `aspectRatio` | `Column` + `.aspectRatio()` | Modifier |

#### Form Widgets (9)
| AURA | ArkTS | Notes |
|------|-------|-------|
| `button` | `Button` | Built-in |
| `input` | `TextInput` | Built-in |
| `textarea` | `TextArea` | Built-in |
| `checkbox` | `Checkbox` | Built-in |
| `switch` | `Toggle({ type: ToggleType.Switch })` | Toggle variant |
| `select` | `Select` | Built-in |
| `slider` | `Slider` | Built-in |
| `radioGroup` | `Radio` + custom logic | Composite |
| `form` | `Column` + form styling | Container |

#### Display Widgets (6)
| AURA | ArkTS | Notes |
|------|-------|-------|
| `text` | `Text` | Built-in |
| `image` | `Image` | Built-in |
| `badge` | `Badge` | Built-in |
| `avatar` | `Image` + `.borderRadius(50%)` | Styled image |
| `separator` | `Divider` | Built-in |
| `skeleton` | `LoadingProgress` or custom | Fallback |

#### Navigation Widgets (7)
| AURA | ArkTS | Notes |
|------|-------|-------|
| `tabs` | `Tabs` | Built-in |
| `tab` | `TabContent` | Built-in |
| `breadcrumb` | Custom `Row` + `Text` | Composite |
| `navLink` | `Text` + click handler | Navigation |
| `sidebar` | `SideBarContainer` | Built-in |
| `menuBar` | Custom component | Composite |
| `dropdownMenu` | `Menu` | Built-in |

#### Feedback Widgets (4)
| AURA | ArkTS | Notes |
|------|-------|-------|
| `alert` | `AlertDialog` | Dialog variant |
| `toast` | `promptAction.showToast()` | System API |
| `progress` | `Progress` / `LoadingProgress` | Built-in |
| `sonner` | `promptAction.showToast()` | Toast variant |

#### Overlay Widgets (7)
| AURA | ArkTS | Notes |
|------|-------|-------|
| `dialog` | `AlertDialog` | Built-in |
| `alertDialog` | `AlertDialog` | Built-in |
| `sheet` | `BindSheet` | Built-in |
| `drawer` | Custom panel | Composite |
| `popover` | `BindMenu` | Built-in |
| `tooltip` | `Tooltip` | Built-in |
| `hoverCard` | `BindMenu` | Menu variant |
| `contextMenu` | `BindContextMenu` | Built-in |

#### Data Widgets (3)
| AURA | ArkTS | Notes |
|------|-------|-------|
| `table` | `List` + `ListItem` | Composite |
| `dataTable` | `List` + custom | Composite |
| `calendar` | `CalendarPicker` | Built-in |

### Prop/Event Mapping

```typescript
// AURA                           // ArkTS
value: .name              →   .text(this.name)
placeholder: "Enter..."    →   .placeholder("Enter...")
disabled: true            →   .enabled(false)
onchange: .Update          →   .onChange((value) => this.dispatch(Msg.Update(value)))
onfocus: .Focus            →   .onFocus(() => this.dispatch(Msg.Focus))
onclick: .Submit           →   .onClick(() => this.dispatch(Msg.Submit))
```

## Implementation Phases

### Phase 1: Core Infrastructure (2-3 widgets)
- Extend annotation parser to support `#[backend(ark, ...)]` syntax
- Update `WidgetRegistry` to parse ark backend annotations
- Test with `col`, `text`, `button`

### Phase 2: Layout & Form (16 widgets)
- Layout: `col`, `row`, `box`, `center`, `card`, `scrollArea`, `aspectRatio`
- Form: `button`, `input`, `textarea`, `checkbox`, `switch`, `select`, `slider`, `radioGroup`, `form`
- Add prop mapping for form controls
- Implement two-way binding for form controls

### Phase 3: Display & Feedback (10 widgets)
- Display: `text`, `image`, `badge`, `avatar`, `separator`, `skeleton`
- Feedback: `alert`, `toast`, `progress`, `sonner`
- Handle resource references (`$r('app.media.xxx')`)

### Phase 4: Navigation & Overlay (14 widgets)
- Navigation: `tabs`, `tab`, `breadcrumb`, `navLink`, `sidebar`, `menuBar`, `dropdownMenu`
- Overlay: `dialog`, `alertDialog`, `sheet`, `drawer`, `popover`, `tooltip`, `hoverCard`, `contextMenu`
- Complex state management for navigation

### Phase 5: Data Widgets (3 widgets)
- Data: `table`, `dataTable`, `calendar`
- Complex composite components

## Testing Strategy

### Unit Tests
- Each widget gets a test case in `crates/auto-lang/test/a2ark/` directory
- Compare generated `.ets` against `.expected.ets` files
- Test prop mapping: AURA prop → ArkTS modifier
- Test event mapping: AURA event → ArkTS callback

### Integration Tests
- Generate full HarmonyOS project
- Build and run in DevEco Studio
- Visual verification for complex widgets

## Success Criteria

1. All 54 AURA widgets have `#[backend(ark, ...)]` annotations
2. ArkGenerator produces valid ArkTS code for all widgets
3. Generated code compiles in DevEco Studio
4. Test coverage > 90% for component mappings
5. Documentation updated with ArkTS examples
