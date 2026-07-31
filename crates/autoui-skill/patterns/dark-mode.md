# Pattern: Dark Mode with Accent Theming

## Use case

Implementing dark/light theme toggle with optional accent color theming.
This pattern involves critical CSS variable scoping rules (C8).

## Standard .at structure

```auto
store AppStore {
    model {
        var dark_mode bool = false
        var accent_color str = "indigo"  // Plan 360
    }

    on {
        .ToggleDarkMode -> {
            .dark_mode = !.dark_mode
        }
        .SetAccent(name str) -> {
            .accent_color = name
        }
    }
}

widget App {
    use store: AppStore

    view {
        col {
            // Root element — generator auto-adds :class="{ dark: store.dark_mode }"
            row {
                button if store.dark_mode { "☀ Light" } else { "🌙 Dark" }
                    { onclick: .ToggleDarkMode }

                // Accent palette (Plan 360)
                row {
                    for accent in store.accent_names {
                        button {
                            style: "background: var(--primary)"
                            onclick: store.SetAccent(accent)
                        }
                    }
                }
            }
        }
    }
}
```

## Key constraints

- Handler must be EXACTLY `.ToggleDarkMode` — generator detects this name (C4, C5)
- Dark mode class is auto-injected on `#app > div` root element
- Accent system (C9) auto-generates: palettes, `applyAccent()`, bootstrap, getters

## The C8 trap (CRITICAL)

CSS variables written by `applyAccent()` must target the SAME DOM element
as the `.dark` CSS rules. The generator applies `.dark` to `#app > div`:

```css
/* Correct: both on same element */
#app > div {
    --primary: ...;  /* from applyAccent */
}
#app > div.dark {
    --primary: ...;  /* from .dark rule */
}

/* WRONG: different elements → dark mode accent fails */
html {
    --primary: ...;  /* from applyAccent (written to <html>) */
}
#app > div.dark {
    --primary: ...;  /* this WINS over <html> parent */
}
```

## Pitfalls

- **P4**: Using wrong handler name (`.ToggleDark` / `.ToggleTheme`) —
  generator won't inject `:class` binding
- **C8**: Accent CSS variables on wrong DOM element → dark mode accent
  disappears (most common bug in Plan 360)
- Dark mode transitions: always clean up old inline style residue when
  switching themes

## Validation checklist

- [ ] Handler named exactly `.ToggleDarkMode` (C5)
- [ ] Store has `dark_mode: bool` state var
- [ ] Accent CSS variables written to correct element (C8)
- [ ] Dark mode works in both light and dark contexts (T12-DARK regression)
