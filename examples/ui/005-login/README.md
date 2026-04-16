# 005-login — Login Form with Validation

A complete login form with email and password fields, real-time validation, error display, and loading state. Demonstrates form patterns and conditional rendering.

## Concepts
- **Form widgets** — `input` with `password`, `placeholder`, and `value` properties for form fields
- **Validation state** — Error messages stored in the model (`email_error`, `password_error`) and validated in the `on` handler
- **Conditional rendering** — `if` blocks show/hide error messages based on model state
- **Loading state** — `var loading bool` tracks submission state for UI feedback
- **Message variants with data** — `EmailChanged(str)` carries the new input value into the handler

## Source

```auto
widget Login {
    msg Msg {
        EmailChanged(str),
        PasswordChanged(str),
        Submit,
        TogglePasswordVisibility
    }

    model {
        var email str = ""
        var password str = ""
        var email_error str = ""
        var password_error str = ""
        var general_error str = ""
        var show_password bool = false
        var loading bool = false
    }

    view {
        col {
            // Title
            text "Sign In"
            class: "text-2xl font-bold text-gray-900 mb-2"
            text "Welcome back! Please enter your credentials."
            class: "text-sm text-gray-500 mb-8"

            // General error banner
            if .general_error != "" {
                row {
                    text .general_error
                    class: "text-red-700 text-sm"
                }
                class: "bg-red-50 border border-red-200 rounded-lg p-3 mb-4"
            }

            // Email field
            col {
                text "Email"
                class: "text-sm font-medium text-gray-700 mb-1"
                input (value: .email) {
                    oninput: .EmailChanged
                    placeholder: "you@example.com"
                    class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                }
                if .email_error != "" {
                    text .email_error
                    class: "text-red-500 text-xs mt-1"
                }
                class: "mb-4"
            }

            // Password field
            col {
                text "Password"
                class: "text-sm font-medium text-gray-700 mb-1"
                input (value: .password) {
                    oninput: .PasswordChanged
                    placeholder: "Enter your password"
                    password: .show_password
                    class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                }
                row {
                    text "Show password"
                    class: "text-sm text-gray-500"
                }
                class: "mt-1 gap-1 items-center"
                if .password_error != "" {
                    text .password_error
                    class: "text-red-500 text-xs mt-1"
                }
                class: "mb-6"
            }

            // Submit button
            button "Sign In" {
                onclick: .Submit
                class: "w-full py-2 bg-blue-500 text-white rounded-lg font-medium hover:bg-blue-600 disabled:bg-blue-300"
            }

            // Footer
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
        .EmailChanged(val) -> {
            .email = val
            .email_error = ""
            .general_error = ""
        }
        .PasswordChanged(val) -> {
            .password = val
            .password_error = ""
            .general_error = ""
        }
        .TogglePasswordVisibility -> {
            .show_password = !.show_password
        }
        .Submit -> {
            var valid = true
            .general_error = ""
            .email_error = ""
            .password_error = ""

            // Validate email
            if .email == "" {
                .email_error = "Email is required"
                valid = false
            } else if !.email.contains("@") {
                .email_error = "Please enter a valid email address"
                valid = false
            }

            // Validate password
            if .password == "" {
                .password_error = "Password is required"
                valid = false
            } else if .password.len() < 8 {
                .password_error = "Password must be at least 8 characters"
                valid = false
            }

            if valid {
                .loading = true
                // Simulated login — in a real app this would call an API
                .loading = false
            }
        }
    }
}
```

## Generated Output

### Vue 3

*(Placeholder: coming soon)*

### Jetpack Compose

*(Placeholder: coming soon)*

### ArkTS (HarmonyOS)

*(Placeholder: coming soon)*

### GPUI (Rust)

*(Placeholder: coming soon)*

### Tauri

*(Placeholder: coming soon)*

### VSCode WebView

*(Placeholder: coming soon)*

## Platform Notes

- Validation is done manually in the `on` handler rather than via a built-in validation framework. This explicit pattern works consistently across all platforms.
- `password: .show_password` maps to `type="password"` or `type="text"` on web, `visualTransformation` on Compose, and `type: InputType.Password` on ArkTS
- The `loading` variable is set but not yet wired to UI disabled states in this example — a future version could add `disabled: .loading` to the submit button
- `if` blocks in the `view` conditionally render elements based on model state, mapping to `v-if` (Vue), `if (state)` (Compose), or `if` conditional builders (GPUI)
- Error clearing happens on every input change, giving immediate feedback as the user corrects mistakes
