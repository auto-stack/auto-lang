# 018-book-reader — E-Book Reader

An e-book reader with a chapter list sidebar, reading area, progress bar, and dark mode toggle.

## Concepts

- **Chapter navigation** — sidebar with chapter list, selected chapter highlighted with `bg-blue-50 text-blue-700`
- **Reading progress bar** — `progress { value: 33, max: 100 }` showing reading position
- **Theme toggle** — `ToggleDark` message toggles `isDark` boolean for dark mode
- **Pagination** — `NextChapter` / `PrevChapter` messages swap chapter content and update progress
- **Rich text display** — `leading-relaxed` typography for comfortable reading

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { ToggleDark, NextChapter, PrevChapter }

    model {
        var isDark bool = false
        var chapter_title str = "Chapter 1: The Beginning"
        var chapter_body str = "It was a dark and stormy night..."
        var progress str = "33%"
        var page_info str = "Page 1 of 3"
        var ch1 str = "Chapter 1"
        var ch2 str = "Chapter 2"
        var ch3 str = "Chapter 3"
    }

    view {
        row {
            col {
                text "My Books" { class: "text-lg font-bold p-4 border-b" }
                col {
                    text .ch1 { class: "bg-blue-50 text-blue-700 rounded-lg" }
                    text .ch2 { class: "hover:bg-gray-50 rounded-lg cursor-pointer" }
                    text .ch3 { class: "hover:bg-gray-50 rounded-lg cursor-pointer" }
                }
                class: "w-56 border-r bg-gray-50"
            }
            col {
                row {
                    button "Back" { onclick: .PrevChapter }
                    text .chapter_title { class: "text-lg font-semibold" }
                    button "Dark" { onclick: .ToggleDark }
                }
                col { text .chapter_body { class: "leading-relaxed px-8 py-6" } }
                row {
                    text .page_info { class: "text-sm text-gray-500" }
                    progress { value: 33, max: 100 }
                    text .progress { class: "text-sm text-gray-500" }
                }
            }
        }
    }

    on {
        .ToggleDark -> { .isDark = .isDark == false }
        .NextChapter -> {
            .chapter_title = "Chapter 2: The Journey"
            .chapter_body = "The road stretched endlessly..."
            .progress = "66%"
        }
    }
}
```

## How to Run

```bash
cd examples/ui/018-book-reader
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Inspiration

Kindle reader, Apple Books.
