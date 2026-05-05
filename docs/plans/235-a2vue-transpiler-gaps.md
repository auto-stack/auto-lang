# Plan: a2vue/a2ts Transpiler Extensions for State, Persistence & Events

## Executive Summary

The a3ui-replica project currently uses **handwritten Vue components** for features like `localStorage` CRUD, `CustomEvent` syncing, and router navigation. This is a workaround. The Auto language parser **can already express** most of these JS APIs (`JSON.parse()`, `window.localStorage.getItem()`, `Math.random()`, etc.), but the **a2vue/a2ts transpiler has critical gaps** that prevent them from working in handler bodies.

This plan identifies the exact blockers and proposes a phased implementation to close them.

---

## 1. Root Cause Analysis

### 1.1 What Already Works

The Auto → AURA → Vue pipeline **already supports** the following constructs for **template expressions and initializers**:

| Auto Syntax | AURA AST | Vue Output |
|------------|----------|------------|
| `JSON.parse(x)` | `MethodCall { object: StateRef("JSON"), method: "parse", args: [x] }` | `JSON.parse(x)` |
| `window.localStorage` | `FieldAccess { object: StateRef("window"), field: "localStorage" }` | `window.localStorage` |
| `Math.random()` | `MethodCall { object: StateRef("Math"), method: "random", args: [] }` | `Math.random()` |
| `Date.now()` | `MethodCall { object: StateRef("Date"), method: "now", args: [] }` | `Date.now()` |
| `array.len` | `MethodCall { ... method: "len" }` | `array.length` ✓ (mapped) |

**Evidence**: `vue.rs` `expr_to_js()` correctly transpiles `AuraExpr::MethodCall` and `AuraExpr::FieldAccess` to JavaScript (lines 2881–2917).

### 1.2 The Critical Blocker: `ts_adapter.rs` Panics on Method Calls in Handlers

**The bug**: `ts_adapter.rs` is responsible for transpiling Auto `Stmt`/`Expr` (from `on` handler bodies) to TypeScript. Its `Expr::Call` arm calls `Call::get_name_text()`, which **panics** if the call name is not a plain `Expr::Ident`:

```rust
// crates/auto-lang/src/ast/call.rs:20-24
impl Call {
    pub fn get_name_text(&self) -> AutoStr {
        match &self.name.as_ref() {
            Expr::Ident(name) => name.clone(),
            _ => panic!("Expected identifier, got {:?}", self.name),  // ← CRASH
        }
    }
}
```

**Impact**: Any method call like `JSON.parse(x)` or `window.localStorage.getItem(key)` in an `on` handler causes a **compiler panic**.

**Reproducer**:
```auto
widget Test {
    on {
        Click -> {
            let parsed = JSON.parse("[]")  // PANIC: Expected identifier, got Dot(...)
        }
    }
}
```

### 1.3 Secondary Gaps

| Gap | Location | Impact |
|-----|----------|--------|
| `ts_adapter.rs` missing `Expr::Dot` in expression transpilation | `ts_adapter.rs:195-308` | Field access in handlers delegates to a2ts, which may not handle all cases |
| Only `.len` → `.length` mapped | `vue.rs:2915` | Other array methods (`.find`, `.push`, `.filter`) pass through as-is, which is fine for JS but may need Auto-native abstractions |
| No `watch` syntax in Auto | `vue.rs:1116-1120` | Only `previewcard_data` triggers `watch` import; no general `watch()` construct |
| No `provide`/`inject` syntax | N/A | No Auto equivalent for Vue's provide/inject |
| No route param access | `vue.rs:1143-1146` | `useRouter()` is imported but `useRoute()` is not; only `route.query.data` is hardcoded in WidgetEditor.vue |
| `NavCall` only supports static paths | `vue.rs:2942-2953` | Cannot construct dynamic paths with variables |

---

## 2. Proposed Implementation Plan

### Phase A: Fix Critical Transpiler Bugs (Unblocks JS Interop)

**Goal**: Make method calls in handler bodies work without panics.

#### A1. Fix `ts_adapter.rs` `Expr::Call` to handle `Expr::Dot` names

**File**: `crates/auto-lang/src/ui_gen/ts_adapter.rs` (line ~228)

Current broken code:
```rust
Expr::Call(call) => {
    let func_name = call.get_name_text().to_string();  // PANICS on Dot
    ...
}
```

Fix: Mirror the logic from `extract.rs`:
```rust
Expr::Call(call) => {
    match call.name.as_ref() {
        Expr::Dot(object, method) => {
            // Method call: object.method(args)
            transpile_expr(object, ctx, out);
            write!(out, ".{}", method).ok();
            write!(out, "(").ok();
            for (i, arg) in call.args.args.iter().enumerate() {
                if i > 0 { write!(out, ", ").ok(); }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
        }
        Expr::Ident(name) => {
            let func_name = name.as_str();
            if ctx.is_api(&func_name) { write!(out, "await {}").ok(); }
            else if func_name == "print" { write!(out, "console.log").ok(); }
            else { write!(out, "{}", func_name).ok(); }
            write!(out, "(").ok();
            for (i, arg) in call.args.args.iter().enumerate() {
                if i > 0 { write!(out, ", ").ok(); }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
        }
        _ => { /* delegate */ }
    }
}
```

#### A2. Fix `ts_adapter.rs` `Expr::Dot` in expression transpilation

**File**: `crates/auto-lang/src/ui_gen/ts_adapter.rs` (line ~201)

Current code handles `self.` and state refs but delegates everything else to a2ts. Need to add explicit `FieldAccess` transpilation for handler body contexts:

```rust
Expr::Dot(object, field) => {
    if let Expr::Ident(name) = object.as_ref() {
        if name.as_str() == "self" || name.as_str() == "." {
            // State ref handling (existing)
            ...
            return;
        }
    }
    // General field access: object.field
    transpile_expr(object, ctx, out);
    write!(out, ".{}", field).ok();
}
```

#### A3. Make `Call::get_name_text()` safe (or replace callers)

Add a non-panicking method:
```rust
impl Call {
    pub fn get_name_text_safe(&self) -> Option<AutoStr> {
        match &self.name.as_ref() {
            Expr::Ident(name) => Some(name.clone()),
            _ => None,
        }
    }
}
```

Audit all callers of `get_name_text()` and ensure they handle `None` gracefully.

**Estimated effort**: 1–2 days
**Risk**: Low — additive fixes, no breaking changes

---

### Phase B: Add Auto-Native Abstractions for Browser APIs

**Goal**: Provide idiomatic Auto syntax for common patterns, rather than requiring raw JS interop.

#### B1. `storage` Module → localStorage

Auto syntax:
```auto
on {
    Save -> {
        storage.set("widgets", json.stringify(widgets))
        let loaded = storage.get("widgets")
        storage.remove("key")
    }
}
```

Transpiler mapping:
```rust
// In ts_adapter.rs Expr::Call handling
"storage.get" => "localStorage.getItem"
"storage.set" => "localStorage.setItem"
"storage.remove" => "localStorage.removeItem"
```

Or implement as a proper Auto module in the standard library.

#### B2. `event` Module → CustomEvent

Auto syntax:
```auto
on {
    Save -> {
        event.dispatch("a2ui-widgets-changed")
        event.dispatch("my-event", { detail: 42 })
    }
}
```

Transpiler mapping:
```typescript
window.dispatchEvent(new CustomEvent("a2ui-widgets-changed"))
window.dispatchEvent(new CustomEvent("my-event", { detail: 42 }))
```

#### B3. `json` Module → JSON

Auto syntax:
```auto
let obj = json.parse(jsonString)
let str = json.stringify(obj)
```

Since `JSON.parse` already works in templates (via `expr_to_js`), this is mainly about making it ergonomic in handlers.

#### B4. `math` / `date` Builtins

Auto syntax:
```auto
let id = math.random()
let now = date.now()
```

Transpiler mapping:
```typescript
Math.random()
Date.now()
```

#### B5. `router` Enhancements

Current `NavCall` supports: `Nav.to("/path", { param: value })` → `router.push({ path: '/path', query: { param: value } })`

Missing:
- Route param access: `route.param("id")` → `useRoute().params.id`
- Query access: `route.query("search")` → `useRoute().query.search`
- Current path: `route.path` → `useRoute().path`

Implementation:
1. Add `useRoute` import when `route.*` is detected
2. Transpile `route.param("id")` to `(useRoute().params as any).id` or `useRoute().params["id"]`

**Estimated effort**: 3–5 days
**Risk**: Medium — requires designing Auto-native APIs that feel idiomatic

---

### Phase C: Add Missing Vue Lifecycle & Reactivity Constructs

**Goal**: Support `watch`, `provide`/`inject`, and computed properties from Auto.

#### C1. `watch` Syntax

Auto syntax:
```auto
widget Editor {
    model { widgetJson str = "[]" }

    watch {
        widgetJson -> {
            // Runs when widgetJson changes
            print("JSON changed")
        }
    }
}
```

Transpiler output:
```typescript
watch(widgetJson, () => {
    console.log("JSON changed")
})
```

Implementation:
1. Add `watch` block to `AuraWidget` AST
2. In `generate_script()`, detect `watch` blocks and add `watch` to Vue imports
3. Generate `watch(stateVar, () => { ... })` calls

#### C2. `provide` / `inject` Syntax

Auto syntax:
```auto
widget App {
    provide {
        widgetStore = widgetStoreRef
    }
}

widget Sidebar {
    inject {
        widgetStore
    }
}
```

Transpiler output:
```typescript
// App.vue
provide('widgetStore', widgetStoreRef)

// Sidebar.vue
const widgetStore = inject('widgetStore')
```

Implementation:
1. Add `provide` and `inject` blocks to `AuraWidget`
2. Add `provide`/`inject` to Vue imports when used
3. Generate calls in script setup

#### C3. Computed Properties in `model`

Current computed properties exist in AURA but may not be fully exposed in Auto syntax. Verify and document.

**Estimated effort**: 4–6 days
**Risk**: Medium — requires AST changes

---

### Phase D: Array & Collection Method Mappings

**Goal**: Map common Auto collection operations to JS array methods.

| Auto | JS |
|------|-----|
| `array.push(item)` | `array.push(item)` |
| `array.find(\|x\| x.id == id)` | `array.find((x) => x.id === id)` |
| `array.filter(\|x\| x.active)` | `array.filter((x) => x.active)` |
| `array.map(\|x\| x.name)` | `array.map((x) => x.name)` |
| `array.len` | `array.length` ✓ (already works) |

Most of these already pass through correctly once Phase A fixes are in place (since they're `MethodCall` AST nodes). The main work is:
1. Verify lambda/closure arguments transpile correctly in method call contexts
2. Add `find`, `filter`, `map`, `push` to the method call whitelist if any validation exists

**Estimated effort**: 1–2 days
**Risk**: Low

---

## 3. Prioritized Roadmap

| Phase | Task | Effort | Unblocks |
|-------|------|--------|----------|
| A1 | Fix `ts_adapter.rs` `Expr::Call` for `Expr::Dot` | 0.5d | All method calls in handlers |
| A2 | Fix `ts_adapter.rs` `Expr::Dot` field access | 0.5d | Property access in handlers |
| A3 | Safe `get_name_text()` + audit callers | 0.5d | Robustness |
| B1 | `storage` module | 1d | localStorage persistence |
| B2 | `event` module | 0.5d | Cross-component events |
| B3 | `json` module | 0.5d | JSON serialization |
| B4 | `math`/`date` builtins | 0.5d | IDs, timestamps |
| B5 | `router` param/query access | 1d | Dynamic routing |
| C1 | `watch` syntax | 2d | Reactive side effects |
| C2 | `provide`/`inject` syntax | 2d | Dependency injection |
| D | Array method verification | 1d | Collection operations |
| **Total** | | **~10–12 days** | Full widget CRUD in Auto |

---

## 4. Immediate Next Steps

1. **Implement Phase A** (fix transpiler panics). This is a hard blocker — everything else depends on it.
2. **Write test cases** for each fixed pattern:
   ```auto
   widget MethodCallTest {
       on {
           Test -> {
               let a = JSON.parse("{}")
               let b = window.localStorage.getItem("key")
               let c = Math.random()
               let d = Date.now()
               array.push(42)
               let e = array.find(|x| x == 42)
           }
       }
   }
   ```
3. Once Phase A is verified, implement `storage` + `event` + `json` modules (Phase B1–B3) to replace the handwritten Vue components in a3ui-replica.

---

## 5. Files to Modify

| File | Changes |
|------|---------|
| `crates/auto-lang/src/ui_gen/ts_adapter.rs` | Fix `Expr::Call` Dot handling; fix `Expr::Dot` field access; add `storage`/`event`/`json` builtin detection |
| `crates/auto-lang/src/ast/call.rs` | Add `get_name_text_safe()` |
| `crates/auto-lang/src/ui_gen/vue.rs` | Add `useRoute` import; add `watch`/`provide`/`inject` generation; add route param transpilation |
| `crates/auto-lang/src/aura/types.rs` | Add `watch`, `provide`, `inject` fields to `AuraWidget` |
| `crates/auto-lang/src/aura/extract.rs` | Extract `watch`/`provide`/`inject` blocks from Auto AST |
| `crates/auto-lang/src/aura/atom.rs` | Update debug formatting if needed |
| `stdlib/auto/` or `stdlib/aura/` | Add `storage`, `event`, `json`, `math`, `date` module definitions |
