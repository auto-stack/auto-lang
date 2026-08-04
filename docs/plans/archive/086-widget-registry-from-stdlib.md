# Plan 086: Widget Registry from Stdlib

Date: 2026-03-22

## Objective

Make `WidgetRegistry` load widget specifications from `stdlib/aura/widgets/*.at` files instead of hardcoded defaults, and support `#[primary]` annotation for shorthand property syntax.

## Current State

### Problem 1: Hardcoded Registry
- `WidgetRegistry::with_defaults()` has hardcoded widget specs in Rust
- Duplicates information that already exists in `.at` files
- Gets out of sync when `.at` files change

### Problem 2: No `#[primary]` Support
- Text widget has `content str = ""` but registry expects `primary_prop = "text"`
- No way to mark a property as primary for shorthand syntax
- User writes `Text (text: "Hello") {}` but wants `Text "Hello" {}`

### Problem 3: Annotations Not Parsed
- `#[spec(...)]` and `#[backend(...)]` exist in `.at` files
- Parser doesn't extract them - just decorative
- No way to populate WidgetSpec from actual widget definitions

## Proposed Solution

### Phase 1: Add `#[primary]` Annotation (Parser)

Extend parser to support `#[primary]` annotation on model properties:

```auto
widget Text {
    model {
        #[primary]
        text str = ""
    }
}
```

**Parser changes:**
1. Extend `PropDecl` AST to include `is_primary: bool` field
2. Update `parse_model_prop()` to parse `#[primary]` annotation
3. Store in `WidgetDecl.model.props[].is_primary`

### Phase 2: Update Text Widget

**File:** `stdlib/aura/widgets/display/text.at`

**Changes:**
1. Rename `content` to `text`
2. Add `#[primary]` annotation
3. Update docstring examples

```auto
widget Text {
    model {
        #[primary]
        text str = ""
        style str = "default"
        // ...
    }
    // ... rest unchanged
}
```

### Phase 3: Create WidgetLoader Module

**New file:** `crates/auto-lang/src/ui_gen/widget/loader.rs`

**Responsibilities:**
1. Scan `stdlib/aura/widgets/` directory recursively
2. Parse each `.at` file using existing AutoLang parser
3. Extract widget specifications from annotations:
   - `#[spec(...)]` → category, has_children, primary_prop
   - `#[backend(...)]` → backend mappings
4. Extract default props from `view {}` literal values
5. Return populated `WidgetSpec` instances

**API:**
```rust
impl WidgetLoader {
    pub fn load_stdlib() -> Result<WidgetRegistry, LoadError>;
    pub fn load_directory(path: &Path) -> Result<WidgetRegistry, LoadError>;
}
```

### Phase 4: Update WidgetSpec

**File:** `crates/auto-lang/src/ui_gen/widget/spec.rs`

**Changes:**
1. Add `from_annotations()` constructor
2. Add `merge_view_defaults()` method
3. Ensure backward compatibility with existing hardcoded approach

### Phase 5: Update WidgetRegistry

**File:** `crates/auto-lang/src/ui_gen/widget/registry.rs`

**Changes:**
1. Add `from_loader()` constructor
2. Deprecate `with_defaults()` (keep for fallback)
3. Add `load_stdlib()` convenience method

### Phase 6: Update Generators

**Files:**
- `crates/auto-lang/src/ui_gen/ark/generator.rs`
- `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Changes:**
1. Replace `WidgetRegistry::with_defaults()` with `WidgetRegistry::from_stdlib()`
2. Handle potential load errors gracefully
3. Ensure backward compatibility

## Implementation Order

1. **Phase 1**: Parser support for `#[primary]` (foundation)
2. **Phase 2**: Update Text widget (demonstrates usage)
3. **Phase 3**: WidgetLoader module (core infrastructure)
4. **Phase 4**: Update WidgetSpec (support new loading)
5. **Phase 5**: Update WidgetRegistry (integration)
6. **Phase 6**: Update generators (consumers)

## Testing Strategy

### Unit Tests
1. `#[primary]` annotation parsing
2. WidgetLoader parsing of widget files
3. WidgetSpec creation from annotations
4. Registry population from loaded widgets
5. Generator compatibility with loaded registry

### Integration Tests
1. Parse `Text "Hello" {}` → generates correct code
2. Load all stdlib widgets → registry fully populated
3. Generator produces same output as before (backward compat)

### Manual Testing
```bash
# Test primary property shorthand
cd examples/quickstart/01-HelloWorld
auto build ark  # Should generate Text("Hello, World!")
```

## Success Criteria

1. `#[primary]` annotation parsed correctly
2. Text widget uses `text` property with `#[primary]`
3. WidgetLoader successfully loads all stdlib widgets
4. WidgetRegistry populated from loaded widgets
5. Generators work with loaded registry
6. Backward compatibility maintained
7. All existing tests pass

## Risks and Mitigations

### Risk: Breaking Existing Code
**Mitigation:** Keep `with_defaults()` as fallback, deprecation warning

### Risk: Parse Errors in Widget Files
**Mitigation:** Error handling in WidgetLoader, skip invalid files with warnings

### Risk: Performance Impact
**Mitigation:** Lazy loading, caching, only load once

### Risk: Circular Dependencies
**Mitigation:** Dependency detection, error on circular imports

## File Naming Convention

**Rules:**
1. Every widget is defined in its own file in `stdlib/aura/widgets/`
2. File names use PascalCase to match widget names (e.g., `Text.at`, `Center.at`, `Col.at`)
3. File location matches widget category (layout/, display/, form/, etc.)

**Current → Target:**
| Current | Target | Widget |
|---------|--------|--------|
| `layout/col.at` | `layout/Col.at` | Col |
| `layout/row.at` | `layout/Row.at` | Row |
| `layout/center.at` | `layout/Center.at` | Center |
| `display/text.at` | `display/Text.at` | Text |
| `display/image.at` | `display/Image.at` | Image |
| `form/button.at` | `form/Button.at` | Button |
| `form/input.at` | `form/Input.at` | Input |

## Files Modified

| File | Changes |
|--------|---------|
| `parser.rs` | Add `#[primary]` annotation parsing |
| `ast/ui.rs` | Extend `PropDecl` with `is_primary` field |
| `stdlib/aura/widgets/display/Text.at` | Rename `content` to `text`, add `#[primary]` |
| `stdlib/aura/widgets/*` | Rename files to PascalCase |
| `ui_gen/widget/spec.rs` | Add `from_annotations()` constructor |
| `ui_gen/widget/loader.rs` | NEW: WidgetLoader implementation |
| `ui_gen/widget/registry.rs` | Add `from_stdlib()` method |
| `ui_gen/ark/generator.rs` | Use `from_stdlib()` instead of `with_defaults()` |
| `ui_gen/jet/generator.rs` | Use `from_stdlib()` instead of `with_defaults()` |

## Estimated Effort

- Phase 1: 2-3 hours (parser changes, testing)
- Phase 2: 1 hour (widget file update)
- Phase 3: 4-5 hours (WidgetLoader implementation)
- Phase 4: 1-2 hours (WidgetSpec updates)
- Phase 5: 1-2 hours (WidgetRegistry updates)
- Phase 6: 1-2 hours (generator updates)
- Testing: 2-3 hours (unit and integration tests)

**Total: 12-18 hours**
