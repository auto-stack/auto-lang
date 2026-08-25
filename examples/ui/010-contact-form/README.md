# 010-contact-form — Contact Form with Submit Feedback

A contact form with name, email, and message inputs plus a submit button that shows a confirmation message.

## Concepts

- Input widget with two-way binding
- Textarea widget for multi-line input
- Button with onclick handler
- Conditional success message

## Source

See `front/app.at`:

```auto
widget App {
    msg { NameChanged, EmailChanged, MessageChanged, Submit }

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
            input { placeholder: "Your Name", value: .name, oninput: .NameChanged }
            input { placeholder: "Your Email", value: .email, oninput: .EmailChanged }
            textarea { placeholder: "Your Message", value: .message, oninput: .MessageChanged }
            button "Send Message" { onclick: .Submit }
            if .submitted {
                text "Thank you! We'll be in touch."
            }
            class: "w-full max-w-lg p-8 gap-4"
        }
    }

    on {
        .NameChanged -> {
            .name = .name
        }
        .EmailChanged -> {
            .email = .email
        }
        .MessageChanged -> {
            .message = .message
        }
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

- Input widget: `input` with `placeholder`, `value`, and `oninput` for two-way binding
- Textarea widget: `textarea` for multi-line text input
- Button with `onclick` bound to the `Submit` message
- Conditional rendering: `if .submitted` shows a success message after submit
- Message block: `on` handlers update model fields and set `submitted` to true
