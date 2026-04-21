# Plan 098: AURA Widget Schema Specification

## Overview

AURA (Auto UI Representation Abstract) needs a schema system to:
1. Validate widget definitions at parse time
2. Provide better error messages for incorrect code
3. Enable LSP features (autocomplete, hover docs, diagnostics)
4. Document the official widget vocabulary

## Schema Structure

### 1. Widget Definition Schema

The top-level `widget` block has strict child requirements:

```auto
widget WidgetName {
    msg Msg { ... }        // REQUIRED: exactly one
    model { ... }          // REQUIRED: exactly one
    computed { ... }       // OPTIONAL: at most one
    view { ... }           // REQUIRED: exactly one
    on { ... }             // OPTIONAL: at most one
}
```

**Constraint Table:**

| Block | Required | Count | Description |
|-------|----------|-------|-------------|
| `msg` | Yes | Exactly 1 | Message type declarations |
| `model` | Yes | Exactly 1 | State variable definitions |
| `computed` | No | 0 or 1 | Computed property definitions |
| `view` | Yes | Exactly 1 | View tree definition |
| `on` | No | 0 or 1 | Event handler definitions |

### 2. View Element Categories

#### 2.1 Layout Elements

Elements that arrange children:

| Element | Description | Props | Children |
|---------|-------------|-------|----------|
| `col` | Vertical layout | `class`, `gap`, `padding`, `align`, `justify` | Any view elements |
| `row` | Horizontal layout | `class`, `gap`, `padding`, `align`, `justify` | Any view elements |
| `grid` | Grid layout | `class`, `columns`, `rows`, `gap` | Any view elements |
| `stack` | Stacked layers | `class` | Any view elements |
| `scroll` | Scrollable container | `class`, `direction` | Any view elements |
| `container` | Generic container | `class`, `max_width`, `padding` | Any view elements |

#### 2.2 Content Elements

Elements that display content:

| Element | Description | Props | Children |
|---------|-------------|-------|----------|
| `text` | Text content | (inline string) | None |
| `button` | Clickable button | `text`, `onclick`, `class`, `disabled` | None |
| `input` | Text input field | `value`, `placeholder`, `onchange`, `onenter`, `type`, `class` | None |
| `checkbox` | Checkbox control | `checked`, `onchange`, `class`, `disabled` | None |
| `toggle` | Toggle switch | `checked`, `onchange`, `class`, `disabled` | None |
| `image` | Image display | `src`, `alt`, `class`, `fit` | None |
| `icon` | Icon display | `name`, `class`, `size` | None |
| `link` | Hyperlink | `href`, `text`, `class` | None |
| `divider` | Horizontal/vertical line | `class`, `direction` | None |
| `spacer` | Flexible space | `class`, `size` | None |

#### 2.3 Typography Elements

| Element | Description | Props | Children |
|---------|-------------|-------|----------|
| `h1` | Heading 1 | `text`, `class` | None |
| `h2` | Heading 2 | `text`, `class` | None |
| `h3` | Heading 3 | `text`, `class` | None |
| `h4` | Heading 4 | `text`, `class` | None |
| `h5` | Heading 5 | `text`, `class` | None |
| `h6` | Heading 6 | `text`, `class` | None |
| `p` | Paragraph | `text`, `class` | None |
| `span` | Inline text | `text`, `class` | None |
| `code` | Code text | `text`, `class` | None |
| `pre` | Preformatted | `text`, `class` | None |

#### 2.4 List Elements

| Element | Description | Props | Children |
|---------|-------------|-------|----------|
| `list` | Generic list | `class` | `list_item` |
| `list_item` | List item | `class`, `onclick` | Any view elements |
| `table` | Table container | `class` | `table_row`, `table_header` |
| `table_row` | Table row | `class` | `table_cell` |
| `table_header` | Table header row | `class` | `table_cell` |
| `table_cell` | Table cell | `text`, `class` | None |

#### 2.5 Control Flow (Special)

These are not regular elements but special syntax:

| Syntax | Description | Constraints |
|--------|-------------|-------------|
| `if cond { ... }` | Conditional | Must have body, optional `else` |
| `if cond { ... } else { ... }` | Conditional with else | Both branches are view trees |
| `for item in .list { ... }` | Iteration | Body is view tree |
| `for idx, item in .list { ... }` | Iteration with index | Index is `int`, item depends on list |

### 3. Prop Type Definitions

#### 3.1 Primitive Types

| Type | Description | Examples |
|------|-------------|----------|
| `string` | String literal | `"hello"`, `"Click me"` |
| `int` | Integer number | `0`, `42`, `-10` |
| `float` | Floating point | `3.14`, `0.5` |
| `bool` | Boolean | `true`, `false` |
| `color` | Color value | `"#FF0000"`, `"red"`, `"rgb(255,0,0)"` |

#### 3.2 Reference Types

| Type | Description | Examples |
|------|-------------|----------|
| `state_ref` | Reference to state | `.count`, `.name` |
| `msg_ref` | Reference to message | `.Inc`, `.Submit` |
| `expr` | Expression | `.count + 1`, `!.done` |
| `closure` | Lambda expression | `\|x\| x * 2`, `\|t\| !t.done` |

#### 3.3 Special Types

| Type | Description | Examples |
|------|-------------|----------|
| `class_binding` | Dynamic class mapping | `{ completed: .done }` |
| `interpolated` | String with bindings | `\`Count: ${.count}\`` |

### 4. Element Schemas (Detailed)

#### 4.1 `button` Element

```typescript
interface ButtonSchema {
  tag: "button";
  props: {
    text?: string;                    // Button label
    onclick?: msg_ref;                // Click handler
    class?: string | class_binding;   // CSS classes
    disabled?: bool | state_ref;      // Disabled state
  };
  children: never;                    // No children allowed
  syntax: {
    // Full form
    "button (text: \"Click\", onclick: .Click) {}",
    // Simplified form
    "button \"Click\" { onclick: .Click }"
  };
}
```

#### 4.2 `input` Element

```typescript
interface InputSchema {
  tag: "input";
  props: {
    value?: state_ref;                // Bound value (two-way)
    placeholder?: string;             // Placeholder text
    type?: "text" | "password" | "email" | "number";
    onchange?: msg_ref;               // Value change handler
    onenter?: msg_ref;                // Enter key handler
    onfocus?: msg_ref;                // Focus handler
    onblur?: msg_ref;                 // Blur handler
    class?: string | class_binding;
    disabled?: bool | state_ref;
    maxlength?: int;
  };
  children: never;
}
```

#### 4.3 `checkbox` Element

```typescript
interface CheckboxSchema {
  tag: "checkbox";
  props: {
    checked?: bool | state_ref;       // Checked state
    onchange?: msg_ref;               // Toggle handler
    class?: string | class_binding;
    disabled?: bool | state_ref;
  };
  children: never;
}
```

#### 4.4 `col` / `row` Elements

```typescript
interface LayoutSchema {
  tag: "col" | "row";
  props: {
    class?: string | class_binding;
    gap?: int | float;                // Spacing between children
    padding?: int | string;           // Inner padding
    align?: "start" | "center" | "end" | "stretch";
    justify?: "start" | "center" | "end" | "between" | "around";
  };
  children: view_element[];           // Any view elements
}
```

#### 4.5 `text` Element

```typescript
interface TextSchema {
  tag: "text";
  props: {
    // Content is inline, not a prop
    // Either: "literal string" or `interpolated ${.var}`
  };
  children: never;
  syntax: {
    "text \"Hello World\"",           // Literal
    "text `Hello ${.name}`",          // Interpolated
  };
}
```

### 5. Model Block Schema

```auto
model {
    name type = initial_value
    name type = initial_value
    ...
}
```

**Constraints:**
- Each variable must have: name, type, initial value
- Type must be valid: `int`, `float`, `bool`, `str`, `List<T>`, or custom type
- Initial value must match type

### 6. Computed Block Schema

```auto
computed {
    name => expression
    name => expression
    ...
}
```

**Constraints:**
- Expression must be pure (no side effects)
- Can reference: state variables (`.var`), other computed properties
- Cannot reference: messages, handlers

### 7. Message Block Schema

```auto
msg MsgName {
    Variant1
    Variant2(payload_type)
    Variant3
    ...
}
```

**Constraints:**
- At least one variant required
- Variants are PascalCase
- Optional payload type in parentheses

### 8. On Block Schema

```auto
on {
    Variant => { statements }
    Variant(param) => { statements }
    ...
}
```

**Constraints:**
- Variant must exist in `msg` block
- Parameters (if any) must match payload type
- Statements can modify state with `.state = value`

## Error Messages

### Examples of Schema-Validated Errors

#### E0981: Missing Required Block
```
error[E0981]: widget missing required block
  --> example.at:1:1
   |
1  | widget Counter {
   | ^^^^^^^^^^^^^^ widget definition
2  |     view { ... }
   | ---------------- view block present
   |
   = help: widget must have a 'msg' block for message declarations
   = help: widget must have a 'model' block for state variables
```

#### E0982: Duplicate Block
```
error[E0982]: duplicate block in widget
  --> example.at:5:5
   |
4  |     model { count int = 0 }
5  |     model { name str = "" }
   |     ^^^^^ duplicate 'model' block
   |
   = help: widget can have at most one 'model' block
```

#### E0983: Unknown Element
```
error[E0983]: unknown view element
  --> example.at:10:5
   |
10 |     mybutton "Click" { }
   |     ^^^^^^^^ unknown element 'mybutton'
   |
   = help: did you mean 'button'?
   = help: available elements: col, row, button, text, input, ...
```

#### E0984: Invalid Prop
```
error[E0984]: invalid prop for element
  --> example.at:12:13
   |
12 |     button "Click" { onhover: .Hover }
   |             ^^^^^^ invalid prop 'onhover' for 'button'
   |
   = help: did you mean 'onclick'?
   = help: valid props for 'button': text, onclick, class, disabled
```

#### E0985: Missing Required Prop
```
error[E0985]: missing required prop
  --> example.at:15:5
   |
15 |     input { }
   |     ^^^^^ 'input' element requires 'value' prop for two-way binding
   |
   = help: add 'value: .stateVar' to bind input to state
```

#### E0986: Invalid Child
```
error[E0986]: element cannot have children
  --> example.at:18:5
   |
18 |     button "Click" { text "child" }
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ 'button' cannot have children
   |
   = help: 'button' is a leaf element, remove the child
```

## Implementation Plan

### Phase 1: Schema Definition (Week 1)
- [ ] Create `AuraSchema` struct with element definitions
- [ ] Define all official elements with props
- [ ] Add type constraints for each prop

### Phase 2: Validation (Week 2)
- [ ] Implement `validate_widget()` function
- [ ] Check required/optional blocks
- [ ] Validate view tree against schema
- [ ] Generate helpful error messages

### Phase 3: LSP Integration (Week 3)
- [ ] Expose schema for completion
- [ ] Provide hover documentation
- [ ] Real-time validation diagnostics

### Phase 4: Documentation (Week 4)
- [ ] Generate schema documentation
- [ ] Create element reference guide
- [ ] Add examples for each element

## File Structure

```
schema/
└── aura.at                    # AutoLang-based schema definition

crates/auto-lang/src/aura/
├── mod.rs                     # Module exports
├── types.rs                   # (existing) AURA types
├── extract.rs                 # (existing) Extraction
├── atom.rs                    # (existing) Atom serialization
├── schema.rs                  # Schema types (ElementDef, PropDef, PropType)
├── schema_loader.rs           # Schema parser (loads aura.at)
└── validate.rs                # (TODO) Validation logic
```

## Schema Definition (Rust)

```rust
// schema.rs

/// Element category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementCategory {
    Layout,     // col, row, grid, etc.
    Content,    // button, text, input, etc.
    Typography, // h1, h2, p, etc.
    List,       // list, list_item, etc.
}

/// Prop type constraint
#[derive(Debug, Clone)]
pub enum PropType {
    String,
    Int,
    Float,
    Bool,
    Color,
    StateRef,
    MsgRef,
    Expr,
    Closure,
    ClassBinding,
    OneOf(Vec<&'static str>),  // Enum-like: "text" | "password" | "email"
}

/// Element prop definition
#[derive(Debug, Clone)]
pub struct PropDef {
    pub name: &'static str,
    pub type_: PropType,
    pub required: bool,
    pub default: Option<&'static str>,
    pub description: &'static str,
}

/// Element definition
#[derive(Debug, Clone)]
pub struct ElementDef {
    pub tag: &'static str,
    pub category: ElementCategory,
    pub props: Vec<PropDef>,
    pub allows_children: bool,
    pub description: &'static str,
}

/// The complete AURA schema
pub struct AuraSchema {
    pub elements: HashMap<&'static str, ElementDef>,
    pub widget_blocks: WidgetBlockSchema,
}

/// Widget block constraints
pub struct WidgetBlockSchema {
    pub required: Vec<&'static str>,     // ["msg", "model", "view"]
    pub optional: Vec<&'static str>,     // ["computed", "on"]
    pub max_one: Vec<&'static str>,      // all blocks
}
```

## Example Schema Entry

```rust
// schema.rs

pub const BUTTON_ELEMENT: ElementDef = ElementDef {
    tag: "button",
    category: ElementCategory::Content,
    props: vec![
        PropDef {
            name: "text",
            type_: PropType::String,
            required: false,
            default: None,
            description: "Button label text",
        },
        PropDef {
            name: "onclick",
            type_: PropType::MsgRef,
            required: false,
            default: None,
            description: "Message to send when clicked",
        },
        PropDef {
            name: "class",
            type_: PropType::ClassBinding,
            required: false,
            default: None,
            description: "CSS class(es), can be dynamic",
        },
        PropDef {
            name: "disabled",
            type_: PropType::Bool,
            required: false,
            default: Some("false"),
            description: "Whether button is disabled",
        },
    ],
    allows_children: false,
    description: "A clickable button element",
};
```

## AutoLang-Based Schema (Recommended Approach)

Instead of hardcoding schemas in Rust, we use AutoLang itself as the schema DSL:

### Benefits

1. **Self-Documenting**: Schema is in the same language it describes
2. **Version Control**: Schema evolves with the codebase
3. **Extensibility**: Add new elements without recompiling
4. **Runtime Loading**: Load schemas dynamically
5. **LSP-Friendly**: Schema file itself has IDE support

### Schema File Structure

```
schema/
├── aura.at           # Core AURA element schemas
├── html.at           # HTML element mappings
└── custom/           # User-defined component schemas
    └── my_widget.at
```

### Example Schema (schema/aura.at)

```auto
// Element definition in AutoLang
element button {
    tag: "button"
    category: "content"
    props: [
        { name: "text", type: "string", description: "Button label" }
        { name: "onclick", type: "msg_ref", description: "Click handler" }
        { name: "class", type: "union:string,class_binding", description: "CSS classes" }
    ]
    allows_children: false
    description: "A clickable button element"
}

// Widget block constraints
widget_blocks {
    required: ["msg", "model", "view"]
    optional: ["computed", "on"]
}

// Full schema export
schema aura {
    version: "1.0.0"
    elements: [button, input, col, row, ...]
    widget: widget_blocks
}
```

### Implementation Plan (AutoLang Approach)

**Phase 1: Schema Parser** ✅ DONE
- [x] Parse `schema/aura.at` file
- [x] Extract `element` definitions
- [x] Build in-memory schema structures
- [x] Support `union:` and `one_of:` type syntax
- [x] Resolve `const` references for type names
- [x] Parse `widget_blocks` constraints

**Phase 2: Validation** ✅ DONE
- [x] Load schema at parser startup
- [x] Validate widgets against schema
- [x] Check required/optional widget blocks
- [x] Validate view elements against schema
- [x] Generate helpful error messages

**Phase 3: LSP Integration**
- [ ] Serve schema for completion
- [ ] Provide hover documentation
- [ ] Real-time validation

**Phase 4: Custom Schemas**
- [ ] Allow user-defined `element` schemas
- [ ] Hot-reload schema changes

### Type Syntax

| Syntax | Meaning | Example |
|--------|---------|---------|
| `"string"` | String type | `"hello"` |
| `"int"` | Integer type | `42` |
| `"msg_ref"` | Message reference | `.Click` |
| `"state_ref"` | State reference | `.count` |
| `"union:a,b"` | Union of types | `"union:string,class_binding"` |
| `"one_of:a,b,c"` | Enum-like | `"one_of:start,center,end"` |

## Status

- [x] Schema specification designed
- [x] AutoLang-based schema file created (`schema/aura.at`)
- [x] Rust schema types implementation (`schema.rs`)
- [x] AutoLang schema parser (`schema_loader.rs`)
- [x] Schema loader tests (4 tests passing)
- [x] Validation logic (`validate.rs`)
- [x] Validation tests (7 tests passing)
- [ ] LSP integration
- [ ] Documentation generation

## Files Created

| File | Description |
|------|-------------|
| `schema/aura.at` | AutoLang-based schema definition (19 elements) |
| `crates/auto-lang/src/aura/schema.rs` | Rust schema types (ElementDef, PropDef, PropType) |
| `crates/auto-lang/src/aura/schema_loader.rs` | Schema parser (loads aura.at) |
| `crates/auto-lang/src/aura/validate.rs` | Widget validation with error codes (E0981-E0986) |

## Usage

```rust
use auto_lang::aura::{load_default_schema, WidgetValidator};

// Load schema from embedded schema/aura.at
let schema = load_default_schema()?;

// Use schema for element lookup
assert!(schema.get_element("button").is_some());
assert!(schema.widget_blocks.is_required("msg"));

// Get suggestions for typos
let suggestion = schema.suggest_similar("buton"); // Returns Some("button")

// Validate a widget
let validator = WidgetValidator::new()?;
match validator.validate_widget(&widget) {
    Ok(()) => println!("Widget is valid!"),
    Err(errors) => {
        for error in &errors {
            eprintln!("{}", error);
        }
    }
}

// Validate a single element
let validation = validator.validate_element("button");
assert!(validation.is_valid());
assert!(!validation.allows_children());
```
