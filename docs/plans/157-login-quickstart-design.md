# 06-Login Quickstart Example Design

**Date:** 2026-03-31
**Status:** Approved
**Approach:** Single PR (All-in-One)

## Overview

Add a new quickstart example (06-Login) that reproduces the HarmonyOS Login page from AURA to ArkTS.

## Goals

1. Extend Grid widget to support responsive layout (GridRow/GridCol)
2. Add maxLength prop to Input widget
3. Create 06-Login example with AURA source and expected ArkTS output
4. Add a2ark test case for Login page

## Widget Enhancements

### 1. Grid Widget

Extend [Grid.at](../stdlib/aura/widgets/data/Grid.at) to support responsive layout:

**New Model Props:**
```auto
model {
    columns ResponsiveValue = { sm: 4, md: 8, lg: 12 }
    gutter int = 12
}
```

**New GridItem Widget:**
```auto
widget GridItem {
    model {
        span ResponsiveValue = { sm: 4, md: 6, lg: 8 }
        offset ResponsiveValue = { md: 1, lg: 2 }
    }
    view {}
}
```

**ArkTS Output:**
```typescript
GridRow({
    columns: { sm: 4, md: 8, lg: 12 },
    gutter: { x: 12 }
}) {
    GridCol({
        span: { sm: 4, md: 6, lg: 8 },
        offset: { md: 1, lg: 2 }
    }) {
        // children
    }
}
```

### 2. Input Widget

Add maxLength prop to [Input.at](../stdlib/aura/widgets/form/Input.at).

**New Model Prop:**
```auto
model {
    // existing props...
    maxLength int = 0  // 0 means no limit
}
```

**ArkTS Output:**
```typescript
TextInput({ placeholder: "..." })
    .maxLength(11)  // Only output if maxLength > 0
    .type(InputType.Password)
```

## Login Example Structure

### Directory Structure

```
examples/quickstart/06-Login/
├── pac.at              # Project config
├── app.at              # Main entry (App widget)
├── aura/
│   └── LoginPage.at    # Login page widget
└── ark/                # Generated ArkTS project
    └── entry/src/main/ets/
        └── pages/
            └── LoginPage.ets
```

### AURA Source (LoginPage.at)

```auto
widget LoginPage {
    model {
        account str = ""
        password str = ""
    }

    view {
        Grid (columns: { sm: 4, md: 8, lg: 12 }, gutter: 12) {
            GridItem (span: { sm: 4, md: 6, lg: 8 }, offset: { md: 1, lg: 2 }) {
                Col (class: "w-full") {
                    // Title component
                    Col (class: "h-[40%] justify-center items-center") {
                        Image (src: "app.media.icon", class: "w-16 h-16 mb-12")
                        Text "Login" (class: "text-xl font-medium")
                        Text "More options" (class: "text-sm text-gray-500 mt-2 mb-4")
                    }

                    // Bottom component
                    Col (class: "h-[60%] p-4") {
                        Col (class: "bg-white rounded-lg") {
                            Input (placeholder: "Account", maxLength: 11, inputType: "number", onchange: UpdateAccount) {}
                            Separator (class: "w-full h-[1px] mx-2 bg-gray-200") {}
                            Input (placeholder: "Password", maxLength: 8, inputType: "password", onchange: UpdatePassword) {}
                        }

                        Row (class: "w-full justify-between mt-4 px-2") {
                            Text "Message Login" (class: "text-sm text-blue-500", onclick: ShowToast) {}
                            Text "Forgot Password" (class: "text-sm text-blue-500", onclick: ShowToast) {}
                        }

                        Button "Login" (class: "w-full h-12 mt-6", onclick: Login) {}
                        Text "Register Account" (class: "text-sm text-blue-500 font-medium", onclick: ShowToast) {}
                    }
                }
            }
        }
    }

    msg {
        UpdateAccount(value: str)
        UpdatePassword(value: str)
        Login
        ShowToast
    }

    on {
        UpdateAccount(value) => { account = value }
        UpdatePassword(value) => { password = value }
        Login => {
            // Login logic (show toast for demo)
        }
        ShowToast => {
            // Show toast
        }
    }
}
```

## Testing Strategy

### Test Location

Add test to `crates/auto-lang/test/a2ark/019_login/`:
- `input.at` - AURA source
- `input.expected.ets` - Expected ArkTS output

### Verification Criteria

| Check | Description |
|-------|-------------|
| GridRow/GridCol | Responsive grid with columns/gutter/offset |
| TextInput account | Placeholder, maxLength=11, InputType.Number |
| TextInput password | Placeholder, maxLength=8, InputType.Password |
| Separator | Line between inputs |
| Row with links | SpaceBetween layout for text links |
| Button | Login button with onClick handler |
| @Extend functions | inputStyle, blueTextStyle generation |

## Implementation Tasks

1. **Extend Grid widget** - Add columns, gutter props and GridItem widget
2. **Extend Input widget** - Add maxLength prop
3. **Create 06-Login example** - Directory structure + AURA source
4. **Add a2ark test** - Test case with expected output
5. **Update a2ark generator** - Handle GridRow/GridCol and maxLength

## Deferred Items

- Resource system ($r references) - hardcoded values for now
- @Extend style functions - inline styles/classes for now
