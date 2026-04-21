# Stdlib Widget Library

## Design

Date: 2026-03-22
Status: Approved

## Objective

Migrate all components from `component-gallery` into `stdlib/aura/widgets` as standardized core widgets, creating a comprehensive widget library for AURA.

## Scope

- **Source**: `examples/component-gallery/source/front/components/*.at` (~45 components)
- **Target**: `stdlib/aura/widgets/` (7 categories)
- **Exclude**: Demo widgets (e.g., `ButtonDemo`, `CardDemo`)

## Category Structure

```
stdlib/aura/widgets/
├── mod.at                    # Re-exports all categories
├── display/
�?  ├── mod.at
�?  ├── Text.at              # Text, H1-H6, Paragraph, Span, Label
�?  ├── Image.at
�?  ├── Badge.at
�?  ├── Avatar.at
�?  ├── Separator.at
�?  └── Skeleton.at
├── form/
�?  ├── mod.at
�?  ├── Button.at            # Button with variants
�?  ├── Input.at             # Input, Textarea
�?  ├── Checkbox.at
�?  ├── Switch.at
�?  ├── Select.at            # Select, Combobox
�?  ├── Slider.at
�?  ├── RadioGroup.at
�?  └── Form.at              # Form, FormField, FormLabel, FormControl
├── layout/
�?  ├── mod.at
�?  ├── Col.at
�?  ├── Row.at
�?  ├── Center.at
�?  ├── Card.at              # Card, CardHeader, CardContent, CardFooter
�?  ├── ScrollArea.at
�?  ├── AspectRatio.at
�?  ├── Collapsible.at
�?  └── Accordion.at         # Accordion, AccordionItem, AccordionTrigger, AccordionContent
├── overlay/
�?  ├── mod.at
�?  ├── Dialog.at            # Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter
�?  ├── AlertDialog.at       # AlertDialog + sub-components
�?  ├── Sheet.at             # Sheet + sub-components
�?  ├── Drawer.at
�?  ├── Popover.at           # Popover, PopoverTrigger, PopoverContent
�?  ├── Tooltip.at
�?  ├── HoverCard.at         # HoverCard, HoverCardTrigger, HoverCardContent
�?  └── ContextMenu.at       # ContextMenu + sub-components
├── navigation/
�?  ├── mod.at
�?  ├── Tabs.at              # Tabs, TabsList, TabsTrigger, TabsContent
�?  ├── Breadcrumb.at        # Breadcrumb + sub-components
�?  ├── NavigationMenu.at    # NavigationMenu + sub-components
�?  ├── Pagination.at        # Pagination + sub-components
�?  ├── Sidebar.at           # Sidebar + sub-components
�?  ├── MenuBar.at           # MenuBar, MenuBarItem, MenuBarContent
�?  ├── DropdownMenu.at      # DropdownMenu + sub-components
�?  └── NavLink.at
├── feedback/
�?  ├── mod.at
�?  ├── Alert.at             # Alert, AlertTitle, AlertDescription
�?  ├── Toast.at             # Toast + sub-components
�?  ├── Progress.at
�?  └── Sonner.at            # Toast notification system
└── data/
    ├── mod.at
    ├── Table.at             # Table, TableHeader, TableBody, TableRow, TableHead, TableCell
    ├── DataTable.at         # DataTable with sorting, filtering, pagination
    └── Calendar.at
```

**Total: ~45 component files in 7 categories**

## Standardization Rules

### Prop Naming Conventions

| Prop Type | Convention | Examples |
|-----------|------------|----------|
| Primary content | `text` with `#[primary]` | `Button "Submit" {}`, `Text "Hello" {}` |
| Value/selection | `value` with `#[primary]` | `Input (value: .name) {}` |
| Visual variant | `variant: str = "default"` | `variant: "outline"`, `variant: "ghost"` |
| Size | `size: str = "md"` | `size: "sm"`, `size: "lg"` |
| Disabled state | `disabled: bool = false` | Consistent across all interactive components |
| Open/visible state | `open: bool = false` | For dialogs, sheets, popovers |
| Placeholder | `placeholder: str = ""` | For inputs, textareas, selects |
| Class/style | `class: str = ""` | Tailwind or custom classes |

### Event Naming

| Event | Pattern | Example |
|-------|---------|---------|
| Click | `onclick: .MsgName` | `Button (onclick: .Submit) {}` |
| Change | `onchange: .MsgName` | `Input (onchange: .UpdateName) {}` |
| Submit | `onsubmit: .MsgName` | `Form (onsubmit: .Save) {}` |
| Open/Close | `onopenchange: .MsgName` | `Dialog (onopenchange: .Toggle) {}` |

### Annotation Requirements

Every widget must have:

```auto
#[spec(category = <Category>, has_children = true/false, primary_prop = "<prop>")]
#[backend(ark, component = "<Component>")]
#[backend(jet, component = "<Component>", import = "<package>")]
#[backend(vue, component = "<tag>", import = "<path>")]
```

### Compound Component Pattern

All sub-widgets of a compound component go in one file:

```auto
// Dialog.at
#[spec(category = Overlay, has_children = true)]
widget Dialog {
    model { #[primary] open bool = false }
    view { ... }
}

#[spec(category = Overlay, has_children = true)]
widget DialogTrigger {
    view { ... }
}

#[spec(category = Overlay, has_children = true)]
widget DialogContent {
    view { ... }
}
```

## Migration Process

### Phase 1: Setup
1. Create new category folders: `overlay/`, `navigation/`, `feedback/`, `data/`
2. Create mod.at for each category
3. Update root `mod.at` to export all categories

### Phase 2: Migrate Components (Per Category)

**Order:**
1. `display/` �?Text, Image, Badge, Avatar, Separator, Skeleton
2. `form/` �?Button, Input, Checkbox, Switch, Select, Slider, RadioGroup, Textarea, Form
3. `layout/` �?Card, ScrollArea, AspectRatio, Collapsible, Accordion
4. `overlay/` �?Dialog, AlertDialog, Sheet, Drawer, Popover, Tooltip, HoverCard, ContextMenu
5. `navigation/` �?Tabs, Breadcrumb, NavigationMenu, Pagination, Sidebar, MenuBar, DropdownMenu, NavLink
6. `feedback/` �?Alert, Toast, Progress, Sonner
7. `data/` �?Table, DataTable, Calendar

**Per-component steps:**
1. Copy component from `component-gallery/source/front/components/*.at`
2. Remove all `*Demo` widgets
3. Apply standardization rules (rename props, add `#[primary]`)
4. Add `#[spec]` and `#[backend]` annotations
5. Add file path comment header
6. Write to target location

### Phase 3: Update Registry

Update `WidgetRegistry::with_defaults()` to include all new widgets with proper mappings.

### Phase 4: Update Generators

Ensure Ark/Jet/Vue generators handle all new component types and props correctly.

### Phase 5: Cleanup

1. Remove or repurpose `component-gallery/`
2. Update examples to use new import paths
3. Run tests

## Success Criteria

1. All 45+ components in stdlib/aura/widgets/
2. No Demo widgets in stdlib
3. Consistent prop naming across all components
4. All widgets have `#[spec]` and `#[backend]` annotations
5. Generators produce correct output for Ark/Jet/Vue
6. All tests pass

## Estimated Effort

| Phase | Time |
|-------|------|
| Phase 1: Setup | 30 min |
| Phase 2: Migrate (~45 components) | 4-6 hours |
| Phase 3: Registry | 1-2 hours |
| Phase 4: Generators | 1-2 hours |
| Phase 5: Cleanup | 30 min |
| **Total** | **7-11 hours |

## Implementation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate all 52 components from component-gallery into stdlib/aura/widgets as standardized core widgets with consistent prop naming, `#[primary]` annotations, and backend mappings.

**Architecture:** Copy component files to new category structure (overlay/, navigation/, feedback/, data/), strip Demo widgets, apply standardization rules (rename props, add `#[primary]`, add `#[spec]`/`#[backend]` annotations), update WidgetRegistry.

**Tech Stack:** AutoLang .at files, WidgetRegistry (Rust), ArkGenerator, JetGenerator, VueGenerator

---

## Phase 1: Setup New Category Structure

### Task 1.1: Create overlay/ Category

**Files:**
- Create: `stdlib/aura/widgets/overlay/mod.at`

**Step 1: Create overlay directory and mod.at**

```auto
// stdlib/aura/widgets/overlay/mod.at

// Overlay components for modals and popups
pub use Dialog, AlertDialog, Sheet, Drawer, Popover, Tooltip, HoverCard, ContextMenu
```

**Step 2: Verify file created**

Run: `ls stdlib/aura/widgets/overlay/`
Expected: `mod.at`

**Step 3: Commit**

```bash
git add stdlib/aura/widgets/overlay/mod.at
git commit -m "feat(widget): add overlay category for modal/popup widgets

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 1.2: Create navigation/ Category

**Files:**
- Create: `stdlib/aura/widgets/navigation/mod.at`

**Step 1: Create navigation directory and mod.at**

```auto
// stdlib/aura/widgets/navigation/mod.at

// Navigation components for menus and routing
pub use Tabs, Breadcrumb, NavigationMenu, Pagination, Sidebar, MenuBar, DropdownMenu, NavLink
```

**Step 2: Verify file created**

Run: `ls stdlib/aura/widgets/navigation/`
Expected: `mod.at`

**Step 3: Commit**

```bash
git add stdlib/aura/widgets/navigation/mod.at
git commit -m "feat(widget): add navigation category for menu/routing widgets

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 1.3: Create feedback/ Category

**Files:**
- Create: `stdlib/aura/widgets/feedback/mod.at`

**Step 1: Create feedback directory and mod.at**

```auto
// stdlib/aura/widgets/feedback/mod.at

// Feedback components for user notifications
pub use Alert, Toast, Progress, Sonner
```

**Step 2: Verify file created**

Run: `ls stdlib/aura/widgets/feedback/`
Expected: `mod.at`

**Step 3: Commit**

```bash
git add stdlib/aura/widgets/feedback/mod.at
git commit -m "feat(widget): add feedback category for notification widgets

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 1.4: Create data/ Category

**Files:**
- Create: `stdlib/aura/widgets/data/mod.at`

**Step 1: Create data directory and mod.at**

```auto
// stdlib/aura/widgets/data/mod.at

// Data display components
pub use Table, DataTable, Calendar
```

**Step 2: Verify file created**

Run: `ls stdlib/aura/widgets/data/`
Expected: `mod.at`

**Step 3: Commit**

```bash
git add stdlib/aura/widgets/data/mod.at
git commit -m "feat(widget): add data category for table/calendar widgets

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 1.5: Update Root mod.at

**Files:**
- Modify: `stdlib/aura/widgets/mod.at`

**Step 1: Read current mod.at**

Run: Read `stdlib/aura/widgets/mod.at`

Current content:
```auto
pub use layout: Col, Row, Stack, Scroll, Grid, Center
pub use form: Button, Input, Switch, Checkbox, Slider
pub use display: Text, Image, Icon, Progress, Divider
pub use navigation: Swiper, Tab
pub use semantic: Header, Footer, Main
```

**Step 2: Update to include new categories**

```auto
// stdlib/aura/widgets/mod.at

// Re-export all widgets for short imports
pub use layout: Col, Row, Center, Card, ScrollArea, AspectRatio, Collapsible, Accordion
pub use form: Button, Input, Checkbox, Switch, Select, Slider, RadioGroup, Textarea, Form
pub use display: Text, Image, Badge, Avatar, Separator, Skeleton
pub use overlay: Dialog, AlertDialog, Sheet, Drawer, Popover, Tooltip, HoverCard, ContextMenu
pub use navigation: Tabs, Breadcrumb, NavigationMenu, Pagination, Sidebar, MenuBar, DropdownMenu, NavLink
pub use feedback: Alert, Toast, Progress, Sonner
pub use data: Table, DataTable, Calendar
```

**Step 3: Commit**

```bash
git add stdlib/aura/widgets/mod.at
git commit -m "feat(widget): update root mod.at with all widget categories

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Phase 2: Migrate Display Components

### Task 2.1: Migrate Badge.at

**Files:**
- Read: `examples/component-gallery/source/front/components/badge.at`
- Create: `stdlib/aura/widgets/display/Badge.at`

**Step 1: Read source component**

Run: Read `examples/component-gallery/source/front/components/badge.at`

**Step 2: Create standardized Badge.at**

```auto
// stdlib/aura/widgets/display/Badge.at

#[spec(category = Display, has_children = true)]
#[backend(ark, component = "Badge")]
#[backend(jet, component = "Badge", import = "androidx.compose.material3.Badge")]
#[backend(vue, component = "span")]

/// Badge component for labels and status indicators.
///
/// # Props
/// - text: Badge text content (primary prop)
/// - variant: "default" | "secondary" | "destructive" | "outline"
///
/// # Example
/// ```auto
/// Badge "New" {}
/// Badge (text: "3", variant: "destructive") {}
/// ```
widget Badge {
    model {
        #[primary]
        text str = ""
        variant str = "default"  // default, secondary, destructive, outline
    }

    computed {
        badgeClass => f"badge badge-${.variant}"
    }

    view {
        span { class: .badgeClass, text: .text }
    }
}
```

**Step 3: Commit**

```bash
git add stdlib/aura/widgets/display/Badge.at
git commit -m "feat(widget): add Badge widget with #[primary] annotation

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 2.2: Migrate Avatar.at

**Files:**
- Read: `examples/component-gallery/source/front/components/avatar.at`
- Create: `stdlib/aura/widgets/display/Avatar.at`

**Step 1: Read source component**

Run: Read `examples/component-gallery/source/front/components/avatar.at`

**Step 2: Create standardized Avatar.at**

Apply same pattern as Badge - add `#[spec]`, `#[backend]`, `#[primary]` for src prop, remove Demo widgets.

**Step 3: Commit**

```bash
git add stdlib/aura/widgets/display/Avatar.at
git commit -m "feat(widget): add Avatar widget with standard annotations

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 2.3: Migrate Separator.at

**Files:**
- Read: `examples/component-gallery/source/front/components/separator.at`
- Create: `stdlib/aura/widgets/display/Separator.at`

Follow same pattern: read source, add annotations, remove Demo, commit.

---

### Task 2.4: Migrate Skeleton.at

**Files:**
- Read: `examples/component-gallery/source/front/components/skeleton.at`
- Create: `stdlib/aura/widgets/display/Skeleton.at`

Follow same pattern.

---

### Task 2.5: Update display/mod.at

**Files:**
- Modify: `stdlib/aura/widgets/display/mod.at`

**Step 1: Update to export all display widgets**

```auto
// stdlib/aura/widgets/display/mod.at

pub use Text, Image, Badge, Avatar, Separator, Skeleton
```

**Step 2: Commit**

```bash
git add stdlib/aura/widgets/display/mod.at
git commit -m "feat(widget): update display mod.at with all widgets

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Phase 3: Migrate Form Components

### Task 3.1: Update existing Button.at

**Files:**
- Modify: `stdlib/aura/widgets/form/Button.at`

The Button widget was already updated in the previous session with `#[primary]` annotation. Verify it's correct and commit if needed.

---

### Task 3.2: Update existing Input.at

**Files:**
- Modify: `stdlib/aura/widgets/form/Input.at`

The Input widget was already updated. Verify and commit if needed.

---

### Task 3.3: Migrate Checkbox.at

**Files:**
- Read: `examples/component-gallery/source/front/components/checkbox.at`
- Create: `stdlib/aura/widgets/form/Checkbox.at`

**Step 1: Read source**

**Step 2: Create standardized version with:**
- `#[spec(category = Form, primary_prop = "checked")]`
- `#[backend]` annotations for ark/jet/vue
- `#[primary]` on `checked` prop
- Remove Demo widgets

**Step 3: Commit**

```bash
git add stdlib/aura/widgets/form/Checkbox.at
git commit -m "feat(widget): add Checkbox widget

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 3.4: Migrate Switch.at

**Files:**
- Read: `examples/component-gallery/source/front/components/switch.at`
- Create: `stdlib/aura/widgets/form/Switch.at`

Same pattern as Checkbox.

---

### Task 3.5: Migrate Select.at

**Files:**
- Read: `examples/component-gallery/source/front/components/select.at`
- Create: `stdlib/aura/widgets/form/Select.at`

Include all sub-widgets (SelectTrigger, SelectValue, SelectContent, SelectItem, SelectGroup, SelectLabel) in one file.

---

### Task 3.6: Migrate Slider.at

**Files:**
- Read: `examples/component-gallery/source/front/components/slider.at`
- Create: `stdlib/aura/widgets/form/Slider.at`

---

### Task 3.7: Migrate RadioGroup.at

**Files:**
- Read: `examples/component-gallery/source/front/components/radiogroup.at`
- Create: `stdlib/aura/widgets/form/RadioGroup.at`

Include RadioGroupItem in same file.

---

### Task 3.8: Migrate Textarea.at

**Files:**
- Read: `examples/component-gallery/source/front/components/textarea.at`
- Create: `stdlib/aura/widgets/form/Textarea.at`

---

### Task 3.9: Migrate Form.at

**Files:**
- Read: `examples/component-gallery/source/front/components/form.at`
- Create: `stdlib/aura/widgets/form/Form.at`

Include FormField, FormLabel, FormControl, FormDescription, FormMessage in same file.

---

### Task 3.10: Update form/mod.at

**Files:**
- Modify: `stdlib/aura/widgets/form/mod.at`

```auto
// stdlib/aura/widgets/form/mod.at

pub use Button, Input, Checkbox, Switch, Select, Slider, RadioGroup, Textarea, Form
```

---

## Phase 4: Migrate Layout Components

### Task 4.1: Migrate Card.at

**Files:**
- Read: `examples/component-gallery/source/front/components/card.at`
- Create: `stdlib/aura/widgets/layout/Card.at`

Include CardHeader, CardContent, CardFooter, CardTitle, CardDescription in same file.

---

### Task 4.2: Migrate ScrollArea.at

**Files:**
- Read: `examples/component-gallery/source/front/components/scrollarea.at`
- Create: `stdlib/aura/widgets/layout/ScrollArea.at`

---

### Task 4.3: Migrate AspectRatio.at

**Files:**
- Read: `examples/component-gallery/source/front/components/aspectratio.at`
- Create: `stdlib/aura/widgets/layout/AspectRatio.at`

---

### Task 4.4: Migrate Collapsible.at

**Files:**
- Read: `examples/component-gallery/source/front/components/collapsible.at`
- Create: `stdlib/aura/widgets/layout/Collapsible.at`

Include CollapsibleTrigger, CollapsibleContent in same file.

---

### Task 4.5: Migrate Accordion.at

**Files:**
- Read: `examples/component-gallery/source/front/components/accordion.at`
- Create: `stdlib/aura/widgets/layout/Accordion.at`

Include AccordionItem, AccordionTrigger, AccordionContent in same file.

---

### Task 4.6: Update layout/mod.at

**Files:**
- Modify: `stdlib/aura/widgets/layout/mod.at`

```auto
// stdlib/aura/widgets/layout/mod.at

pub use Col, Row, Center, Card, ScrollArea, AspectRatio, Collapsible, Accordion
```

---

## Phase 5: Migrate Overlay Components

### Task 5.1: Migrate Dialog.at

**Files:**
- Read: `examples/component-gallery/source/front/components/dialog.at`
- Create: `stdlib/aura/widgets/overlay/Dialog.at`

Include all sub-widgets: DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, DialogClose in same file.

---

### Task 5.2: Migrate AlertDialog.at

**Files:**
- Read: `examples/component-gallery/source/front/components/alertdialog.at`
- Create: `stdlib/aura/widgets/overlay/AlertDialog.at`

Include all sub-widgets in same file.

---

### Task 5.3: Migrate Sheet.at

**Files:**
- Read: `examples/component-gallery/source/front/components/sheet.at`
- Create: `stdlib/aura/widgets/overlay/Sheet.at`

Include SheetTrigger, SheetContent, SheetHeader, SheetTitle, SheetDescription, SheetFooter, SheetClose in same file.

---

### Task 5.4: Migrate Drawer.at

**Files:**
- Read: `examples/component-gallery/source/front/components/drawer.at`
- Create: `stdlib/aura/widgets/overlay/Drawer.at`

---

### Task 5.5: Migrate Popover.at

**Files:**
- Read: `examples/component-gallery/source/front/components/popover.at`
- Create: `stdlib/aura/widgets/overlay/Popover.at`

Include PopoverTrigger, PopoverContent in same file.

---

### Task 5.6: Migrate Tooltip.at

**Files:**
- Read: `examples/component-gallery/source/front/components/tooltip.at`
- Create: `stdlib/aura/widgets/overlay/Tooltip.at`

Include TooltipTrigger, TooltipContent in same file.

---

### Task 5.7: Migrate HoverCard.at

**Files:**
- Read: `examples/component-gallery/source/front/components/hovercard.at`
- Create: `stdlib/aura/widgets/overlay/HoverCard.at`

Include HoverCardTrigger, HoverCardContent in same file.

---

### Task 5.8: Migrate ContextMenu.at

**Files:**
- Read: `examples/component-gallery/source/front/components/contextmenu.at`
- Create: `stdlib/aura/widgets/overlay/ContextMenu.at`

Include all sub-widgets in same file.

---

### Task 5.9: Update overlay/mod.at

**Files:**
- Modify: `stdlib/aura/widgets/overlay/mod.at`

```auto
// stdlib/aura/widgets/overlay/mod.at

pub use Dialog, AlertDialog, Sheet, Drawer, Popover, Tooltip, HoverCard, ContextMenu
```

---

## Phase 6: Migrate Navigation Components

### Task 6.1: Migrate Tabs.at

**Files:**
- Read: `examples/component-gallery/source/front/components/tabs.at`
- Create: `stdlib/aura/widgets/navigation/Tabs.at`

Include TabsList, TabsTrigger, TabsContent in same file.

---

### Task 6.2: Migrate Breadcrumb.at

**Files:**
- Read: `examples/component-gallery/source/front/components/breadcrumb.at`
- Create: `stdlib/aura/widgets/navigation/Breadcrumb.at`

Include all sub-widgets in same file.

---

### Task 6.3: Migrate NavigationMenu.at

**Files:**
- Read: `examples/component-gallery/source/front/components/navigationmenu.at`
- Create: `stdlib/aura/widgets/navigation/NavigationMenu.at`

---

### Task 6.4: Migrate Pagination.at

**Files:**
- Read: `examples/component-gallery/source/front/components/pagination.at`
- Create: `stdlib/aura/widgets/navigation/Pagination.at`

---

### Task 6.5: Migrate Sidebar.at

**Files:**
- Read: `examples/component-gallery/source/front/components/sidebar.at`
- Create: `stdlib/aura/widgets/navigation/Sidebar.at`

---

### Task 6.6: Migrate MenuBar.at

**Files:**
- Read: `examples/component-gallery/source/front/components/menubar.at`
- Create: `stdlib/aura/widgets/navigation/MenuBar.at`

---

### Task 6.7: Migrate DropdownMenu.at

**Files:**
- Read: `examples/component-gallery/source/front/components/dropdownmenu.at`
- Create: `stdlib/aura/widgets/navigation/DropdownMenu.at`

---

### Task 6.8: Migrate NavLink.at

**Files:**
- Read: `examples/component-gallery/source/front/components/nav_link.at`
- Create: `stdlib/aura/widgets/navigation/NavLink.at`

---

### Task 6.9: Update navigation/mod.at

**Files:**
- Modify: `stdlib/aura/widgets/navigation/mod.at`

```auto
// stdlib/aura/widgets/navigation/mod.at

pub use Tabs, Breadcrumb, NavigationMenu, Pagination, Sidebar, MenuBar, DropdownMenu, NavLink
```

---

## Phase 7: Migrate Feedback Components

### Task 7.1: Migrate Alert.at

**Files:**
- Read: `examples/component-gallery/source/front/components/alert.at`
- Create: `stdlib/aura/widgets/feedback/Alert.at`

Include AlertTitle, AlertDescription in same file.

---

### Task 7.2: Migrate Toast.at

**Files:**
- Read: `examples/component-gallery/source/front/components/toast.at`
- Create: `stdlib/aura/widgets/feedback/Toast.at`

Include ToastProvider, ToastViewport, ToastTitle, ToastDescription, ToastAction, ToastClose in same file.

---

### Task 7.3: Migrate Progress.at

**Files:**
- Read: `examples/component-gallery/source/front/components/progress.at`
- Create: `stdlib/aura/widgets/feedback/Progress.at`

---

### Task 7.4: Migrate Sonner.at

**Files:**
- Read: `examples/component-gallery/source/front/components/sonner.at`
- Create: `stdlib/aura/widgets/feedback/Sonner.at`

---

### Task 7.5: Update feedback/mod.at

**Files:**
- Modify: `stdlib/aura/widgets/feedback/mod.at`

```auto
// stdlib/aura/widgets/feedback/mod.at

pub use Alert, Toast, Progress, Sonner
```

---

## Phase 8: Migrate Data Components

### Task 8.1: Migrate Table.at

**Files:**
- Read: `examples/component-gallery/source/front/components/table.at`
- Create: `stdlib/aura/widgets/data/Table.at`

Include TableHeader, TableBody, TableRow, TableHead, TableCell in same file.

---

### Task 8.2: Migrate DataTable.at

**Files:**
- Read: `examples/component-gallery/source/front/components/datatable.at`
- Create: `stdlib/aura/widgets/data/DataTable.at`

---

### Task 8.3: Migrate Calendar.at

**Files:**
- Read: `examples/component-gallery/source/front/components/calendar.at`
- Create: `stdlib/aura/widgets/data/Calendar.at`

---

### Task 8.4: Update data/mod.at

**Files:**
- Modify: `stdlib/aura/widgets/data/mod.at`

```auto
// stdlib/aura/widgets/data/mod.at

pub use Table, DataTable, Calendar
```

---

## Phase 9: Update WidgetRegistry

### Task 9.1: Add display widgets to registry

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

Add Badge, Avatar, Separator, Skeleton to `register_display_widgets()`.

---

### Task 9.2: Add form widgets to registry

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

Add Checkbox, Switch, Select, Slider, RadioGroup, Textarea to `register_form_widgets()`.

---

### Task 9.3: Add layout widgets to registry

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

Add Card, ScrollArea, AspectRatio, Collapsible, Accordion to `register_layout_widgets()`.

---

### Task 9.4: Add overlay widgets to registry

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

Create `register_overlay_widgets()` function and add Dialog, AlertDialog, Sheet, Drawer, Popover, Tooltip, HoverCard, ContextMenu.

---

### Task 9.5: Add navigation widgets to registry

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

Create `register_navigation_widgets()` function and add Tabs, Breadcrumb, NavigationMenu, Pagination, Sidebar, MenuBar, DropdownMenu, NavLink.

---

### Task 9.6: Add feedback widgets to registry

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

Create `register_feedback_widgets()` function and add Alert, Toast, Progress, Sonner.

---

### Task 9.7: Add data widgets to registry

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

Create `register_data_widgets()` function and add Table, DataTable, Calendar.

---

### Task 9.8: Update register_defaults()

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

Call all new register functions in `register_defaults()`:

```rust
fn register_defaults(&mut self) {
    self.register_layout_widgets();
    self.register_form_widgets();
    self.register_display_widgets();
    self.register_overlay_widgets();    // NEW
    self.register_navigation_widgets(); // NEW
    self.register_feedback_widgets();   // NEW
    self.register_data_widgets();       // NEW
    self.register_semantic_widgets();
}
```

---

### Task 9.9: Commit registry changes

```bash
git add crates/auto-lang/src/ui_gen/widget/registry.rs
git commit -m "feat(widget): register all stdlib widgets in WidgetRegistry

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Phase 10: Verify and Test

### Task 10.1: Run compilation check

Run: `cargo check -p auto-lang`
Expected: No errors

---

### Task 10.2: Run widget registry tests

Run: `cargo test -p auto-lang --lib widget`
Expected: All tests pass

---

### Task 10.3: Run Ark generator tests

Run: `cargo test -p auto-lang --lib ark`
Expected: All tests pass (or document failures)

---

### Task 10.4: Final commit

```bash
git add -A
git commit -m "feat(widget): complete stdlib widget library migration

- Migrate 45+ components from component-gallery to stdlib/aura/widgets
- Add #[primary] annotations for shorthand syntax
- Add #[spec] and #[backend] annotations for all widgets
- Organize into 7 categories: display, form, layout, overlay, navigation, feedback, data
- Update WidgetRegistry with all new widgets

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 5 | Setup category structure |
| 2 | 5 | Migrate display components (Badge, Avatar, Separator, Skeleton) |
| 3 | 10 | Migrate form components (Checkbox, Switch, Select, Slider, RadioGroup, Textarea, Form) |
| 4 | 6 | Migrate layout components (Card, ScrollArea, AspectRatio, Collapsible, Accordion) |
| 5 | 9 | Migrate overlay components (Dialog, AlertDialog, Sheet, Drawer, Popover, Tooltip, HoverCard, ContextMenu) |
| 6 | 9 | Migrate navigation components (Tabs, Breadcrumb, NavigationMenu, Pagination, Sidebar, MenuBar, DropdownMenu, NavLink) |
| 7 | 5 | Migrate feedback components (Alert, Toast, Progress, Sonner) |
| 8 | 4 | Migrate data components (Table, DataTable, Calendar) |
| 9 | 9 | Update WidgetRegistry with all widgets |
| 10 | 4 | Verify and test |

**Total: 66 tasks**
