# 010-contact-form — Contact Form with Submit Feedback

A contact form with name, email, and message inputs plus a submit button that shows a confirmation message.

## Concepts

- Input widget with two-way binding — a bare `value: .field` syncs typed text into state on every backend (Plan 448 C), no msg/on boilerplate
- Textarea widget for multi-line input
- Button with onclick handler
- Conditional success message

## Source

See `front/app.at`:

```auto
widget App {
    msg { Submit }

    model {
        var name str = ""
        var email str = ""
        var message str = ""
        var submitted bool = false
    }

    view {
        col {
            text "Contact Us"
            text "We'd love to hear from you. Send us a message and we'll respond as soon as possible."
            input { placeholder: "Your Name", value: .name }
            input { placeholder: "Your Email", value: .email }
            textarea { placeholder: "Your Message", value: .message }
            button "Send Message" { onclick: .Submit }
            if .submitted {
                text "Thank you! We'll be in touch."
            }
            class: "w-full max-w-lg p-8 gap-4"
        }
    }

    on {
        .Submit -> {
            .submitted = true
        }
    }
}
```

## How to Run

```bash
cd examples/ui/010-contact-form
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- Input widget: `input` with `placeholder` and `value` — the bare `value:` binding is two-way
- Textarea widget: `textarea` for multi-line text input
- Button with `onclick` bound to the `Submit` message
- Conditional rendering: `if .submitted` shows a success message after submit
- Message block: `on` handlers update model fields and set `submitted` to true
