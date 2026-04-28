# 021-blog-viewer — Blog with Article List and Detail

A blog reader with an article list on the left and article detail on the right, featuring author info and navigation.

## Concepts

- **Split pane** — article list (`w-96 border-r`) + detail view (`flex-1`)
- **Article cards** — each with title, author, date, summary excerpt, and "Read More" button
- **View switching** — `SelectArticle` / `BackToList` messages toggle between list and detail views
- **Rich text display** — `leading-relaxed` for article body text
- **Metadata display** — author name, publish date in `row` with `gap-2`

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { SelectArticle, BackToList }

    model {
        var view_mode str = "list"
        var blog1_title str = "Getting Started with Rust"
        var blog1_author str = "Alice Chen"
        var blog1_date str = "Apr 15, 2026"
        var blog1_summary str = "An introduction to Rust..."
        var blog1_body str = "Rust is a systems programming language..."
        // ... 3 blog entries
    }

    view {
        col {
            row {
                text "My Blog" { class: "text-xl font-bold" }
                text .blog_count { class: "text-sm text-gray-400 ml-auto" }
            }
            row {
                col {
                    col {
                        text .blog1_title { class: "text-base font-semibold" }
                        row { text .blog1_author; text .blog1_date }
                        text .blog1_summary { class: "text-sm text-gray-600" }
                        button "Read More" { onclick: .SelectArticle }
                    }
                    // ... more articles
                    class: "w-96 border-r overflow-y-auto"
                }
                col {
                    text .blog1_title { class: "text-2xl font-bold p-6" }
                    row { text .blog1_author; text .blog1_date }
                    text .blog1_body { class: "leading-relaxed px-6 py-4" }
                    button "Back to List" { onclick: .BackToList }
                    class: "flex-1 overflow-y-auto"
                }
            }
        }
    }
}
```

## How to Run

```bash
cd examples/ui/021-blog-viewer
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Inspiration

JetNews (Jetpack Compose), Medium.
