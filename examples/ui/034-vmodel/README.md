# 034 — v-model Contracts

Two halves of Vue `v-model` in the Auto DSL:

## Builtin `v-model:open` (dialog / alertdialog / modal)

`open: .state` on the builtin overlay widgets now emits a real
`v-model:open` binding:

```auto
dialog(open: .show) { ... }          // → <Dialog v-model:open="show">
alertdialog(open: .confirm_open) {}  // → <AlertDialog v-model:open="confirm_open">
modal(open: .m) { ... }              // → <Dialog v-model:open="m">
```

Previously the binding was **silently dropped**: `extract_state_ref` only
matched a hand-built bare `Ident` AST, while the parser actually produces
`Dot(Ident("self"), name)` for `.state` refs. `modal` additionally never
reached the shadcn `Dialog` at all (no registry entry → plain `<div>`);
it is now an alias of `dialog`.

## Custom widget as a v-model target (child-side contract)

A widget declares the contract with a `modelValue` prop plus a **quoted msg
variant name** — the colon is legal in a Vue emit name but not in an
identifier:

```auto
widget TextField(label: str, modelValue: str) {
    msg Msg { "update:modelValue"(str) }
    view {
        input { value: .modelValue, oninput: ."update:modelValue" }
    }
    on {
        ."update:modelValue"(v) -> { }
    }
}
```

The compiler keeps the emit name verbatim but sanitizes the handler function
name (`update_modelValue` — a valid JS identifier). The generated SFC:

```vue
const props = defineProps<{ label: string, modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [string] }>()
function update_modelValue(v: any): void { emit('update:modelValue', v) }
// template: <Input :modelValue="modelValue" @update:modelValue="update_modelValue" />
```

Note the prop-backed value is bound **one-way** (`:modelValue`) — a prop is
read-only, so `v-model` on it would be broken; the change goes up through
the `update:modelValue` emit instead.

## Parent side (manual wiring, no sugar)

Parents bind the contract explicitly — prop down + quoted custom event up
(quoted event names ship since the custom-events feature):

```auto
TextField(label: "Name", modelValue: .name) {
    on "update:modelValue": .NameChanged
}
// → <TextField :modelValue="name" @update:modelValue="NameChanged" />
```

Vue parents can use real sugar on the generated component:
`<TextField v-model="name" />`.

## Number coercion

`.to_float()` / `.to_double()` → `parseFloat(...)` (mirrors the existing
`.to_int()` → `parseInt(...)`), used in the `price` computed above.
