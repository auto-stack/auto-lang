# Stdlib Widget Library Design

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
│   ├── mod.at
│   ├── Text.at              # Text, H1-H6, Paragraph, Span, Label
│   ├── Image.at
│   ├── Badge.at
│   ├── Avatar.at
│   ├── Separator.at
│   └── Skeleton.at
├── form/
│   ├── mod.at
│   ├── Button.at            # Button with variants
│   ├── Input.at             # Input, Textarea
│   ├── Checkbox.at
│   ├── Switch.at
│   ├── Select.at            # Select, Combobox
│   ├── Slider.at
│   ├── RadioGroup.at
│   └── Form.at              # Form, FormField, FormLabel, FormControl
├── layout/
│   ├── mod.at
│   ├── Col.at
│   ├── Row.at
│   ├── Center.at
│   ├── Card.at              # Card, CardHeader, CardContent, CardFooter
│   ├── ScrollArea.at
│   ├── AspectRatio.at
│   ├── Collapsible.at
│   └── Accordion.at         # Accordion, AccordionItem, AccordionTrigger, AccordionContent
├── overlay/
│   ├── mod.at
│   ├── Dialog.at            # Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter
│   ├── AlertDialog.at       # AlertDialog + sub-components
│   ├── Sheet.at             # Sheet + sub-components
│   ├── Drawer.at
│   ├── Popover.at           # Popover, PopoverTrigger, PopoverContent
│   ├── Tooltip.at
│   ├── HoverCard.at         # HoverCard, HoverCardTrigger, HoverCardContent
│   └── ContextMenu.at       # ContextMenu + sub-components
├── navigation/
│   ├── mod.at
│   ├── Tabs.at              # Tabs, TabsList, TabsTrigger, TabsContent
│   ├── Breadcrumb.at        # Breadcrumb + sub-components
│   ├── NavigationMenu.at    # NavigationMenu + sub-components
│   ├── Pagination.at        # Pagination + sub-components
│   ├── Sidebar.at           # Sidebar + sub-components
│   ├── MenuBar.at           # MenuBar, MenuBarItem, MenuBarContent
│   ├── DropdownMenu.at      # DropdownMenu + sub-components
│   └── NavLink.at
├── feedback/
│   ├── mod.at
│   ├── Alert.at             # Alert, AlertTitle, AlertDescription
│   ├── Toast.at             # Toast + sub-components
│   ├── Progress.at
│   └── Sonner.at            # Toast notification system
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
1. `display/` → Text, Image, Badge, Avatar, Separator, Skeleton
2. `form/` → Button, Input, Checkbox, Switch, Select, Slider, RadioGroup, Textarea, Form
3. `layout/` → Card, ScrollArea, AspectRatio, Collapsible, Accordion
4. `overlay/` → Dialog, AlertDialog, Sheet, Drawer, Popover, Tooltip, HoverCard, ContextMenu
5. `navigation/` → Tabs, Breadcrumb, NavigationMenu, Pagination, Sidebar, MenuBar, DropdownMenu, NavLink
6. `feedback/` → Alert, Toast, Progress, Sonner
7. `data/` → Table, DataTable, Calendar

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
