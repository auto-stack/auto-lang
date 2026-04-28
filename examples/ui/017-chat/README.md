# 017-chat — WeChat-Style Messenger

A chat application with a contact list sidebar and a message thread with bubbles, timestamps, and an input bar.

## Concepts

- **Split pane** — contact list (left, `w-64`) + message thread (right, `flex-1`)
- **Message bubbles** — different styles for sent vs received messages using `bg-blue-500 text-white` (received) and `bg-gray-200 text-gray-800` (sent)
- **Auto-scroll message list** — `overflow-y-auto` on the message column
- **Conditional send** — `if .input != ""` guards against sending empty messages
- **Input bar** — `row` at the bottom with `input` and send `button`

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { SelectContact, SendMessage, InputChanged }

    model {
        var input str = ""
        var contact1 str = "Alice"
        var contact2 str = "Bob"
        var contact3 str = "Charlie"
        var msg1 str = "Hey, how are you?"
        var msg2 str = "I am good! Thanks for asking."
        // ... more messages
        var active_chat str = "Alice"
    }

    view {
        row {
            col {
                text "Chats" { class: "text-xl font-bold text-gray-800 p-4 border-b" }
                col {
                    text .contact1 { class: "p-3 hover:bg-gray-100 rounded-lg" }
                    text .contact2 { class: "p-3 hover:bg-gray-100 rounded-lg" }
                    text .contact3 { class: "p-3 hover:bg-gray-100 rounded-lg" }
                    class: "flex-1 overflow-y-auto"
                }
                class: "w-64 border-r border-gray-200 bg-gray-50"
            }
            col {
                row { text .active_chat { class: "text-lg font-semibold" } }
                col {
                    // Message bubbles with different styles
                    row { text .msg1 { class: "bg-blue-500 text-white rounded-2xl" } }
                    row { text .msg2 { class: "bg-gray-200 text-gray-800 rounded-2xl" } }
                    class: "flex-1 overflow-y-auto p-4 gap-2"
                }
                row {
                    input { placeholder: "Type a message...", oninput: .InputChanged }
                    button "Send" { onclick: .SendMessage }
                }
            }
        }
    }

    on {
        .SendMessage -> {
            if .input != "" { .msg5 = .input; .input = "" }
        }
    }
}
```

## How to Run

```bash
cd examples/ui/017-chat
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Inspiration

WeChat, Jetchat (Jetpack Compose).
