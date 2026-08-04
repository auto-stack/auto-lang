# Plan 104: Add shadcn-vue Components

## Objective

Add full support for all shadcn-vue components in AutoLang, enabling developers to use any shadcn-vue component by writing Auto (AURA) code that transpiles to Vue + shadcn-vue.

## IMPORTANT: Implementation Process

**When implementing each component, you MUST:**

1. **Use MCP tools to scrape the shadcn-vue documentation**
   - Use `mcp__web_reader__webReader` to fetch content from `https://www.shadcn-vue.com/docs/components/<component-name>`
   - Extract: component props, events, slots, usage examples, and code snippets

2. **Study the official examples carefully**
   - Understand all variants (default, outline, ghost, etc.)
   - Note all sub-components (e.g., Card has CardHeader, CardTitle, CardContent, CardFooter)
   - Identify which props are required vs optional
   - Check for special attributes like `v-model`, `@click`, etc.

3. **Reimplement in Auto code**
   - Create equivalent AURA syntax that generates the same Vue code
   - Add to AURA schema (`schema.rs`)
   - Add to Vue generator (`vue.rs`)
   - Create example page in `component-gallery/source/front/pages/<component>.at`

4. **Verify output matches shadcn-vue patterns**
   - Generated Vue code should match the patterns from the documentation
   - All imports should be correct
   - All props and events should work

## Current State

### Implemented Components
- Button (`button`)
- Table (`table`)
- Input (basic, through HTML elements)

### shadcn-vue Components to Add

Based on https://www.shadcn-vue.com/docs/components/, we need to add support for:

#### Layout Components
1. **Accordion** - Collapsible content panels
2. **Card** - Container with header, content, footer sections
3. **Collapsible** - Single collapsible section
4. **Dialog** - Modal dialog box
5. **Drawer** - Side panel that slides in
6. **Resizable** - Resizable panel groups
7. **Scroll Area** - Scrollable viewport
8. **Separator** - Visual divider
9. **Sheet** - Slide-up modal
10. **Sidebar** - Side navigation
11. **Tabs** - Tabbed content areas

#### Form Components
12. **Button** ✅ (already implemented)
13. **Checkbox** - Checkbox input
14. **Combobox** - Combination of input and dropdown
15. **Date Picker** - Date selection calendar
16. **Form** - Form wrapper with validation
17. **Input** - Text input field
18. **Input OTP** - One-time password input
19. **Label** - Form field label
20. **Native Select** - Native dropdown
21. **Number Field** - Number input
22. **Pin Input** - PIN code input
23. **Radio Group** - Radio button group
24. **Select** - Custom dropdown select
25. **Slider** - Range slider
26. **Switch** - Toggle switch
27. **Textarea** - Multi-line text input
28. **Toggle** - Simple toggle
29. **Toggle Group** - Group of toggles

#### Data Display Components
30. **Avatar** - User avatar image
31. **Badge** - Status badge/label
32. **Breadcrumb** - Navigation path
33. **Calendar** - Calendar widget
34. **Chart** - Chart/graph display
35. **Command** - Command palette (Cmd+K)
36. **Data Table** - Advanced table with features
37. **Pagination** - Page navigation
38. **Progress** - Progress bar
39. **Skeleton** - Loading placeholder
40. **Table** ✅ (already implemented)
41. **Tags Input** - Tag/chip input

#### Feedback Components
42. **Alert** - Alert notification
43. **Aspect Ratio** - Aspect ratio container
44. **Hover Card** - Card that appears on hover
45. **Popover** - Popup content
46. **Sonner** - Toast notifications
47. **Toast** - Toast messages
48. **Tooltip** - Hover tooltip

#### Navigation Components
49. **Breadcrumb** - Navigation path (duplicate of #32)
50. **Dropdown Menu** - Dropdown navigation menu
51. **Menubar** - Horizontal menu bar
52. **Navigation Menu** - Vertical/horizontal nav
53. **Pagination** - Page navigation (duplicate of #37)

#### Overlay Components
54. **Alert Dialog** - Confirmation dialog
55. **Context Menu** - Right-click menu
56. **Dialog** - Modal dialog (duplicate of #4)
57. **Drawer** - Side panel (duplicate of #5)
58. **Hover Card** - Hover popup (duplicate of #44)
59. **Popover** - Popup (duplicate of #45)
60. **Sheet** - Slide-up modal (duplicate of #10)
61. **Tooltip** - Hover tooltip (duplicate of #48)

#### Misc Components
62. **Carousel** - Image/content carousel
63. **Empty** - Empty state placeholder
64. **Kbd** - Keyboard key display
65. **Spinner** - Loading spinner
66. **Typography** - Text styling

## Implementation Approach

For each component, we need to:

### 1. Add to AURA Schema (`crates/auto-lang/src/aura/schema.rs`)

```rust
// Example for Card
ElementDef {
    tag: "card",
    html_tag: "div",
    props: vec![
        PropDef { name: "variant", type_: PropType::OneOf(vec!["default", "outline"]), required: false, ... },
    ],
    slots: vec!["header", "content", "footer"],
    description: "Card container with sections",
}
```

### 2. Add Vue Generation (`crates/auto-lang/src/ui_gen/vue.rs`)

#### In `ShadcnRegistry`:
```rust
components: HashMap<&'static str, (&'static str, Vec<&'static str>)>
```

#### In `tag_to_html`:
```rust
"card" => "div".to_string(),
"cardheader" => "div".to_string(),
// etc.
```

#### In `extract_classes`:
```rust
// Card variants
"card" => "rounded-lg border bg-card text-card-foreground shadow-sm",
"cardheader" => "flex flex-col space-y-1.5 p-6",
// etc.
```

### 3. Add to Component Gallery (`examples/component-gallery/source/front/`)

Create example pages showing:
- Basic usage
- All variants
- Common patterns

## Phase 1: Core Components (High Priority)

These are the most commonly used components:

1. **Card** - Essential for layouts
2. **Input** - Essential for forms
3. **Dialog** - Essential for interactions
4. **Badge** - Common for status display
5. **Tabs** - Common for content organization
6. **Checkbox** - Common for forms
7. **Switch** - Common for toggles
8. **Label** - Essential for forms
9. **Textarea** - Common for forms
10. **Separator** - Common for layouts

## Phase 2: Form Components (Medium Priority)

11. **Select** - Dropdown selection
12. **Combobox** - Search + select
13. **Slider** - Range input
14. **Radio Group** - Radio buttons
15. **Date Picker** - Date selection
16. **Form** - Form validation wrapper

## Phase 3: Layout & Navigation (Medium Priority)

17. **Accordion** - Collapsible sections
18. **Breadcrumb** - Navigation path
19. **Dropdown Menu** - Menu dropdowns
20. **Sidebar** - Side navigation
21. **Pagination** - Page navigation
22. **Scroll Area** - Custom scrollbars

## Phase 4: Feedback & Overlay (Lower Priority)

23. **Alert** - Alert messages
24. **Toast** - Toast notifications
25. **Tooltip** - Hover tooltips
26. **Popover** - Popup content
27. **Hover Card** - Hover cards
28. **Drawer** - Side drawer
29. **Sheet** - Slide-up modal

## Phase 5: Data Display & Misc (Lower Priority)

30. **Avatar** - User avatars
31. **Skeleton** - Loading states
32. **Progress** - Progress bars
33. **Spinner** - Loading spinners
34. **Calendar** - Calendar widget
35. **Carousel** - Content carousel
36. **Command** - Command palette
37. **Tags Input** - Tag input
38. **Empty** - Empty states

## Implementation Steps

### Step 1: Add Component to AURA Schema

File: `crates/auto-lang/src/aura/schema.rs`

1. Define props for the component
2. Define events if applicable
3. Define slots for compound components
4. Add to the ELEMENTS hashmap

### Step 2: Add Vue Generation Support

File: `crates/auto-lang/src/ui_gen/vue.rs`

1. Add to `ShadcnRegistry::new()`
2. Add tag mapping in `tag_to_html()`
3. Add default classes in `extract_classes()`
4. Handle special cases in `node_to_html()`

### Step 3: Create Example Page

File: `examples/component-gallery/source/front/pages/{component}.at`

1. Import the component
2. Show basic usage
3. Show all variants
4. Show common patterns
5. Add to navigation in `app.at`

## Example Implementation: Card Component

### AURA Schema Addition

```rust
// In schema.rs
let mut card_props = Vec::new();
card_props.push(PropDef {
    name: "variant".to_string(),
    type_: PropType::OneOf(vec!["default".to_string(), "outline".to_string()]),
    required: false,
    default: None,
    description: "Card variant style".to_string(),
});

elements.insert("card", ElementDef {
    tag: "card",
    html_tag: "div",
    props: card_props,
    events: vec![],
    slots: vec!["header", "content", "footer"],
    description: "Card container with sections",
});
```

### Vue Generator Addition

```rust
// In ShadcnRegistry
self.components.insert("card", ("@/components/ui/card", vec!["Card", "CardHeader", "CardTitle", "CardDescription", "CardContent", "CardFooter"]));

// In tag_to_html
"card" => "Card".to_string(),
"cardheader" => "CardHeader".to_string(),
"cardtitle" => "CardTitle".to_string(),
"carddescription" => "CardDescription".to_string(),
"cardcontent" => "CardContent".to_string(),
"cardfooter" => "CardFooter".to_string(),
```

### Example Page

```auto
// pages/card.at - Card component documentation page

widget CardPage {
    msg Msg { TabChange(tab: str) }

    model {
        activeTab str = "preview"
    }

    view {
        col {
            Breadcrumb {
                items: [
                    { text: "Docs", href: "/" },
                    { text: "Components", href: "/components" },
                    { text: "Card", href: "/components/card" }
                ]
            }

            h1 (text: "Card") {}
            Paragraph (content: "A card component with header, content, and footer sections.") {}

            h2 (text: "Examples") {}

            // Basic Card
            previewcard (id: "BasicCard") {
                card {
                    cardheader {
                        cardtitle (text: "Card Title") {}
                        carddescription (text: "Card description") {}
                    }
                    cardcontent {
                        text "Card content goes here"
                    }
                    cardfooter {
                        text "Card footer"
                    }
                }
            }
        }
    }
}
```

## Testing Strategy

1. **Unit Tests**: Add tests for each component's AURA parsing
2. **Integration Tests**: Add a2v tests for Vue generation
3. **Visual Tests**: Manual testing in component gallery
4. **Documentation**: Each component page in the gallery serves as documentation

## Success Criteria

1. All 38+ components are supported in AURA schema
2. All components generate correct Vue/shadcn-vue code
3. Component gallery has a page for each component
4. All example pages work correctly in the generated Vue app
5. No missing imports or components in generated code

## Estimated Effort

- Phase 1 (10 components): ~2-3 hours
- Phase 2 (6 components): ~1-2 hours
- Phase 3 (6 components): ~1-2 hours
- Phase 4 (7 components): ~1-2 hours
- Phase 5 (8+ components): ~2 hours

Total: ~8-12 hours of work

## Dependencies

- shadcn-vue must be installed (handled by `auto.exe vue` command)
- All shadcn-vue components are available via `npx shadcn-vue@latest add <component>`

## Notes

1. **Compound Components**: Components like Card, Dialog, etc. have sub-components (CardHeader, DialogContent, etc.) that need to be handled together

2. **State Management**: Some components like Dialog, Sheet, Drawer require state management (open/closed) - may need special handling

3. **Icons**: Some components require icons (Breadcrumb, Command, etc.) - need to handle icon imports

4. **Accessibility**: Ensure generated code maintains shadadn-vue's accessibility features

## MCP Scraping Workflow

### Step 1: Fetch Component Documentation

```bash
# Use mcp__web_reader__webReader to fetch the component page
# Example for Card component:
# URL: https://www.shadcn-vue.com/docs/components/card
```

### Step 2: Extract Key Information

From the fetched content, extract:

1. **Component Name & Description**
2. **All Props** (name, type, default, required)
3. **Events** (event name, parameters)
4. **Slots** (slot name, purpose)
5. **Sub-components** (if compound component)
6. **Usage Examples** (Vue code snippets)
7. **Variants** (e.g., default, outline, destructive for Button)

### Step 3: Map to AURA Schema

Convert the extracted information to AURA schema format:

```rust
// Example mapping from shadcn-vue Card to AURA
// shadcn-vue: <Card>, <CardHeader>, <CardTitle>, <CardDescription>, <CardContent>, <CardFooter>
// AURA: card, cardheader, cardtitle, carddescription, cardcontent, cardfooter
```

### Step 4: Implement Vue Generator

Add mappings in `vue.rs`:

1. **ShadcnRegistry** - Map tag to import path and component names
2. **tag_to_html** - Map AURA tag to Vue component name
3. **extract_classes** - Add default Tailwind classes
4. **Special handling** - For v-model, events, etc.

### Step 5: Create Example Page

Create `component-gallery/source/front/pages/<component>.at` that demonstrates:
- Basic usage
- All variants
- Common patterns
- Interactive examples

## Next Steps

1. Start with Phase 1 components (Card, Input, Dialog, Badge, Tabs, etc.)
2. Use MCP tools to fetch shadcn-vue documentation for each component
3. Implement AURA schema changes based on extracted info
4. Implement Vue generator changes
5. Create example pages that match shadcn-vue's examples
6. Test and iterate

## Component URL Reference

| Component | Documentation URL | Status |
|-----------|-------------------|--------|
| Accordion | https://www.shadcn-vue.com/docs/components/accordion | ❌ Not started |
| Alert | https://www.shadcn-vue.com/docs/components/alert | ❌ Not started |
| Alert Dialog | https://www.shadcn-vue.com/docs/components/alert-dialog | ❌ Not started |
| Aspect Ratio | https://www.shadcn-vue.com/docs/components/aspect-ratio | ❌ Not started |
| Avatar | https://www.shadcn-vue.com/docs/components/avatar | ❌ Not started |
| Badge | https://www.shadcn-vue.com/docs/components/badge | ❌ Not started |
| Breadcrumb | https://www.shadcn-vue.com/docs/components/breadcrumb | ❌ Not started |
| Button | https://www.shadcn-vue.com/docs/components/button | ✅ Implemented |
| Calendar | https://www.shadcn-vue.com/docs/components/calendar | ❌ Not started |
| Card | https://www.shadcn-vue.com/docs/components/card | ❌ Not started |
| Carousel | https://www.shadcn-vue.com/docs/components/carousel | ❌ Not started |
| Chart | https://www.shadcn-vue.com/docs/components/chart | ❌ Not started |
| Checkbox | https://www.shadcn-vue.com/docs/components/checkbox | ❌ Not started |
| Collapsible | https://www.shadcn-vue.com/docs/components/collapsible | ❌ Not started |
| Combobox | https://www.shadcn-vue.com/docs/components/combobox | ❌ Not started |
| Command | https://www.shadcn-vue.com/docs/components/command | ❌ Not started |
| Context Menu | https://www.shadcn-vue.com/docs/components/context-menu | ❌ Not started |
| Data Table | https://www.shadcn-vue.com/docs/components/data-table | ❌ Not started |
| Date Picker | https://www.shadcn-vue.com/docs/components/date-picker | ❌ Not started |
| Dialog | https://www.shadcn-vue.com/docs/components/dialog | ❌ Not started |
| Drawer | https://www.shadcn-vue.com/docs/components/drawer | ❌ Not started |
| Dropdown Menu | https://www.shadcn-vue.com/docs/components/dropdown-menu | ❌ Not started |
| Empty | https://www.shadcn-vue.com/docs/components/empty | ❌ Not started |
| Form | https://www.shadcn-vue.com/docs/components/form | ❌ Not started |
| Hover Card | https://www.shadcn-vue.com/docs/components/hover-card | ❌ Not started |
| Input | https://www.shadcn-vue.com/docs/components/input | ✅ Partial (via HTML) |
| Input OTP | https://www.shadcn-vue.com/docs/components/input-otp | ❌ Not started |
| Kbd | https://www.shadcn-vue.com/docs/components/kbd | ❌ Not started |
| Label | https://www.shadcn-vue.com/docs/components/label | ❌ Not started |
| Menubar | https://www.shadcn-vue.com/docs/components/menubar | ❌ Not started |
| Navigation Menu | https://www.shadcn-vue.com/docs/components/navigation-menu | ❌ Not started |
| Number Field | https://www.shadcn-vue.com/docs/components/number-field | ❌ Not started |
| Pagination | https://www.shadcn-vue.com/docs/components/pagination | ❌ Not started |
| Pin Input | https://www.shadcn-vue.com/docs/components/pin-input | ❌ Not started |
| Popover | https://www.shadcn-vue.com/docs/components/popover | ❌ Not started |
| Progress | https://www.shadcn-vue.com/docs/components/progress | ❌ Not started |
| Radio Group | https://www.shadcn-vue.com/docs/components/radio-group | ❌ Not started |
| Range Calendar | https://www.shadcn-vue.com/docs/components/range-calendar | ❌ Not started |
| Resizable | https://www.shadcn-vue.com/docs/components/resizable | ❌ Not started |
| Scroll Area | https://www.shadcn-vue.com/docs/components/scroll-area | ❌ Not started |
| Select | https://www.shadcn-vue.com/docs/components/select | ❌ Not started |
| Separator | https://www.shadcn-vue.com/docs/components/separator | ❌ Not started |
| Sheet | https://www.shadcn-vue.com/docs/components/sheet | ❌ Not started |
| Sidebar | https://www.shadcn-vue.com/docs/components/sidebar | ❌ Not started |
| Skeleton | https://www.shadcn-vue.com/docs/components/skeleton | ❌ Not started |
| Slider | https://www.shadcn-vue.com/docs/components/slider | ❌ Not started |
| Sonner | https://www.shadcn-vue.com/docs/components/sonner | ❌ Not started |
| Spinner | https://www.shadcn-vue.com/docs/components/spinner | ❌ Not started |
| Stepper | https://www.shadcn-vue.com/docs/components/stepper | ❌ Not started |
| Switch | https://www.shadcn-vue.com/docs/components/switch | ❌ Not started |
| Table | https://www.shadcn-vue.com/docs/components/table | ✅ Implemented |
| Tabs | https://www.shadcn-vue.com/docs/components/tabs | ❌ Not started |
| Tags Input | https://www.shadcn-vue.com/docs/components/tags-input | ❌ Not started |
| Textarea | https://www.shadcn-vue.com/docs/components/textarea | ❌ Not started |
| Toast | https://www.shadcn-vue.com/docs/components/toast | ❌ Not started |
| Toggle | https://www.shadcn-vue.com/docs/components/toggle | ❌ Not started |
| Toggle Group | https://www.shadcn-vue.com/docs/components/toggle-group | ❌ Not started |
| Tooltip | https://www.shadcn-vue.com/docs/components/tooltip | ❌ Not started |
