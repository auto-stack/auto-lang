# Plan 099: shadcn-vue Migration

## Overview

Migrate the Vue.js code generator from plain Tailwind CSS to shadcn-vue components, providing a polished, accessible UI component library out of the box.

## Goals

1. Generate Vue 3 components using shadcn-vue primitives
2. Support all 43 AURA schema elements
3. Provide complete, working components (not just styled HTML)
4. Ensure accessibility via Radix Vue primitives
5. Generate necessary project setup files

## Current State vs Target State

### Before (Plain Tailwind)
```vue
<template>
  <button class="px-4 py-2 rounded bg-blue-500 text-white">
    Click me
  </button>
  <div class="fixed inset-0 bg-black/50">
    <div class="bg-white p-4 rounded">Modal content</div>
  </div>
</template>
```

### After (shadcn-vue)
```vue
<script setup>
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogTrigger } from '@/components/ui/dialog'
</script>

<template>
  <Button>Click me</Button>
  <Dialog>
    <DialogContent>Modal content</DialogContent>
  </Dialog>
</template>
```

## Element Mapping

### Layout Elements (5)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `col` | `<div class="flex flex-col">` | Use Tailwind (no component) |
| `row` | `<div class="flex flex-row">` | Use Tailwind (no component) |
| `grid` | `<div class="grid">` | Use Tailwind (no component) |
| `scroll` | `<ScrollArea>` | shadcn-vue ScrollArea |
| `container` | `<div class="container">` | Use Tailwind (no component) |

### Content Elements (8)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `button` | `<Button>` | Full component |
| `input` | `<Input>` | Full component |
| `textarea` | `<Textarea>` | Full component |
| `checkbox` | `<Checkbox>` | Full component |
| `toggle` | `<Switch>` | Full component |
| `select` | `<Select>`, `<SelectContent>`, `<SelectItem>` | Full component set |
| `option` | `<SelectItem>` | Part of Select |
| `link` | `<a class="...">` | Use Tailwind |

### Data Elements (8)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `table` | `<Table>` | Full component |
| `thead` | `<TableHeader>` | Full component |
| `tbody` | `<TableBody>` | Full component |
| `tr` | `<TableRow>` | Full component |
| `th` | `<TableHead>` | Full component |
| `td` | `<TableCell>` | Full component |
| `tree` | Custom with `<Collapsible>` | Build from primitives |
| `tree_item` | Custom with `<CollapsibleItem>` | Build from primitives |

### Navigation Elements (2)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `tabs` | `<Tabs>`, `<TabsList>` | Full component set |
| `tab` | `<TabsTrigger>`, `<TabsContent>` | Full component set |

### Overlay Elements (2)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `modal` | `<Dialog>`, `<DialogContent>` | Full component set |
| `tooltip` | `<Tooltip>`, `<TooltipContent>` | Full component set |

### Form Elements (3)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `slider` | `<Slider>` | Full component |
| `radio` | `<RadioGroup>`, `<RadioGroupItem>` | Full component set |
| `radiogroup` | `<RadioGroup>` | Container for radios |

### Feedback Elements (3)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `progress` | `<Progress>` | Full component |
| `badge` | `<Badge>` | Full component |
| `spinner` | `<Skeleton>` or custom | Use loading state |

### Display Elements (2)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `card` | `<Card>`, `<CardHeader>`, `<CardContent>` | Full component set |
| `avatar` | `<Avatar>`, `<AvatarImage>`, `<AvatarFallback>` | Full component set |

### Typography Elements (5)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `text` | `<span>` | Use Tailwind |
| `h1`-`h6` | `<h1>`-`<h6>` | Use Tailwind typography |
| `p` | `<p>` | Use Tailwind |
| `span` | `<span>` | Use Tailwind |

### Media Elements (2)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `image` | `<img>` | Use Tailwind |
| `icon` | Custom or `<Icon>` | Integrate with icon library |

### Utility Elements (2)

| AURA Tag | shadcn-vue Component | Notes |
|----------|---------------------|-------|
| `divider` | `<Separator>` | Full component |
| `spacer` | `<div class="flex-1">` | Use Tailwind |

## Implementation Phases

### Phase 1: Setup Infrastructure (Week 1)

- [ ] Create component import mapping table
- [ ] Update `VueGenerator` struct with shadcn-vue mode
- [ ] Add component registry for tracking imports
- [ ] Generate `components.json` config file
- [ ] Create project scaffold template (package.json, vite.config, etc.)

### Phase 2: Core Components (Week 1-2)

- [ ] Implement `button` → `<Button>` mapping
- [ ] Implement `input` → `<Input>` mapping
- [ ] Implement `textarea` → `<Textarea>` mapping
- [ ] Implement `checkbox` → `<Checkbox>` mapping
- [ ] Implement `toggle` → `<Switch>` mapping
- [ ] Implement `select/option` → `<Select>` mapping

### Phase 3: Layout & Navigation (Week 2)

- [ ] Implement `scroll` → `<ScrollArea>` mapping
- [ ] Implement `tabs/tab` → `<Tabs>` mapping
- [ ] Implement `card` → `<Card>` mapping
- [ ] Implement `divider` → `<Separator>` mapping

### Phase 4: Overlay & Feedback (Week 2-3)

- [ ] Implement `modal` → `<Dialog>` mapping
- [ ] Implement `tooltip` → `<Tooltip>` mapping
- [ ] Implement `progress` → `<Progress>` mapping
- [ ] Implement `badge` → `<Badge>` mapping
- [ ] Implement `spinner` → loading state

### Phase 5: Data Components (Week 3)

- [ ] Implement `table/thead/tbody/tr/th/td` → `<Table>` mapping
- [ ] Implement `tree/tree_item` → custom tree component
- [ ] Implement `avatar` → `<Avatar>` mapping

### Phase 6: Form Components (Week 3)

- [ ] Implement `slider` → `<Slider>` mapping
- [ ] Implement `radio/radiogroup` → `<RadioGroup>` mapping
- [ ] Add form validation integration

### Phase 7: Migration & Testing (Week 4)

- [ ] Migrate TodoMVC example to shadcn-vue
- [ ] Create component showcase/demo page
- [ ] Write integration tests for generated components
- [ ] Update documentation

## Generated Project Structure

```
generated-widget/
├── src/
│   ├── components/
│   │   └── ui/              # shadcn-vue components
│   │       ├── button/
│   │       ├── input/
│   │       ├── dialog/
│   │       └── ...
│   ├── lib/
│   │   └── utils.ts         # cn() helper
│   ├── App.vue
│   └── main.ts
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── components.json          # shadcn-vue config
└── tsconfig.json
```

## Generated Component Example

### Input (AURA)
```auto
widget TextInput {
    msg Msg { Input }
    model { text str = "" }
    view {
        col {
            input { value: .text, placeholder: "Enter text", onchange: .Input }
        }
    }
}
```

### Output (Vue + shadcn-vue)
```vue
<script setup>
import { ref } from 'vue'
import { Input } from '@/components/ui/input'

const text = ref('')

const handleInput = (event) => {
    text.value = event.target.value
}
</script>

<template>
    <div class="flex flex-col">
        <Input
            v-model="text"
            placeholder="Enter text"
            @input="handleInput"
        />
    </div>
</template>
```

## Component Import Registry

```rust
// In vue.rs
struct ComponentRegistry {
    imports: HashMap<&'static str, Vec<&'static str>>,
}

impl ComponentRegistry {
    fn new() -> Self {
        let mut imports = HashMap::new();

        // Content
        imports.insert("button", vec!["Button"]);
        imports.insert("input", vec!["Input"]);
        imports.insert("textarea", vec!["Textarea"]);
        imports.insert("checkbox", vec!["Checkbox"]);
        imports.insert("toggle", vec!["Switch"]);
        imports.insert("select", vec!["Select", "SelectContent", "SelectItem", "SelectTrigger", "SelectValue"]);

        // Navigation
        imports.insert("tabs", vec!["Tabs", "TabsList", "TabsTrigger", "TabsContent"]);

        // Overlay
        imports.insert("modal", vec!["Dialog", "DialogContent", "DialogTrigger", "DialogTitle", "DialogDescription"]);
        imports.insert("tooltip", vec!["Tooltip", "TooltipContent", "TooltipProvider", "TooltipTrigger"]);

        // Form
        imports.insert("slider", vec!["Slider"]);
        imports.insert("radio", vec!["RadioGroup", "RadioGroupItem"]);

        // Feedback
        imports.insert("progress", vec!["Progress"]);
        imports.insert("badge", vec!["Badge"]);

        // Display
        imports.insert("card", vec!["Card", "CardHeader", "CardTitle", "CardContent"]);
        imports.insert("avatar", vec!["Avatar", "AvatarImage", "AvatarFallback"]);

        // Data
        imports.insert("table", vec!["Table", "TableHeader", "TableBody", "TableRow", "TableHead", "TableCell"]);

        // Utility
        imports.insert("divider", vec!["Separator"]);
        imports.insert("scroll", vec!["ScrollArea"]);

        Self { imports }
    }
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `crates/auto-lang/src/ui_gen/vue.rs` | Add shadcn-vue component generation |
| `crates/auto-lang/src/ui_gen/mod.rs` | Export new functions |
| `tmp/todomvc/vue/` | Migrate to shadcn-vue |

## Dependencies

### package.json additions
```json
{
  "dependencies": {
    "vue": "^3.4.0",
    "@vueuse/core": "^10.0.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.0.0",
    "tailwind-merge": "^2.0.0",
    "radix-vue": "^1.0.0"
  },
  "devDependencies": {
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.0.0",
    "postcss": "^8.0.0"
  }
}
```

## Success Criteria

1. All 43 AURA elements generate working shadcn-vue components
2. Generated components are accessible (WCAG 2.1 AA)
3. TodoMVC example works with shadcn-vue
4. Component imports are correctly generated
5. No manual intervention needed for basic usage

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| shadcn-vue API changes | Pin versions, test regularly |
| Complex components (tree) | Build custom on top of primitives |
| Bundle size | Tree-shaking, lazy loading |
| Learning curve | Good documentation, examples |

## Status

- [x] Phase 1: Setup Infrastructure ✅ DONE
- [ ] Phase 2: Core Components
- [ ] Phase 3: Layout & Navigation
- [ ] Phase 4: Overlay & Feedback
- [ ] Phase 5: Data Components
- [ ] Phase 6: Form Components
- [ ] Phase 7: Migration & Testing

## Phase 1 Completion Summary

**Implemented in `crates/auto-lang/src/ui_gen/vue.rs`:**

1. **ShadcnRegistry** - Component mapping table for all 43 elements
2. **VueMode enum** - Plain vs Shadcn generation modes
3. **VueGenerator updates**:
   - `new_shadcn()` constructor
   - `with_mode()` builder pattern
   - Component tracking via `shadcn_components_used`
4. **map_tag()** - Returns shadcn component names in Shadcn mode
5. **extract_classes()** - Skips default classes for shadcn components

**Project scaffold generators:**
- `generate_components_json()` - shadcn-vue configuration
- `generate_package_json()` - npm dependencies
- `generate_vite_config()` - Vite + Vue setup
- `generate_tailwind_config()` - Tailwind with CSS variables
- `generate_utils_ts()` - `cn()` helper function
- `generate_base_css()` - CSS custom properties theme
