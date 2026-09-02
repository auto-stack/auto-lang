# 005-login — Login Form

A complete login form with email and password fields, conditional error display, and a submit button. Demonstrates form patterns and conditional rendering.

## Concepts

- **Input widget** — `input` with `value`, `placeholder`, and `type` properties for form fields
- **Two-way binding** — a bare `value: .field` input syncs typed text into state on every backend (Plan 448 C); side effects go inline (`oninput: () => {…}`, Plan 448 B)
- **Conditional rendering** — `if` blocks show/hide error messages based on model state
- **Button widget** — `button` with `onclick` event for form submission
- **Form pattern** — Grouping labeled inputs, error messages, and a submit button in a card layout

## Source

See `front/app.at`:

```auto
widget App {
    msg { Submit }

    model {
        var email str = ""
        var password str = ""
        var email_error str = ""
        var password_error str = ""
    }

    view {
        col {
            text "Sign In"
            class: "text-2xl font-bold mb-2"
            text "Welcome back! Please enter your credentials."
            class: "text-sm text-gray-500 mb-8"

            col {
                text "Email"
                class: "text-sm font-medium text-gray-700 mb-1"
                input (value: .email) {
                    oninput: () => { if .email != "" { .email_error = "" } }
                    placeholder: "you@example.com"
                    class: "w-full px-3 py-2 border rounded-lg"
                }
                if .email_error != "" {
                    text .email_error
                    class: "text-red-500 text-xs mt-1"
                }
                class: "mb-4"
            }

            col {
                text "Password"
                class: "text-sm font-medium text-gray-700 mb-1"
                input (value: .password) {
                    oninput: () => { if .password != "" { .password_error = "" } }
                    placeholder: "Enter your password"
                    type: "password"
                    class: "w-full px-3 py-2 border rounded-lg"
                }
                if .password_error != "" {
                    text .password_error
                    class: "text-red-500 text-xs mt-1"
                }
                class: "mb-6"
            }

            button "Sign In" {
                onclick: .Submit
                class: "w-full py-2 bg-blue-500 text-white rounded-lg font-medium"
            }

            row {
                text "Don't have an account?"
                class: "text-sm text-gray-500"
                text "Sign up"
                class: "text-sm text-blue-500 font-medium"
                class: "gap-1"
            }
            class: "mt-4 justify-center"

            class: "bg-white rounded-xl shadow-lg p-8 max-w-md mx-auto w-full"
        }
    }

    on {
        .Submit -> {
            .email_error = ""
        }
    }
}
```

## How to Run

```bash
cd examples/ui/005-login
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- `input` widget with `type: "password"` for masked text entry
- `if` conditional rendering to show/hide error messages
- Form layout pattern: labels, inputs, error text, and submit button in a card
- Error state stored in model fields (`email_error`, `password_error`)
- `button` with `onclick` for form submission
