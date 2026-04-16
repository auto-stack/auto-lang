# 010-contact-form — Contact Form with Dropdown and Submit Feedback

A complete contact form with name input, email input, subject dropdown, message textarea, and a submit button. After submission, a confirmation message replaces the form area.

## Concepts
- Select/dropdown widget (subject choices)
- Textarea widget (multi-line message input)
- Form composition (multiple inputs in a column)
- Submit feedback (conditional confirmation message)

## Source

```auto
widget ContactForm {
    msg Msg { Submit, NameChanged, EmailChanged, SubjectChanged, MessageChanged }

    model {
        var name str = ""
        var email str = ""
        var subject str = "general"
        var message str = ""
        var submitted bool = false
    }

    view {
        col {
            text "Contact Us"
            text "We'd love to hear from you. Send us a message and we'll respond as soon as possible."
            input { placeholder: "Your Name", value: .name, onchange: .NameChanged }
            input { placeholder: "Your Email", value: .email, onchange: .EmailChanged }
            select { value: .subject, options: ["general", "support", "sales"], onchange: .SubjectChanged }
            textarea { placeholder: "Your Message", value: .message, onchange: .MessageChanged }
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
            print(f"Contact form submitted by ${.name} (${.email})")
        }
        .NameChanged -> { .name = .name }
        .EmailChanged -> { .email = .email }
        .SubjectChanged -> { .subject = .subject }
        .MessageChanged -> { .message = .message }
    }
}
```

## Generated Output

### Vue 3
<!-- Placeholder: coming soon -->

### Jetpack Compose
<!-- Placeholder: coming soon -->

### ArkTS (HarmonyOS)
<!-- Placeholder: coming soon -->

### GPUI (Rust)
<!-- Placeholder: coming soon -->

### Tauri
<!-- Placeholder: coming soon -->

### VSCode WebView
<!-- Placeholder: coming soon -->

## Platform Notes
- The `select` widget maps to a native dropdown: `<select>` in HTML, `ExposedDropdownMenuBox` in Jetpack Compose, `Select` in ArkTS, and a custom popup menu in GPUI.
- The `textarea` widget maps to a multi-line text input: `<textarea>` in HTML, `OutlinedTextField` with `maxLines` in Jetpack Compose, `TextArea` in ArkTS, and a multi-line text input in GPUI.
- The `input` widgets use `value` for two-way binding. In Vue this maps to `v-model`; in Jetpack Compose it maps to a state-backed `value` parameter with an `onValueChange` callback.
- The submit feedback uses conditional rendering (`if .submitted`). A real app would typically show a toast notification instead. The toast approach requires a `toast` widget which is aspirational for some generators (see plan P1).
- The `onchange` handlers receive the new value from the input. In the current pattern, `.name = .name` is a placeholder -- in practice, the input event carries the updated value that gets assigned to the model field.
- `max-w-lg` constrains the form width for desktop readability. On mobile targets the form expands to full width.
